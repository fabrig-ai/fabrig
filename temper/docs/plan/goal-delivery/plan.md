# goal-delivery

A goal's run modifies its checkout, delivers to its own run branch, and lands that through a change into g (`inputs/request.md`; `temper/docs/design/draft/goal-as-designer.md`, 4). All five of the request's items are in the plan, in six tasks, and nothing is dropped.

## Parts and tasks

| Request's item | Part | Task | Trees | Estimate | Needs |
|---|---|---|---|---|---|
| 1. The failing story | `story-red` | `goal-story` | temper tests | 50 min | none |
| 2. A goal may deliver | `delivery-grant` | `charter-delivery` | jig | 50 min | none |
| 3. `"run"` beside `"mine"` | `run-change` | `run-branch` | temper | 90 min | none |
| 4. Later runs deliver too | `goal-workspace` | `goal-start` | temper | 110 min | `charter-delivery`, `run-branch` |
| 1, 4. The story and its walks | `story-green` | `goal-walks` | temper | 90 min | `goal-story`, `goal-start` |
| 1. The real sides | `real-story` | `goal-real-story` | temper tests | 80 min | `goal-start` |
| 5. The design | none | written by this cut | jig, temper | | |

## Critical path

`run-branch`, `goal-start`, `goal-walks`: 290 minutes. `goal-real-story` runs beside `goal-walks`.

## What the cut decided

- **A writer slot was the plan's real question.** A goal's run holds its run branch's slot until it answers, and the engine hears of its pushes only then, so today a change on that branch would decide nothing until the goal's first run ended. That contradicts the draft's section 3: "Doesn't wait to finish". So a change on a branch its requester holds no longer waits for that requester's own run, and steps at the head it reads (`forge.md`, 8.5; jig's `connectors.md`, section 11). Its own delegates still wait for the slot, so a repair of a design change waits for the goal's run to answer or park. That is the price: the alternative, no slot on r at all, would make every delivery look like drift.
- **What grants `deliver`, in jig's terms:** a report among the outcomes, the policy's `modify`, and the root saying its workspace delivers (`Charter.delivers`), as a fact of the connectors' items. That keeps jig out of temper's business, and holds "where it holds a goal branch" in temper, where a goal branch is known. A chat is Open, a reviewer has a verdict, and neither an advisor nor a chat holds a goal branch, so all three inspect alone whatever their charter's `modify`. No new policy field or tool family, which would be a stored format and a public settings word.
- **A later run's start is the existing merge in progress:** `Start::Merge { branch: g, base: r's head }`, with g as its landing, pushing to r. I weighed `Start::Merged { g, [r's head] }`, which reads like the draft's words, but a conflict there fails the preparation for good: nothing would ever settle it for a goal. As a merge in progress, a conflict is the run's to settle, a head g holds is no merge, and every push fast-forwards r, even where the repository squashes (`forge.md`, 4.4). The worker needs no change, and `goal-start`'s three checkout stories hold that.
- **No stored format and no protocol changes.** A run change's row is a `"mine"` change's. `ForgeRepository.delivers` and `Charter.delivers` stay in the engine's process, and smith's wire already carries a `deliver` beside a report. So no question goes to the owners.
- **The story comes first, ignored.** It can't merge red, so `goal-story` writes it against the trunk and shows where it fails, and `goal-walks` lifts the ignore.

## Worth knowing

- **The reference's charter 5 turns on.** `temper/docs/examples/reference/settings.json` gives its goal `modify: true`, inert until now. Once this lands, a reference goal holding g is offered `modify` and `deliver`, while its words say "Never fix code yourself" and "Never write your goal branch yourself". The request leaves those words to another plan, so they stay. Until such a goal delivers, its checkout holds g's tree as before, though it takes r's writer slot and may save work. The plan that changes those words should know.
- **Stock goals don't change:** stock charter 1 has `modify: false`.
- **Checks at delivery** stand on `checks_must_pass` in the charter and on smith's own tests. The fake forge's files carry no mode, so `goal-real-story` likely can't seed an executable `.temper/pre-pr`.
- **`forge.md`, section 20 keeps two defaults:** a goal's run branch goes as its goal closes, with whatever it held; and a saved start overtaken by a repair ends its run stale.

## Overlaps

- `run-branch` and `goal-start` write `temper-engine-domain/src/route/forge_route.rs`, as `model-catalogue-2/catalogue-clients` does across `route/**`. Each spec confines its edit to named functions. The request expects this.
- `run-branch` grows the `change` schema, so a room figure may move in `temper-service/src/tests/stock_brief.rs`, which `model-catalogue-2/stock-models` also writes. The spec limits that to the figure.
- `goal-real-story` may add helpers to `temper/tests/real-loop/src/**`, and stays out of `tests/fixtures/**`, which `model-catalogue-2/process-fixtures` writes.
- `run-branch` and `goal-start` both write `temper-engine-domain`, so they run in that order, one writer per crate.
