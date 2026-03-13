#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

if [[ "$(uname -s)" != "Darwin" ]]; then
  echo "apple goldens can only be refreshed on macOS" >&2
  exit 1
fi

UPDATE_GOLDENS=1 cargo test --test apple_golden -- --nocapture
