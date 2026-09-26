# Leaf e-Ink – Hardwaretest

Dieser erste, eigenständige Rust-Test zeigt `Leaf e-Ink Test` auf einem
Waveshare **7.5 inch e-Paper HAT V2** (800 × 480, Schwarz/Weiß) am Raspberry
Pi 3 B mit 64-Bit Raspberry Pi OS. Seine Befehlsfolge entspricht `epd_7in5_V2_test.py` aus dem
[Waveshare-e-Paper-Repository](https://github.com/waveshare/e-Paper).

## Verdrahtung und Pi-Konfiguration

Das Waveshare-HAT nutzt die Standardpins: SPI0 mit CE0 (BCM 8), MOSI (10),
SCLK (11), dazu RST (17), DC (25), BUSY (24) und die vom Waveshare-Python-
Treiber aktivierte Panelversorgung PWR (18). SPI muss aktiviert sein,
beispielsweise mit `sudo raspi-config` unter *Interface Options → SPI*.
Danach muss `/dev/spidev0.0` existieren.

Die Anwendung verwendet die Kernel-Schnittstellen `/dev/gpiochip0` und
`/dev/spidev0.0`; die veraltete GPIO-Sysfs-Schnittstelle und direkte
Registerzugriffe werden nicht benutzt. Sie muss mit `sudo` ausgeführt werden.
Auf dem Pi ist keine Rust-Toolchain erforderlich.

Die Startseite verwendet standardmaessig das Hochformat (480 × 800 Pixel).
Sie wird im Zeichenpuffer um 90 Grad gedreht; falls die geplante physische
Montage die Oberkante auf die andere Seite legt, kann der Wert
`PORTRAIT_ROTATION` in `src/main.rs` von `Rotate90` auf `Rotate270` wechseln.
Sie zeigt zudem ein fest eingebettetes, 32 × 32 Pixel grosses 1-Bit-Bitmap mit
drei Buechern; dafuer ist keine Bilddatei oder Laufzeit-Abhaengigkeit noetig.

## Geplanter Tastenanschluss

Das Waveshare-HAT hat keinen durchgeschleiften GPIO-Header und das 7,5-Zoll-
V2-Display besitzt keinen Touch-Controller. Fuer die spaetere Bedienung wird
deshalb ein [Geekworm G341 GPIO Extension Header](https://www.amazon.de/dp/B0BD79QW8K)
zwischen Pi und Display-HAT verwendet. Er verteilt die vorhandenen 40 Pins auf
zwei gleichwertige Anschluesse; das Display belegt einen, der andere fuehrt die
Taster heraus. Er erweitert keine GPIOs und benoetigt keine zusaetzliche
Software.

Die drei rastfreien Taster werden jeweils gegen Masse geschaltet:

| Funktion | G341-/Pi-Pin | BCM-GPIO | Verbindung |
| --- | ---: | ---: | --- |
| Zurueck | 29 | 5 | Taster nach Pin 34 (GND) |
| Weiter | 31 | 6 | Taster nach Pin 34 (GND) |
| Menue | 33 | 13 | Taster nach Pin 34 (GND) |

Diese Pins kollidieren nicht mit SPI0/CE0 oder den Display-Steuerleitungen.
Die noch zu implementierende Eingabelogik konfiguriert sie als aktive Low-
Eingaenge mit Pull-up und entprellt sie in Software.

## Lokaler Cross-Build für den Pi 3 B (64 Bit)

```sh
./e-ink/build-pi-3.sh
```

Das Skript baut ein Release-Binärprogramm für `aarch64-unknown-linux-gnu`
(ARM64). Es benötigt Docker sowie
[`cross`](https://github.com/cross-rs/cross); die Ausgabe liegt unter
`e-ink/target/aarch64-unknown-linux-gnu/release/leaf-e-ink-test`.

## Deployment vom Entwicklungsrechner

```sh
./e-ink/deploy.sh
```

Das Skript erstellt zuerst das ARM64-Release lokal, kopiert anschließend nur die
Binaerdatei nach `~/leaf/e-ink/leaf-e-ink-test` und startet die statische
Leaf-Startseite mit Root-Rechten.
Es setzt funktionierende
SSH-Anmeldung für `l0tek@192.168.113.110` voraus.

> Wichtig: Dies ist **nur** für das V2-Schwarzweiß-Panel. Die äußerlich ähnlich
> benannten V1-, HD- und dreifarbigen 7.5-Zoll-Module verwenden andere Treiber.
