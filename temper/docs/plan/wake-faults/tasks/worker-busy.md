+++
id = "worker-busy"
part = "start-classes"
kind = "feature"
needs = ["start-refusals"]
files = ["temper/crates/temper-worker-domain/src/hand_out.rs", "temper/crates/temper-worker-domain/src/agents.rs", "temper/crates/temper-worker-domain/src/translate.rs", "temper/crates/temper-worker-domain/src/tests/lifecycle.rs", "temper/crates/temper-worker-domain/src/tests/entrance.rs"]
effort = "medium"
minutes = 45
+++

## Goal

temper's worker turns smith's *busy* into an invalid start: an agent
smith had no room for (`smith::End::Busy`, `agents.rs`'s `ended`) and a
run it refused as busy (`agent::Refusal::Busy`, `hand_out.rs`'s
`Finish::of`) both reach the hub as `FromAgent::StartRefused { cause:
StartCause::Other, .. }`, so the task spends a try (now permanent, after
task `start-refusals`) where a busy host is no failure. With
`FromAgent::StartBusy` from `start-refusals`, the worker says busy, and
the hub answers the start refused busy: placed again, no try.

Acceptance:

- **The failing rows first,** beside `refused_start_retains_cleanup_diagnostic`
  (`temper/crates/temper-worker-domain/src/tests/lifecycle.rs`) or
  `a_worker_shutting_down_refuses_assignments_as_busy`
  (`src/tests/entrance.rs`, near line 95), whichever already drives a
  start to its answer: a run smith answers `RunResult::Refused { refusal:
  Refusal::Busy }`, and an agent that ends `End::Busy` before it starts,
  each answer the engine once with `host::Ending::Refused(host::Refusal::Busy)`,
  0 turns, 0 spent, after the agent's cleanup as today. Run them before
  the fix: today each answers `Refused(Invalid(Start { cause: Other, .. }))`.
  If the existing test is a loop over refusals, these are rows of it.
- **The fix:** the worker keeps whether a refused start was busy (its
  `StartRefusal` in `hand_out.rs` gains it, say an enum of busy or an
  invalid `StartCause`, or `Finish::Refused` carries it) and sends
  `FromAgent::StartBusy { owner }` for it where it sends `StartRefused`
  today (`agents.rs`'s `ended` and `gone`); every other refusal as today,
  `refused_start_retains_cleanup_diagnostic`'s invalid row unchanged.
- Your status entry deleted (`notes/common.md`): in
  `temper/docs/design/domain/worker.status.md` the first **Not yet** 4.3
  sentence, "a start smith refuses as busy ...".

## Inputs

- `temper/docs/design/domain/worker.md`, 4.3 (*Refused*).
- `jig/docs/design/domain/hosts.md`, section 8's last paragraph, and
  6.3 (*refused*).
- `temper/docs/plan/wake-faults/notes/common.md`; `notes/contracts.md`,
  section 1.

## Owns and may change

- Owns the files in `files`; `translate.rs` only where `unstarted` or
  `start_cause` must tell busy apart.
- Reader files: other `temper-worker-domain` tests that assert a busy
  refusal's ending (`src/tests/harness.rs` names smith's busy), edited
  only to the rule, named.
- Pre-approved: if smith's `End::Busy` can't reach `ended` with a bound
  agent in the harness, test it through `translate::unstarted`'s caller
  as far as the harness reaches and say so.

## Steps

1. The rows; run them; record the failures seen.
2. The busy refusal kept and sent as `StartBusy`.
3. Break it once (send `StartRefused` again), see the rows fail, restore.
4. Your status entry.
5. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo
   clippy -p temper-worker-domain --all-targets -- -D warnings`; `heavy
   cargo nextest run -p temper-worker-domain`; `heavy cargo check -p
   temper-service --tests`.
6. Commit and push.

## Report
