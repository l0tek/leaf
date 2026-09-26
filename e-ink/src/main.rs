//! Erste statische Startseite fuer das Waveshare 7.5" e-Paper HAT V2
//! (800 x 480, S/W).
//!
//! Die Befehlsfolge entspricht dem funktionierenden Waveshare-Python-Beispiel
//! `epd_7in5_V2_test.py`. BCM: RST=17, DC=25, BUSY=24, SPI0/CE0.

use std::{
    error::Error,
    io::Write,
    time::{Duration, Instant},
};

use embedded_graphics::{
    image::{Image, ImageRaw},
    mono_font::{
        MonoTextStyleBuilder,
        ascii::{FONT_6X10, FONT_9X15_BOLD, FONT_10X20},
    },
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{Line, PrimitiveStyle, Rectangle},
    text::{Baseline, Text, TextStyleBuilder},
};
use epd_waveshare::{epd7in5_v2::Display7in5, graphics::DisplayRotation, prelude::*};
use gpio_cdev::{Chip, LineHandle, LineRequestFlags};
use spidev::{SpiModeFlags, Spidev, SpidevOptions};

const RST_PIN: u8 = 17;
const DC_PIN: u8 = 25;
const BUSY_PIN: u8 = 24;
const PWR_PIN: u8 = 18;
const FRAME_BYTES: usize = 800 * 480 / 8;
const PORTRAIT_ROTATION: DisplayRotation = DisplayRotation::Rotate90;
// Standardwert von /sys/module/spidev/parameters/bufsiz auf Raspberry Pi OS.
const SPI_CHUNK_BYTES: usize = 4096;

// 32 × 32 Pixel, 1 Bit pro Pixel, Big-Endian: drei Buecher auf einem Regal.
// Das Bitmap wird ueber `color_converted()` nach Schwarz/Weiss abgebildet.
const BOOKS_ICON_BITMAP: [u8; 128] = [
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x1f, 0xf0, 0x00, 0x00, 0x1f, 0xf0, 0x00, 0x00, 0x18, 0x30, 0x00, 0x00, 0x18, 0x30, 0x00,
    0x3f, 0xd8, 0x30, 0x00, 0x3f, 0xd8, 0x30, 0x00, 0x30, 0xd8, 0x30, 0x00, 0x30, 0xd8, 0x37, 0xfc,
    0x30, 0xd8, 0x37, 0xfc, 0x30, 0xd8, 0x36, 0x0c, 0x30, 0xd8, 0x36, 0x0c, 0x30, 0xd8, 0x36, 0x0c,
    0x30, 0xd8, 0x36, 0x0c, 0x30, 0xd8, 0x36, 0x0c, 0x30, 0xd8, 0x36, 0x0c, 0x30, 0xd8, 0x36, 0x0c,
    0x30, 0xd8, 0x36, 0x0c, 0x30, 0xd8, 0x36, 0x0c, 0x30, 0xd8, 0x36, 0x0c, 0x30, 0xd8, 0x36, 0x0c,
    0x30, 0xd8, 0x36, 0x0c, 0x30, 0xd8, 0x36, 0x0c, 0x3f, 0xdf, 0xf7, 0xfc, 0x3f, 0xdf, 0xf7, 0xfc,
    0x00, 0x00, 0x00, 0x00, 0x3f, 0xff, 0xff, 0xfc, 0x3f, 0xff, 0xff, 0xfc, 0x00, 0x00, 0x00, 0x00,
];

struct Epd {
    spi: Spidev,
    busy: LineHandle,
    dc: LineHandle,
    rst: LineHandle,
    // Das Waveshare-Python-Modul schaltet BCM 18 in module_init() auf High.
    // Der Pin muss waehrend der gesamten Display-Nutzung gehalten werden.
    _pwr: LineHandle,
}

impl Epd {
    fn command(&mut self, command: u8) -> Result<(), Box<dyn Error>> {
        self.dc.set_value(0)?;
        self.spi.write_all(&[command])?;
        Ok(())
    }

    fn data(&mut self, data: &[u8]) -> Result<(), Box<dyn Error>> {
        self.dc.set_value(1)?;
        for chunk in data.chunks(SPI_CHUNK_BYTES) {
            self.spi.write_all(chunk)?;
        }
        Ok(())
    }

    fn command_with_data(&mut self, command: u8, data: &[u8]) -> Result<(), Box<dyn Error>> {
        self.command(command)?;
        self.data(data)
    }

    fn reset(&mut self) {
        self.rst.set_value(1).expect("RST auf High setzen");
        std::thread::sleep(Duration::from_millis(20));
        self.rst.set_value(0).expect("RST auf Low setzen");
        std::thread::sleep(Duration::from_millis(2));
        self.rst.set_value(1).expect("RST auf High setzen");
        std::thread::sleep(Duration::from_millis(20));
    }

    fn wait_until_idle(&mut self, phase: &str) -> Result<(), Box<dyn Error>> {
        let started = Instant::now();
        loop {
            self.command(0x71)?;
            if self.busy.get_value()? != 0 {
                break;
            }
            if started.elapsed() > Duration::from_secs(30) {
                return Err(format!("Zeitueberschreitung beim BUSY-Signal ({phase})").into());
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        std::thread::sleep(Duration::from_millis(20));
        Ok(())
    }

    fn init(&mut self) -> Result<(), Box<dyn Error>> {
        println!("Reset …");
        self.reset();
        println!("Booster-Konfiguration …");
        self.command_with_data(0x06, &[0x17, 0x17, 0x28, 0x17])?;
        println!("Spannungsversorgung einschalten …");
        self.command_with_data(0x01, &[0x07, 0x07, 0x28, 0x17])?;
        self.command(0x04)?;
        std::thread::sleep(Duration::from_millis(100));
        println!("Warte auf BUSY nach Einschalten …");
        self.wait_until_idle("Initialisierung")?;
        println!("Panel-Konfiguration …");
        self.command_with_data(0x00, &[0x1f])?;
        self.command_with_data(0x61, &[0x03, 0x20, 0x01, 0xe0])?;
        self.command_with_data(0x15, &[0x00])?;
        self.command_with_data(0x50, &[0x10, 0x07])?;
        self.command_with_data(0x60, &[0x22])?;
        Ok(())
    }

    fn display(&mut self, frame: &[u8]) -> Result<(), Box<dyn Error>> {
        assert_eq!(frame.len(), FRAME_BYTES);
        self.command(0x10)?;
        self.data(frame)?;
        self.command(0x13)?;
        for chunk in frame.chunks(SPI_CHUNK_BYTES) {
            let inverted: Vec<u8> = chunk.iter().map(|byte| !byte).collect();
            self.data(&inverted)?;
        }
        self.command(0x12)?;
        std::thread::sleep(Duration::from_millis(100));
        self.wait_until_idle("Vollrefresh")
    }

    fn sleep(&mut self) -> Result<(), Box<dyn Error>> {
        self.command_with_data(0x50, &[0xf7])
    }
}

fn draw_start_page(display: &mut Display7in5) {
    let small = MonoTextStyleBuilder::new()
        .font(&FONT_6X10)
        .text_color(Color::Black)
        .build();
    let heading = MonoTextStyleBuilder::new()
        .font(&FONT_9X15_BOLD)
        .text_color(Color::Black)
        .build();
    let title = MonoTextStyleBuilder::new()
        .font(&FONT_10X20)
        .text_color(Color::Black)
        .build();
    let top = TextStyleBuilder::new().baseline(Baseline::Top).build();
    let border = PrimitiveStyle::with_stroke(Color::Black, 2);
    let fine_line = PrimitiveStyle::with_stroke(Color::Black, 1);

    display.clear(Color::White).expect("Bildpuffer loeschen");
    Rectangle::new(Point::new(16, 16), Size::new(448, 768))
        .into_styled(border)
        .draw(display)
        .expect("Rahmen zeichnen");

    Text::with_text_style("LEAF", Point::new(34, 38), title, top)
        .draw(display)
        .expect("Titel zeichnen");
    Text::with_text_style("E-INK READER", Point::new(34, 64), small, top)
        .draw(display)
        .expect("Untertitel zeichnen");
    Text::with_text_style("STARTSEITE", Point::new(354, 48), small, top)
        .draw(display)
        .expect("Seitentitel zeichnen");
    Line::new(Point::new(34, 92), Point::new(446, 92))
        .into_styled(fine_line)
        .draw(display)
        .expect("Kopfzeile zeichnen");

    Rectangle::new(Point::new(34, 120), Size::new(412, 370))
        .into_styled(border)
        .draw(display)
        .expect("Inhaltskarte zeichnen");
    Text::with_text_style("Willkommen.", Point::new(58, 150), heading, top)
        .draw(display)
        .expect("Begruessung zeichnen");
    let books_icon = ImageRaw::<BinaryColor>::new(&BOOKS_ICON_BITMAP, 32);
    Image::new(&books_icon, Point::new(378, 148))
        .draw(&mut display.color_converted())
        .expect("Buecherbitmap zeichnen");
    Text::with_text_style("Deine Bibliothek", Point::new(58, 194), title, top)
        .draw(display)
        .expect("Status zeichnen");
    Text::with_text_style("ist bereit.", Point::new(58, 222), title, top)
        .draw(display)
        .expect("Status zeichnen");
    Text::with_text_style(
        "Ein ruhiger Ort fuer deine Buecher.",
        Point::new(58, 274),
        small,
        top,
    )
    .draw(display)
    .expect("Importhinweis zeichnen");

    Rectangle::new(Point::new(58, 330), Size::new(364, 102))
        .into_styled(fine_line)
        .draw(display)
        .expect("Bibliotheksstatus zeichnen");
    Text::with_text_style("0 BUECHER", Point::new(82, 354), heading, top)
        .draw(display)
        .expect("Buchanzahl zeichnen");
    Text::with_text_style(
        "Der EPUB-Import folgt als naechster Schritt.",
        Point::new(82, 388),
        small,
        top,
    )
    .draw(display)
    .expect("Importhinweis zeichnen");

    Line::new(Point::new(34, 650), Point::new(446, 650))
        .into_styled(fine_line)
        .draw(display)
        .expect("Fusszeile zeichnen");
    Text::with_text_style("ZURUECK", Point::new(52, 682), heading, top)
        .draw(display)
        .expect("Zurueck zeichnen");
    Text::with_text_style("MENUE", Point::new(198, 682), heading, top)
        .draw(display)
        .expect("Menue zeichnen");
    Text::with_text_style("WEITER", Point::new(344, 682), heading, top)
        .draw(display)
        .expect("Weiter zeichnen");
    Text::with_text_style(
        "Tastenanschluss folgt ueber GPIO-Erweiterung.",
        Point::new(82, 724),
        small,
        top,
    )
    .draw(display)
    .expect("Tastenhinweis zeichnen");
}

fn main() -> Result<(), Box<dyn Error>> {
    println!("Initialisiere Waveshare 7.5\" e-Paper V2 …");
    println!("Oeffne SPI0/CE0 …");
    let mut spi = Spidev::open("/dev/spidev0.0")?;
    let options = SpidevOptions::new()
        .bits_per_word(8)
        .max_speed_hz(4_000_000)
        .mode(SpiModeFlags::SPI_MODE_0)
        .build();
    spi.configure(&options)?;
    println!("Oeffne GPIO …");
    let mut gpio = Chip::new("/dev/gpiochip0")?;
    println!("Schalte Panelversorgung (GPIO 18) ein …");
    let pwr =
        gpio.get_line(u32::from(PWR_PIN))?
            .request(LineRequestFlags::OUTPUT, 1, "leaf-eink")?;
    println!("Konfiguriere BUSY (GPIO 24) …");
    let busy =
        gpio.get_line(u32::from(BUSY_PIN))?
            .request(LineRequestFlags::INPUT, 0, "leaf-eink")?;
    println!("Konfiguriere DC (GPIO 25) …");
    let dc = gpio
        .get_line(u32::from(DC_PIN))?
        .request(LineRequestFlags::OUTPUT, 1, "leaf-eink")?;
    println!("Konfiguriere RST (GPIO 17) …");
    let rst =
        gpio.get_line(u32::from(RST_PIN))?
            .request(LineRequestFlags::OUTPUT, 1, "leaf-eink")?;
    let mut epd = Epd {
        spi,
        busy,
        dc,
        rst,
        _pwr: pwr,
    };

    println!("Initialisiere Controller …");
    epd.init()?;
    println!("Display initialisiert; zeichne Leaf-Startseite …");
    let mut display = Display7in5::default();
    display.set_rotation(PORTRAIT_ROTATION);
    draw_start_page(&mut display);

    println!("Sende Vollrefresh …");
    epd.display(display.buffer())?;
    epd.sleep()?;
    println!("Fertig. Das Display befindet sich im Ruhemodus.");
    Ok(())
}
