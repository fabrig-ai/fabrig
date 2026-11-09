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
| `window-pages` | `tasks/window-pages.md` | | temper | 1 and 2: the failing rows (window length against the page, the restart's row) and the fix: a window's later pages keep its slot |

Further batches (being written): a window turn too large for a load page
is loaded cut with a marker and the run starts fresh from its tail, in
jig (store, core, reference root) and then temper (service routing).

## Design written by this cut

- `temper/docs/design/domain/root.md`, 8: *A window's pages keep their
  load slot*. Its status: a **Not yet** 8, which `window-pages` removes.
- `jig/docs/design/domain/engine.md`, 16: *A transcript's load with no
  slot* now says a root never lets room fail a load.

## Critical path

`window-pages` (about 2 hours) alone so far.

## Overlaps

None with the drafts beside this one: `store-crash-safe/writer-rows`
changes `jig/tests/core` and `jig-domain-core`'s admission, not these
files; `ui-reaches` and `ui-p3e-words` change temper's client, service
stock and protocol files.
