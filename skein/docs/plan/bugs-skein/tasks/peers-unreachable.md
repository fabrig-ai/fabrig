+++
id = "peers-unreachable"
part = "fake-peers"
kind = "feature"
needs = []
files = ["skein/crates/skein-fake-peers/Cargo.toml", "skein/crates/skein-fake-peers/src/llm.rs", "skein/crates/skein-fake-peers/src/oauth.rs", "skein/crates/skein-fake-peers/src/forge.rs", "skein/crates/skein-fake-peers/src/transport/native.rs"]
effort = "medium"
minutes = 15
+++

# fake-peers' impossible cells are `unreachable!`

## Goal

`skein-fake-peers` writes `panic!` for its impossible io cells. They are unreachable, and the rules say how to
write one: `unreachable!("why")` (`programming-model.md`, 5.4), with `panic!` out (10.3) and a step-machine fake
held to it (`testing-strategy.md`, 4). The crate's own lint table lacks `panic = "deny"`, which the root has.
(Bug 19, its fake-peers half, re-checked: holds as a breach of the rule; no runtime bug.)

Acceptance:

- `llm.rs`, `oauth.rs`, `forge.rs` (the io cell) and `transport/native.rs`: `unreachable!`, the messages kept.
- `forge.rs`'s `setup_mut`: the same; its doc says "Panics for a GitHub face", which stands.
- `[lints.clippy]` in the crate's `Cargo.toml` has `panic = "deny"`.

**What stands in for a test that fails on the old code** (`notes/common.md`): the cells are unreachable through
any public entrance, so no test can reach them. The lint is the guard: add it first, run clippy, quote its five
findings on the old code, then fix them.

## Inputs

- `skein/docs/plan/bugs-skein/notes/common.md`.
- `skein/docs/foundation/programming-model.md`, 5.4, 10.3; `skein/docs/foundation/testing-strategy.md`, 4.

## Owns and may change

- Owns: the crate's `Cargo.toml` (the one lint line); the sites in the four files.
- No reader changes: no test asserts these messages. `skein/clippy.toml`'s `allow-panic-in-tests` should cover
  `transport/tests.rs`; if it does not, a scoped `#[expect(clippy::panic, reason = "...")]` there.
- `temper/crates/temper/src/fake/control.rs` has the same shape: another tree's, `## Left`.

## Steps

1. The lint line. `heavy cargo clippy -p skein-fake-peers --all-targets -- -D warnings`; quote it.
2. The five sites.
3. Clippy again, `heavy cargo nextest run -p skein-fake-peers`, `-p skein-fake-peers-tests`. `cargo fmt`,
   `python3 -I -B skein/scripts/old_forms.py`.
4. Commit, push.

## Report

As the implementer's text says.
