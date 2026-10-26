# Measurements at the base

Read 2026-10-15 from the `Gate-Result` trailers of the last four code
landings on each repository's `origin/main` (`git log origin/main
--format='%h %(trailers:key=Gate-Result,valueonly)'`), each a gate run
on carbon-forge. Nothing else was measured.

| Repository | Landings | Focused suite (s) | Fuzzy suite (s) | Tests (focused / fuzzy) |
|---|---|---|---|---|
| jig | `06ac4c46`, `485337db`, `b50df428`, `415ac928` | 10.887, 9.623, 10.910, 9.783: median 10.33 | 15.587, 13.705, 15.765, 14.226: median 14.91 | 3,568 / 109 |
| temper | `b1994cdf`, `145468f1`, `1b9e6d22`, `eb9ec31f` | 4.050, 3.898, 4.040, 3.959: median 4.00 | 15.276, 14.673, 14.054, 13.619: median 14.36 | 1,265 / 46 |

The budgets are 15 s focused and 60 s fuzzy in both (each repository's
`docs/development/workflow.md`, section 2). One gate's swing on the gate
machine is about 1.3 s (skein's development.md, 5.6).
