#!/bin/sh
set -eu
project_dir=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
data_dir=${XDG_DATA_HOME:-"$HOME/.local/share"}
bin_dir="$HOME/.local/bin"
if [ ! -x "$project_dir/target/release/leaf" ]; then
    echo 'Bitte zuerst cargo build --release ausführen.' >&2
    exit 1
fi
mkdir -p "$bin_dir" "$data_dir/applications" "$data_dir/icons/hicolor/scalable/apps"
install -m 755 "$project_dir/target/release/leaf" "$bin_dir/leaf"
install -m 644 "$project_dir/assets/leaf.svg" "$data_dir/icons/hicolor/scalable/apps/leaf.svg"
# Desktop launchers need an absolute executable path, independent of shell PATH.
python3 - "$project_dir/packaging/leaf.desktop" "$data_dir/applications/leaf.desktop" "$bin_dir/leaf" <<'PY'
import pathlib, sys
source, destination, binary = sys.argv[1:]
binary = binary.replace('\\', '\\\\').replace('"', '\\"').replace('`', '\\`').replace('$', '\\$').replace('%', '%%')
pathlib.Path(destination).write_text(pathlib.Path(source).read_text().replace('Exec=leaf', f'Exec="{binary}"'))
PY
printf 'Leaf installiert: %s\n' "$bin_dir/leaf"
