# README: status

2026-11-25. Where temper's code lags or departs from [README.md](README.md).

- **Design departs:** 2.7: draft-13's request to let the landing launcher
  pass on macOS (2026-11-25) asked for focused tests on Linux of what
  changes. The launcher's decisions are sh and Python, no step's, so its
  tests stay `temper-box::suite`'s end-to-end `forge_land` tables, which
  the real pair runs on Linux after landing.

