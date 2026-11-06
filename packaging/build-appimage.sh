#!/usr/bin/env bash
# Builds a portable AppImage from the current working tree.
#
# The AppImage uses the target system's GTK3, ALSA and graphics drivers; only
# the application binary and its icons/audio assets are packaged.
#
# Usage: packaging/build-appimage.sh
# Output: dist/med-tracker-<version>-<arch>.AppImage (+ .sha256)
#
# Environment:
#   MED_TRACKER_SKIP_BUILD=1  Package the existing release binary.
#   CARGO_TARGET_DIR=...      Use a different Cargo target directory.
#   ARCH=...                  Target architecture label (default: x86_64).
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

APP="med-tracker"
ARCH="${ARCH:-x86_64}"
VERSION="$(sed -n 's/^version[[:space:]]*=[[:space:]]*"\(.*\)"/\1/p' Cargo.toml | head -n1)"

TARGET_DIR="${CARGO_TARGET_DIR:-target}"
if [[ "$TARGET_DIR" != /* ]]; then
  TARGET_DIR="$ROOT/$TARGET_DIR"
fi

BINARY="$TARGET_DIR/release/Med-Tracker"
DIST="$ROOT/dist"
WORK="$ROOT/.appimage-build"
APPDIR="$WORK/AppDir"
TOOLS="$ROOT/.tools"
APPIMAGETOOL="$TOOLS/appimagetool-$ARCH.AppImage"
APPIMAGETOOL_URL="https://github.com/AppImage/appimagetool/releases/download/continuous/appimagetool-$ARCH.AppImage"
OUTPUT="$DIST/${APP}-${VERSION}-${ARCH}.AppImage"

if [[ -z "$VERSION" ]]; then
  echo "Could not read the version from Cargo.toml" >&2
  exit 1
fi

for tool in cargo curl strip file; do
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

mkdir -p "$TOOLS" "$DIST" "$WORK"
if [[ ! -x "$APPIMAGETOOL" ]]; then
  echo "Downloading appimagetool..." >&2
  curl -fL --retry 3 -o "$APPIMAGETOOL.part" "$APPIMAGETOOL_URL"
  chmod +x "$APPIMAGETOOL.part"
  mv "$APPIMAGETOOL.part" "$APPIMAGETOOL"
fi

rm -rf "$APPDIR"
install -d \
  "$APPDIR/usr/bin/icons" \
  "$APPDIR/usr/bin/audio" \
  "$APPDIR/usr/share/applications" \
  "$APPDIR/usr/share/icons/hicolor/512x512/apps" \
  "$APPDIR/usr/share/icons/hicolor/scalable/apps" \
  "$APPDIR/usr/share/licenses/$APP"

# startup::use_resource_root() resolves icons/ and audio/ next to the
# executable, so mirror the .deb bundle layout inside usr/bin.
install -m755 "$BINARY" "$APPDIR/usr/bin/Med-Tracker"
strip --strip-unneeded "$APPDIR/usr/bin/Med-Tracker"
ln -s Med-Tracker "$APPDIR/usr/bin/$APP"
install -m644 icons/*.png icons/*.svg "$APPDIR/usr/bin/icons/"
install -m644 audio/*.wav "$APPDIR/usr/bin/audio/"

install -m644 packaging/med-tracker.desktop "$APPDIR/usr/share/applications/$APP.desktop"
install -m644 packaging/med-tracker.desktop "$APPDIR/$APP.desktop"
install -m644 icons/med-tracker.png "$APPDIR/usr/share/icons/hicolor/512x512/apps/$APP.png"
install -m644 icons/med-tracker.svg "$APPDIR/usr/share/icons/hicolor/scalable/apps/$APP.svg"
install -m644 icons/med-tracker.png "$APPDIR/$APP.png"
ln -sf "$APP.png" "$APPDIR/.DirIcon"
install -m644 LICENSE "$APPDIR/usr/share/licenses/$APP/LICENSE"

cat >"$APPDIR/AppRun" <<'EOF'
#!/bin/sh
HERE="$(dirname "$(readlink -f "$0")")"
exec "$HERE/usr/bin/Med-Tracker" "$@"
EOF
chmod 755 "$APPDIR/AppRun"

rm -f "$OUTPUT"
# appimagetool is itself an AppImage. Extracting it keeps the build working
# on hosts and containers where FUSE2 is unavailable. Older runtimes only
# understand the command-line form, so fall back to that.
if ! APPIMAGE_EXTRACT_AND_RUN=1 ARCH="$ARCH" VERSION="$VERSION" \
  "$APPIMAGETOOL" --no-appstream "$APPDIR" "$OUTPUT"; then
  echo "Retrying appimagetool with --appimage-extract-and-run..." >&2
  ARCH="$ARCH" VERSION="$VERSION" \
    "$APPIMAGETOOL" --appimage-extract-and-run --no-appstream "$APPDIR" "$OUTPUT"
fi

echo "Built $OUTPUT"
( cd "$DIST" && sha256sum "$(basename "$OUTPUT")" >"$(basename "$OUTPUT").sha256" )
file "$OUTPUT"
