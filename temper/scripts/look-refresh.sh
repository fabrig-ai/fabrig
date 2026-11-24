#!/bin/sh
# Refresh the requests stack (temper/docs/design/client/web/journeys.md, 7).
root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd) || exit 1
exec cargo run --locked --quiet --manifest-path "$root/Cargo.toml" -p temper-client-journeys -- look-refresh "$@"
