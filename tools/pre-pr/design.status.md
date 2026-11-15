# design: status

2026-11-15. Where `.temper/pre-pr`'s code lags or departs from [design.md](design.md).

- **Not yet:** 1, 3, 4, 5, 6, 8 and 9: the check selects no boundary packages, so a docs-only delivery runs citations alone and a package on an unprefixed `.config/gate/always.txt` line is tested only where it changed or depends on what did; it runs no fuzzy tests; its tests pin neither [in flight: change 3858].
- **Not yet:** 9: the boundary world still runs the tables' rows end to end, thirteen tests where 9.1 keeps ten stories, and `tools/pre-pr/test_pre_pr.py` lacks the base's and the selection's tables, each tree's count failing, a moved pin, the summary's bound and the unset hold [in flight: change 3666].
- **Not yet:** 1, 6, 7 and 9: the check writes no failures file and its summary has no failures line, a cache hit writing none either; its tests pin neither [in flight: change 3690].

No other gaps in sections 1–9. Section 10's later work remains outside the delivery
check.
