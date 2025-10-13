#!/usr/bin/env bash
set -euo pipefail

if [ $# -lt 1 ]; then
  echo "Usage: $(basename "$0") <target-triple|native>" >&2
  exit 1
fi

TARGET="$1"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
BIN_NAME="new_demo"
ASSETS_DIR="${PROJECT_ROOT}/assets"
DIST_ROOT="${PROJECT_ROOT}/dist"

if [ ! -d "$ASSETS_DIR" ]; then
  echo "Assets directory not found at ${ASSETS_DIR}" >&2
  exit 1
fi

case "$TARGET" in
  native)
    TARGET_DIR="${PROJECT_ROOT}/target/release"
    ;;
  *)
    TARGET_DIR="${PROJECT_ROOT}/target/${TARGET}/release"
    ;;
esac

if [ ! -d "$TARGET_DIR" ]; then
  HOST_TRIPLE="$(rustc -vV | sed -n 's/^host: //p')"
  if [ "$TARGET" = "$HOST_TRIPLE" ]; then
    TARGET_DIR="${PROJECT_ROOT}/target/release"
  else
    echo "Compiled artifacts for target '${TARGET}' not found in ${TARGET_DIR}" >&2
    exit 1
  fi
fi

DIST_DIR="${DIST_ROOT}/${TARGET}"
rm -rf "${DIST_DIR}"
mkdir -p "${DIST_DIR}"

if [ "$TARGET" = "wasm32-unknown-unknown" ]; then
  WASM_PATH="${TARGET_DIR}/${BIN_NAME}.wasm"
  if [ ! -f "$WASM_PATH" ]; then
    echo "WASM binary not found at ${WASM_PATH}. Build it with 'cargo build --release --target wasm32-unknown-unknown' first." >&2
    exit 1
  fi

  if command -v wasm-bindgen >/dev/null 2>&1; then
    if wasm-bindgen "$WASM_PATH" --target web --out-dir "$DIST_DIR"; then
      if [ -f "${PROJECT_ROOT}/web/index.html" ]; then
        cp "${PROJECT_ROOT}/web/index.html" "${DIST_DIR}/"
      fi
    else
      echo "wasm-bindgen failed (version mismatch?). Falling back to raw .wasm export." >&2
      rm -rf "${DIST_DIR:?}/"*
      cp "$WASM_PATH" "$DIST_DIR/"
    fi
  else
    echo "wasm-bindgen not found; copied raw .wasm output. Install it with 'cargo install wasm-bindgen-cli --version 0.2.100' for a browser-ready package." >&2
    cp "$WASM_PATH" "$DIST_DIR/"
  fi
else
  BIN_PATH="${TARGET_DIR}/${BIN_NAME}"
  if [[ "$TARGET" == *"windows"* ]]; then
    BIN_PATH="${BIN_PATH}.exe"
  fi

  if [ ! -f "$BIN_PATH" ]; then
    echo "Binary not found at ${BIN_PATH}. Ensure the target '${TARGET}' has been built in release mode." >&2
    exit 1
  fi

  cp "$BIN_PATH" "$DIST_DIR/"
fi

rsync -a --delete "${ASSETS_DIR}/" "${DIST_DIR}/assets/"

if command -v zip >/dev/null 2>&1; then
  (cd "${DIST_ROOT}" && zip -rq "${TARGET}.zip" "${TARGET}")
else
  echo "zip not found; skipping archive step." >&2
fi

echo "Packaged ${TARGET} build into ${DIST_DIR}"
