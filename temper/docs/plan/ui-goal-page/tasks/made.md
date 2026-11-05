+++
id = "made"
part = "made"
kind = "interface"
needs = []
files = [
  "jig/crates/jig-domain-core/src/documents/task_tree.rs",
  "jig/crates/jig-client-protocol-wire/schema/client-v19.schema",
  "jig/crates/jig-client-protocol-wire/src/generated/v19.rs",
  "jig/crates/jig-client-protocol-wire/golden/**",
  "jig/crates/jig-client-domain/src/contract/documents.rs",
  "jig/crates/jig-protocol-engine-clients/src/translate/task.rs",
  "jig/crates/jig-client-protocol-link/src/translate/documents_part3.rs",
  "jig/tests/core/tests/suite/fields.rs",
]
effort = "medium"
minutes = 60
+++

# A task's document says when it was made

## Goal

`made: Wall` on the client's `Task` document, the record's `created_at`, through `jig-domain-core`'s
document, the wire and the client's contract. Temper's goal page says "since" and this time; this task draws
nothing. Nothing else in jig changes.

Acceptance:

- `made` stands on all three types as `temper/docs/plan/ui-goal-page/notes/contracts.md`, When a task was
  made, gives it, and through both translators.
- The world's test below passes.
- The drift tests and goldens of the client's wire agree with its schema, changed in place
  (`jig/docs/design/protocol/README.md`, 7).
- Your branch closes `jig/docs/design/client/domain/contract.status.md`'s line for 5.1's `made`.

## Inputs

- `temper/docs/plan/ui-goal-page/notes/common.md`, `temper/docs/plan/ui-goal-page/notes/contracts.md`.
- `jig/docs/design/client/domain/contract.md`, 5.1, *a task*.
- `jig/docs/design/protocol/README.md`, 7; `skein/docs/design/codec.md` on generated code, golden bytes and
  drift.

## Owns and may change

Owns, as `files` lists them. This task leaves jig for one thing alone: the other tree's lines that name the
required field.

May also change, each only for the field:

- `Task` literals in jig: `jig/crates/jig-client-domain/src/tests/fixtures.rs`,
  `jig/crates/jig-client-view/src/tests/**`, `jig/crates/jig-client-protocol-link/src/tests/fixtures/documents.rs`,
  `jig/crates/jig-client-protocol-wire/src/tests/fixtures/task_values.rs`,
  `jig/crates/jig-protocol-engine-clients/src/tests/fixtures/documents.rs`,
  `jig/testing/jig-client-scripted-engine/src/codec/values/documents.rs`,
  `jig/testing/jig-client-test-records/src/lib.rs`, `jig/tests/client/src/stories.rs`,
  `jig/tests/client/src/stories/text_pages.rs`, `jig/tests/client/tests/suite/fuzzy/memory.rs`,
  `jig/examples/ops/client-view/src/tests.rs`.
- `Task` literals in temper: `temper/tools/gallery/src/worlds.rs`,
  `temper/crates/temper-client-view/tests/suite/{task_pages.rs,change_page.rs}`,
  `temper/crates/temper-client-view/src/tests/fixtures/task.rs`,
  `temper/crates/temper-client-view/src/tests/chat.rs`,
  `temper/crates/temper-client-tools/src/tests/fixtures.rs`,
  `temper/crates/temper-client-domain/src/tests/fixtures.rs`.
- The wire's fixtures and goldens that embed a `Task`, and any layout digest a test holds.

Options, each pre-approved:

- If a further literal or part shows up, the compiler's list wins: edit it, only for the field.
- Where a literal has no time at hand, give it the literal's own `last_activity`, or `Wall::from_nanos(0)`.
- `temper-client-tools/src/jig/render/document/`: don't print `made`; `ctl task show` doesn't change. If the
  compiler asks for an arm, give it what prints nothing.
- If `Task`'s wire record is too long for a limit the codec holds, say which and raise that limit where it
  stands: formats change in place.

## Steps

1. **The document:** `pub made: Wall` after `last_activity` in `documents/task_tree.rs`'s `Task`;
   `made: row.created_at` in `task_with_lineage`, beside `last_activity: row.last_activity`.
2. **The wire and the contract:** the schema's `record Task`, generated code, both translators
   (`translate/task.rs`'s `last_activity: value.last_activity.as_nanos()`, `documents_part3.rs`'s
   `last_activity: Wall::from_nanos(value.last_activity())`), the contract. Regenerate the generated code and
   the goldens as the drift tests say: never by hand.
3. **The test,** in `jig/tests/core/tests/suite/fields.rs`, its world beside
   `a_chats_flags_and_last_activity_survive_a_cold_restart`:
   `a_tasks_document_says_when_it_was_made`: set the clock, make a task, move the clock and say something to
   it; read its document: `made` is the first wall, `last_activity` the second; after a cold restart, the
   same.
4. **The readers,** since the format moves: `heavy cargo nextest run -p jig-domain-core -p
   jig-client-protocol-wire -p jig-client-protocol-link -p jig-protocol-engine-clients -p jig-client-domain -p
   jig-client-view -p jig-client-scripted-engine -p jig-core-world`, then `heavy cargo check --tests -p
   temper-client-domain -p temper-client-view -p temper-client-tools -p temper-client-world -p
   temper-client-gallery`.
5. **Check,** the implementer's text, then push.

## Report

As `notes/common.md`, Your report, with how many goldens moved and how you regenerated them.
