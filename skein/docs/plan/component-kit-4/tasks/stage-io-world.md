+++
id = "stage-io-world"
part = "stage-sites"
kind = "migration"
needs = ["io-stage", "world-kit", "counts"]
files = ["skein/tests/io/src/**", "skein/tests/io/tests/suite/memory.rs", "skein/tests/io/tests/suite/process_memory.rs"]
effort = "high"
minutes = 50
+++

## Goal

io's own world (tests/io): its owner, its process and usage hosts and its append and output worlds run their io stage through `skein_io::stage`, so every one of them
checks the kernel's submission room before taking work
(`skein/docs/design/draft/component-kit.md, 3.3`; testing-strategy.md, 6),
and their worlds use the world surfaces where they forward or merge by hand
(3.5).

Acceptance:

- Each site below calls `skein_io::stage::up`/`down` instead of its own
  loops, or keeps `io::resume` for a reason `notes/migrations.md`
  allows, named in the report. The `io_resume` count falls by the sites
  moved.
- In the packages you own: `Host` impls whose `next_deadline` hand-merges
  a mechanism deadline with their policy deadline implement
  `next_mechanism_deadline` instead; trivial `Referee` overrides go.
- The tests of every package you own pass, and those of their direct
  dependents within skein check (`heavy cargo check --tests -p ...`).

## Inputs

- `skein/docs/plan/component-kit-4/notes/common.md` and
  `notes/migrations.md`.
- `skein/crates/skein-io/src/stage.rs` (the contract);
  `skein/crates/skein-shell/src/drive.rs` (`Host`, `Forward`);
  `skein/testing/skein-world/src/referee.rs`.
- The sites (`rg -n 'io::resume'` in each path):
  - `skein/tests/io/src/append.rs`, `src/output.rs`, `src/processes.rs`, `src/usage.rs`, `src/world.rs`;
  - `skein/tests/io/tests/suite/memory.rs`, `tests/suite/process_memory.rs` (two).
  - The memory suites meter each entry point apart: they keep `io::resume` where they meter it (notes/migrations.md), named in the report.

## Owns and may change

- Owns `tests/io/src/`, `tests/suite/memory.rs`, `tests/suite/process_memory.rs`. `tests/suite/stage.rs` is task io-stage's test: leave it.
- tests/io's `enum Lifecycle` (`src/processes.rs`: Running, Exited, Closed) is not lib's and stays.

## Steps

1. Move each site (notes/migrations.md, the io stage), the narrowest test
   of its package after each.
2. The world surfaces in the packages you own (notes/migrations.md).

3. `cargo fmt`; `heavy cargo clippy -p skein-io-world --all-targets -- -D warnings`;
   `heavy cargo nextest run -p skein-io-world`; the dependents' checks
   (`cargo tree -i <package> --workspace -e normal,dev --depth 1` finds
   them).
4. `old_forms.py --write`; commit and push.

## Report

As common.md says, with each site: moved, or kept and why.
