+++
id = "bx-script-forge"
part = "box-groups"
kind = "feature"
needs = ["bx-commands"]
files = [
  "temper/crates/temper-fake/src/llm.rs",
  "temper/crates/temper-fake/src/tests.rs",
  "temper/crates/temper-fake/src/tests/llm_forge.rs",
  "temper/crates/temper/src/fake.rs",
  "temper/crates/temper/src/fake/tests.rs",
  "temper/crates/temper/src/setup.rs",
  "temper/crates/temper/src/setup/tests.rs",
]
effort = "high"
minutes = 40
+++

## Goal

The playground's scripted model delegates on the playground's forge
whatever its number. `application.demo.llm`'s scripts name the playground's
repository by its resource tag for forge 1 (`4294967297`, forge 1's
repository 1: `temper/crates/temper-init/src/tests/documents.rs`, the
comment before the CSV chat's test); forges are numbered at runtime and
never reused (contracts, section 2), so after a reset the next playground's
forge is 2 or more. `temper fake llm DIRS` (the deployment form, run first
in the playground's group) finds the playground's forge N from its group
file and serves its scripts with every resource tag of forge 1 given forge
N instead. This answers the playground designer's contract problem in
`temper/docs/plan/playground-reset-r4/notes/progress-playground.md`.

Acceptance:

- `temper_fake::llm::Document::for_forge(self, forge: u16) -> Result<Document, Refusal>`
  (a name this task creates; `Err` is the joint-bound refusal below): every `resource` member of every tool call's
  arguments in its scripts (cued, asked and otherwise), at any depth, whose
  value is an integer tag of forge 1 (`tag >> 32 == 1`), becomes `(N << 32)
  | (tag & 0xffff_ffff)`; nothing else of the arguments changes (members'
  order, other numbers, strings, whitespace aside: the arguments are
  re-encoded only when a tag changes); with N = 1, the document is unchanged.
- The binary's `fake llm` deployment form (`fake.rs`, `Source::Demo`) reads
  `<data>/box/groups/` for the one file whose `playground` is true (through
  `temper_forge_setup::file::read`); with one, it serves `for_forge(N)`; with
  none, the scripts as written (the document's own playground forge, D1);
  with more than one, the highest N, named on stderr.
- The rewritten scripts stay within the fake's byte bounds: `for_forge`
  checks them again through `check_scripts` and returns its `Err`
  (`scripts past their joint byte bound`) if a longer number pushes past
  it; `llm_configuration` maps that refusal to its `String` error as
  `Source::Document` does.
- The tests below pass, and temper-fake's and the binary's existing tests.

## Inputs

- `temper/docs/plan/playground-reset-r4/notes/common.md` and
  `temper/docs/plan/playground-reset-r4/notes/contracts.md` (sections 2, 3).
- `temper/docs/design/protocol/setup.md`, 8.4 (the demo's fake LLM and its
  scripts).
- `temper/docs/plan/playground-reset-r4/notes/progress-playground.md`,
  `## Contract problems` (the scripted model's scripts name forge 1).
- The code: `temper/crates/temper-fake/src/llm.rs` (`Document`, `decode`,
  the scripts' reading), `skein/crates/skein-fake-llm-domain/src/api.rs`
  (`Scripts`, `Script`, `Turn`, `Line::Call`'s `arguments`),
  `temper/crates/temper/src/fake.rs` (`llm_configuration`), bx-commands'
  `setup::entry` and bx-kinds' `temper_forge_setup::file`.

## Owns and may change

- Owns the files in the header; `tests/llm_forge.rs` is new, `tests.rs`
  takes its `mod` line only.
- Neighbours: none.
- Options:
  - The rewrite: parse each call's arguments with `skein_json` and re-emit
    with its writer (default), or rewrite the number's token bytes in place
    (no re-encoding) if the tokenizer gives each number's byte range; either
    keeps the bytes of everything else.
  - Finding N: a function in the binary's `setup.rs`,
    `pub(crate) fn playground_forge(directories: &super::Directories) ->
    Result<Option<u16>, String>` (default), beside bx-commands' `entry`.

## Steps

1. `llm.rs`: `Document::for_forge` as above, over `scripts.cued`,
   `scripts.asked` and `scripts.otherwise`'s `Line::Call { arguments, .. }`.
2. `setup.rs`: `playground_forge` (Options); `fake.rs`'s `Source::Demo` arm
   calls it and `for_forge` before `llm::configuration`.
3. Tests:
   - `temper-fake/src/tests/llm_forge.rs`:
     `the_templates_csv_chat_delegates_on_forge_n` (the init template's
     `application.demo.llm` decoded, `for_forge(5)`: the CSV script's
     `resource` is `21474836481`);
     `forge_one_leaves_the_scripts_as_they_are`;
     `other_numbers_and_another_forges_tags_are_kept`;
     `a_rewrite_past_the_joint_bound_is_refused`.
   - `temper/src/setup/tests.rs`: `the_playground_forge_is_its_group_files_number`,
     `no_playground_file_is_none`.
4. `cargo fmt`; `heavy cargo clippy -p temper-fake -p temper --all-targets
   -- -D warnings`; `heavy cargo nextest run -p temper-fake --lib` and
   `-p temper --lib`; dependents build: `heavy cargo check -p temper-service
   --tests`. Commit, push.

## Report

Fab's sections; under `## Interfaces`: `llm::Document::for_forge`,
`setup::playground_forge`, the rule for which tags change.

## Answers

2026-11-01. May `Document::for_forge` return `Result<Document, Refusal>` to
refuse a rewrite past the joint byte bound? Yes (option A): the signature is
`pub fn for_forge(self, forge: u16) -> Result<Document, Refusal>`. It runs
`check_scripts` over the rewritten scripts (the same check `decode` makes, so
`Document::domain`'s "checked scripts" invariant holds) and returns its
refusal; `configuration` and `domain` stay infallible. In `fake.rs`'s
`Source::Demo` arm, clone `demo.llm()`, call `for_forge(n)` and map the
refusal with `String::from_utf8_lossy(&why.line())` as the `Source::Document`
arm does. `a_rewrite_past_the_joint_bound_is_refused` asserts the `Err` and
its diagnostic.
