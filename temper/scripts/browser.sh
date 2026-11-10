#!/bin/sh
# The browser tier (client/web/testing.md, 3), outside the gate.
set -eu
cd "$(dirname "$0")/../.."
exec python3 -I -B tools/tiers/run temper browser -- "$@"
