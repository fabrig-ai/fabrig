# landing-fixes-3

What the first real landing through temper met (`inputs/request.md`). Three ranked items in three tasks. Nothing is dropped.

## Parts and tasks

| Request's item | Task | Trees | Estimate |
|---|---|---|---|
| 1. A spec's report can't fit `finish` | `report-room` | jig, temper | 75 min |
| 2. A receipt pads the head | `head-words` | temper | 40 min |
| 3. A compare says too large on a small change | `compare-words` | temper | 45 min |

Item 1 ranks first but went out last, because it needed the most reading. Items 2 and 3 need nothing from it.

## What the cut decided

- **Item 1: a larger bound, 1024 to 3,584 bytes.** The whole path is sized for 10,660 except jig's `CONTRACT_TEXT_MAX` and the 4,096-byte outcome held by smith and the hub.
  - **Why not a named place:** a coding run holds no notes (`CODING_TOOLS` is 514), so a note means granting a family, teaching a foreman to read it, and a reader following a pointer.
  - **What it costs:** one constant, two sentences of stock and no stored change.
  - **What it doesn't do:** a report past 3,584 bytes is still cut. The run says under Left what it cut, and doesn't ask. Going to about 10 KB takes smith, jig and temper together, so past the request's three tasks: `temper/docs/design/domain/agent.md`, 10.
- **Item 2: the cause holds.** The survey also found `CI failed at head` and `CI passed at head` padding a head, and `head-words` fixes those too. Every other printer is width-aware or prints a short prefix.
- **Item 3: the refusal changes and the bounds don't.** The request's cause is half of it: `forge.client.rows` is 8, and a compare also holds both whole sides of every changed file within `forge.client.answer_bytes`, so one line changed in a long file is refused too. The refusal already named `read_pull_files`, but it never said the ids were read, and it pointed at a host bound that wouldn't help.

## A departure

`report-room` crosses jig and temper, against "a branch stays in its tree". The request caps the plan at three tasks, and splitting it would make four and drop item 3. jig's part is one value and its tests.

## Overlaps

- `head-words` touches `temper-engine-domain/src/route/forge_route.rs`, which `landing-fixes-2/repair-cause` and `update-recovers/update-hold-words` are also changing. The edit is confined to the two cancel reasons.
- `report-room` touches `temper-service/src/tests/stock.rs`, which `model-catalogue-1/stock-seven` is also changing. The edit is confined to two tests.
- All three remove their own line from `temper/docs/design/domain/agent.status.md`. The lines are kept apart so that their merges don't meet.

## Critical path

None: the three tasks need nothing from one another, so the longest is `report-room`, at 75 minutes.
