# root: status

2026-11-09. Where temper's code lags or departs from [root.md](root.md).

Section 4's journal stops carry their reason through the service to an
exit and a log line, including a refused store answer, a decision past
the store's write edge and an exhausted commit counter.

Sections 5.7 and 9.1 have no known core wait in the project-making
reproduction. The one-process world's reduced and whole incident stories
check the core calls and a fresh client before and after reopening the
same disk. The reduced sequence makes a third project membership; the
earlier goal, forget, settings change and refused declare are not needed
for the reproduced client failure. A client limited to two memberships
rejects the mandatory `me` while the root stays ready; the native and web
presets now take nine, as ctl does (`../client/tools.md`, section 12).
The stored-state row reopens the disk made with that client limit under
the fixed preset, without changing records. This proves no cause for the
request's public read of a private project returning 503; that remains unexplained
(section 13).

Section 7.1 has no known gap in assigning an asking charter no home,
explicit, saved-work or context repository. Its run therefore has no
workspace; the other charters keep their repository selection.

Section 9.1's owner handoff is present: after both domains load, the root
hands the forge `people.sole_owner()` before its restore decision. The
connector's retained-link scope gap remains in `forge.status.md`, 4.6.

Section 5.2 has no known implementation gap in sizing task reads by the
live attempt's worker reply bound, else the local agent's, else
`documents.text_bytes`, within the core and journal bounds. The six-row
`task_reads_take_their_workers_reply_bound` table covers the worker-only
1,500-byte specification in one call and a smaller worker bound in two,
both fallbacks, continuation and the journal ceiling
(`temper/tests/engine/domain/tests/suite/host_calls.rs`). The host-answer
room is shared across the box's runs, as `fleet.calls` bounds the fleet
rather than each run. No concurrency gap is known in that room; the
settings-answer bound gap remains below.

Section 10 has no known gap in deriving both watch rooms from the admitted
streams. Its settings-answer bound gap remains below.
The charter's stored-policy size refusal carries supplied bytes and the
`root.journal.run_bytes` maximum from jig through ctl, a run's settings
tool and JSON. The policy's parts count toward that size; the refusal
kind remains shape.

Section 8's transcript load has no known gap at an activation: a task
whose window would find every load slot taken, a retired one awaiting
its reclaim among them, is answered busy and stays due, as with the
archive's read table full (`temper-engine-domain/src/route.rs`,
`activate`; `loads.rs`, `has_room`). With the shipped two load slots,
tasks due together each load their window, none failing its preparation
or spending a try (`tests/engine/domain`'s
`tasks_due_together_past_the_load_slots_each_load_their_window`).
Whether room was why about 23 briefs failed on their windows after a
deployment's restart on 2026-11-08 is unproven: its evidence is outside
the repository. A window's later page is below.

Section 12 has no known gap in a duplicate person's identity: a store
holding one refuses the start as jig's `Failure::Link`, the second
person's row naming the first's, and the engine stops with that reason,
`temper serve: restart refused during PHASE: RECORD links missing
LINKED`, as for every link jig's start cannot make
(`tests/engine/domain`'s `a_duplicate_person_identity_refuses_naming_both_rows`;
`temper-engine-shell/src/serve/host.rs`, `start_failure`).

## Gaps and departures

- **Not yet:** `ui-ask-m1`'s `proposal-read` part was never designed,
  according to the retirement brief. Its landed run proposal read needs
  design coverage reconciliation against `jig/docs/design/domain/engine.md`,
  7.3; this is a design gap, not an absent implementation.

- **Not yet:** 5.2: real-deployment savings are unobserved: no supplied
  real goal's `read_task` count establishes what the room saves, where the
  request `read-task-room` of 2026-11-06 takes that count as its proof.
  The engine world holds the worker-bound reads on a script alone; no
  real host or LLM is exercised
  (`temper/tests/engine/domain/tests/suite/host_calls.rs`).
- **Not yet:** 7.2: a section for each
  document a charter names (the forge's brief sources are CI, reviews, the
  pull request and the home: `temper-engine-domain-forge/src/boundary.rs`,
  `BriefSource`), and a merge-queue ejection's reason. 9.1: the
  rehearsing root, `temper upgrade --check`.
- **Not yet:** 8: a transcript window's later page, requested without
  another activation, still answers the core `TranscriptLoad::Failed`
  when no load slot is free (`temper-engine-domain/src/restart.rs`,
  `request_load`; `archive.rs`'s
  `a_transcript_continuation_without_load_room_remains_preparation_failure`),
  where the section answers a full table busy and jig's
  `domain/engine.md`, 9.1 counts room as capacity, never transient. The
  slot its previous page held stays taken until the iteration's reclaim,
  so a window past one page, loading beside another load, can spend a
  transient try on room; no story shows whether one does.
- **Not yet:** 9.1: the forge's link of its restored records panics at
  the first it cannot make, `forge link KIND -> NEED for KEY`, where
  step 3 refuses the start naming the record and what it links to, as
  jig's children's do
  (`temper-engine-domain-forge/src/reconcile.rs`, `reconcile`;
  `missing_link` already finds the link). The request
  `store-crash-safe` of 2026-11-09 asks that a store failing a link
  check at start stop with a reason, not a panic; its plan built that
  in jig alone.
- **Not yet:** 10: the settings-answer bound must include temper's declared
  application kinds, because `temper-engine-domain/src/limits.rs`, `valid`,
  passes an empty kinds slice to the core's `reads::answer_bound`, while
  `decision.rs`, `host_within`, relies on that bound for settings output
  held before its commit. The application settings' contribution is omitted;
  no reason is recorded.
