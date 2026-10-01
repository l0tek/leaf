<?php
declare(strict_types=1);

// Leaf-Synchronisierung: nur Lesestände, keine Bücher oder Nutzerdaten.
// Der 256-Bit-Schlüssel ist die Berechtigung für eine Gerätegruppe.
header('Content-Type: application/json; charset=utf-8');
header('Access-Control-Allow-Origin: *');
header('Access-Control-Allow-Methods: POST, OPTIONS');
header('Access-Control-Allow-Headers: Content-Type');
if ($_SERVER['REQUEST_METHOD'] === 'OPTIONS') {
    http_response_code(204);
    exit;
}
if ($_SERVER['REQUEST_METHOD'] !== 'POST') {
    http_response_code(405);
    echo json_encode(['ok' => false, 'error' => 'Nur POST ist erlaubt.']);
    exit;
}

function reply(array $value, int $status = 200): never {
    http_response_code($status);
    echo json_encode($value, JSON_UNESCAPED_SLASHES);
    exit;
}
function valid_key(mixed $key): bool {
    return is_string($key) && preg_match('/^[a-f0-9]{64}$/D', $key) === 1;
}
function valid_bookmark(mixed $bookmark): bool {
    return is_array($bookmark)
        && isset($bookmark['id'], $bookmark['updated_at'])
        && is_string($bookmark['id']) && strlen($bookmark['id']) <= 128
        && is_int($bookmark['updated_at']) && $bookmark['updated_at'] >= 0;
}

$request = json_decode(file_get_contents('php://input'), true);
if (!is_array($request) || !isset($request['action']) || !is_string($request['action'])) {
    reply(['ok' => false, 'error' => 'Ungültige Anfrage.'], 400);
}

$directory = __DIR__ . '/data';
if (!is_dir($directory) && !mkdir($directory, 0700, true) && !is_dir($directory)) {
    reply(['ok' => false, 'error' => 'Datenspeicher nicht verfügbar.'], 500);
}
try {
    $db = new PDO('sqlite:' . $directory . '/leaf.sqlite', null, null, [
        PDO::ATTR_ERRMODE => PDO::ERRMODE_EXCEPTION,
        PDO::ATTR_DEFAULT_FETCH_MODE => PDO::FETCH_ASSOC,
    ]);
    $db->exec('PRAGMA journal_mode=WAL');
    $db->exec('CREATE TABLE IF NOT EXISTS bookmarks (
        sync_key TEXT NOT NULL, book_id TEXT NOT NULL, payload TEXT NOT NULL,
        updated_at INTEGER NOT NULL, PRIMARY KEY (sync_key, book_id)
    )');
} catch (Throwable) {
    reply(['ok' => false, 'error' => 'Datenspeicher nicht verfügbar.'], 500);
}

if ($request['action'] === 'create') {
    reply(['ok' => true, 'key' => bin2hex(random_bytes(32)), 'bookmarks' => []]);
}
if (!in_array($request['action'], ['pull', 'push'], true) || !valid_key($request['key'] ?? null)) {
    reply(['ok' => false, 'error' => 'Ungültiger Synchronisationsschlüssel.'], 400);
}
$key = $request['key'];

if ($request['action'] === 'push') {
    $bookmarks = $request['bookmarks'] ?? null;
    if (!is_array($bookmarks) || count($bookmarks) > 500) {
        reply(['ok' => false, 'error' => 'Ungültige Lesestände.'], 400);
    }
    $upsert = $db->prepare('INSERT INTO bookmarks (sync_key, book_id, payload, updated_at)
        VALUES (:key, :id, :payload, :updated)
        ON CONFLICT(sync_key, book_id) DO UPDATE SET payload = excluded.payload, updated_at = excluded.updated_at
        WHERE excluded.updated_at > bookmarks.updated_at');
    foreach ($bookmarks as $bookmark) {
        if (!valid_bookmark($bookmark)) {
            reply(['ok' => false, 'error' => 'Ungültiger Lesestand.'], 400);
        }
        $payload = json_encode($bookmark, JSON_UNESCAPED_SLASHES);
        if ($payload === false || strlen($payload) > 8192) {
            reply(['ok' => false, 'error' => 'Lesestand ist zu groß.'], 400);
        }
        $upsert->execute([':key' => $key, ':id' => $bookmark['id'], ':payload' => $payload, ':updated' => $bookmark['updated_at']]);
    }
}

$select = $db->prepare('SELECT payload FROM bookmarks WHERE sync_key = :key ORDER BY book_id');
$select->execute([':key' => $key]);
$bookmarks = [];
foreach ($select as $row) {
    $bookmark = json_decode($row['payload'], true);
    if (is_array($bookmark)) $bookmarks[] = $bookmark;
}
reply(['ok' => true, 'bookmarks' => $bookmarks]);
