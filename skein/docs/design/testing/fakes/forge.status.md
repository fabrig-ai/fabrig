# fake-forge: status

2026-11-21. Where skein's code lags or departs from [forge.md](forge.md).


Unlisted sections have no known gap. `github.md`, section 1.1's App JWT check uses
the key's public half and verifies without signing; its exact-token/key
cache and the claims checked on every request are present.

- **Not yet:** 2.3: a merge recorded at a commit, and manual merges
  (`allow_manual_merge`, `autodetect_manual_merge`): the fake refuses
  `Do: manually-merged` (`skein-fake-forge-domain/src/pulls.rs`)
  [in flight: change 813].
- **Not yet:** 2.2: a pull request's size, counted by the fake and written by both protocol layers [in flight: change 532].
- **Not yet:** 2.2: a close refused by policy (`scenario::refuse_close`) [in flight: change 11616].
- **Not yet:** 2.3: one runner for every job (`scenario::runners`) [in flight: change 11616].
- **Not yet:** 2.3: lagging views (`scenario::lag`) [in flight: change 11615].
- **Not yet:** 2.4: what fell: the domain keeps no record of the faults it injects (`Fell`, `pop_fell`) [in flight: change 11597].
- **Not yet:** 2.3: `Write::Cancel` on a commit's CI run: the fake cancels dispatched runs alone (`skein-fake-forge-domain/src/dispatch.rs`) [in flight: change 11633].
