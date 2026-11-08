#!/usr/bin/env bash
# Build THIS working tree's site on the build box (uncommitted changes included) and serve it
# here at http://127.0.0.1:5320 through an SSH tunnel, until Ctrl+C. Localhost is a secure
# context, so audio works. Nothing builds locally, and the live site is untouched.
#   deploy/preview.sh            build, start, and hold the tunnel open
#   PORT=5321 deploy/preview.sh  another local port
set -euo pipefail
BOX=${BOX:-quidnug-docker}
PORT=${PORT:-5320}
root=$(git -C "$(dirname "$0")" rev-parse --show-toplevel)

git -C "$root" ls-files -z --cached --others --exclude-standard |
  tar -C "$root" --null --ignore-failed-read -T - -cf - |
  ssh "$BOX" "bash ~/chimera/ci/preview-run.sh $PORT"
echo "preview: http://127.0.0.1:$PORT   (Ctrl+C closes the tunnel; the box keeps the container until the next preview)"
exec ssh -N -o ExitOnForwardFailure=yes -L "127.0.0.1:$PORT:127.0.0.1:$PORT" "$BOX"
