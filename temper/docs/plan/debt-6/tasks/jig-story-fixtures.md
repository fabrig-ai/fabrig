+++
id = "jig-story-fixtures"
part = "jig-review"
kind = "feature"
needs = ["jig-delegate-story"]
files = ["jig/tests/client/src/stories/shared/b_inbox.rs", "jig/tests/client/src/stories/shared/b_held.rs", "jig/tests/client/src/stories/shared/b_scenarios.rs", "jig/tests/client/src/stories/shared/b_seed.rs", "jig/tests/client/src/stories/shared/b_family.rs"]
effort = "medium"
minutes = 45
+++

# jig: the stories' fixtures say one thing once

## Goal

The review's Part B, findings 3 and 4. `b_inbox::scripted` is a false loop: one body branches twice on
`group`, once per list. It becomes two helpers, `need` and `update`, and two flat lists. Writing an
object and an inbox source take a dozen lines each, repeated: a general `install` and `inbox_source`
replace them. `b_held`'s inline `use`s and mid-file constants move to the top.

`jig-delegate-story` ran first: read what it left in `b_held.rs` and `b_scenarios.rs`, and fold only what
is there. The review's line numbers may have moved.

Acceptance:

- `b_inbox::scripted` holds no loop over groups and no `if group ==`.
- `rg -c 'Change::Object \{' jig/tests/client/src/stories/shared/b_inbox.rs jig/tests/client/src/stories/shared/b_held.rs jig/tests/client/src/stories/shared/b_scenarios.rs jig/tests/client/src/stories/shared/b_seed.rs`
  reads nothing, but in `b_seed.rs`, which defines `install`: there it reads 1.
- `rg -n '^    use |^const ' jig/tests/client/src/stories/shared/b_held.rs` shows no `use` inside a function
  and no constant below the first `fn`.
- `heavy cargo nextest run -p jig-client-world` passes with no story's words or asserts changed, and the
  part's stories read the same.

## Inputs

- `temper/docs/plan/debt-6/notes/common.md`, `AGENTS.md`, `jig/AGENTS.md`.
- `jig/docs/design/client/domain/testing.md`, 4 and 5.

## Owns and may change

- Owns: the fixture code of `b_inbox.rs`, `b_held.rs`, `b_scenarios.rs` and `b_seed.rs`.
- `b_family.rs` adopts the helpers if its sites are the same shape, and is left alone if not.
- If a site's `project` is not 1, give `install` no parameter for it: leave that site as it is, and name
  it in the report.
- Not this task's: `b_config.rs`, `b_conversations.rs`, `b_goals.rs`, which `jig-story-tidy` owns.

## Steps

1. In `b_seed.rs`, for all the `b_*` fixtures:

   ```rust
   /// One object of project 1, as every person story's scripted seed writes it.
   pub(super) fn install(front: &mut Scripted, key: c::ObjectKey, value: c::ReadValue);
   /// The rows of one inbox group, for one party.
   pub(super) fn inbox_source(front: &mut Scripted, party: u64, group: c::InboxReadGroup, rows: Vec<c::InboxEntry>);
   ```

   `b_scenarios`'s `install(front, task)` becomes a call of it.
2. `b_inbox::scripted`:

   ```rust
   let needs = vec![need(front, PROPOSER, 1), need(front, SECOND_PROPOSER, 2)];
   inbox_source(front, PARTY, c::InboxReadGroup::Needs, needs);
   let updates = vec![update(front, NEWER_RESULT, 4), update(front, OLDER_RESULT, 3)];
   inbox_source(front, PARTY, c::InboxReadGroup::Updates, updates);
   ```

   Each helper writes its object and returns its entry. A row's position and time are one value today:
   one parameter.
3. Move `b_held`'s `use`s and constants to the top.
4. Hand-break, since a story's fixture changed: keep the two rows of `updates` in their places and swap
   their position and time, `update(front, NEWER_RESULT, 3), update(front, OLDER_RESULT, 4)`, and
   `a_person_s_inbox_lists_needs_oldest_first_and_updates_newest_first` goes red on the scripted front.
5. `cargo fmt`; `heavy cargo clippy -p jig-client-world --all-targets -- -D warnings`;
   `heavy cargo nextest run -p jig-client-world`; the citations check. One commit for each finding. Push.

## Report

The common note's table. Net lines; the review expects about -60 from these two findings.

## Answers

2026-11-04. Question: step 4's row swap stays green because the view sorts by time; may the break swap
the updates' position and time instead, or should the story assert the raw source order?

Swap the position and time; step 4 now says so. The story's rule is an order by time, which
`jig/crates/jig-client-view/src/state/page/inbox.rs`, `merge` and `earlier`, holds whatever order the
source supplies, so the old step 4 broke nothing the story names. The raw source order is no rule of
this story: add no assertion, and no story's words or asserts change. The break is a run's to observe:
if this one stays green too, stop and ask again. Run 1's commit `f87ec2782c` stands; finish the rest of
the acceptance on it.
