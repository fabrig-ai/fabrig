+++
id = "walk"
part = "walk"
kind = "docs"
needs = []
files = [
  "temper/docs/plan/ui-walk-3/report.md",
  "temper/docs/plan/ui-walk-3/findings.md",
  "temper/docs/plan/ui-walk-3/driver/walk.py",
  "temper/docs/plan/ui-walk-3/driver/cdp.py",
  "temper/docs/plan/ui-walk-3/driver/README.md",
  "temper/crates/temper-fake/src/backends.rs",
]
effort = "high"
minutes = 120
+++

# The seven steps again, with W3 and W6 walked through

## Goal

Replay the second walk's seven steps on this trunk, on fresh fake boxes, at 1440 and at 420, beside the mock, and write down what differs, against the mock and against the second walk. Walk W3 and W6 to their ends, as they now should reach. This task fixes nothing in the client: its product is the table and the findings.

Acceptance:

- `driver/walk.py` runs from this trunk at either width from one command, and `driver/README.md` says so.
- `report.md` has a row for each state, at both widths (`notes/walk.md`, The table), for this commit. A state left out is a failure of this task.
- **W3:** on its own fresh box, before any setting changes, type "Show drafts and goals for this project." into Configure temper; the proposal's card in the chat; then the project's Needs you shows it, the rail's count says 1, Home's card shows it; press Accept and its sheet's Accept; the count goes, the rail changes (Drafts and Goals appear), the chat's card says accepted. Rows at both widths for each of these, and `temper ctl inbox --json` kept. Compare with `walk-2/findings.md`, W3.
- **W6:** a landing by "Land a local hello status change.", on the fake ChatGPT sign-in's personal plan with no funding step, as the second walk had; Home, then the page closed and the box left running; at least 31 minutes by the clock, with the stock `forge.sitting_ms`; Home's brief shows its sentence, lede and byline. The 420 row reuses that live box (`--brief-from`), as before.
- **W6, unfunded:** a separate fresh box where the period can't pay the status run: Home's headline is the computed "The last change landed SPAN ago. Nothing needs you.", with no brief's word, no byline and no error. This box may shorten `forge.sitting_ms` in its configuration, as `a_plans_project_has_its_brief_on_a_return_past_the_sitting` in `temper/crates/temper/tests/fake_backends/walk/status.rs` does; a row says so.
- `findings.md` has W3's and W6's rows, each with its fate, and a row `N1`, `N2`, ... for each difference no decision explains and for anything that got worse (`notes/common.md`), each with a size and an owner. A `large` one has its reproduction.
- The screenshots and `index.html` are on the builder (`notes/walk.md`), and `report.md` is copied beside them. Every box and Chromium you started is stopped.

## Inputs

- `temper/docs/plan/ui-walk-3/inputs/request.md`, all of it.
- `temper/docs/plan/ui-walk-3/notes/common.md`, `notes/walk.md`, `notes/decisions.md`.
- `temper/docs/plan/ui-walk-3/driver/README.md`, and `walk-2/report.md`, `walk-2/findings.md`.
- `temper/docs/design/client/README.md`, 3.5, *Home on any project* and the page you're looking at; 3.6, *The sheet*.
- `temper/docs/design/domain/forge.md`, 5.6 and 5.7.
- `temper/docs/design/client/greenfield/README.md`; the mock's pages and `app.js` as you need them.

## Owns and may change

Owns, as `files` lists them. `driver/focus.py` and `driver/additions.py` are `additions`'; `walk.py` is yours, and `additions` imports from it as it stood at your base, so keep `Browser`, `Box` and `Walk`'s method names where you can.

May also change, for the walk's sake alone: `temper/crates/temper-fake/src/tests/**` and any test that lists `model_document`'s scripts, the lines a script you add needs; and a `temper-fake` limit that bounds scripts, if you reach it.

Options, each pre-approved, the first named being the default:

- **The unfunded box:** a second member on the project, so that the plan no longer serves its run (`temper/docs/design/domain/forge.md`, 5.7; `jig/docs/design/domain/accounts.md`, 4.2). If the brief still shows there, set the project's period down through ctl (`temper/docs/help/budget.md`). If neither reaches it, the state as `temper/tests/client/tests/suite/status.rs` makes it is the world's alone: the row is `unreachable`, the gallery shows it, and it's a finding that a person can't reach it to see.
- **W3's literal `against`:** a fresh box's newest revision. If the proposal is refused as stale, read how `settings` numbers a fresh box, fix the driver's order, and say so; if the cue's literal is wrong for a fresh box, fix the literal.
- **A walk that stops on a fault:** a state you can't get past is a finding with its reproduction. Reach what's beyond it on another fresh box or by `temper ctl`, say so on those rows, and go on. Don't fix it here, unless the fault is in your driver or the fake's scripts.
- **The driver's `ctl` fallback for settings:** still evidence of a failure, never a pass.

## Steps

1. Build and copy, by `driver/README.md`. Run one box far enough to see what has moved since the driver's last walk; follow it.
2. The driver at 1440, then at 420, with `--brief-from`. W3's and the unfunded box beside them.
3. The table, the findings, `index.html`; row by row against `walk-2/report.md`.
4. If you changed `backends.rs`: `heavy cargo nextest run -p temper-fake --lib`, and `heavy cargo nextest run --profile process-temper -p temper --test process -E 'test(a_plans_project_has_its_brief_on_a_return_past_the_sitting)'`, `cargo fmt`, `heavy cargo clippy -p temper-fake --all-targets -- -D warnings`.
5. `python3 -I -B tools/citations/citations.py --base origin/main`. Commit and push.

## Report

As `notes/common.md`, Your report, and: how many boxes, how long each walk took, the counts of each verdict against the second walk's, and which fake scripts you added.
