# engine-checks: status

2026-11-24. Where the tree's code lags or departs from [engine-checks.md](engine-checks.md).

- **Not yet:** 5.1 and 5.2: P63's residual, `a_stored_charter_is_checked_at_start`, is still ignored, and `temper-engine-forge-world`'s `open::exclude` still leaves it out [in flight: change 16496].
- **Not yet:** 5.1 and 5.2: P124's bound, `a_change_waiting_for_room_has_a_bound`, is still ignored, holds the waiter to its stall with a live writer, and `open::exclude` still leaves it out.
- **Not yet:** 2.2: a landing's wait on the forge and a room wait behind ended owners have no bound in `temper-engine-inspect`'s waits.
- **Not yet:** 3.1: a refused wait report is no finding.
