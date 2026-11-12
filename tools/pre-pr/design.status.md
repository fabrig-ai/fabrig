# design: status

2026-11-12. Where `.temper/pre-pr`'s code lags or departs from [design.md](design.md).

- **Not yet:** 1, 4, *The hold*, 9: the check takes no hold: a deployment's cargo that lends the build directory only while a cargo command runs held it for one-process's listing alone, and on 2026-11-12 a check failed with a built test executable gone before one-process started it (`FileNotFoundError` from `tools/tiers/one_process.py`'s Popen); the boundary world has no fake hold [in flight: change 1023].

- **Not yet:** 2, *An unchecked save is stepped past*: the base is the merge base as git gives it, an unchecked save included, so a repair resumed after its save selects only its new edits.
- **Not yet:** 1, 5, 6, 8: nothing is built or tested at the base; a lint or
  test failure is inherited only when `.temper/known-reds.md` lists it, a
  file over the cap fails as `known-reds`, and a build failure is always the
  delivery's. 9: the tests say so, and the base builds' tests go.
- **Not yet:** 1, 4, 5, 6: the focused tests run under a `cargo nextest run` of
  the check's own, with a JUnit overlay, not through `tools/tiers/one-process
  default`; no starved line, alone label or `check one-process`. 9: the tests
  fake nextest, not one-process.

No other gaps in sections 1–9. Section 10's later work remains outside the delivery
check.
