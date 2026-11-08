# CHIMERA build for the quidnug-docker pipeline. deploy/ci/agent.sh runs it for
# every commit on main; deploy/check.sh runs it on a working tree.
#
# Context: a checkout of the repo. This file lives outside it (on the box at
# ~/chimera/build), so any commit builds, whether or not it carries deploy/.
#
#   toolchain  Rust + the wasm32 target + Bun: the repo's own build tools
#   deps       bun install, cached until package.json or bun.lock changes
#   build      bun run build: wasm engine, typecheck, Vite bundle -> /src/dist
#   test       bun run test: wasm build, cargo test --workspace, bun test tests
#   site       nginx serving dist: the image the pipeline deploys (default target)
#
# Cargo's registry and target dir live in BuildKit cache mounts on the box, so a
# rebuild recompiles only what changed, and nothing builds on a dev machine.

ARG RUST_IMAGE=rust:1.99-slim-trixie
ARG BUN_IMAGE=oven/bun:1.4.4
ARG NGINX_IMAGE=nginx:1.30-alpine

FROM ${BUN_IMAGE} AS bun

FROM ${RUST_IMAGE} AS toolchain
RUN rustup target add wasm32-unknown-unknown
COPY --from=bun /usr/local/bin/bun /usr/local/bin/bun
RUN ln -s bun /usr/local/bin/bunx && bun --version
WORKDIR /src

FROM toolchain AS deps
COPY package.json bun.lock ./
RUN --mount=type=cache,id=chimera-bun,target=/root/.bun/install/cache \
    bun install --frozen-lockfile

FROM deps AS build
COPY . .
RUN --mount=type=cache,id=chimera-cargo-registry,target=/usr/local/cargo/registry \
    --mount=type=cache,id=chimera-cargo-target,target=/src/target,sharing=locked \
    bun run build

FROM build AS test
RUN --mount=type=cache,id=chimera-cargo-registry,target=/usr/local/cargo/registry \
    --mount=type=cache,id=chimera-cargo-target,target=/src/target,sharing=locked \
    bun run test

FROM ${NGINX_IMAGE} AS site
ARG GIT_SHA=local
# Quoted delimiter: no Dockerfile variable expansion, so nginx's $uri survives.
COPY <<"EOF" /etc/nginx/conf.d/default.conf
# Vite fingerprints everything under /assets, so those cache for a year; every
# other path revalidates, so a fresh deploy shows on the next reload.
server {
    listen 80;
    server_name _;
    root /usr/share/nginx/html;
    index index.html;
    # Behind the tunnel the origin sees plain http; keep redirects relative.
    absolute_redirect off;

    gzip on;
    gzip_comp_level 6;
    gzip_min_length 1024;
    gzip_types application/wasm application/javascript text/javascript text/css application/json image/svg+xml text/plain;

    location / {
        try_files $uri $uri/ =404;
        add_header Cache-Control "no-cache" always;
    }

    location /assets/ {
        try_files $uri =404;
        add_header Cache-Control "public, max-age=31536000, immutable" always;
    }

    location = /version.json {
        add_header Cache-Control "no-store" always;
    }

    # Pipeline status page and build logs, written by chimera-ci.
    location = /_ci { return 301 /_ci/; }
    location /_ci/ {
        alias /srv/ci/;
        autoindex on;
        charset utf-8;
        default_type text/plain;
        add_header Cache-Control "no-store" always;
    }
}
EOF
COPY --from=build /src/dist /usr/share/nginx/html
RUN printf '{"sha":"%s","built":"%s"}\n' "$GIT_SHA" "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
      > /usr/share/nginx/html/version.json
