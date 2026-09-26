#!/usr/bin/env bash
set -euo pipefail

# Erst lokal für den 64-Bit-Pi bauen und nur das fertige Programm kopieren.
pi_host="l0tek@192.168.113.110"
remote_dir="leaf/e-ink"
script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
binary="$script_dir/target/aarch64-unknown-linux-gnu/release/leaf-e-ink-test"

"$script_dir/build-pi-3.sh"
ssh "$pi_host" "mkdir -p '$remote_dir'"
scp "$binary" "$pi_host:$remote_dir/leaf-e-ink-test"
ssh -t "$pi_host" "sudo '$remote_dir/leaf-e-ink-test'"
