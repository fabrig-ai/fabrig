+++
id = "one-process-documents"
part = "composition"
kind = "migration"
needs = ["playground-documents", "hosted-git-moves"]
files = [
  "temper/tests/one-process/Cargo.toml",
  "temper/tests/one-process/src/lib.rs",
  "temper/tests/one-process/src/fixtures.rs",
  "temper/tests/one-process/src/stories.rs",
  "temper/tests/one-process/src/restart_control.rs",
  "Cargo.lock",
]
effort = "medium"
minutes = 60
+++

# The one-process world takes its composition's documents from `temper-playground`

## Goal

The one-process world's composition and the playground's are one (`temper/docs/design/playground.md`,
3; `temper/docs/design/domain/testing.md`, 2.5). Its documents come first: the values
`playground-documents` copied into `temper_playground::{documents, fakes}` are read from there,
and the one-process world's own copies, and the test kits it took them from, go. Its stories
change no meaning: every test passes with the same counts and verdicts.

Acceptance:

- `temper/tests/one-process/src/fixtures.rs`, `stories.rs` and `restart_control.rs` hold no copy
  of an item `temper_playground::documents` or `fakes` holds: each such item is deleted and its
  uses read the playground's (a re-export or a `use`, as you choose). The harness's own edits of
  the deployment's document (~139, ~181-183) stay here, applied to the playground's document.
- `lib.rs`'s `#[path = "../../real-loop/src/settings.rs"] mod settings;` goes if the playground's
  copy replaces every use; `fixtures.rs`'s path-included `turn_bounds` likewise.
- `Cargo.toml` gains `temper-playground.workspace = true` and drops each of
  `jig-channel-world`, `smith-agent-process-world` and `skein-llm-world` that nothing in the crate
  uses any more (one still used by the harness stays; say which and where).
- `heavy cargo nextest run -p temper-one-process-world` passes with the trunk's count.

## Inputs

- `temper/docs/plan/playground-temper/notes/common.md` and `notes/contracts.md`, C1 and C3.
- The report of `playground-documents`: its list of public items and the item each copies.

## Owns and may change

- `temper/tests/one-process/Cargo.toml`, `src/lib.rs` (module lines only), `src/fixtures.rs`,
  `src/stories.rs`, `src/restart_control.rs` (deleted if empty; its `mod` line then goes).
- Reader files of the deleted items, in this crate only, edited in `use` lines and paths alone:
  `src/world.rs`, `src/machine.rs`, `src/referee*.rs`, `src/referee/**`, `src/oracle.rs`,
  `src/ctl.rs`, `src/m15*.rs`, `src/m15/**`, `src/lower*.rs`, `src/lower/**`,
  `tests/suite/**`. Nothing else of `world.rs` changes: `one-process-composed` owns it next.
- If a playground item differs from the one-process copy it replaces (a value, a field), do not
  change either: stop, that is a question naming both.

## Steps

1. Replace the copies, item by item; `heavy cargo check -p temper-one-process-world --tests`.
2. The manifest; `heavy cargo nextest run -p temper-one-process-world`, its count against the
   trunk's.
3. `cargo fmt`; old forms for temper; `heavy cargo clippy -p temper-one-process-world
   --all-targets -- -D warnings`. Commit; push.

## Report

As the implementer's text says, and: each item deleted and what replaces it; the dependencies
dropped or kept.
