# D042: Shared scenarios run bundled, several to one test

- **Kind:** tests
- **Where:** `tests/conformance/tests/suite/scenarios.rs`, `tests/conformance/tests/suite/fuzzy/scenarios.rs`, `testing/jig-conformance/src/scenarios.rs`
- **Seen:** 2026-10-31 at `c30cf6a03`, in the jig code-quality review (order of work, item 19: "unbundle the 50 bundled scenarios")
- **Seen:** 2026-10-31 at `1b5c98834`, in the quality run's debt list (item 19: kept bundled, "Call 15, the world's CPU budget")
- **Checked:** 2026-11-03 at `41d63da8c3`
- **Value:** low (a failure names its scenario at once)
- **Size:** S
- **Status:** kept (each test pays a world's set-up once; the quality run's designer kept the bundles for the worlds' CPU budget)
- **Paid when:** kept; if the CPU budget allows one test per scenario, each shared scenario gets its own test and this file goes.

## What

`every_shared_adversarial_story_runs_on_the_testing_application` (`scenarios.rs`, line 13)
loops over `ALL`, the 13 shared scenarios (`testing/jig-conformance/src/scenarios.rs`, line 38),
and stops at the first that fails; the fuzzy suite does the same. The quality run made
other bundles report every failing case; this one reports the first.

## Why it matters

A bundled test hides which scenarios fail after the first, and its time is the sum of
theirs.

## What pays it back

Kept. A cheap step while kept: collect every scenario's error before failing, as the
other bundles do.
