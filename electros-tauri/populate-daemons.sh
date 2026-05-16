#!/bin/bash
# Downloads elemento client daemons into electros-tauri/electros-daemons/ by invoking
# the repository populate script from this directory (does not modify populate_daemons.sh).
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

export GITHUB_TOKEN="${GITHUB_TOKEN:-${CI_TOKEN:-}}"

if [ -z "${GITHUB_TOKEN:-}" ]; then
  echo "GITHUB_TOKEN or CI_TOKEN not set — skipping daemon download."
  echo "Place daemon binaries under: $SCRIPT_DIR/electros-daemons/{mac|linux|win}/{arch}/"
  exit 0
fi

cd "$SCRIPT_DIR"
bash "$REPO_ROOT/populate_daemons.sh"
