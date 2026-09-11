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

Aktueller Auftrag: Projektstatus ermitteln und das Navigations- und Anzeigeproblem
mit epub.js lösen. Zu Beginn war der Arbeitsbaum sauber. Kein Commit oder Push für
diese Sitzung beauftragt.

## Implementiert

- EPUB-Import: ZIP-Container, OPF-Metadaten und Spine-Reihenfolge; neue Importe
  behalten zusätzlich das Originalarchiv für epub.js.
- Neue Importe werden mit Buch-CSS, Bildern, internen Links und EPUB-Navigation
  isoliert durch epub.js dargestellt. Die Ammonia-bereinigte Textansicht bleibt als
  Kompatibilitätsweg für das Beispielbuch und alte Speicherstände erhalten.
- Kapitelliste, Vor/Zurück, Schriftgröße 16–28 px, Hell-/Dunkelmodus.
- Responsive Oberfläche und eingebautes Beispielbuch.
- Buchübersicht als Listenansicht mit lokal erzeugten Coverkarten aus Titel und
  Autor; EPUB-Coverbilder werden weiterhin nicht importiert.
- Ein zuletzt geöffnetes Buch samt Kapitel, Seite und Einstellungen gespeichert.
  Seitentasten und vertikale Wischgesten springen jeweils um eine Viewporthöhe;
  freies Scrollen ist deaktiviert. Die Seitennummer wird bei Scrollereignissen
  gespeichert und nach Einbinden der Leseansicht bei fertigem Layout wiederhergestellt.
  Kapitelwechsel und Buchimport beginnen auf Seite 1. Alte Zustände ohne Seitennummer
  bleiben lesbar (Serde-Default 0).
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
2. epub.js-Rendition mit realen EPUB-2-/EPUB-3-Dateien auf Desktop, Android und
   Windows interaktiv prüfen: NAV/NCX, verschachtelte Einträge, Bilder, Buch-CSS,
   interne Links, sehr große Kapitel, Drehung und geänderte Schriftgröße.
3. PDF, DRM, Suche und Lesezeichen werden nicht unterstützt.
4. Bei jedem Speichern wird derzeit das gesamte Buch einschließlich des Base64-
   kodierten Originalarchivs serialisiert; bei großen Büchern ist die Leistung zu prüfen.
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

## Umsetzung und Prüfungen am 9. September 2026

- Frische Android-ARM64-Test-APK mit der epub.js-Integration über
  `packaging/build-android.sh` erfolgreich erstellt: 15 MiB,
  SHA-256 `c8949743927df97663bf3e367f75926140d911ea128273868501271fdf6abeb8`.
  Die Prüfsummendatei wurde erfolgreich verifiziert. Das per USB erkannte Tablet
  `R9JRB01BT5M` meldete jedoch wiederholt `unauthorized`; die Installation wurde
  deshalb noch nicht ausgeführt und wartet auf Bestätigung des RSA-Dialogs am Gerät.
- Navigations- und Anzeigeproblem grundlegend auf epub.js 0.3.93 umgestellt.
  Neue EPUB-Importe speichern das Originalarchiv und werden offline durch eine echte,
  paginierte epub.js-Rendition angezeigt. Seitenwechsel verwenden `next`/`prev`,
  Kapitelaktionen EPUB-Hrefs und der gespeicherte Lesestand eine EPUB-CFI statt einer
  fragilen Pixelposition. epub.js liefert EPUB-NAV/NCX und die aktuelle Spine-Position
  an die Dioxus-Oberfläche zurück. Bilder, Buch-CSS und interne Links bleiben erhalten.
  Das Beispielbuch und ältere gespeicherte Bücher ohne Originalarchiv verwenden als
  kompatiblen Rückfall weiterhin die bisherige bereinigte Textansicht.
- epub.js und JSZip sind fest versioniert, lokal eingebettet und benötigen zur Laufzeit
  kein CDN. Original-Lizenztexte liegen unter `third-party/`; Bundle-Prüfsummen:
  epub.js `06eae15745107b4aa508c95538275251f69bfb9f1175621fc458d9f42ed082d4`,
  JSZip `acc7e41455a80765b5fd9c7ee1b8078a6d160bbbca455aeae854de65c947d59e`.
- Bestanden: `cargo fmt --check`, `cargo test` (7 Tests),
  `cargo clippy --all-targets -- -D warnings`, WebAssembly-Check, JavaScript-Syntaxcheck
  aller drei Bundles und Dioxus-Web-Release-Build. Der bekannte wasm-opt-DWARF-Fehler
  erschien erneut; Dioxus erstellte die Client-Ausgabe erfolgreich. Noch nicht erfolgt:
  interaktiver Laufzeittest der neuen Rendition mit realen EPUBs auf Desktop/Android/
  Windows; neue Plattformartefakte wurden nicht gebaut.

- Projektüberblick, `README.md`, Git-Status und vorhandene Build-Skripte geprüft;
  Arbeitsbaum war und ist bis auf diese Übergabedatei sauber.
- Frischer Linux-Desktop-Release mit `cargo build --locked --release` erstellt:
  `target/release/leaf` (x86-64-ELF, 9,3 MiB).
- Frische Android-ARM64-Test-APK mit `packaging/build-android.sh` erstellt:
  `dist/Leaf-android-arm64.apk` (23 MiB) einschließlich SHA-256-Datei.
- Frische Windows-x64-Ausgabe mit `packaging/build-windows.sh` erstellt:
  `dist/windows-x64/Leaf.exe` (PE32+-GUI, 6,4 MiB), portablem ZIP und SHA-256-Datei.
  Dafür wurde lokal `cargo-xwin` 0.23.1 installiert und der Clang/Lld-Linker des
  vorhandenen Android-NDK verwendet. Die bekannten LNK4099-Hinweise zu fehlenden
  Microsoft-PDB-Dateien traten auf, der Build endete dennoch erfolgreich.
- Verifiziert: beide SHA-256-Prüfsummen, ZIP-Archivtest sowie Dateiformate. Kein
  Laufzeittest auf Android, Linux oder Windows in dieser Sitzung; insbesondere
  ersetzt der Cross-Build keinen Windows-Laufzeittest.
- `dist/Leaf-android-arm64.apk` mit `adb install -r` auf dem per USB erkannten
  Samsung Galaxy Tab A7 Lite SM-T220 (`R9JRB01BT5M`) installiert/aktualisiert;
  die Paketabfrage bestätigt `de.leaf.reader`, VersionCode 1, minSdk 24 und
  targetSdk 34. Bestehende App-Daten bleiben bei dieser Update-Installation erhalten.
- Frischen Windows-Installer mit `packaging/build-installer.sh` erstellt:
  `dist/Leaf-Setup-x64.exe` (NSIS-PE, 3,4 MiB) und SHA-256-Datei. Wegen fehlender
  Systempakete wurden NSIS 3.10, nsis-common und osslsigncode nur temporär unter
  `/tmp/leaf-installer-tools.*` entpackt. Der Build prüfte den eingebetteten
  Microsoft-WebView2-Bootstrapper erfolgreich; Installer-Prüfsumme bestätigt.
- Die Buchübersicht auf eine Listenansicht umgestellt. Jede Zeile hat eine eigene,
  aus Titel und Autor erzeugte Coverkarte, Metadaten, Lesefortschritt und den
  Öffnen-Hinweis. Auf kleinen Bildschirmen werden Cover und Typografie verdichtet.
- Bestanden: `cargo fmt --check`, `cargo test` (7 Tests),
  `cargo clippy --all-targets -- -D warnings` und Web-Release-Build. Der
  Web-Build meldet weiterhin die bekannte wasm-opt-DWARF-Warnung, beendet den
  Client-Build aber erfolgreich.
- APK nach der Listenansichts-Änderung frisch mit `packaging/build-android.sh`
  erstellt und per `adb install -r` auf dem Samsung Galaxy Tab A7 Lite SM-T220
  installiert/aktualisiert. `de.leaf.reader` ist als VersionCode 1, minSdk 24
  und targetSdk 34 vorhanden; die APK-Prüfsumme ist bestätigt.
- Seitenweise Leseansicht umgesetzt: Die dauerhaft sichtbaren Seitentasten springen
  um eine Viewporthöhe; vertikales Wischen/Trackpad-Scrollen bleibt verfügbar.
  Kapitelaktionen bleiben separat. Die berechnete Seitennummer wird gespeichert und
  nach Layoutaufbau wiederhergestellt; alte Zustände ohne Feld starten auf Seite 1.
- Bestanden: `cargo fmt --check`, `cargo test` (7 Tests einschließlich Migration
  ohne Seitennummer), `cargo clippy --all-targets -- -D warnings`, Web-Check und
  Linux-Release-Build. Der Web-Release-Build meldet weiterhin die bekannte
  wasm-opt-DWARF-Warnung, erstellt die Client-Ausgabe jedoch erfolgreich.
- Frische Seitenansichts-APK mit `packaging/build-android.sh` erstellt und per
  `adb install -r` auf dem Samsung Galaxy Tab A7 Lite SM-T220 installiert.
  Paketpfad, VersionCode 1, minSdk 24 und targetSdk 34 sind per ADB bestätigt;
  die APK-Prüfsumme ist gültig. Die Update-Installation erhält vorhandene App-Daten.
- Frische Windows-x64-EXE, portables ZIP und NSIS-Installer erstellt. Der Installer
  nutzt die neue EXE und enthält den bei jedem Build erneut gegen Microsoft-Roots
  geprüften WebView2-Bootstrapper. APK-, EXE- und Installer-Prüfsummen sowie der
  ZIP-Archivtest sind erfolgreich. Der Cross-Linker meldete die bekannten
  LNK4099-Hinweise zu fehlenden Microsoft-PDB-Dateien; ein Windows-Laufzeittest
  wurde weiterhin nicht durchgeführt.
- Android-Korrektur: Die erste horizontale CSS-Spaltenvariante blendete bei Android
  den Text und die Navigation aus; eine feste Spaltenfläche überschritt zusätzlich
  das Tile-Speicherlimit des WebView. Ersetzt durch vertikalen, nativen Lesescroll
  und dauerhaft sichtbare Seitentasten. Auf dem SM-T220 geprüft: nach dem WebView-
  Start erscheint Seite 1 von 42, `Seite →` wechselt zu Seite 2 von 42, ein
  anschließender Wisch zu Seite 3 von 42.
- Die korrigierte APK installiert sowie Windows-EXE, ZIP und Installer erneut
  erstellt. `cargo fmt --check`, 7 Rust-Tests und Clippy mit `-D warnings` sind
  bestanden; APK-, EXE- und Installer-Prüfsummen sowie ZIP-Archivtest sind gültig.
- Freies Scrollen abgeschaltet: Die Lesefläche verbirgt ihren Überlauf und fängt
  vertikale Touch-Gesten ab. Ein Wisch ab 48 px ruft einen einzelnen Seitensprung
  aus; die Seitentasten verwenden denselben Sprung. Auf dem SM-T220 geprüft:
  Seite 1 von 42 per Wisch zu Seite 2 von 42, danach per Taste zu Seite 3 von 42.
  APK installiert sowie Windows-EXE, ZIP und Installer erneut erstellt; alle
  zugehörigen Prüfsummen und der ZIP-Archivtest sind gültig.
- Touch-Steuerung gegen verbleibendes WebView-Scrollen verschärft: Touchstart und
  -bewegung werden in der Capture-Phase abgefangen, nativer Überlauf bleibt
  verborgen und die Seitensprünge erfolgen ohne Smooth-Scrollanimation. Auf dem
  SM-T220 nach APK-Update per Wisch direkt zu Seite 4 von 42 geprüft. Windows-EXE,
  ZIP und Installer erneut synchronisiert; Prüfsummen und ZIP-Archivtest gültig.
- Lesebereich gegen Kopf- und Fußleiste geschützt: Beide Leisten sind opak; ein
  36-px-Sicherheitsbereich blendet angeschnittenen Text aus. Die Seitenschritte
  überlappen um diesen Bereich, damit kein Text zwischen Seiten verloren geht.
  Auf dem SM-T220 nach APK-Update geprüft: vollständige Leisten, Text nur im
  geschützten Innenbereich und Seitennavigation weiterhin sichtbar.
- Seitenwisch auf horizontal geändert: Links/rechts-Gesten ab 48 px wechseln eine
  Seite; vertikale Bewegungen lösen keinen Seitenwechsel aus. APK gebaut und auf
  dem SM-T220 installiert. `cargo fmt --check`, 7 Tests und Clippy bestanden.
- Der vom Benutzer auf dem Tablet gespeicherte Screenshot wurde ausschließlich
  gelesen. Er zeigte, dass die unterste Textzeile noch in den Navigationsbereich
  hineinragte. Die Seitengrenzen werden deshalb nun aus den tatsächlichen
  Textzeilen des gerenderten Kapitels berechnet: Jede neue Seite beginnt mit der
  ersten zuvor nicht vollständig sichtbaren Zeile. Die Kopf- und
  Navigationsschatten (je 36 px) bleiben dabei ausgespart. Horizontale Gesten
  lösen dieselbe Seitennavigation wie die Tasten aus; vertikale Gesten bleiben
  ohne Wirkung. Die daraus gebaute ARM64-APK wurde per `adb install -r` auf dem
  SM-T220 installiert. Bestanden: `cargo fmt --check`, `cargo test` (7 Tests)
  und `cargo clippy --all-targets -- -D warnings`. Die konkrete Darstellung nach
  diesem Update steht noch für die Sichtprüfung auf dem Gerät aus; beim
  abschließenden Geräte-Screenshot war das Tablet gesperrt.
- Der anschließende entsperrte Test zeigte einen Zählerfehler und unvollständiges
  Zeichnen des Android-WebView bei der experimentellen Zeilenvermessung. Diese
  Variante wurde verworfen. Die stabile Seitenermittlung per Viewporthöhe ist
  wiederhergestellt; Kopf- und Seitennavigation überdecken jetzt jeweils 56 px
  (statt 36 px), also mehr als eine Textzeile. Dadurch bleibt am Rand kein
  angeschnittener Text sichtbar. Die korrigierte ARM64-APK wurde erfolgreich mit
  `adb install -r` auf dem SM-T220 aktualisiert. `cargo fmt --check`, `cargo test`
  (7 Tests) und `cargo clippy --all-targets -- -D warnings` bestanden.
- Nach einem weiteren Screenshot wurde die eigentliche Überlagerung bestätigt:
  Die CSS-Schatten der Kopf- und Seitennavigationsleiste lagen über dem Text.
  Beide Schatten wurden entfernt; der Lesebereich ist jetzt ausschließlich der
  Flex-Bereich zwischen den Leisten und wird von ihnen nicht überdeckt. Die
  korrigierte APK wurde erneut erfolgreich per `adb install -r` auf dem SM-T220
  installiert. Formatprüfung, 7 Tests und Clippy mit `-D warnings` bestanden.
- Der danach vom Benutzer gespeicherte Screenshot zeigte weiterhin eine halbe
  Zeile direkt vor den Seitentasten. Seitensprünge rasten deshalb nach dem
  Sprung mit einer lokalen Caret-/Zeilenprüfung an beiden sichtbaren Rändern
  ein; es wird dabei nicht mehr das ganze Kapitel vermessen. APK aktualisiert
  und auf dem SM-T220 nach Neustart geprüft: Seite 2 von 49 zeigt oben und unten
  ausschließlich vollständige Textzeilen. Formatprüfung, 7 Tests und Clippy mit
  `-D warnings` bestanden.
- Frische Windows-x64-EXE, portables ZIP und NSIS-Installer aus dem aktuellen
  Stand gebaut. Der Installer liegt unter `dist/Leaf-Setup-x64.exe`; seine
  SHA-256-Prüfsumme wurde mit `sha256sum -c` bestätigt. Beim Cross-Link traten
  ausschließlich die bekannten LNK4099-Hinweise zu fehlenden Microsoft-PDBs auf.
  Ein Windows-Laufzeittest wurde nicht durchgeführt.
- Die aktuelle Android-APK wurde erneut gebaut und ihre SHA-256-Prüfsumme
  bestätigt. Das Update auf dem SM-T220 scheiterte zunächst erwartungsgemäß an
  einem anderen, zuvor verwendeten Debug-Signaturschlüssel. Nach ausdrücklicher
  Bestätigung wurden die alte App samt ihren lokalen Daten deinstalliert und die
  neue, mit dem aktuellen Schlüssel signierte APK erfolgreich installiert.
  Paket, VersionCode 1, minSdk 24, targetSdk 34 und APK-Signaturversion 2 sind
  per ADB bestätigt.
- Korrektur für horizontales Wischen in importierten EPUBs: Der bisherige
  Touch-Handler galt nur für die Text-Kompatibilitätsansicht. Jede von epub.js
  geladene Iframe-Buchseite erhält nun einen eigenen Handler; Wischgesten ab
  48 px mit überwiegend horizontaler Bewegung rufen `next` beziehungsweise
  `prev` auf. JavaScript-Syntaxprüfung, Formatprüfung, 7 Rust-Tests und Clippy
  mit `-D warnings` bestanden. Die neue APK-Prüfsumme ist gültig, die APK wurde
  auf dem SM-T220 als Update installiert und gestartet. Die interaktive
  Bestätigung der Geste auf dem Gerät steht noch aus.
- Schriftwahl ergänzt: In der Kopfzeile stehen pro Buch die Familien Serif,
  Sans und Mono zur Verfügung. Die Auswahl wird zusammen mit dem Lesestand
  gespeichert, in beiden Lesewegen angewandt und ältere Zustände ohne dieses
  Feld verwenden weiter Serif. Die aktuelle ARM64-APK wurde gebaut, ihre
  Prüfsumme bestätigt und auf dem SM-T220 als Update installiert und gestartet.
  Bestanden: Formatprüfung, JavaScript-Syntaxprüfung, 7 Rust-Tests und Clippy
  mit `-D warnings`. Die visuelle Prüfung aller drei Familien auf dem Gerät
  steht noch aus.
- Einheitliches Buch-Icon ergänzt: Aus einer neu erzeugten, moosgrünen
  Hardcover-Grafik wurden `assets/leaf-icon.png` (512 px), ein mehrgrößiges
  `assets/leaf-icon.ico` für Windows und fünf Android-Dichtevarianten erzeugt.
  Der Linux-Installer registriert das PNG als `leaf-icon`; Windows kompiliert
  das ICO beim Cross-Build per `llvm-rc` als EXE-Ressource. Da Dioxus 0.7 seine
  Android-Launcher-Icons nicht aus `bundle.icon` übernimmt, ersetzt
  `build-android.sh` die generierten Ressourcen und führt danach Gradle erneut
  aus. Der Android-Build, Shell-Syntaxprüfung, Formatprüfung und 7 Rust-Tests
  bestanden; die APK-Prüfsumme wurde bestätigt und auf dem SM-T220 als Update
  installiert. Die entpackte 192-px-APK-Ressource stimmt bytegenau mit der
  Projektdatei überein. Windows konnte hier nicht neu gebaut werden, weil das
  MSVC-Rust-Target und cargo-xwin nicht mehr lokal vorhanden sind.

## Umsetzung und Prüfungen am 10. September 2026

- Nach Aktualisierung auf Commit `079e61f` wurde die Windows-x64-EXE mit
  `packaging/build-windows.sh` frisch erstellt, einschließlich portablem ZIP
  und dessen Prüfsumme. Für den Cross-Build wurden das vorhandene
  `x86_64-pc-windows-msvc`-Target, cargo-xwin sowie die Clang-/LLVM-Werkzeuge
  des Android-NDK verwendet. Der Linker meldete nur die bekannten LNK4099-
  Hinweise zu nicht verfügbaren Microsoft-PDB-Debugdateien; der Release-Build
  war erfolgreich.
- Der neue NSIS-Installer liegt unter `dist/Leaf-Setup-x64.exe`. NSIS und
  osslsigncode wurden nur temporär außerhalb des Repositories bereitgestellt.
  Das Build-Skript hat den eingebetteten Microsoft-WebView2-Bootstrapper gegen
  die Microsoft-Root-Zertifikate geprüft. Die Installer-Prüfsumme wurde danach
  mit `sha256sum -c dist/Leaf-Setup-x64.exe.sha256` bestätigt; `file` erkennt
  ihn als NSIS-Windows-GUI-Installer. Kein Laufzeittest unter Windows wurde
  durchgeführt.

## Umsetzung und Prüfungen am 11. September 2026

- Linux-Release-App mit `cargo build --locked --release` erfolgreich erstellt:
  `target/release/leaf` ist eine x86-64-ELF-Datei (9,8 MiB).
- Windows-x64-App mit `packaging/build-windows.sh` erfolgreich erstellt:
  `dist/windows-x64/Leaf.exe` sowie `dist/Leaf-windows-x64.zip`. Die EXE-
  Prüfsumme und der ZIP-Archivtest wurden bestätigt. Beim Cross-Link traten
  ausschließlich die bekannten LNK4099-Hinweise zu fehlenden Microsoft-PDBs auf.
- NSIS-Installer mit `packaging/build-installer.sh` erfolgreich erstellt:
  `dist/Leaf-Setup-x64.exe` (3,6 MiB). Weil `osslsigncode` und NSIS nicht im
  PATH lagen, wurden sie nur temporär unter `/tmp` aus den konfigurierten
  Paketquellen entpackt. Der Build prüfte den Microsoft-WebView2-Bootstrapper;
  die Installer-Prüfsumme wurde anschließend mit `sha256sum -c` bestätigt.
  Kein Laufzeittest unter Windows wurde durchgeführt.
- Korrektur der Text-Kompatibilitätsansicht: Seitensprung, Seitenzähler und
  Wiederherstellung verwenden nun einheitlich die vollständige Höhe der
  Lesefläche. Die frühere Reserve von 112 px und die nachträgliche
  Zeilen-Ausrichtung erzeugten eine Überlappung, durch die ein Rest der
  vorherigen Seite auf der nächsten angezeigt wurde. Die korrigierte
  Windows-x64-EXE, das portable ZIP und der NSIS-Installer wurden frisch
  erstellt; EXE- und Installer-Prüfsummen sowie der ZIP-Archivtest sind
  erfolgreich. `cargo fmt --check`, `cargo test` (7 Tests) und Clippy mit
  `-D warnings` bestanden. Kein Laufzeittest unter Windows wurde durchgeführt.
- Die Linux-Release-App wurde nach der Korrektur mit
  `cargo build --locked --release` neu erstellt. Eine frische Android-ARM64-APK
  wurde mit `packaging/build-android.sh` unter dem vorhandenen JDK 17, SDK und
  NDK erzeugt; ihre SHA-256-Prüfsumme ist gültig. Das über USB verbundene
  Samsung Galaxy Tab A7 Lite SM-T220 (`R9JRB01BT5M`) war autorisiert. Die
  Update-Installation per `adb install -r` wurde jedoch abgewiesen
  (`INSTALL_FAILED_UPDATE_INCOMPATIBLE`), weil die auf dem Gerät vorhandene
  App mit einem anderen Debug-Schlüssel signiert ist. Eine Deinstallation zur
  Neuinstallation wurde nicht vorgenommen, da sie lokale Leaf-Daten löscht.
- Nach ausdrücklicher Freigabe wurde `de.leaf.reader` auf dem SM-T220
  deinstalliert; die lokalen Leaf-Daten wurden dabei entfernt. Die frische APK
  wurde anschließend erfolgreich installiert und gestartet. Per ADB bestätigt:
  VersionCode 1, minSdk 24 und targetSdk 34.
- Der NSIS-Installer wurde anschließend erneut aus der korrigierten
  Windows-x64-EXE erzeugt. `dist/Leaf-Setup-x64.exe` ist als Windows-GUI-
  Installer erkannt, und seine SHA-256-Prüfsumme ist gültig. Ein Laufzeittest
  unter Windows wurde nicht durchgeführt.
- Ladebildschirm für die Leseansicht ergänzt: Er verdeckt die Oberfläche beim
  Öffnen, bis die Textansicht ihr Layout aufgebaut hat oder epub.js die erste
  Position meldet. Er wird auch bei Kapitelwechseln erneut gezeigt und
  verhindert damit das kurzzeitige Anzeigen unformatierter Inhalte.
  Formatprüfung, 7 Rust-Tests und Clippy mit `-D warnings` bestanden.
- Frische Artefakte nach dieser Änderung: Linux-Release-App, Android-ARM64-APK,
  Windows-x64-EXE, portables ZIP und NSIS-Installer. APK-, EXE- und
  Installer-Prüfsummen sowie ZIP-Archivtest sind gültig. Die APK wurde mit
  `adb install -r` auf dem verbundenen SM-T220 aktualisiert (Installationszeit
  per ADB bestätigt). Die visuelle Prüfung des neuen Ladebildschirms auf den
  Geräten sowie ein Windows-Laufzeittest stehen noch aus.
