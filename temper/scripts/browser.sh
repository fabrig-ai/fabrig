#!/bin/sh
# The browser tier (client/browser.md, 4.3), outside the gate.
set -eu
cd "$(dirname "$0")/../.."
exec python3 -I -B tools/tiers/run temper browser -- "$@"
