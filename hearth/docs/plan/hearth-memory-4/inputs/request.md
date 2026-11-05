# Request: hearth to the dynamic memory model, round 4: the config machines

From fabrig/hearth, 2026-11-05, on the user's decision to migrate hearth, low priority, one small plan at a time. Rounds 2 and 3 uncapped the git and mmdebstrap machines. Read
`hearth/docs/development/debt/D001-room-checks-and-max-out.md` whole, and do what they did. **D001 isn't paid by this round: keep its file, updated.** What hearth does for a user doesn't change.

## The machine

`hearth-protocol-config` (`src/json.rs`, its `MaxOut` and `MAX_OUT` forms): they go, the queues beside it are uncapped, and its reservations go, as
`skein/docs/foundation/programming-model.md`, 6.3 and 7.1 say: only the edges refuse. `io::MAX_OUT_*`, the kernel's edge, stays.

Before a check goes, D001's table says what it keeps. A check it doesn't list, or an order with no test yet, gets the row and a test that fails when the order breaks, first, as round 3's order task did.

If `hearth-service-builder` or `hearth-protocol-builder` needs a constant this machine loses, make the smallest change that keeps the service compiling and behaving as it does, and name what the service round must finish.

## And one sentence

`hearth/docs/design/protocol/format.md`, 3.5, about line 336, still names a container from the example fleet. Write the sentence around the target it becomes, `chat-prod`, and remove the question about it from the document's open questions (section 13: "A bash fleet's container name in 3.5"). The id base beside it stays. This is the user's word to neutralize; it's unrelated to the machine, so keep it its own commit.

## Rules

- Keep it small. The records machine and the service's `iterate` are later rounds, and so is D004. `hearth/scripts/old-forms.txt` lowers with `--write`.
- D001's rows for what this round removes go, and its Measure and next step are current. Its order held only by the design, resumed work before the next event, is the service round's: leave it.
- The focused suite is over the 30 CPU s `tools/tiers/budgets.toml` holds (about 32 on the gate's record): this round doesn't raise it. New tests stay small.
- A ceiling moved by this round gives the peak that moved it, and why.
- A skein change hearth seems to need is a question, not a task.
- `tools/citations/citations.py --all` stays at 0, and `tests/boundary/tests/outward.rs` passes.
- All parts must be in the plan when the cut ends.
