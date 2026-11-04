+++
id = "smith-bench-observing"
part = "smith-bench-adapters"
kind = "feature"
needs = []
files = [
  "smith/benchmarks/src/process.rs",
  "smith/benchmarks/src/agent/mod.rs",
  "smith/benchmarks/src/agent/codex.rs",
  "smith/benchmarks/src/agent/codex/prepare.rs",
  "smith/benchmarks/src/run.rs",
]
effort = "high"
minutes = 70
+++

# smith D010, the supervisor: a replay is a Codex observation, and `process.rs` names no agent

## Goal

Every process carries a `codex::Observer`, built for smith runs too, and `Capture::Codex` is
reached in production only for offline replays. A replay becomes a Codex observation, so
`Capture` has one way to read an agent, `ProcessResult.observer` and `Capture::Codex` go, and
`process.rs` asks the adapter what it asked by matching on it. This is the first of two tasks on
`smith/docs/development/debt/D010-bench-runner-knows-agents.md`; `smith-bench-collect` follows
with the runner's collection and archives. The item stays open: leave its file.

Acceptance:

- `Capture` is `Raw` and `Agent(Box<Observing>)`; `ProcessResult` and `Output` hold no
  `codex::Observer`.
- `rg -n 'Smith|Codex|smith|codex|Claude' smith/benchmarks/src/process.rs` reads nothing;
  `src/process/lower_tests.rs` keeps its fixtures.
- In `run.rs`, `process.observer` reads nothing and `Capture::Codex` reads nothing.
- `heavy cargo nextest run -p smith-bench` passes, with its count not below today's. A test's
  assertions change only where it read `result.observer` or passed `Capture::Codex`.
- No file under `smith/benchmarks/tests/recorded/`, `smith/benchmarks/baselines/` or a result
  schema changes: a replay's `result.json` holds what it held.

## Inputs

- `temper/docs/plan/debt-4/notes/common.md`, `AGENTS.md`, `smith/AGENTS.md`.
- `smith/docs/development/debt/D010-bench-runner-knows-agents.md`, whole.
- `smith/docs/design/benchmarks.md`, 6 (adapters, their five steps) and 8.3. They need no change.

## Owns and may change

- Owns: `src/process.rs`, `src/agent/mod.rs`, `src/agent/codex.rs`, `src/agent/codex/prepare.rs`,
  and in `src/run.rs` the reads below. `smith-bench` is this task's alone in this wave.
- May change, each only for the fields and variants that go: `src/run/passive.rs` (the fixture's
  `observer` line), `src/process/lower_tests.rs`, `tests/suite/runner.rs`,
  `tests/suite/codex_adapter.rs`, `src/tests.rs`, `src/tests/**`.
- Not this task's: `collect_observation`, the archives, `source_arm`, `invocation`'s agent
  branches, `run/cache.rs`, `run/startup.rs`, `run/traces.rs`, `run/outcomes.rs`: they keep their
  `Observing` matches, and `smith-bench-collect` moves them.
- Pre-approved: if a recorded test needs a replay's `answered` to fall on the line it falls on
  today and `Observing::feed_line` puts it on another, keep the test's assertion and make the
  replay's `feed_line` report `Seen::Answer` there, saying so in your report.

## Steps

1. `codex::Observing` (`agent/codex/prepare.rs`) holds what has been prepared, if anything:

   ```rust
   pub struct Observing {
       /// None for an offline stream replay, which has nothing to collect.
       pub prepared: Option<Box<PreparedCodex>>,
       pub observer: Observer,
       pub collected: Option<super::Rollouts>,
       pub collection_error: Option<String>,
   }
   ```

   `collect` on a replay returns a `HarnessFault` naming the absent preparation; nothing calls it
   there, since step 3's guard stands before it.

2. In `agent/mod.rs`, `Observing` gains what the supervisor and the runner read, each an
   exhaustive match with one line per arm:

   ```rust
   impl Observing {
       /// An offline stream replay: Codex's JSON lines, with no preparation behind them.
       #[must_use]
       pub fn replay() -> Observing;
       /// When the person's input closes unless the task says.
       #[must_use]
       pub fn close(&self) -> Close;
       /// The native trace followed as the process writes it, opened.
       pub fn trace_file(&self) -> Result<Option<std::fs::File>, String>;
       /// The paths written outside the workspace, as far as this stream shows them.
       #[must_use]
       pub fn writes_outside(&self, workspace: &Path) -> crate::Measure<Vec<PathBuf>>;
       /// The stream's own counts and tokens, before anything is collected.
       pub fn stream_measures(&self, result: &mut crate::AttemptResult) -> Result<(), String>;
   }
   ```

   `close` holds what `run.rs`'s `script` and `process.rs`'s `execute` both spell: `AfterAnswer`
   for a smith face other than `Exec`, otherwise `AfterPrompt`; `script` and `execute` call it, and
   `Capture::Raw` stays `AfterPrompt` at the call. `trace_file` holds `process.rs`'s `trace_file`,
   whose body moves, and `process.rs` keeps only the `Raw` arm. `writes_outside` takes `run.rs`'s
   `scope` and `lexical`, which move to `agent/codex/prepare.rs`; for smith it is
   `Measure::unavailable("event stream does not expose every shell filesystem effect")`.
   `stream_measures` sets `result.counts` and `result.tokens` as `observe` sets them from
   `process.observer`; for smith it sets what a default observer gave, which `collect_observation`
   then overwrites as it does today.

3. In `process.rs`, `Capture::Codex`, `Output.observer`, `ProcessResult.observer` and the
   `use crate::agent::codex` line go; the module doc says "parsed agent records". `Output::line`'s
   `Capture::Agent` arm is the only reading arm. `execute`'s tail hands `output.capture`'s
   observation over with no match on the agent.

4. In `run.rs`:
   - `invocation`'s offline arm passes `Capture::Agent(Box::new(Observing::replay()))`.
   - `observe` reads `process.observing` for its classification, counts, tokens and scope. It is
     `Some` on every path `attempt` takes: say so with `.ok_or("an attempt has its observer")?`
     at the top of `observe`. `settled_outcome`'s answer is
     `process.observing.as_ref().and_then(|observing| observing.answer())`.
   - In `collect_observation`'s Codex arm, the whole arm runs only with `observing.prepared` set,
     so a replay leaves `result.delegation` as today; the arm at `observe` that takes
     `prepared.borrowed` does the same.
   - In `run/cache.rs`, a replay answers
     `Measure::unavailable("cache probe requires its selected adapter")`, today's text for it: one
     added arm or guard, nothing else there.

5. `tests/suite/runner.rs`: `Capture::Codex` becomes `Capture::Agent(Box::new(Observing::replay()))`
   and `result.observer.classify(..)` reads `result.observing`, through one local helper. Each test
   keeps its other assertions. `src/run/passive.rs`'s fixture loses its `observer` line.

6. `cargo fmt`, `python3 -I -B smith/scripts/old_forms.py` from the root, `heavy cargo clippy -p
   smith-bench --all-targets -- -D warnings`, `heavy cargo nextest run -p smith-bench`. Commit
   and push.

## Report

The tests that changed and how; any replay field that moved, which should be none; the calls you
made inside the sketches.
