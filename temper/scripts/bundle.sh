#!/bin/sh
# Build the browser bundle (client/browser.md, 4.3; playground.md, 7).
set -eu
profile=debug
flag=
features=
playground=false
while [ "$#" -gt 0 ]; do
    case "$1" in
    --release) profile=release; flag=--release ;;
    --loopback) features="${features:+$features,}loopback-navigation" ;;
    --record) features="${features:+$features,}record" ;;
    --playground) playground=true ;;
    *) break ;;
    esac
    shift
done
if [ "$#" -ne 1 ]; then
    echo "usage: scripts/bundle.sh [--release] [--loopback] [--record] OUT | scripts/bundle.sh --playground OUT" >&2
    exit 2
fi
if [ "$playground" = true ] && { [ -n "$flag" ] || [ -n "$features" ]; }; then
    echo "scripts/bundle.sh: --playground cannot be combined with other options" >&2
    exit 2
fi
out=$1
cd "$(dirname "$0")/.."
target=${CARGO_TARGET_DIR:-../target}
if [ -e "$out" ] && [ ! -e "$out/index.html" ]; then
    echo "scripts/bundle.sh: $out exists and holds no index.html" >&2
    exit 1
fi
shell=crates/temper-client-shell-web
if [ "$playground" = false ]; then
    cargo build --target wasm32-unknown-unknown $flag ${features:+--features "$features"} -p temper-client-shell-web
    cargo build -p skein-ui-bundle
    "$target/debug/skein-ui-bundle" \
        --module "$target/wasm32-unknown-unknown/$profile/temper_client_shell_web.wasm" \
        --name temper_client_shell_web --prefix /bundle/ \
        --stylesheet "$shell/temper.css" \
        --stylesheet "$shell/task-pages.css" \
        --files "$shell/files" \
        --noscript 'temper needs WebAssembly and scripts.' \
        --out "$out" >&2
else
    cargo build --target wasm32-unknown-unknown --profile web --features playground -p temper-client-shell-web
    cargo build --target wasm32-unknown-unknown --profile playground -p temper-playground
    cargo build -p skein-ui-bundle
    directory=$("$target/debug/skein-ui-bundle" \
        --module "$target/wasm32-unknown-unknown/web/temper_client_shell_web.wasm" \
        --name temper_client_shell_web --prefix /bundle/ \
        --worker "temper_playground=$target/wasm32-unknown-unknown/playground/temper_playground.wasm" \
        --page "$shell/playground/index.html" \
        --stylesheet "$shell/temper.css" \
        --stylesheet "$shell/task-pages.css" \
        --stylesheet "$shell/playground/playground.css" \
        --files "$shell/files" \
        --out "$out")
    echo "$directory" >&2
    sed "s|{bundle}|/bundle/${directory##*/}|g" "$shell/playground/headers" > "$out/headers"
fi
echo "$(cd "$out" && pwd)"
