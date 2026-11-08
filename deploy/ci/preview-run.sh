#!/usr/bin/env bash
# Box side of deploy/preview.sh: build the tree arriving on stdin as the site image
# chimera-web:preview and (re)start it as chimera-preview, published on the box's loopback
# only (127.0.0.1:<port>), so it is reachable through an SSH tunnel and nowhere else.
#   preview-run.sh <port> < tree.tar
set -euo pipefail
port=${1:-5320}
home=$(cd "$(dirname "$0")/.." && pwd) # ~/chimera
mkdir -p "$home/checks"
tree=$(mktemp -d "$home/checks/preview.XXXXXX")
trap 'rm -rf "$tree"' EXIT
tar -C "$tree" -xf -
docker build -q -f "$home/build/Dockerfile" --target site --build-arg GIT_SHA=preview -t chimera-web:preview "$tree" >/dev/null
docker rm -f chimera-preview >/dev/null 2>&1 || true
docker run -d --name chimera-preview --restart no -p "127.0.0.1:$port:80" chimera-web:preview >/dev/null
echo "chimera-preview is up on the box at 127.0.0.1:$port"
