# ui-qa-small: the plan

Home's false empties and three lines of words, from temper/qa's walk on a real model: `temper/docs/plan/ui-qa-small/inputs/request.md`.

## The parts and the tasks

| Part | Task | Needs | Effort, minutes | What it gives |
|---|---|---|---|---|
| `words` | `three-lines` | | medium, 30 | items 2, 3 and 4: the signed-out word, and the stock's sentences on a repository's name and a proposal's Accept |

Item 1, Home's false empties, is still being written: its part and task join this table with the next push. The critical path as it stands is `three-lines`.

## What this cut decided

- **Item 4 is the model paraphrasing the stock, not a tool.** The tool answers `proposed as t1/4 to the deployment's owners` (`temper-engine-smith/src/answers.rs`, `proposed`), and the stock said "Give its number; say it awaits an owner." So the stock changes and the answer stays: the key is `temper ctl`'s, and worlds read it.
- **The stock's sentence is replaced, not only added to.** "Give its number" would contradict "without its key", so it goes, and "first give a proposal's number" with it.
- **A member's proposal stays true.** The request's sentence says "their Accept". A member's proposal waits for an owner (`temper/docs/design/domain/agent.md`, 8.1), so the stock says "or an owner's if they are not one".
- **One word changes, as the request names.** `temper serve printed` stays in `temper open`'s and the terminal's words, and in `commands.rs`. They are the same kind of fault, outside this request, and `three-lines` lists them under `## Left`.

## Overlaps not avoided

- `temper/crates/temper-service/src/stock/configuring.txt`, with `model-catalogue-1/configuring-told`. That branch already conflicts with main in four places, measured with `git merge-file` at its merge base. All three of this plan's edits together leave it at four, so they add none. Whichever lands second takes the other's paragraphs and makes its edits in them.
- `temper/crates/temper-service/src/tests/stock.rs` and `stock_brief.rs`, with `model-catalogue-1/stock-seven` and others: counted baselines, regenerated on a catch-up.
- `temper/tools/gallery/src/tests.rs` and `temper/tests/client-browser/src/lib.rs`: one line each here.
