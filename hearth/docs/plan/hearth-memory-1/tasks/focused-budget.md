+++
id = "focused-budget"
part = "budget"
kind = "feature"
needs = ["gate-table", "large-render", "valid"]
files = ["hearth/tests/tree/tests/suite/frames.rs", "hearth/tests/builder/**"]
effort = "high"
minutes = 60
+++

# The focused suite at 30 CPU s

## Goal

hearth's focused suite measured 35 to 36 CPU s on 11-04. The user's budget is 30. The fixed scenarios should bring most of that: measure it as the
gate does, and repair what's left.

Acceptance:

- A measurement of hearth's focused suite, by `tools/tiers/cpu.py` with `NEXTEST_DOUBLE_SPAWN=0`, is in the report: the total, and the 15 slowest
  tests. Fuzzy and the slow tiers don't count.
- At 30 CPU s or under, or the report says what's left and by how much. The budget stays at 30 either way.
- No assertion or case goes, and no test moves to another tier.

## Inputs

- `hearth/docs/plan/hearth-memory-1/notes/common.md`
- `hearth/docs/plan/hearth-memory-1/inputs/request.md`, 4
- `skein/docs/foundation/testing-strategy.md`, 8.1
- `tools/tiers/design.md`, 7, 10.1 and 10.2

## Owns and may change

- **Owns:** the files under `files`, and the test of any hearth package that the measurement shows is slow, its fixtures included.
- **Measuring:** a `default` run over `package(/^hearth(-|$)/)`. `cpu.py` is a library for callers (`tools/tiers/design.md`, 10.2), so a short
  driver is throwaway code, untracked and gone before you push. Measure twice on trunk's tip, take the lower figure, and name the machine.
- **Options, each pre-approved, in this order:**
  1. At 30 or under, change nothing, and report it.
  2. Otherwise find the cause. `frames::no_cut_of_a_generated_frame_reads_as_a_record` (2.5 s) and the builder's domain tests (about 0.9 s each)
     are where the request looks first. A cost in the test or its fixture, you fix. A cost in hearth's own code that the test merely shows,
     you may fix if the change doesn't alter what hearth does.
  3. If neither reaches 30, stop repairing and report the measurements: that's the user's. Don't raise the figure.

## Steps

1. Measure. Stop here if the suite is within budget.
2. Repair by `skein/docs/foundation/testing-strategy.md`, 8.1's questions, slowest first, re-measuring after each.
3. `cargo fmt`, clippy on the packages you changed, their focused tests.
4. Commit each repair with its before and after, and push.

## Report

The first line says the figure: `Done: hearth's focused suite is N CPU s`. Then both measurements, the 15 slowest tests before and after, each
repair and what it saved, and what's left over 30, if anything.
