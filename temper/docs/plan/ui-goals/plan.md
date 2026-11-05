# ui-goals: the plan

From `inputs/request.md`: a goal's report as a note, the project's Goals page, the rail's Goals and the
tab bar, drawn as the greenfield mock's `goals.html`. The design is cut ahead of the tasks:
`temper/docs/design/client/README.md`, 3.5 and 5.3; `browser.md`, 2.3 and 2.5;
`temper/docs/design/domain/agent.md`, 8.3 and 4.3.2; `temper/docs/design/protocol/configuration.md`, 13.4; and jig's
`client/domain/contract.md`, 5.1, `domain/tasks.md`, 3, and `domain/engine.md`, 9.

## Parts and tasks

| # | Task | Needs | Effort, minutes | What it lands |
|---|---|---|---|---|
| 1 | `report-parse` | | high, 40 | `report::parse`, `read`, `newest` |
| 2 | `delegate-counts` | | high, 90 | `TaskLine.delegates_done` and `delegates_total`, from a kept counter |
| 3 | `brief-numbers` | | medium, 30 | a brief's task section names its number and project |
| 4 | `report-stock` | 3 | high, 90 | the stock's note family and scope, a goal's own scope, both charters |
| 5 | `goals-address` | | medium, 60 | `Address::Goals`, its path, its arms, a heading in the frame |
| 6 | `goals-look` | | medium, 50 | the rules and tokens, IBM Plex Mono |
| 7 | `goals-domain` | 1, 2, 5 | high, 90 | its query, a report per row, its needs, `goals::state` |
| 8 | `goals-row` | 6, 7 | high, 75 | `goals::row`, `goals::ways` |
| 9 | `goals-page` | 8 | high, 90 | the page in both shells, `goals.txt` |
| 10 | `goals-gallery` | 9 | high, 90 | its states; Solo and Reference gain goals, notes and a selection |
| 11 | `has-goals` | 7 | high, 80 | `Domain::has_goals` |
| 12 | `goals-rail` | 10, 11 | high, 75 | the rail and the tab bar, `home-with-goals` |
| 13 | `box-goal` | 4, 12 | high, 75 | the fake box's goal script and its walk: the meeting point |
| 14 | `goals-proof` | 13 | high, 90 | renders beside the mock at three widths, the status closed |

## Critical path

`delegate-counts` (or `goals-address`), `goals-domain`, `goals-row`, `goals-page`, `goals-gallery`, `goals-rail`, `box-goal`,
`goals-proof`: ~675 implementer-minutes, then the trunk's end check. Tasks 1 to 6 start at once. The view, the stylesheet, the gallery
and the terminal's goldens then take one writer at a time, as the request asks, with `has-goals` beside them in the domain.

## Overlaps with the drafts beside this one

- `temper-fake/src/backends.rs`: `claude-always/always-offered` owns it; `box-goal`, late in the order, changes only `model_document`'s
  `"cued"` and `"asked"`, on whatever its base holds.
- `temper/docs/design/client/README.status.md`, `protocol/configuration.status.md`: `claude-always/fetching-words` edits them; this plan's lines
  are its own, so they should merge.
- `temper.css`, `frame.rs`, `words.rs`, `pages.rs`, `root.rs`, `look.rs`, and the gallery's `names.txt`, `samples.rs`, `tests.rs`: `ui-needs`,
  `ui-changes` and `look-round-5` are still being cut, so their files aren't known. This plan keeps to its marker in `temper.css` and its own
  `pages/goals.rs`, and regenerates on a catch-up (`notes/common.md`).

## What the cut settled

- **A goal's authority holds notes of its own scope,** where 4.3.2 said no notes (`agent.md`, 4.3.2).
- **"6 / 9" is on the task line,** neither the goal's document nor the tree's read, as the request
  offered. No record held a count of ended direct delegates, so all three places need the same stored
  counter; they differ in how it reaches the client. The line is the smallest: three lines of logic, no read
  per row, and about as many literals to touch as the document, against more for the tree or the page. A
  row therefore needs no `Read::Tasks` by `parent` either, the request's second read per row: only its
  report. Formats change in place (`jig/docs/design/protocol/README.md`, 7). The fallback `6+ / 9+` isn't
  needed.
- **Existing stores don't decode** after `delegate-counts`: there is no supported deployment.
- **A brief names its task and project,** beyond the request. The tool's goal scope needs both numbers and
  no brief or tool gave them, so a real model could only guess; the reference's charter 5 already asks for
  goal-scope notes on that footing. One line in the brief's task section is the fewest moving parts.
- **A goal's scope is narrowed in the route,** since `route/goal.rs` cleared `notes` for every goal, and the
  stock's tool mask gains note: the request's one line in `stock.rs` alone gave a goal no `note`. Observers
  (a role with no spend) keep no note scope.
- **The report's words are sentences in charter 1's goal paragraph,** not a paragraph of their own: its
  words "this paragraph alone is yours; the next is the chat's" are the charter's contract, and a test's.
- **"The tasks" read per row goes,** and with it the request's `Read::Tasks` by `parent`, since the line counts; planning is no delegates by
  `delegates_total`.
- **"More goals" replaces the page,** as "more closed chats" does, and the headline keeps counting the rows read before it, so that its number stays true.
- **A goal that needs the person and has no report** takes running's headline, or planning's with no delegates: the request's table gives none.
- **The ways' order:** the request says number order and shows the mock's. The stock's `goal` is number 2 and the reference's `goal for a draft` is
  4, so the real order is "task by task, plan, tasks, land whole", against the mock's the other way round. Number order stands, since it is the one a
  project's settings decide.
- **`Domain::has_goals` is a `bool`:** `false` until delivered, so a rail doesn't flicker on a project with none. Its two handles add to every
  page's tables, so `handles` and `watches` rise by 2 and 1 in all three presets, and `delegation::bound` reserves one more. These are
  table bounds, not budgets.
- **The fake's report names literal numbers,** those a fresh box's first chat makes true: its scripts are static. A later chat's goal is refused its note
  and ends reportless, a normal state.
- **The draft statuses:** neither `draft/reference-code-production.status.md` nor `draft/template-places.status.md` holds a lag this plan closes.
- **Dropped:** nothing of the request. "Start one from a draft" is `ui-goal-page`'s, as the request says.
