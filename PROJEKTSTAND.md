# Projektstand — Leaf

Stand: 8. September 2026 · Version 0.1.0
Repository: https://github.com/l0tek/leaf

Diese Datei ist der Einstieg für die nächste Arbeitssitzung. Sie dokumentiert den
vorhandenen Code und die tatsächlich durchgeführten Prüfungen. Nach wesentlichen
Änderungen aktualisieren; neue Anforderungen aus der aktuellen Sitzung haben Vorrang.

## Ziel und bisherige Entscheidungen

Leaf ist ein E-Book-Reader in Rust und Dioxus 0.7.10 (Cargo.lock). Zunächst als
Web-App erstellt, danach auf Wunsch um eine eigenständige Desktop-App, eine
Windows-x64-EXE und einen Windows-Installer mit bedarfsgerechter Installation von
WebView2 erweitert. Desktop ist das Standard-Cargo-Feature; Web bleibt optional.
Die Oberfläche und das Beispielbuch sind deutsch. Bücher werden lokal verarbeitet.

Aktueller Auftrag: Projektstatus prüfen, aktuelle Lesestelle automatisch speichern
und eine Android-APK erstellen. Zu Beginn war der Git-Arbeitsstand sauber.
Kein Commit oder Push für diese Sitzung beauftragt.

## Implementiert

- EPUB-Import: ZIP-Container, OPF-Metadaten und Spine-Reihenfolge.
- HTML-Bereinigung mit Ammonia; Skripte, Bilder, Links und Buch-CSS entfernt.
- Kapitelliste, Vor/Zurück, Schriftgröße 16–28 px, Hell-/Dunkelmodus.
- Responsive Oberfläche und eingebautes Beispielbuch.
- Ein zuletzt geöffnetes Buch samt Kapitel, Scrollposition und Einstellungen gespeichert.
  Die Scrollposition wird bei Scrollereignissen gespeichert und nach Einbinden der Leseansicht
  bei fertigem Layout wiederhergestellt; Kapitelwechsel und Buchimport beginnen oben. Alte Zustände
  ohne Scrollposition bleiben lesbar (Serde-Default 0).
- Native Speicherung als JSON über temporäre Datei und anschließendes Ersetzen;
  im Browser über Local Storage. Speicherfehler werden angezeigt.
- Desktop-Fenster 1120 × 800, Mindestgröße 420 × 520.
- Linux-Installationsskript für Benutzerverzeichnis, Symbol und Startmenü.
- Windows-MSVC-Cross-Build, statische C-Laufzeit und WebView2-Loader,
  GUI-Subsystem ohne zusätzliches Konsolenfenster im Release-Modus.
- NSIS-Installer: Installation pro Benutzer, Startmenü, Windows-Deinstallation.
  Prüft WebView2 über HKLM (32-Bit-Registryansicht) und HKCU; Version muss größer
  als 0.0.0.0 sein. Bei Bedarf führt er den eingebetteten Microsoft-Bootstrapper
  mit `/silent /install` aus, wartet auf die Registrierung und bricht bei fehlender
  Runtime vor dem Kopieren der App ab. Der Runtime-Download benötigt Internet.
- Installer-Deinstallation erhält Lesedaten und die gemeinsam genutzte Runtime.

## Orientierung im Code

| Datei | Aufgabe |
| --- | --- |
| `src/main.rs` | Dioxus-Oberfläche, Fensterkonfiguration, Zustandsmodell, Speicherung und Speichertest |
| `src/book.rs` | EPUB-Parser, Bereinigung, Beispielbuch und Parsertests |
| `assets/main.css` | Oberfläche, Themes und responsive Darstellung |
| `assets/leaf.svg` | Linux-App-Symbol |
| `Cargo.toml`, `Cargo.lock` | Abhängigkeiten und Desktop-/Web-Features |
| `Dioxus.toml` | Plattform- und Bundle-Metadaten |
| `packaging/install.sh`, `packaging/leaf.desktop` | Linux-Benutzerinstallation |
| `packaging/build-android.sh` | Android-ARM64-Test-APK und Prüfsumme |
| `packaging/build-windows.sh` | Windows-EXE, ZIP und Prüfsumme |
| `packaging/leaf-installer.nsi` | Windows-Installations-/Deinstallationslogik |
| `packaging/build-installer.sh` | Download, Signaturprüfung, NSIS-Build und Prüfsumme |
| `packaging/WINDOWS.txt` | Anleitung im Windows-Paket |

## Build und Prüfungen

Allgemeine Einrichtung und Befehle stehen in [README.md](README.md).

Bereits erfolgreich durchgeführt:

- `cargo test`: drei Tests (ungültiges EPUB, Spine-Reihenfolge/HTML-Bereinigung,
  persistenter Zustand einschließlich Ersetzen einer vorhandenen Datei).
- `cargo clippy --all-targets -- -D warnings` für Desktop.
- `cargo fmt --check`.
- WebAssembly-Check mit `--no-default-features --features web` und Dioxus-Web-Build.
- Linux-Release-Build; kurzer echter Fensterstart und erzeugte Zustandsdatei geprüft.
- Windows-x64-Release-Build mit cargo-xwin 0.23.1. PE-Header bestätigt GUI/x64;
  Importtabelle enthält Windows-System-DLLs, keine separate C-Runtime-/Loader-DLL.
  Linker meldete fehlende Microsoft-PDB-Debugsymbole (LNK4099), Build erfolgreich.
- NSIS 3.10 mit `-WX`: Installer kompiliert ohne Warnungen.
- Microsoft-Bootstrapper: primäre Authenticode-Signatur inklusive Zeitstempel und
  Zertifikatskette erfolgreich mit osslsigncode geprüft. Die zusätzliche interne
  EdgeBuild-Signatur wird nicht als Vertrauensanker verwendet.
- Setup-Archiv mit 7-Zip geprüft; eingebettete App und Bootstrapper bytegenau mit
  Eingabedateien verglichen. ZIP-Inhalt und SHA-256-Prüfsummen geprüft.

Nicht durchgeführt:

- App- oder Installer-Laufzeittest auf echtem Windows; Wine ist nicht installiert.
- Vollständiger interaktiver Browser-/Desktop-Test des Imports mit realen Büchern.
- macOS-Build, Code-Signing oder veröffentlichte GitHub-Releases.

## Vorhandene lokale Ausgaben

Diese Dateien sind generiert und absichtlich nicht in Git. Auf einem frischen Clone
über die Build-Skripte neu erstellen; sie sind nicht automatisch GitHub-Downloads.

- `target/release/leaf` — Linux-App.
- `dist/windows-x64/Leaf.exe` — Windows-App.
- `dist/windows-x64/README.txt`, `SHA256SUMS.txt`.
- `dist/Leaf-windows-x64.zip` — portable Windows-Ausgabe.
- `dist/Leaf-Setup-x64.exe`, `dist/Leaf-Setup-x64.exe.sha256` — Windows-Setup.

Lokale Werkzeugdetails dieser Sitzung (keine portable Voraussetzung):

- Rust 1.97.0; Dioxus CLI 0.7.10; Linux GTK 3 und WebKitGTK 4.1 vorhanden.
- Windows-Target `x86_64-pc-windows-msvc` mit rustup installiert.
- cargo-xwin in `/tmp/leaf-win-tools/bin`; `clang-cl` dort ist ein Symlink auf
  Clang aus Android NDK 27.2.12479018. NDK-`bin` enthält auch `lld-link`.
- NSIS, osslsigncode und 7-Zip ohne Root-Rechte nach `/tmp/leaf-nsis/root`
  entpackt. NSIS benötigte `NSISDIR=/tmp/leaf-nsis/root/usr/share/nsis`.
- `/tmp`-Werkzeuge sind vergänglich. Für einen neuen Rechner die in der README
  genannten regulären Werkzeuge installieren und über PATH bereitstellen.
- Microsoft SDK/CRT liegt im cargo-xwin-Benutzercache. Erstmaliges Entpacken
  dauerte mehrere Minuten; laufende CPU-/Dateiaktivität zeigte Fortschritt.
- Unter Snap-VS-Code verursachten geerbte `GTK_PATH`/`GIO_MODULE_DIR` einen
  GLIBC-Konflikt. Start mit `env -u GTK_PATH -u GIO_MODULE_DIR …` funktionierte.
- Snap kann auch `XDG_DATA_HOME` umleiten. Speicherort daher aus der Umgebung
  ableiten, nicht ausschließlich `~/.local/share` voraussetzen.

## Bekannte Grenzen und sinnvolle nächste Schritte

1. Zuerst Windows testen: frisches Benutzerkonto ohne WebView2, vorhandene Runtime,
   fehlendes Netzwerk, blockierte Installation, `/S`, Update bei geschlossener und
   laufender App, Deinstallation und Erhalt der Lesedaten.
2. EPUB-Kompatibilität mit realen EPUB-2-/EPUB-3-Dateien prüfen. Der Parser hängt
   Manifest-Hrefs derzeit direkt an den OPF-Basispfad; URI-Decoding, Fragmente und
   relative `..`-Segmente sind noch nicht umfassend behandelt. Navigation kommt
   aus Kapitelüberschriften, nicht aus EPUB-NAV/NCX.
3. Textorientierter MVP: keine Bilder, PDF, DRM, eingebetteten Buchstile,
   internen Links, Suche, Lesezeichen oder Mehrbuch-Bibliothek.
4. Die Scrollposition wird in Pixeln gespeichert; Größen-/Schriftänderungen können
   den sichtbaren Text verschieben. Bei jedem Speichern wird derzeit das gesamte
   Buch serialisiert; bei großen Büchern ist die Scrollleistung noch zu prüfen.
   Web-Speicher ist begrenzt. Parser-Limits: 30 MiB Eingabedatei, 8 MiB pro Textelement,
   40 MiB gesamte Kapitelquellen. Die Dateiauswahl liest zunächst die Datei,
   bevor der Parser die Eingabegröße prüft.
5. App und Installer sind nicht signiert. Kein Release-Publishing/CI eingerichtet.
   Bei Releases Versionen in Cargo.toml und NSIS sowie Paketdokumentation abstimmen.

## Umsetzung und Prüfungen am 8. September 2026

- Android-Feature `mobile`, Paket `de.leaf.reader`, ARM64, minSdk 24,
  targetSdk 34, compileSdk 36. Speicherung über JNI-`getFilesDir()` im privaten
  App-Verzeichnis; Speicherpfad wird einmal ermittelt.
- `packaging/build-android.sh` erstellt `dist/Leaf-android-arm64.apk` und
  SHA-256-Datei. Optimierter Rust-Release-Code, Android-Debugsignatur; keine
  Store-Veröffentlichung oder produktive Release-Signatur.
- Die Wiederherstellung wartet auf das Mounten und Layout-Frames. Ein
  ResizeObserver berücksichtigt nachträgliche Größenänderungen beim Android-Start
  bis zur ersten Benutzerinteraktion. Automatische Scrollereignisse während der
  anfänglichen Wiederherstellung überschreiben den Lesestand nicht.
- Dioxus 0.7.10 fängt Datei-Inputs für seinen Desktop-Dialog ab. Android verwendet
  deshalb einen eigenen, außerhalb des Dioxus-Baums angelegten WebView-Dateiinput
  (`assets/android-picker.js`). Die Eval-Verbindung bleibt während der Auswahl
  offen; Übertragung in bestätigten 64-KiB-Blöcken, Größenprüfung vor dem Lesen.
- Tatsächlich bestanden: vier Rust-Tests (einschließlich alter JSON-Zustände ohne
  Scrollposition und atomarem Ersetzen mit Scrollposition), Desktop-Clippy mit
  `-D warnings`, Formatprüfung, Shell-Syntaxprüfung und Web-Release-Build.
  Der Web-Build meldet eine wasm-opt-DWARF-Warnung, wird aber erfolgreich beendet.
- Chromium/Playwright gegen die gebaute Web-Version: 1234 px über Neuladen
  wiederhergestellt, Kapitelwechsel beginnt bei 0, erneuter Klick auf das aktive
  Kapitel erhält die Lesestelle.
- Echter Android-Test auf Samsung Galaxy Tab A7 Lite SM-T220, Android 14:
  APK per ADB installiert/aktualisiert, App gestartet; Test-EPUB über den
  Android-Systemdateidialog aus Downloads importiert, Titel und zwei Kapitel
  in Anzeige/Zustandsdatei geprüft. Scrollposition per WebView-DOM und
  `reading.json` verglichen und nach `am force-stop` plus Kaltstart geprüft.
- Testdatei: `/sdcard/Download/Leaf-Test.epub`. Anschließend hat der Benutzer
  selbst die Dateiauswahl geöffnet; diese wurde offen gelassen. Keine vorhandenen
  Benutzerbücher gelöscht. Ein zusätzlicher Wischtest während dieser Auswahl
  konnte die Leseansicht nicht bedienen; die zuvor bestandene Kaltstartprüfung
  erfolgte mit dem Beispielbuch. Abbrechen der Dateiauswahl noch nicht separat
  auf dem Gerät geprüft.
- Noch offen: große/reale EPUB-Dateien, weitere Android-Geräte, Rotation/
  geänderte Schriftgröße und Release-Signierung. Die ältere Windows-EXE und
  der Windows-Installer wurden in dieser Sitzung nicht neu gebaut.


## Wiedereinstieg

1. `AGENTS.md`, diese Datei und README lesen; Git-Status und aktuellen Auftrag prüfen.
2. Vorhandene Änderungen erhalten; generierte Artefakte nicht mit Quellen verwechseln.
3. Nur für die konkrete Änderung relevante Prüfungen ausführen. Windows-Verhalten
   nicht als getestet darstellen, solange kein Windows-Laufzeittest vorliegt.
4. Nach Abschluss neue Entscheidungen, offene Punkte und Testergebnisse hier
   nachtragen. Commit/Push nur entsprechend dem jeweils autorisierten Auftrag.
