# ui-qa-small: the plan

Home's false empties and three lines of words, from temper/qa's walk on a real model: `temper/docs/plan/ui-qa-small/inputs/request.md`.

## The parts and the tasks

| Part | Task | Needs | Effort, minutes | What it gives |
|---|---|---|---|---|
| `words` | `three-lines` | | medium, 30 | items 2, 3 and 4: the signed-out word, and the stock's sentences on a repository's name and a proposal's Accept |
| `empties` | `no-false-empties` | | high, 75 | item 1: no page claims empty until its read has answered, and a lost read drops nothing known |

The two tasks need nothing of each other and run side by side, so the critical path is `no-false-empties`. Two slices, no walk: the request asks no browser proof, and for item 1 a view test is enough.

## What this cut decided

- **Item 4 is the model paraphrasing the stock, not a tool.** The tool answers `proposed as t1/4 to the deployment's owners` (`temper-engine-smith/src/answers.rs`, `proposed`), and the stock said "Give its number; say it awaits an owner." So the stock changes and the answer stays: the key is `temper ctl`'s, and worlds read it.
- **The stock's sentence is replaced, not only added to.** "Give its number" would contradict "without its key", so it goes, and "first give a proposal's number" with it.
- **A member's proposal stays true.** The request's sentence says "their Accept". A member's proposal waits for an owner (`temper/docs/design/domain/agent.md`, 8.1), so the stock says "or an owner's if they are not one".
- **One word changes, as the request names.** `temper serve printed` stays in `temper open`'s and the terminal's words, and in `commands.rs`. They are the same kind of fault, outside this request, and `three-lines` lists them under `## Left`.

- **Item 1 has two causes, and both are fixed.** The views asked only whether a list was empty, and a watch backing off its opening is Behind with no `latest`, which fits the walk's Behind with T1 live. Both causes are read from the code; the walk itself isn't reproduced, and the ruling holds whatever provoked it. The domain also took a lost read for an answer: `Chats` and `Goals` dropped their ended rows on `ReadNone::Unreachable`, `HasGoals` forgot its goals, and `Inbox::delivered` turned true after a refused section read.
- **"The same claim" reaches further than the words the request names:** `Word::ChatsNone`, `Word::InboxUpdatesEmpty`, the terminal's count "0", `Word::HomeNeedsNone` in Home's headline, `Word::GoalsEmpty` and `Word::GoalsNone`. `ChangesEmpty` and `NoForges` are already guarded and stay as they are.
- **`known` beside `delivered`, not a tighter `delivered`.** To begin's gate and existing tests rest on `delivered`, which still turns false during a refresh. `known` latches, so Home's headline and Needs you hold through a refresh instead of going blank.
- **What a page keeps lasts as long as the page.** A new `Inbox` on navigation starts with nothing known and draws titles alone, which the ruling allows.
- **To begin's gate is unchanged.** A refused closed read still counts as delivered there. It claims no empty list, so it stays outside this request.

## Overlaps not avoided

- `temper/crates/temper-service/src/stock/configuring.txt`, with `model-catalogue-1/configuring-told`. That branch already conflicts with main in four places, measured with `git merge-file` at its merge base. All three of this plan's edits together leave it at four, so they add none. Whichever lands second takes the other's paragraphs and makes its edits in them.
- `temper/crates/temper-service/src/tests/stock.rs` and `stock_brief.rs`, with `model-catalogue-1/stock-seven` and others: counted baselines, regenerated on a catch-up.
- `temper/tools/gallery/src/tests.rs` and `temper/tests/client-browser/src/lib.rs`: one line each here.
- `temper/crates/temper-client-domain/src/pages/chats.rs` and `tests/pages/chats.rs`, with `long-words/composer-domain`: other fields and functions of the same type.
- `temper/crates/temper-client-view/src/pages/home.rs` and `tests/home.rs`, with `model-check/home-line`, already merged in its trunk: other functions of the same file.
- `temper/docs/design/client/README.md` and `README.status.md`, with several plans: small separate hunks.
