# look-round-13: the plan

The look request `connect-flows` (`97ad67393b`), landed by `temper/docs/design/draft/look-requests.md`, 3 to 5.

## Parts

- `look`: the stylesheet and look tokens (`connect-look`).
- `dismiss`: `DismissMaking` (`dismiss-making`).
- `welcome`: "All ways" leaves the welcome (`welcome-ways`).

Later batches add the model's pages, the forge's pages, the sheet, and the proof with its account.

## Critical path, so far

`connect-look` → the pages. `dismiss-making` → the sheet's rows.

## Overlaps

`temper.css` and `temper-client-view` sit beside `look-round-11`, `model-catalogue-2`'s Models tab and `ui-proposal-card`; rules go under their markers, and whichever lands second catches up.
