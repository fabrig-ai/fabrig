#!/bin/sh
# Build the browser bundle (client/browser.md, 4.3): scripts/bundle.sh [--release] [--loopback] [--record] OUT.
set -eu
profile=debug
flag=
features=
while [ "$#" -gt 0 ]; do
    case "$1" in
    --release) profile=release; flag=--release ;;
    --loopback) features="${features:+$features,}loopback-navigation" ;;
    --record) features="${features:+$features,}record" ;;
    *) break ;;
    esac
    shift
done
if [ "$#" -ne 1 ]; then
    echo "usage: scripts/bundle.sh [--release] [--loopback] [--record] OUT" >&2
    exit 2
fi
out=$1
cd "$(dirname "$0")/.."
if [ -e "$out" ] && [ ! -e "$out/index.html" ]; then
    echo "scripts/bundle.sh: $out exists and holds no index.html" >&2
    exit 1
fi
cargo build --target wasm32-unknown-unknown $flag ${features:+--features "$features"} -p temper-client-shell-web
cargo build -p skein-ui-bundle
shell=crates/temper-client-shell-web
target/debug/skein-ui-bundle \
    --module "target/wasm32-unknown-unknown/$profile/temper_client_shell_web.wasm" \
    --name temper_client_shell_web --prefix /bundle/ \
    --stylesheet "$shell/temper.css" \
    --stylesheet "$shell/task-pages.css" \
    --files "$shell/files" \
    --noscript 'temper needs WebAssembly and scripts.' \
    --out "$out" >&2
echo "$(cd "$out" && pwd)"
