#!/bin/sh
set -eu
project_dir=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
cd "$project_dir"
for tool in curl openssl osslsigncode makensis python3; do
    command -v "$tool" >/dev/null || { printf 'Build-Werkzeug fehlt: %s\n' "$tool" >&2; exit 1; }
done
[ -f dist/windows-x64/Leaf.exe ] || { echo 'Bitte zuerst packaging/build-windows.sh ausführen.' >&2; exit 1; }
mkdir -p target/installer-deps dist
bootstrapper=target/installer-deps/MicrosoftEdgeWebview2Setup.exe
curl --fail --location --proto '=https' --proto-redir '=https' --retry 3 \
    'https://go.microsoft.com/fwlink/p/?LinkId=2124703' -o "$bootstrapper.download"
# Microsoft roots from https://www.microsoft.com/pkiops/docs/repository.htm.
# These are used only for build verification, never installed into a system store.
for certificate in MicRooCerAut_2010-06-23 MicRooCerAut2011_2011_03_22; do
    curl --fail --location --proto '=https' --proto-redir '=https' --retry 3 \
        "https://www.microsoft.com/pki/certs/$certificate.crt" \
        -o "target/installer-deps/$certificate.crt"
done
openssl x509 -inform DER -in target/installer-deps/MicRooCerAut_2010-06-23.crt -out target/installer-deps/microsoft-roots.pem
openssl x509 -inform DER -in target/installer-deps/MicRooCerAut2011_2011_03_22.crt >> target/installer-deps/microsoft-roots.pem
osslsigncode verify -index 0 \
    -CAfile target/installer-deps/microsoft-roots.pem \
    -TSA-CAfile target/installer-deps/microsoft-roots.pem \
    -in "$bootstrapper.download" > target/installer-deps/signature-verification.log 2>&1 || {
        cat target/installer-deps/signature-verification.log >&2
        exit 1
    }
mv "$bootstrapper.download" "$bootstrapper"
makensis -WX "-DPROJECT_DIR=$project_dir" packaging/leaf-installer.nsi
python3 - <<'PY'
import hashlib
from pathlib import Path
p = Path('dist/Leaf-Setup-x64.exe')
p.with_suffix('.exe.sha256').write_text(hashlib.sha256(p.read_bytes()).hexdigest() + '  ' + p.name + '\n')
PY
