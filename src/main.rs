#![cfg_attr(
    all(target_os = "windows", not(debug_assertions)),
    windows_subsystem = "windows"
)]

mod book;
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
struct Reading {
    book: book::Book,
    chapter: usize,
    #[serde(default)]
    page: usize,
    size: u32,
    dark: bool,
    #[serde(default)]
    scroll_top: f64,
    #[serde(default)]
    cfi: String,
    #[serde(default = "default_font_value")]
    font: String,
}
impl Default for Reading {
    fn default() -> Self {
        Self {
            book: book::demo(),
            chapter: 0,
            page: 0,
            size: 20,
            dark: false,
            scroll_top: 0.0,
            cfi: String::new(),
            font: default_font().into(),
        }
    }
}

fn default_font() -> &'static str {
    "serif"
}

fn default_font_value() -> String {
    default_font().into()
}

fn normalized_font(font: &str) -> &'static str {
    match font {
        "sans" => "sans",
        "mono" => "mono",
        _ => default_font(),
    }
}

fn font_family(font: &str) -> &'static str {
    match normalized_font(font) {
        "sans" => "Arial, Helvetica, sans-serif",
        "mono" => "'Courier New', monospace",
        _ => "Georgia, 'Times New Roman', serif",
    }
}
#[derive(Clone, Serialize, Deserialize)]
struct Library {
    books: Vec<Reading>,
    active: Option<usize>,
}
impl Default for Library {
    fn default() -> Self {
        Self {
            books: vec![Reading::default()],
            active: None,
        }
    }
}
impl Library {
    fn import(&mut self, book: book::Book) {
        let index = self
            .books
            .iter()
            .position(|entry| entry.book == book)
            .unwrap_or_else(|| {
                self.books.push(Reading {
                    book,
                    ..Reading::default()
                });
                self.books.len() - 1
            });
        self.active = Some(index);
    }
}
fn decode_library(value: &str) -> Option<Library> {
    let mut library = serde_json::from_str::<Library>(value).ok().or_else(|| {
        serde_json::from_str::<Reading>(value)
            .ok()
            .map(|reading| Library {
                books: vec![reading],
                active: Some(0),
            })
    })?;
    // Ungültige Kapitel dürfen weder Indexzugriffe noch einen falschen Buchwechsel auslösen.
    let active = library
        .active
        .filter(|&i| i < library.books.len() && !library.books[i].book.chapters.is_empty());
    library.active = active.map(|i| {
        library.books[..i]
            .iter()
            .filter(|r| !r.book.chapters.is_empty())
            .count()
    });
    library.books.retain(|r| !r.book.chapters.is_empty());
    for state in &mut library.books {
        state.chapter = state.chapter.min(state.book.chapters.len() - 1);
        state.size = state.size.clamp(16, 28);
        state.scroll_top = state.scroll_top.max(0.0);
        state.font = normalized_font(&state.font).into();
    }
    Some(library)
}

#[derive(Deserialize)]
struct PageMetrics {
    count: usize,
    page: usize,
}

#[derive(Deserialize)]
struct EpubMetrics {
    #[serde(default)]
    cfi: String,
    #[serde(default)]
    href: String,
    #[serde(default)]
    chapter: usize,
    #[serde(default)]
    page: usize,
    #[serde(default = "one")]
    pages: usize,
    #[serde(default)]
    at_start: bool,
    #[serde(default)]
    at_end: bool,
    #[serde(default)]
    error: String,
    #[serde(default)]
    toc: Vec<EpubToc>,
}

#[derive(Deserialize)]
struct EpubToc {
    label: String,
    href: String,
}

fn one() -> usize {
    1
}

fn epub_action(action: &str) {
    let action = action.to_owned();
    spawn(async move {
        let _ = document::eval(&format!("window.leafEpub?.{}();", action)).await;
    });
}

fn epub_display(href: &str) {
    let target = serde_json::to_string(href).unwrap_or_else(|_| "null".into());
    spawn(async move {
        let _ = document::eval(&format!("window.leafEpub?.display({target});")).await;
    });
}

const PAGE_SAFE_INSET: f64 = 56.0;

fn page_height(viewport_height: f64) -> f64 {
    (viewport_height - 2.0 * PAGE_SAFE_INSET).max(1.0)
}

fn turn_page(delta: i32) {
    spawn(async move {
        let _ = document::eval(&format!(
            "const reader = document.querySelector('.reading-scroll'); if (reader) reader.__leafTurnPage?.({});",
            delta
        ))
        .await;
    });
}

fn install_page_gestures() {
    spawn(async move {
        let _ = document::eval(
            r#"
            window.leafPageGestureCleanup?.();
            const reader = document.querySelector('.reading-scroll');
            if (reader) {
                const lineAt = y => {
                    const box = reader.getBoundingClientRect();
                    for (const x of [box.left + 42, box.left + box.width / 2, box.right - 42]) {
                        const caret = document.caretRangeFromPoint?.(x, y);
                        if (!caret || caret.startContainer.nodeType !== Node.TEXT_NODE) continue;
                        const text = caret.startContainer;
                        if (!text.textContent.length) continue;
                        const offset = Math.min(Math.max(0, caret.startOffset), text.textContent.length - 1);
                        const range = document.createRange();
                        range.setStart(text, offset);
                        range.setEnd(text, offset + 1);
                        const rect = range.getBoundingClientRect();
                        if (rect.height > 0) return rect;
                    }
                    return null;
                };
                reader.__leafAlignPage = () => {
                    let box = reader.getBoundingClientRect();
                    const topLine = lineAt(box.top + 1);
                    if (topLine && topLine.top < box.top) {
                        reader.scrollBy({ top: topLine.bottom - box.top + 1, behavior: 'instant' });
                    }
                    box = reader.getBoundingClientRect();
                    const bottomLine = lineAt(box.bottom - 1);
                    if (bottomLine && bottomLine.top < box.bottom && bottomLine.bottom > box.bottom) {
                        reader.scrollBy({ top: -(box.bottom - bottomLine.top + 1), behavior: 'instant' });
                    }
                };
                reader.__leafTurnPage = delta => {
                    reader.scrollBy({ top: delta * Math.max(1, reader.clientHeight - 112), behavior: 'instant' });
                    reader.__leafAlignPage();
                };
                let start_x = null;
                const start = event => {
                    event.preventDefault();
                    start_x = event.touches[0]?.clientX ?? null;
                };
                const move = event => {
                    event.preventDefault();
                    event.stopPropagation();
                };
                const end = event => {
                    const end_x = event.changedTouches[0]?.clientX;
                    if (start_x !== null && end_x !== undefined) {
                        const distance = start_x - end_x;
                        if (Math.abs(distance) >= 48) {
                            reader.__leafTurnPage(Math.sign(distance));
                        }
                    }
                    start_x = null;
                };
                reader.addEventListener('touchstart', start, { passive: false, capture: true });
                reader.addEventListener('touchmove', move, { passive: false, capture: true });
                reader.addEventListener('touchend', end, { passive: false, capture: true });
                reader.addEventListener('touchcancel', end, { passive: false, capture: true });
                window.leafPageGestureCleanup = () => {
                    reader.removeEventListener('touchstart', start, true);
                    reader.removeEventListener('touchmove', move, true);
                    reader.removeEventListener('touchend', end, true);
                    reader.removeEventListener('touchcancel', end, true);
                };
            }
            "#,
        )
        .await;
    });
}
fn restore() -> Library {
    load_reading()
        .and_then(|value| decode_library(&value))
        .unwrap_or_default()
}

fn cover_letter(title: &str) -> char {
    title
        .chars()
        .find(|character| character.is_alphanumeric())
        .map(|character| character.to_uppercase().next().unwrap_or(character))
        .unwrap_or('B')
}

fn load_reading() -> Option<String> {
    #[cfg(target_arch = "wasm32")]
    {
        web_sys::window()
            .and_then(|w| w.local_storage().ok().flatten())
            .and_then(|s| s.get_item("leaf.reading").ok().flatten())
    }
    #[cfg(all(
        any(feature = "desktop", feature = "mobile"),
        not(target_arch = "wasm32")
    ))]
    {
        std::fs::read_to_string(storage_path()?).ok()
    }
    #[cfg(all(
        not(any(feature = "desktop", feature = "mobile")),
        not(target_arch = "wasm32")
    ))]
    {
        None
    }
}

#[cfg(all(
    any(feature = "desktop", feature = "mobile"),
    not(target_arch = "wasm32")
))]
fn storage_path() -> Option<std::path::PathBuf> {
    #[cfg(target_os = "android")]
    {
        static PATH: std::sync::OnceLock<Option<std::path::PathBuf>> = std::sync::OnceLock::new();
        PATH.get_or_init(android_storage_path).clone()
    }
    #[cfg(not(target_os = "android"))]
    directories::ProjectDirs::from("de", "leaf", "Leaf")
        .map(|dirs| dirs.data_local_dir().join("reading.json"))
}

#[cfg(target_os = "android")]
fn android_storage_path() -> Option<std::path::PathBuf> {
    let context = ndk_context::android_context();
    // Dioxus/Wry hält VM und Activity während der App-Laufzeit gültig.
    let vm = unsafe { jni::JavaVM::from_raw(context.vm().cast()) }.ok()?;
    let mut env = vm.attach_current_thread().ok()?;
    let activity = unsafe { jni::objects::JObject::from_raw(context.context().cast()) };
    let dir = env
        .call_method(&activity, "getFilesDir", "()Ljava/io/File;", &[])
        .ok()?
        .l()
        .ok()?;
    let dir = env.auto_local(dir);
    let path = env
        .call_method(&dir, "getAbsolutePath", "()Ljava/lang/String;", &[])
        .ok()?
        .l()
        .ok()?;
    let path = env.auto_local(jni::objects::JString::from(path));
    let path: String = env.get_string(&path).ok()?.into();
    Some(std::path::PathBuf::from(path).join("reading.json"))
}

#[cfg(all(
    any(feature = "desktop", feature = "mobile"),
    not(target_arch = "wasm32")
))]
fn save_file(path: &std::path::Path, state: &Library) -> std::io::Result<()> {
    use std::io::Write;
    let parent = path
        .parent()
        .ok_or_else(|| std::io::Error::other("Speicherpfad fehlt"))?;
    std::fs::create_dir_all(parent)?;
    let mut file = tempfile::NamedTempFile::new_in(parent)?;
    serde_json::to_writer(&mut file, state)?;
    file.flush()?;
    file.as_file().sync_all()?;
    file.persist(path).map_err(|e| e.error)?;
    Ok(())
}

fn save(state: &Library) -> bool {
    #[cfg(target_arch = "wasm32")]
    {
        web_sys::window()
            .and_then(|w| w.local_storage().ok().flatten())
            .map(|s| {
                s.set_item(
                    "leaf.reading",
                    &serde_json::to_string(state).unwrap_or_default(),
                )
                .is_ok()
            })
            .unwrap_or(false)
    }
    #[cfg(all(
        any(feature = "desktop", feature = "mobile"),
        not(target_arch = "wasm32")
    ))]
    {
        storage_path().is_some_and(|path| save_file(&path, state).is_ok())
    }
    #[cfg(all(
        not(any(feature = "desktop", feature = "mobile")),
        not(target_arch = "wasm32")
    ))]
    {
        let _ = state;
        false
    }
}

fn main() {
    #[cfg(feature = "desktop")]
    dioxus::LaunchBuilder::desktop()
        .with_cfg(
            dioxus::desktop::Config::new()
                .with_window(
                    dioxus::desktop::WindowBuilder::new()
                        .with_title("Leaf · E-Book-Reader")
                        .with_inner_size(dioxus::desktop::tao::dpi::LogicalSize::new(1120.0, 800.0))
                        .with_min_inner_size(dioxus::desktop::tao::dpi::LogicalSize::new(
                            420.0, 520.0,
                        )),
                )
                .with_menu(None),
        )
        .launch(App);
    #[cfg(not(feature = "desktop"))]
    dioxus::launch(App);
}

#[cfg(all(
    test,
    any(feature = "desktop", feature = "mobile"),
    not(target_arch = "wasm32")
))]
mod storage_tests {
    use super::*;
    #[test]
    fn old_reading_without_position_or_page_remains_readable() {
        let mut value = serde_json::to_value(Reading::default()).unwrap();
        value.as_object_mut().unwrap().remove("scroll_top");
        value.as_object_mut().unwrap().remove("page");
        value.as_object_mut().unwrap().remove("cfi");
        value.as_object_mut().unwrap().remove("font");
        let restored: Reading = serde_json::from_value(value).unwrap();
        assert_eq!(restored.scroll_top, 0.0);
        assert_eq!(restored.page, 0);
        assert!(restored.cfi.is_empty());
        assert_eq!(restored.font, "serif");
    }

    #[test]
    fn migrates_legacy_book_with_reading_position() {
        let reading = Reading {
            chapter: 1,
            scroll_top: 321.0,
            ..Reading::default()
        };
        let library = decode_library(&serde_json::to_string(&reading).unwrap()).unwrap();
        assert_eq!(library.active, Some(0));
        assert_eq!(library.books[0].scroll_top, 321.0);
        assert_eq!(library.books[0].chapter, 1);
    }
    #[test]
    fn keeps_books_and_positions_when_closed_and_reopened() {
        let mut library = Library::default();
        assert_eq!(library.active, None);
        library.books[0].scroll_top = 654.0;
        let mut second = book::demo();
        second.title = "Zweites Buch".into();
        library.import(second.clone());
        library.books[1].chapter = 1;
        library.active = None;
        let restored = decode_library(&serde_json::to_string(&library).unwrap()).unwrap();
        assert_eq!(restored.active, None);
        assert_eq!(restored.books.len(), 2);
        assert_eq!(restored.books[0].scroll_top, 654.0);
        assert_eq!(restored.books[1].chapter, 1);
        library.import(second);
        assert_eq!(library.books.len(), 2);
        assert_eq!(library.active, Some(1));
        assert_eq!(library.books[1].chapter, 1);
    }
    #[test]
    fn invalid_active_index_returns_to_library() {
        let library = Library {
            active: Some(99),
            ..Library::default()
        };
        assert_eq!(
            decode_library(&serde_json::to_string(&library).unwrap())
                .unwrap()
                .active,
            None
        );
    }
    #[test]
    fn saves_and_replaces_reading_without_leaving_temp_files() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("data/reading.json");
        let mut state = Library::default();
        save_file(&path, &state).unwrap();
        state.books[0].chapter = 1;
        state.books[0].scroll_top = 1234.5;
        state.active = None;
        save_file(&path, &state).unwrap();
        let restored = decode_library(&std::fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(restored.books[0].chapter, 1);
        assert_eq!(restored.books[0].scroll_top, 1234.5);
        assert_eq!(restored.active, None);
        assert_eq!(
            std::fs::read_dir(path.parent().unwrap()).unwrap().count(),
            1
        );
    }
}

async fn pick_android_epub() -> Result<Option<Vec<u8>>, String> {
    #[derive(Deserialize, Default)]
    #[serde(default)]
    struct Message {
        bytes: Vec<u8>,
        done: bool,
        cancelled: bool,
        error: Option<String>,
    }
    let mut picker = document::eval(include_str!("../assets/android-picker.js"));
    let mut bytes = Vec::new();
    loop {
        let message: Message = picker
            .recv()
            .await
            .map_err(|_| "Die Android-Dateiauswahl ist nicht verfügbar.".to_string())?;
        if let Some(error) = message.error {
            return Err(error);
        }
        if message.cancelled {
            return Ok(None);
        }
        if bytes.len() + message.bytes.len() > 30 * 1024 * 1024 {
            return Err("Das EPUB darf maximal 30 MB groß sein.".into());
        }
        bytes.extend(message.bytes);
        if message.done {
            return Ok(Some(bytes));
        }
        picker
            .send(true)
            .map_err(|_| "Die Dateiübertragung wurde unterbrochen.".to_string())?;
    }
}

#[component]
fn Reader(mut library: Signal<Library>, index: usize, saved: Signal<bool>) -> Element {
    let mut reading = use_signal(move || library.peek().books[index].clone());
    let mut menu = use_signal(|| false);
    let mut page_count = use_signal(|| 1usize);
    let mut epub_at_start = use_signal(|| true);
    let mut epub_at_end = use_signal(|| false);
    let mut epub_error = use_signal(String::new);
    use_effect(move || {
        library.write().books[index] = reading.read().clone();
    });
    let appearance = use_memo(move || {
        let state = reading.read();
        (state.size, state.dark, state.font.clone())
    });
    use_effect(move || {
        let (size, dark, font) = appearance();
        let font = serde_json::to_string(&font).unwrap_or_else(|_| "\"serif\"".into());
        spawn(async move {
            let _ = document::eval(&format!(
                "window.leafEpub?.appearance({size}, {}, {font});",
                if dark { "true" } else { "false" },
            ))
            .await;
        });
    });
    let mut reader_mounted = use_signal(|| false);
    let mut position_ready = use_signal(|| false);
    let location = use_memo(move || (reading.read().chapter, reading.read().size));
    use_effect(move || {
        if reading.peek().book.epub.is_some() {
            return;
        }
        if !reader_mounted() {
            return;
        }
        let expected = location();
        position_ready.set(false);
        page_count.set(1);
        let saved_page = reading.peek().page;
        spawn(async move {
            // Styles und Kapitelinhalt müssen vor der Seitenermittlung im Layout sein.
            let _ = document::eval("await new Promise(requestAnimationFrame); await new Promise(requestAnimationFrame);").await;
            if *location.peek() == expected {
                let mut evaluator = document::eval(&format!(
                    r#"
                    const reader = document.querySelector('.reading-scroll');
                    if (reader) {{
                        const height = Math.max(1, reader.clientHeight - 112);
                        const count = Math.max(1, Math.ceil(reader.scrollHeight / height));
                        const page = Math.min({saved_page}, count - 1);
                        reader.scrollTo({{left: 0, top: page * height, behavior: 'instant'}});
                        reader.__leafAlignPage?.();
                        dioxus.send({{count, page}});
                    }}
                    "#
                ));
                if let Ok(metrics) = evaluator.recv::<PageMetrics>().await
                    && *location.peek() == expected
                {
                    page_count.set(metrics.count);
                    reading.write().page = metrics.page;
                    position_ready.set(true);
                }
            }
        });
    });
    let state = reading.read().clone();
    let chapter = &state.book.chapters[state.chapter];
    let count = state.book.chapters.len();
    let progress = (state.chapter + 1) * 100 / count;
    let pages = page_count();
    rsx! {
        document::Style { {include_str!("../assets/main.css")} }
        div { class: if state.dark { "app dark" } else { "app" },
            aside { class: if menu() { "sidebar open" } else { "sidebar" },
                a { class: "brand", href: "#", "◒" span { "leaf" } }
                p { class: "tagline", "Ein guter Ort für Geschichten." }
                ImportBook { onimport: move |book| library.write().import(book) }
                div { class: "section-label", "DEIN BUCH" }
                div { class: "book-card", div { class: "cover", "L" } div { strong { "{state.book.title}" } small { "{state.book.author}" } } }
                div { class: "section-label contents-label", "INHALT" span { "{count} Kapitel" } }
                nav { aria_label: "Inhaltsverzeichnis",
                    for (index, ch) in state.book.chapters.iter().enumerate() {
                        button { class: if index == state.chapter { "chapter active" } else { "chapter" }, onclick: move |_| {
                            if reading.peek().book.epub.is_some() {
                                epub_display(&reading.peek().book.chapters[index].href);
                            } else {
                                let mut r = reading.write();
                                if r.chapter != index { r.chapter = index; r.page = 0; r.scroll_top = 0.0; }
                            }
                            menu.set(false);
                        },
                            span { class: "chapter-number", "{index + 1:02}" } span { "{ch.title}" }
                        }
                    }
                }
                div { class: "sidebar-footer", span { class: "status-dot" } if saved() { "Lesestand lokal gespeichert" } else { "Speicher voll oder nicht verfügbar" } }
            }
            main {
                header {
                    button { class: "mobile-menu icon-button", aria_label: "Inhaltsverzeichnis umschalten", onclick: move |_| menu.toggle(), "☰" }
                    button { class: "close-book", onclick: move |_| {
                        if reading.peek().book.epub.is_some() { epub_action("destroy"); }
                        let mut state = library.write();
                        state.books[index] = reading.peek().clone();
                        state.active = None;
                    }, "Buch schließen" }
                    div { class: "tools",
                        button { class: "icon-button", aria_label: "Schrift verkleinern", disabled: state.size <= 16, onclick: move |_| { reading.write().size -= 2; }, "A−" }
                        span { class: "font-size", "{state.size}" }
                        button { class: "icon-button", aria_label: "Schrift vergrößern", disabled: state.size >= 28, onclick: move |_| { reading.write().size += 2; }, "A+" }
                        select { class: "font-family", aria_label: "Schriftart", value: "{state.font}", onchange: move |event| reading.write().font = normalized_font(&event.value()).into(),
                            option { value: "serif", "Serif" }
                            option { value: "sans", "Sans" }
                            option { value: "mono", "Mono" }
                        }
                        span { class: "divider" }
                        button { class: "icon-button", aria_label: "Farbschema wechseln", onclick: move |_| { let dark = reading.read().dark; reading.write().dark = !dark; }, if state.dark { "☀" } else { "☾" } }
                    }
                }
                if let Some(epub_data) = state.book.epub.clone() {
                    div { class: "epub-viewer", id: "epub-viewer",
                        onmounted: move |_| {
                            let config = serde_json::json!({
                                "data": epub_data,
                                "cfi": reading.peek().cfi,
                                "size": reading.peek().size,
                                "dark": reading.peek().dark,
                                "font": reading.peek().font,
                            });
                            spawn(async move {
                                let script = format!(
                                    "{}\n{}\n{}\nleafEpubOpen({});",
                                    include_str!("../assets/jszip.min.js"),
                                    include_str!("../assets/epub.min.js"),
                                    include_str!("../assets/epub-reader.js"),
                                    config
                                );
                                let mut evaluator = document::eval(&script);
                                while let Ok(metrics) = evaluator.recv::<EpubMetrics>().await {
                                    if !metrics.error.is_empty() {
                                        epub_error.set(format!("EPUB-Anzeige fehlgeschlagen: {}", metrics.error));
                                        continue;
                                    }
                                    if !metrics.toc.is_empty() {
                                        reading.write().book.chapters = metrics.toc.into_iter().map(|item| book::Chapter {
                                            title: item.label,
                                            href: item.href,
                                            html: String::new(),
                                        }).collect();
                                        continue;
                                    }
                                    let mut current = reading.write();
                                    current.cfi = metrics.cfi;
                                    let location_path = metrics.href.split('#').next().unwrap_or("");
                                    current.chapter = current.book.chapters.iter().position(|chapter| {
                                        chapter.href.split('#').next().unwrap_or("") == location_path
                                    }).unwrap_or(metrics.chapter).min(current.book.chapters.len() - 1);
                                    current.page = metrics.page;
                                    page_count.set(metrics.pages);
                                    epub_at_start.set(metrics.at_start);
                                    epub_at_end.set(metrics.at_end);
                                }
                            });
                        }
                    }
                    if !epub_error().is_empty() { p { class: "epub-error notice", role: "alert", "{epub_error}" } }
                } else {
                div { class: "reading-scroll", tabindex: "0", "data-position-ready": "{position_ready}",
                    onmounted: move |_| {
                        reader_mounted.set(true);
                        install_page_gestures();
                    },
                    onscroll: move |event| {
                        if !position_ready() { return; }
                        let height = page_height(event.client_height().max(1) as f64);
                        let top = event.scroll_top().max(0.0);
                        let current_page = (top / height).round() as usize;
                        let current_count = ((event.scroll_height().max(1) as f64) / height).ceil() as usize;
                        page_count.set(current_count.max(1));
                        if reading.peek().page != current_page
                            || (reading.peek().scroll_top - top).abs() > 0.5
                        {
                            let mut state = reading.write();
                            state.page = current_page;
                            state.scroll_top = top;
                        }
                    },
                    key: "{index}-{state.chapter}",
                    article { style: "--reading-size: {state.size}px; --reading-font: {font_family(&state.font)}",
                        div { class: "eyebrow", span {} "KAPITEL {state.chapter + 1} VON {count}" }
                        h1 { "{chapter.title}" }
                        div { class: "ornament", "✳" }
                        div { class: "prose", dangerous_inner_html: "{chapter.html}" }
                        div { class: "chapter-end", "· · ·" }
                        div { class: "chapter-navigation",
                            button { disabled: state.chapter == 0, onclick: move |_| { let mut r = reading.write(); r.chapter -= 1; r.page = 0; r.scroll_top = 0.0; }, "← Kapitel" }
                            span { "Kapitel {state.chapter + 1} / {count}" }
                            button { disabled: state.chapter + 1 >= count, onclick: move |_| { let mut r = reading.write(); r.chapter += 1; r.page = 0; r.scroll_top = 0.0; }, "Kapitel →" }
                        }
                    }
                    footer { "Eine Seite nach der anderen." span { "Nimm dir Zeit." } }
                }
                }
                div { class: "page-controls", aria_label: "Seitennavigation",
                    button { disabled: if state.book.epub.is_some() { epub_at_start() } else { state.page == 0 }, onclick: move |_| if reading.peek().book.epub.is_some() { epub_action("prev") } else { turn_page(-1) }, "← Seite" }
                    span { "Seite {state.page + 1} / {pages}" }
                    button { disabled: if state.book.epub.is_some() { epub_at_end() } else { state.page + 1 >= pages }, onclick: move |_| if reading.peek().book.epub.is_some() { epub_action("next") } else { turn_page(1) }, "Seite →" }
                }
                div { class: "progress-track", div { style: "width: {progress}%" } }
                div { class: "bottom-bar", span { "{state.book.author}" } span { "Kapitel {state.chapter + 1} von {count}" } }
            }
        }
    }
}

#[component]
fn ImportBook(onimport: EventHandler<book::Book>) -> Element {
    let mut error = use_signal(String::new);
    let mut busy = use_signal(|| false);
    rsx! {
                if cfg!(target_os = "android") {
                    button { class: "import", disabled: busy(),
                        onclick: move |_| async move {
                            busy.set(true);
                            error.set(String::new());
                            match pick_android_epub().await {
                                Ok(Some(bytes)) => match book::parse(&bytes) {
                                    Ok(book) => {
                                        onimport.call(book);
                                    }
                                    Err(message) => error.set(message),
                                },
                                Ok(None) => {},
                                Err(message) => error.set(message),
                            }
                            busy.set(false);
                        },
                        if busy() { "Wird geöffnet …" } else { "+  EPUB öffnen" }
                    }
                } else {
                label { class: "import", r#for: "epub", if busy() { "Wird geöffnet …" } else { "+  EPUB öffnen" } }
                input { id: "epub", class: "file-input", r#type: "file", accept: ".epub,application/epub+zip", disabled: busy(),
                    onchange: move |evt| async move {
                        let files = evt.files();
                        if let Some(file) = files.first() {
                            busy.set(true); error.set(String::new());
                            match file.read_bytes().await {
                                Ok(bytes) => match book::parse(&bytes) {
                                    Ok(book) => { onimport.call(book); }
                                    Err(message) => error.set(message),
                                },
                                Err(_) => error.set("Die Datei konnte nicht gelesen werden.".into()),
                            }
                            busy.set(false);
                        }
                    }
                }
                }
                if !error().is_empty() { p { class: "notice", role: "alert", "{error}" } }

    }
}

#[component]
fn App() -> Element {
    let mut library = use_signal(restore);
    let mut saved = use_signal(|| true);
    use_effect(move || saved.set(save(&library.read())));
    let active = use_memo(move || library.read().active);
    rsx! {
        document::Style { {include_str!("../assets/main.css")} }
        if let Some(index) = active() {
            Reader { key: "{index}", library, index, saved }
        } else {
            main { class: "library",
                header { div { class: "brand", "◒" span { "leaf" } } span { "Deine Bücher. Dein Lesemoment." } }
                section { class: "library-content",
                    div { class: "library-heading",
                        div { h1 { "Deine Bücher" } p { "Wähle ein Buch und lies dort weiter, wo du aufgehört hast." } }
                        div { class: "library-import", ImportBook { onimport: move |book| library.write().import(book) } }
                    }
                    if !saved() { p { class: "notice", role: "alert", "Speicher voll oder nicht verfügbar" } }
                    if library.read().books.is_empty() {
                        p { class: "empty-library", "Noch keine Bücher vorhanden. Öffne eine EPUB-Datei, um sie deiner Übersicht hinzuzufügen." }
                    }
                    div { class: "library-list", aria_label: "Bücherübersicht",
                        for (index, entry) in library.read().books.iter().enumerate() {
                            button { class: "library-book", onclick: move |_| library.write().active = Some(index),
                                div { class: "library-cover", style: "--cover-tone: {index * 47 % 360}deg",
                                    span { class: "cover-monogram", "{cover_letter(&entry.book.title)}" }
                                    span { class: "cover-title", "{entry.book.title}" }
                                    span { class: "cover-author", "{entry.book.author}" }
                                }
                                div { class: "library-book-details",
                                    strong { "{entry.book.title}" }
                                    small { "{entry.book.author}" }
                                    span { "Kapitel {entry.chapter + 1} von {entry.book.chapters.len()}" }
                                }
                                span { class: "resume-book", "Buch öffnen →" }
                            }
                        }
                    }
                }
            }
        }
    }
}
