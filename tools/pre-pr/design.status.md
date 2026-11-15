# design: status

2026-11-15. Where `.temper/pre-pr`'s code lags or departs from [design.md](design.md).

- **Not yet:** 1, 3, 4, 5, 6, 8 and 9: the check selects no boundary packages, so a docs-only delivery runs citations alone and a package on an unprefixed `.config/gate/always.txt` line is tested only where it changed or depends on what did; it runs no fuzzy tests; its tests pin neither [in flight: change 3858].

No other gaps in sections 1–9. Section 10's later work remains outside the delivery
check.
