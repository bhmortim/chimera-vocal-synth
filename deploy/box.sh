#!/usr/bin/env bash
# Run any command against THIS working tree (uncommitted changes included) on the build
# box, in the pipeline's toolchain (Rust + wasm32 + Bun), with a persistent dev copy and
# cargo cache there, so repeated runs are incremental. Nothing builds locally.
#   deploy/box.sh 'cargo test -p chimera --test engine'
#   deploy/box.sh 'cargo run --release -q -p chimera --example bench'
#   deploy/box.sh 'bun install --frozen-lockfile && bun run typecheck'
set -euo pipefail
BOX=${BOX:-quidnug-docker}
[ $# -gt 0 ] || { echo "usage: deploy/box.sh '<command>'" >&2; exit 2; }
root=$(git -C "$(dirname "$0")" rev-parse --show-toplevel)

git -C "$root" ls-files -z --cached --others --exclude-standard |
  tar -C "$root" --null --ignore-failed-read -T - -cf - |
  ssh "$BOX" "bash ~/chimera/ci/box-run.sh $(printf '%q' "$*")"
