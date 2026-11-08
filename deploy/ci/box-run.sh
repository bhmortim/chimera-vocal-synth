#!/usr/bin/env bash
# Box side of deploy/box.sh: refresh ~/chimera/dev from the tar on stdin, keeping its
# target/ and node_modules/ so builds stay incremental, then run the command there in the
# pipeline's toolchain image, as brian, with cargo's home on a persistent volume.
#   box-run.sh '<command>' < tree.tar
set -euo pipefail
home=$(cd "$(dirname "$0")/.." && pwd) # ~/chimera
dev=$home/dev
mkdir -p "$dev"
find "$dev" -mindepth 1 -maxdepth 1 ! -name target ! -name node_modules -exec rm -rf {} +
tar -C "$dev" -xf -

docker build -q --target toolchain -t chimera-toolchain:local -f "$home/build/Dockerfile" "$home/build" >/dev/null
# cargo's registry, as brian; the toolchain itself stays in the image
docker volume create chimera-dev-cargo >/dev/null
docker run --rm -v chimera-dev-cargo:/cargo chimera-toolchain:local chown "$(id -u):$(id -g)" /cargo

exec docker run --rm --init -u "$(id -u):$(id -g)" \
  -e HOME=/tmp -e CARGO_HOME=/cargo -e CARGO_TERM_COLOR=never \
  -v chimera-dev-cargo:/cargo -v "$dev:/src" -w /src \
  chimera-toolchain:local bash -c "$1"
