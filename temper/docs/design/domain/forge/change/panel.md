# A change's panel

This document holds a change's panel, split from [`README.md`](README.md).

## 1. Its panel

What the connector makes of a change for the client's panel
(`../../../client/forge/served.md`, 2), as the core asks for the document
(`../../routing.md`, 3), from its record and its working set:

- **From its record:** its repository, by owner and name from the
  repository's record; its branch, landing branch and head; its pull
  request's number; its state, with the step it was held at; the task
  its one delegate runs; its gates and their verdicts, each judged at
  the current head with its freshness (`gates.md`, 1), a gate whose review runs at
  that head pending until it reports; its repairs, resolutions and
  updates used, with their limits from `change_policy`; and its
  questions, its route, those taken of its bound and its open ones
  (`questions.md`, 1).
- **CI at its head** is what its last step read (`../keeping-up.md`, 3), a held change's
  read for a green head (`repairs.md`) among them, kept in memory
  beside its record, one verdict per change, while the change is live: a
  restart forgets it, and the panel says unknown until the change's next
  step reads it again. A change that has landed or ended keeps its last
  verdict at its exact head with its record (`ChangeRow`'s `ci`, written
  by the save its finish already makes), restored with the row, since no
  step will read it again and its panel stays as long as its record
  (`../../../client/forge/served.md`, 2). The panel carries separately whether the
  repository has CI and its effective landing rules ask for it (`gates.md`, 2).
  Where CI is not required, the client says “CI not required”, even if
  the retained observation is pending or failed. The observed verdict
  remains evidence at its exact head; a no-CI landing never makes it a
  passing verdict. The requirement is read from the held rules when the
  panel is made, including after restoration.
- **Its wait for the outbox's room:** none, or `OutboxRoom` while its
  record holds a place in the outbox's line (`../room.md`, 7), cleared
  by the save that admits it or takes it out of the line. The client
  says `Waiting for room in the forge outbox.` while it is not held.
- **Its place in its queue** is computed as the document is made,
  in `../queues.md`'s order among the changes queued or first in its
  landing branch at the core's time: its place from 1, and the tasks
  ahead of it, nearest first.
- **Marked** as 9.2 of the client's says: its own subject each time its
  record is saved and each time a step reads a CI verdict at its head
  other than the one kept; and, when it enters its landing branch's
  queue, leaves it or moves in it, the changes behind it, nearest first,
  within the views' subjects marked per change (`../../root.md`, section 9).
  One past that bound shows its new place at its own next mark.
- **In its project's scope:** a change into a repository the project
  holds. Any other task, or a scope that is not that project, is
  unknown. Before the change's record exists, the engine still answers
  Gone; the client's bounded reopen and its limit are in
  `../../../client/forge/served.md`, 2, **Its subject**.
- **Kept as long as its record:** the record outlives the task's end, so
  a landed or cancelled change still has its panel, until a new change
  needs its room (`records.md`, section 1).
- **Its rules and its charter:** what the landing rules wait on at its
  head, the rule and criterion its last verdict named (`gates.md`, 2), and the
  forge's charter it was made with (`gates.md`, 3).

## 2. Open questions

None.
