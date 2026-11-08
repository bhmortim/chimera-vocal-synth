#!/usr/bin/env bash
# Build or test THIS working tree, uncommitted changes included, on the build box
# with the pipeline's Dockerfile. Nothing builds locally; nothing deploys.
#   deploy/check.sh          bun run build, then bun run test (the repo's full check)
#   deploy/check.sh build    bun run build only (wasm engine, typecheck, Vite bundle)
set -euo pipefail
BOX=${BOX:-quidnug-docker}
target=${1:-test}
case $target in build | test) ;; *) echo "usage: deploy/check.sh [build|test]" >&2; exit 2 ;; esac
root=$(git -C "$(dirname "$0")" rev-parse --show-toplevel)

# Tracked plus untracked-but-not-ignored files: what a commit of this tree would hold.
git -C "$root" ls-files -z --cached --others --exclude-standard |
  tar -C "$root" --null --ignore-failed-read -T - -cf - |
  ssh "$BOX" "bash ~/chimera/ci/check-tree.sh $target"
