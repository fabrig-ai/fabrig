#!/bin/sh
# Install the binary with its own browser bundle (client/browser.md, 2.6).
set -eu
if [ "$#" -ne 1 ]; then
    echo "usage: scripts/install.sh PREFIX" >&2
    exit 2
fi
prefix=$1
cd "$(dirname "$0")/.."
mkdir -p "$prefix/bin"
prefix=$(cd "$prefix" && pwd)
# A person's binary, whatever the environment says (client/browser.md, 2.6).
TEMPER_BUNDLE=embed cargo build --release --locked -p temper
install -m 0755 target/release/temper "$prefix/bin/temper"
printf '%s\n' "$prefix/bin/temper"
