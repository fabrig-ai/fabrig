# hearth-memory-3: rules every task names

The request is `hearth/docs/plan/hearth-memory-3/inputs/request.md`. The design this plan builds on is
`hearth/docs/design/domain/render.md`, 5.3, 5.5 and 8.5, and the list of checks is
`hearth/docs/development/debt/D001-room-checks-and-max-out.md`, "What each room check keeps".

## What doesn't move

- **What hearth does for a user doesn't change:** same specs, same digests, same records, same refusals at the edges.
- **D001 stays open.** This round takes `hearth-protocol-mmdebstrap` off the old form and no other crate. The config and records machines,
  `hearth-service-builder`'s `iterate`, `hearth-protocol-builder`'s `MaxOut` and `MAX_OUT_*`, `io::MAX_OUT_*`, `QUEUE = 3` in
  `hearth/tests/builder/src/fixtures/bounds.rs` and D004 stay as they are.
- **`tail.rs` stays as it is.** `admit`'s `tail.lines.room() == 0`, `Tail`'s capped queue and every `decoder_bound` are the edge's: a child's output.
- **skein doesn't change.** Something hearth seems to need from skein is a `Question:`, never a task's edit.
- **A branch stays in hearth.**
- **No assertion or case goes** but those your task names.

## A test of an order

Each one holds an order in `skein/docs/foundation/programming-model.md`, 7.2's sense: what stays ahead of what, and what keeps it there.

- **It passes today,** on the code as it stands. It is there to fail if the order goes, later.
- **Its name states the order,** and it sits in `hearth/tests/mmdebstrap`, under load: seeded cuts, not one clean delivery.
- **A queue the test owns is `Queue::uncapped()`,** so it goes on biting once the machines' caps are gone: a capped queue turns a broken order into a
  panic on a full queue.
- **It is small:** fixed seeds, under 0.05 CPU s each. `hearth/docs/design/domain/testing.status.md`, 9.1 has the focused suite over the 30 of
  `tools/tiers/budgets.toml`, so a test that must cost more says so in the report, with the measurement.
- **It bites.** Your task names the rule to break for each test. Break it by hand, see the test fail for that reason, and restore it.

## The end of every task

- From `hearth/`, `python3 -I -B scripts/old_forms.py`, with `--write` only to lower a count for what your task removed.
- From the root, `python3 -I -B tools/citations/citations.py --all` stays at 0.
- The report gives the CPU seconds of the tests you added or changed, from nextest's times.
