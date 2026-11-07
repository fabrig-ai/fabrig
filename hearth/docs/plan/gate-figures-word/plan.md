# gate-figures-word: every tree's gate figures carry the user's word

The request: `inputs/request.md`. The user confirmed on 11-07 that a tree's gate figures, 30 for focused and 120 for fuzzy, are CPU seconds per tree. `tools/tiers/budgets.toml` and hearth's testing design still said the question "stays open". This plan records what was decided, in those two places alone. No figure, entry or test changes.

## Parts and tasks

| Part | Task | Needs | What it makes run | Effort, minutes |
|---|---|---|---|---:|
| `comment` | `tasks/comment.md` | | `budgets.toml` lines 4, 6 and 7 (Hearth's fuzzy, Skein's, Jig's) take Smith's form; `test_cpu.py` passes and the real file validates | medium, 10 |

**Critical path:** `comment`, about 10 minutes. No build.

## Design

Written by this cut, on its branch, before the spec: `hearth/docs/design/domain/testing.md`.

- **1, "In one page":** the fuzzy suite takes 120 CPU seconds. The parenthesis on the 10-22 doubling stays.
- **9.1:** the fuzzy suite takes at most 120 CPU seconds, by the user's word of 11-07, kept and measured as the focused suite's. The idle-machine wall-time clause is gone. nextest's global timeouts stay as the suites' wall backstop (`skein/docs/foundation/testing-strategy.md`, 8.1).
- **11, "The suites' figures":** keeps only what is still open, namely what the workspace's global timeouts hold and whether `hearth-render-world` stays on every change. The item stays, since something is left.

`testing.status.md` is unchanged: it doesn't repeat the open question. Its **Departs** on 9.1 is about there being no hearth-only global timeout, and it still holds. `citations.py --base origin/main` is clean on the cut's branch (0 broken).

## What was decided

- **The cut writes the design; the task writes `budgets.toml`.** The request asks for one direct task with two checks. A cut writes the design its specs cite, and an implementer's text keeps `docs/design/` out of its hands. So the doc edit is the cut's commit, checked here with the citation check, and the one task changes the comment lines and runs `test_cpu.py`. The plan is still one task.
- **One part, one task.** The request says "no parts", but fab needs every task in a part and every part in a slice, so the plan has one of each.
- **Effort `medium`.** The request asks for low effort. fab takes only `high` or `medium`, so the task uses `medium`.
- **9.1 keeps the doubling's parenthesis,** as section 1 does. The request names it for section 1 and asks for nothing more in 9.1 than the CPU figure, the user's word, how it's kept and measured, and the timeout as backstop.
- **The check is `test_cpu.py`,** as the request says, plus `cpu.py`'s `budgets()` run on the real file. `test_cpu.py` reads only fixture copies it writes itself, so the one-liner is what shows the real file still parses.
- **Lines left as they are:** line 3 (Hearth's focused, the 11-04 ruling) as the request says, and line 2 (live pairs and other suites without entries), which doesn't say "stays open".

## Left behind, by the request's limits

- `tools/tiers/design.md`, 7.3 (around line 380) and 8 (around lines 623 to 630), and `tools/tiers/design.status.md`, still say Skein's, Smith's and Jig's CPU figures are open. The request forbids any other design change here. A later plan closes them.
- `hearth/docs/development/hearth-fuzzy-entry.md` reports, as of its own plan, that the 11 question stayed open. It is a development report of what was true then, not design, so it stays.
- The strategy's 8.1 ("until a tier's CPU budget is set") stays, as the request says.

## Overlaps

None. No draft beside this one names `tools/tiers/budgets.toml` or `hearth/docs/design/domain/testing.md`. `look-round-16` and `per-wake` touch skein's UI, temper's client and jig's crates and design.
