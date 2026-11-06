# ui-small-1: the plan

The request: what the user met on their own box, and a flake (`inputs/request.md`). Three tasks at most, focused and fuzzy alone, but for item 3's story by name.

## Parts and tasks

| Part | Task | Needs | Request item | What it does |
|---|---|---|---|---|
| small-words | `small-words` | | 2, 4 | the approver line, a chat's forge look, Home's repeat, a repair's blame |
| printed-link | `printed-link` | | 3 | a count of links, stderr in the panic, a fault fixed if found |
| link-why | `link-why` (to come) | | 1 | a link that can't sign in says why |

## Critical path

None to speak of: the three run side by side, each alone.

## Decided

- **A repair of a semantic conflict** says "Repairing: CI failed at its updated head.", the request leaving the words to the cut. It has no hole: naming the base was the blame. The domain's word for the reason stays, since only the person's line misled.
- **Home's empty *Needs you*** loses its first sentence and keeps the rest; the headline keeps "Nothing needs you." This departs from the mock, which says it twice (`temper/docs/design/client/README.status.md`).
- **The approver sentence** stays where it is, in the paragraph on reading the inventory, with its condition added: a test reads its first half.

## Overlaps

Could not be avoided, since the request names these files:

- `temper/crates/temper-service/src/stock/configuring.txt`: `ui-proposal-2/chat-revises` and `model-catalogue-2/configuring-learns`. One sentence here; whichever lands second catches up, as the request says.
- `temper/crates/temper-service/src/tests/stock.rs`: `ui-proposal-2/chat-revises` and `limits-fixtures/service-fixture`. A row and a pinned length.
- `temper/crates/temper-client-view/src/words.rs`: `limits-fixtures/view-words`, `look-round-13/forge-cards` and `ui-proposal-2/card-and-latest`. Single lines.
- `temper/tools/gallery/**`: several plans. One row, regenerated on a catch-up.
- `temper/crates/temper/tests/process.rs`: `limits-heavy/box-lifetime`. Two assertions.
- `temper/crates/temper/tests/support/**`: `model-catalogue-2/process-fixtures`. Three new functions beside `printed_link`.
