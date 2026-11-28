# briefs: status

2026-11-28. Where the tree's code lags or departs from [briefs.md](briefs.md).

- **Not yet:** 1, 1.1 and 1.4: the brief child's placing of the sections connectors plan, its
  gather time, and its answers named by connector and token [in flight: change 22491].
- **Not yet:** 1, 1.1 and 1.4: the engine asking connectors to plan, gather, cut and drop a
  brief's sections through the connectors' port, and keeping a brief complete until its run is
  prepared; jig's roots answering none of it [in flight: change 22500].
- **Not yet:** 1 and 1.4: temper's forge planning, gathering and cutting its sections through the
  connectors' port, and temper's root answering none of the brief [in flight: change 22502].
- **Design departs:** 1.4 departs from `temper/docs/design/draft/jig-domain-api.md`, 6.3 and 6.6,
  in three places, each with its reason in 1.4: a brief's plan is an operation that ends later,
  not a judgement; it is asked of every connector that serves sections, not only of those the
  task names; and a cut is answered with its size, not a notification. Decided by the goal's
  designer, 2026-11-28.

- **Not yet:** 1 and 2: a standing task's Now, its own scope, no contract and no transcript tail on a session's first run (`../tasks/standing.md`, sections 2 and 6).
- **Not yet:** 1.2: the waiting section's open questions, with their askers, numbers, words and answer form; nothing plans `Core::Waiting` yet [in flight: change 838].
- **Not yet:** 1.1, *A dependency's result*: every failed result read, busy, no row, another's row or one not ended, counts as transient with the one reason `results: a dependency's result could not be read`, naming neither the task nor what was read (`jig-domain-core/src/routing/run/results.rs`, `loaded`; `runs/retained.rs`, `dependency_result`).

- **Not yet:**  engine slots, hello and placement remain `fleet.status.md` (Not yet, the whole document).  the point-load fences remain `vocabulary.status.md`, 1.2, and `decisions.status.md`, 1.3.
  1: the brief's lineage and calls sections, and the waiting section's proposals and escalations, which reach a run as inbox entries instead.  turn-call objects remain `executors-transcripts.status.md`, 1.  the missing tools remain `executors-tools.status.md`, 1.
   the restart-oracle gaps remain `../testing.status.md`, 9.
   the pending-reasons continuation gap remains `settings-proposals.status.md`, 1.
   the settings-holder gap remains `settings-proposals.status.md`, 1.
   the ignored restore stories remain `settings.status.md`, 10.

- **Not yet:** 2.1, *Its runs' parts are its party's*: a brief's
  transcript tail renders a private tool's calls whole, `call TOOL:` and
  its input and then its result's summary
  (`jig-protocol-transcript`'s `brief.rs`, which takes no private names),
  and the tail is a text of the fresh start's turn outside the private
  section, so the turn's projection keeps it and every other party reads
  it (temper's `temper/docs/design/domain/agent/ask-temper.status.md`, 4).

- **Not yet:** 2.1, *Read and kept by its party alone*: a party naming
  their own scope and another party's entry by its name is refused as
  moved, with that entry's revision, not as unknown: the core admits the
  scope as the party's own (`routing/effect/note.rs`,
  `edit_note_request`), and the notes child answers an entry of another
  scope `Moved` (`jig-domain-notes`'s `domain.rs`, `Action::Edit`); a
  run's `NoteDelete` or revision of an entry another scope holds is
  answered the same way.

- **Not yet:** 2.1, *Read by those runs*: no check holds
  `core.brief_core_budgets.preferences` above `notes.party_entries` lines of
  `notes.party_bytes` numbered, so a deployment whose overrides pass it
  fails every asking brief of a full scope as not fitting its budget, and
  one that lowers either bound under kept lines fails them too. No test
  reads a preferences gather that fails, for room, refusal or fit.

- **Not yet:** 2.2: the wait table bounded by `brief.briefs`, not by live tasks, and a wait that finds it full aborts the engine (`jig-domain-core/src/routing/run/preparation.rs`, `start_brief`; `routing/effect/note.rs`, `defer_brief`).
- **Not yet:** 2.1: the `preferences` section is given where the run's task's authority holds the asking family, whatever its charter names, so a run whose charter names no asking family reads its person's preferences too (`jig-domain-core/src/routing/run/preferences.rs`, `party`).
