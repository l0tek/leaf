#!/bin/sh
set -eu

project_dir=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
cd "$project_dir"

python3 - <<'PY'
import re
from pathlib import Path

cargo = Path("Cargo.toml")
source = cargo.read_text()
match = re.search(r'(?m)^version = "(\d+)\.(\d+)\.(\d+)"$', source)
if not match:
    raise SystemExit("Paketversion in Cargo.toml nicht gefunden.")
major, minor, patch = map(int, match.groups())
old = match.group(0).split('"')[1]
new = f"{major}.{minor}.{patch + 1}"
cargo.write_text(source[:match.start()] + f'version = "{new}"' + source[match.end():])

lock = Path("Cargo.lock")
source = lock.read_text()
pattern = r'(\[\[package\]\]\nname = "leaf"\nversion = ")' + re.escape(old) + r'(")'
source, count = re.subn(pattern, r'\g<1>' + new + r'\g<2>', source, count=1)
if count != 1:
    raise SystemExit("Leaf-Version in Cargo.lock nicht gefunden.")
lock.write_text(source)

nsi = Path("packaging/leaf-installer.nsi")
source = nsi.read_text()
source, app_count = re.subn(r'(!define APP_VERSION ")' + re.escape(old) + r'(")', r'\g<1>' + new + r'\g<2>', source, count=1)
source, vi_count = re.subn(r'(VIProductVersion ")' + re.escape(old) + r'\.0(")', r'\g<1>' + new + r'.0\g<2>', source, count=1)
if app_count != 1 or vi_count != 1:
    raise SystemExit("NSIS-Version nicht vollständig aktualisiert.")
nsi.write_text(source)

readme = Path("README.md")
source = readme.read_text()
source, count = re.subn(r'(Version )' + re.escape(old) + r'(?=:)', r'\g<1>' + new, source, count=1)
source = source.replace(f"Leaf-Setup-{old}-x64.exe", f"Leaf-Setup-{new}-x64.exe")
source = source.replace(f"Leaf-{old}-android-arm64.apk", f"Leaf-{new}-android-arm64.apk")
if count != 1:
    raise SystemExit("Versionsüberschrift in README.md nicht gefunden.")
readme.write_text(source)
print(f"Version erhöht: {old} -> {new}")
PY
