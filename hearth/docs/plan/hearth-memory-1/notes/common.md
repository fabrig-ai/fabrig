# hearth-memory-1: rules every task names

The request is `hearth/docs/plan/hearth-memory-1/inputs/request.md`. `notes/contracts.md` holds the shapes that cross tasks.

## What doesn't move

- **What hearth does for a user doesn't change:** same specs, same digests, same refusals at the edges.
- **D001 stays:** `MAX_OUT`, room checks and capped queues are later rounds. Rename, reshape or remove none of them. Deleting a formula that
  reads `MAX_OUT` lowers the `max_out` count, and that is the only way this plan lowers it.
- **skein doesn't change.** Something hearth seems to need from skein is a `Question:`, never a task's edit.
- **A branch stays in hearth,** but for `gate-table`, which is in `tools/tiers/`.
- **No assertion or case goes** but those your task names. A test the plan replaces keeps every row, and every assertion that doesn't read a
  formula.

## A ceiling

The rule is `skein/docs/foundation/testing-strategy.md`, 6. Follow `hearth/tests/git/tests/suite/memory.rs` and
`hearth/tests/builder/tests/suite/memory.rs`, which already work this way.

1. Keep the scenario fixed: the same rows, against named limits. Where a test ran at `SHIPPED` and costs over 0.2 CPU s, give it small limits
   and scale its rows, as `hearth/tests/git/src/fixtures.rs` has `SMALL` and `hearth/tests/render/src/fixtures/settings.rs` has `LIMITS`. A
   scenario that must reach a limit shows it by what it holds (entries, bytes, entities), asserted with a named size, never by nearly filling a
   formula.
2. Measure its peak: `Meter::check` returns the step's peak. Print it for the run, then remove the print.
3. Name the ceiling `const CEILING: u64` inside the one test that checks it, a quarter above the peak, rounded up to two significant digits
   (a 79,584-byte peak takes 100,000). No test shares a ceiling, and none is computed from a limit, a type's size or a formula.
4. A test that runs two sets of limits takes a ceiling for each, both beside it, `SHIPPED_CEILING` and `SMALL_CEILING`, or one if their peaks
   are within a factor of two.
5. A test that also holds what a step hands out to a second bound takes a second named ceiling for it, `HANDED_OUT_CEILING`.
6. Show it bites: lower it once below the peak, see the test fail on the ceiling, restore it. This stands in for the implementer text's check
   for a new test, and the report gives one line per test.
7. **The commit gives each peak and each ceiling,** one line per test.

A test named `..._no_more_than_the_worst_case` becomes `..._no_more_than_its_ceiling`, or `..._their_ceiling` where its subject is plural.

A test that fails after a ceiling or formula change is a finding first. Don't move a ceiling to pass it unless a measurement says the real peak
moved, and say why it did.

## A formula left

`notes/contracts.md`, 1 says which formulas stay and what each is called.

- It keeps its checked arithmetic and its `None` on overflow.
- Its doc comment says what it bounds, and no comment says "worst case".
- **It has a test of the edge's value,** in the crate that holds it, unless the contract names a test it already has: a row of limits and the
  expected `Some(..)`, built from named sizes and `size_of` terms, and a row that overflows to `None`.

## The end of every task

- Lower `hearth/scripts/old-forms.txt` with `python3 -I -B scripts/old_forms.py --write`, run from `hearth/`.
- Report the CPU seconds of the tests you changed, before and after, from nextest's times.
