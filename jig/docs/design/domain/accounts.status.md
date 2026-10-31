# accounts: status

2026-10-31. Where jig's code lags or departs from [accounts.md](accounts.md).

- **Built:** 1 to 7: runtime sources, holder/payer selection, model preferences, program registry/placement, notices and durable restore. 3.5: stable two-clock deadlines. 5: pasted sign-in completion restricted to its starter; deployment-plan sign-in refused. 6: account export/import with bounded JIGM records, refresh and restart/deadline cancellation.
- **Built:** 3.6: a restored set past its limit, or a stored name past its byte bound, refuses the start naming key, set and limit (`RestoreLimit`, the core's `start::Failure::AccountsPastLimit`), at every start; an account a source names that is not held, or a missing reservation, panics at its link. Restore loads and links alone, judging no row.
- **Not yet:** Process-tree containment; first-plan-use words; retained sign-in answers can name a dead visit address after restart.
- **Built:** 3.6: validity checks the accounts-times-sources retained count independently of outputs.
