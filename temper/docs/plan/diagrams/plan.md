# diagrams: the plan

The request: build `skein/docs/design/draft/diagrams.md`, its cases the acceptance, byte for byte (`inputs/request.md`). Focused and fuzzy alone: the cases' scenes are checked as bytes, and temper/ui looks after landing. The look isn't this plan's to change.

## Parts and tasks

| Part | Task | Needs | What it does |
|---|---|---|---|
| scene-height | `scene-height` | | `Scene::set_height`, the SVG in the figure's flow, the page's lazy scene, `skein-ui.css`'s rules and the journeys' copy |
| flow | `flow-data` | | `skein_ui::flow`: owner, kind, tokens, data; empty `read` and `layout` files, so that the next two don't meet |
| flow | `flow-read` | `flow-data` | the subset, the fence's meeting point in `markdown.rs`, the reader's tests and the case's text |
| flow | `flow-layout` | `flow-data`, `scene-height` | `flow::draw`, and its table against the `.svg` cases |
| flow | `flow-cases` | `flow-read`, `flow-layout` | the meeting point in skein: `.md` to `.svg` whole; the text shell's golden |
| temper-flow | `temper-flow-look` | | `temper.css`'s rules for a flow's tokens |
| temper-flow | `temper-flow` | `flow-read`, `flow-layout` | the page's route and names, the limits; the meeting point in temper: `host::page` prints a case |
| temper-flow | `stock-flows` | `temper-flow` | `chat.txt` and `configuring.txt` name the subset |
| temper-flow | `flows-close` | every other task | the status files |

## Critical path

`flow-data` or `scene-height`, then `flow-layout` (or `flow-read`, the longer), then `temper-flow`, `stock-flows` and `flows-close`: about 25 + 60 + 40 + 15 + 10 = 150 implementer-minutes.

## Decided

- **The design is written with this cut:** `skein/docs/design/ui.md`, 3.3, 3.5, 10.1, 11.2, 11.5, 15, 17, 18; `ui-web.md`, 8, 12; `ui-text.md`, 5; temper's `client/README.md`, 3.5, `client/browser.md`, 2.5, `client/journeys.md`, 7, `domain/agent.md`, 8. The draft stays, with its cases, since tests read them and where they go is the user's: its status now holds that alone.
- **skein-ui's number** is `u32::MAX`, for no application numbers an owner there: temper's are below 65,538 and jig's the same.
- **An empty name leaves a drawing with no name,** in `Builder::drawing`, rather than a second builder call: the `review` case has none, and every caller today passes one.
- **The figure's marker,** `data-scene-height`, is on the figure, since a case's SVG bytes hold no attribute for it.
- **Zero cost for a page with no flow:** the reader takes no scratch until an info string says `mermaid`, and stops at a quarter of `drawing_bytes` in nodes. The page part makes its `Scene`, all reserved at once, the first time it draws. `temper-client-view`'s limits rise (2,048 bytes, 24 nodes' scene) without costing a page with none.
- **Where the subset left a choice, it stays code:** at least one edge, so a lone node isn't a flow; a node labelled twice, the same words or not; `:::` with anything but `strong`; a second `accTitle`; `<` other than `<br>`.
- **A scene past temper's limits, or a flow past 24 nodes,** shows its fallback: a flow's limits fit a document's real flow, and nothing is cut.
- **`read_growing`:** `end`'s nodes go in the fallback's paragraph, the block holding the last word, so temper's "writing" mark is hidden with the fallback while the flow's drawn at a growing reply's very end.
- **The gallery stays undrawn:** it measures no box, so a print shows a flow's fallback. temper/ui looks in a real chat after landing.
- **No browser test, no walk,** by the request. `flow-cases` and `temper-flow` hold the bytes in focused tests.
- **No fuzzy row:** the reader is a line machine with no state from one fence to the next, and its table rows name each refusal. The existing Markdown bound's tests stand.
- **The reference stays without figures** (the user's word of 2026-11-06). Its open item stays, its words corrected: flows are built now, and the reference shows none by that word.

## Overlaps

- `temper-flow-look` adds a block under `/* ==== ui: chats ==== */` in `temper.css`, where parallel plans add theirs; the marker exists so that they merge.
- `stock-flows` meets `model-catalogue-2`'s `configuring-learns` and `stock-models`, on `configuring.txt` and `tests/stock*.rs`. It comes last in this plan to stay off them, and its spec holds a catch-up's rule.
- `flow-read` and `flow-layout` both reach `skein-ui`, in separate files that `flow-data` leaves ready. `skein-limits-heavy/sweeps` touched `skein/crates/skein-ui/src/tests/date.rs` and `phrases.rs`, and `skein/tests/ui/`: no task here changes those.
- `scene-height`'s reprint writes `temper/docs/design/client/journeys/skein-ui.css`, a generated file, so a conflict there is resolved by reprinting.
