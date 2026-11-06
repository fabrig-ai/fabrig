+++
id = "fix-flow"
part = "fix-flow"
kind = "feature"
needs = ["fix-look"]
files = [
  "temper/docs/plan/ui-walk-3/findings.md",
  "temper/docs/plan/ui-walk-3/additions.md",
  "temper/docs/plan/ui-walk-3/real-check.md",
  "temper/crates/temper-client-domain/src/**",
  "temper/crates/temper-client-shell-web/src/**",
  "temper/crates/temper-fake/src/**",
]
effort = "high"
minutes = 75
+++

# This walk's small findings that the view alone can't fix, and the real check's

## Goal

Fix each row of `findings.md`, `additions.md` and `real-check.md` whose owner is `fix-flow`, each with a test red first, and fill in its `state`. The rows came after this spec: read them.

Acceptance:

- Every such row has a `state`: its commit and the test that was red first; `not reproduced`; or `large, moved`, with its reproduction and its owner changed to `fabrig/ui`.
- Each fix is its own commit, and says the finding's id.
- The packages you changed pass, and their readers' (below); what's generated is regenerated.
- A fix to the real check's fault has its test red in a world that needs no download or real service. Its real proof is `walk-again`'s.
- **Each row's `state` names the states its fix touched,** by page, state and width, so that `walk-again`, which reads the rows and not your report, walks those and no more.
- **With no such row:** commit "No findings for this task." in `findings.md`, and report `Done:`. That commit releases the next task.

## Inputs

- `temper/docs/plan/ui-walk-3/findings.md`, `additions.md`, `real-check.md`, `report.md`; the driver, for a reproduction.
- `temper/docs/plan/ui-walk-3/notes/common.md` (*A fix*) and `notes/decisions.md`.
- `temper/docs/design/client/README.md`, the section of the page in question; `temper/docs/design/domain/testing.md`, 2, for which world fits an engine's fault.

## Owns and may change

Owns, as `files` lists them.

May also change, for a fix's fallout alone: what `fix-look` may (`temper/docs/plan/ui-walk-3/tasks/fix-look.md`, Owns and may change); a design line a fix makes wrong; `temper-fake`'s tests and a limit bounding scripts. **An engine or service fault:** the smallest fix in `temper/crates/temper-engine-*` or `temper-service`, with its world's test red first, and name the files in your report; if it needs more than a line or two in one of them, its row is `large, moved`.

**A change of meaning runs its readers' tests:** a fix that changes what a shared item means (a value it returns, an order it keeps) also runs the tests of every package that reads it. A client-domain change: `temper-client-view`, `temper-client-shell-web`, `temper-client-shell-terminal`, and the world `temper/tests/client`. A `temper-fake` script: its process walks, each alone, under `--profile process-temper`.

Options, each pre-approved:

- **A row whose cause proves not to be where its owner says:** fix it where it is, if it's small. If it isn't, `large, moved`.
- **A row that can't be reproduced outside a box:** reproduce it on a fresh fake box with a throwaway script, then find the lowest tier that shows it. If none does, a browser story's steps (`notes/common.md`, *No new browser-tier test*), with its measurements.
- **A file `sign-in-new-tab` is changing:** `temper-client-domain/src/pages/accounts/request.rs` and the client wire. A fix there is lines added; a fix that overlaps that plan's owed behaviour is `owed` (`notes/common.md`).

## Steps

1. Read the rows; start with what loses a person's work.
2. Each: the test, seen red; the fix; the test green; its row's `state`; commit.
3. The generated files, once at the end.
4. **Check,** the implementer's text, for each package you changed and the readers above. `python3 -I -B tools/citations/citations.py --base origin/main`. Push.

## Report

As `notes/common.md`, Your report, and: a line per row, its state.
