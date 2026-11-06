+++
id = "walk-again"
part = "walk-again"
kind = "docs"
needs = ["fix-flow"]
files = [
  "temper/docs/plan/ui-final-walk/report.md",
  "temper/docs/plan/ui-final-walk/findings.md",
  "temper/docs/plan/ui-final-walk/real-checks.md",
  "temper/docs/plan/ui-final-walk/driver/**",
]
effort = "high"
minutes = 120
+++

# The walk again, on the fixed tip: the plan's result

## Goal

Run the driver again from your base, the trunk with every fix, at 1440 and 420, on fresh boxes. Replace `report.md` with the result. This is the page the user reads.

Acceptance:

- `report.md` is the request's *The result*: the table, at both widths, every state, for this commit, as `notes/walk.md` says. A short opening above the table: how far the client matches the mock, in plain words, and what's still different and why.
- **Every earlier finding is settled:** each `W` and `R` row whose `state` claims a fix has been seen fixed in this walk, or its `state` says it wasn't and it has a new row.
- **The list for `fabrig/ui`:** each `large` finding, as its own line under a heading of that name in `report.md`, with its reproduction. Don't plan them.
- **New small findings:** fix one here, with a test red first, if it's a word, a link or a look rule that fits in a commit, and walk that state again. Otherwise it's an `A` row, named on that list.
- `real-checks.md`'s verdicts are in `report.md`, in a section of their own. If `real-checks.md` says this task must run the fetch again, run it as `real-checks`' spec does.
- On the builder: `~/src/fabrig/ui-final-walk/index.html`, the contact page, linking each state's pair from `~/src/fabrig/ui-final-walk/walk-again/`, with `report.md` copied beside it. `report.md` names the builder and, if they differ, the builders of `walk` and `real-checks`, whose folders it links by name.

## Inputs

- `temper/docs/plan/ui-final-walk/inputs/request.md`, *The walk*, *The result*.
- `temper/docs/plan/ui-final-walk/notes/common.md`, `notes/walk.md`, `notes/decisions.md`.
- `temper/docs/plan/ui-final-walk/driver/README.md`; `findings.md`, `real-checks.md` and the `report.md` you're replacing.
- `git log` of `fix-look` and `fix-flow`'s commits: what changed.

## Owns and may change

Owns, as `files` lists them.

May also change: for a new small finding only, what `fix-look` may (`temper/docs/plan/ui-final-walk/tasks/fix-look.md`, Owns and may change); and `temper/crates/temper-fake/src/backends.rs`, where a fix moved what a script needs.

Options, each pre-approved:

- **The driver stopped on a fixed page:** a name that a fix changed is the driver's to follow; change it, and say so.
- **Your builder isn't `walk`'s:** nothing here needs `walk`'s folder; this walk makes its own pairs.
- **More small findings than fit:** the first walk had its chance at fixing, so what doesn't fit goes on the list as `small`, and you end.

## Steps

1. Build and copy; run the driver at 1440, then at 420.
2. Compare, row by row, with the first walk's report. Anything that got worse is a finding.
3. Fixes that fit, if any, then walk those states again.
4. `report.md`, `findings.md`, `index.html`, on a clean look at each.
5. If you changed code: **Check,** the implementer's text. `python3 -I -B tools/citations/citations.py --base origin/main`. Commit and push.

## Report

As `notes/common.md`, Your report, and: the counts of each verdict against the first walk's, the list's lines, and the builder and folder of the contact page.
