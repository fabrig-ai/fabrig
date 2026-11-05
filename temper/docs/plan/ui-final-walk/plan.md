# ui-final-walk: the plan

The whole client walked as a person would, against the greenfield mock, as the request says: `temper/docs/plan/ui-final-walk/inputs/request.md`. No design is written: this plan compares what's built with what's designed, and fixes what's small. Decisions and rulings are carried in `notes/decisions.md`; the walk's method in `notes/walk.md`; the rules in `notes/common.md`.

## The tasks

| # | Task | Needs | Effort, minutes | What it gives |
|---|---|---|---|---|
| 1 | `walk` | | high, 150 | the driver, `report.md`, `findings.md`, the pairs |
| 2 | `real-checks` | | high, 75 | `real-checks.md`: the real fetch, a real Sign out |
| 3 | `fix-look` | 1 | high, 90 | the small findings in the sheets and the view, each red first |
| 4 | `fix-flow` | 2, 3 | high, 90 | the small findings elsewhere, and the real checks' |
| 5 | `walk-again` | 4 | high, 120 | the final `report.md`, the list for `fabrig/ui`, the contact page |

## The critical path

`walk` (150) → `fix-look` (90) → `fix-flow` (90) → `walk-again` (120): 450 implementer-minutes, then the trunk's end check. `real-checks` (75) sits beside `walk`, and `fix-flow` waits for both.

## What the cut decided

- **The fix tasks' rows don't exist yet.** A pushed spec never changes, so `fix-look` and `fix-flow` read `findings.md` and `real-checks.md` from the trunk, by owner. `walk` assigns the owners by the rule in `notes/common.md`.
- **Two fix tasks, chained,** since they'd meet in `temper.css`, the gallery, the goldens and the prints: the sheets and the view first, whatever's deeper second, where a row can be handed on.
- **The real checks are their own task,** beside the walk: they need a different build (the embedded bundle), a clean `PATH` and `HOME`, and they block nothing of the walk. Their faults go to `fix-flow`. Their departure from `AGENTS.md`, 3 is only the download the request names.
- **Every task ends with a commit,** with nothing to fix too: a task that passes with nothing to commit doesn't release the next (`notes/common.md`).
- **The driver is committed in this directory,** not thrown away, so that `walk-again` replays it on any builder. It leaves main with the plan, and neither the outward check nor any test reads it.
- **The fake's limits are walked around, not hidden:** its scripts' literal note scopes hold on a box's first chat and first goal, its `change` names `work/widget`, and Home's brief needs 30 minutes by the clock. `notes/walk.md`, The fake model, gives each a default: several fresh boxes, a script for `work/hello`, a real wait. Each row names its box or way, and what the fake can't reach is shown by ctl or the gallery.
- **No new browser-tier test:** that tier stands at its CPU budget; budgets are the user's. Fixes are red in lower tiers.
- **A look rule's test** is a test in `temper-client-shell-web/tests/suite/` reading `temper.css`, as no stylesheet test exists beyond the bundle's list. It's new, and small; the pair is the rule's proof.
- **Stale words in the decisions:** Away and Phone are listed as not started, and have landed. `notes/decisions.md` says so at its head, and the rulings win.

Nothing here changes a stored format unless a finding needs it. A fix that does says so in its report, and a wire change re-records its journeys in that task.

## Overlaps with the drafts beside this one

- **`walk` and `real-checks`** touch only this directory, and `walk` perhaps `temper-fake/src/backends.rs`: nothing another draft is changing, save the wide rename below.
- **`debt-24/temper-kit-names`** lists every `temper/**/*.rs` and `Cargo.toml`. It can't be avoided by `fix-look` and `fix-flow`, or by anyone in temper, and it's a mechanical rename of test kits, so a clash is small.
- **`note-parts`' merged work** has touched `temper-engine-smith` and `temper-service`'s stock files: `fix-flow` reaches those only if a finding does.
- **`configuring-walks`, `reference-walks`, `v0.1.0-ladder`** are queued with no tasks yet listed: unknown.
