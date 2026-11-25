#!/bin/sh
# The journeys' one writer: temper/docs/design/client/journey-writer.md, section 4.
root=$(CDPATH='' cd "$(dirname "$0")/../.." && pwd) || exit 1
exec cargo run --locked --quiet --manifest-path "$root/Cargo.toml" -p temper-client-journeys -- writer "$@"
