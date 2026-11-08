+++
id = "start-refusals"
part = "start-classes"
kind = "interface"
needs = []
files = ["jig/crates/jig-domain-hub-link/src/payload.rs", "jig/crates/jig-domain-hub-link/src/tests.rs", "jig/crates/jig-domain-host/src/boundary.rs", "jig/crates/jig-domain-host/src/domain.rs", "jig/crates/jig-domain-host/src/hosted/start_refused.rs", "jig/crates/jig-domain-host/src/tests.rs", "jig/testing/jig-test-worker/src/hand_out.rs", "jig/testing/jig-test-worker/src/domain.rs"]
effort = "high"
minutes = 60
+++

## Goal

A CI repair's four attempts ended **invalid** with no turn, no model use
and no spend (17.9 s, then 0.58, 0.25 and 0.20 s each), and the task was
held at invalid count 4. Every start a host refuses before the run's
first turn, other than for its transcript, reaches the engine as
`Failed(Class::Invalid)` (`jig-domain-hub-link`'s `payload::ending`),
the class of a result that broke its contract, though no model gave
anything; and smith's own *busy* (no room for an agent or a run) reaches
it the same way, though a busy host is no failure. The design now has a
refused start spend a **permanent** try (an assignment its host found
invalid), and an agent's busy refusal answered **busy**. This task is
jig's half; task `worker-busy` is temper's worker.

Acceptance:

- **The failing table first:** `unusable_transcripts_are_transient_at_admission_and_at_the_terminal`
  (`jig/crates/jig-domain-hub-link/src/tests.rs`, near line 522) becomes
  the refused start's table, renamed to say so (say
  `a_refused_start_is_transient_permanent_or_busy`), its rows
  `notes/contracts.md`, section 2's: `Refused(Busy)` → `End::Refused`;
  `Invalid(Transcript)` and `Invalid(Start { cause: Transcript, .. })` →
  `Failed(Transient)`; `Invalid(Start { cause: Other, .. })`,
  `Invalid(Start { cause: Grants, .. })`, `Invalid(Charter)`,
  `Invalid(Grants)` and `Invalid(Messages)` → `Failed(Permanent)`, each
  at admission and at the terminal as the table checks today. Run it
  before the fix: the permanent rows give `Invalid`.
- **The class:** `payload::ending` maps `Refused(Invalid(_))` other than a
  transcript's to `Failed(Class::Permanent)`, as `job_ending` already
  maps a job's invalid assignment.
- **The busy start:** `FromAgent::StartBusy { owner: Token }`, as
  `notes/contracts.md`, section 1 says, routed in `domain.rs` beside
  `StartRefused` and answered from `hosted/start_refused.rs` (a shared
  body, or a sibling function there) with `Ending::Refused(Refusal::Busy)`:
  the pending start's workspace released, its held refusals ended,
  nothing kept, a start the engine no longer wants taking the
  cancellation's path. Rows beside
  `a_refused_start_answers_invalid_with_its_words_and_frees_its_slot` and
  `a_start_refusal_after_cancel_keeps_the_cancellations_answer`
  (`jig/crates/jig-domain-host/src/tests.rs`): a busy start answers busy
  and frees its slot; after a cancel it keeps the cancellation's answer.
- **jig's test worker** sends `StartBusy` for smith's busy:
  `hand_out.rs`'s `smith::Refusal::Busy` (today `StartCause::Other`) and
  `domain.rs`'s `smith::End::Busy` (today `StartCause::Other`), every
  other refusal as today.
- Your status entries deleted (`notes/common.md`): in
  `jig/docs/design/domain/hosts.status.md` the **Not yet** sentence for
  6.3, 8; in `tasks.status.md` the 5.5 sentence; in `engine.status.md`
  the 8 sentence "an invalid start other than a transcript's is the
  invalid class, not the permanent one."

## Inputs

- `jig/docs/design/domain/tasks.md`, 5.5 (*Tries, within a task*: the
  classes, *invalid*, and the refusal before anything ran).
- `jig/docs/design/domain/hosts.md`, 6.2 (step 9), 6.3 (*refused*) and
  section 8's last paragraph.
- `jig/docs/design/domain/engine.md`, section 8 (*A refused start keeps
  nothing*).
- `temper/docs/plan/wake-faults/notes/common.md`; `notes/contracts.md`,
  sections 1 and 2.

## Owns and may change

- Owns the files in `files`.
- Compiler-forced arms for the new variant wherever `FromAgent` is
  matched: `jig/crates/jig-domain-host/src/boundary.rs` (its owner),
  `jig/tests/host/src/refusals.rs`, `jig/tests/host/src/world.rs` and any
  other; name each. temper's worker produces `FromAgent` and matches none
  of it; if a temper file must change to compile, it is a compiler-forced
  arm only, named.
- Tests that pinned a refused start's class as invalid through the hub
  link (look under `jig/tests/**` and `jig/crates/jig-domain-core/src/tests/**`
  for an `Invalid(Start ..)` or `Invalid(Charter|Grants|Messages)`
  ending fed through `payload::ending`): updated to permanent, named.
  `jig-domain-core`'s `run_choice.rs` rows that feed
  `tasks::End::Failed(tasks::Class::Invalid)` directly test the core's
  handling of that end, not this mapping: leave them.
- Pre-approved: if `start_refused.rs`'s body is clearer split, a
  `start_busy` beside it in the same file.

## Steps

1. The table; run it; record the failure seen.
2. The mapping; `StartBusy` and its handling; the host rows.
3. The test worker's busy.
4. Break each once (map invalid back to `Invalid`; answer `StartBusy` as
   an invalid start), see the rows fail, restore.
5. Your status entries.
6. `cargo fmt`; `python3 -I -B jig/scripts/old_forms.py`; `heavy cargo
   clippy -p jig-domain-hub-link -p jig-domain-host -p jig-test-worker
   -p jig-host-world --all-targets -- -D warnings`; `heavy cargo nextest
   run -p jig-domain-hub-link -p jig-domain-host -p jig-host-world -p
   jig-channel-world -p jig-core-world`; `heavy cargo check -p
   temper-worker-domain --tests` and every package `cargo tree -i
   jig-domain-host` lists in temper.
7. Commit and push.

## Report

## Rebase (2026-11-08, step merge, foreman wake-faults/start-refusals/foreman-1)

resolve engine.status.md by preserving both tasks’ gap deletions
The merge conflicts in `jig/docs/design/domain/engine.status.md`, in the first **Not yet** bullet.
Integration commit `f15f70e6d7` (“Keep covered undecided entries parked until a change”) removed the covered-entry gap; preserve that deletion.
This branch removes section 8’s invalid-start classification gap; preserve that deletion too.
The combined bullet must read:

- **Not yet:** 7.1: a message or an entry that arrives between a run's
  preparation and its claim, and would add to its reasons, is kept back
  rather than preparing the run again. 7.4: a finish while the task's
  delegates are live fails its attempt invalid rather than parking it.

Keep the remaining status entries unchanged. `tasks.status.md` auto-merged without a reported conflict.
