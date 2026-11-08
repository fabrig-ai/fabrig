+++
id = "finish-parks"
part = "finish-parks"
kind = "feature"
needs = []
files = ["jig/crates/jig-domain-tasks/src/run.rs", "jig/crates/jig-domain-tasks/src/run/ends.rs", "jig/crates/jig-domain-tasks/src/tests/kept.rs", "jig/testing/jig-channel-kit/src/program_runs.rs", "jig/tests/channel/tests/suite/program_runs.rs", "jig/tests/tasks/tests/suite/lifecycle.rs", "jig/tests/tasks/tests/suite/funding.rs"]
effort = "high"
minutes = 75
+++

## Goal

A goal's first run on the program backend ended **failed, invalid**,
three times, a minute after its last successful read: its model ended
its cut with a report while the delegates it had just made were live.
The tasks child refuses a finish while its task's delegates are live
(`Problem::live_delegates`), but an agent's run has ended by then, so the
core's refused-answer route (`jig-domain-core`'s `route_refused_payload`)
fails the attempt `Invalid` with no reason, and the next run is told
nothing. The design now has such a finish **park** the run, as a finish
while kept does: no try spent, its finish not taken, the task waiting for
its delegates' results.

Acceptance:

- **The failing row first, at the lowest tier with smith's program
  backend and a scripted program:** a new `Case` of
  `jig/testing/jig-channel-kit/src/program_runs.rs` (say
  `Case::Delegated`) in which the running task has a live delegate when
  the scripted `claude` ends its turn with a report outcome
  (`StructuredOutput`), and a test in
  `jig/tests/channel/tests/suite/program_runs.rs` beside
  `a_program_run_answers_through_claude` asserting the run's terminal is
  `End::Parked`, the task `Active(Idle)` with its delegate live, its tries
  unchanged (no `invalid` counted) and no result committed. Run it before
  the fix and report what it showed (expected today: a terminal
  `Failed(Class::Invalid)`).
- **The fix, in the tasks child:** in `run.rs`'s terminal admission, a
  valid `End::Finished` from an **agent** task's attempt
  (`Executor::Agent`), not asking to cancel delegates and with no result
  follow-ups, while the task has live delegates and is not closing, is
  admitted as `End::Parked` (the kept path's park: turns kept, saved state
  kept, no try, `activation_ended` as for any park, `EndKind::Parked` in
  its activity), where today it pushes
  `Request::Refused { .. live_delegates .. }`. Every other executor keeps
  today's refusal. The task then wakes by its wake policy when its
  delegates' results arrive (`jig/docs/design/domain/tasks.md`, 8.3).
- A unit row in `jig/crates/jig-domain-tasks/src/tests/kept.rs` (or a new
  test there beside `shared_kept_rule_separates_claims_from_finish_authority`):
  an agent task with one live delegate answers `Finished` (report,
  `cancel_delegates: false`): parked, idle, no try, result not taken; the
  same with `cancel_delegates: true` still closes as today.
- The tasks tier's pins of the refusal are rewritten to the rule:
  `corrected_finish_cancels_delegates_and_closes_deepest_first`
  (`jig/tests/tasks/tests/suite/lifecycle.rs`) parks task 1 on its first
  finish, claims it again, and closes the tree deepest first on its
  corrected finish, as it does today; `funding.rs`'s row near line 199
  asserts the park. Where the tasks world's referee or the fuzzy
  `amendments.rs` (near line 492) expects `Refused(LiveDelegates)` for an
  agent's finish, it accepts the park instead.
- Your status entries deleted (`notes/common.md`): in
  `jig/docs/design/domain/tasks.status.md` the **Not yet** sentence for
  5.6, and in `jig/docs/design/domain/engine.status.md` the 7.4 clause
  "a finish while the task's delegates are live fails its attempt invalid
  rather than parking it;".

## Inputs

- `jig/docs/design/domain/engine.md`, 7.4 (*Finished done while kept, it
  parks* and *Finished while its delegates are live, it parks*).
- `jig/docs/design/domain/tasks.md`, 5.5 (*the run's own*, *invalid*) and
  5.6 (*A task ends after its delegates*).
- `smith/docs/design/protocol/programs.md`, 5.5 (*The outcome, one path*)
  and smith's `domain/run.md`, 7.3, for why the run can't hear a refusal.
- `temper/docs/plan/wake-faults/notes/common.md`; `notes/contracts.md`,
  section 4.

## Owns and may change

- Owns the files in `files`. In `run.rs`, only the live-delegates check
  of the terminal admission (today lines ~503–512) and what it needs;
  task `covered-entries` edits `run.rs`'s claim (`read_escalations`, line
  ~136) at the same time (`notes/contracts.md`, section 4).
- `jig-domain-core`'s `route_refused_payload`
  (`jig/crates/jig-domain-core/src/routing/run/calls.rs`) is left as it
  is: other refusals still take it.
- Reader files, edited only to follow the rule: the tasks world's referee
  and stories under `jig/testing/jig-tasks-kit/src/**`, and
  `jig/tests/tasks/tests/suite/fuzzy/amendments.rs`; name each.
- Pre-approved: if the channel kit can't give the running task a live
  delegate (through the core's fixtures or the program's `delegate` call
  over MCP) within its `Case` set-up, write the failing row in the core
  world instead (`jig/tests/core`, an agent run with a live delegate
  answering `Finished`), keep the channel row as far as it goes, and say
  so. If parking needs a `kept` field rather than a branch beside
  `refuses_live`, put it in `run/ends.rs` beside `run_parks`.

## Steps

1. The channel row; run it; record the failure seen.
2. The park in `run.rs` (or `run/ends.rs`); the unit row in `kept.rs`.
3. The tasks tier's rows rewritten; break the rule once (restore the
   refusal), see the channel and unit rows fail, restore.
4. Delete your status entries.
5. `cargo fmt`; `python3 -I -B jig/scripts/old_forms.py`; `heavy cargo
   clippy -p jig-domain-tasks -p jig-channel-kit -p jig-channel-world
   -p jig-tasks-kit -p jig-tasks-world --all-targets -- -D warnings`
   (only the kits you changed); `heavy cargo nextest run -p
   jig-domain-tasks -p jig-channel-world -p jig-tasks-world`, then the
   tasks world's fuzzy suite with `--profile fuzzy`; `heavy cargo check
   -p jig-domain-core --tests`; `heavy cargo nextest run -p
   jig-core-world` (read the count).
6. Commit and push.

## Report
