# hearth-memory-5: rules every task names

The request is `hearth/docs/plan/hearth-memory-5/inputs/request.md`. The design this plan builds on is `hearth/docs/design/domain/render.md`, 8.4 and 8.5, and the list of checks is `hearth/docs/development/debt/D001-room-checks-and-max-out.md`, "What each room check keeps".

## What doesn't move

- **What hearth does for a user doesn't change:** same specs, same digests, same records, same refusals at the edges.
- **D001 stays open.** This round takes `hearth-client-protocol-records` off the old form and no other crate. `hearth-service-builder`'s `iterate`, `hearth-protocol-builder`'s `MaxOut` and `MAX_OUT_*`, `io::MAX_OUT_*`, the queues of `Layer` in `hearth-protocol-builder/src/tests/fixtures.rs`, `QUEUE = 3` in `hearth/tests/builder/tests/suite/bounds.rs`, D004 and D013 stay as they are.
- **The printer's edges stay.** A record past `Limits::record` is refused (`hearth/docs/design/domain/render.md`, 8.4), and `a_refused_spec_is_left_out_and_the_call_ends_failed`, in `hearth-protocol-builder`, holds it. `Limits`, `valid` and `Printer::new`'s assertion stay. So do the sized `Writer`s and `lengths` in `form/`, and the depth `Stack` in `parts/exports.rs`: none reserves room for an output.
- **skein doesn't change.** Something hearth seems to need from skein is a `Question:`, never a task's edit.
- **A branch stays in hearth.**
- **No assertion or case goes** but those your task names.

## A test of an order

Each one holds an order in `skein/docs/foundation/programming-model.md`, 7.2's sense: what stays ahead of what, and what keeps it there.

- **It passes today,** on the code as it stands. It is there to fail if the order goes, later.
- **Its name states the order,** because D001's table cites it.
- **Where it sits:** hearth has no records world, and this round makes none. The printer's tests sit beside `Machine`, in `hearth-client-protocol-records/src/tests/printer.rs`, over every chunk size and fault point the task names. `Client` and the printer together are held in `hearth/tests/builder`'s protocol world, under its seeded latencies.
- **A queue the test owns is `Queue::uncapped()`,** so it goes on biting once the caps are gone: a capped queue turns a broken order into a panic on a full queue.
- **It is small:** fixed seeds, under 0.05 CPU s each. The focused suite is over the 30 CPU s of `tools/tiers/budgets.toml`, so a test that must cost more says so in the report, with the measurement.
- **It bites.** Your task names the rule to break for each test. Break it by hand, see the test fail for that reason, and restore it.

## The end of every task

- From `hearth/`, `python3 -I -B scripts/old_forms.py`, with `--write` only to lower a count for what your task removed.
- From the root, `python3 -I -B tools/citations/citations.py --all` stays at 0.
- The report gives the CPU seconds of the tests you added or changed, from nextest's times.
