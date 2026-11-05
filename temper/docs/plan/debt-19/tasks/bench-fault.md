+++
id = "bench-fault"
part = "bench-fault"
kind = "feature"
needs = ["paths-bench"]
files = ["smith/benchmarks/src/**", "smith/benchmarks/tests/**", "smith/docs/development/debt/D011-bench-string-errors.md"]
effort = "high"
minutes = 150
+++

# smith: the benchmark harness carries one typed fault

## Goal

Pay `smith/docs/development/debt/D011-bench-string-errors.md`. The harness returns `Fault`
(`temper/docs/plan/debt-19/notes/contracts.md`, 1) where it returns `String`, `Setup` or `HarnessFault` today; `?` replaces
`.map_err(|error| error.to_string())`; immediately called closures and the step variables in `run.rs` become functions. Mechanical once the
kinds are chosen, and they are chosen. Result files, summaries and printed text stay the same, byte for byte.

Acceptance:

- `rg -n 'Result<.*, String>|Ok::<.*, String>' smith/benchmarks/src` lists only the edges (at most 12 lines): a function that parses a process's command line
  (`src/main.rs`, `Inputs::parse`, `probe::command`'s option parsing) or renders a report's text. Your report lists each kept line and why. The item's own `Measure` under-counts
  (246 by this search at base, not 205), so this search is the check. `probe::command` itself returns a `Fault`: only what parses arguments is an edge.
- `rg -n 'map_err\(\|error\| error\.to_string\(\)\)' smith/benchmarks/src` finds nothing (184 at base).
- `rg -n '\bHarnessFault\b|\bisolate::Setup\b|\bSetup \{' smith/benchmarks smith/tests -g '*.rs'` finds nothing.
- `rg -n '\}\)\(\)' smith/benchmarks/src` finds nothing (20 at base), nor does `smith/benchmarks/src/run.rs` assign a step variable.
- `heavy cargo nextest run -p smith-bench` passes. Recorded results and summaries under `smith/benchmarks/tests/recorded/` and `smith/benchmarks/summaries/` don't change.
- New tests in `src/fault.rs` pin: a missing file reads as `Io`; a malformed result reads as `Format`; a copied arm's digest mismatch reads as `Mismatch`; a refused seed reads as
  `Setup(SetupStep::Seed)`, and through `run::attempt`'s path a refused attempt names the same step in `Incomplete::Setup` as at base; `Display` is the detail alone.
- By hand: remap `From<std::io::Error>` to another kind (the first test goes red); label a step of `run.rs`'s preparation with another `SetupStep` (a test goes red). Restore each.
- Production lines fall: the branch's net production is at most 0 (estimate: -150, give or take 150; rustfmt decides). The item is deleted.

## Inputs

- `temper/docs/plan/debt-19/notes/common.md`
- `temper/docs/plan/debt-19/notes/contracts.md`, 1: binding
- `smith/docs/development/debt/D011-bench-string-errors.md`
- `smith/docs/design/benchmarks.md`, 8.3 (the ends, which don't change)

## Owns and may change

- **Owns:** the files in `files`. Tests in `smith/benchmarks/` may change how they reach an error (`.expect_err(..).detail.contains(..)`, an `Err(Fault::new(..))`): what each asserts stays,
  and a test that asserted the words still asserts them.
- **Reader files:** `smith/tests/end-to-end/**` and `smith/tests/agent-process/**` read `smith_bench`: call sites only, if the compiler asks.
- **If `Refusal` proves in the way at a site,** convert with its `From`; don't change `Refusal`.
- **If a kind fits no site,** use the nearest, and name the site in your report. A new `FaultKind` variant is a question.
- **If a closure attaches a cleanup** (`arm.rs`'s `archive`, `isolate.rs`'s `prepare`), a named function holds the work and its caller the cleanup, so the cleanup still runs on every error path it runs on today.
- **The `Setup` label:** `.map_err(Fault::at(SetupStep::Seed))?` or the like, at each call in `run.rs`'s preparation that today runs with `step` holding that value. A fault that already has a `Setup(step)`
  kind keeps it (`step = refusal.step`, `step = error.step` today). A reader of `error.reason` reads `error.detail`.

## Steps

1. `src/fault.rs` with its tests. `pub mod fault` and `pub use fault::{Fault, FaultKind};` in `src/lib.rs`.
2. From the leaves up, a commit per family as it compiles: `process`, `guard`, `isolate`, `credential`, `task`, `formats`, `tokens`, `stats`, `result`, `summary`, `grade`; `agent/*`; `arm`, `run`, `batch`, `fix`, `probe`, `parity`, `check`, `validate`, `suite`; `main`.
3. `End::HarnessError { what }` and `End::Refused { setup }` take the fault's `detail`, and every string that reaches a result, a report or stderr is the one it replaces. A `format!("cannot ...: {error}")` keeps its words.
4. The closures, the `step` variables.
5. Break the rules by hand, as the acceptance says.
6. Delete the item. `cargo fmt`; `python3 -I -B smith/scripts/old_forms.py`; `heavy cargo clippy -p smith-bench --all-targets -- -D warnings`; `heavy cargo nextest run -p smith-bench`; `heavy cargo check -p smith-agent-process-world --tests`;
   `heavy cargo check -p smith --tests --features end-to-end`. The citations check. Commit and push.

## Report

The common note's items, the lines the first search still lists, the sites where a kind was a judgement call, what you broke and what failed.
