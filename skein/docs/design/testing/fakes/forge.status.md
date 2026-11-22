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
