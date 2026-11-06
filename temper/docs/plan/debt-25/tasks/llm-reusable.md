+++
id = "llm-reusable"
part = "spot-review"
kind = "feature"
needs = []
files = ["skein/crates/skein-llm-connection/src/component.rs"]
effort = "medium"
minutes = 20
+++

# skein-llm-connection: `Connection::reusable` in place of three "nothing" lists

## Goal

Finding 3's skein part in `temper/docs/plan/debt-25/inputs/debt23-review.md`, checked against the code. `component.rs` writes out every phase three times to find an `Idle | Ready` connection or an `Idle` one, in `serve_waiting` and the function before it, and `serve_waiting` carries a `too_many_lines` exception for it.

Acceptance:

- `Connection::reusable()` answers `Idle | Ready`, with an exhaustive match of its own. `Connection::idle()` is added only if a site reads `Idle` alone. The three lists read through them.
- `serve_waiting`'s `#[expect(clippy::too_many_lines)]` goes. It's 109 lines at this plan's base, so the lists should bring it under. If it doesn't fit, take the eviction loop out as a function, in the same order. Don't raise clippy's threshold.
- No test changes. Production falls by about 15 lines.

## Inputs

- `temper/docs/plan/debt-25/notes/common.md`
- `temper/docs/plan/debt-25/inputs/debt23-review.md`, finding 3

## Owns and may change

- **Owns:** the file in the header. The `Connection` type's own file, if it lives elsewhere in this crate, for the accessors.
- `call.rs`'s own `too_many_lines` exception stays: the finding doesn't name it.

## Steps

1. The accessors, then the three sites.
2. `cargo fmt`; `(cd skein && python3 -I -B scripts/old_forms.py)`; `heavy cargo clippy -p skein-llm-connection --all-targets -- -D warnings`; `heavy cargo nextest run -p skein-llm-connection --lib`, then `-p skein-llm-connection-world`. Commit and push.

## Report

The common note's items. This is a move: no rule to break by hand.
