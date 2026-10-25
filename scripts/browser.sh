#!/bin/sh
# The browser tier (protocol/browser.md, 15), outside the gate (agents'
# docs/design/landing-flow.md, 4): jig's test application's bundle and
# jig-test, then the browser suite. Its arguments go to nextest, so that
# one test runs alone: scripts/browser.sh -E 'test(=NAME)'.
set -eu

cd "$(dirname "$0")/.."

# skein's testing-strategy.md, section 8: a test is a process.
export NEXTEST_DOUBLE_SPAWN=0
cargo build --target wasm32-unknown-unknown -p jig-client-test-shell-web
cargo build -p skein-ui-bundle
target/debug/skein-ui-bundle \
    --module target/wasm32-unknown-unknown/debug/jig_client_test_shell_web.wasm \
    --name jig_client_test_shell_web --prefix /bundle/ \
    --stylesheet crates/jig-client-shell-web/jig.css \
    --stylesheet testing/jig-client-test-shell-web/jig_client_test_shell_web.css \
    --pictures jig=crates/jig-client-shell-web/pictures \
    --pictures jig-test=testing/jig-client-test-shell-web/pictures \
    --files testing/jig-client-test-shell-web/files \
    --noscript 'jig needs WebAssembly and scripts.' \
    --out target/jig-test-bundle
cargo build -p jig-test
JIG_TEST_BUNDLE="$(pwd)/target/jig-test-bundle"
export JIG_TEST_BUNDLE
cargo nextest run --workspace --profile browser "$@"
