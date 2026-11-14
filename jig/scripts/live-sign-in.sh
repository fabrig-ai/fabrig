#!/usr/bin/env bash
# The person-driven live layer (protocol/service.md, section 8): a fresh
# ChatGPT sign-in, a real model's answer and a confirmed sign-out.
set -euo pipefail
umask 077
cd "$(dirname "$0")/.."
# JIG_WRAPPER, when set and not empty, names one command that runs the build.
# When unset or empty, cargo runs plainly.
printf '%s\n' 'The browser must reach 127.0.0.1:1455 on this machine: ssh -L 1455:127.0.0.1:1455 HOST'
export NEXTEST_DOUBLE_SPAWN=0
${JIG_WRAPPER:+"$JIG_WRAPPER"} cargo nextest run -p jig-test --profile live --no-run
# The test's scratch goes under this script's own directory, so that a run cut
# short (an interrupt, nextest's timeout) leaves no secret behind: the
# secrets' stores and the agent trace are erased however it ends.
scratch=$(readlink -f "$(mktemp -d)")
# The test's world starts jig-test in a process group of its own. Once the
# runner has ended, stop any children left holding the scratch and port 1455
# before erasing the records.
stop() {
    local process left=()
    for process in /proc/[0-9]*; do
        if [[ $(readlink "$process/cwd" 2>/dev/null) == "$scratch"/* ]]; then
            left+=("${process#/proc/}")
        fi
    done
    ((${#left[@]})) || return 0
    kill -KILL "${left[@]}" 2>/dev/null || true
    for _ in {1..100}; do
        kill -0 "${left[@]}" 2>/dev/null || break
        sleep 0.1
    done
}
erase() {
    [[ -d $scratch ]] || return 0
    stop
    find "$scratch" \( -type d -name secrets -prune -o -type f -name agent-trace.jsonl \) -exec rm -rf {} +
}
runner=
cleanup() {
    local status=$?
    trap '' INT TERM
    if [[ -n $runner ]]; then
        kill -TERM "$runner" 2>/dev/null || true
        wait "$runner" || true
        runner=
    fi
    erase
    if ((status == 0)); then
        rm -rf "$scratch"
    else
        printf '%s\n' "Diagnostics, without secrets: $scratch" \
            "A sign-in that did not sign out may still be listed at the issuer. Disconnect it in ChatGPT's settings, under Apps."
    fi
}
trap cleanup EXIT
interrupted=0
trap 'interrupted=130' INT
trap 'interrupted=143' TERM
status=0
TMPDIR=$scratch cargo nextest run -p jig-test --profile live --no-capture \
    -E 'test(=a_person_signs_in_to_chatgpt_and_a_real_model_answers)' &
runner=$!
if ((interrupted != 0)); then
    exit "$interrupted"
fi
wait "$runner" || status=$?
if ((interrupted != 0)); then
    exit "$interrupted"
fi
runner=
exit "$status"
