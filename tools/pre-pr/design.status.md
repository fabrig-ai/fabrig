# design: status

2026-11-15. Where `.temper/pre-pr`'s code lags or departs from [design.md](design.md).

- **Not yet:** 1, 4, 5, 6 and 9: the check still reads the tree's list of known reds beside `temper/reds`, with its cap and the `known-reds` failure, prints `(known red, …)` for an entry of the list, and its tests pin the list [in flight: change 3057].
- **Not yet:** 1, 6, 7 and 9: the check writes no failures file and its summary has no failures line, a cache hit writing none either; its tests pin neither [in flight: change 3690].

No other gaps in sections 1–9. Section 10's later work remains outside the delivery
check.
