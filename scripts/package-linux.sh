#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

if [[ "$(uname -s)" != "Linux" ]]; then
  echo "Этот скрипт собирает Linux-пакет и должен запускаться на Linux." >&2
  exit 1
fi

VERSION="$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -n 1)"
ARCH="$(dpkg --print-architecture 2>/dev/null || echo amd64)"
PKG="dist/edu-scheduler_${VERSION}_${ARCH}"

echo "==> Сборка release (${ARCH})"
cargo build --release

rm -rf "$PKG"
mkdir -p \
  "$PKG/DEBIAN" \
  "$PKG/usr/bin" \
  "$PKG/usr/share/applications" \
  "$PKG/usr/share/icons/hicolor/128x128/apps" \
  "$PKG/usr/share/icons/hicolor/256x256/apps" \
  "$PKG/usr/share/icons/hicolor/512x512/apps"

install -m 0755 target/release/edu-scheduler "$PKG/usr/bin/edu-scheduler"
install -m 0644 packaging/linux/edu-scheduler.desktop "$PKG/usr/share/applications/edu-scheduler.desktop"
install -m 0644 assets/icons/icon-128.png "$PKG/usr/share/icons/hicolor/128x128/apps/edu-scheduler.png"
install -m 0644 assets/icons/icon-256.png "$PKG/usr/share/icons/hicolor/256x256/apps/edu-scheduler.png"
install -m 0644 assets/icons/icon-512.png "$PKG/usr/share/icons/hicolor/512x512/apps/edu-scheduler.png"

cat > "$PKG/DEBIAN/control" <<CONTROL
Package: edu-scheduler
Version: ${VERSION}
Section: education
Priority: optional
Architecture: ${ARCH}
Maintainer: v-aleks <https://github.com/v-aleks/edu-scheduler>
Depends: libc6 (>= 2.35), libxkbcommon0, libfontconfig1, libx11-6, libwayland-client0, libvulkan1 | libgl1
Description: Offline-first расписание преподавателей
 Локальное десктопное приложение для управления недельным расписанием
 преподавателей: занятия, перемены, недельный и месячный календарь.
CONTROL

echo "==> deb и tar.gz"
dpkg-deb --build --root-owner-group "$PKG" "dist/edu-scheduler_${VERSION}_${ARCH}.deb" >/dev/null

tar -C target/release -czf "dist/edu-scheduler-${VERSION}-linux-${ARCH}.tar.gz" edu-scheduler

echo "Готово:"
ls -lh dist/*.deb dist/*.tar.gz
