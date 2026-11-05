+++
id = "hand-face"
part = "connection-worlds"
kind = "interface"
needs = []
files = ["skein/tests/forge/machine/src/hand.rs", "skein/tests/forge/machine/src/lib.rs", "skein/tests/forge/machine/Cargo.toml", "skein/tests/forge/machine/tests/suite/main.rs", "skein/tests/forge/machine/tests/suite/hand.rs", "skein/tests/forge/machine/tests/suite/hand/**", "skein/crates/skein-forge-connection/src/tests.rs", "skein/crates/skein-forge-connection/src/tests/free_rate.rs", "skein/crates/skein-forge-connection/src/tests/abort.rs", ".config/fixture-readers.toml", "Cargo.lock"]
effort = "high"
minutes = 40
+++

# The machine world gains `hand`, and `abort` runs on it

## Goal

The surface skein D030's moves build on: `Harness`, lifted into the machine world, proven on one small file, `abort`, and on `free_rate`'s one test that seeds nothing. See `temper/docs/plan/debt-24/notes/hand.md`.

Acceptance:

- `skein/tests/forge/machine/src/hand.rs` holds `Harness`, `CallHeap` and the fixtures `free_rate` and the other moved files read, copied whole from the crate's `src/tests/fixtures.rs`, public where a test reads them. `pub mod hand;` in `lib.rs`.
- `tests/suite/hand.rs`, `hand/forgejo.rs` and `hand/github.rs` are laid out as the note says, so the next two tasks hold different files.
- `abort.rs`'s 5 tests move whole to `hand/forgejo/abort.rs`, and `forgejo_ignores_github_rate_headers` moves to `hand/forgejo/free_rate.rs`. `free_rate`'s other 13 reach `github_calls` and stay in the crate, for `hand-seeded`. Each test's asserts read as they did.
- **A baseline first.** Before any move: `heavy cargo nextest run -p skein-forge-machine-world -p skein-forge-connection`, with its total and slowest times in the report. After the move, the same.
- The crate's `Harness` stays for now: everything else still reads it. `heavy cargo nextest run -p skein-forge-connection --lib` and `-p skein-forge-machine-world` pass; the moved tests count 6 in the world, and the crate 6 fewer.

## Inputs

- `temper/docs/plan/debt-24/notes/common.md`
- `temper/docs/plan/debt-24/notes/hand.md`
- `skein/docs/development/debt/D030-forge-connection-harness-tests.md`
- `skein/docs/design/forge-connection.md`, 10
- `skein/docs/foundation/testing-strategy.md`, 2.1, 2.4

## Owns and may change

- **Owns:** the files in the header. The crate's `src/tests.rs` loses `abort`'s `mod` line, and `free_rate.rs` keeps its 13 and the helpers they read.
- **`hand.rs`'s `use`s** name `skein_forge_connection::` items. A private item only `Harness`' seeding helpers read isn't copied yet: `github_calls`, `github_account` and `github_repository` wait for `connection-seam`, and this task's 6 don't need them. If `Harness` itself reads a private item, stop and ask, naming it.
- **`Harness` keeps its name and its fields' names,** since 400 tests spell them.

## Steps

1. The baseline.
2. `hand.rs`; the suite's three files; the world's manifest.
3. The move, by the one-shot script the later tasks reuse; keep the script in your report, as text.
4. `cargo fmt`; `(cd skein && python3 -I -B scripts/old_forms.py)`; `heavy cargo clippy -p skein-forge-machine-world --all-targets -- -D warnings`; the tests above. Commit and push.

## Report

The common note's items; the baseline and after; the assert comparison; the script. A move: no rule to break by hand.

## Answers

### 2026-11-05: Run connection-seam first and copy the seeding fixtures through it, or move only the one Forgejo test? (debt-24/hand-face-1, foreman debt-24/hand-face/foreman-1)

Neither reorder: `connection-seam` stays where it is, and this task moves what reaches no private item.
The run is right: 12 of the 13 reach `github_calls`, which reads `test_calls_ready` and `test_github_mut`, and `a_304_counts_free_and_cost_before_plan_refusal` has GitHub rows, so only `forgejo_ignores_github_rate_headers` can move now. `free_rate`'s 13 others go to `hand-seeded`, in `hand/forgejo/free_rate.rs`, the one file.
So that the surface and the script are proven on a whole file, this task also takes `abort.rs` (5 tests, public reach, read from `hand-forgejo`): the script's `git mv` runs on it. Acceptance above says 6. The baseline in `hand-face-1`'s report stands if the base hasn't moved; re-measure otherwise.
