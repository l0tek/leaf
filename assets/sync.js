// Gemeinsamer Fetch-Weg für Web, Desktop-WebView und Android-WebView.
// Das Rust-Frontend wartet auf die Antwort über dioxus.send().
async function leafSync(endpoint, request) {
  try {
    const response = await fetch(endpoint, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(request),
    });
    const body = await response.json();
    if (!response.ok && body.ok === undefined) {
      body.ok = false;
      body.error = "Der Sync-Dienst hat einen Fehler gemeldet.";
    }
    dioxus.send(body);
  } catch (_) {
    dioxus.send({ ok: false, error: "Der Sync-Dienst ist nicht erreichbar." });
  }
}
