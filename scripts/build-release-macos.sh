#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo ""
echo "============================================================"
echo " tamux macOS release build"
echo "============================================================"
echo ""
echo "This wrapper builds native macOS Rust binaries plus DMG/zip artifacts via electron-builder."
echo "Use TAMUX_SIGN_IDENTITY for codesign and notarize separately if required."

exec "$SCRIPT_DIR/build-release.sh" "$@"
