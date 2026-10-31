# accounts: status

2026-10-31. Where jig's code lags or departs from [accounts.md](accounts.md).

- **Built:** 1 to 7: runtime sources, holder/payer selection, model preferences, program registry/placement, notices and durable restore. 3.5: stable two-clock deadlines. 5: pasted sign-in completion restricted to its starter; deployment-plan sign-in refused. 6: account export/import with bounded JIGM records, refresh and restart/deadline cancellation.
- **Built:** 3.6: a restored set past its limit, or a stored name past its byte bound, refuses the start naming key, set and limit (`RestoreLimit`, the core's `start::Failure::AccountsPastLimit`), at every start; an account a source names that is not held, or a missing reservation, panics at its link. Restore loads and links alone, judging no row.
- **Not yet:** Process-tree containment; first-plan-use words; retained sign-in answers can name a dead visit address after restart. Other remaining detail unknown. lags: 3.6: validity checks the accounts-times-sources retained count independently of outputs, until `ready/jig/c-tail-2` lands.
- **4.2, a project in a key's uses** (2026-10-31): built. The check takes the task's project with its payer (`jig-domain-core/src/coverage.rs`); a key naming the project covers every party's work there, a plan's uses are unchanged. Tests: `a_key_naming_a_project_pays_for_a_persons_work_in_it_and_nowhere_else`, `a_plan_naming_a_project_still_does_not_cover_another_persons_work_in_it`, and `a_move_keeps_a_project_key_covering_the_new_financial_payer` (`jig/tests/client-protocol`).
