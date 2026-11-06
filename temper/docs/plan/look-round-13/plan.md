# look-round-13: the plan

The look request `connect-flows` (`97ad67393b`), landed by `temper/docs/design/draft/look-requests.md`, 3 to 5.

## Parts

- `look`: the stylesheet and look tokens (`connect-look`).
- `dismiss`: `DismissMaking` (`dismiss-making`).
- `welcome`: "All ways" leaves the welcome (`welcome-ways`).

- `model`: `model-cards` (one set of cards, the panels, the demo key and holder buttons go), `model-end` (Ready goes, Accounts' line, its empty state, the opened row).
- `forge`: `forge-cards` (Choose and "A Forgejo elsewhere"), `forge-yours` ("Your Forgejo"), `forge-rows` (setting up in the row, Ready goes).

- `sheet`: `list-near` (skein: a bound list's foot says it's near), `sheet-state` (the domain's half), `sheet-view` (the dialog, Adopt, "Forge number" goes, the old page goes), `sheet-copies` (Make empty, Copy public), `sheet-rows` (Copying, failed, Copy again, Dismiss), `sheet-story` (browser stories, compiled only).
- `proof`: `look-proof` (the render check against all 23 pages, the screenshots, the account, the design in line).

## Decisions

- **Dismissal keeps its scope in the request** (temper/ui, 2026-11-06): `DismissMaking` carries `into: Option<u32>`, the failed row's project or none for the deployment. Admission derives its scope from that field, so the same key joins its kept answer after erasure and restart. An existing row with another project is refused. This departs from the task's row lookup and the initial contract's request without `into`.

- **`Address::Adopt` stays,** as the sheet's address: every link keeps working, a reload draws the sheet, and the fewest readers change. Cancel goes to `Repositories`.
- **`Near` reaches a list, in skein,** rather than the picker reading all at once (it would read what isn't wanted) or a list dressed as a `log` (a log's live region and following are wrong for checkboxes).
- **Plural sends are the client's,** one `Adopt` at a time and one `MakeRepository` a line; the wire is unchanged for them. `DismissMaking` and `MakeRefusal::NotFailed` are new, in the wire and the store, with no compatibility kept (the user's rule).
- **`picker.rows` is a client constant of 2000,** not configuration, since it bounds the client's own memory.
- **`Text::Forge` stays in `Form` for `temper ctl`:** only Settings' "Forge number" goes.
- **Owned Forgejo from Settings** opens the welcome's sheet; the request leaves it unmocked, as the welcome has it.
- **Failed at `Taken`** offers "Adopt it" for "Copy again" (`repository-copies.md`, 9); the request's pages show only the sheet's.
- **Home's own sheet and the two ways into the sheet** (a project with no repository, To begin's second step) stay as they are: the request leaves them unmocked.
- **`model_check`'s "All ways" stays:** a stopped try's pane isn't a way page.
- Taken on the laptop's recommendations, reversible by the user: the demo API key goes; "Use deployment accounts" is one quiet link; "All ways" leaves the welcome; search and scroll replace "Next" (`forge.md`, 5.1); the forge sits inside "Adopt" (`repository-copies.md`, 9); "Forge number" goes.

## Critical path, so far

`dismiss-making` → `sheet-state` → `sheet-view` (also `list-near`, `connect-look`) → `sheet-copies` → `sheet-rows` → `sheet-story` → `look-proof`, about 505 implementer-minutes. Beside it: `connect-look` → `forge-cards` → `forge-yours` → `forge-rows`, and `connect-look`, `welcome-ways` → `model-cards` → `model-end`; both end at `sheet-story`.

## Overlaps

`temper.css` and `temper-client-view` sit beside `look-round-11`, `model-catalogue-2`'s Models tab and `ui-proposal-card`; rules go under their markers, and whichever lands second catches up. Unavoidable: `temper/tools/gallery/**` (`look-round-11/picker-menu`, `ui-proposal-card/card-states`), `temper/docs/design/client/README.status.md`, `mockups/STATUS.md` and `STATUS.md` (`ui-proposal-card/card-close`, `model-catalogue-2/models-proof`) at `look-proof`; `temper-client-protocol-wire` and its goldens (`model-catalogue-2` changes `jig`'s wire, not this one, so this overlap is small); `skein-ui-web` (`ui-proposal-card/focus-reveal`, already merged). Prints are regenerated, not merged. Within this plan, `temper-client-view` is shared by several families: each owns its own files, and the same-file overlaps are ordered by `needs` (`forge/connect.rs` by the forge's chain, `pages/connect_model.rs` by the model's, `pages/settings.rs` by the sheet's).
