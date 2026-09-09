# Leaf

![Leaf](assets/leaf.svg)

Ein eigenständiger EPUB-Reader in Rust und Dioxus 0.7. Die Desktop-App öffnet ein eigenes Fenster und funktioniert offline, ohne Browser oder Entwicklungsserver. Dioxus verwendet die System-WebView zur Darstellung.

## Entwicklungsstand

Version 0.1.0: Text-EPUB-Reader mit Linux-Desktop-App, Windows-x64-EXE,
Windows-Installer, Android-APK (ARM64) und optionaler Web-Version. Ein Windows-Laufzeittest steht noch aus.
Der vollständige technische Stand und die nächsten offenen Prüfungen stehen in
[PROJEKTSTAND.md](PROJEKTSTAND.md). Hinweise für spätere Arbeitssitzungen:
[AGENTS.md](AGENTS.md).

`dist/` und `target/` sind lokale Build-Ausgaben und nicht im Repository enthalten.
Die unten genannten EXE-/ZIP-Dateien entstehen durch die jeweiligen Build-Skripte.

## Projekt holen

```sh
git clone https://github.com/l0tek/leaf.git
cd leaf
```

Benötigt wird eine aktuelle Rust-Toolchain mit Cargo und Edition-2024-Unterstützung;
gebaut wurde mit Rust 1.97.0. `Cargo.lock` ist für konsistente Abhängigkeiten eingecheckt.

## Desktop starten

```sh
cargo run
```

Ausführbare Release-App bauen und direkt starten:

```sh
cargo build --release
./target/release/leaf
```

Das CSS und das Beispielbuch sind in der ausführbaren Datei enthalten. Die Datei kann auf kompatible Linux-Systeme mit den benötigten Systembibliotheken kopiert werden; Rust und Dioxus CLI werden dort nicht zum Starten benötigt.

Unter Debian/Ubuntu werden zum Bauen GTK 3, WebKitGTK 4.1 sowie die üblichen Build-Werkzeuge benötigt:

```sh
sudo apt install build-essential pkg-config libgtk-3-dev libwebkit2gtk-4.1-dev libxdo-dev libssl-dev
```

Optionale Installation für den aktuellen Linux-Benutzer, einschließlich Anwendungsmenü-Eintrag (benötigt Python 3):

```sh
./packaging/install.sh
```

Installiert nach `~/.local/bin/leaf` und `${XDG_DATA_HOME:-~/.local/share}`. Die Installation ist kein distributionsunabhängiges AppImage. Ein macOS-Build wurde hier nicht geprüft. Für Windows steht ein Cross-Build zur Verfügung.

### Start aus einer Snap-IDE

Falls der Start aus VS Code als Snap mit einem `GLIBC_PRIVATE`-Fehler abbricht, entferne dessen geerbte GTK-/GIO-Pfade nur für diesen Aufruf:

```sh
env -u GTK_PATH -u GIO_MODULE_DIR ./target/release/leaf
```

## Windows-Installer

`dist/Leaf-Setup-x64.exe` installiert Leaf für das aktuelle Benutzerkonto nach
`%LOCALAPPDATA%\Programs\Leaf`, erstellt Startmenü-Einträge und registriert die
Deinstallation in den Windows-Einstellungen.

Das Setup prüft die von Microsoft dokumentierten WebView2-Registry-Werte für
Benutzer- und Systeminstallationen. Fehlt die Runtime, führt es den eingebetteten,
Microsoft-signierten Evergreen-Bootstrapper aus. Nur dann wird eine Internetverbindung
für den Runtime-Download benötigt. Nach der Installation prüft es die Registry erneut;
ohne erkannte Runtime bricht es vor dem Kopieren von Leaf ab. Die C-Laufzeit und der
WebView2-Loader sind bereits in Leaf eingebunden.

Deinstallation entfernt Leaf, erhält jedoch Lesedaten und die gemeinsam genutzte
WebView2 Runtime. Vor Updates oder Deinstallation Leaf schließen.
Unbeaufsichtigte Installation: `Leaf-Setup-x64.exe /S`.

Installer unter Linux erneut bauen:

```sh
# Build-Abhängigkeiten: nsis, curl, openssl, osslsigncode, python3
./packaging/build-installer.sh
```

Das Skript benötigt die zuvor gebaute `dist/windows-x64/Leaf.exe`, lädt den aktuellen
Bootstrapper über Microsoft HTTPS und prüft dessen primäre Authenticode-Signatur
inklusive Zeitstempel gegen die Microsoft-Root-Zertifikate. Es installiert keine
Zertifikate in den System-Zertifikatsspeicher. Ausgabe: Setup-EXE und SHA-256-Prüfsumme.

Validiert: NSIS-Build ohne Warnungen, Archivintegrität und eingebettete Dateien.
Ein echter Installationstest unter Windows steht noch aus (Runtime vorhanden/fehlend,
offline, Update, Deinstallation). Der Leaf-Installer selbst ist nicht signiert.

## Windows-EXE (x64)

Ausgabe: `dist/windows-x64/Leaf.exe` und `dist/Leaf-windows-x64.zip`.
Die EXE enthält Oberfläche und Beispielbuch und öffnet kein Konsolenfenster.
Voraussetzung auf Windows ist die [Microsoft WebView2 Runtime](https://learn.microsoft.com/en-us/microsoft-edge/webview2/concepts/distribution).
Der Windows-Build wird unter Linux erstellt; ein Laufzeittest unter Windows steht noch aus.

Erneut unter Linux bauen (Python 3, clang-cl und lld-link im PATH):

```sh
rustup target add x86_64-pc-windows-msvc
cargo install cargo-xwin --locked
./packaging/build-windows.sh
```

Das Skript bindet die C-Laufzeit statisch ein und erstellt EXE, ZIP und SHA-256-Prüfsumme.
Für das eingebettete Windows-Icon wird zusätzlich `llvm-rc` benötigt; bei gesetztem
`ANDROID_NDK_HOME` verwendet das Skript automatisch dessen LLVM-Ressourcencompiler.
[cargo-xwin](https://github.com/rust-cross/cargo-xwin) lädt dafür Microsoft CRT und Windows SDK in seinen lokalen Cache.

## Android-APK (ARM64)

Die lokale Datei `dist/Leaf-android-arm64.apk` ist für Android ab Version 7
(API 24) auf ARM64-Geräten vorgesehen. Zum Installieren auf das Gerät kopieren
und öffnen oder `adb install -r dist/Leaf-android-arm64.apk` verwenden.

Erneut bauen: JDK 17, Android-SDK (Plattform 36), NDK und Dioxus CLI 0.7.10
installieren, dann die Pfade an die eigene Installation anpassen:

```sh
export JAVA_HOME=/pfad/zum/jdk17
export ANDROID_HOME="$HOME/Android/Sdk"
export ANDROID_NDK_HOME="$ANDROID_HOME/ndk/27.2.12479018"
rustup target add aarch64-linux-android
./packaging/build-android.sh
```

Die APK enthält optimierten Rust-Code, ist aber mit dem lokalen Android-Debugschlüssel
signiert und als Testpaket gedacht. Für eine Veröffentlichung ist ein dauerhafter
Release-Schlüssel erforderlich. Die Prüfsumme steht in
`dist/Leaf-android-arm64.apk.sha256`. Auf einem Samsung Galaxy Tab A7 Lite (SM-T220, Android 14) wurden Installation,
EPUB-Import über die Android-Dateiauswahl und Wiederherstellung der Lesestelle
nach vollständigem App-Neustart geprüft. Android speichert `reading.json` im privaten Dateienverzeichnis der
App; eine Deinstallation entfernt diese Daten.

Die Einrichtung folgt der [Dioxus-Mobile-Dokumentation](https://dioxuslabs.com/learn/0.7/guides/platforms/mobile/).
Das Compile-SDK ist explizit auf 36 gesetzt, damit es zu den AndroidX-Abhängigkeiten passt.
Android, der Linux-Anwendungsstarter und die Windows-EXE verwenden dasselbe
Buch-Icon in jeweils passenden Bildgrößen.

## Funktionen und Speicherung

- DRM-freie EPUB-Dateien lokal importieren (bis 30 MB)
- Inhaltsverzeichnis aus EPUB-NAV/NCX (mit Spine-Fallback bei älteren Beständen)
- Paginierte Anzeige über das lokal eingebettete epub.js, einschließlich Buchbildern,
  Buchstyles, internen Links, Seiten- und Kapitelnavigation
- Schriftwahl (Serif, Sans oder Mono), letzte EPUB-CFI-Position, Kapitel, aktuelle
  Seite und Einstellungen automatisch
  lokal speichern
- Responsive Leseansicht und deutsches Beispielbuch
- Bereinigung importierter HTML-Inhalte; keine externen Buchressourcen

Die Desktop-App speichert `reading.json` im lokalen Anwendungsdatenverzeichnis (Linux: `${XDG_DATA_HOME:-~/.local/share}/leaf/reading.json`). Schreiben erfolgt über eine temporäre Datei mit anschließendem Ersetzen. Bei einem Speicherfehler erscheint ein Hinweis in der Seitenleiste. Es werden keine Bücher hochgeladen. epub.js paginiert importierte Bücher in einer isolierten Buchansicht; Seitentasten, interne Links und Inhaltsverzeichnis verwenden die EPUB-eigenen Ziele. Die aktuelle Stelle wird als EPUB-CFI gespeichert und beim erneuten Öffnen wiederhergestellt. Das eingebaute Beispielbuch und ältere gespeicherte Importe ohne Originalarchiv verwenden weiterhin die bisherige Textansicht.

PDF und DRM werden nicht unterstützt. Die Web-Version unterliegt weiterhin dem
begrenzten Local Storage des Browsers; das Original-EPUB wird für die Offline-Anzeige
zusammen mit dem Lesestand gespeichert.

## Drittanbieter-Code

Leaf bettet [epub.js 0.3.93](https://github.com/futurepress/epub.js) und
[JSZip 3.10.1](https://github.com/Stuk/jszip) lokal ein. Dadurch benötigt die
Leseansicht keine Internetverbindung. Die Lizenztexte liegen unter `third-party/`.

## Optionale Web-Version

```sh
rustup target add wasm32-unknown-unknown
cargo install dioxus-cli --version '~0.7' --locked
dx serve --platform web
```

Im Web werden die Daten im begrenzten Local Storage des Browsers gespeichert. Bei großen Büchern kann die Speicherung fehlschlagen; Lesen bleibt in der Sitzung möglich.

## Projektstruktur

- `src/main.rs`: Oberfläche, Desktop-Start und Speicherung.
- `src/book.rs`: EPUB-Import, Bereinigung und Beispielbuch.
- `assets/`: CSS und App-Symbol.
- `packaging/`: Linux-Installation, Windows-Cross-Build und NSIS-Setup.
- `PROJEKTSTAND.md`: technische Übergabe und offene Aufgaben.

## Prüfen

```sh
cargo test
cargo clippy --all-targets -- -D warnings
cargo check --no-default-features --features web --target wasm32-unknown-unknown
```

[Dioxus Desktop-Dokumentation](https://dioxuslabs.com/learn/0.7/guides/platforms/desktop/)
