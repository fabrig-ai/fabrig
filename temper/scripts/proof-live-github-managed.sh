#!/bin/sh
# Owner-run managed GitHub sitting; implementers run fake only.
set -eu
umask 077
# Preserve the person's answers before the build wrapper can read stdin.
exec 3<&0
cd "$(dirname "$0")/.."
# TEMPER_TEST_WRAPPER, when set and not empty, names one command that runs each build
# command below; unset, cargo runs plainly.
case ${1:-} in live|fake|fake-wide) mode=$1 ;; *) printf '%s\n' 'usage: proof-live-github-managed.sh {live|fake|fake-wide} [PORT [CONVERSION.json]]' >&2; exit 2 ;; esac
[ "$#" -le 3 ] || exit 2
port=${2:-8765}
case $port in ''|*[!0-9]*) exit 2 ;; esac
[ "$port" -ge 1 ] && [ "$port" -le 65535 ] || exit 2
# Connect makes the App under the person's own account (protocol/setup/github.md, 2), so a live
# sitting says whose: the relay admits that login's conversion alone. No default.
if [ "$mode" = live ]; then
    person=${TEMPER_TEST_MANAGED_PERSON:-}
    case $person in
        ''|-*|*[!A-Za-z0-9-]*)
            printf '%s\n' 'live needs TEMPER_TEST_MANAGED_PERSON, the GitHub login that makes the App' >&2
            exit 2
            ;;
    esac
else
    person=managed-person
fi
export CARGO_NET_OFFLINE=true TEMPER_INTERNAL_BUNDLE=none
export CARGO_TARGET_DIR="$HOME/src/rust/tmp/owners/github-managed/target" NEXTEST_DOUBLE_SPAWN=0
scratch=$(readlink -f "$(mktemp -d "${TMPDIR:-/tmp}/temper-live-github-managed.XXXXXX")")
runner=
cleanup() {
    status=$?
    trap - EXIT INT TERM HUP
    if [ -n "$runner" ] && kill -0 "$runner" 2>/dev/null; then
        kill -TERM "$runner" 2>/dev/null || true
        attempt=0
        while kill -0 "$runner" 2>/dev/null && [ "$attempt" -lt 100 ]; do
            sleep 0.1
            attempt=$((attempt + 1))
        done
    fi
    # Fresh cwd ownership, never another process's environment or command line.
    for process in /proc/[0-9]*; do
        case $(readlink "$process/cwd" 2>/dev/null || true) in
            "$scratch"|"$scratch"/*) kill -KILL "${process#/proc/}" 2>/dev/null || true ;;
        esac
    done
    rm -rf -- "$scratch"
    printf '%s\n' 'scratch wiped'
    exit "$status"
}
trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM
trap 'exit 129' HUP
if [ "$mode" = live ]; then
    evidence=${3:-$HOME/src/rust/tmp/owners/github-managed/reports/m3-conversion.json}
else
    evidence=${3:-$scratch.conversion.json}
fi
# create_new in the relay protects existing evidence, including symlinks.
if [ -e "$evidence" ] || [ -L "$evidence" ]; then
    printf '%s\n' 'conversion evidence path already exists' >&2
    exit 2
fi
printf '%s\n' 'building the sitting offline'
if ! ${TEMPER_TEST_WRAPPER:+"$TEMPER_TEST_WRAPPER"} cargo test --locked -p temper-box --test live_github_managed --no-run --message-format=json < /dev/null > "$scratch/build.jsonl" 2> "$scratch/build.log"; then
    # Compilation has no sitting credentials. Surface its diagnostics before cleanup.
    python3 - "$scratch/build.jsonl" <<'PYBUILD'
import json
import sys
for line in open(sys.argv[1]):
    if line.startswith('{'):
        row = json.loads(line)
        if row.get('reason') == 'compiler-message':
            print(row['message']['rendered'], file=sys.stderr)
PYBUILD
    tail -n 15 "$scratch/build.log" >&2
    exit 1
fi
if [ "$mode" = live ]; then
    # Keep TEMPER_INTERNAL_BUNDLE=none; build a separate, explicit browser bundle in this scratch.
    printf '%s\n' 'building the browser bundle offline'
    if ! ${TEMPER_TEST_WRAPPER:+"$TEMPER_TEST_WRAPPER"} cargo build --locked --profile web --target wasm32-unknown-unknown -p temper-client-web-page < /dev/null > "$scratch/web-build.log" 2>&1; then
        tail -n 15 "$scratch/web-build.log" >&2
        exit 1
    fi
    if ! ${TEMPER_TEST_WRAPPER:+"$TEMPER_TEST_WRAPPER"} cargo build --locked -p skein-ui-bundle < /dev/null > "$scratch/bundler-build.log" 2>&1; then
        tail -n 15 "$scratch/bundler-build.log" >&2
        exit 1
    fi
    "$CARGO_TARGET_DIR/debug/skein-ui-bundle" \
        --module "$CARGO_TARGET_DIR/wasm32-unknown-unknown/web/temper_client_web_page.wasm" \
        --name temper_client_web_page --prefix /bundle/ \
        --stylesheet crates/temper-client-shell-web/temper.css \
        --stylesheet crates/temper-client-shell-web/task-pages.css \
        --files crates/temper-client-shell-web/files \
        --noscript 'temper needs WebAssembly and scripts.' --out "$scratch/bundle" > "$scratch/bundle.log" 2>&1
fi
# A background job's stdin is /dev/null when job control is off (POSIX): keep the person's on fd 3.
python3 scripts/proof-live-github-managed.py "$mode" "$port" "$scratch" "$evidence" "$person" <&3 3<&- &
runner=$!
wait "$runner"
runner=
