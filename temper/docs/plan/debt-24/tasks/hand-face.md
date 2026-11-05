+++
id = "hand-face"
part = "connection-worlds"
kind = "interface"
needs = []
files = ["skein/tests/forge/machine/src/hand.rs", "skein/tests/forge/machine/src/lib.rs", "skein/tests/forge/machine/Cargo.toml", "skein/tests/forge/machine/tests/suite/main.rs", "skein/tests/forge/machine/tests/suite/hand.rs", "skein/tests/forge/machine/tests/suite/hand/**", "skein/crates/skein-forge-connection/src/tests.rs", "skein/crates/skein-forge-connection/src/tests/free_rate.rs", ".config/fixture-readers.toml", "Cargo.lock"]
effort = "high"
minutes = 40
+++

# The machine world gains `hand`, and `free_rate` runs on it

## Goal

The surface skein D030's moves build on: `Harness`, lifted into the machine world, proven on one small file. See `temper/docs/plan/debt-24/notes/hand.md`.

Acceptance:

- `skein/tests/forge/machine/src/hand.rs` holds `Harness`, `CallHeap` and the fixtures `free_rate` and the other moved files read, copied whole from the crate's `src/tests/fixtures.rs`, public where a test reads them. `pub mod hand;` in `lib.rs`.
- `tests/suite/hand.rs`, `hand/forgejo.rs` and `hand/github.rs` are laid out as the note says, so the next two tasks hold different files.
- `free_rate`'s 13 tests that don't seed move to `hand/forgejo/free_rate.rs`. The one that seeds stays in the crate, for `hand-seeded`. Each test's asserts read as they did.
- **A baseline first.** Before any move: `heavy cargo nextest run -p skein-forge-machine-world -p skein-forge-connection`, with its total and slowest times in the report. After the move, the same.
- The crate's `Harness` stays for now: everything else still reads it. `heavy cargo nextest run -p skein-forge-connection --lib` and `-p skein-forge-machine-world` pass; the moved tests count 13 in the world, and the crate 13 fewer.

## Inputs

- `temper/docs/plan/debt-24/notes/common.md`
- `temper/docs/plan/debt-24/notes/hand.md`
- `skein/docs/development/debt/D030-forge-connection-harness-tests.md`
- `skein/docs/design/forge-connection.md`, 10
- `skein/docs/foundation/testing-strategy.md`, 2.1, 2.4

## Owns and may change

- **Owns:** the files in the header. The crate's `src/tests.rs` loses `free_rate`'s test lines, and `free_rate.rs` keeps the one left.
- **`hand.rs`'s `use`s** name `skein_forge_connection::` items. A private item only `Harness`' seeding helpers read isn't copied yet: its helper waits for `connection-seam`, and `free_rate`'s 13 don't need it. If `Harness` itself reads a private item, stop and ask, naming it.
- **`Harness` keeps its name and its fields' names,** since 400 tests spell them.

## Steps

1. The baseline.
2. `hand.rs`; the suite's three files; the world's manifest.
3. The move, by the one-shot script the later tasks reuse; keep the script in your report, as text.
4. `cargo fmt`; `(cd skein && python3 -I -B scripts/old_forms.py)`; `heavy cargo clippy -p skein-forge-machine-world --all-targets -- -D warnings`; the tests above. Commit and push.

## Report

The common note's items; the baseline and after; the assert comparison; the script. A move: no rule to break by hand.
