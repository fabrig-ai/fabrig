# design: status

2026-11-11. Where `.temper/pre-pr`'s code lags or departs from [design.md](design.md).

- **Not yet:** 2, *An unchecked save is stepped past*: the base is the merge base as git gives it, an unchecked save included, so a repair resumed after its save selects only its new edits [in flight: change 68].
- **Not yet:** 1, 5, 6, 8: nothing is built or tested at the base; a lint or
  test failure is inherited only when `.temper/known-reds.md` lists it, a
  file over the cap fails as `known-reds`, and a build failure is always the
  delivery's. 9: the tests say so, and the base builds' tests go
  [in flight: change 215].

No other gaps in sections 1–9. Section 10's later work remains outside the delivery
check.
