# look-round-13: the plan

The look request `connect-flows` (`97ad67393b`), landed by `temper/docs/design/draft/look-requests.md`, 3 to 5.

## Parts

- `look`: the stylesheet and look tokens (`connect-look`).
- `dismiss`: `DismissMaking` (`dismiss-making`).
- `welcome`: "All ways" leaves the welcome (`welcome-ways`).

- `model`: `model-cards` (one set of cards, the panels, the demo key and holder buttons go), `model-end` (Ready goes, Accounts' line, its empty state, the opened row).
- `forge`: `forge-cards` (Choose and "A Forgejo elsewhere"), `forge-yours` ("Your Forgejo"), `forge-rows` (setting up in the row, Ready goes).

Later batches add the sheet and the proof with its account.

## Critical path, so far

`connect-look` → `forge-cards` → `forge-yours` → `forge-rows`; `connect-look` and `welcome-ways` → `model-cards` → `model-end`; `dismiss-making` → the sheet's rows.

## Overlaps

`temper.css` and `temper-client-view` sit beside `look-round-11`, `model-catalogue-2`'s Models tab and `ui-proposal-card`; rules go under their markers, and whichever lands second catches up.
