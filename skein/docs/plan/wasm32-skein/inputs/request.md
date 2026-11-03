# wasm32-skein: skein's lint expectations hold on wasm32

`temper-playground` can't join fab's gate list `.config/gate/wasm32.txt` (the gate's `clippy-wasm32` step). Its next blocker is skein's, of the same kind plan `wasm32-jig` fixed in jig. Measured by agents/loop:
- `unfulfilled_lint_expectations` for `clippy::large_enum_variant` on `Request`, at `skein/crates/skein-llm-connection/src/boundary.rs:49`;
- the expectation is unmet on wasm32, where sizes differ.

## The task

Make every skein crate in `temper-playground`'s wasm32 graph pass:

`cargo clippy --target wasm32-unknown-unknown -p temper-playground -- -D warnings`

as far as skein's own code goes. Sweep skein's crates for every lint expectation that holds on one target and not the other. For each, prefer `cfg_attr(not(target_arch = "wasm32"), expect(..., reason = "..."))`, or make the code right for both. Never `#[allow]` (`AGENTS.md` 2). Native clippy stays green.

Also try the skein crates the designs build for wasm32 that were red before `wasm32-rustls` (17:53Z), and say in the report which pass now:
- `skein-tls`;
- `skein-fake-forge-github`;
- `skein-fake-peers`.

One task in skein; no design. If smith's, jig's or temper's code then fails, name it in the report, unfixed.
