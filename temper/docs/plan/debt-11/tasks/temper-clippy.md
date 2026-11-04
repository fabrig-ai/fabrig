+++
id = "temper-clippy"
part = "temper"
kind = "migration"
needs = ["temper-profiles"]
files = ["temper/clippy.toml", "temper/**/clippy.toml", "temper/tools/clippy.toml", "temper/crates/temper/tests/support/**", "temper/crates/temper-client-bundle/build/**", "temper/testing/**", "temper/crates/**/*.rs", "temper/tests/**/*.rs", "temper/tools/**/*.rs"]
effort = "high"
minutes = 60
+++

# temper: every `clippy.toml` a variant

## Goal

Take temper's `clippy.toml` files to `step` and `ordinary`, per the contracts' section 4, with one copy for the five
tools. temper has no `ring` crate. The headers go true: "The fake is ordinary deterministic test code" sits on
`temper`'s own.

Acceptance:

- `python3 -I -B tools/lints/lints.py temper` reads 0 findings.
- `heavy cargo clippy -p PKG ... --all-targets -- -D warnings` passes for every temper package a changed `clippy.toml`
  reaches. `temper/clippy.toml` and `temper/tests/clippy.toml` reach most of the tree: run `heavy cargo clippy` with a
  `-p` for each temper package.
- `heavy cargo clippy --target wasm32-unknown-unknown` passes for each temper package in `.config/gate/wasm32.txt`.

## Inputs

- `temper/docs/plan/debt-11/notes/common.md`
- `temper/docs/plan/debt-11/notes/contracts.md`, 1 and 4
- `tools/lints/design.md`, 4

## Owns and may change

- **Owns:** every `clippy.toml` under `temper/`, and the new `temper/tools/clippy.toml`.
- **May change:** temper's code, for what a variant turns red, by `notes/common.md`'s list.
- **Overlap:** `macos-cheap` is changing `temper/crates/temper/tests/support/parent.rs` and `csv_repair.rs`, which this
  task doesn't need. Keep your edits in `tests/support/` to attribute lines.
- **Options:**
  - `std::env::var` in `temper/crates/temper/tests/support/relay_managed.rs`, `live_github.rs` and `managed_client.rs`
    reads what a live or relay test is handed. `thread::scope` in `relay_managed.rs`, `account_move.rs` and
    `forge_land/rung.rs` runs what those tests need beside the world. Each takes a scoped
    `#[expect(clippy::disallowed_methods, reason = "...")]`.
  - `temper-client-bundle/build/main.rs` reads `PROFILE`, which cargo sets for a build script: the same, with that
    reason.
  - `temper-fixtures` and `temper-github-app` go from empty lists to `ordinary`. What fires is a scoped exception, or
    `BTreeMap` for a `HashMap` where its order reaches nothing that a test reads.
  - `temper-client-shell-web` and `skein-ui-example-shell-web` had their own reasons for one ban list. The bans are
    the same.

## Steps

1. Copy the variants by the contracts' section 4. Add `temper/tools/clippy.toml` and delete the five under it.
2. Clippy, as above. Fix by `notes/common.md`'s order. Cargo tracks a `clippy.toml` a crate read; if a result looks
   stale, `heavy cargo clean -p PKG` in your slot.
3. Fmt, old forms, the check, citations.
4. Tests: `heavy cargo nextest run -p PKG` for each package whose code changed beyond a deleted or added attribute.
5. Commit and push.

## Report

The report of your implementer text, and: each ban you saw move, by crate, against the contracts' section 4; the
exceptions you added, each with its reason; the branch's net lines.
