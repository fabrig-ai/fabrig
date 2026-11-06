+++
id = "admit-policy"
part = "admit-policy"
kind = "feature"
needs = []
files = ["skein/crates/skein-llm-connection/src/component.rs", "skein/crates/skein-llm-connection/src/tests.rs"]
effort = "medium"
minutes = 20
+++

# `admit_policy` goes, and `admit` is test-only

## Goal

Finding 1 of `temper/docs/plan/debt-29/inputs/debt27-review.md`, its second half. `skein-llm-connection/src/component.rs`'s `pub(crate) admit_policy` takes a `drop_reasoning: Option<bool>`, and only one test passes `Some`. The cut went further than the finding: the public `admit` above it has no caller outside this crate's tests, and the one shipping path, `start`, always passes `Some(drop_reasoning)`. So the `Option` exists for tests alone.

skein's D040 said what such an item becomes: deleted, called from shipping code, or made test-only. `admit` is made test-only, `admit_policy` folds into it, and `measure_start` takes a plain `bool`. A removal this spec names is agreed.

Acceptance:

- `rg -n 'admit_policy' skein -g '*.rs'` finds nothing.
- `rg -n 'drop_reasoning: Option<bool>' skein/crates/skein-llm-connection` finds nothing.
- `admit` is `#[cfg(test)]` and no longer `pub`; `cargo doc`'s public face of the crate loses it.
- `measured_admission_keeps_each_providers_lengths_and_refusals` runs both values of `drop_reasoning` still, with its `expected` and its two assertions as they are.
- The net is fewer lines.

## Inputs

- `temper/docs/plan/debt-29/inputs/debt27-review.md`, finding 1.
- `skein/docs/design/llm-connection.md`, 4, for what a start admits.
- `temper/docs/plan/debt-29/notes/common.md`.

## Owns and may change

Owns the two files in the header. No other crate is touched: `rg -n 'admit\(' skein smith jig temper -g '*.rs'` shows the cut other types' `admit`s and none of this one outside the crate; check again before deleting the `pub`. If a shipping caller appears, stop at `admit_policy` and keep `admit` public, and say so.

`admit`'s shape is yours within this: `#[cfg(test)] pub(crate) fn admit(&mut self, call, endpoint, prompt, credential, drop_reasoning: bool)`, with the about eight test calls in `tests.rs` and `component.rs`'s inline tests passing `false`, is the default. The field `destination.limits.drop_reasoning` isn't read there any more; `start`'s caller decides, as `smith-protocol-llm/src/component.rs` does, and the field stays for what else reads it.

If a test's outcome depended on `None` reading the endpoint's limit, stop and ask.

## Steps

`measure_start`'s last parameter becomes `drop_reasoning: bool`; the `match` on it goes, and `start` passes its value directly. Move `admit` beside the crate's other test-only code, with `admit_policy`'s body.

In the test, `pool.admit(Token::new(7), 0, input, Some(value), enabled)`.

No new test. `heavy cargo nextest run -p skein-llm-connection`, then the world that reads this crate: `heavy cargo nextest run -p` on `skein/tests/llm-connection`'s package (`rg -m1 '^name' skein/tests/llm-connection/Cargo.toml`). `heavy cargo check --tests -p smith-protocol-llm`. `cargo fmt`; `(cd skein && python3 -I -B scripts/old_forms.py)`; `heavy cargo clippy -p skein-llm-connection --all-targets -- -D warnings`. Commit, push.

## Report

As your text says, with the two searches' readings and the net lines.
