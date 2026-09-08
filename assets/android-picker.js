await new Promise(resolve => {
// Eigenes DOM-Element: Dioxus' Desktop-Dateidialog unterstützt Android nicht.
const input = document.createElement("input");
input.type = "file";
input.accept = ".epub,application/epub+zip,application/octet-stream";
input.style.display = "none";
document.body.appendChild(input);
input.addEventListener("click", event => event.stopPropagation());
input.addEventListener("cancel", () => {
    input.remove();
    dioxus.send({done: true, cancelled: true});
    resolve();
}, {once: true});
input.addEventListener("change", async () => {
    try {
        const file = input.files[0];
        if (!file) {
            dioxus.send({done: true, cancelled: true});
        } else if (file.size > 30 * 1024 * 1024) {
            dioxus.send({error: "Das EPUB darf maximal 30 MB groß sein."});
        } else {
            // Begrenzte Nachrichten statt einer riesigen JSON-Nachricht.
            for (let offset = 0; offset < file.size; offset += 64 * 1024) {
                const bytes = new Uint8Array(await file.slice(offset, offset + 64 * 1024).arrayBuffer());
                dioxus.send({bytes: Array.from(bytes)});
                await dioxus.recv();
            }
            dioxus.send({done: true});
        }
    } catch (_) {
        dioxus.send({error: "Die Datei konnte nicht gelesen werden."});
    } finally {
        input.remove();
        resolve();
    }
}, {once: true});
input.click();

});
