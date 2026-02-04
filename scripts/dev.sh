#!/usr/bin/env bash
# Wrapper: run Aura from project root (for npm run dev or from scripts/)
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
exec "$SCRIPT_DIR/../aura" "$@"
