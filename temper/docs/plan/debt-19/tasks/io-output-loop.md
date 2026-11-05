+++
id = "io-output-loop"
part = "io-worlds"
kind = "migration"
needs = ["io-world-loop"]
files = ["skein/tests/io/src/output.rs"]
effort = "medium"
minutes = 25
+++

# skein: the output world runs on `skein_world`'s loop

## Goal

The second of three steps of `skein/docs/development/debt/D032-world-harness-copies.md`. `conversation` runs on `skein_world`, as `io-world-loop` left the others, and the item's **Measure** reads 0.

Acceptance:

- `rg -c 'advance_to' skein/tests/io/src | awk -F: '{s+=$2} END{print s}'` reads 0 or nothing (3 at the plan's base).
- The digest of `temper/docs/plan/debt-19/notes/contracts.md`, 2 is the same at your base and at your tip.
- `heavy cargo nextest run -p skein-io-world` passes, with `suite/output.rs` and `fuzzy/output.rs`'s assertions unchanged, and `output::Outcome` keeps its fields.
- Net test lines fall (estimate -10), and no production file changes.

## Inputs

- `temper/docs/plan/debt-19/notes/common.md`
- `temper/docs/plan/debt-19/notes/contracts.md`, 2: binding
- `skein/docs/development/debt/D032-world-harness-copies.md`
- `skein/tests/io/src/world.rs`, as `io-world-loop` left it

## Owns and may change

- **Owns:** `skein/tests/io/src/output.rs`.
- **The referee is bare:** `next_deadline` of `None`, since any wake moves the clock and changes a trace's times. `conversation` has neither a ledger nor a referee today, and gains none.
- **`Driver::iterate` keeps its order,** and its `within`: `io-on-stage` moves those.
- **The done check** comes before the busy check in this loop, and after it in the harness's. The referee's `passed` reads done, so the world ends on the same iteration: the digest shows it. If it doesn't, stop (contracts, 2).
- **The limit of `20_000` iterations** is in `conversation`'s doc comment. The harness's own limit may be another: if a run passes both, nothing changes; keep the doc comment true.

## Steps

1. The probe at your base; `conversation` on the harness; the probe at your tip; `diff`; delete the probe.
2. `cargo fmt`; `python3 -I -B skein/scripts/old_forms.py`; `heavy cargo clippy -p skein-io-world --all-targets -- -D warnings`; `heavy cargo nextest run -p skein-io-world`. The citations check. Commit and push.

No test pins a new rule, so nothing here needs breaking by hand.

## Report

The common note's items, the digests' line count and that they're equal, and the net lines.
