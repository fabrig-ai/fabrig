+++
id = "late-reasons"
part = "late-reasons"
kind = "feature"
needs = []
files = ["jig/crates/jig-domain-core/src/routing/run/parts.rs", "jig/crates/jig-domain-core/src/routing/run/preparation.rs", "jig/crates/jig-domain-core/src/routing/sibling_routes.rs", "jig/tests/core/tests/suite/per_wake.rs", "jig/testing/jig-core-kit/src/stories/per_wake.rs", "jig/testing/jig-test-profile/src/charters.rs"]
effort = "high"
minutes = 80
+++

## Goal

A goal's run that woke as its landing's result came due read only its
charter's shared text, not the part whose `on` covers results, and ended
without that part's work. The core keeps a prepared run when a word that
arrives between its preparation and its claim isn't offered by the
prepared parts (`preparation_current`), and puts the prepared reasons back
(`preparation_facts`), so the word is kept back for a later run. The
design now has every reason waiting at the claim join the run: a message
or an entry that arrives between preparation and claim and would add to
the run's reasons **prepares the run again**.

Acceptance:

- **The failing rows first,** in `per_wake_scenarios`
  (`jig/tests/core/tests/suite/per_wake.rs`), where
  `notes/contracts.md`, section 3 puts them: the story
  `a_question_arriving_after_preparation_waits_for_the_next_run` (seed
  866) becomes one row function over the arriving kind, say
  `arriving_after_preparation(seed, kind)`, with two rows:
  - **866, a question:** the run woken by a person's words is prepared,
    its workspace completion held; a delegate's question arrives; the
    claimed run's instructions are `S\n\nQ` (model M2), its reasons
    `WORDS | QUESTION`, offered `QUESTION`, its start carrying the words
    and the question, and no later run is needed for the question.
  - **870, a result:** the same, with a delegate's result arriving (the
    delegate ends `Finished` with a report): instructions `S\n\nR`,
    reasons holding `RESULT`, offered `RESULT`, the result among the
    start's messages.
  For that, `jig_test_profile::per_wake_charter`
  (`jig/testing/jig-test-profile/src/charters.rs`) gains a fourth part,
  `(Reasons::RESULT, b"R")`. Run both before the fix: today each claims
  the prepared run with `S` alone and keeps the word back.
- **The fix, in the core:** `routing/run/parts.rs`'s
  `preparation_current` returns false (the preparation is stale and the
  run prepares again) when the task's reasons now
  (`core.tasks.run_context(task).reasons`) hold a kind the prepared
  `context.reasons` don't; a newer word that adds no reason and isn't
  offered is still kept, as today. `preparation_facts` then no longer
  restores the prepared reasons over added ones (its lines ~60–64 go, or
  are cut to what still bites). `preparation.rs`'s `start_messages` and
  `sibling_routes.rs`'s routing while preparing follow from the same
  check; change them only as the fix needs and say how.
- Every story of `per_wake_scenarios` and the core world passes: where
  one asserted instructions or reasons for a run with a result among its
  reasons, the fourth part changes it; update and name each.
- Your status entries deleted (`notes/common.md`): in
  `jig/docs/design/domain/tasks.status.md` the **Not yet** sentence
  beginning "8.6: a message that arrives between a run's preparation",
  and in `engine.status.md` the 7.1 sentence "a message or an entry that
  arrives between a run's preparation and its claim, and would add to its
  reasons, is kept back rather than preparing the run again."

## Inputs

- `jig/docs/design/domain/tasks.md`, 8.6 (*What a run is offered*,
  *Every reason waiting at its claim*, *What isn't offered keeps its
  place*).
- `jig/docs/design/domain/engine.md`, 7.1, steps 2 and 3.
- `temper/docs/design/draft/per-wake.md`, sections 2 and 4.
- `temper/docs/plan/wake-faults/notes/common.md`; `notes/contracts.md`,
  section 3.

## Owns and may change

- Owns the files in `files`; in `per_wake.rs` and the kit's
  `stories/per_wake.rs`, only where `notes/contracts.md`, section 3 says:
  task `covered-entries` adds rows 868 and 869 at the same time. A kit
  helper for a delegate's result (say `result_from(world, child) ->
  tasks::Word`) goes after `question_from`.
- Reader files: other `jig-domain-core` routing that calls
  `preparation_current`, `preparation_facts` or `preparation_offered`
  (`routing/children.rs`, `routing/run/results.rs`,
  `routing/task/save.rs`): call sites only. A core unit test that pins a
  kept-back word between preparation and claim
  (`jig/crates/jig-domain-core/src/tests/**`) is updated to the rule and
  named. Other users of `per_wake_charter`: assertions that the fourth
  part changes, named.
- Your status entries in `jig/docs/design/domain/tasks.status.md` and
  `engine.status.md`.
- Pre-approved: if comparing whole reasons re-prepares for a kind that
  chooses no new part and that costs a story more than one extra
  preparation, compare the parts and offered kinds those reasons choose
  instead, keep the claim's reasons the fresh ones, and say so.

## Steps

1. The fourth part; rows 866 and 870; run them; record the failures seen.
2. `preparation_current` and `preparation_facts`; whatever the start's
   messages need.
3. Break the rule once (keep the prepared run again when the newer word
   isn't offered), see both rows fail, restore.
4. Your status entries.
5. `cargo fmt`; `python3 -I -B jig/scripts/old_forms.py`; `heavy cargo
   clippy -p jig-domain-core -p jig-core-kit -p jig-core-world -p
   jig-test-profile --all-targets -- -D warnings`; `heavy cargo nextest
   run -p jig-domain-core -p jig-core-world`; `heavy cargo nextest run -p
   jig-core-world --profile fuzzy`; `heavy cargo check -p
   jig-channel-world --tests` and every other package that names
   `jig-test-profile` (`cargo tree -i jig-test-profile` lists them).
6. Commit and push.

## Report

## Rebase (2026-11-08, step merge, foreman wake-faults/late-reasons/foreman-1)

Resolve both status conflicts by preserving late-reasons and covered-entries deletions.
Rebase onto the landing trunk. The conflicting commit is `f15f70e6d77ef81cf31137be103ac344dab2471a` (“Keep covered undecided entries parked until a change”), now in `origin/wake-faults/integration`.
- In `jig/docs/design/domain/engine.status.md`, remove both the 7.1 late-arrival gap and the covered-entry sentence. Retain the 7.4 live-delegates gap and section 8 invalid-start gap, unless subsequently removed on the target.
- In `jig/docs/design/domain/tasks.status.md`, remove both the 8.6 late-arrival gap naming `preparation_current` and the 8.1/8.3/8.6/9 covered-entry gap naming `Task.read_escalations`. Preserve all other target-side status edits.
Both tasks delete adjacent sentences in the same paragraphs; combine their deletions rather than selecting either whole side.
The two `per_wake.rs` files merged automatically. Preserve both tasks’ rows and helpers under `notes/contracts.md`, section 3.
No core behavior change is needed to resolve this merge refusal.
