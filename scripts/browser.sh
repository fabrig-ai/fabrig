#!/bin/sh
# The browser tier, outside the gate (agents' docs/design/landing-flow.md,
# 4): the UI example's bundle and server, then the browser suite
# (ui-web.md, 14.1). Its arguments go to nextest, so that one test runs
# alone: scripts/browser.sh -E 'test(=NAME)'.
set -eu

cd "$(dirname "$0")/.."

# testing-strategy.md, section 8: nextest starts each test directly.
export NEXTEST_DOUBLE_SPAWN=0

# The browser tier uses a dev-profile module under the server's strict policy.
cargo build --target wasm32-unknown-unknown -p skein-ui-example-shell-web
cargo run -p skein-ui-bundle -- \
    --module target/wasm32-unknown-unknown/debug/skein_ui_example_shell_web.wasm \
    --name skein_ui_example_shell_web --prefix /bundle/ --stylesheet examples/ui/shell-web/notes.css --out target/ui-example-bundle \
    --noscript 'Notes needs WebAssembly and scripts.'
cargo build -p skein-ui-example-server
SKEIN_UI_EXAMPLE_BUNDLE="$(pwd)/target/ui-example-bundle"
export SKEIN_UI_EXAMPLE_BUNDLE
cargo nextest run --workspace --profile browser "$@"
