+++
id = "delegate-counts"
part = "delegate-counts"
kind = "interface"
needs = []
files = [
  "jig/crates/jig-domain-tasks/src/boundary.rs",
  "jig/crates/jig-domain-tasks/src/domain.rs",
  "jig/crates/jig-domain-tasks/src/**/closing.rs",
  "jig/crates/jig-protocol-store/schema/tasks-v9.schema",
  "jig/crates/jig-protocol-store/src/generated/tasks_v9.rs",
  "jig/crates/jig-protocol-store/src/codec/tasks/record.rs",
  "jig/crates/jig-protocol-store/golden/tasks/**",
  "jig/crates/jig-domain-core/src/documents/task_lines.rs",
  "jig/crates/jig-client-protocol-wire/schema/client-v19.schema",
  "jig/crates/jig-client-protocol-wire/src/generated/v19.rs",
  "jig/crates/jig-client-protocol-wire/golden/**",
  "jig/crates/jig-client-domain/src/contract/queries.rs",
  "jig/crates/jig-protocol-engine-clients/src/translate/queries.rs",
  "jig/crates/jig-client-protocol-link/src/translate/queries.rs",
]
effort = "high"
minutes = 90
+++

# A task line counts its direct delegates

## Goal

Every list of tasks says how many of a task's direct delegates are done, and how many there are, cancelled
ones left out, true past any page, at no cost per row. Temper's Goals page draws it as "6 / 9"; this task
draws nothing.

Acceptance:

- `TaskRecord.ended_delegates` and `contract::TaskLine.delegates_done` and `delegates_total` as
  `temper/docs/plan/ui-goals/notes/contracts.md`, A line's counts of its delegates, gives them, on the store
  and the client's wire, and through every translation between.
- The world's test below passes, with more ended delegates than one page of `rows` holds.
- The drift tests and goldens of both families agree with their schemas, changed in place
  (`jig/docs/design/protocol/README.md`, 7).
- The status lines this task closes are gone: `jig/docs/design/client/domain/contract.status.md` and
  `jig/docs/design/domain/tasks.status.md` (the *Not yet* naming these counts).

## Inputs

- `temper/docs/plan/ui-goals/notes/common.md`, `temper/docs/plan/ui-goals/notes/contracts.md`.
- `jig/docs/design/domain/tasks.md`, 3, *Its delegates' endings*.
- `jig/docs/design/client/domain/contract.md`, 5.1, *a task line*.
- `jig/docs/design/protocol/README.md`, 7; skein's `skein/docs/design/codec.md`, on generated code, golden
  bytes and drift.

## Owns and may change

Owns, as `files` lists them. This task leaves jig for one thing alone: the other tree's lines that name the
required field.

May also change, each only for the fields:

- `TaskRecord` literals: `jig/crates/jig-protocol-store/src/tests/fixtures/record.rs`,
  `jig/tests/store/src/root.rs`, `jig/testing/jig-test-store-files/src/tests/routes/fixtures.rs`,
  `temper/crates/temper-service/src/tests/fixtures.rs`.
- `TaskLine` literals: `jig/crates/jig-protocol-engine-clients/src/tests/fixtures/live.rs`,
  `jig/crates/jig-client-scripted-engine/src/queries.rs`, `jig/crates/jig-client-view/src/tests/fixtures.rs`,
  `jig/crates/jig-client-view/src/state/domain/{pages.rs,owner_inbox.rs}`,
  `jig/tests/client/**/stories.rs`, `jig/tests/client/**/stories/text_pages.rs`,
  `temper/crates/temper-client-domain/src/tests/pages/chats.rs`,
  `temper/crates/temper-client-tools/src/tests/fixtures/reads.rs`,
  `temper/tests/client/tests/suite/{task_pages.rs,change_page.rs}`,
  `temper/crates/temper-client-view/src/tests/chats.rs`, `temper/tools/gallery/src/worlds.rs`.
- `TaskLineParts`: `jig/crates/jig-client-scripted-engine/src/codec/values/queries.rs`,
  `jig/crates/jig-client-test-records/src/live_values.rs`. A wire fixture or a golden that embeds a line
  (`record_task_line_*`, `record_inbox_entry_*`, and those the drift test names).
- The store's layouts where a test holds their digest (`jig/tests/codec-families.rs`,
  `jig-protocol-store/src/tests/settings_layout.rs`).

Options, each pre-approved:

- If the field's width elsewhere is another integer, use `u32` on the line and convert as the subset allows.
- If a further literal or part shows up, the compiler's list wins: edit it, only for the fields.
- `temper-client-tools/src/jig/render/document/read_queries.rs`, `encode_task_line`: if the tools' output is
  a golden, leave it; don't print the counts.

## Steps

1. **The counter:** `ended_delegates: Endings` on `TaskRecord` beside `ended_below`, in the record's schema,
   its codec and its golden bytes. In `closing.rs`, where the parent removes the ended delegate and adds its
   `ended_below`, add that one's ending to `ended_delegates`. Read `moving.rs`: a moved delegate adds nothing.
2. **The row:** `documents/task_lines.rs`, `delegates_done` and `delegates_total` as the note says, on both
   the live and the ended paths: an ended row's counter is final, since a requester ends after its delegates.
3. **The wire and the contract:** the client's schema's `TaskLine`, generated code, both translators, the
   contract, the goldens. Regenerate the generated code and the goldens as the drift tests say: never by hand.
4. **Tests, in `jig/tests/core/tests/suite/ended_indexes.rs` or a neighbour of its world:**
   - `a_line_counts_its_direct_delegates_past_a_page`: more delegates than `rows` (8 in standard): some
     done, one failed, one cancelled, one live; the parent's line says done and total, the cancelled left out.
   - `a_grandchild_is_not_a_delegates_count`: a delegate's delegates count on its line alone.
   - `an_ended_parents_line_keeps_its_counts`, read from the ended group.
5. **A change of meaning:** none to what a line already holds, but the formats move, so run the readers:
   `heavy cargo nextest run -p jig-domain-tasks -p jig-domain-core -p jig-protocol-store -p
   jig-client-protocol-wire -p jig-client-protocol-link -p jig-protocol-engine-clients -p jig-client-domain
   -p jig-client-view`, then `heavy cargo check --tests -p temper-client-domain -p temper-client-view -p
   temper-client-tools -p temper-service -p temper-client-world -p temper-client-gallery` (take each package's name from its `Cargo.toml`
   where these differ).
6. **Check,** the implementer's text, then push.

## Report

As `notes/common.md`, Your report, with how many goldens moved and how you regenerated them.

## Rebase (2026-11-05, step merge, foreman ui-goals/delegate-counts/foreman-1)

regenerate Story A’s log after rebasing onto the updated integration branch.
The conflict is in `temper/tests/playground/tests/fixtures/story-a.log`.
Task commit `630edb1688` refreshes its hashes for delegate counts; integration commit `ecf06ea9ec` (“Name brief task scopes so runs can address notes”) refreshes overlapping hashes for brief text.
Preserve both code changes and regenerate the fixture from the combined code; do not hand-merge hashes or keep either side’s log wholesale.
`temper/docs/plan/ui-goals/notes/common.md` authorizes regenerating stale goldens and requires regeneration on catch-up.
The writer is `temper/tests/playground/tests/suite/walk.rs`, enabled by `SKEIN_CODEGEN_WRITE`.
Run `SKEIN_CODEGEN_WRITE=1 TEMPER_BUNDLE=none heavy cargo nextest run -p temper-playground-world --test suite -E 'test(=walk::story_a_walks_from_the_welcome_to_the_csv_change_landed)'`.
Then verify replay with `TEMPER_BUNDLE=none heavy cargo nextest run -p temper-playground-world --test suite -E 'test(=walk::story_a_kept_log_replays_natively)'`.
Keep the resolution limited to the regenerated fixture.
