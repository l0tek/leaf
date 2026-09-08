#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."

: "${JAVA_HOME:?JAVA_HOME muss auf ein JDK 17 oder neuer zeigen}"
: "${ANDROID_HOME:?ANDROID_HOME muss auf das Android-SDK zeigen}"
: "${ANDROID_NDK_HOME:?ANDROID_NDK_HOME muss auf das installierte Android-NDK zeigen}"
export PATH="$JAVA_HOME/bin:$ANDROID_HOME/platform-tools:$PATH"

dx build --android --release --no-default-features --features mobile --target aarch64-linux-android

# Dioxus erstellt ohne Release-Schlüssel eine mit dem Android-Debugschlüssel
# signierte APK. Der enthaltene Rust-Code wird trotzdem im Release-Modus gebaut.
mkdir -p dist
apk="target/dx/leaf/release/android/app/app/build/outputs/apk/debug/app-debug.apk"
if [[ ! -f "$apk" ]]; then
    echo "Die erwartete Test-APK fehlt: $apk" >&2
    exit 1
fi
cp "$apk" dist/Leaf-android-arm64.apk
(cd dist && sha256sum Leaf-android-arm64.apk > Leaf-android-arm64.apk.sha256)
echo "Erstellt: dist/Leaf-android-arm64.apk"
