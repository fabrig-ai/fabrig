#!/bin/sh
# A person's sitting (testing/live-tier.md): fresh ChatGPT sign-in and real stories.
set -eu
umask 077
cd "$(dirname "$0")/.."
# TEMPER_TEST_WRAPPER, when set and not empty, names one command that runs each build
# command below; unset, cargo runs plainly.
if [ "${1-}" = rung-b0-offline ]; then
    [ "$#" -eq 1 ] || { printf '%s\n' 'usage: scripts/proof-live.sh rung-b0-offline' >&2; exit 2; }
    export NEXTEST_DOUBLE_SPAWN=0
    # No live account is admitted by the offline proofs (testing/process-tier.md).
    exec ${TEMPER_TEST_WRAPPER:+"$TEMPER_TEST_WRAPPER"} cargo nextest run -p temper-box --test proof_live --profile process-temper --no-fail-fast
fi
case ${1-} in
    '') test_name=a_real_model_answers_a_chat_and_pushes_a_small_change ;;
    forge-land) test_name=a_person_asks_chatgpt_to_land_a_change_on_the_owned_forge ;;
    rung-b0) test_name=rung_b0_a_person_agrees_a_plan_before_the_change_lands ;;
    forge-land-claude-key) test_name=a_person_asks_claude_on_an_api_key_to_land_a_change_on_the_owned_forge ;;
    forge-land-claude) test_name=a_person_asks_claude_code_to_land_a_change_on_the_owned_forge ;;
    *) printf '%s\n' 'usage: scripts/proof-live.sh [forge-land|forge-land-claude|forge-land-claude-key|rung-b0]' >&2; exit 2 ;;
esac
[ "$#" -le 1 ] || { printf '%s\n' 'usage: scripts/proof-live.sh [forge-land|forge-land-claude|forge-land-claude-key|rung-b0]' >&2; exit 2; }
# Resolve paths even when the last name (a fresh story's store) is absent.
resolved() {
    python3 -I -c 'import os, sys; print(os.path.realpath(sys.argv[1]))' "$1"
}
# The key is admitted by stat alone, never opened, read or printed.
if [ "${1-}" = forge-land-claude-key ]; then
    [ -z "${TEMPER_TEST_LIVE_ACCOUNT-}" ] || { printf '%s\n' 'TEMPER_TEST_LIVE_ACCOUNT is for forge-land' >&2; exit 2; }
    [ -z "${TEMPER_TEST_LIVE_CLAUDE-}" ] || { printf '%s\n' 'TEMPER_TEST_LIVE_CLAUDE is for forge-land-claude' >&2; exit 2; }
    [ -n "${TEMPER_TEST_LIVE_ANTHROPIC_KEY-}" ] || { printf '%s\n' 'TEMPER_TEST_LIVE_ANTHROPIC_KEY is required for forge-land-claude-key' >&2; exit 2; }
    python3 -I - "$TEMPER_TEST_LIVE_ANTHROPIC_KEY" <<'PYTHON'
import os
import stat
import sys

def refuse(message):
    print("TEMPER_TEST_LIVE_ANTHROPIC_KEY " + message, file=sys.stderr)
    sys.exit(2)

try:
    metadata = os.stat(sys.argv[1], follow_symlinks=False)
except OSError:
    refuse("names no file")
if not stat.S_ISREG(metadata.st_mode):
    refuse("is not a regular file or is a link")
if stat.S_IMODE(metadata.st_mode) != 0o600:
    refuse("is not 0600")
if metadata.st_uid != os.geteuid():
    refuse("is not owned by this user")
if metadata.st_size == 0:
    refuse("is empty")
PYTHON
fi
# Inspect only metadata, and at Claude admission list entry names once, without
# opening any entry (temper/docs/design/testing/live-tier.md, 5).
held_metadata() {
    python3 -I - "$1" "$2" <<'PYTHON'
import os
import stat
import sys

kind, path = sys.argv[1:]

def refuse(message):
    source = "CLAUDE" if kind == "claude-in" else "ACCOUNT"
    print(f"TEMPER_TEST_LIVE_{source} {message}", file=sys.stderr)
    sys.exit(2)

def directory_identity(metadata):
    return (metadata.st_dev, metadata.st_ino, stat.S_IMODE(metadata.st_mode), metadata.st_uid)

try:
    before = os.lstat(path)
except OSError:
    before = None

if kind == "account-in":
    if before is None or not stat.S_ISREG(before.st_mode):
        refuse("names no regular file")
    if stat.S_IMODE(before.st_mode) != 0o600:
        refuse("is not 0600")
    sys.exit(0)

if kind == "account-back":
    if (before is None or not stat.S_ISREG(before.st_mode) or
            stat.S_IMODE(before.st_mode) != 0o600 or before.st_size <= 0):
        sys.exit(1)
    print(f"LOGIN BACK: regular file mode=600 bytes={before.st_size} owner={before.st_uid}")
    sys.exit(0)

if kind == "claude-in":
    if before is None:
        refuse("names no directory")
    if not stat.S_ISDIR(before.st_mode):
        refuse("is not a directory or is a link")
    if stat.S_IMODE(before.st_mode) != 0o700:
        refuse("is not 0700")
    if before.st_uid != os.geteuid():
        refuse("is not owned by this user")
    name = os.path.basename(path)
    if not 1 <= len(name) <= 32 or any(c not in "abcdefghijklmnopqrstuvwxyz0123456789-" for c in name):
        refuse("has no valid source name (1 to 32 lowercase letters, digits or hyphens)")
    try:
        fd = os.open(path, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW)
        try:
            held = os.fstat(fd)
            if not stat.S_ISDIR(held.st_mode) or directory_identity(held) != directory_identity(before):
                refuse("changed during admission")
            if not os.listdir(fd):
                refuse("is empty")
        finally:
            os.close(fd)
    except OSError:
        refuse("cannot be opened as a directory without following a link")
    before = held
elif before is None or not stat.S_ISDIR(before.st_mode):
    sys.exit(1)

dev, ino, mode, uid = directory_identity(before)
print(f"{dev}:{ino}:{mode:o}:{uid}:directory")
PYTHON
}
claude_identity=
if [ "${1-}" = forge-land-claude ]; then
    [ -n "${TEMPER_TEST_LIVE_CLAUDE-}" ] || { printf '%s\n' 'TEMPER_TEST_LIVE_CLAUDE is required for forge-land-claude' >&2; exit 2; }
    [ -z "${TEMPER_TEST_LIVE_ACCOUNT-}" ] || { printf '%s\n' 'TEMPER_TEST_LIVE_ACCOUNT is for forge-land' >&2; exit 2; }
    while [ "${TEMPER_TEST_LIVE_CLAUDE%/}" != "$TEMPER_TEST_LIVE_CLAUDE" ]; do
        TEMPER_TEST_LIVE_CLAUDE=${TEMPER_TEST_LIVE_CLAUDE%/}
    done
    claude_identity=$(held_metadata claude-in "$TEMPER_TEST_LIVE_CLAUDE") || exit 2
    TEMPER_TEST_LIVE_CLAUDE=$(resolved "$(dirname -- "$TEMPER_TEST_LIVE_CLAUDE")")/$(basename -- "$TEMPER_TEST_LIVE_CLAUDE")
    export TEMPER_TEST_LIVE_CLAUDE
elif [ -n "${TEMPER_TEST_LIVE_CLAUDE-}" ]; then
    printf '%s\n' 'TEMPER_TEST_LIVE_CLAUDE is for forge-land-claude' >&2
    exit 2
fi
store=
held_landing=false
rung=false
rerun=false
if [ "${1-}" = forge-land ] || [ "${1-}" = forge-land-claude ] || [ "${1-}" = forge-land-claude-key ] || [ "${1-}" = rung-b0 ]; then
    if [ "${1-}" = rung-b0 ]; then
        rung=true
        [ -n "${TEMPER_TEST_LIVE_ACCOUNT-}" ] || { printf '%s\n' 'TEMPER_TEST_LIVE_ACCOUNT is required for rung-b0' >&2; exit 2; }
        store=${TEMPER_TEST_LIVE_STORE:-$PWD/../rung-b0-$(date -u +%Y%m%dT%H%M%SZ)-$$}
        # Rung boxes and their retained evidence belong to this task directory.
        export TMPDIR=${TMPDIR:-$PWD/..}
        export TEMPER_TEST_LIVE_KEEP=${TEMPER_TEST_LIVE_KEEP:-$PWD/../live-evidence}
    else
        stores=${TEMPER_TEST_LIVE_STORE:-$HOME/src/rust/tmp/live-store}
        mkdir -p -- "$stores"
        stores=$(resolved "$stores")
        store=$stores/$1
        export TEMPER_TEST_LIVE_STORE="$stores"
    fi
    if [ -e "$store" ] || [ -L "$store" ]; then
        if [ ! -d "$store" ] || [ -n "$(ls -A -- "$store")" ]; then
            rerun=true
            printf 'rerunning on the kept deployment in %s; needs no person\n' "$store"
        fi
    fi
    store_parent=$(dirname -- "$store")
    mkdir -p -- "$store_parent"
    store=$(resolved "$store")
    if [ "$rung" = true ]; then export TEMPER_TEST_LIVE_STORE="$store"; fi
    if [ -n "${TEMPER_TEST_LIVE_ACCOUNT-}" ]; then
        # A held record moved in by ctl account import (client/ctl/areas.md), which removes it.
        [ "$rerun" = false ] || { printf '%s\n' 'a held account moves into a fresh store only' >&2; exit 2; }
        # Its path goes to ctl account import alone, never into this script's words.
        held_metadata account-in "$TEMPER_TEST_LIVE_ACCOUNT" || exit 2
        TEMPER_TEST_LIVE_ACCOUNT=$(resolved "$TEMPER_TEST_LIVE_ACCOUNT")
        export TEMPER_TEST_LIVE_ACCOUNT
        if [ "${1-}" = forge-land ]; then held_landing=true; fi
        printf '%s\n' 'moving the held ChatGPT account in; needs no person'
    fi
elif [ -n "${TEMPER_TEST_LIVE_ACCOUNT-}" ]; then
    printf '%s\n' 'TEMPER_TEST_LIVE_ACCOUNT is for forge-land or rung-b0' >&2
    exit 2
fi
export NEXTEST_DOUBLE_SPAWN=0
export TEMPER_INTERNAL_BUNDLE=${TEMPER_INTERNAL_BUNDLE:-none}
scratch=$(mktemp -d "${TMPDIR:-/tmp}/temper-live.XXXXXX")
scratch=$(resolved "$scratch")
# Snapshot process working directories in one process, without opening their files.
processes_in() {
    python3 -I - "$1" "$2" <<'PYTHON'
import os
import subprocess
import sys

def proc_cwds():
    try:
        processes = os.scandir("/proc")
    except OSError:
        return
    with processes:
        for process in processes:
            if not process.name.isdecimal():
                continue
            try:
                cwd = os.readlink(process.path + "/cwd")
            except OSError:
                continue
            yield process.name, cwd

def lsof_cwds():
    try:
        output = subprocess.run(
            ["/usr/sbin/lsof", "-a", "-u", str(os.geteuid()), "-d", "cwd", "-Fpn"],
            stdout=subprocess.PIPE, stderr=subprocess.DEVNULL,
        ).stdout
    except OSError:
        return
    # A process may disappear during the listing; read the fields lsof found.
    pid = None
    for line in os.fsdecode(output).split("\n"):
        field, value = line[:1], line[1:]
        if field == "p":
            pid = value if value.isascii() and value.isdecimal() else None
        elif field == "n" and pid and value.startswith("/"):
            yield pid, os.path.realpath(value)

scratch, store = sys.argv[1:]
cwds = lsof_cwds() if sys.platform == "darwin" else proc_cwds()
print(" ".join(pid for pid, cwd in cwds if
    (scratch and cwd.startswith(scratch + "/")) or
    (store and (cwd == store or cwd.startswith(store + "/")))))
PYTHON
}
# A kept deployment may already have processes. Only this sitting's new
# processes belong to its cleanup, even when its box refuses a busy store.
# Every process whose cwd is in this newly created private scratch was started
# by this sitting. As in jig's live-sign-in.sh, stop those owned descendants
# before keeping or erasing their stores, including after an interruption.
# Called by the EXIT trap below.
# shellcheck disable=SC2317
stop_owned() {
    [ "$launched" = true ] || return 0
    [ -d "$scratch" ] || return 0
    owned=
    for process in $(processes_in "$scratch" "$store"); do
        case " $store_before " in
            *" $process "*) ;;
            *) owned="$owned $process" ;;
        esac
    done
    for process in $owned; do kill -KILL "$process" 2>/dev/null || true; done
    attempt=0
    left=
    while [ "$attempt" -lt 100 ]; do
        left=
        for process in $owned; do
            if kill -0 "$process" 2>/dev/null; then left="$left $process"; fi
        done
        [ -n "$left" ] || break
        owned=$left
        attempt=$((attempt + 1))
        sleep 0.1
    done
    [ -z "$left" ]
}
# The installed hook has 60 seconds, including its group of child processes.
# shellcheck disable=SC2317
give_back() {
    python3 -I -c '
import os
import signal
import subprocess
import sys

hook = subprocess.Popen(sys.argv[1:], start_new_session=True)
try:
    hook.wait(timeout=60)
except subprocess.TimeoutExpired:
    try:
        os.killpg(hook.pid, signal.SIGTERM)
        # A stopped group must be able to take the pending TERM.
        os.killpg(hook.pid, signal.SIGCONT)
    except ProcessLookupError:
        pass
    hook.wait()
' "$1"
}
# This runs in the background; exec makes the follower PID the one finish can kill.
follow_notice() {
    exec python3 -I - "$1" "$2" <<'PYTHON'
import os
import shutil
import sys
import time

path, runner = sys.argv[1], int(sys.argv[2])
with open(path, "rb") as notice:
    while True:
        try:
            os.kill(runner, 0)
            running = True
        except ProcessLookupError:
            running = False
        # Read after sampling the runner, so the last pass drains its final writes.
        shutil.copyfileobj(notice, sys.stdout.buffer)
        sys.stdout.buffer.flush()
        if not running:
            break
        time.sleep(0.2)
PYTHON
}
notice=$scratch/notice
: > "$notice"
chmod 600 "$notice"
export TEMPER_TEST_LIVE_NOTICE="$notice"
launched=false
runner=
follower=
# shellcheck disable=SC2317
finish() {
    status=$?
    trap - EXIT INT TERM HUP
    set +e
    if [ -n "$runner" ]; then
        kill -TERM "$runner" 2>/dev/null
        wait "$runner" 2>/dev/null
    fi
    if [ -n "$follower" ]; then
        kill "$follower" 2>/dev/null
        wait "$follower" 2>/dev/null
    fi
    if [ "$rung" = true ] || [ "$held_landing" = true ]; then
        # Anything at the lent path is left alone, including unsafe metadata.
        if [ ! -e "$TEMPER_TEST_LIVE_ACCOUNT" ] && [ ! -L "$TEMPER_TEST_LIVE_ACCOUNT" ]; then
            if [ -x "$store/give-back" ]; then
                give_back "$store/give-back" >> "$notice" 2>&1
            fi
            if [ ! -e "$TEMPER_TEST_LIVE_ACCOUNT" ] && [ ! -L "$TEMPER_TEST_LIVE_ACCOUNT" ]; then
                TEMPER_TEST_LIVE_STORE="$store" TMPDIR="$scratch" ${TEMPER_TEST_WRAPPER:+"$TEMPER_TEST_WRAPPER"} cargo nextest run -p temper-box --profile live-temper --ignore-default-filter --run-ignored only \
                    -E 'binary(=live) & test(=the_held_record_returns_on_a_box_of_its_own)' >> "$notice" 2>&1
            fi
        fi
    fi
    stopped=true
    stop_owned || { stopped=false; status=1; }
    if [ "$rung" = true ] || [ "$held_landing" = true ]; then
        if returned=$(held_metadata account-back "$TEMPER_TEST_LIVE_ACCOUNT" 2>/dev/null); then
            printf '%s\n' "$returned" >> "$notice"
        else
            printf 'LOGIN NOT BACK: final stat failed; store %s retained for recovery\n' "$store" >&2
            status=1
        fi
    fi
    if [ -n "$claude_identity" ]; then
        returned=$(held_metadata claude-back "$TEMPER_TEST_LIVE_CLAUDE" 2>/dev/null)
        if [ "$stopped" = true ] && [ "$returned" = "$claude_identity" ]; then
            printf '%s\n' 'CLAUDE LOGIN BACK: box stopped; same private directory, inode and owner; used in place' >> "$notice"
        else
            printf '%s\n' 'CLAUDE LOGIN NOT BACK: deployment did not stop or held directory metadata changed' >> "$notice"
            status=1
        fi
    fi
    printf '%s\n' 'Last live notices:'
    tail -n 30 "$notice"
    keep=${TEMPER_TEST_LIVE_KEEP:-$HOME/src/rust/tmp/live-evidence}
    tag=$(sed -n 's/^TAG \([^; ]*\).*$/\1/p' "$notice" | head -n 1 | tr -cd 'A-Za-z0-9_.:-')
    [ -n "$tag" ] || tag=$(date -u +%Y-%m-%dT%H:%M:%SZ)-$$
    if mkdir -p -- "$keep" && chmod 700 "$keep"; then
        # The kept notice never carries the transient CLIENT bearer.
        kept=$keep/$tag.notice
        if sed 's/secret=[^&[:space:]]*/secret=…/g' "$notice" > "$kept" && chmod 600 "$kept"; then
            printf 'notice %s\n' "$kept"
        else
            printf 'could not retain notice in %s\n' "$keep" >&2
            status=1
        fi
    else
        printf 'could not create private evidence directory %s\n' "$keep" >&2
        status=1
    fi
    if [ "$status" -eq 0 ]; then
        rm -rf -- "$scratch"
    else
        # Retain scratch evidence, and redact the transient link there too.
        sed 's/secret=[^&[:space:]]*/secret=…/g' "$notice" > "$scratch/notice.redacted"
        mv -- "$scratch/notice.redacted" "$notice"
        printf 'kept %s; erase it with rm -rf -- %s once read\n' "$scratch" "$scratch"
    fi
    [ -z "$store" ] || printf 'kept store %s\n' "$store"
    exit "$status"
}
trap finish EXIT
trap 'exit 130' INT
trap 'exit 143' TERM
trap 'exit 129' HUP
# A build failure or a signal also returns through finish, returning the login before stopping the box.
${TEMPER_TEST_WRAPPER:+"$TEMPER_TEST_WRAPPER"} cargo nextest run -p temper-box --profile live-temper --no-run
if [ "$rerun" = false ] && [ -z "${TEMPER_TEST_LIVE_ACCOUNT-}" ] && [ -z "$claude_identity" ] && [ "${1-}" != forge-land-claude-key ]; then
    printf '%s\n' 'Approve the printed ChatGPT address in a browser that reaches this machine.' \
        'For a remote browser: ssh -L 1455:127.0.0.1:1455 -L <client port>:127.0.0.1:<client port> HOST'
fi
store_before=
if [ -n "$store" ] && [ -d "$store" ]; then
    store_before=$(processes_in "" "$store")
fi
launched=true
TMPDIR=$scratch ${TEMPER_TEST_WRAPPER:+"$TEMPER_TEST_WRAPPER"} cargo nextest run -p temper-box --profile live-temper --ignore-default-filter \
    --run-ignored only --no-capture -E "binary(=live) & test(=$test_name)" &
runner=$!
follow_notice "$notice" "$runner" &
follower=$!
status=0
wait "$runner" || status=$?
runner=
# Let the follower flush notices appended just before the run returned.
wait "$follower" || true
follower=
exit "$status"
