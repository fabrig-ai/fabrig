# design: status

2026-11-17. Where `.temper/pre-pr`'s code lags or departs from [design.md](design.md).

- **Not yet:** 1, 6, 7 and 9: the check writes the failures file to the git directory's `temper/pre-pr-failures.json`, where an agent's `read` cannot reach it, and its failures line names that file's absolute path, a cache hit's too.
- **Not yet:** 1, 3, 4, 5, 8 and 9: no delivery is prose only: one of Markdown under `docs/` alone builds and runs the boundary packages' focused and fuzzy tests, there is no outward text check, and `.config/gate/read-by-tests.txt` does not declare the command-line tools' `*-args.md` references or `temper/docs/design/testing/qa/setups.md`.

No other gaps in sections 1–9. Section 10's later work remains outside the delivery
check.
