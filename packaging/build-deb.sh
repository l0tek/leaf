#!/bin/sh
set -eu

project_dir=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
cd "$project_dir"

for tool in cargo dpkg dpkg-deb dpkg-shlibdeps sha256sum; do
    command -v "$tool" >/dev/null || { printf 'Build-Werkzeug fehlt: %s\n' "$tool" >&2; exit 1; }
done

version=$(sed -n 's/^version = "\(.*\)"$/\1/p' Cargo.toml | head -n 1)
[ -n "$version" ] || { echo 'Paketversion konnte nicht ermittelt werden.' >&2; exit 1; }
architecture=$(dpkg --print-architecture)
[ "$architecture" = amd64 ] || { echo 'Der Debian-Build wird derzeit nur für amd64 unterstützt.' >&2; exit 1; }

cargo build --locked --release

stage=$(mktemp -d "${TMPDIR:-/tmp}/leaf-deb.XXXXXX")
trap 'rm -rf -- "$stage"' EXIT HUP INT TERM
mkdir -p "$stage/DEBIAN" "$stage/usr/bin" \
    "$stage/usr/share/applications" "$stage/usr/share/icons/hicolor/512x512/apps" \
    "$stage/usr/share/doc/leaf"

install -m 755 target/release/leaf "$stage/usr/bin/leaf"
install -m 644 packaging/leaf.desktop "$stage/usr/share/applications/leaf.desktop"
install -m 644 assets/leaf-icon.png "$stage/usr/share/icons/hicolor/512x512/apps/leaf-icon.png"
install -m 644 README.md "$stage/usr/share/doc/leaf/README.md"

# dpkg-shlibdeps benötigt auch beim Abfragen über -O eine Debian-
# Kontrollumgebung. Sie bleibt im temporären Staging-Verzeichnis.
mkdir -p "$stage/shlibdeps/debian"
printf 'Source: leaf\nSection: text\nPriority: optional\nMaintainer: Leaf\nStandards-Version: 4.7.0\n' \
    > "$stage/shlibdeps/debian/control"
dependencies=$(cd "$stage/shlibdeps" && dpkg-shlibdeps -O -e"$stage/usr/bin/leaf" | sed -n 's/^shlibs:Depends=//p')
rm -rf -- "$stage/shlibdeps"
{
    printf 'Package: leaf\n'
    printf 'Version: %s\n' "$version"
    printf 'Section: text\n'
    printf 'Priority: optional\n'
    printf 'Architecture: %s\n' "$architecture"
    [ -z "$dependencies" ] || printf 'Depends: %s\n' "$dependencies"
    printf 'Maintainer: Leaf\n'
    printf 'Description: Ein ruhiger EPUB-Reader\n'
    printf ' Leaf ist ein lokaler EPUB-Reader auf Basis von Rust und Dioxus.\n'
} > "$stage/DEBIAN/control"

mkdir -p dist
package="dist/leaf_${version}_${architecture}.deb"
dpkg-deb --root-owner-group --build "$stage" "$package"
(cd dist && sha256sum "$(basename "$package")" > "$(basename "$package").sha256")
printf 'Erstellt: %s\n' "$package"
