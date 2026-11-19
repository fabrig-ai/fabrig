# store: status

2026-11-18. Where temper's code lags or departs from [store.md](store.md).


- **Not yet:** 4: `ShownTest`: a tier row's many names are `FailingTest`s,
  widened to 4,096 bytes in the test-id jobs' wide text, and up to
  1,024 of them, so the wide text refuses to parse [in flight: change
  9327].
- **Not yet:** 4: `ApplicationAdopted`'s `landing` [in flight: change
  8088].
- **Not yet:** 2: family 30, `LandingLog`: its members keep no goal or
  milestone, its landings no tiers' entries, and it holds its last
  `landings_kept` landings whether or not a tier has judged them
  [in flight: change 8589].


- **Not yet:** 2: family 41, `LandedDraft`, neither kept nor read
  [in flight: change 663].
- **Not yet:** 2: family 42, `Follow`, a follow's record [in flight: change 105].

- **Not yet:** 2: the `Change` row's `follows` [in flight: change 99].




- **Not yet:** 2: family 22 `Delivery` (no `Key` or `Stored` variant); family 30 `LandingLog`'s trailers and tier verdicts, until `../domain/after-landing.md`, sections 6, 8 and 9 need them; the families of findings, `Finding`, `Mark` and `QaPass`.
- **Design departs:** 2, from the request `ui-away` of 2026-11-05, decided by its plan's cut: `Away` keeps `off` where the request has `on`, and `Decided` keeps `party`, a field it doesn't name (`../domain/forge/attention.status.md`, 4 and 5).



