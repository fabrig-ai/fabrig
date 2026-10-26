#!/bin/sh
# The tiers the gate leaves out that main must keep green (agents'
# roles/owners.md, 4.5): the browser tier, then the TLS every-byte sweep,
# nextest's exhaustive profile (forge-connection.md, 10), once main has it.
# Nothing runs this until the user designs how it should run.
set -eu
cd "$(dirname "$0")/.."
browser=0
scripts/browser.sh || browser=$?
if grep -q '^\[profile\.exhaustive\]' .config/nextest.toml; then
    cargo nextest run --workspace --profile exhaustive
else
    echo "after-landing: no [profile.exhaustive] in .config/nextest.toml; nothing more to run"
fi
exit "$browser"
