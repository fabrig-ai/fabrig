# design: status

2026-11-09. Where `.temper/pre-pr`'s code lags or departs from [design.md](design.md).

- **Not yet:** 2: the check reads no base; it runs the format check and the
  counted baselines alone, on every delivery. 3: no selection. 4: checks 3
  and 4 (the lints and the focused tests) are not run, and no check is
  announced or timed. 5: nothing is run at a base. 6: no summary. 7: the
  cache holds the tree alone. 9: the boundary world covers the format
  check, the baselines and the tree cache alone.
