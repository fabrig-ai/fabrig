# debt-26: the plan

From `inputs/request.md`: pay debt items, twenty-sixth round, in four independent parts. The trunk is `debt-26/integration`. No temper client view, no `temper.css`, no `jig-client-view`: fabrig/ui has priority.

## Parts and tasks

| # | Task | Request's part | Needs | Effort, minutes | What it lands |
|---|---|---|---|---|---|
| 1 | `records` | 1 | | medium, 10 | D004 `kept`; the plaintext shim's Departs entry says the user left it |
| 2 | `forgejo-kinds` | 1 | | high, 70 | the rule in `forge.status.md`; `_ =>` under a named expect; the five wrappers and their `Option` parameters gone |
| 3 | `world-kits-item` | 2 | | medium, 20 | jig's D044; `testing.md`'s "in any tree" says the gate's reach |
| 4 | `forge-fixtures` | 2 | 5, 6 | high, 120 | skein's D050 opened and paid: a seam-gated fixtures module both read, and the twins cut |
| 5 | `seam-shapes` | 3, finding 3 | 7 | high, 50 | `mod own` gone, one attribute, unread names gone, a gate on `seam`'s one world |
| 6 | `hand-leftovers` | 3, finding 4 | | medium, 40 | `hand` split by dialect; `ok_dated`, the double alias and `renewed_*`'s twin gone |
| 7 | `forge-until` | 4, findings 3 and 6 | | high, 45 | the `until` parameter undone; `connect`'s `Closed` arm; the status says what's true |
| 8 | `plan-copies` | 3, finding 4 | | medium, 20 | `commits.rs`'s copies; sign-in's `Option` pair |
| 9 | `debug-rows` | 4, finding 1 | | high, 40 | the credential rows go red on a printed raw-bytes field |
| 10 | `jig-spot` | 4, findings 2, 4 and 6 | | medium, 40 | the `STOP_HOLD` and `STOPPED_CHAT` sweeps; `settled_answer`; D030, D034 and D035's words |
| 11 | `hearth-spot` | 4, finding 5 | | medium, 15 | `keeping`, `git` and the `Work` alias gone |

## Critical path

`forge-until`, `seam-shapes`, `forge-fixtures`: ~215 implementer-minutes, then the trunk's end check. Everything else runs beside it.

## What the cut settled

Each finding was read against the trunk first, as the request asks.

- **The plaintext shim's Departs entry already existed** in `skein/docs/design/link.status.md`; it said the decision was the user's still to make. `records` records that they left it.
- **D004 goes `kept`:** what's left aligns the two roots or waits on the same decision, so the user's deferral is all that's left.
- **`reading.rs`'s `Some(_) | None`** follows guards on two optional times and picks no kind: exempted, not changed.
- **`testing.md`'s sentence** is true of its words: the gate does read every tree's edges onto `skein/tests/`. What misleads is that it reads as the whole rule, with about 93 edges onto smith's and temper's `tests/` it never reads. The correction says so.
- **Finding 1 of the debt-25 review, `Component`:** the review's `Locked` path doesn't bite. `lock_candidate` holds `Secret`s and scope names, and `Upload`'s own `Debug` prints a length. `Web.document` is the raw-bytes field, so the row reaches it, and costs more than the review's +10: the spec's estimate is +35.
- **The fixtures' size:** the review reckons about -180. The cut counts eleven items, 59 lines, identical between `src/tests/fixtures.rs` and `hand`. The kit's near-copies of `limits`, `shared` and `settings` can't join: temper's and other worlds read the kit, and only the machine world may turn `seam` on. So `forge-fixtures` is judged by "no item twice", and the kit's copy is named as forced.
- **The status entry on debt-25's arithmetic** in `forge-connection.status.md` goes with `forge-until`: a plan's arithmetic is no gap in the code, and that undo makes its numbers false.

## Skipped, and why

From the debt-24 review's finding 4, both checked against the code:

- **Dialect `admitted.rs`:** the copy is forced. The crate's tests see their own test build's `Limits`, the dialect world sees the shipping crate's, and no `#[path]` may cross a package. Paying it needs a second seam on `skein-forge`, a shipping crate, and a wider gate, for 55 lines of test data the two sides keep on purpose as their own numbers: it would cost more than it pays.
- **Echo's `ClientHost`:** the copy is forced too. The test sits in the shell's package so that Cargo supplies its binary, `skein-echo-client` is held to the step subset and can't `impl Host`, and a new kit's manifest alone is longer than the 28 lines. `Delegate` on `Proc` would save lines, but only by adding a second `ClientHost`.

## Overlaps with drafts beside this one

- `hearth-spot` works in `hearth-protocol-builder` beside `hearth-memory-5/machine`, which changes `src/client.rs` of that crate: no file is shared.
- Inside this plan, `plan-copies` and `forgejo-kinds` both may touch `plans/commits.rs`, in different functions.
- No other file named by a draft beside this one is touched here.

## After the plan

Net lines: parts 3 and 4 fall by the specs' estimates. `debug-rows` is the one that grows, by about 35 lines, for a rule the request asks to bite; `forge-fixtures` carries the largest fall.
