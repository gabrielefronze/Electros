#!/bin/bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

echo "Setting up electros-tauri…"

# Symlinks to GUI assets (idempotent)
ln -sfn ../elemento-gui-new/electros "$SCRIPT_DIR/electros"
ln -sfn ../elemento-gui-new/electros.iconset "$SCRIPT_DIR/electros.iconset"

# Terminal UI for daemon logs (copy from electron package, read-only source)
if [ -d "$REPO_ROOT/electros-electron/terminal" ]; then
  mkdir -p "$SCRIPT_DIR/terminal"
  cp -R "$REPO_ROOT/electros-electron/terminal/"* "$SCRIPT_DIR/terminal/" 2>/dev/null || true
fi

# Custom titlebar (same sources as electros-electron)
if [ -d "$REPO_ROOT/electros-electron/titlebar" ]; then
  mkdir -p "$SCRIPT_DIR/titlebar"
  cp -R "$REPO_ROOT/electros-electron/titlebar/"* "$SCRIPT_DIR/titlebar/" 2>/dev/null || true
fi

# npm dependencies
cd "$SCRIPT_DIR"
npm install

# elemento-gui-new deps (for Vite during dev/build)
if [ -f "$REPO_ROOT/elemento-gui-new/package.json" ]; then
  echo "Installing elemento-gui-new dependencies…"
  (cd "$REPO_ROOT/elemento-gui-new" && npm install)
fi

# Rust toolchain hint
if ! command -v cargo >/dev/null 2>&1; then
  echo "Warning: Rust/cargo not found. Install from https://rustup.rs/"
else
  echo "Rust: $(rustc --version)"
fi

# Optional daemon binaries (does not modify root populate_daemons.sh)
if [ -x "$SCRIPT_DIR/populate-daemons.sh" ]; then
  echo "Populating electros-daemons for Tauri…"
  "$SCRIPT_DIR/populate-daemons.sh" || echo "Daemon populate skipped (set GITHUB_TOKEN to download)."
fi

echo "electros-tauri setup complete."
echo "Dev: start Vite in elemento-gui-new, then: cd electros-tauri && npm run dev"
