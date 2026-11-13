# root: status

2026-11-12. Where temper's code lags or departs from [root.md](root.md).


Section 4's journal stops carry their reason through the service to an
exit and a log line, including a refused store answer, a decision past
the store's write edge and an exhausted commit counter.

`routing.md`, section 7, and `root.md`, section 8.1 have no known core wait in the project-making
reproduction. The one-process world's reduced and whole incident stories
check the core calls and a fresh client before and after reopening the
same disk. The reduced sequence makes a third project membership; the
earlier goal, forget, settings change and refused declare are not needed
for the reproduced client failure. A client limited to two memberships
rejects the mandatory `me` while the root stays ready; the native and web
presets now take nine, as ctl does (`../client/ctl/README.md`, section 9).
The stored-state row reopens the disk made with that client limit under
the fixed preset, without changing records. This proves no cause for the
request's public read of a private project returning 503; that remains unexplained
(`root.md`, section 12).

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
`domain/engine/executors.md`, 3, *A run's proposal read*, routed as `routing.md`, 2 routes
jig's named actions (`route/see/proposal.rs`, `NamedAction::ReadProposal`).

Section 6.1 has no known gap in assigning an asking charter no home,
explicit, saved-work or context repository. Its run therefore has no
workspace; the other charters keep their repository selection.

Section 8.1's owner handoff is present: after both domains load, the root
hands the forge `people.sole_owner()` before its restore decision. The
connector's retained-link scope gap remains in `forge/adoption.status.md`, 6.

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

Section 7's transcript load has no known gap at an activation: a task
whose window would find every load slot taken, a retired one awaiting
its reclaim among them, is answered busy and stays due, as with the
archive's read table full (`temper-engine-domain/src/route.rs`,
`activate`; `loads.rs`, `has_room`).

Section 7 has no known gap in *A window's pages keep their load slot*.
`loads.rs`, `loaded`, holds an answered window page that names a next
one for its waiter, uncut, and `next` issues the next page in that
slot in the same pass. A page not continued in its pass frees its slot
at `reclaim` (`archive.rs`'s
`a_transcript_continuation_keeps_its_slot_while_every_other_slot_is_taken`;
`tests/engine/domain`'s `loads::an_uncontinued_transcript_page_releases_its_slot_at_reclaim`).
With the shipped two load slots and two turns a page, windows of 2, 3
and 7 turns load whole while another chat's read holds the other slot.
A restart's due tasks with windows of one turn and of three each load
theirs and resume, none failing its preparation or spending a try
(`preparation::a_window_of_any_length_loads_while_another_load_holds_a_slot`,
`tasks_due_together_past_the_load_slots_each_load_their_window`). The
`transcript-window` request of 2026-11-09 reports goals held with
`transcript: its window could not be loaded`, and about 23 briefs
failed on their windows after a restart on 2026-11-08. Whether either
took the path these rows close is unproven: that evidence is outside
the repository. On that path, `root.loads.loads` refused a window's
next page while its last page's slot awaited the pass's reclaim and the
other slot was held. `documents.text_bytes` sizes a task read's room,
never a transcript's page.

Section 7 has no known gap in a window's turn handed on cut. The
store's cut page reaches the core as `TranscriptLoad::Cut`, as a
`TurnText::Cut` beside its row (`temper-engine-domain/src/boundary.rs`;
`archive.rs`, `transcript_page`). `temper-service`'s `route.rs`, `page`,
cuts a window page's first row past `root.loads.transcript_bytes`, or
alone past `root.loads.bytes`, the same way, and a later row past them
still ends the page before it. A store `Unread` or `Corrupt` page still
reaches the core as `Unread`, and the archive's row and text guards
still answer `Failed` (`store_route`'s
`window_turns_past_store_row_or_page_room_are_carried_cut` and
`unread_and_corrupt_window_pages_still_fail_the_load_once`;
`archive.rs`'s `malformed_transcript_rows_and_texts_still_answer_failed`).
Under the shipped `root.loads` and `root.journal`, a turn just inside
the row bound loads whole. One just past it, one twice the store's page
room, and one amid four short turns each start fresh at once, with the
marker in the turn's place in the tail, no brief failed, no try spent
and the root ready (`preparation::a_window_with_a_large_turn_starts_fresh_from_the_tail_that_fits`).

Section 11 has no known gap in a duplicate person's identity: a store
holding one refuses the start as jig's `Failure::Link`, the second
person's row naming the first's, and the engine stops with that reason,
`temper-box serve: restart refused during PHASE: RECORD links missing
LINKED`, as for every link jig's start cannot make
(`tests/engine/domain`'s `a_duplicate_person_identity_refuses_naming_both_rows`;
`temper-service-shell/src/host.rs`, `start_failure`).

## Gaps and departures

- **Not yet:** 6.2: a goal's design writer's brief line: no brief says
  that `deliver` pushes Markdown straight to the goal branch
  (`temper-engine-domain/src/assemble.rs`, `plan_home_access`).
- **Not yet:** 6.2: a section for each
  document a charter names (the forge's brief sources are CI, reviews, the
  pull request and the home: `temper-engine-domain-forge/src/boundary.rs`,
  `BriefSource`), and a merge-queue ejection's reason.
- **Not yet:** 8.1: the forge's link of its restored records panics at
  the first it cannot make, `forge link KIND -> NEED for KEY`, where
  step 3 refuses the start naming the record and what it links to, as
  jig's children's do
  (`temper-engine-domain-forge/src/reconcile.rs`, `reconcile`;
  `missing_link` already finds the link). Serve retains its fail-stop report and backtrace. The request
  `store-crash-safe` of 2026-11-09 asks that a store failing a link
  check at start stop with a reason, not a panic; its plan built that
  in jig alone.
- **Not yet:** 9: the settings-answer bound must include temper's declared
  application kinds, because `temper-engine-domain/src/limits.rs`, `valid`,
  passes an empty kinds slice to the core's `reads::answer_bound`, while
  `decision.rs`, `host_within`, relies on that bound for settings output
  held before its commit. The application settings' contribution is omitted;
  no reason is recorded.
