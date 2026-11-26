# cli: status

2026-11-26. Where the repository's code lags or departs from [cli.md](cli.md).

- **Departs:** 9: check 5 counts a declared variable's own table row as a
  read or set, so a declaration that nothing reads or sets passes.
- **Not yet:** 6 and 8: skein has no test table and no test-env
  document; `skein_world::test_env::TESTS` and the fuzzy sweeps' two
  variables, read once in a process, come with the case seam
  [in flight: change 19499].
