+++
id = "connection-states"
part = "forge-states"
kind = "feature"
needs = []
files = ["skein/crates/skein-forge-connection/src/connection.rs", "skein/crates/skein-forge-connection/src/connection/**", "skein/crates/skein-forge-connection/src/component.rs", "skein/crates/skein-forge-connection/src/component/**", "skein/crates/skein-forge-connection/src/tests/connection.rs", "skein/crates/skein-forge-connection/src/tests/dialects.rs", "skein/crates/skein-forge-connection/src/tests/reads.rs", "skein/crates/skein-forge-connection/src/tests/fixtures.rs"]
effort = "high"
minutes = 100
+++

# forge-connection: a connection's phases hold their data

## Goal

`Connection` has a `Copy` `Phase` tag with `draining`, `closing`, `exchanging`, `ended`, `upload`, `receive`, `json_body` and `json_next` beside it, and the phase is assigned at twelve sites. It gets the data-carrying `State` of `skein/docs/design/forge-connection.md`, 8.1. Deadlines stay as they are: `connection-deadlines` takes them next.

Acceptance:

- `Connection` holds `State` as the note sketches it, and those fields are gone. `exchanging` is gone if every read of it can ask the state, and otherwise say what stayed. `Phase` is computed by `phase()`, never stored.
- Each of the twelve assignments (`connection.rs:154, 204, 241, 269, 340, 395, 436, 460, 564, 577, 590, 610`) takes the state's data with it: nothing sets a flag and a state apart.
- The C9 pin is written first and committed apart, passing on the trunk's code. It passes after, and it has been broken by hand. C7, C8, C10 and C11's named tests pass unchanged.
- **`rg -n 'self\.deadline =|c\.deadline =' skein/crates/skein-forge-connection/src`** lists the same sites as before: this task moves no deadline.
- The note's tests pass with no assertion changed. Tests change `.phase` to `.phase()` only (`tests/connection.rs` 10, `tests/dialects.rs` 9, `tests/reads.rs` 3, `tests/fixtures.rs` 1, `connection/tests.rs` 2), and the component's 22 reads the same. No `Phase::` value in a test changes. If a test's events change, stop and ask.
- Net production lines: the cut's estimate is about +60.

## Inputs

- `temper/docs/plan/debt-21/notes/common.md`
- `temper/docs/plan/debt-21/notes/forge-states.md`, binding
- `skein/docs/development/debt/D020-forge-connection-states.md` (its D016 line is stale: that closed, and `Connection` stands on `skein_link::Link`)
- `skein/docs/foundation/programming-model.md`, 5.4
- `skein/docs/design/forge-connection.md`, 8.1

## Owns and may change

- **Owns:** the files in `files`. `connection/github.rs` and `connection/forgejo.rs` are among them.
- **Stay off** `hooks.rs` and `hooks/connection.rs`: `hooked-states` holds them, and may run beside you.
- **The design's table and today's code differ in three places,** and the code stays: connecting and handshaking go through Idle, never straight to Sending; a Closed connection is retired and a new one opened; an answered exchange goes through Idle before the next Sending. `forge-states-close` records them.
- **`Copy` isn't relied on:** `Phase`'s `PartialEq` is.

## Steps

1. Read 8.1, then `connection.rs` whole. The C9 pin; commit it.
2. `State`, `Exchange`, `phase()`; the twelve transitions; the readers' `.phase()`.
3. The break by hand, restored.
4. `cargo fmt`; `(cd skein && python3 -I -B scripts/old_forms.py --write)`; `heavy cargo clippy -p skein-forge-connection --all-targets -- -D warnings`; the note's tests; the citations check. Commit and push.

## Report

The common note's items; the pin and what you broke; what stayed beside the state and why.
