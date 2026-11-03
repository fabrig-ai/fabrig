+++
id = "status"
part = "status"
kind = "docs"
needs = ["ui-web-surface", "bundle-worker", "playground-crate", "ui-web-log", "ui-web-worker-page", "ui-web-worker-web", "playground-slices", "playground-visitor", "playground-memory", "playground-replay", "playground-worker"]
files = ["skein/docs/design/playground.status.md", "skein/docs/design/ui-web.status.md", "skein/docs/design/STATUS.md"]
effort = "medium"
minutes = 25
+++

# skein: the playground kit's status, once built

## Goal

Once every task of this plan has merged, the status files say where skein's code still lags
`skein/docs/design/playground.md` and `skein/docs/design/ui-web.md`, 18, and nothing more
(`DESIGN.md`, 2). This task departs from the implementer's text, which keeps
`docs/design/` out of reach: its three status files are its own.

Acceptance:

- Read the trunk's code for each of playground.md's sections 2 to 8 and ui-web.md's 18, and
  each merged task's report where fab keeps it beside the plan, and write only the gaps that
  stand, dated today (`date -u`), in `DESIGN.md`, 2's form:
  - `skein/docs/design/playground.status.md`: a **Not yet** entry per section that still lags
    (likely only 8's "For the portable target": the gate's wasm32 clippy list is fab's, outside
    this repository, so write it as "8: `skein-playground` in the gate's wasm32 clippy
    (fab's gate list)" unless the gate already runs it), a **Departs** entry for what was
    built otherwise with its reason (from the reports), or "No gap." when none stands;
  - `skein/docs/design/ui-web.status.md`: 18's entry removed; "No gap." if nothing else
    stands;
  - `skein/docs/design/STATUS.md`: its summary paragraph's sentence on `skein-playground`
    rewritten to what stands (or removed, and `playground.md` and `ui-web.md` named among those
    with no gap, in that sentence's order), nothing else of it changed.
- `python3 -I -B tools/citations/citations.py --base origin/main` reports nothing broken.

## Inputs

- `skein/docs/plan/playground-skein/notes/common.md`; `DESIGN.md`, 2 and 4.
- `skein/docs/design/playground.md`, 2 to 8; `skein/docs/design/ui-web.md`, 18; their status
  files and `STATUS.md` as they stand.

## Owns and may change

- Owns the three files in the header; nothing else. A gap found that a task should have
  closed is written as **Not yet** and listed in the report, never fixed here.

## Steps

1. Read the code and the reports; write the three files.
2. The citations check.
3. Commit (why: the status says what of the playground kit still lags); push your branch.

## Report

As the implementer's text says, and also: each status entry written, and why.

## Answers

2026-11-03. The six reports said to be missing (bundle-worker, playground-crate,
ui-web-worker-web, playground-visitor, playground-memory, playground-worker): where are they?
They are in the run directory, each under its run's name, `<task>-<n>/report.md`, not under
`<task>/` (which holds only the foreman's). Read the last run's of each:
`/home/free/src/fabrig/fab/runs/playground-skein/` + `bundle-worker-2/report.md`,
`playground-crate-1/report.md`, `ui-web-worker-web-1/report.md`,
`playground-visitor-2/report.md`, `playground-memory-1/report.md`,
`playground-worker-1/report.md` (an earlier run of the same task, `-1` where `-2` exists, asked a
question its later run settled). The requirement stands. Check each for a departure from
playground.md or ui-web.md, 18 (bundle-worker-2 reports none); write a **Departs** entry only
for one built otherwise than the design says, with its reason, and keep the edits already pushed
otherwise.
