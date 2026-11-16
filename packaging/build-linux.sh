#!/bin/sh
set -eu
project_dir=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
cd "$project_dir"
[ "${LEAF_VERSION_BUMPED:-0}" = 1 ] || "$project_dir/packaging/bump-version.sh"
cargo build --locked --release
printf 'Erstellt: target/release/leaf\n'
