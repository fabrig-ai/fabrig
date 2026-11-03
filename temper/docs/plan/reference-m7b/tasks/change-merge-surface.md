+++
id = "change-merge-surface"
part = "change-merge"
kind = "interface"
needs = []
files = ["temper/crates/temper-protocol-store/schema/store-v11.schema", "temper/crates/temper-protocol-store/src/**", "temper/crates/temper-protocol-store/golden/**", "temper/crates/temper-engine-domain-forge-change/src/boundary.rs", "temper/crates/temper-engine-domain-forge/src/boundary.rs", "temper/crates/temper-engine-domain-forge/src/items.rs"]
effort = "medium"
minutes = 110
+++

## Goal

The shapes the merge job's change steps build on, behaviourless: `Delegate::Merge`, the facts
`merge_job` and `contains_catch_up`, the change row's `catch_up` and `merge_result`, their
stored records, and the merge job's workspace item (`notes/contracts.md`, C2 to C4). No change
makes a merge job yet: `change-merge-steps` and `change-merge-route` give the arms their
behaviour.

Acceptance:

- C3's `Delegate::Merge { base, from }` and `Facts::{merge_job, contains_catch_up}`, and the
  forge's `ChangeRow::{catch_up, merge_result}`, exist as C3 names them;
- C4's schema changes are in `store-v11.schema` in place, the generated code and goldens
  regenerated, and a stored change row with `Delegate::Merge` and both new fields set
  round-trips through the store's records (`temper-protocol-store/src/records/change.rs`);
- the forge's `items.rs` gives `Delegate::Merge` C2's item: the change's repository, writable,
  `Start::Merge { branch, base }` (`branch` the change's branch where `from` is `Some`, its
  landing branch where `from` is `None`), its push the change's branch, its holder the change,
  named `home`;
- every other forced arm is inert, as Steps says, and every existing test passes unchanged.

## Inputs

- `temper/docs/design/domain/forge.md, 8.4` ("The merge job"), `8.2` step 1 (a catch-up);
  `temper/docs/design/domain/worker.md, 4.4` ("The merge kind").
- `temper/docs/plan/reference-m7b/notes/contracts.md`, C2, C3, C4; `notes/common.md` (schemas in
  place).

## Owns and may change

- `temper/crates/temper-protocol-store`: the schema, `src/records/change.rs`, its generated
  `v11.rs`, goldens, `src/tests/fixtures.rs` and `src/tests/records.rs`.
- `temper/crates/temper-engine-domain-forge-change/src/boundary.rs`: `Delegate::Merge`, the two
  `Facts` fields.
- `temper/crates/temper-engine-domain-forge/src/boundary.rs`: `ChangeRow`'s two fields, and their
  save and restore wherever the row is translated; `src/items.rs`: the item.
- Forced arms and literals, inert, in any file of temper: `Facts` literals take
  `merge_job: false` and `contains_catch_up: Status::Pending` (the forge's `domain.rs` facts
  builder, the forge-change crate's test `facts()` and `prepare_facts()`); `ChangeRow` literals
  `catch_up: None, merge_result: None`; a `Delegate` match in the forge-change domain treats
  `Merge` as it treats `Resolve` where the arm reads a delegate's kind (a result moves the change
  as a resolution's would); in the forge's `domain.rs` and the root's `assemble.rs`, `Merge`
  takes `Resolve`'s arm; in `temper-engine-domain/src/route/forge_route.rs`'s `change_delegate`,
  the `Merge` arm returns `None` (no delegate is made: the change never asks for one yet), with
  a one-line comment naming `change-merge-route`. No `panic!`, `unreachable!` or `_` arm.
- Not this task's: any behaviour, `forge_route.rs` beyond that arm, the change tool.

## Steps

1. The schema (C4), regenerated; the records' translation both ways, exhaustive.
2. The domain types (C3) and the item (C2).
3. Tests:
   - `a_change_row_with_a_merge_delegate_and_a_catch_up_round_trips` in the store's
     `src/tests/records.rs`, with fixtures for the new arm;
   - `a_merge_delegate_asks_for_a_merge_in_progress_named_home` in the forge's tests (beside the
     resolution's item test, if one exists; else in `src/tests.rs`): `from: Some(_)` starts on the
     change's branch, `from: None` on its landing branch, both pushing the change's branch.
   Each cites `temper/docs/design/domain/forge.md, 8.4`.
4. `SKEIN_CODEGEN_WRITE=1 heavy cargo nextest run -p temper-protocol-store -E 'test(drift::)'`
   (the store's drift test; find its exact filter in the crate), again without the variable,
   `git diff --stat` reads only the schema, generated code and goldens there; then
   `heavy cargo nextest run -p temper-protocol-store`, `-p temper-engine-domain-forge-change
   --lib`, `-p temper-engine-domain-forge --lib`, `-p temper-engine-domain --lib`; `cargo fmt`;
   `python3 -I -B temper/scripts/old_forms.py`; clippy on the four packages,
   `--all-targets -- -D warnings`.
5. Commit and push.

## Report

As the implementer's text says; list every file with a forced arm.

## Answers

2026-11-03. **May `src/tests/sizes.rs`'s `APPLICATION_ROW_BYTES` and its derivation comment
change, or should C4 be revised?** Both, as follows. `sizes.rs` is in the task's `files`, and a
literal that prices the schema the task changes is not an "existing test" the acceptance means:
update the literal and its derivation comment to what the generated bound and the application's
sizing return, the assertions otherwise unchanged. C4's `merge_result` is revised to `option
bytes 512` (`notes/contracts.md`, R2: the merge kind's result is 512 bytes, the tasks child's
bound), so regenerate the schema, generated code and goldens again; by the run's own accounting
(33 for `catch_up`, 517 for `merge_result`, 25 for the delegate) the literal becomes 119,767;
take the value the code returns and say in the report if it differs. Nothing else of the pushed
work changes.
