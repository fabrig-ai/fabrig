# store-versions: status

2026-11-22. Where the tree's code lags or departs from [store-versions.md](store-versions.md).

- **Not yet:** 1: `ShownTest`: a tier row's many names are `FailingTest`s,
  widened to 4,096 bytes in the test-id jobs' wide text, and up to
  1,024 of them, so the wide text refuses to parse [in flight: change
  9327].
- **Not yet:** 1: `ApplicationAdopted`'s `landing` [in flight: change
  8088].
- **Not yet:** 1: a failing test's id is refused at start only past the
  store's 1,024, not past a finding's signature's 512, and a tier's lists
  of failing tests are bounded at the stock's 16 [in flight: change
  8564].
