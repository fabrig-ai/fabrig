# hearth-memory-2: rules every task names

The request is `hearth/docs/plan/hearth-memory-2/inputs/request.md`. The design this plan builds on is
`hearth/docs/design/domain/render.md`, 2.2 and 8.5.

## What doesn't move

- **What hearth does for a user doesn't change:** same specs, same digests, same records, same refusals at the edges.
- **D001 stays open.** This round takes `hearth-protocol-git` off the old form and no other crate. The mmdebstrap, config and records machines,
  `hearth-service-builder`'s `iterate`, `io::MAX_OUT_*`, and `QUEUE = 3` in `hearth/tests/builder/src/fixtures/bounds.rs` stay as they are. D004
  rewrites `io_up` and `io_down`, and comes with the service round: leave them.
- **skein doesn't change.** Something hearth seems to need from skein is a `Question:`, never a task's edit.
- **A branch stays in hearth.**
- **No assertion or case goes** but those your task names.

## A test of an order

Each one holds an order in `skein/docs/foundation/programming-model.md`, 7.2's sense: what stays ahead of what, and what keeps it there.

- **It passes today,** on the code as it stands. It is there to fail if the order goes, later.
- **Its name states the order,** and it sits in the world your task names, under load: cut, fragmented or delayed input, not one clean delivery.
- **A queue the test owns is `Queue::uncapped()`,** so it goes on biting once the machines' caps are gone: a capped queue would turn a broken order
  into a panic on a full queue, which an uncapped one doesn't give.
- **It is small:** fixed seeds, small limits, under 0.05 CPU s each. hearth's focused suite stands at 30.2 CPU s against the 30 of
  `tools/tiers/budgets.toml`, so a test that must cost more says so in the report, with the measurement.
- **It bites.** Your task names the rule to break for each test. Break it by hand, see the test fail for that reason, and restore it.

## The end of every task

- From `hearth/`, `python3 -I -B scripts/old_forms.py`, with `--write` only to lower a count for what your task removed.
- From the root, `python3 -I -B tools/citations/citations.py --all` stays at 0.
- The report gives the CPU seconds of the tests you added or changed, from nextest's times.
