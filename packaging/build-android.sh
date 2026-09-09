#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."

: "${JAVA_HOME:?JAVA_HOME muss auf ein JDK 17 oder neuer zeigen}"
: "${ANDROID_HOME:?ANDROID_HOME muss auf das Android-SDK zeigen}"
: "${ANDROID_NDK_HOME:?ANDROID_NDK_HOME muss auf das installierte Android-NDK zeigen}"
export PATH="$JAVA_HOME/bin:$ANDROID_HOME/platform-tools:$PATH"

android_project="target/dx/leaf/release/android/app"
android_app="$android_project/app"
res="$android_app/src/main/res"
# Ein vorheriger abgebrochener Lauf kann noch unsere PNG-Varianten enthalten.
# Dioxus erzeugt dann zunächst wieder seine WebP-Dateien; beides hätte denselben
# Android-Ressourcennamen. Nur generierte Zielressourcen werden hier entfernt.
for density in mdpi hdpi xhdpi xxhdpi xxxhdpi; do
    rm -f "$res/mipmap-$density/ic_launcher.png"
done

dx build --android --release --no-default-features --features mobile --target aarch64-linux-android

# Dioxus 0.7 bringt eigene Launcher-Ressourcen mit und berücksichtigt
# `bundle.icon` auf Android noch nicht. Die vorbereiteten Dichtevarianten
# ersetzen sie deshalb reproduzierbar vor dem abschließenden Gradle-Lauf.
rm -f "$res/mipmap-anydpi-v26/ic_launcher.xml"
for density in mdpi hdpi xhdpi xxhdpi xxxhdpi; do
    rm -f "$res/mipmap-$density/ic_launcher.webp"
    install -D -m 644 "assets/android-icon/mipmap-$density/ic_launcher.png" \
        "$res/mipmap-$density/ic_launcher.png"
done
(cd "$android_project" && ./gradlew --no-daemon assembleDebug)

# Dioxus erstellt ohne Release-Schlüssel eine mit dem Android-Debugschlüssel
# signierte APK. Der enthaltene Rust-Code wird trotzdem im Release-Modus gebaut.
mkdir -p dist
apk="$android_app/build/outputs/apk/debug/app-debug.apk"
if [[ ! -f "$apk" ]]; then
    echo "Die erwartete Test-APK fehlt: $apk" >&2
    exit 1
fi
cp "$apk" dist/Leaf-android-arm64.apk
(cd dist && sha256sum Leaf-android-arm64.apk > Leaf-android-arm64.apk.sha256)
echo "Erstellt: dist/Leaf-android-arm64.apk"
