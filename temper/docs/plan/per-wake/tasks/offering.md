+++
id = "offering"
part = "offering"
kind = "feature"
needs = ["surface"]
files = ["jig/crates/jig-domain-tasks/src/wake.rs", "jig/crates/jig-domain-tasks/src/waiting.rs", "jig/crates/jig-domain-tasks/src/inbox.rs", "jig/crates/jig-domain-tasks/src/admission.rs", "jig/crates/jig-domain-tasks/src/run.rs", "jig/crates/jig-domain-tasks/src/domain.rs", "jig/crates/jig-domain-tasks/src/reasons.rs", "jig/crates/jig-domain-tasks/src/tests/offering.rs", "jig/crates/jig-domain-tasks/src/tests.rs", "jig/docs/design/domain/tasks.status.md", "jig/docs/design/domain/engine.status.md", "jig/docs/design/domain/hosts.status.md"]
effort = "high"
minutes = 150
+++

# per-wake: the tasks child keeps a run's reasons and offers it only what its parts cover

## Goal

The tasks child's half of `jig/docs/design/domain/tasks.md`, 8.6: it gives each activation
its reasons, keeps them across a retry and a continuation and ends them with the first
activation; it relays to a live run only what its claim was offered; a turn's fence takes
only the messages offered to its attempt; and an entry left undecided when a run ends
wakes the next. The core's half (choosing parts, the start's messages, what the claim is
offered) is the parts-core task's, built to the same fields in parallel.

Acceptance:

- **Reasons at activation.** `run_context` gives `reasons`: the record's `reasons` with
  `Reasons::of` the kind of every inbox message and every entry waiting for the task's
  decision (`proposals::waiting_for`, `escalation::waiting_for`) whose policy wakes the task
  now: its `rule` `Immediate`, or `Batch` or `Paced` with the word `eligible`; `Never`, a
  non-last result under `last_or_failure` and news a pace holds give nothing.
- **Kept across attempts.** A claim sets the record's `reasons` and `offered` (the
  surface's). When a run's end is applied (`run::activation`): an `End::Parked` with no
  narrowing whose task is neither kept (`kept.blocks_claims()`) nor owing (`kept.owes`), and
  an `End::Finished`, set `reasons` to `Reasons::EMPTY` (the first activation, or the
  activation, ends); a run parked kept, a failed, refused, exceeded or rechecked end, and a
  narrowing's park leave it. So the next activation's reasons are the kinds that wake it,
  and a retry's or a continuation's are the attempt's before it with the kinds that wake it
  now.
- **Relayed only if offered.** `after_message` relays a message to a claimed or running
  attempt only when `offered(&word, row.offered)`; otherwise it does nothing (the message
  stays). `waiting::next` skips a physical inbox message `offered` refuses; entries pass
  (they are always offered).
- **The fence takes only what was offered.** A turn takes (`inbox::take`, called from
  `admission::turn`) only the inbox messages numbered at most its fence that are in the
  turn's `outstanding` (the ids offered to the attempt and not yet read), and never a
  refused one. A message not offered, or held by a pace, that the fence passes stays.
- **An entry left undecided wakes the next run.** `wake::after_run`, once the inbox is
  judged again and the task is still `Idle`, judges each entry still waiting for its
  decision as `after_message` judges a message arriving, so the task becomes due with
  `due_reason` `Why::Message { message: <the entry's number> }` and its next `run_context`
  holds that entry's kind.
- **Tests, failing first,** in `jig/crates/jig-domain-tasks/src/tests/offering.rs`, two
  tables:
  - reasons, rows `(name, setup, expected reasons)`: made and activated (`FIRST`); made
    with a person's words waiting (`FIRST | WORDS`); after a park idle, woken by a question
    (`QUESTION`); after a park idle, a non-last result under `last_or_failure`
    (not due, and with the last result `RESULT`); a failed attempt's retry with a timer
    waiting (the attempt's reasons `| TIMER`); a run parked kept by its own question,
    continued by its answer (`FIRST`, since `answer` gives none); a finished-done run kept
    by a dependency, woken by `met` (`FIRST`);
  - offering, rows `(name, offered mask, what arrives or is read, expected relays,
    expected inbox after)`: a question to a run offered `FIRST` only is not relayed and
    stays after a fence past it; an answer, an amendment and a person's words to the same
    run are relayed; a task's words are not; an entry is relayed; under `Reasons::ALL`
    every kind is relayed, as before; a pace-held news a fence passes stays; a question
    kept back wakes the task when its run parks idle; an escalation offered and left
    undecided when the run parks makes the task due again with `Why::Message`.
  Break one rule by hand (relay regardless, take the prefix), see its rows fail, restore.
- Existing tables that pin a fence or a relay (`src/tests/decision_reads.rs`, the kept
  table in `src/tests/kept.rs`, the wake table in `src/tests.rs`) pass unchanged: this task
  changes what `Reasons::ALL` gives nobody.
- Delete from the status files these sentences, each with its bullet's lead where it is
  left empty, and nothing else (a departure from the implementer's text, which closes
  `docs/design/`: each status must say what is built when this lands):
  `jig/docs/design/domain/tasks.status.md`, "3, 8.6: a task keeps no reasons, ..." and
  "8.2, 8.3, 8.6, 9: every message waiting is offered to a run, ... `after_run`)."; in
  `jig/docs/design/domain/engine.status.md`, "7.2, 7.4: a turn's commit takes every message
  up to its fence, ... `tasks.status.md`, 8.6."; and
  `jig/docs/design/domain/hosts.status.md`'s bullet "6.4, 9: a turn's fence takes every
  message before it, ...".
- This changes what a shared item means (which messages a fence takes, what a live run
  hears): run the focused tests of `jig-domain-tasks`, `jig-domain-core`, the tasks world
  (`jig/tests/tasks`) and the core world (`jig/tests/core`), and `heavy cargo check -p
  temper-engine-domain --tests`.

## Inputs

- `temper/docs/plan/per-wake/notes/common.md`; `notes/contracts.md`, sections 1 and 2.
- `jig/docs/design/domain/tasks.md`, 8.2 ("Not offered, a message stays too"), 8.3 (a pace,
  and judged again), 8.6 (whole), section 9 ("Waking"); `jig/docs/design/domain/engine.md`,
  7.2 (a turn's commit) and 7.4.

## Owns and may change

- Owns the files in the header (`reasons.rs` for a helper the contract leaves out, no
  change to its public items).
- Readers: a test elsewhere in `jig-domain-tasks`, `jig/tests/tasks` or `jig/tests/core`
  that pinned a fence taking a never-offered message, or a relay of one: update its
  expectation to the design's and list it with the design line it now follows.
- Pre-approved: if `admission::turn`'s `outstanding` is not the attempt's offered and
  unread ids before this turn, take the offered set from where the turn's caller keeps it
  (`jig-domain-core/src/routing/run/turns.rs` passes it) and say so; if judging an entry
  through `after_message` would relay it twice to a live run, judge it only when the task
  is `Idle` (as above) and say what you saw.
- Not yours: the core's start messages and the claim's offered mask (parts-core), the
  charter's parts, the later choice, the store.

## Steps

1. The two tables, failing for the rule's reason.
2. Reasons at activation, at the claim and at a run's end; relays; the fence; the entry's
   wake.
3. The tables pass; the bites above.
4. The status sentences.
5. `cargo fmt`; `python3 -I -B jig/scripts/old_forms.py`; `heavy cargo clippy -p
   jig-domain-tasks --all-targets -- -D warnings`; the focused tests named above; the
   citations check.
6. Commit (why: a run hears only what its parts are for, and the rest waits for a run that
   has them); push your branch.

## Report

As the implementer's text says, and also: each existing expectation changed, with its
design line; how the waking kinds are judged at activation.
