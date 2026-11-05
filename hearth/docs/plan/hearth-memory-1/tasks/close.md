+++
id = "close"
part = "close"
kind = "docs"
needs = ["focused-budget"]
files = ["hearth/docs/development/debt/D002-process-worst-case.md", "hearth/docs/development/debt/D003-memory-tests-on-formulas.md", "hearth/docs/development/debt/D001-room-checks-and-max-out.md", "hearth/docs/design/domain/render.md", "hearth/docs/design/domain/render.status.md", "hearth/docs/design/domain/testing.md", "hearth/docs/design/domain/testing.status.md", "hearth/docs/design/STATUS.md"]
effort = "medium"
minutes = 25
+++

# D002 and D003 are paid

## Goal

D002's and D003's **Paid when** hold: check that against the trunk, delete both files, and bring the status files up to the code. **This task
departs from the implementer's text:** it edits the status files named under `files`, because the code it closes was built by this plan.

Acceptance:

- Each **Paid when** is checked by its **Measure** and the report gives the figures. If one doesn't hold, stop and ask rather than delete it.
- `hearth/docs/development/debt/D002-process-worst-case.md` and `D003-memory-tests-on-formulas.md` are deleted.
- `python3 -I -B tools/citations/citations.py --all` reports 0 broken.
- `heavy cargo nextest run -p fabrig-boundary --test outward` passes.

## Inputs

- `hearth/docs/plan/hearth-memory-1/notes/contracts.md`, 4
- `hearth/docs/development/debt/D002-process-worst-case.md`
- `hearth/docs/development/debt/D003-memory-tests-on-formulas.md`
- `hearth/docs/development/debt/README.md`
- `DESIGN.md`, 2 and 4
- The merged reports' figures: `git log --grep large` on the trunk, for `large-render`'s commit.

## Owns and may change

- **Owns:** the files under `files`.
- **`render.md` and `testing.md`:** only the large render's figures in 8.4, and only if `large-render`'s commit gives others.
- **`render.status.md`:** the **Not yet** 8.4 line goes, as what it names is built.
- **`testing.status.md`:** the **Not yet** 8.4 line goes. In **Departs:** 9.1, replace the 35 to 36 CPU s with `focused-budget`'s figure and date, and say
  what's left over 30, if anything.
- **`STATUS.md`:** its summary lists "layer peak formulas" among the unsettled departures: take that out.
- **`D001-room-checks-and-max-out.md`:** its last lines say D002 is independent, and that `Queue::<..>::price(limits.queue)` in the process sum
  goes with it. Remove what's no longer so. Change nothing else there.
- **`hearth/docs/plan/hearth-docs/**`:** a landed plan's files. If they fail citations once D002 is gone, stop and ask.

## Steps

1. Run both **Measure** commands, and read each **Paid when** against the trunk.
2. Make the edits above, then delete the two files.
3. Run the citations check and `outward`.
4. Commit and push.

## Report

The first line, the two measures, the status lines changed, and the citations and `outward` results.
