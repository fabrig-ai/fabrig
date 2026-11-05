+++
id = "crate-tests-move"
part = "crate-tests-move"
kind = "feature"
needs = ["codec-once"]
files = ["skein/crates/skein-forge/tests/**", "skein/crates/skein-forge/Cargo.toml", "skein/crates/skein-fail-stop/tests/**", "skein/crates/skein-fail-stop/src/bin/**", "skein/crates/skein-fail-stop/Cargo.toml", "skein/crates/skein-fake-forge-forgejo/tests/**", "skein/crates/skein-fake-forge-forgejo/Cargo.toml", "skein/tests/forge/dialect/**", "skein/tests/fail-stop/**", "skein/tests/fake-forge/Cargo.toml", "skein/tests/fake-forge/tests/suite/main.rs", "skein/tests/fake-forge/tests/suite/forgejo_memory.rs", "Cargo.toml", "Cargo.lock", ".config/fixture-readers.toml", "skein/docs/design/testing.status.md", "skein/docs/development/debt/D026-crate-tests-directories.md", "skein/docs/development/debt/D028-worlds-as-kits.md"]
effort = "medium"
minutes = 50
+++

# skein: no crate keeps a `tests/` directory

## Goal

Pay `skein/docs/development/debt/D026-crate-tests-directories.md`: what drove three crates from outside moves under
`skein/tests/`, and skein-forge's suite stops compiling under step-crate lints.

A move, with `git mv` so that history follows. No assertion changes.

Acceptance:

- `ls -d skein/crates/*/tests 2>/dev/null | wc -l` reads 0.
- The counts of `#[test]`s are the same before and after: 170 in skein-forge's suite at this plan's base. Your report gives each
  suite's count, measured by name in `cargo nextest list` before and after.
- `rg -c '#!\[expect' skein/tests/forge/dialect` falls from 65 to what clippy still asks for under `harness`, each with its
  reason.
- `[dev-dependencies]` of `skein-forge` and `skein-fake-forge-forgejo` hold nothing the crates' own `src/` doesn't use.
- `tests/boundary`'s `fail_stop` and `direction` tests pass, and so does the gate's lint check of the new crates' `[lints]`.
- The item is deleted, and `testing.status.md`'s `6:` departure about these three leaves. Its other `6:` entries stay.

## Inputs

- `temper/docs/plan/debt-17/notes/common.md`
- `skein/docs/development/debt/D026-crate-tests-directories.md`: its counts are older; this spec's stand.
- `skein/docs/design/testing.md`, 6 (this cut wrote the new rows) and 7's budget for `tests/forge`
- `tools/lints/design.md`, 2.2

## Owns and may change

- **Owns:** the files in this spec's header.
- **Names:** `skein/tests/forge/dialect`, package `skein-forge-dialect-tests`; `skein/tests/fail-stop`, package
  `skein-fail-stop-tests`. A package has to start `skein-`, or `.config/nextest.toml`'s `package(/^skein-/)` misses it.
- **A new crate needs no `src/lib.rs`:** `skein/tests/fake-forge` has only its `tests/`. `skein-fail-stop-tests` has the fixture's
  `[[bin]]` as well.
- **Two `#[path]`s stay, each saying why in a comment beside it,** reaching a crate's own test fixtures, which its unit tests
  share:
  - `skein/tests/forge/dialect/tests/common/mod.rs`, to `skein/crates/skein-forge/src/forgejo/tests/fixtures/admitted.rs`;
  - `skein/tests/fake-forge/tests/suite/forgejo_memory.rs`, to `skein/crates/skein-fake-forge-forgejo/src/tests/fixtures.rs`,
    with its `dead_code` expect.

  They point from a test crate into a step crate, the way dependencies go. Neither file holds lint expects a second regime would
  trip, but for that one.
- **`skein/docs/development/debt/D028-worlds-as-kits.md`:** after `codec-once`, its `#[path]` sentence reads:

  ```markdown
  - Three `#[path]`s cross a crate's root:
    `skein/tests/fake-peers/tests/suite/github_memory.rs` compiles a file
    of `skein/tests/fake-forge`, and `skein/tests/forge/dialect/tests/common/mod.rs`
    and `skein/tests/fake-forge/tests/suite/forgejo_memory.rs` compile fixtures
    their crates' unit tests share. The review counted 10; most are gone.
  ```

- **`.config/fixture-readers.toml`:** the `skein-forge/src/forgejo/tests/fixtures/*.rs` entry gets `default` and `fuzzy` reader
  rows for `skein-forge-dialect-tests`, and its comment names the new path. Add what its checker asks for and nothing else.
- **Stay as they are:** `debt/reviews/*`, which are dated records.
- **Budget:** `testing.md`, 7 keeps `tests/forge` within half of each suite's. These tests stay in the suite, so the tree's total
  doesn't move, but they join `tests/forge`'s share. Measure the moved suite's CPU in your run. If it takes `tests/forge` past
  half, this is over budget: take the measurement back as a question, and don't move a test to escape it.

## Steps

1. ```
   git mv skein/crates/skein-forge/tests skein/tests/forge/dialect/tests
   git mv skein/crates/skein-fail-stop/tests skein/tests/fail-stop/tests
   git mv skein/crates/skein-fail-stop/src/bin skein/tests/fail-stop/src/bin
   git mv skein/crates/skein-fake-forge-forgejo/tests/suite/memory.rs skein/tests/fake-forge/tests/suite/forgejo_memory.rs
   ```

   Make the parent directories first. `skein-fake-forge-forgejo/tests/suite/main.rs` goes: its `use` of the crate's names, which
   `fixtures.rs` reaches through `crate::`, and its `extern crate alloc` go to `skein/tests/fake-forge/tests/suite/main.rs`, with
   `mod forgejo_memory;`. Its allocator is the one there already (`skein_world::domain::heap` re-exports `skein_heap`). If
   the names clash with that suite's, nest them in a module and say so.

2. Manifests: the two crates in the root `Cargo.toml`'s `members`, one line each, beside their neighbours;
   `skein-fake-forge-tests` gains `skein-fake-forge-forgejo`. `skein-fail-stop` loses its `[[bin]]`, since `CARGO_BIN_EXE_` reaches
   only a package's own tests. `skein-forge-dialect-tests` takes `skein-forge`, `skein-json`, `skein-lib` and `skein-scratch`.
   `python3 -I -B tools/lints/lints.py --set harness skein/tests/forge/dialect skein/tests/fail-stop`. `Cargo.lock` moves by these
   packages.

3. Delete the file-level `#![expect(...)]`s that no longer fire, since an unfulfilled one is a warning; let `heavy cargo clippy` say
   which. About 47 of the 65 go. `#![cfg(test)]` stays in each `main.rs`, because `harness` denies `unwrap_used` outside test code.
   The fixture binary's `#[expect(clippy::disallowed_methods)]` stays: `Builder::spawn` is banned there.

4. `heavy cargo nextest run -p skein-forge-dialect-tests -p skein-fail-stop-tests -p skein-fake-forge-tests -p skein-forge -p
   skein-fake-forge-forgejo -p skein-fail-stop`, then the root boundary package's `fail_stop` and `direction` tests, named by
   `tests/boundary/Cargo.toml`.

5. The status line and `D028`, then delete the item. `cargo fmt`; `python3 -I -B skein/scripts/old_forms.py`; `heavy cargo clippy` naming
   those packages, `--all-targets -- -D warnings`; `python3 -I -B tools/lints/lints.py`, which checks; the citations check.
   Commit and push.

## Report

The common note's items, the counts, the moved suite's CPU, and the expects that stayed. A move: nothing to break by hand.
