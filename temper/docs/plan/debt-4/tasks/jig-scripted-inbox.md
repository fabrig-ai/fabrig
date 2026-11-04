+++
id = "jig-scripted-inbox"
part = "jig-scripted-rules"
kind = "feature"
needs = ["jig-scripted-phase"]
files = [
  "jig/testing/jig-client-scripted-engine/src/reads.rs",
  "jig/testing/jig-client-scripted-engine/src/tests.rs",
  "jig/tests/client/src/stories/shared/**",
  "jig/tests/client/src/stories/text_pages.rs",
  "jig/tests/client/src/stories.rs",
  "jig/docs/design/client/domain/testing.status.md",
]
effort = "high"
minutes = 100
+++

# jig D028, inbox order: the fake serves rows as seeded, and a shared story holds the order

## Goal

The fake re-sorts inbox rows by its copy of the core's key, which differs from the core's Updates
order at equal times. The sort goes: a story seeds its rows in the order the core keeps, and one
shared story holds that order on the scripted and joint fronts. The second of four tasks on
`jig/docs/development/debt/D028-scripted-engine-reimplements-core.md`; the item stays open.

Acceptance:

- `rg -n 'inbox_order|sort_by_key' jig/testing/jig-client-scripted-engine/src/reads.rs` reads nothing.
- The new story passes on the scripted and joint fronts, with both Needs and Updates holding two
  rows, each read from the tree.
- `jig/tests/client/tests/golden/**` does not move; a fixture's rows move instead.
- `E/tests.rs`'s `inbox_updates_at_the_same_time_use_descending_position_before_task` is gone: it
  pinned removed code. No other assertion changes.
- The note's `What a task runs` passes, with `-p jig-ops-world`.

## Inputs

- `temper/docs/plan/debt-4/notes/common.md`, `temper/docs/plan/debt-4/notes/jig-scripted.md`, `AGENTS.md`,
  `jig/AGENTS.md`.
- `jig/docs/development/debt/D028-scripted-engine-reimplements-core.md`, whole.
- `jig/docs/design/client/domain/testing.md`, 4 and 5; `jig/docs/design/domain/engine.md`, 11.2.
- The core's order: `jig-domain-core/src/documents/inbox_deployment.rs`, `key`, for Needs; `jig-domain-core/src/reads/rows.rs`
  and `jig-domain-tasks/src/index.rs`, `Update::key`, for Updates.

## Owns and may change

- Owns: `jig-client-scripted-engine/src/reads.rs` and its tests; `jig/tests/client/src/stories/shared/**`.
- May change, for row order only: `jig/tests/client/src/stories.rs`, `src/stories/text_pages.rs`,
  `jig-client-scripted-engine/src/tests/application.rs`, `tests/golden/inbox.txt`'s fixture if one
  feeds it, and any other fixture that seeds `Rows::Inbox` out of the core's order.
- Status: remove the `testing.status.md` **Not yet** entry for inbox rows, date line today's.
- Pre-approved: the story's rows on the joint front may come from `jig_fake_workers` scripts (two
  `ProposeGoal`s, two `Finish`es, as `Seed::PendingGoal` and `Seed::ChatResult` do), spread by the
  world's clock. If Updates cannot hold two rows on the joint front at a reasonable length of
  seed, keep the fake's sort for neither group, write Needs' story, and ask: its first line
  `Question:`, with what the core would need.

## Steps

1. Remove `inbox_order`, its sort and comment from `E/reads.rs`; rows keep the filter.
2. Remove `E/tests.rs`'s test. Run the world; each failing fixture seeds its rows in the core's
   order: Needs oldest first, Updates newest first. Those rows do not move once ordered.
3. Add `Facts::InboxOrder` and, in a new `stories/shared/b_inbox.rs`:

   ```rust
   /// jig/docs/design/domain/engine.md, 11.2.
   pub fn a_person_s_inbox_lists_needs_oldest_first_and_updates_newest_first<F: Front<Test>>(
       world: &mut World<Test, F>,
   );
   ```

   The scripted side seeds two `Source`s, Needs and Updates, by `crate::stories::inbox_entry`.
   The story goes to the inbox, reads each group's cards in tree order, and compares them with
   the order the facts wrote.
4. One line in `person_story_suite!`. Run all three fronts.
5. What the note's `What a task runs` names; the citations check. Commit and push.

## Report

Each fixture you reordered; what the story seeds on the joint front; the net lines in `E`.
