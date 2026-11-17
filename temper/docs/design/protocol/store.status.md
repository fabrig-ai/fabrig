# store: status

2026-11-16. Where temper's code lags or departs from [store.md](store.md).



- **Not yet:** 2: family 41, `LandedDraft`, neither kept nor read
  [in flight: change 663].
- **Not yet:** 2: family 42, `Follow`, a follow's record [in flight: change 105].

- **Not yet:** 2: the `Change` row's `follows` [in flight: change 99].




- **Not yet:** 2: family 22 `Delivery` (no `Key` or `Stored` variant); family 30 `LandingLog`'s trailers and tier verdicts, until `../domain/after-landing.md`, sections 6, 8 and 9 need them; the families of findings, `Finding`, `Mark` and `QaPass`.
- **Design departs:** 2, from the request `ui-away` of 2026-11-05, decided by its plan's cut: `Away` keeps `off` where the request has `on`, and `Decided` keeps `party`, a field it doesn't name (`../domain/forge/attention.status.md`, 4 and 5).

- **Not yet:** 4, the tier rows' ceilings of 257: nothing checks `forge.changes` against them before the store's sizing, so a value past 128 refuses the start as past the codec's bounds, naming no limit.

- **Not yet:** 4, the tier rows' ceilings of 2049: the schema still holds 257, so `forge.changes` is at most 128.

