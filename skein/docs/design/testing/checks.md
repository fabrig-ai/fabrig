# Checks performed by test kits

## 1. The counter

`skein_checks::count(kind, n)` records work that a shared test helper
performs so a change in coverage is visible even when every test still
passes. A store rehearsal counts one `store-check` for its commit, a
shared seeded sweep one `seed` for each seed run, and a shared mutation
helper one `mutation` for each generated mutation judged. The gate's use
of these counts is in `tools/tiers/budgets.md`.

`skein-checks` depends only on the standard library. A test kit that
counts checks takes the crate directly, so it need not build a world
harness for the counter.

The helper reads `TIERS_CHECKS_FILE` once per process. An unset or empty
value leaves it idle. Otherwise it opens the path for append and writes
one `KIND COUNT\n` line per call. A kind is a lowercase ASCII letter then
up to 31 lowercase ASCII letters, digits or hyphens. The count is a
positive decimal integer. An open or write failure panics with the path.
The kit tests the malformed-kind guard in a fresh process to check that
an invalid call fails before it can write a line.

## 2. Later

No work deferred.
