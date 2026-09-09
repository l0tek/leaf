#!/bin/sh
set -eu
project_dir=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
cd "$project_dir"
# Requires cargo-xwin, clang-cl, lld-link and the Rust Windows MSVC target.
XWIN_ARCH=x86_64 RUSTFLAGS="${RUSTFLAGS:-} -C target-feature=+crt-static"
export XWIN_ARCH RUSTFLAGS
if [ -z "${LLVM_RC:-}" ] && [ -n "${ANDROID_NDK_HOME:-}" ] \
    && [ -x "$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin/llvm-rc" ]; then
    LLVM_RC="$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin/llvm-rc"
    export LLVM_RC
fi
cargo xwin build --locked --release --target x86_64-pc-windows-msvc
mkdir -p dist/windows-x64
cp target/x86_64-pc-windows-msvc/release/leaf.exe dist/windows-x64/Leaf.exe
cp packaging/WINDOWS.txt dist/windows-x64/README.txt
python3 - <<'PY'
import hashlib
from pathlib import Path
from zipfile import ZipFile, ZIP_DEFLATED
folder = Path('dist/windows-x64')
with ZipFile('dist/Leaf-windows-x64.zip', 'w', ZIP_DEFLATED) as archive:
    for name in ('Leaf.exe', 'README.txt'):
        archive.write(folder / name, name)
p = folder / 'Leaf.exe'
(folder / 'SHA256SUMS.txt').write_text(hashlib.sha256(p.read_bytes()).hexdigest() + '  Leaf.exe\n')
PY
