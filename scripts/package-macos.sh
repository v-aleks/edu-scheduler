#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

if [[ "$(uname -s)" != "Darwin" ]]; then
  echo "Этот скрипт собирает macOS-пакет и должен запускаться на macOS." >&2
  exit 1
fi

VERSION="$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -n 1)"
ARCH="$(uname -m)"
export MACOSX_DEPLOYMENT_TARGET="${MACOSX_DEPLOYMENT_TARGET:-11.0}"

echo "==> Сборка release (${ARCH}, macOS ${MACOSX_DEPLOYMENT_TARGET}+)"
cargo build --release

APP="dist/Edu Scheduler.app"
rm -rf "$APP"
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"

cp target/release/edu-scheduler "$APP/Contents/MacOS/edu-scheduler"
sed "s/__VERSION__/${VERSION}/g" packaging/macos/Info.plist > "$APP/Contents/Info.plist"
cp assets/icons/AppIcon.icns "$APP/Contents/Resources/AppIcon.icns"
chmod +x "$APP/Contents/MacOS/edu-scheduler"

echo "==> Ad-hoc подпись"
codesign --force --deep --sign - "$APP"

echo "==> zip с .app"
ditto -c -k --keepParent "$APP" "dist/EduScheduler-${VERSION}-macos-${ARCH}.app.zip"

echo "==> DMG"
DMG="dist/EduScheduler-${VERSION}-macos-${ARCH}.dmg"
rm -rf dist/staging
mkdir -p dist/staging
cp -R "$APP" dist/staging/
ln -s /Applications dist/staging/Applications

if hdiutil create \
  -volname "Edu Scheduler" \
  -srcfolder dist/staging \
  -ov -format UDZO \
  "$DMG" >/dev/null 2>&1; then
  echo "DMG создан."
elif diskutil image create from --format UDZO \
  --volumeName "Edu Scheduler" dist/staging "$DMG" >/dev/null 2>&1; then
  echo "DMG создан через diskutil."
else
  rm -f "$DMG"
  echo "ВНИМАНИЕ: не удалось создать DMG в этом окружении; остаётся .app.zip." >&2
fi

rm -rf dist/staging

echo "Готово:"
ls -lh dist/*.zip
[[ -f "$DMG" ]] && ls -lh "$DMG"
