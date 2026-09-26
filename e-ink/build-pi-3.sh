#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
target="aarch64-unknown-linux-gnu"

if command -v cross >/dev/null 2>&1; then
    cross_command="$(command -v cross)"
elif [[ -x "$HOME/.cargo/bin/cross" ]]; then
    cross_command="$HOME/.cargo/bin/cross"
else
    printf 'cross fehlt. Installieren mit: cargo install cross --locked\n' >&2
    exit 1
fi

(cd "$script_dir" && "$cross_command" build --release --target "$target")
printf 'ARM64-Binaerdatei: %s\n' "$script_dir/target/$target/release/leaf-e-ink-test"
