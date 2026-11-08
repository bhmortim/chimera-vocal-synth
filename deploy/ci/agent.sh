#!/usr/bin/env bash
# chimera-ci: pull-based CI/CD for CHIMERA on the build box (quidnug-docker).
#
# Every $POLL_SECONDS it asks GitHub for the tip of $BRANCH. For a new tip, on
# first start, and whenever $CHIMERA_HOME/rebuild exists, it runs one pipeline:
#
#   checkout  the commit into $CHIMERA_HOME/src
#   build     the site image (wasm engine + Vite bundle in nginx) from build/Dockerfile
#   deploy    swap chimera-web to it, then check it serves this commit, locally and
#             through the tunnel
#   test      bun run test, in the background; a newer commit cancels it, so the
#             next deploy never waits on tests
#
# A failed build or deploy leaves the previous build serving. Every run writes a
# log and the status page under $CHIMERA_HOME/ci-status, served by the site at /_ci/.
set -uo pipefail

: "${CHIMERA_HOME:?CHIMERA_HOME must be set: the same absolute path on the host and in here}"
REPO_URL=${REPO_URL:-https://github.com/bhmortim/chimera-vocal-synth.git}
BRANCH=${BRANCH:-main}
POLL_SECONDS=${POLL_SECONDS:-20}
PUBLIC_URL=${PUBLIC_URL:-}
RUN_TESTS=${RUN_TESTS:-1}
KEEP_IMAGES=${KEEP_IMAGES:-5}
KEEP_RUNS=${KEEP_RUNS:-100}
BOX_NAME=${BOX_NAME:-the build box}

SRC=$CHIMERA_HOME/src
STATUS=$CHIMERA_HOME/ci-status
REPO_WEB=${REPO_URL%.git}
US=$'\x1f' # run-record field separator: not whitespace, so empty fields survive `read`
COMPOSE=(docker compose --project-directory "$CHIMERA_HOME" -f "$CHIMERA_HOME/docker-compose.yml")
BUILD=(docker build --progress=plain -f "$CHIMERA_HOME/build/Dockerfile")

mkdir -p "$STATUS/logs" "$STATUS/runs" "$HOME"

say() { printf '%s %s\n' "$(date -u +%FT%TZ)" "$*"; }
esc() { local s=${1//&/&amp;}; s=${s//</&lt;}; s=${s//>/&gt;}; printf '%s' "${s//\"/&quot;}"; }
dur() { if (($1 >= 60)); then printf '%dm%02ds' $(($1 / 60)) $(($1 % 60)); else printf '%ds' "$1"; fi; }
mark() {
  case $1 in
    ok) printf '<span class="ok">ok</span>' ;;
    failed) printf '<span class="bad">failed</span>' ;;
    running) printf '<span class="run">running</span>' ;;
    '') printf '<span class="mut">&middot;</span>' ;;
    *) printf '<span class="mut">%s</span>' "$1" ;;
  esac
}

# The current run. Its record is one line in $STATUS/runs/<id>:
#   id sha subject author started seconds build deploy tests
RUN_ID='' RUN_SHA='' RUN_SUBJECT='' RUN_AUTHOR='' RUN_START=0 S_BUILD='' S_DEPLOY='' S_TESTS='' LOG=''
TEST_PID=''

write_record() { # file secs
  local tmp
  tmp=$(mktemp "$STATUS/runs/.new.XXXXXX") || return 1
  printf "%s$US%s$US%s$US%s$US%s$US%s$US%s$US%s$US%s\n" "$RUN_ID" "$RUN_SHA" "$RUN_SUBJECT" "$RUN_AUTHOR" \
    "$RUN_START" "$2" "$S_BUILD" "$S_DEPLOY" "$S_TESTS" >"$tmp"
  chmod 644 "$tmp" && mv -f "$tmp" "$1"
}

save_run() {
  write_record "$STATUS/runs/$RUN_ID" $(($(date +%s) - RUN_START))
  render
}

render() {
  local live checked rows='' busy=0 f id sha subj author start secs b d t cls tag page
  live=$(cat "$STATUS/live-sha" 2>/dev/null) || live=''
  checked=$(cat "$STATUS/last-check" 2>/dev/null) || checked='not yet'
  for f in $(ls -1 "$STATUS/runs" | grep -E '^[0-9]+$' | sort -rn | head -n 50); do
    IFS=$US read -r id sha subj author start secs b d t <"$STATUS/runs/$f" || continue
    case " $b $d $t " in *' running '*) busy=1 ;; esac
    cls='' tag=''
    if [ -n "$live" ] && [ "$sha" = "$live" ]; then cls=' class="live"' tag=' <span class="ok">live</span>'; fi
    rows+="<tr$cls><td>$id</td><td><a href=\"$REPO_WEB/commit/$sha\"><code>${sha:0:7}</code></a>$tag</td>"
    rows+="<td class=\"subj\">$(esc "$subj")<div class=\"mut\">$(esc "$author")</div></td>"
    rows+="<td>$(date -u -d "@$start" '+%b %d %H:%M')</td><td>$(dur "$secs")</td>"
    rows+="<td>$(mark "$b")</td><td>$(mark "$d")</td><td>$(mark "$t")</td>"
    rows+="<td><a href=\"logs/$id-${sha:0:12}.log\">log</a></td></tr>"$'\n'
  done
  [ -n "$rows" ] || rows='<tr><td colspan="9" class="mut">No runs yet.</td></tr>'
  page=$(mktemp "$STATUS/.index.XXXXXX") || return 1
  cat >"$page" <<EOF
<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<meta http-equiv="refresh" content="$( ((busy)) && echo 5 || echo 30)">
<title>chimera ci</title>
<style>
:root{color-scheme:light dark;--fg:#1d1d1f;--bg:#fbfbfd;--mut:#6e6e73;--line:#e3e3e8;--ok:#1a7f37;--bad:#cf222e;--run:#9a6700;--hi:#eef5ff}
@media (prefers-color-scheme:dark){:root{--fg:#ececf1;--bg:#111214;--mut:#8e8e96;--line:#2a2b30;--ok:#3fb950;--bad:#f85149;--run:#d29922;--hi:#15202e}}
*{box-sizing:border-box}
body{margin:0;padding:28px 16px;font:14px/1.5 system-ui,-apple-system,"Segoe UI",sans-serif;color:var(--fg);background:var(--bg)}
main{max-width:1000px;margin:0 auto}
h1{font-size:20px;margin:0 0 6px}
p{margin:0 0 18px;color:var(--mut)}
a{color:inherit}
code{font:12.5px ui-monospace,SFMono-Regular,Menlo,Consolas,monospace}
.scroll{overflow-x:auto}
table{border-collapse:collapse;width:100%;min-width:720px}
th,td{text-align:left;padding:8px;border-bottom:1px solid var(--line);vertical-align:top;white-space:nowrap}
th{font-size:12px;font-weight:600;color:var(--mut)}
td.subj{white-space:normal;min-width:240px}
tr.live td{background:var(--hi)}
.mut{color:var(--mut)}.ok{color:var(--ok)}.bad{color:var(--bad)}.run{color:var(--run)}
</style>
</head>
<body><main>
<h1>chimera ci</h1>
<p>Builds <code>$BRANCH</code> of <a href="$REPO_WEB">${REPO_WEB#https://github.com/}</a> on $BOX_NAME and deploys it to <a href="/">${PUBLIC_URL:-this site}</a>. Checks GitHub every ${POLL_SECONDS}s (last: $checked).$([ -n "$live" ] && echo " Live: <code>${live:0:7}</code>.")</p>
<div class="scroll"><table>
<tr><th>#</th><th>commit</th><th>change</th><th>started (UTC)</th><th>took</th><th>build</th><th>deploy</th><th>tests</th><th></th></tr>
$rows
</table></div>
</main></body>
</html>
EOF
  chmod 644 "$page" && mv -f "$page" "$STATUS/index.html"
}

checkout() { # sha
  if [ ! -d "$SRC/.git" ]; then
    rm -rf "$SRC" && git clone --quiet --no-checkout "$REPO_URL" "$SRC" || return 1
  fi
  git -C "$SRC" fetch --quiet --prune origin '+refs/heads/*:refs/remotes/origin/*' || return 1
  git -C "$SRC" -c advice.detachedHead=false checkout --quiet --force "$1" || return 1
  git -C "$SRC" clean -ffdxq
  git -C "$SRC" log -1 --format='%h %s (%an, %cd)' --date=iso
}

wait_for() { # url sha tries delay: until the url (a version.json) names the commit
  local i
  for ((i = 0; i < $3; i++)); do
    curl -fsS --max-time 10 "$1" 2>/dev/null | grep -q "$2" && return 0
    sleep "$4"
  done
  return 1
}

prune() {
  # Older site images; the newest $KEEP_IMAGES stay for a quick manual rollback.
  docker image ls chimera-web --format '{{.CreatedAt}}\t{{.Tag}}' | awk -F'\t' '$2 != "latest" && $2 != "<none>"' |
    sort -r | tail -n +$((KEEP_IMAGES + 1)) | cut -f2 |
    while read -r tag; do docker image rm "chimera-web:$tag" >/dev/null 2>&1 && echo "pruned image chimera-web:$tag"; done
  # Old run records and their logs.
  ls -1 "$STATUS/runs" | grep -E '^[0-9]+$' | sort -n | head -n -"$KEEP_RUNS" |
    while read -r id; do rm -f "$STATUS/runs/$id" "$STATUS/logs/$id-"*.log; done
}

pipeline() { # sha
  RUN_ID=$(cat "$STATUS/next-id" 2>/dev/null) || RUN_ID=1
  echo $((RUN_ID + 1)) >"$STATUS/next-id"
  RUN_SHA=$1 RUN_SUBJECT='' RUN_AUTHOR='' RUN_START=$(date +%s)
  S_BUILD=running S_DEPLOY='' S_TESTS=''
  LOG=$STATUS/logs/$RUN_ID-${RUN_SHA:0:12}.log
  # Record the attempt first: a broken commit is not retried on every poll.
  echo "$RUN_SHA" >"$STATUS/last-sha"
  : >"$LOG"
  save_run
  say "run $RUN_ID: ${RUN_SHA:0:12}"

  if ! { echo "== chimera-ci run $RUN_ID: $REPO_URL $BRANCH @ $RUN_SHA"; echo "== checkout"; checkout "$RUN_SHA"; } >>"$LOG" 2>&1; then
    S_BUILD=failed; finish; return
  fi
  RUN_SUBJECT=$(git -C "$SRC" log -1 --format=%s | tr "$US\t" '  ')
  RUN_AUTHOR=$(git -C "$SRC" log -1 --format=%an | tr "$US\t" '  ')
  save_run

  if ! { echo "== build (site image)"
         "${BUILD[@]}" --target site --build-arg "GIT_SHA=$RUN_SHA" \
           -t "chimera-web:${RUN_SHA:0:12}" -t chimera-web:latest "$SRC"; } >>"$LOG" 2>&1; then
    S_BUILD=failed; finish; return
  fi
  S_BUILD=ok S_DEPLOY=running
  save_run

  if ! { echo "== deploy"
         "${COMPOSE[@]}" up -d --no-deps web &&
           echo "== verify" &&
           wait_for http://chimera-web/version.json "$RUN_SHA" 30 1 &&
           echo "chimera-web serves ${RUN_SHA:0:12}"; } >>"$LOG" 2>&1; then
    S_DEPLOY=failed; finish; return
  fi
  echo "$RUN_SHA" >"$STATUS/live-sha"
  if [ -n "$PUBLIC_URL" ]; then
    if wait_for "$PUBLIC_URL/version.json?run=$RUN_ID" "$RUN_SHA" 15 2; then
      echo "$PUBLIC_URL serves ${RUN_SHA:0:12}"
    else
      echo "warning: $PUBLIC_URL isn't serving ${RUN_SHA:0:12} yet (is chimera-cloudflared up?)"
    fi >>"$LOG" 2>&1
  fi
  S_DEPLOY=ok
  save_run
  say "run $RUN_ID: ${RUN_SHA:0:12} is live"
  prune >>"$LOG" 2>&1

  if [ "$RUN_TESTS" = 1 ]; then start_tests; else S_TESTS=skipped; finish; fi
}

finish() {
  save_run
  say "run $RUN_ID: build=${S_BUILD:--} deploy=${S_DEPLOY:--} tests=${S_TESTS:--}"
}

# Tests run in a background subshell holding a copy of the run's state, so the
# loop keeps polling. A newer commit cancels them (TERM -> stop docker build).
start_tests() {
  S_TESTS=running
  save_run
  (
    child=''
    trap 'kill -TERM "$child" 2>/dev/null; wait "$child" 2>/dev/null
          echo "== tests cancelled: a newer commit arrived" >>"$LOG"
          S_TESTS=cancelled; finish; exit 0' TERM
    echo "== test (bun run test)" >>"$LOG"
    "${BUILD[@]}" --target test --output type=cacheonly "$SRC" >>"$LOG" 2>&1 &
    child=$!
    if wait "$child"; then S_TESTS=ok; else S_TESTS=failed; fi
    finish
  ) &
  TEST_PID=$!
}

cancel_tests() {
  if [ -n "$TEST_PID" ] && kill -0 "$TEST_PID" 2>/dev/null; then
    kill -TERM "$TEST_PID" 2>/dev/null
    wait "$TEST_PID" 2>/dev/null
  fi
  TEST_PID=''
}

# A run that was mid-flight when the agent last stopped can't finish now.
for f in "$STATUS"/runs/[0-9]*; do
  [ -e "$f" ] || continue
  IFS=$US read -r RUN_ID RUN_SHA RUN_SUBJECT RUN_AUTHOR RUN_START secs S_BUILD S_DEPLOY S_TESTS <"$f" || continue
  case " $S_BUILD $S_DEPLOY $S_TESTS " in *' running '*) ;; *) continue ;; esac
  S_BUILD=${S_BUILD/running/interrupted} S_DEPLOY=${S_DEPLOY/running/interrupted} S_TESTS=${S_TESTS/running/interrupted}
  write_record "$f" "$secs"
done
RUN_ID='' RUN_SHA='' RUN_SUBJECT='' RUN_AUTHOR='' RUN_START=0 S_BUILD='' S_DEPLOY='' S_TESTS=''

trap 'say "stopping"; cancel_tests; exit 0' TERM INT
say "chimera-ci: watching $REPO_URL $BRANCH every ${POLL_SECONDS}s; site ${PUBLIC_URL:-(none)}"
while :; do
  tip=$(timeout 30 git ls-remote "$REPO_URL" "refs/heads/$BRANCH" 2>/dev/null | cut -f1)
  if [ -n "$tip" ]; then date -u '+%H:%M:%S UTC' >"$STATUS/last-check"; else say "warning: no answer from $REPO_URL"; fi
  force=''
  if [ -e "$CHIMERA_HOME/rebuild" ]; then rm -f "$CHIMERA_HOME/rebuild"; force=1; fi
  if [ -n "$tip" ] && { [ -n "$force" ] || [ "$tip" != "$(cat "$STATUS/last-sha" 2>/dev/null)" ]; }; then
    cancel_tests
    pipeline "$tip"
  fi
  render
  sleep "$POLL_SECONDS" &
  wait $!
done
