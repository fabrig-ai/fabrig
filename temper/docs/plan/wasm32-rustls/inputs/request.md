# wasm32-rustls: temper-playground builds for wasm32

`temper-playground` must build and pass clippy for `wasm32-unknown-unknown`. `temper/docs/design/playground.md` 3 ("its wasm32 build proving it") and 9 put it in the gate's wasm32 list (`.config/gate/wasm32.txt`). The jig crates' lint expectations were fixed by plan `wasm32-jig`. The next blocker, measured by agents/loop at main after 15:58Z:
- `cargo clippy --target wasm32-unknown-unknown -p temper-playground -- -D warnings` fails in rustls 0.23.41 with E0599, "no function or associated item named `now` found for struct `UnixTime`" (`src/ticketer.rs:56`, 186, 200);
- that is rustls's std-time path, which does not exist on wasm32-unknown-unknown.

So something in `temper-playground`'s dependency graph brings rustls, or rustls's `std` feature, into the wasm32 build.

## The task

Find which edge brings rustls in (`cargo tree --target wasm32-unknown-unknown -p temper-playground -i rustls -e features`), and cut it for wasm32. Prefer, in order:
1. making the dependency, or the feature that pulls rustls, target-gated (`[target.'cfg(not(target_arch = "wasm32"))'.dependencies]`) in the crate that has no use for TLS in the browser;
2. moving the TLS-using code behind a feature the playground does not enable.

The playground's world is calm and in-process (`playground.md` 4): it opens no TLS connection. Then `temper-playground`'s wasm32 clippy must pass, and its native build and tests stay green: `heavy cargo nextest run -p temper-playground`, plus the crates you touch.

One task; no design, unless a design section names the dependency, in which case its status says what changed. Leave `playground-browser`'s files alone (it is building beside you): `temper-client-shell-web` and `tests/playground`.

When it passes, the report says so, so that agents/loop can add `temper-playground` to the list.
