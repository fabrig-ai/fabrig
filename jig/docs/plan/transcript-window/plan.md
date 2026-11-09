# transcript-window: the plan

A deployment at fabrig `69cff99edb` (`inputs/request.md`) held a goal's
coordinator, about an hour old, again and again with `transcript: its
window could not be loaded`, and a goal a few minutes old the same way;
after each of two restarts five tasks failed so for about a minute. The
evidence is outside the repository, so the plan pins each candidate with
failing rows, fixes what they show, and writes the window's rule into
the design.

## What the code says

- `transcript: its window could not be loaded` is the core's word for
  `TranscriptLoad::Failed` and for a row `append_transcript` refuses.
  temper's root answers `Failed` only from `archive.rs`,
  `transcript_failed`: a continuation page refused a load slot, a page
  its load seam cut, texts not matching rows, a row not a turn. A store
  answer that fails reaches the core as `Unread` (`... could not be
  read`), which a retry turns into a fresh start without the tail.
- A window is read `root.loads.rows` (2) turns a page. The page's slot
  stays retired until the pass's reclaim, and the continuation is asked
  in that pass, so it needs the table's other slot (`root.loads.loads` is
  2). Any other load in flight refuses it. A window past one page (three
  turns, a few minutes of tool calls) then fails at almost every try
  while the engine is busy: the held coordinator, the short goal, and a
  restart's due tasks reading their windows together. `restart-loads`
  (worker-relinks) made the activation wait for its first page's room and
  kept the continuation's failure (`archive.rs`'s test
  `a_transcript_continuation_without_load_room_remains_preparation_failure`).
- `documents.text_bytes` (512 on the deployment) sizes a task read's room
  (`temper-engine-domain/src/limits.rs`, `task_read_room`), never a
  transcript's page: ruled out by reading; no row.

## Parts and tasks

| Part | Task | Needs | Trees | Request item |
|---|---|---|---|---|
| `window-pages` | `tasks/window-pages.md` | | temper | 1 and 2: rows by the window's length against the page, the restart's row; the fix: a window's later pages keep its first page's slot |
| `turn-cut` | `tasks/turn-cut.md` | | jig (temper: the new field's two lines) | 1 and 2: rows by a turn's size against the page's room; the fix: the store hands such a turn on cut, the core keeps a marker and starts fresh from the tail |
| `temper-cut` | `tasks/temper-cut.md` | `window-pages`, `turn-cut` | temper (jig: one status sentence) | 1 and 2: rows by a turn's size against temper's bounds; the fix: the cut carried to the core, and temper's own row and page bounds cut rather than end the page |

Three tasks: the request's limit. Item 3, the design, is this cut's own
writing, not a task. Nothing is dropped.

Which bound refused: `root.loads.loads` (2), for a window's later page,
with its earlier page's slot retired until the pass's reclaim
(`window-pages` proves it by row). The restart's minute is the same path:
`restart-loads` made the activation wait for room, but a restarted task's
window past one page still asks its later pages without it, while other
due tasks hold the table (`window-pages`, Steps 2, rules it in or out).
The request's other candidates: a row past the window's room is
`turn-cut`'s and `temper-cut`'s (it was `Unread`, `... could not be read`,
not the words reported; and in temper a row past the root's own row
bound ends the page in `page_failed`); `append_transcript`'s bound
refuses no size (ruled out by two existing core tests, `turn-cut`
names them); the store's load refusing past a size is the same `Unread`.

## Contracts

`notes/contracts.md`: the store's `Event::Turns.cut`, the core's
`TranscriptLoad::Cut` and `CUT_TURN_TEXT`, and temper's default
`TurnText`. `turn-cut` builds the jig half alone; until `temper-cut`
lands, temper answers a cut page as it answered the store's `Unread`.

## Design written by this cut

- `temper/docs/design/domain/root.md`, 8: *A window's pages keep their
  load slot*; *An issued transcript load that fails*: a cut turn reaches
  the core cut, and so does a turn past the root's own page bounds. Its
  status: two **Not yet** 8 entries, apart: `window-pages` removes the
  first, `temper-cut` the second.
- `jig/docs/design/protocol/store.md`, 8.3: *A window's turn past its
  page's room*; *Direct complete-turn requests* now excludes a window's.
  Its status: a **Not yet** 8.3, which `turn-cut` removes.
- `jig/docs/design/domain/engine.md`, 7.2: *A turn too large for a page*,
  and *A window closed by an upgrade* now hands such a turn on cut; 9: the
  brief's tail carries a cut turn's marker; 16: *A transcript's load with
  no slot*. Its status: a **Not yet** 7.2, which `turn-cut` removes.

## Critical path

`window-pages` (about 2 hours) and `turn-cut` (about 2.7 hours) run
together; `temper-cut` (about 2.3 hours) follows both: about 5 hours.

## Merging main

main moved past this plan's base: `worker-relinks`' retire wrote a
**Not yet** 8 in `temper/docs/design/domain/root.status.md` for the same
later-page gap. This branch merged main and kept the trunk's entry in
its place, the one `window-pages` removes, so the trunk's merges stay
clean; `temper-cut` brings main's two status sentences on that gap
(temper's root status, jig's engine status) up to date.

## Overlaps

None with the drafts beside this one: `store-crash-safe/writer-rows`
changes `jig/tests/core/tests/suite/main.rs`, `named_people.rs` and
`jig-domain-core`'s admission and delegation, and
`jig/docs/design/domain/engine.status.md`, where `turn-cut` removes an
entry at the file's end and `temper-cut` edits one sentence; `ui-reaches`
and `ui-p3e-words` change temper's client, service stock, family and
protocol files, not its store routing.
