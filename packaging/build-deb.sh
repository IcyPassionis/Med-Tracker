#!/usr/bin/env bash
# Builds an installable Debian package from the current working tree.
#
# Usage: packaging/build-deb.sh
# Output: dist/med-tracker_<version>_<arch>.deb
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

APP="med-tracker"
BINARY="$ROOT/target/release/Med-Tracker"
VERSION="$(sed -n 's/^version[[:space:]]*=[[:space:]]*"\(.*\)"/\1/p' Cargo.toml | head -n1)"
ARCH="$(dpkg --print-architecture)"
DIST="$ROOT/dist"
STAGE="$DIST/${APP}_${VERSION}_${ARCH}"

if [[ -z "$VERSION" ]]; then
  echo "Could not read the version from Cargo.toml" >&2
  exit 1
fi

for tool in cargo dpkg dpkg-deb dpkg-shlibdeps; do
  if ! command -v "$tool" >/dev/null 2>&1; then
    echo "Required tool not found: $tool" >&2
    exit 1
  fi
done

if [[ "${MED_TRACKER_SKIP_BUILD:-0}" != "1" ]]; then
  cargo build --release --locked
fi

if [[ ! -x "$BINARY" ]]; then
  echo "Missing $BINARY; run 'cargo build --release' first." >&2
  exit 1
fi

if command -v desktop-file-validate >/dev/null 2>&1; then
  desktop-file-validate packaging/med-tracker.desktop
fi

rm -rf "$STAGE"
install -d \
  "$STAGE/DEBIAN" \
  "$STAGE/usr/bin" \
  "$STAGE/usr/lib/$APP/icons" \
  "$STAGE/usr/lib/$APP/audio" \
  "$STAGE/usr/share/applications" \
  "$STAGE/usr/share/icons/hicolor/scalable/apps" \
  "$STAGE/usr/share/icons/hicolor/256x256/apps" \
  "$STAGE/usr/share/licenses/$APP"

install -m755 "$BINARY" "$STAGE/usr/lib/$APP/Med-Tracker"
install -m644 icons/*.png icons/*.svg "$STAGE/usr/lib/$APP/icons/"
install -m644 audio/*.wav "$STAGE/usr/lib/$APP/audio/"
ln -s "/usr/lib/$APP/Med-Tracker" "$STAGE/usr/bin/$APP"
install -m644 packaging/med-tracker.desktop "$STAGE/usr/share/applications/$APP.desktop"
install -m644 icons/med-tracker.svg "$STAGE/usr/share/icons/hicolor/scalable/apps/$APP.svg"
install -m644 icons/med-tracker.png "$STAGE/usr/share/icons/hicolor/256x256/apps/$APP.png"
install -m644 LICENSE "$STAGE/usr/share/licenses/$APP/LICENSE"

# Let dpkg compute the shared-library dependencies (this also gets Ubuntu's
# t64 package names such as libgtk-3-0t64 right).
SHLIB_DIR="$(mktemp -d)"
trap 'rm -rf "$SHLIB_DIR"' EXIT
mkdir -p "$SHLIB_DIR/debian"
cat >"$SHLIB_DIR/debian/control" <<EOF
Source: $APP

Package: $APP
Architecture: any
Maintainer: local <local@localhost>
Description: local package build
 local package build
EOF
DEPENDS="$(cd "$SHLIB_DIR" && dpkg-shlibdeps -O "$BINARY" 2>/dev/null | sed -n 's/^shlibs:Depends=//p')"
if [[ -z "$DEPENDS" ]]; then
  echo "dpkg-shlibdeps could not determine dependencies" >&2
  exit 1
fi

INSTALLED_SIZE="$(du -sk "$STAGE/usr" | cut -f1)"
MAINTAINER="$(git config user.name 2>/dev/null || true)"
MAINTAINER_EMAIL="$(git config user.email 2>/dev/null || true)"
if [[ -n "$MAINTAINER" && -n "$MAINTAINER_EMAIL" ]]; then
  MAINTAINER="$MAINTAINER <$MAINTAINER_EMAIL>"
else
  MAINTAINER="Med Tracker Maintainers <noreply@example.com>"
fi

cat >"$STAGE/DEBIAN/control" <<EOF
Package: $APP
Version: $VERSION
Architecture: $ARCH
Maintainer: $MAINTAINER
Installed-Size: $INSTALLED_SIZE
Depends: $DEPENDS
Recommends: xdg-desktop-portal, libvulkan1
Section: utils
Priority: optional
Homepage: https://github.com/IcyPassionis/Med-Tracker
Description: Desktop medication tracker with schedules and reminders
 Med-Tracker is a small Rust/Iced desktop application that tracks medication
 schedules, doses and refills. It provides alarms with sound, desktop
 notifications, calendar views and a system tray icon for X11 and Wayland.
EOF

OUTPUT="$DIST/${APP}_${VERSION}_${ARCH}.deb"
rm -f "$OUTPUT"
dpkg-deb --root-owner-group --build "$STAGE" "$OUTPUT" >/dev/null

echo "Built $OUTPUT"
( cd "$DIST" && sha256sum "$(basename "$OUTPUT")" >"$(basename "$OUTPUT").sha256" )
dpkg-deb --info "$OUTPUT" | sed -n '1,12p'
