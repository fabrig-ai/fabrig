# look-round-8: plan

"Configure temper" shows temper's own way: the card, and the reference beside the page. It lands the look request `configure-temper-own-way`, released by the user on 2026-11-06, by `temper/docs/design/draft/look-requests.md`, sections 3 to 5.

## The parts

The design is this cut's own, pushed with the specs: `temper/docs/design/client/README.md`, 3.5; `temper/docs/design/protocol/documentation.md`, 2.2 and 3; `temper/docs/design/domain/agent.md`, 8.1; and `temper/docs/design/draft/own-way.status.md`.

| Task | What it does | Needs | Minutes |
|---|---|---|---|
| `own-way-card` | the card, "Not sure yet?" in the head, the spacing, the two acts | none | 60 |
| `own-way-pane` | the stock page and its kind, the pane, the modes, the gallery's states | `own-way-card` | 90 |
| `own-way-words` | the configuring agent's paragraph | `own-way-pane` | 25 |
| `own-way-proof` | the render check, the screenshots, the design's text, the account | `own-way-pane`, `own-way-words` | 70 |

**The critical path** is the four in a line, about 245 implementer-minutes. The card and the pane share `configure.rs`, `look.rs`, `words.rs` and `temper.css`, so one follows the other. The words wait for the page they name.

## What the cut decided

- **The reference is a page of temper's compiled documentation,** a new kind, `way`. `temper-docs` is the one light crate both sides can reach: the client's view takes its `const`, and the agent's `read_docs` reads its table. A plain `include_bytes!` could reach only one side.
- **The agent reads it with `read_docs`, not in its instructions.** `configuring.txt` is already held by the stock's fit tests, and no limit moves.
- **The figures' boxes are made from the file's own bullets,** so that the user fills them in one place.
- **The table:** Workers says "The deployment's configuration" and QA says "Later", because `findings.md` isn't built. Workers' "temper itself" says "Several machines": the request's numbers aren't used. The line under the table says "unless it says otherwise".
- **Section 3's words moved** from "checked by CI and a review" to "whose tests run along the way": the reference's goal branches have neither (`reference-code-production.md`, sections 6 and 10).
- **Two overlay rules are narrowed or moved:** the thread's padding reaches only this opening; the table's row names are `**bold**` in a cell, because skein's reader has no row header. `notes/contracts.md`, 5, lists every rule.
- **A chat started beside the reference closes it.** Keeping it open isn't asked for.
- **Public interface:** `temper help` gains a page and `temper_docs::Kind` a variant. No stored format or protocol changes.

## Overlaps with other drafts

None of these could be avoided; each spec says how to meet it.

- **`configuring-words/stock-words`** rewrites `configuring.txt` and the same length line in `temper-service/src/tests/stock.rs`, and grows the words toward the stock's fit limits. That is why the words come late. On a catch-up: keep its words, add the paragraph, measure the length again, shorten only this paragraph if the fit fails.
- **`chat-reading-d/numbered-lists`** changes what the picture's list becomes. The rules hold both forms and the tests read the terminal's print, so either order works. If it lands first, `.temper-doc-flow ul` is not written; if second, that rule is left dead, harmless, for a later clean-up.
- **`look-round-6`, `look-round-7`, `chat-reading-a`, `ui-walk-large` and `ui-rename-focus`** add to `temper-client-view/src/look.rs`, `words.rs` and `temper.css`. Tokens and word numbers renumber on a catch-up; new rules stay under this plan's marker.
- **`first-usable-model` and `plainer-panels`** are cuts in flight, their files not yet known.
