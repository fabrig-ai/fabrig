+++
id = "paths-bench"
part = "smith-paths"
kind = "migration"
needs = []
files = ["smith/benchmarks/Cargo.toml", "smith/benchmarks/src/lib.rs", "smith/benchmarks/src/fixture.rs", "smith/benchmarks/src/fixture/**", "smith/benchmarks/src/agent/smith/observe.rs", "smith/benchmarks/src/agent/smith/mod.rs", "smith/benchmarks/src/agent/smith/prepare.rs", "smith/benchmarks/tests/common/**", "smith/benchmarks/tests/suite/**"]
effort = "medium"
minutes = 30
+++

# smith: the benchmarks' shared test code has one owner, and their modules an ordinary layout

## Goal

The first of two steps of `smith/docs/development/debt/D015-path-modules.md`, on `smith-bench` alone. Its library compiles three
files of its tests' common code with `#[path]`; they move into the crate behind a `fixture` feature, as smith's other shared test
code does. Every other `#[path]` of the package becomes an ordinary module path.

Acceptance:

- `rg -n '#\s*\[\s*path' smith/benchmarks -g '*.rs'` finds nothing (11 at base), and `smith/benchmarks/tests/common/` is gone.
- `heavy cargo nextest run -p smith-bench` passes, with the same number of tests in the same binaries as at your base
  (`heavy cargo nextest list -p smith-bench`, before and after). No assertion changes.
- `smith/scripts/old-forms.txt`'s `path_attribute` is lowered by what you removed.

## Inputs

- `temper/docs/plan/debt-19/notes/common.md`
- `smith/docs/development/debt/D015-path-modules.md`

## Owns and may change

- **Owns:** the files in `files`.
- **Reader files, spelling only:** the five callers of `crate::request_fixture` under `smith/benchmarks/src/batch/`,
  `crate::json_fixture`'s caller `smith/benchmarks/src/tests/comparison.rs` and `crate::agent::smith::recorded_events`' caller
  `smith/benchmarks/src/process/lower_tests.rs`.
- **Don't change** any `Result` or error type of the harness: `bench-fault` does, after you.
- **If a file you move includes a golden by relative path,** adjust the path; the golden doesn't move.

## Steps

1. **The fixtures.** `smith/benchmarks/Cargo.toml` gains `[features] fixture = []` and the dev-dependency on itself,
   `smith-bench = { path = ".", features = ["fixture"] }` (as `smith-local-protocol`'s manifest has). `src/lib.rs` gains
   `#[cfg(any(test, feature = "fixture"))] pub mod fixture;`. `tests/common/request.rs`, `json.rs` and `events.rs` move to
   `src/fixture/request.rs`, `json.rs` and `events.rs`, reading `crate::` where `request.rs` read `super::bench::`.
   `src/lib.rs`'s `#[cfg(test)] use crate as bench;`, `request_fixture` and `json_fixture` go, as do `observe.rs`'s `test_events` and
   `agent/smith/mod.rs`'s `recorded_events` re-export; their callers read `crate::fixture::...`.
2. **The suite.** `tests/common/` moves to `tests/suite/common/`, whose `mod.rs` re-exports the three by their spelling today
   (`pub use smith_bench::fixture::events;`, `pub use smith_bench::fixture::json;`, `pub use smith_bench::fixture::request::request;`), so the
   suite's 424 `common::` uses don't change. `tests/common/parity.rs` moves to `tests/suite/parity/common.rs`, still `mod common` of
   `parity.rs`.
3. **The plain ones.** These `#[path]`s point where the module's name already leads: remove the attribute, move nothing.
   `tests/suite/smith_adapter.rs` (2), `tests/suite/events.rs` (3), `src/agent/smith/prepare.rs` (1).
4. `cargo fmt`; `python3 -I -B smith/scripts/old_forms.py --write` from `smith/`'s rule (see the common note); `heavy cargo clippy -p smith-bench
   --all-targets -- -D warnings`; `heavy cargo nextest run -p smith-bench`; `heavy cargo check -p smith-agent-process-world --tests`;
   `heavy cargo check -p smith --tests --features end-to-end`. The citations check. Commit and push.

No test pins a new rule, so nothing here needs breaking by hand.

## Report

The common note's items, and the tests' count before and after.
