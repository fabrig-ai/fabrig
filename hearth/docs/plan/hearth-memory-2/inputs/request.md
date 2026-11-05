# Request: hearth to the dynamic memory model, round 2: what the room checks keep, and the git machine

From fabrig/hearth, 2026-11-05, on the user's decision to migrate hearth, low priority, one small plan at a time. Round 1 (`hearth-memory-1`) took
`worst_case` to 0. This round starts D001 (`hearth/docs/development/debt/D001-room-checks-and-max-out.md`; read it whole), `max_out` 84. The later
rounds take the other machines, then `hearth-service-builder`. **D001 isn't paid by this round: keep its file, updated.** What hearth does for a user
doesn't change.

## 1. What each room check keeps

Before anything goes, by `skein/docs/foundation/programming-model.md`, 7.2: list every `.room()` call, `io_room`, `protocol_room` and `within` in hearth's
step code and service, and say what each keeps: a capacity, or an order. Put the list in the status file of the design document that owns it, or in D001 if none
does. For each order a check keeps, whether or not on purpose, write the test that fails if the order goes, in the world that holds it, passing today. An order
the design doesn't state goes into the design, as a statement of what hearth does. If a check keeps nothing, say so.

## 2. The git machine, uncapped

`hearth-protocol-git` is the first machine, and the bottom of the stack. Its `MaxOut`, `UP_MAX_OUT` and `DOWN_MAX_OUT` go, its internal queues are uncapped, and its
`.room()` calls go, as `skein/docs/foundation/programming-model.md`, 6.3 and 7.1 say: only the edges refuse. skein's other machines are the model. `io::MAX_OUT_*`, the
kernel's edge, stays. The orders section 1 found stay held by their tests.

If `hearth-service-builder` or `hearth-protocol-builder` needs a constant this machine loses, make the smallest change that keeps the service compiling and
behaving as it does, and name what the service round must finish. If the order proves wrong, and the service must move first, stop and ask.

## Rules

- Keep it small, and leave the other machines (mmdebstrap, config, records) and the service's `iterate` to later rounds. `hearth/scripts/old-forms.txt` lowers with `--write`.
- D004 rewrites the same `io_up` and `io_down`, and comes with the service round: leave it.
- `hearth/tests/builder/src/fixtures/bounds.rs`'s `QUEUE = 3` goes only with the service.
- The focused suite stays under 30 CPU s, as `tools/tiers/budgets.toml` holds it (30.2 today, gate's record). New order tests stay small; if a test must be slow, say so.
- A ceiling moved by this round gives the peak that moved it, and why.
- A skein change hearth seems to need is a question, not a task.
- `tools/citations/citations.py --all` stays at 0, and `tests/boundary/tests/outward.rs` passes.
- All parts must be in the plan when the cut ends.
