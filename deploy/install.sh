#!/usr/bin/env bash
# Install or update the CHIMERA pipeline on the build box from this checkout's deploy/.
#   deploy/install.sh            copy deploy/ to quidnug-docker:~/chimera, (re)start ci + cloudflared
#   deploy/install.sh --rebuild  the same, then rebuild main now (after editing build/)
# Nothing builds locally. The box's .env and cloudflared/credentials.json are never
# overwritten.
set -euo pipefail
BOX=${BOX:-quidnug-docker}
here=$(cd "$(dirname "$0")" && pwd)

tar -C "$here" --exclude=./.env --exclude='./cloudflared/*.json' --exclude='./*.cmd' -cf - . |
  ssh "$BOX" 'mkdir -p ~/chimera/ci-status && tar -C ~/chimera -xf - &&
    cd ~/chimera && { [ -f .env ] || cp .env.example .env; }'

ssh "$BOX" 'cd ~/chimera &&
  if [ ! -f cloudflared/credentials.json ]; then
    echo "missing ~/chimera/cloudflared/credentials.json (the tunnel secret); see deploy/README.md" >&2; exit 1
  fi &&
  docker compose up -d --build ci cloudflared'

if [ "${1:-}" = --rebuild ]; then ssh "$BOX" 'touch ~/chimera/rebuild'; fi
echo "site: https://chimera.round.rocks   pipeline: https://chimera.round.rocks/_ci/"
