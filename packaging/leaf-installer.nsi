Unicode True
!include "MUI2.nsh"
!include "LogicLib.nsh"
!include "x64.nsh"
!include "WinVer.nsh"
!include "WordFunc.nsh"

!ifndef PROJECT_DIR
  !error "PROJECT_DIR muss auf das Projektverzeichnis zeigen."
!endif
!define APP_VERSION "0.1.0"
!define UNINSTALL_KEY "Software\Microsoft\Windows\CurrentVersion\Uninstall\Leaf"
!define WEBVIEW_KEY "Software\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}"

Name "Leaf"
OutFile "${PROJECT_DIR}/dist/Leaf-Setup-x64.exe"
InstallDir "$LOCALAPPDATA\Programs\Leaf"
RequestExecutionLevel user
SetCompressor /SOLID lzma
ShowInstDetails show
ShowUninstDetails show
BrandingText "Leaf · E-Book-Reader"
VIProductVersion "0.1.0.0"
VIAddVersionKey /LANG=1031 "ProductName" "Leaf"
VIAddVersionKey /LANG=1031 "FileDescription" "Leaf Windows Setup"
VIAddVersionKey /LANG=1031 "FileVersion" "${APP_VERSION}"
VIAddVersionKey /LANG=1031 "LegalCopyright" "Leaf"

!define MUI_ABORTWARNING
!define MUI_WELCOMEPAGE_TEXT "Dieses Setup installiert Leaf für dein Benutzerkonto.$\r$\n$\r$\nFalls Microsoft WebView2 Runtime fehlt, wird sie automatisch von Microsoft heruntergeladen und installiert. Dafür ist eine Internetverbindung erforderlich.$\r$\n$\r$\nBitte schließe Leaf vor der Installation."
!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_INSTFILES
!define MUI_FINISHPAGE_RUN "$INSTDIR\Leaf.exe"
!define MUI_FINISHPAGE_RUN_TEXT "Leaf starten"
!insertmacro MUI_PAGE_FINISH
!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES
!insertmacro MUI_LANGUAGE "German"

Var WebViewInstalled
Var BootstrapperExit
Var Attempts

Function .onInit
  ${IfNot} ${RunningX64}
    MessageBox MB_OK|MB_ICONSTOP "Leaf benötigt ein 64-Bit-Windows." /SD IDOK
    SetErrorLevel 1633
    Quit
  ${EndIf}
  ${IfNot} ${AtLeastWin10}
    MessageBox MB_OK|MB_ICONSTOP "Leaf benötigt Windows 10 oder neuer." /SD IDOK
    SetErrorLevel 1633
    Quit
  ${EndIf}
  SetShellVarContext current
FunctionEnd

; Microsoft documents HKLM's 32-bit view and HKCU for Evergreen detection.
; A missing/empty/zero version must not count as an installed runtime.
Function DetectWebView
  StrCpy $WebViewInstalled 0
  SetRegView 32
  StrCpy $0 ""
  ReadRegStr $0 HKLM "${WEBVIEW_KEY}" "pv"
  ${If} $0 != ""
    ${VersionCompare} "$0" "0.0.0.0" $1
    ${If} $1 == 1
      StrCpy $WebViewInstalled 1
    ${EndIf}
  ${EndIf}
  StrCpy $0 ""
  ReadRegStr $0 HKCU "${WEBVIEW_KEY}" "pv"
  ${If} $0 != ""
    ${VersionCompare} "$0" "0.0.0.0" $1
    ${If} $1 == 1
      StrCpy $WebViewInstalled 1
    ${EndIf}
  ${EndIf}
  SetRegView 64
FunctionEnd

Section "Leaf" SEC_MAIN
  SectionIn RO
  Call DetectWebView
  ${If} $WebViewInstalled == 0
    InitPluginsDir
    SetOutPath "$PLUGINSDIR"
    File "${PROJECT_DIR}/target/installer-deps/MicrosoftEdgeWebview2Setup.exe"
    DetailPrint "WebView2 fehlt. Microsoft Runtime wird heruntergeladen und installiert …"
    StrCpy $BootstrapperExit "Start fehlgeschlagen"
    ClearErrors
    ExecWait '"$PLUGINSDIR\MicrosoftEdgeWebview2Setup.exe" /silent /install' $BootstrapperExit
    ${If} ${Errors}
      Goto webview_failed
    ${EndIf}
    ; Some bootstrapper versions finish before registration is visible.
    StrCpy $Attempts 0
    ${Do}
      Call DetectWebView
      ${If} $WebViewInstalled == 1
        ${ExitDo}
      ${EndIf}
      Sleep 1000
      IntOp $Attempts $Attempts + 1
    ${LoopUntil} $Attempts >= 60
    ${If} $WebViewInstalled == 0
      Goto webview_failed
    ${EndIf}
    DetailPrint "WebView2 ist installiert."
    ${If} $BootstrapperExit == 3010
      SetRebootFlag true
    ${EndIf}
  ${Else}
    DetailPrint "WebView2 ist bereits installiert."
  ${EndIf}
  Goto install_leaf
webview_failed:
  DetailPrint "WebView2 konnte nicht installiert werden. Exit-Code: $BootstrapperExit"
  MessageBox MB_OK|MB_ICONSTOP "WebView2 konnte nicht installiert werden (Code: $BootstrapperExit).$\r$\nBitte Internetverbindung und Windows-Richtlinien prüfen und das Setup erneut starten.$\r$\nAlternativ: https://developer.microsoft.com/microsoft-edge/webview2/$\r$\n$\r$\nLeaf wurde nicht installiert." /SD IDOK
  SetErrorLevel 1603
  Abort
install_leaf:
  SetOutPath "$INSTDIR"
  SetOverwrite on
  ClearErrors
  File "${PROJECT_DIR}/dist/windows-x64/Leaf.exe"
  File /oname=README.txt "${PROJECT_DIR}/packaging/WINDOWS.txt"
  ${If} ${Errors}
    SetErrorLevel 1603
    Abort "Dateien konnten nicht installiert werden. Bitte Leaf schließen und erneut versuchen."
  ${EndIf}
  WriteUninstaller "$INSTDIR\Uninstall.exe"
  CreateDirectory "$SMPROGRAMS\Leaf"
  CreateShortcut "$SMPROGRAMS\Leaf\Leaf.lnk" "$INSTDIR\Leaf.exe"
  CreateShortcut "$SMPROGRAMS\Leaf\Leaf deinstallieren.lnk" "$INSTDIR\Uninstall.exe"
  SetRegView 64
  WriteRegStr HKCU "${UNINSTALL_KEY}" "DisplayName" "Leaf"
  WriteRegStr HKCU "${UNINSTALL_KEY}" "DisplayVersion" "${APP_VERSION}"
  WriteRegStr HKCU "${UNINSTALL_KEY}" "Publisher" "Leaf"
  WriteRegStr HKCU "${UNINSTALL_KEY}" "DisplayIcon" "$INSTDIR\Leaf.exe"
  WriteRegStr HKCU "${UNINSTALL_KEY}" "InstallLocation" "$INSTDIR"
  WriteRegStr HKCU "${UNINSTALL_KEY}" "UninstallString" '$\"$INSTDIR\Uninstall.exe$\"'
  WriteRegStr HKCU "${UNINSTALL_KEY}" "QuietUninstallString" '$\"$INSTDIR\Uninstall.exe$\" /S'
  WriteRegDWORD HKCU "${UNINSTALL_KEY}" "NoModify" 1
  WriteRegDWORD HKCU "${UNINSTALL_KEY}" "NoRepair" 1
SectionEnd

Section "Uninstall"
  SetShellVarContext current
  ; Never recursively delete user-selected folders or saved reading data.
  ClearErrors
  Delete "$INSTDIR\Leaf.exe"
  ${If} ${Errors}
    MessageBox MB_OK|MB_ICONSTOP "Bitte Leaf schließen und die Deinstallation erneut starten." /SD IDOK
    SetErrorLevel 1603
    Abort
  ${EndIf}
  Delete "$INSTDIR\README.txt"
  Delete "$INSTDIR\Uninstall.exe"
  RMDir "$INSTDIR"
  Delete "$SMPROGRAMS\Leaf\Leaf.lnk"
  Delete "$SMPROGRAMS\Leaf\Leaf deinstallieren.lnk"
  RMDir "$SMPROGRAMS\Leaf"
  SetRegView 64
  DeleteRegKey HKCU "${UNINSTALL_KEY}"
  ; WebView2 is shared with other applications and stays installed.
SectionEnd
