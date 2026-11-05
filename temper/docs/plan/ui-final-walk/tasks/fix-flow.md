+++
id = "fix-flow"
part = "fix-flow"
kind = "feature"
needs = ["fix-look", "real-checks"]
files = [
  "temper/docs/plan/ui-final-walk/findings.md",
  "temper/docs/plan/ui-final-walk/real-checks.md",
  "temper/crates/temper-client-domain/src/**",
  "temper/crates/temper-client-shell-web/src/**",
  "temper/crates/temper-fake/src/**",
]
effort = "high"
minutes = 90
+++

# The small findings that the view alone can't fix, and the real checks'

## Goal

Fix each row of `findings.md` whose owner is `fix-flow`, and each small row of `real-checks.md`, each with a test red first, and fill in its `state`. The rows came after this spec: read them.

Acceptance:

- Every such row has a `state`: its commit and the test that was red first; `not reproduced`; or `large, moved`, with its reproduction and its owner changed to `fabrig/ui`.
- Each fix is its own commit, and says the finding's id.
- The packages you changed pass, and their readers' (below); what's generated is regenerated.
- A fix to a real check's fault has its test red in a world that needs no download or real service. Its real proof is `walk-again`'s, where the note below asks for one.
- **With no such row:** commit "No findings for this task." in `findings.md`, and report `Done:`. That commit releases the next task.

## Inputs

- `temper/docs/plan/ui-final-walk/findings.md`, `real-checks.md`, `report.md`; the driver, for a reproduction.
- `temper/docs/plan/ui-final-walk/notes/common.md` (*A fix*) and `notes/decisions.md`.
- `temper/docs/design/client/README.md`, the section of the page in question; `temper/docs/design/domain/testing.md`, 2, for which world fits an engine's fault; `temper/docs/design/protocol/setup.md`, 6.6, for a fetch's.

## Owns and may change

Owns, as `files` lists them.

May also change, for a row's fix alone: the one or two files of `temper/crates/` it names (the engine's `temper-engine-*`, `temper-service`, `temper-forge-setup`, `temper-init`, `temper`) and of `jig/` or `skein/` where the fault is theirs, fixed in the lowest tier that shows it (`AGENTS.md`, 5); `temper/crates/temper-client-view/src/**` where a domain's change forces it; generated files, as `notes/common.md` says; a design line a fix makes wrong.

Options, each pre-approved:

- **A stored format, a protocol, a public interface** may change, with no compatibility (the request); a wire change makes journeys' recordings stale, and you re-record them here, by `again`.
- **A fix that would need more than one or two files, or a decision:** `large, moved`.
- **A change of what a shared item means:** run the tests of every package that reads it, and name them in your report.
- **A real check's fault in Claude Code's fetch that only the real download can prove:** fix it, and say in `real-checks.md` that `walk-again` must run the fetch again. `walk-again` reads that line.

## Steps

1. Read the rows.
2. Each: the test, seen red; the fix; the test green; its row's `state`; commit.
3. The generated files, once at the end.
4. **Check,** the implementer's text: the packages you changed and their direct dependents. `python3 -I -B tools/citations/citations.py --base origin/main`. Push.

## Report

As `notes/common.md`, Your report, and: a line per row, its state.
