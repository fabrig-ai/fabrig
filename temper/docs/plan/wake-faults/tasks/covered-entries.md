+++
id = "covered-entries"
part = "covered-entries"
kind = "feature"
needs = []
files = ["jig/crates/jig-domain-tasks/src/wake.rs", "jig/crates/jig-domain-tasks/src/escalation.rs", "jig/crates/jig-domain-tasks/src/proposals.rs", "jig/crates/jig-domain-tasks/src/domain.rs", "jig/crates/jig-domain-tasks/src/admission.rs", "jig/crates/jig-domain-tasks/src/reasons.rs", "jig/crates/jig-domain-tasks/src/tests/offering.rs", "jig/tests/core/tests/suite/per_wake.rs", "jig/testing/jig-core-kit/src/stories/per_wake.rs", "temper/docs/help/task.md"]
effort = "high"
minutes = 100
+++

## Goal

A goal's lead passed a held repair's question to its person and parked;
for three minutes, 19 runs each woke on that same escalation, decided
nothing and parked again. The rule, settled by the user: **if a part of
the ended run covered the entry's kind and the run left it undecided, the
entry doesn't wake the task again by itself**; it is offered at the next
wake, whatever wakes it, and wakes the task when it changes. If no part
of the ended run covered it, it wakes the task again, as now. This
replaces the read escalation's person-answer wait
(`Task.read_escalations`, `awaits_person`), whose conditions the
deployment's run didn't meet.

Acceptance:

- **The failing story first:** rows of `per_wake_scenarios`
  (`jig/tests/core/tests/suite/per_wake.rs`), where
  `notes/contracts.md`, section 3 puts them:
  - **868, `an_escalation_its_part_left_waits_for_a_change`:** from
    `an_undecided_escalation_wakes_its_part`'s course (the first run,
    part F, doesn't cover the escalation and wakes the next run with
    part H: that is the *not covered* case, which stays as it is), the
    second run (offered `ESCALATION`) takes a turn reading the entry and
    parks with it undecided: no new assignment, the task `Active(Idle)`,
    the child still held with its entry waiting at the task. Then a
    person's words (`people::Ask::Say`) wake the task: its run's reasons
    are `WORDS | ESCALATION`, its instructions `S\n\nH`, and its start
    carries the words and the entry. It parks undecided again: no wake.
    Then the entry changes, the child held anew (released by a party and
    held again by a second `Exceeded` end, a new entry with the hold's new
    revision): the task wakes with reasons `ESCALATION` and the new entry.
    Run it before the fix: today the first park wakes the task at once.
  - **869, `a_covered_escalation_stays_parked_across_a_restart`:** the
    covered park of 868, then the world restarted from its store (as
    `a_goal_interrupted_in_its_first_cut_keeps_its_parts_and_choice`
    does): no assignment; the task idle; the entry waiting.
- **The rule, in the tasks child:** `wake::after_run`, for each proposal
  and escalation still waiting for the idle task, calls `after_message`
  (wakes) only when the entry was refused by the ended attempt, or its
  kind is not among the kinds the ended attempt was offered:
  `!record.offered.meets(Reasons::of(&word.kind))` (a charter with no
  parts, or a task with its own instructions, is offered `Reasons::ALL`,
  so it covers every kind). One small function names it, say
  `pub(crate) fn covered(word: &Word, offered: Reasons) -> bool` in
  `reasons.rs`, beside `offered`.
- **Restart:** `escalation::wake_restored` and `proposals::wake_restored`
  skip an `Active(Idle)` holder whose stored `offered` covers the entry's
  kind; every other holder is woken as today. No stored field is added.
- **The exception goes:** `Task.read_escalations`,
  `wake::read_escalations`, `awaits_person`, `admission.rs`'s call and
  `run.rs`'s claim-time reset are removed.
- **Unit rows** in `jig/crates/jig-domain-tasks/src/tests/offering.rs`:
  the table near lines 334–518 gains rows for an idle holder with
  `offered: R::ESCALATION` (escalation not due), `offered: R::PROPOSAL`
  (proposal not due), the same refused (due), and `offered: R::QUESTION`
  (escalation due); the existing "entry wakes next" and "proposal wakes
  next" rows (`offered FIRST`) stay. `a_parked_escalation_holder_waits_for_its_person_delegate`
  and its helpers (near lines 775–882) are cut to the rows that still
  bite under the rule: with `R::ALL` the holder stays idle whether or not
  a person delegate is pending; with an offered set that doesn't cover
  escalations it wakes even with one pending; its restart check follows
  the restart rule above.
- **The help page:** `temper/docs/help/task.md`'s *Awaiting a person*
  bullet says the rule in a person's words: an agent whose instructions
  covered a waiting decision and that parks leaves it waiting; that same
  decision doesn't wake it again until it changes or something else wakes
  it, when it is offered again; one whose run had no instructions for it
  is woken again; the decision's stall time keeps running. Keep it within
  the page's bound; `heavy cargo nextest run -p temper-docs` passes.
- Your status entries deleted (`notes/common.md`): in
  `jig/docs/design/domain/tasks.status.md` the **Not yet** sentence
  beginning "8.1, 8.3, 8.6, 9:", and in `engine.status.md` the 7.4 clause
  "an entry left undecided under a covering part wakes the task again".

## Inputs

- `jig/docs/design/domain/tasks.md`, 8.6 (*An entry left undecided
  waits for a change, once a run covered it*), 8.1 (its closing
  paragraph), 8.3 (the paragraph after *timers*), section 9 (*Waking*).
- `jig/docs/design/domain/engine.md`, 7.4 (*An entry left undecided*).
- `temper/docs/design/draft/per-wake.md`, section 4 (*An entry left
  undecided wakes the next run, unless the run covered it*).
- `jig/docs/design/protocol/store.md`, section 5 (the offered kinds kept
  with a task).
- `temper/docs/plan/wake-faults/notes/common.md`; `notes/contracts.md`,
  sections 3 and 4.

## Owns and may change

- Owns the files in `files`. In `jig/crates/jig-domain-tasks/src/run.rs`,
  only the claim's `task.read_escalations = Box::new([]);` (today line
  ~136): task `finish-parks` edits that file's terminal admission at the
  same time (`notes/contracts.md`, section 4).
- In `per_wake.rs` and the kit's `stories/per_wake.rs`, only where
  `notes/contracts.md`, section 3 says: task `late-reasons` rewrites row
  866 and adds 870 at the same time.
- A kit helper the rows need (a party's release, a second `Exceeded`, a
  restart from the store) goes in `stories/per_wake.rs` after
  `escalation`; one that exists elsewhere in `jig-core-kit` is reused.
- Reader files: `jig/docs/design/domain/tasks.status.md` and
  `engine.status.md`, your entries only.
- Pre-approved: if a party's release of the held child isn't reachable in
  the core world without new fixtures beyond a helper, the change case of
  868 becomes an `offering.rs` unit row (a new entry for the same held
  task, under a new hold revision, wakes the idle holder whose offered
  covers escalations), and 868 ends at the second park; say so. If a
  restart story would need more than `setup` on both sides, 869 becomes
  the unit restart row and the story is dropped; say so.

## Steps

1. Rows 868 and 869; run them; record the failures seen.
2. `covered`, `after_run`, both `wake_restored`; remove the exception.
3. The `offering.rs` rows; break the rule once (wake on every undecided
   entry again; then skip refused entries too), see the rows fail for
   that reason, restore.
4. The help bullet; your status entries.
5. `cargo fmt`; `python3 -I -B jig/scripts/old_forms.py` and
   `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p
   jig-domain-tasks -p jig-core-kit -p jig-core-world --all-targets --
   -D warnings`; `heavy cargo nextest run -p jig-domain-tasks -p
   jig-core-world -p jig-tasks-world -p temper-docs`; `heavy cargo
   nextest run -p jig-tasks-world --profile fuzzy`; `heavy cargo check -p
   jig-domain-core --tests`.
6. Commit and push.

## Report
