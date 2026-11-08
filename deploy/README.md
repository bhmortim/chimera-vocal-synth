# CHIMERA pipeline: build on quidnug-docker, serve through a Cloudflare tunnel

Push to `main` on [bhmortim/chimera-vocal-synth](https://github.com/bhmortim/chimera-vocal-synth)
and, a minute or two later, it's live at **https://chimera.round.rocks**. Nothing builds on the
dev machine: compiling, bundling and testing all happen in Docker on quidnug-docker, and the
caches live there too.

- Site: https://chimera.round.rocks (`/version.json` names the commit it serves)
- Pipeline status and build logs: https://chimera.round.rocks/_ci/

## What happens on a push

`chimera-ci` (a container on the box) checks GitHub every 20 s. For each new tip of `main`:

1. **checkout**: the commit, into `~/chimera/src`
2. **build**: `bun run build` (wasm engine, typecheck, Vite bundle) inside
   [build/Dockerfile](build/Dockerfile), baked into an nginx image `chimera-web:<sha>`
3. **deploy**: `chimera-web` is swapped to the new image, then checked: it must serve this
   commit's `version.json`, on the box and through the tunnel
4. **test**: `bun run test` (wasm build, `cargo test --workspace`, `bun test tests`) in the same
   toolchain, after the deploy, so a slow test run never holds back a deploy. A newer push
   cancels a test run that's still going.

A failed build or deploy leaves the previous build serving. Cargo's registry and `target/`
and Bun's cache are BuildKit cache mounts on the box, so rebuilds are incremental.

## Day to day

- **Iterate:** commit, `git push`, watch `/_ci/`, reload the site.
- **Check a change before pushing** (uncommitted changes included), still on the box, from
  PowerShell or cmd:
  ```powershell
  deploy\check.cmd          # bun run build + bun run test
  deploy\check.cmd build    # just the build
  ```
  From Git Bash: `deploy/check.sh`, `deploy/check.sh build`.
- **Rebuild main now** (for example after a flaky failure): `ssh quidnug-docker touch ~/chimera/rebuild`
- **Agent log:** `ssh quidnug-docker docker logs -f chimera-ci`
- **Roll back** to an earlier build (the newest five images are kept):
  ```powershell
  ssh quidnug-docker 'docker image ls chimera-web'
  ssh quidnug-docker 'docker tag chimera-web:<sha12> chimera-web:latest && cd ~/chimera && docker compose up -d web'
  ```
  The next push deploys over it as usual.

## Changing the pipeline

Edit the files here, then copy them to the box and restart the agent and tunnel:

```powershell
deploy\install.cmd             # after editing ci/, docker-compose.yml or cloudflared/config.yml
deploy\install.cmd --rebuild   # after editing build/Dockerfile: also rebuilds main
```

From Git Bash: `deploy/install.sh` (same flags). The `.cmd` launchers run the `.sh` scripts with
Git for Windows' bash on purpose: a bare `bash` in PowerShell is WSL's, which has no ssh setup
for the box.

The pipeline reads its Dockerfile from the box (`~/chimera/build/`), not from the commit, so
any commit builds, including upstream's.

## Where things live on the box (`~/chimera`)

| Path | What |
|---|---|
| `docker-compose.yml`, `.env` | the stack: `chimera-web`, `chimera-ci`, `chimera-cloudflared`; settings (copied from [.env.example](.env.example) on first install) |
| `build/` | the build definition |
| `ci/` | the agent ([ci/agent.sh](ci/agent.sh)) and the box side of `check.sh` |
| `cloudflared/` | tunnel `chimera` config and `credentials.json` (the secret; only on the box and in `~/.cloudflared` on the PC that created it) |
| `src/` | the agent's checkout |
| `ci-status/` | the status page, run records and logs, served at `/_ci/` |

## One-time setup (already done)

```powershell
cloudflared tunnel create chimera                          # writes $HOME\.cloudflared\<id>.json
cloudflared tunnel route dns chimera chimera.round.rocks    # CNAME -> <id>.cfargotunnel.com
scp $HOME/.cloudflared/<id>.json quidnug-docker:chimera/cloudflared/credentials.json
ssh quidnug-docker chmod 444 chimera/cloudflared/credentials.json
deploy\install.cmd
```

The site is public: anyone with the URL can open it and its `/_ci/` page. To make it private,
put a Cloudflare Access application in front of `chimera.round.rocks`.
