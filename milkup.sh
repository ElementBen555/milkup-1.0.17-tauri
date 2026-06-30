#!/bin/bash
# Milkup launcher
export DISPLAY=:0
exec "$(dirname "$0")/node_modules/.pnpm/electron@37.10.3/node_modules/electron/dist/electron" \
  "$(dirname "$0")/dist-electron/main/index.js" \
  --no-sandbox "$@"
