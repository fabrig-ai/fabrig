#!/bin/sh
# The tiers the gate leaves out that main must keep green (agents'
# roles/owners.md, 4.5): today the browser tier. Nothing runs this until
# the user designs how it should run.
set -eu
cd "$(dirname "$0")/.."
exec scripts/browser.sh
