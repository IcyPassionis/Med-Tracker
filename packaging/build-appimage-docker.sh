#!/usr/bin/env bash
# Builds the AppImage inside an Ubuntu 22.04 container so the result only
# requires glibc 2.35 (Linux Mint 21+, Ubuntu 22.04+, Debian 12+).
#
# Usage: packaging/build-appimage-docker.sh
# Output: dist/med-tracker-<version>-x86_64.AppImage (+ .sha256)
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

IMAGE="med-tracker-appimage:ubuntu22.04"
CARGO_VOLUME="med-tracker-appimage-cargo"

if ! command -v docker >/dev/null 2>&1; then
  echo "docker is required. Install it with: sudo apt install docker.io" >&2
  exit 1
fi

if ! docker image inspect "$IMAGE" >/dev/null 2>&1; then
  echo "Building $IMAGE (first run only)..." >&2
  docker build -f packaging/appimage.Dockerfile -t "$IMAGE" .
fi

mkdir -p "$ROOT/.tools" "$ROOT/dist" "$ROOT/target/appimage"

docker run --rm -i \
  --user "$(id -u):$(id -g)" \
  -e HOME=/tmp \
  -e CARGO_TARGET_DIR=/src/target/appimage \
  -v "$ROOT:/src" \
  -v "$CARGO_VOLUME:/usr/local/cargo/registry" \
  -w /src \
  "$IMAGE" \
  ./packaging/build-appimage.sh "$@"
