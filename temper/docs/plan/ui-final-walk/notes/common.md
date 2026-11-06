# ui-final-walk: the rules every task names

The request is `temper/docs/plan/ui-final-walk/inputs/request.md`: read all of it, it is short, and its *For every task of this plan* binds you and is not repeated here. `notes/decisions.md` beside this note holds what is already decided: a departure listed there is not a finding, and you don't reverse one. Where this note and the implementer's text differ, this note wins.

This plan writes no design. It compares the built client with the mock (`temper/docs/design/client/greenfield/`, its `README.md`) and fixes what is small.

## Departures from the implementer's text

- **Decide, don't ask.** A `Question:` is only for what truly blocks your task. A stored format, a protocol or a public interface may change freely, with no compatibility; say so in your report. A small edit just outside your files (a re-export, a `mod` line, a design line, a golden) is made and named in your report.
- **Design text:** a fix that makes a design line wrong edits that line, and a status entry it closes, in the same commit; name it in your report. Nothing else under `docs/design/`, but the generated prints.
- **The driver is committed, not thrown away:** the walk's CDP driver lives in `temper/docs/plan/ui-final-walk/driver/`, so that the second walk replays it on any builder. That directory leaves main with the plan and no test reads it. It holds no sign-in link, key or token: it reads the link from the box's output when it runs. Any other render script stays throwaway, untracked and gone before the push.
- **A box, Chromium and the driver run directly,** outside `heavy`: they build nothing. Every build and test still goes through `heavy`.
- **Every task ends with a commit, even with nothing to fix.** A task that passes with nothing to commit doesn't release the next one. Its commit may be only its lines in `findings.md`, with `state` filled in or "No findings for this task."

## The files this plan writes

All in `temper/docs/plan/ui-final-walk/`, each with one writer at a time, as the chain orders them:

- **`report.md`:** the page-by-page table (`notes/walk.md`, The table). `walk` writes it; `walk-again` replaces it.
- **`findings.md`:** one table, one row a finding. `walk` writes `W1`, `W2`, ...; `fix-look` and `fix-flow` fill in `state`; `walk-again` adds what it finds as `A1`, `A2`, ...
- **`real-checks.md`:** the two real checks, and their findings as `R1`, `R2`, ... `real-checks` writes it; `fix-flow` fills in `state`.

A row of `findings.md`:

```
| id | where (step, page, state, width) | seen | the mock or rule says | kind | size | owner | state |
```

- **kind:** `look`, `word`, `link`, `stale-read` or `other`.
- **size:** `small` is a look rule, a word, a missing link or a stale read, one fix in one or two files, with a test. `large` is anything that needs a plan's own design, or a decision the user hasn't made.
- **owner:** `fix-look` for what the sheets and the view alone hold: `temper.css`, `task-pages.css`, `temper-client-view`, a word, a link the view draws. `fix-flow` for all else that is small: the client's domain, the shell, the engine, the fake's scripts, the real checks. A finding that needs both is `fix-flow`'s. `fabrig/ui` for a `large` one, which also gets a section below the table: its reproduction, step by step, and what you saw.
- **state:** empty until a fix task fills it: the commit and the test that was red first, `not reproduced`, or `large, moved` with the reason.

## Artefacts on the builder

- **Under `~/src/fabrig/ui-final-walk/<your task>/`,** outside any worktree, never committed: screenshots, logs, box directories. Your report and `report.md` name the builder (`hostname`).
- **A box's directories are absolute, empty, private (0700),** under `~/src/fabrig/ui-final-walk/<your task>/boxes/<n>/`.
- **Copy what a walk runs** (`temper`, `fake-claude`, the bundle) to `~/src/fabrig/ui-final-walk/<your task>/build/` first, so that a later build in your slot doesn't move them under a running box.
- **Stop what you start:** each box by SIGTERM, waiting for its `Stopped`, and each Chromium. At your end remove your `boxes/` and `build/`: the screenshots, the driver's logs and `index.html` stay.
- **What the box prints is the fake's dummy, but the real boxes' links aren't:** see `real-checks`. A sign-in link never goes in a committed file, a report or a screenshot's name.

## A fix

- **Red first:** each fix has a test that fails before it, in the lowest tier that shows it; your report names the test and how it failed. A fix's test replaces the implementer's "break the rule once by hand".
- **Where:** a word or a link, `temper/crates/temper-client-view/src/tests/` beside its page's file (`home.rs`, `needs.rs`, `goal.rs`, ...). A stale read, `temper/crates/temper-client-domain/src/tests/`, or a system world where the fault is the engine's. A test in the `temper` package's `process` binary runs alone, under `--profile process-temper`: the default profile leaves it out, and a run that selected nothing proves nothing. A look rule, `temper/crates/temper-client-shell-web/tests/suite/`, in a test named for the rule that reads `temper.css` as text, and asserts the selector and its declaration: small and local, since no stylesheet test exists beyond the bundle's list. The screenshot pair is the rule's proof; the test is only what keeps it.
- **No new browser-tier test:** `browser-temper` stands at its CPU budget (about 85 s). Only where nothing lower can show a fault, add steps to an existing story in `temper/tests/client-browser/tests/browser_all/` and give the measurements in your report. Over budget is red.
- **Look rules** go under their area's `/* ==== ui: AREA ==== */` marker in `temper.css`, in the `--gf-*` tokens, copying the mock's selector in `mock.css` onto the client's own classes, removing what they replace where it stands.
- **After a fix to the view or the sheets:** the gallery's `names.txt` and `temper/tools/gallery/src/tests.rs` if its states change; the terminal's goldens, `SKEIN_CODEGEN_WRITE=1 heavy cargo nextest run -p temper-client-shell-terminal --lib`, reading the diff; the journeys' prints, `heavy cargo run -p temper-client-journeys`. A stale recording, `heavy cargo run -p temper-client-journeys -- again NAME`, as the request's *Recording flakes* says. Help pages and `docs/help/reference/*` that a change makes stale are yours.
- **Don't redesign,** and don't reverse a listed decision. A fix that grows past small stops, and its row becomes `large, moved`, with its reproduction.

## Your report

As the implementer's text says, and: the builder, the folder under `~/src/fabrig/ui-final-walk/`, and any finding left, with why.
