#!/bin/bash
# Build Betacom.app từ mã nguồn. Cách dùng: bash build.sh
set -euo pipefail
cd "$(dirname "$0")"

APP="build/Betacom.app"
SOURCES=(Sources/*.swift)
MIN_OS="12.0"

if ! command -v swiftc >/dev/null 2>&1; then
    echo "Chưa có công cụ build của Apple."
    echo "Hãy chạy lệnh sau, bấm Install, đợi cài xong rồi chạy lại: bash build.sh"
    echo
    echo "    xcode-select --install"
    exit 1
fi

echo "==> Dọn bản build cũ"
rm -rf build
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"

echo "==> Biên dịch cho máy Intel"
swiftc -O -target "x86_64-apple-macos$MIN_OS" -o build/betacom-x86_64 "${SOURCES[@]}"

echo "==> Biên dịch cho chip Apple (M1, M2…)"
if swiftc -O -target "arm64-apple-macos$MIN_OS" -o build/betacom-arm64 "${SOURCES[@]}" 2>/dev/null; then
    lipo -create build/betacom-x86_64 build/betacom-arm64 -output "$APP/Contents/MacOS/Betacom"
    echo "    Đã tạo bản chạy được trên cả Intel và chip Apple"
else
    cp build/betacom-x86_64 "$APP/Contents/MacOS/Betacom"
    echo "    Bỏ qua (máy này chỉ build được bản Intel — vẫn chạy bình thường trên máy của bạn)"
fi

echo "==> Đóng gói"
cp Resources/Info.plist "$APP/Contents/Info.plist"

ICONSET="build/AppIcon.iconset"
mkdir -p "$ICONSET"
for size in 16 32 128 256 512; do
    sips -z $size $size Resources/AppIcon.png --out "$ICONSET/icon_${size}x${size}.png" >/dev/null
    double=$((size * 2))
    sips -z $double $double Resources/AppIcon.png --out "$ICONSET/icon_${size}x${size}@2x.png" >/dev/null
done
iconutil -c icns "$ICONSET" -o "$APP/Contents/Resources/AppIcon.icns"

# Ký "ad-hoc" để macOS cho chạy (không cần tài khoản Apple Developer).
codesign --force --sign - "$APP" >/dev/null 2>&1 || true

ditto -c -k --keepParent "$APP" build/Betacom-mac.zip
rm -rf "$ICONSET" build/betacom-x86_64 build/betacom-arm64

echo
echo "Xong! Ứng dụng nằm ở: $(pwd)/$APP"
echo "Mở thử:  open \"$APP\""
echo "Cài vào Applications:  cp -R \"$APP\" /Applications/"
