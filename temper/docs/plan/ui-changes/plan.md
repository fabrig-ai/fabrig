# ui-changes: the plan

From `inputs/request.md`: the `Changes` page as the greenfield mock draws it, the watch `changes` that
serves it, and a small `LandingLog`, family 30, so that what landed keeps its title past its change's
close. The first of three plans of the area, `ui-settings` and `ui-settings-tabs` after it.

The design is cut ahead of the tasks and lands with the first push: `temper/docs/design/client/README.md`,
3.5 (*Changes*), 4.6, 5.3, 5.10 and section 11; `client/browser.md`, 2.3; `client/forge.md`, sections 3, 9
and 10; `domain/after-landing.md`, 10; `domain/forge.md`, 9; `protocol/store.md`, 2; their status files.

## Parts and tasks

| # | Task | Needs | Effort, minutes | What it lands |
|---|---|---|---|---|
| 1 | `changes-wire` | | medium, 35 | the subject and its document on the wire; refused by the engine |
| 2 | `landing-log` | | high, 100 | family 30, written at each merge, erased with its repository |

## The critical path, as it stands

`landing-log` (100). Later tasks follow in the next push.

## Decisions that change a format, a protocol or an interface

There is no supported deployment, so each is decided here and reaches the user through this list.

- **Stored:** family 30 `LandingLog` joins `store-v11` in place (`protocol/store.md`, 4). Beyond the request,
  each landing also keeps its branch: "main last moved 1 h ago" must name a branch with nothing queued.
- **Protocol:** `Watching` gains `changes`, with `ChangesSubject` and `ChangesPanel` (`client/README.md`, 4.6).
- **Limits, public names:** `forge.landings_kept` (16), `forge.changes_rows` (32), `forge.changes_landed` (8).

## Decisions the request left open

- **A pull merged by someone else, or by the forge's own queue, is logged too:** a change of temper's did
  land. The request's "moves from outside" stays out: a push with no change.
- **`ChangesPanel.landing`** names the branch while nothing is queued, and a queue with no rows carries a red
  branch with nothing ready, so "main red" holds on an empty page. The request's document had no place for
  either.
- **`more` is one count across the queues,** which fill in order, the fewest moving parts.
- **The first row's time is `Change.since`,** when the step it is at began: no store of when checks began.
- **Landed recently's time is 5.12's *clock*,** since the mock draws "05:01".
- **A project that lands by turn** has an empty Queue on this page and lists what it lands: the mock draws
  one queue, and turn is not the default (`domain/landing.md`, 1).

## Overlaps with the drafts beside it

- `changes-wire` and `landing-log` touch no file of `claude-always`, `ui-signin`, `debt-17` or `hearth-memory-3`.
