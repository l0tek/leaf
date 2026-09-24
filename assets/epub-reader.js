window.leafEpubOpen = async function (config) {
  window.leafEpub?.destroy();
  const binary = atob(config.data);
  const bytes = new Uint8Array(binary.length);
  for (let index = 0; index < binary.length; index += 1) bytes[index] = binary.charCodeAt(index);

  const book = ePub(bytes.buffer);
  const rendition = book.renderTo("epub-viewer", {
    width: "100%",
    height: "100%",
    flow: "paginated",
    manager: "default",
    spread: "none"
  });
  const palette = {
    light: { body: { color: "#343a32", background: "#faf9f6" } },
    dark: { body: { color: "#e0e3d8", background: "#21251f" } }
  };
  rendition.themes.register("light", palette.light);
  rendition.themes.register("dark", palette.dark);
  rendition.themes.select(config.dark ? "dark" : "light");
  rendition.themes.default({
    body: {
      "font-family": `${config.font || "Georgia, 'Times New Roman', serif"} !important`,
      "line-height": "1.8 !important",
      "padding": "0 5% !important",
      "touch-action": "none !important",
      "overscroll-behavior": "contain !important"
    },
    img: { "max-width": "100% !important", "max-height": "100% !important" },
    a: { color: "#526846 !important" }
  });
  rendition.themes.fontSize(`${config.size}px`);
  rendition.themes.font(config.font || "Georgia, 'Times New Roman', serif");

  // epub.js rendert jedes Spine-Element in einem eigenen Iframe. Der Handler
  // muss deshalb an dessen Dokument hängen, nicht nur an den äußeren Viewer.
  // So funktioniert die Geste auch nach einem Seiten- oder Kapitelwechsel.
  rendition.hooks.content.register(contents => {
    const document = contents.document;
    let startX = null;
    let startY = null;
    const start = event => {
      const touch = event.touches[0];
      if (!touch || event.touches.length !== 1) return;
      startX = touch.clientX;
      startY = touch.clientY;
    };
    const move = event => {
      if (startX === null || event.touches.length !== 1) return;
      event.preventDefault();
    };
    const end = event => {
      const touch = event.changedTouches[0];
      if (!touch || startX === null || startY === null) return;
      const horizontal = startX - touch.clientX;
      const vertical = startY - touch.clientY;
      if (Math.abs(horizontal) >= 48 && Math.abs(horizontal) > Math.abs(vertical)) {
        if (horizontal > 0) rendition.next();
        else rendition.prev();
      }
      startX = null;
      startY = null;
    };
    const cancel = () => { startX = null; startY = null; };
    document.addEventListener("touchstart", start, { passive: true, capture: true });
    document.addEventListener("touchmove", move, { passive: false, capture: true });
    document.addEventListener("touchend", end, { passive: true, capture: true });
    document.addEventListener("touchcancel", cancel, { passive: true, capture: true });
  });

  let pageCounts = [];
  let measuringPages = false;
  let paginationTimer = null;
  let lastLocation = null;

  const pageMetric = location => {
    const section = location.start.index || 0;
    const localPage = Math.max(0, (location.start.displayed?.page || 1) - 1);
    const knownPages = pageCounts.length ? pageCounts : [Math.max(1, location.start.displayed?.total || 1)];
    const before = knownPages.slice(0, section).reduce((sum, pages) => sum + pages, 0);
    return {
      cfi: location.start.cfi,
      href: location.start.href || "",
      chapter: section,
      page: before + localPage,
      pages: Math.max(1, knownPages.reduce((sum, pages) => sum + pages, 0)),
      at_start: Boolean(location.atStart),
      at_end: Boolean(location.atEnd)
    };
  };
  const report = location => {
    lastLocation = location;
    if (!measuringPages) dioxus.send(pageMetric(location));
  };
  rendition.on("relocated", report);
  rendition.on("displayError", error => dioxus.send({ error: String(error?.message || error) }));

  // rendition.display() ist auf WebView2 vor dem anschließenden relocated-
  // Ereignis fertig. Die Seitenzahl darf deshalb erst aus genau diesem
  // Ereignis gelesen werden, nicht aus einer möglicherweise alten Ansicht.
  const displayAt = target => new Promise((resolve, reject) => {
    let settled = false;
    const finish = (location, error) => {
      if (settled) return;
      settled = true;
      clearTimeout(timeout);
      rendition.off("relocated", located);
      if (error) reject(error);
      else if (location?.start) resolve(location);
      else reject(new Error("EPUB-Position konnte nicht bestimmt werden."));
    };
    const located = location => finish(location);
    const timeout = setTimeout(() => finish(rendition.currentLocation()), 5000);
    rendition.on("relocated", located);
    rendition.display(target).catch(error => finish(null, error));
  });

  // epub.js meldet displayed.total nur für das aktuell gerenderte Spine-Element.
  // Für eine Seitennummer über das ganze Buch wird jedes Element einmal mit der
  // aktuellen Viewport-Größe paginiert. Währenddessen bleiben Zwischenpositionen
  // unsichtbar; anschließend wird die ursprüngliche CFI wiederhergestellt.
  const measurePages = async () => {
    if (measuringPages) return;
    const restore = lastLocation?.start?.cfi || rendition.currentLocation()?.start?.cfi;
    const sections = book.spine.spineItems || [];
    if (!sections.length) return;
    measuringPages = true;
    try {
      const counts = [];
      for (const section of sections) {
        const location = await displayAt(section.href);
        counts[location.start.index] = Math.max(1, location.start.displayed?.total || 1);
      }
      pageCounts = Array.from({ length: sections.length }, (_, index) => counts[index] || 1);
      await displayAt(restore || config.cfi || sections[0].href);
    } catch (error) {
      dioxus.send({ error: `Gesamtseitenzahl konnte nicht ermittelt werden: ${error?.message || error}` });
    } finally {
      measuringPages = false;
    }
    // relocated kann bei WebView2 erst nach display() eintreffen. Die im
    // Handler zwischengespeicherte Position ist deshalb verlässlicher als
    // ein weiterer synchroner currentLocation()-Aufruf.
    const location = rendition.currentLocation() || lastLocation;
    if (location) dioxus.send(pageMetric(location));
  };
  const schedulePageMeasurement = () => {
    clearTimeout(paginationTimer);
    paginationTimer = setTimeout(() => { measurePages(); }, 100);
  };

  window.leafEpub = {
    book,
    rendition,
    next: () => rendition.next(),
    prev: () => rendition.prev(),
    display: target => rendition.display(target),
    appearance: (size, dark, font) => {
      rendition.themes.fontSize(`${size}px`);
      rendition.themes.font(font || "Georgia, 'Times New Roman', serif");
      rendition.themes.select(dark ? "dark" : "light");
      schedulePageMeasurement();
    },
    destroy: () => {
      rendition.destroy();
      book.destroy();
      clearTimeout(paginationTimer);
      window.leafEpub = null;
    }
  };
  const navigation = await book.loaded.navigation;
  const flatten = (items, result = []) => {
    for (const item of items || []) {
      if (item.label && item.href) result.push({ label: item.label.trim(), href: item.href });
      flatten(item.subitems, result);
    }
    return result;
  };
  const toc = flatten(navigation.toc);
  if (toc.length) dioxus.send({ toc });
  await displayAt(config.cfi || undefined);
  await measurePages();
};
