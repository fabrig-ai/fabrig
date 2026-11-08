# store: status

2026-11-07. Where temper's code lags or departs from [store.md](store.md).

Section 2 has no known implementation gap in provider 0's subject.
The forge's codec writes the 10-byte forge and user pair, reads it only
for provider 0 and rejects other lengths. Its table holds the exact bytes,
both integer maxima and invalid forms. There is no migration or store-version
change, as no supported deployment requires compatibility.

Section 2's family 25 stores the optional `ForgeLink` in store v11 in
place, without a version bump or migration. Both numbers must be nonzero;
the record codec and its fuzzy round trip include the field. Links for
several members remain section 7's deferred work.


- **Not yet:** 2: family 22 `Delivery` (no `Key` or `Stored` variant); family 30 `LandingLog`'s trailers and tier verdicts, until `../domain/after-landing.md`, sections 6, 8 and 9 need them; the families of findings, `Finding`, `Mark` and `QaPass`.
- **Design departs:** 2, from the request `ui-away` of 2026-11-05, decided by its plan's cut: `Away` keeps `off` where the request has `on`, and `Decided` keeps `party`, a field it doesn't name (`../domain/forge.status.md`, 5.8 and 5.9).
