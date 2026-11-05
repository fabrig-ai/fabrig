# ui-goals: the plan

From `inputs/request.md`: a goal's report as a note, the project's Goals page, the rail's Goals and the
tab bar, drawn as the greenfield mock's `goals.html`. The design is cut ahead of the tasks:
`temper/docs/design/client/README.md`, 3.5 and 5.3; `browser.md`, 2.3 and 2.5;
`temper/docs/design/domain/agent.md`, 8.3 and 4.3.2.

## Parts and tasks

| # | Task | Needs | Effort, minutes | What it lands |
|---|---|---|---|---|
| 1 | `report-parse` | | high, 40 | `report::parse`, `read`, `newest` |
| 2 | `delegate-counts` | | high, 90 | `TaskLine.delegates_done` and `delegates_total`, from a kept counter |
| 3 | `brief-numbers` | | medium, 30 | a brief's task section names its number and project |
| 4 | `report-stock` | 3 | high, 90 | the stock's note family and scope, a goal's own scope, both charters |

The rest of the plan follows in later pushes: the stock and the charters, the delegates' counts, the
address and the domain's page, the view, its look, the gallery, the rail and the proof.

## Critical path

As it stands: `delegate-counts`, ~90 implementer-minutes, beside `report-parse`.

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
