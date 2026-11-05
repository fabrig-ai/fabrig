+++
id = "agent-on-stage"
part = "host-lanes"
kind = "feature"
needs = []
files = ["smith/crates/smith-agent-service/src/service.rs", "smith/crates/smith-agent-service/src/service/up.rs", "smith/crates/smith-agent-service/src/service/stages.rs", "smith/crates/smith-agent-service/src/service/teardown.rs", "smith/crates/smith-agent-service/src/service/tests.rs"]
effort = "high"
minutes = 30
+++

# smith: the agent service drives io through `stage`

## Goal

Part of `smith/docs/development/debt/D003-host-io-route-table.md`. `up.rs`'s hand-driven io loop (ready, completions, fired) becomes: split the kernel's completions by route, then `io::stage::up` on an `io_completions` queue. `io_valid` calls `io::valid` where it rebuilds it by hand, and keeps its own composition checks.

Acceptance:

- `rg -n 'io::resume' smith/crates/smith-agent-service/src/service/up.rs` finds nothing. `rg -n 'operations\(|checked_mul\(2\)' smith/crates/smith-agent-service/src/service.rs` lists nothing of `io_valid`'s.
- `work_pending`'s and `settled`'s equivalents (`stages.rs`, `teardown.rs`) count `io_completions`, since the stage's early return can leave a completion there. A new test pins it: with the ready list still holding work when a completion arrives, the service neither settles nor sleeps before the completion is taken. Break it by hand (skip the queue in the check); it goes red. Restore.
- `service/tests.rs`'s `invalid_limits_are_refused` for `io_valid` gains a `1 << 24` sockets row. Restore a hand-written doubling by hand; the row goes red. Restore.
- `heavy cargo nextest run -p smith-agent-service -p smith-agent-shell -p smith-agent-world -p smith-agent-process-world -p smith-hosts-world -p smith-local-process-world` passes, with no assertion changed. `kernel_submission_credit_holds_both_lower_routes` and `a_one_entry_watermark_keeps_the_host_edge_capped_and_effects_progressing` pin the order.
- The branch's net production lines fall (estimate: -10). `io_resume` falls by one.

## Inputs

- `temper/docs/plan/debt-20/notes/common.md`
- `smith/docs/development/debt/D003-host-io-route-table.md` (stale on one point: its skein side is done)
- `skein/docs/design/io.md`, 2 and 5
- `skein/crates/skein-io/src/stage.rs`
- `smith/docs/design/protocol/limits.md`, 4

## Owns and may change

- **Owns:** the files in `files`.
- **The agent's `operations` table stays:** the item's **Paid when** names `HostIo`, and its `next_operation` seeds `files.seed_randomness`, so removing it could change a file's randomness. `smith/docs/development/debt/D020-hand-set-token-tags.md` (`worlds-on-stage` writes it) keeps it.
- **`io::down` loops stay,** and `stage::down` stays out: `[4, 2, 0]`-style pacing tests pin them.
- **Corners the stage changes,** which no test should see: more than `sockets` entries ready (entities, refusals and a pending `gone` together) finish before completions, as `skein/docs/design/io.md`, 2 says; more than `sockets` timers due fire in the same pass. **If a test's events change, stop and ask.** Don't change an assertion.

## Steps

1. `io_valid`; its row.
2. The split, `io_completions`, `stage::up`; the two checks; the new test; the breaks by hand.
3. `cargo fmt`; `(cd smith && python3 -I -B scripts/old_forms.py --write)`; `heavy cargo clippy -p smith-agent-service --all-targets -- -D warnings`; the tests above. The citations check. Commit and push.

## Report

The common note's items; what you broke and what failed.
