# Request: hearth to the dynamic memory model, round 3: the mmdebstrap machine

From fabrig/hearth, 2026-11-05, on the user's decision to migrate hearth, low priority, one small plan at a time. Round 2 (`hearth-memory-2`) uncapped the git
machine and listed what every room check keeps, in `hearth/docs/development/debt/D001-room-checks-and-max-out.md`: read it whole, and do git's change the same
way. **D001 isn't paid by this round: keep its file, updated.** What hearth does for a user doesn't change.

## The machine

`hearth-protocol-mmdebstrap`: its `MaxOut`, `UP_MAX_OUT` and `DOWN_MAX_OUT` go, the queues beside it are uncapped, and its reservations go, as
`skein/docs/foundation/programming-model.md`, 6.3 and 7.1 say: only the edges refuse. `io::MAX_OUT_*`, the kernel's edge, stays.

Before a check goes, D001's table says what it keeps. `tail.rs`'s `admit` keeps a capacity at an edge and an order, the oldest line leaving first, and its test is named there:
it stays, and that test keeps passing. A check the table doesn't list, or whose order has no test yet, gets the row and the test first.

If `hearth-service-builder` or `hearth-protocol-builder` needs a constant this machine loses, make the smallest change that keeps the service compiling and behaving as
it does, as round 2 did, and name what the service round must finish.

## Rules

- Keep it small. The config and records machines and the service's `iterate` are later rounds, and so is D004. `hearth/scripts/old-forms.txt` lowers with `--write`.
- D001's rows for what this round removes go, and its Measure and next step are current. It records one order held only by the design, resumed work before the next event: that's the service round's, leave it.
- The focused suite stays under what `tools/tiers/budgets.toml` holds. New tests stay small.
- A ceiling moved by this round gives the peak that moved it, and why.
- A skein change hearth seems to need is a question, not a task.
- `tools/citations/citations.py --all` stays at 0, and `tests/boundary/tests/outward.rs` passes.
- All parts must be in the plan when the cut ends.
