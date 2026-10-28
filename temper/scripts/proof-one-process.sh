#!/bin/sh
# The process tier (testing.md, 2.6), run once through heavy --gate.
set -eu
cd "$(dirname "$0")/.."
export NEXTEST_DOUBLE_SPAWN=0
cargo build -p temper --bin temper -p temper-fake-claude --bin fake-claude
export TEMPER_TEST_CLAUDE="$(pwd)/target/debug/fake-claude"
cargo nextest run -p temper --profile process "$@"
