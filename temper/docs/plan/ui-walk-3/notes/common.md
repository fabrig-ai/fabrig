# ui-walk-3: the rules every task names

The request is `temper/docs/plan/ui-walk-3/inputs/request.md`: read all of it, it is short, and its *For every task of this plan* binds you and is not repeated here. `notes/decisions.md` beside this note holds what is already decided: a departure listed there is not a finding, and you don't reverse one. Where this note and the implementer's text differ, this note wins.

This plan writes no design. It compares the built client with the mock (`temper/docs/design/client/greenfield/`, its `README.md`) and with the second walk (`temper/docs/plan/ui-walk-3/walk-2/`), and fixes what is small.

## Departures from the implementer's text

- **Decide, don't ask.** A `Question:` is only for what truly blocks your task. A stored format, a protocol or a public interface may change freely, with no compatibility; say so in your report. A small edit just outside your files (a re-export, a `mod` line, a design line, a golden) is made and named in your report.
- **Design text:** a fix that makes a design line wrong edits that line, and a status entry it closes, in the same commit; name it in your report. Nothing else under `docs/design/`, but the generated prints.
- **The driver is committed, not thrown away:** `temper/docs/plan/ui-walk-3/driver/` lets a later task replay it on any builder. That directory leaves main with the plan, and no test reads it. It holds no sign-in link, key or token: it reads the link from the box's output when it runs. Any other render script stays throwaway, untracked and gone before the push.
- **A box, Chromium and the driver run directly,** outside `heavy`: they build nothing. Every build and test still goes through `heavy`.
- **Every task ends with a commit, even with nothing to fix.** A task that passes with nothing to commit doesn't release the next one. Its commit may be only its lines in its file, with `state` filled in or "No findings for this task."

## The files this plan writes

All in `temper/docs/plan/ui-walk-3/`, each with one writer at a time, as the tasks' `needs` order them:

- **`report.md`:** the page-by-page table (`notes/walk.md`, The table). `walk` writes it; `walk-again` makes it the plan's result.
- **`findings.md`:** one table, one row a finding. `walk` writes it: the second walk's W3 and W6 again, and what it finds new as `N1`, `N2`, ... `fix-look` and `fix-flow` fill in `state`; `walk-again` adds what it finds as `B1`, `B2`, ...
- **`additions.md`:** `additions` writes it: the same table for the states the second walk never held, then the same findings table, its new rows `M1`, `M2`, ..., and the fate of A2, A3 and A4. `fix-look` and `fix-flow` fill in `state`; `walk-again` folds it into the two above and removes it.
- **`real-check.md`:** `real-login` writes it: R1's steps and verdict, and any fault as `R2`, `R3`, ... `fix-flow` fills in `state`; `walk-again` folds its verdict into `report.md`, and removes it.

The second walk's files, in `walk-2/`, are read only. So are `findings.md` rows' ids from it: W3, W6, A2, A3, A4 and R1 keep their ids here, and an id is never reused.

A row of a findings table:

```
| id | where (step, page, state, width) | seen | the mock or rule says | kind | size | owner | state |
```

- **kind:** `look`, `word`, `link`, `stale-read` or `other`.
- **size:** `small` is a look rule, a word, a missing link or a stale read, one fix in one or two files, with a test. `large` is anything that needs a plan's own design, or a decision the user hasn't made.
- **owner:** `fix-look` for what the sheets and the view alone hold: `temper.css`, `task-pages.css`, `temper-client-view`, a word, a link the view draws. `fix-flow` for all else that is small: the client's domain, the shell, the engine, the fake's scripts, the real check. A finding that needs both is `fix-flow`'s. `fabrig/ui` for a `large` one, which also gets a section below the table: its reproduction, step by step, and what you saw.
- **state:** empty until a fix task fills it: the commit and the test that was red first, `not reproduced`, or `large, moved` with the reason. A second-walk row that this walk saw fixed says `fixed, seen` and its evidence.

**Named, not a finding** (the request says so). What `chat-reading` hasn't landed (`temper/docs/design/draft/chat-reading.status.md`; plans `chat-reading-b` and `chat-reading-c` are still running). The behaviour `sign-in-new-tab` and `model-check` still owe (`temper/docs/design/draft/sign-in-new-tab.status.md`, `model-check.status.md`): a new tab, coming back, the fixed page's words, the trial's run. Each goes on a line of its own under the table, as `owed`, naming the draft and section. What looks wrong in what has landed is a finding as usual.

## Artefacts on the builder

- **Under `~/src/fabrig/ui-walk-3/<your task>/`,** outside any worktree, never committed: screenshots, logs, box directories. Your report and `report.md` name the builder (`hostname`). Nothing of these is copied to the repository but the report.
- **A box's directories are absolute, empty, private (0700),** under `~/src/fabrig/ui-walk-3/<your task>/boxes/<n>/`.
- **Copy what a walk runs** (`temper`, `fake-claude`, the bundle) to `~/src/fabrig/ui-walk-3/<your task>/build/` first, so that a later build in your slot doesn't move them under a running box.
- **Stop what you start:** each box by SIGTERM, waiting for its `Stopped`, and each Chromium. At your end remove your `boxes/` and `build/`: the screenshots, the driver's logs and `index.html` stay.
- **A sign-in link never goes in a committed file, a report or a screenshot's name.** A real box's links are real.

## A fix

- **Red first:** each fix has a test that fails before it, in the lowest tier that shows it; your report names the test and how it failed. A fix's test replaces the implementer's "break the rule once by hand".
- **Where:** a word or a link, `temper/crates/temper-client-view/src/tests/` beside its page's file (`home.rs`, `needs.rs`, `goal.rs`, ...). A stale read, `temper/crates/temper-client-domain/src/tests/`, or a system world where the fault is the engine's. A test in the `temper` package's `process` binary runs alone, under `--profile process-temper`: the default profile leaves it out, and a run that selected nothing proves nothing. A look rule, `temper/crates/temper-client-shell-web/tests/suite/`, in a test named for the rule that reads `temper.css` as text (`switcher.rs` holds the form) and asserts the selector and its declaration. The screenshot pair is the rule's proof; the test is only what keeps it.
- **No new browser-tier test:** `browser-temper` stands near its CPU budget. Only where nothing lower can show a fault, add steps to an existing story in `temper/tests/client-browser/tests/browser_all/` and give the measurements in your report. Over budget is red.
- **Look rules** go under their area's `/* ==== ui: AREA ==== */` marker in `temper.css`, in the `--gf-*` tokens, copying the mock's selector in `mock.css` onto the client's own classes, removing what they replace where it stands.
- **After a fix to the view or the sheets:** the gallery's `names.txt` and `temper/tools/gallery/src/tests.rs` if its states change; the terminal's goldens, `SKEIN_CODEGEN_WRITE=1 heavy cargo nextest run -p temper-client-shell-terminal --lib`, reading the diff; the journeys' prints, `heavy cargo run -p temper-client-journeys`. A stale recording, `heavy cargo run -p temper-client-journeys -- again NAME`, as the request's *Recording flakes* says. Help pages and `docs/help/reference/*` that a change makes stale are yours.
- **Don't redesign,** and don't reverse a listed decision. A fix that grows past small stops, and its row becomes `large, moved`, with its reproduction.
- **Stay off what other plans are changing, where a fix lets you:** `chat-reading-c` and `chat-reading-b` are in `temper-client-view/src/pages/{chat,chat_line,agent}.rs`, `temper-client-view/src/limits.rs` and `temper.css`; `sign-in-new-tab` is in `temper-client-domain/src/pages/accounts/request.rs`, the client wire and the service's routes. Add to them; move and reformat nothing you don't change. A finding whose fix belongs to such a plan's owed work is `owed`, not a fix here.

## Your report

As the implementer's text says, and: the builder, the folder under `~/src/fabrig/ui-walk-3/`, and any finding left, with why.
