# design: status

2026-11-17. Where `.temper/pre-pr`'s code lags or departs from [design.md](design.md).

- **Not yet:** 1, 4, 5, 6, 7 and 9: the check reads no `TIERS_LEDGER` and passes no `--ledger` or `--gate`, so a layer over its budget or a test past its reference is a warning at every delivery; it reads no exit 1 through `cpu-summary` and names no failure `budget …` or `reference …`; the passed-tree cache's record has three fields, keying no ledgers; and story 11 and the ledgers' unit tests are not written [in flight: change 6350].

No other gaps in sections 1–9. Section 10's later work remains outside the delivery
check.
