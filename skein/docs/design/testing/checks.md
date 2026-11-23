# Checks performed by test kits

## 1. The counter

`skein_checks::count(kind, n)` records work that a shared test helper
performs so a change in coverage is visible even when every test still
passes. The gate's use of these counts is in `tools/tiers/budgets.md`.

- **`store-check`** counts each cold rehearsal that returns a verdict,
  including a refused or stalled start. The parent counts each
  successful commit-cut restart when the child reports success.
- **`seed`** counts each seed case the shared sweep runs and checks.
  A sweep that compares schedules counts each schedule it checks,
  including the reference run. A fault-ledger sweep counts each
  completed run whose ledger it folds.
- **`mutation`** counts each generated variant supplied to a judge
  that returns. Creating, selecting, borrowing or discarding a variant
  records nothing. The mutation marker follows the input through fixture
  preparation. A judgment over a combined input counts each constituent
  mutation, and one judgment that compares several readings of the same
  variant counts it once. An unchanged control carries zero.

Loops that judge generated variants accumulate the counts in
`MutationBatch` and append the total once after the loop. A batch that
judges no mutations writes no line. A shared seed sweep can append the
total after its checks return too.

`skein-checks` depends only on the standard library. The listed kit
files own counter calls and the creation of mutation markers. Stories
use the kit's check or pass marked inputs to its judge. The repository
checks the list in `tests/boundary/tests/check_counts.txt`.

The helper reads `TIERS_CHECKS_FILE` once per process. An unset or empty
value leaves it idle. Otherwise it opens the path for append and writes
one `KIND COUNT\n` line per call. A kind is a lowercase ASCII letter then
up to 31 lowercase ASCII letters, digits or hyphens. The count is a
positive decimal integer. An open or write failure panics with the path.
The kit tests the malformed-kind guard in a fresh process to check that
an invalid call fails before it can write a line.

## 2. Later

No work deferred.
