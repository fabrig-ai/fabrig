# producer-failure: the plan

From `inputs/request.md`. `walk = false`: the comparison's rerun is the proof.

## Parts and tasks

| Task | Part | Needs | Effort, minutes | What it lands |
|---|---|---|---|---|
| `failed-producer-holds` | `failed-producer` | | high, 60 | item 1: the hold, the fresh producer on release, the title default |
| `stock-after-asking` | `stock-words` | | medium, 15 | item 3: one sentence in `coding.txt` |
| `own-question-parks` | `asker-finish` | | high, 40 | item 2, jig: a run's own finish parks while its own question is open |
| `asker-left` | `asker-left` | `own-question-parks` | high, 75 | item 2, temper: the failing story, and a left question says so |
| `status-close` | `status-close` | all four | medium, 5 | the status files' entries leave |

## What was decided

- **Item 1:** the cause in the request holds, from reading: `producing` reads
  `Status::Failed` only in its early arms and when no head is pushed. Beyond the
  request, a release of that hold had no defined path, since 8.4 spoke only of "no
  head pushed". It makes a fresh producer, from the pushed branch, whose success gives
  the title. `Facts.titled`, computed and never stored, keeps a head with no title from
  opening by any other path.
- **Item 2 belongs in jig** (`jig/docs/design/domain/tasks.md`, 5.7), as a rule of any
  task with its own question open: the keep is jig's, and an application may not
  decide it (`jig/AGENTS.md`).
- **Item 2 parks; it does not refuse at the call.** This departs from the request's
  recommendation, recorded in `forge.status.md`. smith judges a `finish` with no call to
  its host, and the tasks child decides at the answer's commit (`engine.md`, 7.4). A
  refusal at the call would need a new crossing from jig through the worker to smith,
  which the request does not name, and it costs a turn that parking does not. The
  request's story still runs, with parking in place of the refusal. Its words reach no
  run. This is raised as the plan's question `finish-refused-at-the-call`; no task waits
  for it.
- **What still ends such a task:** a run failing without a finish, a cancel, and a
  procedure's failure, so `Kept::parks` stays as it is and `run_parks` sits beside it.
- **A left question:** a passed-up question says so at its answer, not at its asker's
  end, since saying it once at the end would need a stored latch.
- **Not covered:** an issue's title on a forge. The request's empty title is the pull
  request's; `issues` makes its own from a goal's spec.

## Critical path

`own-question-parks` (40), then `asker-left` (75), then `status-close` (5): about two
hours. `failed-producer-holds` (60) runs beside them.

## Overlaps not avoided

- `repair-green` changes `temper-engine-domain-forge-change/src/domain.rs` and
  `forge.status.md`. Item 1 has to change the same file; its edits stay in `producing`
  and `released_state`.
- `repair-green`'s `deployment-release` changes `jig-domain-tasks/src/control.rs` and
  `tasks.status.md`. `own-question-parks` does not touch `control.rs`.
- `ui-chat-draft`'s `chat-stock` changed `temper-service/src/tests/stock.rs` and
  `stock_brief.rs`. `stock-after-asking` adds one test there.
- `own-question-parks` leaves two temper tests red until `asker-left` merges. Nothing
  checks the trunk between them, and the spec names them.
