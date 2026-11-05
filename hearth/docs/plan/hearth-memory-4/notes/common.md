# hearth-memory-4: rules every task names

The request is `hearth/docs/plan/hearth-memory-4/inputs/request.md`. The design this plan builds on is `hearth/docs/design/domain/render.md`, 2.3, 8.4 and 8.5, and the list of
checks is `hearth/docs/development/debt/D001-room-checks-and-max-out.md`, "What each room check keeps".

## What doesn't move

- **What hearth does for a user doesn't change:** same specs, same digests, same records, same refusals at the edges.
- **D001 stays open.** This round takes `hearth-protocol-config` off the old form and no other crate. The records machine, `hearth-service-builder`'s `iterate`,
  `hearth-protocol-builder`'s `MaxOut` and `MAX_OUT_*`, `io::MAX_OUT_*`, `Client`'s queues in `client.rs`, `QUEUE = 3` in `hearth/tests/builder/src/fixtures/bounds.rs` and D004 stay as they are.
- **The decoder's edges stay.** `Stack::with_capacity(limits.depth)` under the tokenizer's depth refusal, `Map::with_capacity(limits.keys)` whose full map is `Fault::Keys`, the lists sized by a
  count already read (`List::with_capacity(entries.len())` and the like, in `document.rs`, `entries.rs`, `declaration.rs` and `schema.rs`) and every `decoder_bound` and `decoded_bound`.
  None reserves room for an output: `skein/docs/foundation/programming-model.md`, 6.3 keeps formulas at the edges, and these lists allocate as they fill.
- **skein doesn't change.** Something hearth seems to need from skein is a `Question:`, never a task's edit.
- **A branch stays in hearth.**
- **No assertion or case goes** but those your task names.

## A test of an order

Each one holds an order in `skein/docs/foundation/programming-model.md`, 7.2's sense: what stays ahead of what, and what keeps it there.

- **It passes today,** on the code as it stands. It is there to fail if the order goes, later.
- **Its name states the order,** and it sits in `hearth/tests/config`, under load: seeded cuts, not one clean delivery.
- **A queue the test owns is `Queue::uncapped()`,** so it goes on biting once the machine's caps are gone: a capped queue turns a broken order into a panic on a full queue.
- **It is small:** fixed seeds, under 0.05 CPU s each. The focused suite is over the 30 CPU s of `tools/tiers/budgets.toml`, so a test that must cost more says so in the report, with the
  measurement.
- **It bites.** Your task names the rule to break for each test. Break it by hand, see the test fail for that reason, and restore it.

## The end of every task

- From `hearth/`, `python3 -I -B scripts/old_forms.py`, with `--write` only to lower a count for what your task removed.
- From the root, `python3 -I -B tools/citations/citations.py --all` stays at 0.
- The report gives the CPU seconds of the tests you added or changed, from nextest's times.
