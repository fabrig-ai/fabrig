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
  "temper/crates/temper-playground/src/documents.rs",
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

## Answers

2026-11-03. **Q: the playground's `world_document` applies the chat stories' policy
unconditionally; the one-process copy only under `stories::uses_stock()`, after
`stories::configure`. A seam in the playground, or reconstruct the values here?**
A seam in the playground, made on the answerer's branch (unbuilt: heavy was not reachable from
the answer call; this run builds and checks it). `temper_playground::documents` now has:

- `world_document_with(edit: impl FnOnce(&mut Value)) -> Value`: the same steps as before, with
  `edit` applied after `stage_policies` and the engine's members' removal, before the shared
  limits (`people.amendment_bytes` ...) and `turn_bounds::apply`;
- `chat_policy(&mut Value)`: the block that was inline (seed projects on policy 2, delegation
  depth 4, the five chat limits);
- `world_document()` = `world_document_with(chat_policy)`, its value unchanged.

The one-process world's `world_document` becomes
`documents::world_document_with(|value| { stories::configure(value); if stories::uses_stock() {
documents::chat_policy(value) } })`, then `lower::configure`, `lower::github::configure`,
`m15::configure`. Moving `stories::configure` after the members' removal changes nothing (it
touches no `engine.charters/templates/policies/rules`), and the harness's three configures after
`turn_bounds::apply` change nothing (they set none of its keys): this is no reconstruction and
every value stays as today. Do not undo any of the playground's edits. You may change
`temper/crates/temper-playground/src/documents.rs` only to repair this seam if it fails to
build or lint; add `heavy cargo nextest run -p temper-playground --lib` and
`heavy cargo clippy -p temper-playground --all-targets -- -D warnings` to step 3.

The same rule for any other item (`files`' held-change handover, an agent edit): a difference
that is a story's or the harness's conditional edit stays here, applied over the playground's
item; a value that differs unconditionally is still a question.

**Q: `playground-documents`' report is absent.** Its list of public items is
`temper/crates/temper-playground/src/documents.rs`, `documents/*.rs` and `fakes.rs`, `fakes/*.rs`
as merged on the trunk: read the public items there, each named as the one-process item it
copies (`playground-documents`' spec lists the sources). Name each one you replace in your
report.
