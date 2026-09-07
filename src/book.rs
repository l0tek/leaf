use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    io::{Cursor, Read},
};

#[derive(Clone, Serialize, Deserialize)]
pub struct Chapter {
    pub title: String,
    pub html: String,
}
#[derive(Clone, Serialize, Deserialize)]
pub struct Book {
    pub title: String,
    pub author: String,
    pub chapters: Vec<Chapter>,
}

fn entry(zip: &mut zip::ZipArchive<Cursor<&[u8]>>, path: &str) -> Result<String, String> {
    let file = zip
        .by_name(path)
        .map_err(|_| format!("Datei im EPUB fehlt: {path}"))?;
    if file.size() > 8 * 1024 * 1024 {
        return Err("Ein Kapitel ist zu groß (max. 8 MB).".into());
    }
    let mut text = String::new();
    file.take(8 * 1024 * 1024 + 1)
        .read_to_string(&mut text)
        .map_err(|_| "EPUB enthält ungültigen Text.".to_string())?;
    Ok(text)
}

pub fn parse(bytes: &[u8]) -> Result<Book, String> {
    if bytes.len() > 30 * 1024 * 1024 {
        return Err("Das EPUB darf maximal 30 MB groß sein.".into());
    }
    let mut zip = zip::ZipArchive::new(Cursor::new(bytes))
        .map_err(|_| "Keine gültige EPUB-Datei.".to_string())?;
    let container = entry(&mut zip, "META-INF/container.xml")?;
    let doc = roxmltree::Document::parse(&container)
        .map_err(|_| "Ungültiger EPUB-Container.".to_string())?;
    let path = doc
        .descendants()
        .find(|n| n.has_tag_name("rootfile"))
        .and_then(|n| n.attribute("full-path"))
        .ok_or("EPUB-Paket fehlt.")?;
    let base = path
        .rsplit_once('/')
        .map(|(b, _)| format!("{b}/"))
        .unwrap_or_default();
    let package = entry(&mut zip, path)?;
    let doc = roxmltree::Document::parse(&package)
        .map_err(|_| "Ungültige EPUB-Metadaten.".to_string())?;
    let metadata = |tag| {
        doc.descendants()
            .find(|n| n.tag_name().name() == tag)
            .and_then(|n| n.text())
            .unwrap_or("")
            .to_owned()
    };
    let mut book = Book {
        title: metadata("title"),
        author: metadata("creator"),
        chapters: vec![],
    };
    if book.title.is_empty() {
        book.title = "Unbenanntes Buch".into();
    }
    let items: HashMap<_, _> = doc
        .descendants()
        .filter(|n| n.has_tag_name("item"))
        .filter_map(|n| Some((n.attribute("id")?, n.attribute("href")?)))
        .collect();
    let mut total = 0;
    for item in doc.descendants().filter(|n| n.has_tag_name("itemref")) {
        if item.attribute("linear") == Some("no") {
            continue;
        }
        let href = items
            .get(item.attribute("idref").unwrap_or(""))
            .ok_or("Kapitelreferenz fehlt.")?;
        let source = entry(&mut zip, &format!("{base}{href}"))?;
        total += source.len();
        if total > 40 * 1024 * 1024 {
            return Err("Der entpackte Buchinhalt ist zu groß.".into());
        }
        let chapter_doc = roxmltree::Document::parse(&source).ok();
        let title = chapter_doc
            .as_ref()
            .and_then(|d| {
                d.descendants()
                    .find(|n| matches!(n.tag_name().name(), "h1" | "h2" | "title"))
                    .map(|n| {
                        n.descendants()
                            .filter_map(|c| c.text().filter(|_| c.is_text()))
                            .collect::<String>()
                    })
            })
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| format!("Kapitel {}", book.chapters.len() + 1));
        // No active content, remote resources, or navigation from imported books.
        let html = ammonia::Builder::default()
            .rm_tags(&["img", "a", "style"])
            .clean(&source)
            .to_string();
        book.chapters.push(Chapter { title, html });
    }
    if book.chapters.is_empty() {
        return Err("Dieses EPUB enthält keine lesbaren Kapitel.".into());
    }
    Ok(book)
}

pub fn demo() -> Book {
    Book { title: "Ein neuer Anfang".into(), author: "Willkommen bei Leaf".into(), chapters: vec![
        Chapter { title: "Die Kunst der kleinen Pause".into(), html: "<p>Es gibt diese stillen Momente, in denen die Welt für einen Augenblick langsamer wird. Der Tee dampft noch, das Licht fällt weich durch das Fenster, und vor uns liegt eine Geschichte, die darauf wartet, entdeckt zu werden.</p><p>Ein Buch aufzuschlagen bedeutet, eine Tür zu öffnen. Wir wissen noch nicht, wohin sie führt. Vielleicht in eine fremde Stadt, vielleicht in eine längst vergangene Zeit. Manchmal führt sie uns einfach ein Stück näher zu uns selbst.</p><h2>Raum für Geschichten</h2><p>Leaf ist dein Platz für solche Momente. Ohne Eile. Ohne Ablenkung. Nur du und die nächste Seite.</p><p>Öffne ein eigenes EPUB über die Schaltfläche links. Deine Kapitel erscheinen im Inhaltsverzeichnis. Schriftgröße und Farbschema kannst du jederzeit anpassen.</p><blockquote>Man muss nicht weit reisen, um neue Welten zu entdecken. Manchmal genügt eine einzige Seite.</blockquote><p>Mach es dir bequem. Die Geschichte beginnt hier.</p>".into() },
        Chapter { title: "Dein nächstes Kapitel".into(), html: "<p>Jede Geschichte beginnt mit Neugier. Welche möchtest du heute entdecken?</p><p>Importiere ein DRM-freies EPUB, um loszulesen. Leaf merkt sich dein zuletzt geöffnetes Buch und Kapitel auf diesem Gerät, sofern ausreichend lokaler Speicher verfügbar ist.</p><h2>Lesen in deinem Rhythmus</h2><p>Nutze das Inhaltsverzeichnis oder die Pfeile unter dem Text, um zwischen Kapiteln zu wechseln. Wähle einen dunklen Hintergrund für den Abend oder warmes Papier für den Tag.</p>".into() }
    ] }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    fn epub() -> Vec<u8> {
        let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
        for (name, text) in [
            (
                "META-INF/container.xml",
                "<container><rootfile full-path='OPS/book.opf'/></container>",
            ),
            (
                "OPS/book.opf",
                "<package><metadata><title>Testbuch</title><creator>Autor</creator></metadata><manifest><item id='b' href='b.xhtml'/><item id='a' href='a.xhtml'/></manifest><spine><itemref idref='a'/><itemref idref='b'/></spine></package>",
            ),
            (
                "OPS/a.xhtml",
                "<html><body><h1>Erstes</h1><script>alert(1)</script><p onclick='evil()'>Hallo</p><img src='https://example.com/x'/></body></html>",
            ),
            (
                "OPS/b.xhtml",
                "<html><body><h1>Zweites</h1><p>Welt</p></body></html>",
            ),
        ] {
            zip.start_file(name, zip::write::SimpleFileOptions::default())
                .unwrap();
            zip.write_all(text.as_bytes()).unwrap();
        }
        zip.finish().unwrap().into_inner()
    }
    #[test]
    fn reads_spine_and_sanitizes() {
        let b = parse(&epub()).unwrap();
        assert_eq!(b.title, "Testbuch");
        assert_eq!(b.chapters[0].title, "Erstes");
        assert_eq!(b.chapters[1].title, "Zweites");
        assert!(!b.chapters[0].html.contains("script"));
        assert!(!b.chapters[0].html.contains("onclick"));
        assert!(!b.chapters[0].html.contains("https://"));
    }
    #[test]
    fn rejects_invalid_archive() {
        assert!(parse(b"not an epub").is_err());
    }
}
