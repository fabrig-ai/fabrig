#!/bin/sh
# Build the browser bundle (client/web/testing.md, 3; playground.md, 7).
set -eu
profile=debug
flag=
playground=false
while [ "$#" -gt 0 ]; do
    case "$1" in
    --release) profile=web; flag="--profile web" ;;
    --playground) playground=true ;;
    *) break ;;
    esac
    shift
done
if [ "$#" -ne 1 ]; then
    echo "usage: scripts/bundle.sh [--release] OUT | scripts/bundle.sh --playground OUT" >&2
    exit 2
fi
if [ "$playground" = true ] && [ -n "$flag" ]; then
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
    cargo build --target wasm32-unknown-unknown $flag -p temper-client-web-page
    cargo build -p skein-ui-bundle
    "$target/debug/skein-ui-bundle" \
        --module "$target/wasm32-unknown-unknown/$profile/temper_client_web_page.wasm" \
        --name temper_client_web_page --prefix /bundle/ \
        --title temper --placeholder "$shell/loading.html" \
        --placeholder-stylesheet "$shell/loading.css" \
        --stylesheet "$shell/temper.css" \
        --stylesheet "$shell/task-pages.css" \
        --files "$shell/files" \
        --noscript 'temper needs WebAssembly and scripts.' \
        --out "$out" >&2
else
    page=crates/temper-playground-page
    cargo build --target wasm32-unknown-unknown --profile web -p temper-playground-page
    cargo build --target wasm32-unknown-unknown --profile playground -p temper-playground
    cargo build -p skein-ui-bundle
    directory=$("$target/debug/skein-ui-bundle" --gzip off \
        --module "$target/wasm32-unknown-unknown/web/temper_playground_page.wasm" \
        --name temper_playground_page --prefix /bundle/ \
        --worker "temper_playground=$target/wasm32-unknown-unknown/playground/temper_playground.wasm" \
        --page "$page/playground/index.html" \
        --stylesheet "$shell/temper.css" \
        --stylesheet "$shell/task-pages.css" \
        --stylesheet "$page/playground/playground.css" \
        --files "$shell/files" \
        --out "$out")
    echo "$directory" >&2
    sed "s|{bundle}|/bundle/${directory##*/}|g" "$page/playground/headers" > "$out/headers"
fi
echo "$(cd "$out" && pwd)"
