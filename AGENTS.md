# Arbeiten an Leaf

- Zu Beginn einer Arbeitssitzung `PROJEKTSTAND.md` und `README.md` lesen.
- `git status` prüfen und vorhandene Benutzeränderungen erhalten.
- Aktuelle Benutzeraufträge haben Vorrang vor älteren Notizen im Projektstand.
- Nach wesentlichen Änderungen `PROJEKTSTAND.md` aktualisieren: Umsetzung,
  tatsächlich ausgeführte Prüfungen, Einschränkungen und nächste offene Schritte.
- Die Benutzeroberfläche und Projektdokumentation sind deutsch.
- `target/` und `dist/` sind generiert und bleiben außerhalb von Git.
- Erfolgreiche Cross-Builds nicht mit Laufzeittests auf Windows gleichsetzen.
- Jeder Auslieferungs-Build erhöht zwingend vor dem Bauen die Patch-Version.
  Für Linux, Windows, Windows-Setup und Android ist dafür
  `packaging/bump-version.sh` zu verwenden. `packaging/build-all.sh` erhöht
  einmal und erzeugt daraus alle drei Plattform-Ausgaben mit derselben Version.
  Diese Buildprozess-Regel darf nicht entfernt oder umgangen werden.
