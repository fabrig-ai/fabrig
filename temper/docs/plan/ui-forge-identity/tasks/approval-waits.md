+++
id = "approval-waits"
part = "approval"
kind = "feature"
needs = ["link-decide"]
files = [
  "temper/crates/temper-engine-domain-forge/src/approvals.rs",
  "temper/crates/temper-engine-domain-forge/src/views.rs",
  "temper/crates/temper-engine-domain-forge/src/domain.rs",
  "temper/crates/temper-engine-domain-forge/src/verification.rs",
  "temper/crates/temper-engine-domain-forge/src/lib.rs",
  "temper/crates/temper-engine-domain-forge/src/tests/standing.rs",
  "temper/crates/temper-engine-domain-forge/src/tests/panel.rs",
  "temper/crates/temper-engine-domain-forge/src/tests/views.rs",
  "temper/crates/temper-engine-domain-forge/src/tests/change.rs",
  "temper/crates/temper-engine-domain-forge/src/tests/landing.rs",
  "temper/docs/design/domain/forge.status.md",
]
effort = "high"
minutes = 70
+++

# The forge answers the changes waiting on a person's approval

## Goal

Under Needs you, a change whose merge waits on a review on the forge will be a card for whoever's review it waits on (`temper/docs/design/domain/forge.md`, 8.12). The root joins the people; the forge answers the waits. Three things in `temper-engine-domain-forge`:

- **The subject:** `APPROVALS_SUBJECT`, marked wherever `changes` is (`changes_mark`).
- **The facts' time and end:** a change's review facts keep when they were first kept at their head, and go when its merge is admitted (8.7), marking `APPROVALS_SUBJECT` and the change's panel.
- **The read:** `Domain::approval_waits`, each live change waiting on an approval at its last head, with what its card says.

Nothing reads them yet but the tests: the root's read is `approval-read`.

Acceptance:

1. `APPROVALS_SUBJECT`, `ApprovalWait`, `Domain::approval_waits` and the changed `keep_review_facts` are as `temper/docs/plan/ui-forge-identity/notes/contracts.md`, *The approval read*, *The subject* and *The forge's part*, write them, exported from `lib.rs`, with doc comments in the design's words.
2. `changes_mark` answers the three subjects; every test pinning its marks holds the third (Steps, 4).
3. A merge's `Committed` drops its change's facts and marks `[APPROVALS_SUBJECT, task]`; a commit of any other entry drops nothing.
4. The new table test of `approval_waits` (Steps, 5) and the drop's test bite.
5. In `temper/docs/design/domain/forge.status.md`, the **Not yet** entry *8.7: a change's review facts are removed only when it ends...* is gone, and the 8.12 entry reads, whole: *8.12: the root answers no document of the subject `approvals`, and the read of a proposal's items says nothing of an approval on an owned Forgejo.*, which `approval-read` removes.
6. `heavy cargo nextest run -p temper-engine-domain-forge` and `-p temper-engine-domain --lib` pass; clippy clean for both.

This task departs from the implementer's text in one way: two status entries are yours to edit (acceptance 5), because the design already says what this task builds.

## Inputs

- `temper/docs/plan/ui-forge-identity/notes/common.md`.
- `temper/docs/plan/ui-forge-identity/notes/contracts.md`, *The approval read*: *The subject* and *The forge's part* bind.
- `temper/docs/design/domain/forge.md`, 8.7, the bullet *The panel's approval evidence*; 8.12, the bullets *The subject `approvals`*, *The forge's part* and *Marked*; 8.8, *Its place in its queue*, for how `behind` counts.

## Owns and may change

**Owned:** the `files` above. `domain.rs` only in `keep_review_facts`, `committed`, `retain_terminal_ci`, `Domain`'s fields and its constructor: `link-record` and `link-decide` write it too.

**Readers and neighbours:**

- `temper/crates/temper-engine-domain/src/route/forge_route.rs`, `retain_review_wait`: its one call, `domain.forge.keep_review_facts(task, facts)`, becomes `keep_review_facts(env.wall, task, facts)`, the one-line edit there. Nothing else in that file: it is shared with `link-lookup` and temper/migration's `goal-delivery` and `call-room`.
- `verification.rs` and `landing_log.rs` take `changes_mark`'s array as they do; `verification.rs`'s `save` takes `Option<[u64; 3]>`, the compiler's edit. `landing_log.rs`'s needs no edit, it boxes the array.
- **A change of meaning:** every mark of `changes` now carries a third subject, one fewer of a queue's followers fitting `MARKED`. Run the readers' tests: `temper-engine-domain --lib` (above) and the engine's domain world's `heavy cargo nextest run -p temper-engine-domain-world --test suite -E 'test(/panels::|forge_routes::approval_identity::/)'`. A world test that pins a mark list exactly takes the third subject, named in the report.

**Options, each with its default:**

- `since`'s home: a map beside `review_facts`, `review_since: Map<u64, Wall>`, the same capacity, removed wherever `review_facts` is (`retain_terminal_ci`, the drop). Default that; a value type `(ReviewFacts, Wall)` instead is yours if it reads simpler, `standing.rs`'s two readers edited for it.
- The merge's change: the entry's `task` when it names a change in `changes`; otherwise each change whose `change.landing` is that task, as a turn's landing merges its members. Default both; if the second never holds facts in this crate, say so and keep the first alone.
- `behind`: the changes after the change in `changes_order(row, now, limits)` (`views/changes.rs`), 0 where it isn't listed. If `changes_order` must become `pub(crate)` for `approvals.rs`, that's the visibility edit.
- The drop's test: in `tests/landing.rs`, whose `landing_fixture` makes a turn landing's merge entry (91, of landing task 61, for `fixtures::CHANGE`) and commits it (~275): keep facts for `CHANGE`, commit the merge, see them go. Default that, which holds the turn's case; a procedure merge's row too if a fixture in `tests/landing.rs` or `tests/change.rs` makes its entry in a few lines.

## Steps

1. `views.rs`: the constant, and

   ```rust
   pub(crate) fn changes_mark(d: &Domain, repository: skein_forge::api::Repository) -> Option<[u64; 3]> {
       let project = d.repositories.get(&repository)?.project;
       Some([changes_subject(project), crate::status_subject(project), APPROVALS_SUBJECT])
   }
   ```

2. `domain.rs`: `keep_review_facts(&mut self, wall: Wall, task: u64, facts: ReviewFacts) -> bool`, its doc comment saying when `since` moves (the contract). In `committed`: when `d.entries.get(&number)`'s `entry.effect.write` is `skein_forge::api::Write::Merge { .. }`, remove the change's facts and `since`, and if any were there, `emit(out, Request::Marked { tasks: Box::new([APPROVALS_SUBJECT, change]) })`. Every `match` lists its variants.
3. `approvals.rs`, `Domain::approval_waits(&self, now, limits)`: over `self.changes`, a row that isn't `terminal()`, whose `change.state` is `State::First { .. }` or `State::Landing { .. }`, with `pull: Some(_)` and an adopted repository; whose facts' `head` is `change.last_head`; whose `panel_approvals(row)` hold none `refused` and at least one with `remaining` above 0. `short` is those, as `panel_approvals` gives them; `whole` the facts' `reviews_complete`; `title` fitted as the changes document fits it, `crate::text::fit(&row.title, crate::LOGGED_TITLE_BYTES, "…".as_bytes())` (`views/changes.rs`); `owner` and `name` the repository record's. Sorted by `(since, task)`. No `unwrap`, no indexing, no `as`.
4. The mark readers: `tests/panel.rs`' two follower tests (`[51, changes_subject(5), status_subject(5), APPROVALS_SUBJECT, 52]`, and the priority test one follower fewer at the bound), `tests/change.rs` (~405), and `tests/views.rs`' `marked` and `queue_marked` helpers, which also require `APPROVALS_SUBJECT`, so the four `changes_marks_*` tests hold it. `standing.rs`' `keep_review_facts` calls take a wall.
5. The table, in `tests/standing.rs` beside `approval_panel_distinguishes_unread_incomplete_and_counted_reviews`, with its `fixture()` and `keep`:

   ```rust
   #[test]
   fn approval_waits_are_live_changes_short_of_an_approval_at_their_last_head() {
       // (state, pull, facts head, complete, reviewers, expected: None or Some((whole, short count)))
       // First, at the last head, complete, none counted        -> Some((true, 1))
       // Landing, the same                                       -> Some((true, 1))
       // a state neither First nor Landing (Queued, Held)        -> None
       // facts at another head; no facts                         -> None
       // read in part                                            -> Some((false, 1))
       // enough counted; one asked for changes (refused)         -> None
       // no pull request; a terminal row                         -> None
   }
   ```

   Then, in the same test or one beside it if the fixture makes that clearer: facts kept at wall A, the same head again at B keeps `since` A, another head at C makes it C; and, if the fixture queues a second change on the branch in a line or two, `behind` 1 for the first.
6. The drop's test (option above): a merge entry committed drops the change's facts (its panel's approval reads `Unread` again) and marks `[APPROVALS_SUBJECT, task]`; a commit of a non-merge entry drops nothing. Bites: remove the drop, see it fail; remove the `First`/`Landing` filter in `approval_waits`, see a row of step 5 fail; restore.
7. The status words (acceptance 5).
8. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p temper-engine-domain-forge -p temper-engine-domain --all-targets -- -D warnings`; `heavy cargo nextest run -p temper-engine-domain-forge`; `heavy cargo nextest run -p temper-engine-domain --lib`; the world's filter above. Commit ("temper-engine-domain-forge: answer the changes waiting on a person's approval, its facts dropped at the merge's admission"), push.

## Report

As the implementer's text says, with each rule broken for a bite and the failure seen, the world tests whose marks changed, and the one-line edit in `forge_route.rs`.
