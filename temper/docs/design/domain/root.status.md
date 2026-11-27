# root: status

2026-11-26. Where temper's code lags or departs from [root.md](root.md).



- **Departs:** 9: the store's fill mark is jig's standard profile's, not small's,
  because standard's map budget is that mark and temper's boxes take standard: under
  small's mark, sign-ins would be refused once the map passed small's budget
  (`temper/crates/temper-engine-domain/src/limits.rs`, `FILL_MARK`).



Section 4's journal stops carry their reason through the service to an
exit and a log line, including a refused store answer, a decision past
the store's write edge and an exhausted commit counter.

`routing.md`, section 1 has no known gap in a run's read as its person: a `see`
flight of a person's address asks the core `PersonRead` and takes
`Now::PersonAnswer` back (`route/see/person.rs`, `route/now.rs`), and an
`Ask::Read` whose reply is such a flight is answered by the forge's
document for the person and the project's scope, any other connector's
by none (`route/asks.rs`); nothing of it is committed. The root's own
tests read each address's reads as a step function
(`temper-engine-domain/src/tests/see.rs`) and the engine world's
`configuring/asking_person.rs` reads them through jig and the forge. The
run's proposal read that `see proposal` makes is jig's
`domain/engine/executors-tools.md`, 1, *A run's proposal read*, routed as `routing.md`, 2 routes
jig's named actions (`route/see/proposal.rs`, `NamedAction::ReadProposal`).

`routing.md`, section 2 has no known implementation gap in sizing task reads by the
live attempt's worker reply bound, else the local agent's, else
`documents.text_bytes`, within the core and journal bounds. The six-row
`task_reads_take_their_workers_reply_bound` table covers the worker-only
1,500-byte specification in one call and a smaller worker bound in two,
both fallbacks, continuation and the journal ceiling
(`temper/tests/engine/domain/tests/suite/host_calls.rs`). The host-answer
room is shared across the box's runs, as `fleet.calls` bounds the fleet
rather than each run. No concurrency gap is known in that room; the
settings-answer bound gap remains below.

Section 9 has no known gap in deriving both watch rooms from the admitted
streams. Its settings-answer bound gap remains below.
The charter's stored-policy size refusal carries supplied bytes and the
`root.journal.run_bytes` maximum from jig through ctl, a run's settings
tool and JSON. The policy's parts count toward that size; the refusal
kind remains shape.

Section 11 has no known gap in a duplicate person's identity: a store
holding one refuses the start as jig's `Failure::Link`, the second
person's row naming the first's, and the engine stops with that reason,
`temper-box serve: restart refused during PHASE: RECORD links missing
LINKED`, as for every link jig's start cannot make
(`tests/engine/domain`'s `a_duplicate_person_identity_refuses_naming_both_rows`;
`temper-service-shell/src/host.rs`, `start_failure`).

## Gaps and departures

- **Not yet:** 9: the settings-answer bound must include temper's declared
  application kinds, because `temper-engine-domain/src/limits.rs`, `valid`,
  passes an empty kinds slice to the core's `reads::answer_bound`, while
  `decision.rs`, `host_within`, relies on that bound for settings output
  held before its commit. The application settings' contribution is omitted;
  no reason is recorded.
