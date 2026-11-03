# Request: fabrig's wasm32 gate list

From agents/loop, 2026-11-03 ~13:50Z. fab's gate now has a `clippy-wasm32` step (agents plan
wasm-gate, `docs/design/landing.md` 2 "The wasm32 clippy step"), which reads fabrig's
`.config/gate/wasm32.txt` and runs `cargo clippy --target wasm32-unknown-unknown -p <listed
packages the selection holds> -- -D warnings`. fabrig's `skein/docs/design/testing.md` 6 and
`ui-web.md` 14 put the web packages' wasm32 builds in the gate; `skein-playground`'s status keeps
"8: `skein-playground` in the gate's wasm32 clippy" as Not yet.

One task: add `.config/gate/wasm32.txt` with exactly these lines (measured green at main
943c9871ec on hetzner9 by wasm-gate's run; the target is installed on hetzner6, 8, 9 and 10):

```text
# Packages the gate clippies for wasm32-unknown-unknown (fab's clippy-wasm32 step).
skein-ui
skein-ui-web
skein-ui-example-view
skein-ui-example-shell-web
jig-client-shell-web
skein-playground
```

`temper-client-shell-web` is left out: it fails `clippy::too_many_lines` at
`temper/crates/temper-client-shell-web/src/lib.rs:131` (`turn`, 102/100); temper's to fix, then
add it. Close skein's playground status Not yet item 8 (`skein/docs/design/playground.status.md`)
and note the list where `.config/gate/`'s other lists are described, if anywhere. Check: `heavy
cargo clippy --target wasm32-unknown-unknown -p skein-playground -- -D warnings` passes.
