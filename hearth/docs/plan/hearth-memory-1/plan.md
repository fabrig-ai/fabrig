# hearth-memory-1: the plan

Round 1 of hearth's move to the dynamic memory model: it pays D002 and D003, adds the large render and holds the focused budget. D001's forms
are later rounds. The design this plan builds on is `hearth/docs/design/domain/render.md`, 8.4 and `hearth/docs/design/domain/testing.md`, 8.4
and 9.1, which this cut wrote.

## Parts and tasks so far

- **`sums`:** `sums` deletes the top of the sum. It comes first because every family's formula loses its last production reader there.
- **`large`:** `large-render` needs nothing.
- **`budget`:** `gate-table` needs nothing.

Still to come in later pushes: the families' ceilings and renames, `valid`, the budget's measurement, and closing D002 and D003.

## Overlaps

No other draft is changing these files.
