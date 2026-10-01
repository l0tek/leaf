# Leaf-Sync-Endpunkt

Dieses Verzeichnis wird als `https://mnemonic.guru/leaf/` bereitgestellt. Es
benötigt PHP 8.1 oder neuer mit aktivierter Erweiterung `pdo_sqlite`.

1. `index.php` in das Web-Root-Verzeichnis `leaf/` hochladen.
2. Dem Webserver Schreibrechte ausschließlich für `leaf/data/` geben. Das
   Verzeichnis wird beim ersten Aufruf mit Modus 0700 angelegt.
3. HTTPS erzwingen. Die API nimmt nur JSON-POST-Anfragen an.

Der Sync-Schlüssel ist ein zufällig erzeugtes Geheimnis mit 256 Bit. Wer ihn
kennt, kann die Lesestände dieser Gerätegruppe lesen und ändern; er gehört daher
nicht in Screenshots oder öffentliche Texte. Es werden nur Buchkennung,
Leseposition, Darstellungseinstellung und Änderungszeit gespeichert. EPUB-Dateien
verbleiben auf den Geräten.
