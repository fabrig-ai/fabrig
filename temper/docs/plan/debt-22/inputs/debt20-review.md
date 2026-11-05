# Spot review of debt-20's jig part (2026-11-05, merge 788ddd9d4b)

Read-only reviewer, using the user's quality criteria. Nothing was compiled or run: what a test catches is from
reading. Line numbers are at the merge.

Verdict: the production change is good and preserves behaviour; nothing stored or refused changes. The test growth is
only partly earned: all 1,446 lines are new, about a third assert behaviour, and about 450 could go with no rule lost.
The tables were a fair migration pin, but are weak after merge: a new variant adds no row by force, and each table is
one more place to edit, against D015's "added in one place".

1. Cuts:
   - `message_families_keep_their_inbox_and_activity_behaviour` (`jig-domain-tasks/src/tests.rs:289-475`): `spoken` and `hint` equal
     the `family` column, and the other booleans derive from it (-140).
   - `Row` in `jig-domain-core/src/tests/classify_subjects.rs:25-201`: `retained`, `application`, `chips` follow from `shape` (-135).
   - The 37 `F::Settled` rows copy one production arm: keep two (-45). The 19 identical `None` rows in the act table: a loop (-15).
   - `LIMITS` at `jig-domain-tasks/src/limits.rs:195` is byte-identical to `tests/revisions.rs:25`: reuse it (-49).
   - `history_copy_orders_held_and_history_with_the_revision_number` is redundant (-13).
   - Aliases `OutcomeFamily as OutcomeClass`, `AskFamily as AskClass` (`jig-domain-people/src/tests.rs:2516`, `2621`).
2. **Gap: four tests can pass with their rule broken.**
   - `word_bytes_covers_fixed_notices_and_caller_limits` (`limits.rs:247`): `.max(inbox.min(87))` at :123-125 never wins, so deleting it
     stays green. Drop the term; keep the `NOTICE - 1` and `NOTICE + 1` rows.
   - `news_batch` in the tasks table separates only category 6 of eight (`wake.rs:34`): merging `Notice` into `Words` stays green.
     Use `category: u8` and assert `category(&kind)`.
   - `classify_subjects.rs:248`: `assert!(refused.pop().is_some())` is always true. Assert the popped request is `Now::WatchRefused`.
   - `remap_models_updates_a_live_authority_and_preserves_all` (`stored/models.rs:87`): 64 lines for one arm; dropping `proposal`,
     `recurring` or `History` stays green. A 15-line `Models::remap` unit test does more (-50).
3. `Slot` holds two answers, so one split is typed three times (`inbox.rs:193`, `stored/fit.rs:391`, `:479`): `MessageFamily::Separate(Separate)`
   for the four, about -35 production. `finish_requested` (`boundary.rs:237`) becomes one `==`.
4. **Gap: `Write::Copy` is generic by name but specified and tested only for settings.** `jig/docs/design/protocol/store.md` doesn't
   state the missing-`from` failure; `Copy` reads the map, so `Save(from)` then `Copy` in one commit copies old bytes, despite "as
   the writes in order would" (:754). `jig/tests/store/src/model.rs:35` panics on anything but a settings revision. Add two
   sentences and about 20 lines of test. Add a one-row load-refusal pin for `FinishRequested` (`tests/startup/detail.rs:448-455`).
5. `settings::remap` rebuilds `moved` per value (`jig-domain-core/src/settings/models.rs:18`): split `moves(kept, names)` and pass `&moved`.
6. `AskFamily`'s project arm repeats (`domain/asks.rs:5`, `capacity.rs:103`, `domain/deletion.rs:118`): `AskFamily::named_project()`. `control()`
   (`acts/objects/task.rs:69`) re-fetches `row` with an `.expect`: pass it in.

Good, not findings: `Selectors`, `touch_views`, the `apply_roles` collapse, `control()`, `copy` through `open::get`. Or-pattern lines fell:
people 1,014 to 602, tasks 861 to 640, core 3,238 to 2,986. Long lists in `roster`, `chips` etc. are ruled by programming-model.md 10.
