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
    size: u32,
    dark: bool,
}
impl Default for Reading {
    fn default() -> Self {
        Self {
            book: book::demo(),
            chapter: 0,
            size: 20,
            dark: false,
        }
    }
}
fn restore() -> Reading {
    let value = load_reading();
    if let Some(mut state) = value.and_then(|value| serde_json::from_str::<Reading>(&value).ok())
        && !state.book.chapters.is_empty()
    {
        state.chapter = state.chapter.min(state.book.chapters.len() - 1);
        state.size = state.size.clamp(16, 28);
        return state;
    }
    Reading::default()
}

fn load_reading() -> Option<String> {
    #[cfg(target_arch = "wasm32")]
    {
        web_sys::window()
            .and_then(|w| w.local_storage().ok().flatten())
            .and_then(|s| s.get_item("leaf.reading").ok().flatten())
    }
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    {
        std::fs::read_to_string(storage_path()?).ok()
    }
    #[cfg(all(not(feature = "desktop"), not(target_arch = "wasm32")))]
    {
        None
    }
}

#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
fn storage_path() -> Option<std::path::PathBuf> {
    directories::ProjectDirs::from("de", "leaf", "Leaf")
        .map(|dirs| dirs.data_local_dir().join("reading.json"))
}

#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
fn save_file(path: &std::path::Path, state: &Reading) -> std::io::Result<()> {
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

fn save(state: &Reading) -> bool {
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
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    {
        storage_path().is_some_and(|path| save_file(&path, state).is_ok())
    }
    #[cfg(all(not(feature = "desktop"), not(target_arch = "wasm32")))]
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

#[cfg(all(test, feature = "desktop", not(target_arch = "wasm32")))]
mod storage_tests {
    use super::*;
    #[test]
    fn saves_and_replaces_reading_without_leaving_temp_files() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("data/reading.json");
        let mut state = Reading::default();
        save_file(&path, &state).unwrap();
        state.chapter = 1;
        state.dark = true;
        save_file(&path, &state).unwrap();
        let restored: Reading =
            serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(restored.chapter, 1);
        assert!(restored.dark);
        assert_eq!(restored.book.title, state.book.title);
        assert_eq!(
            std::fs::read_dir(path.parent().unwrap()).unwrap().count(),
            1
        );
    }
}

#[component]
fn App() -> Element {
    let mut reading = use_signal(restore);
    let mut error = use_signal(String::new);
    let mut busy = use_signal(|| false);
    let mut menu = use_signal(|| false);
    let mut saved = use_signal(|| true);
    use_effect(move || {
        saved.set(save(&reading.read()));
    });
    let state = reading.read().clone();
    let chapter = &state.book.chapters[state.chapter];
    let count = state.book.chapters.len();
    let progress = (state.chapter + 1) * 100 / count;
    rsx! {
        document::Style { {include_str!("../assets/main.css")} }
        div { class: if state.dark { "app dark" } else { "app" },
            aside { class: if menu() { "sidebar open" } else { "sidebar" },
                a { class: "brand", href: "#", "◒" span { "leaf" } }
                p { class: "tagline", "Ein guter Ort für Geschichten." }
                label { class: "import", r#for: "epub", if busy() { "Wird geöffnet …" } else { "+  EPUB öffnen" } }
                input { id: "epub", class: "file-input", r#type: "file", accept: ".epub,application/epub+zip", disabled: busy(),
                    onchange: move |evt| async move {
                        let files = evt.files();
                        if let Some(file) = files.first() {
                            busy.set(true); error.set(String::new());
                            match file.read_bytes().await {
                                Ok(bytes) => match book::parse(&bytes) {
                                    Ok(book) => { let mut r = reading.write(); r.book = book; r.chapter = 0; menu.set(false); }
                                    Err(message) => error.set(message),
                                },
                                Err(_) => error.set("Die Datei konnte nicht gelesen werden.".into()),
                            }
                            busy.set(false);
                        }
                    }
                }
                if !error().is_empty() { p { class: "notice", role: "alert", "{error}" } }
                div { class: "section-label", "DEIN BUCH" }
                div { class: "book-card", div { class: "cover", "L" } div { strong { "{state.book.title}" } small { "{state.book.author}" } } }
                div { class: "section-label contents-label", "INHALT" span { "{count} Kapitel" } }
                nav { aria_label: "Inhaltsverzeichnis",
                    for (index, ch) in state.book.chapters.iter().enumerate() {
                        button { class: if index == state.chapter { "chapter active" } else { "chapter" }, onclick: move |_| { reading.write().chapter = index; menu.set(false); },
                            span { class: "chapter-number", "{index + 1:02}" } span { "{ch.title}" }
                        }
                    }
                }
                div { class: "sidebar-footer", span { class: "status-dot" } if saved() { "Lesestand lokal gespeichert" } else { "Speicher voll oder nicht verfügbar" } }
            }
            main {
                header {
                    button { class: "mobile-menu icon-button", aria_label: "Inhaltsverzeichnis umschalten", onclick: move |_| menu.toggle(), "☰" }
                    div { class: "breadcrumb", "DEIN LESEMOMENT" span { " / " } "{state.book.title}" }
                    div { class: "tools",
                        button { class: "icon-button", aria_label: "Schrift verkleinern", disabled: state.size <= 16, onclick: move |_| { reading.write().size -= 2; }, "A−" }
                        span { class: "font-size", "{state.size}" }
                        button { class: "icon-button", aria_label: "Schrift vergrößern", disabled: state.size >= 28, onclick: move |_| { reading.write().size += 2; }, "A+" }
                        span { class: "divider" }
                        button { class: "icon-button", aria_label: "Farbschema wechseln", onclick: move |_| { let dark = reading.read().dark; reading.write().dark = !dark; }, if state.dark { "☀" } else { "☾" } }
                    }
                }
                div { class: "reading-scroll", key: "{state.book.title}-{state.chapter}",
                    article { style: "--reading-size: {state.size}px",
                        div { class: "eyebrow", span {} "KAPITEL {state.chapter + 1} VON {count}" }
                        h1 { "{chapter.title}" }
                        div { class: "ornament", "✳" }
                        div { class: "prose", dangerous_inner_html: "{chapter.html}" }
                        div { class: "chapter-end", "· · ·" }
                        div { class: "navigation",
                            button { disabled: state.chapter == 0, onclick: move |_| { reading.write().chapter -= 1; }, "← Zurück" }
                            span { "{state.chapter + 1} / {count}" }
                            button { disabled: state.chapter + 1 >= count, onclick: move |_| { reading.write().chapter += 1; }, "Weiter →" }
                        }
                    }
                    footer { "Eine Seite nach der anderen." span { "Nimm dir Zeit." } }
                }
                div { class: "progress-track", div { style: "width: {progress}%" } }
                div { class: "bottom-bar", span { "{state.book.author}" } span { "Kapitel {state.chapter + 1} von {count}" } }
            }
        }
    }
}
