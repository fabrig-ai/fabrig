#!/bin/sh
# Compare a request with what landed (temper/docs/design/client/journey-writer.md, 6.1).
root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd) || exit 1
exec cargo run --locked --quiet --manifest-path "$root/Cargo.toml" -p temper-client-journeys -- look-compare "$@"
