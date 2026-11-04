#!/bin/sh
# The gate (docs/development/workflow.md, section 1): formatting, lints as
# errors, then both test suites in full: the default one, then the fuzzy
# one. agents' `land` runs it through the machine's queue.
set -eu

cd "$(dirname "$0")/.."

# Each step's time, for the landing flow's reports (agents'
# docs/design/landing-flow.md, 10): `gate-step NAME SECONDS EXIT` on stderr.
step() {
    step_name=$1
    shift
    step_start=$(date +%s%N)
    step_exit=0
    "$@" || step_exit=$?
    step_ms=$((($(date +%s%N) - step_start) / 1000000))
    printf 'gate-step %s %d.%03d %d\n' "$step_name" $((step_ms / 1000)) $((step_ms % 1000)) "$step_exit" >&2
    return "$step_exit"
}

# A workspace with no crate has nothing to check, and cargo refuses it.
if cargo metadata --no-deps --format-version 1 | grep -q '"workspace_members":\[\]'; then
    echo "check.sh: the workspace has no crate yet: nothing to check"
    exit 0
fi

# Until the first world's fuzzy test lands, the fuzzy profile selects none(),
# and nextest fails a run that selects no test unless told otherwise. That
# landing gives the profiles their filters (.config/nextest.toml).
fuzzy_none=warn
if ls tests/*/tests/fuzzy_*.rs >/dev/null 2>&1; then
    if grep -qx 'default-filter = "none()"' .config/nextest.toml; then
        echo "check.sh: a fuzzy test exists, but .config/nextest.toml's profiles still lack their filters (workflow.md, section 1)" >&2
        exit 1
    fi
    fuzzy_none=fail
fi

# skein's programming-model.md, 2: every binary installs the fail-stop hook first.
step fail-stop python3 -I -B scripts/fail_stop.py
step fmt cargo fmt --check
step clippy cargo clippy --workspace --all-targets -- -D warnings
# skein's testing-strategy.md, section 8: a test is a process.
export NEXTEST_DOUBLE_SPAWN=0
step build cargo nextest run --workspace --no-run
step focused cargo nextest run --workspace
step fuzzy cargo nextest run --workspace --profile fuzzy --no-tests="$fuzzy_none"
