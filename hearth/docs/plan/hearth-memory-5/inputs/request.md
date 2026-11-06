# Request: hearth to the dynamic memory model, round 5: the records machine

From fabrig/hearth, 2026-11-05, on the user's decision to migrate hearth, low priority, one small plan at a time. Rounds 2 to 4 uncapped the git, mmdebstrap and config machines. Read
`hearth/docs/development/debt/D001-room-checks-and-max-out.md` whole, and do what they did. **D001 isn't paid by this round: keep its file, updated.** What hearth does for a user doesn't change. This is the last machine before the service.

## The machine

`hearth-client-protocol-records` (`src/printer.rs`, its `MaxOut` and `MAX_OUT` forms): they go, the queues beside it are uncapped, and its reservations go, as
`skein/docs/foundation/programming-model.md`, 6.3 and 7.1 say: only the edges refuse. `io::MAX_OUT_*`, the kernel's edge, stays. The printer's refusal of a record past its limit (`hearth/docs/design/domain/render.md`, 8.4) is an edge: it stays, with its test.

Before a check goes, D001's table says what it keeps. A check it doesn't list, or an order with no test yet, gets the row and a test that fails when the order breaks, first, as the earlier order tasks did. The printer takes one record at a time, and `protocol_down` relies on that: `orders::what_a_call_prints_and_stores_does_not_depend_on_its_queues` holds it, and it keeps passing.

If `hearth-service-builder` or `hearth-protocol-builder` needs a constant this machine loses, make the smallest change that keeps the service compiling and behaving as it does, and name what the service round must finish.

## Rules

- Keep it small. The service's `iterate` is the next round, and so is D004. `hearth/scripts/old-forms.txt` lowers with `--write`.
- D013 (records written three times) is fabrig/debt's: leave it.
- D001's rows for what this round removes go, and its Measure and next step are current. Its order held only by the design, resumed work before the next event, is the service round's: leave it.
- The focused suite is over the 30 CPU s `tools/tiers/budgets.toml` holds (about 32 on the gate's record): this round doesn't raise it. New tests stay small.
- A ceiling moved by this round gives the peak that moved it, and why.
- A skein change hearth seems to need is a question, not a task.
- `tools/citations/citations.py --all` stays at 0, and `tests/boundary/tests/outward.rs` passes.
- All parts must be in the plan when the cut ends.
