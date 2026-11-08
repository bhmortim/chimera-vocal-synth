#!/usr/bin/env bash
# Box side of deploy/check.sh: build a working tree that arrives as a tar on stdin,
# with the pipeline's Dockerfile, then throw the tree away. Shares the pipeline's
# cargo and bun caches, so it's as incremental as a CI run.
#   check-tree.sh build|test < tree.tar
set -euo pipefail
target=${1:-test}
home=$(cd "$(dirname "$0")/.." && pwd) # ~/chimera
mkdir -p "$home/checks"
tree=$(mktemp -d "$home/checks/tree.XXXXXX")
trap 'rm -rf "$tree"' EXIT
tar -C "$tree" -xf -
docker build --progress=plain -f "$home/build/Dockerfile" --target "$target" --output type=cacheonly "$tree"
