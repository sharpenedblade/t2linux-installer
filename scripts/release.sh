#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUT_DIR="${ROOT_DIR}/dist"
BIN_NAME="t2linux-installer"
BUILD_BINARY=1
BUILD_APPIMAGE=1
TEMP_ICON=0

cleanup() {
  if [[ "${TEMP_ICON}" -eq 1 ]]; then
    rm -f "${ROOT_DIR}/icon.png"
  fi
}
trap cleanup EXIT

usage() {
  cat <<'USAGE'
Build release artifacts for t2linux-installer.

Usage:
  scripts/release.sh [--out-dir <path>] [--no-binary] [--no-appimage]

Artifacts:
  - Native release binary: dist/t2linux-installer
  - AppImage (single-file app): dist/*.AppImage
USAGE
}

while [[ $# -gt 0 ]]; do
  case "$1" in
  --out-dir)
    OUT_DIR="$2"
    shift 2
    ;;
  --no-binary)
    BUILD_BINARY=0
    shift
    ;;
  --no-appimage)
    BUILD_APPIMAGE=0
    shift
    ;;
  -h | --help)
    usage
    exit 0
    ;;
  *)
    echo "Unknown argument: $1"
    usage
    exit 1
    ;;
  esac
done

mkdir -p "$OUT_DIR"
cd "$ROOT_DIR"

if [[ "$BUILD_BINARY" -eq 1 ]]; then
  echo "==> Building release binary"
  cargo build --release
  cp "target/release/${BIN_NAME}" "${OUT_DIR}/${BIN_NAME}"
  chmod +x "${OUT_DIR}/${BIN_NAME}"
  echo "Saved: ${OUT_DIR}/${BIN_NAME}"
fi

if [[ "$BUILD_APPIMAGE" -eq 1 ]]; then
  if ! cargo appimage --version >/dev/null 2>&1; then
    cat <<'ERR'
cargo-appimage is not installed.
Install it with:
  cargo install cargo-appimage
ERR
    exit 1
  fi

  if [[ ! -f "${ROOT_DIR}/icon.png" ]]; then
    # Minimal fallback icon so appimagetool can complete even when no project icon is set.
    printf '%s' 'iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAwMCAO+/aWQAAAAASUVORK5CYII=' \
      | base64 --decode >"${ROOT_DIR}/icon.png"
    TEMP_ICON=1
  fi

  echo "==> Building AppImage"
  cargo appimage

  APPIMAGE_PATH="$(
    find target/appimage target -maxdepth 1 -type f -name '*.AppImage' 2>/dev/null | head -n 1 || true
  )"
  if [[ -z "${APPIMAGE_PATH}" ]]; then
    echo "AppImage build completed but no .AppImage file was found in target/appimage"
    exit 1
  fi

  cp "${APPIMAGE_PATH}" "${OUT_DIR}/"
  chmod +x "${OUT_DIR}/$(basename "${APPIMAGE_PATH}")"
  echo "Saved: ${OUT_DIR}/$(basename "${APPIMAGE_PATH}")"
fi

echo "==> Done"
