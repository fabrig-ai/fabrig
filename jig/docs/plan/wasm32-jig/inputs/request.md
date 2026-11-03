# wasm32-jig: jig's crates pass wasm32 clippy

`temper-playground` can't join fab's gate list `.config/gate/wasm32.txt` (the gate's `clippy-wasm32` step). Its wasm32 clippy fails in its jig dependencies, not in temper:
- `unfulfilled_lint_expectations` at `jig/crates/jig-domain-people/src/boundary.rs:225`: `pub enum Ask` has `#[expect(clippy::large_enum_variant)]`, which isn't met on wasm32, where sizes differ;
- the same kind of failure in `jig-domain-tasks`.

## The task

Make every jig crate that `temper-playground` depends on pass:

`cargo clippy --target wasm32-unknown-unknown -p temper-playground -- -D warnings`

as far as jig's own code goes. For each lint expectation met on one target and not the other, prefer `cfg_attr(not(target_arch = "wasm32"), expect(..., reason = "..."))`, or make the code right for both. Never `#[allow]` (`AGENTS.md` 2). Also run the native clippy, so both targets stay green.

One task in jig; no design. If temper's own code then fails, name it in the report. Do not fix it here: plan `playground-browser` owns `temper-client-shell-web`.
