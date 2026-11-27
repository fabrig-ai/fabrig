# decisions: status

2026-11-27. Where the tree's code lags or departs from [decisions.md](decisions.md).


- **Not yet:** 1.1: the whole project-deletion commit has no
  established admission price against `journal.writes`, because the
  established allowance covers settings-proposal withdrawals alone,
  excluding the remaining state writes, closing the project and cancellation
  operations. No reason is recorded.
- **Not yet:** 1.3: the engine's table of loads in flight (`Loads::new`) has no capacity from the limits, so the bound lives only in its room checks [in flight: change 21078].
