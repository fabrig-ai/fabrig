+++
id = "smith-bench-collect"
part = "smith-bench-adapters"
kind = "feature"
needs = ["smith-bench-observing"]
files = [
  "smith/benchmarks/src/run.rs",
  "smith/benchmarks/src/run/**",
  "smith/benchmarks/src/agent/**",
  "smith/docs/development/debt/D010-bench-runner-knows-agents.md",
  "smith/docs/development/debt/README.md",
  "smith/docs/design/benchmarks.status.md",
  "smith/docs/design/STATUS.md",
]
effort = "high"
minutes = 90
+++

# smith D010, the runner: collection, archives and source selection are the adapters' methods

## Goal

`run.rs` and its `run/` modules still branch on the agent: to collect, to archive, to pick a
login and a source arm, to judge startup. Each branch moves behind one method of `Agent`,
`PreparedAgent` or `Observing`, so a new arm is an adapter and no edit here
(`smith/docs/design/benchmarks.md`, 6). The method moves code, and adds no layer. This task pays
`smith/docs/development/debt/D010-bench-runner-knows-agents.md`: it deletes the item.

Acceptance:

- `rg -n 'Smith|Codex|smith|codex|Claude' smith/benchmarks/src/run.rs smith/benchmarks/src/process.rs smith/benchmarks/src/run/`
  reads only these:
  - `run.rs`'s `user.name=Smith benchmark` and `user.email=smith-bench@example.invalid`, the
    seeded repository's author; its `#[serde(rename = "codex_home")]`, a stored name; its
    test's `smith-bench-copied-source` path;
  - `run/outcomes.rs`'s `#[cfg(test)]` module.
- `heavy cargo nextest run -p smith-bench` passes, with its count not below today's, and no test
  assertion changes but for the texts this spec rewords.
- The item's file and its README row are gone, and `python3 -I -B tools/citations/citations.py
  --all` reads 0.
- `smith/docs/design/benchmarks.status.md` has no 6 entry under **Departs**, and its **Not yet**
  6 entry names `benchmarks/src/agent/mod.rs`.
- `git diff --stat` adds at most 60 lines net over `smith/benchmarks/src`: moved code, and one
  line per arm for the methods. Past that, say why in your report.

## Inputs

- `temper/docs/plan/debt-4/notes/common.md`, `AGENTS.md`, `smith/AGENTS.md`.
- `smith/docs/development/debt/D010-bench-runner-knows-agents.md`, whole.
- `smith/docs/design/benchmarks.md`, 6 and 10; `smith/docs/design/benchmarks.status.md`.
- `smith-bench-observing`'s merged `Observing::replay`, `close`, `trace_file`, `writes_outside` and
  `stream_measures`, the form to follow.

## Owns and may change

- Owns: `src/run.rs`, `src/run/**`, `src/agent/**`; the item, its row; the status entries below.
- May change: `tests/suite/**` and `src/tests/**`, for a moved item's path or a reworded text only.
- Status: `smith/docs/design/benchmarks.status.md`, as acceptance says, with the date line today's;
  `smith/docs/design/STATUS.md`, 18: "with three departures". No other design file moves, and no
  test reads a status file (`rg -l 'status\.md' smith/tests smith/benchmarks/tests` lists none).
- Pre-approved: if a method needs a value `Work` holds, make `Work` `pub(crate)` and pass it. If
  a site cannot move without changing the order of `result.artifacts`, keep the order and give
  the adapter a second method. The order of `archive` calls stays as it is today, since
  `result.json` lists them in that order.

## Steps

1. An archive handle, in `run.rs`, replacing `archive`'s two leading arguments at the sites this
   task moves; `archive` itself stays for the runner's own files:

   ```rust
   /// The attempt's archive directory (benchmarks.md, 10).
   #[derive(Clone, Copy, Debug)]
   pub(crate) struct Archive<'a> {
       pub(crate) guard: &'a Guard,
       pub(crate) directory: &'a Path,
   }

   impl Archive<'_> {
       pub(crate) fn put(&self, result: &mut AttemptResult, name: &str, bytes: &[u8]) -> Result<(), String>;
   }
   ```

2. The adapters' surface, in `agent/mod.rs`: each an exhaustive match, one line per arm, the
   bodies in `agent/codex/` and `agent/smith/`.

   ```rust
   impl Agent {
       /// Why this agent cannot run yet, if it cannot.
       #[must_use]
       pub fn refusal(self) -> Option<&'static str>;
       /// The login directory this adapter reads, from the harness's read-only one.
       pub fn home(self, source: &Path) -> Result<PathBuf, String>;
       /// Whether a pinned request's authored arguments are this adapter's to pass on.
       #[must_use]
       pub fn takes_arguments(self, arguments: &[String]) -> bool;
       /// The stream carries the event checks a task may decide on.
       #[must_use]
       pub fn reports_events(self) -> bool;
       /// The arm and version this attempt's executable is, checked against its identity.
       pub(crate) fn source_arm(self, work: Work<'_>, identity: &Identity) -> Result<(ArmRef, String), String>;
       /// Prepare, with the adapter's own setup check where it has one.
       pub(crate) fn prepare_checked(self, context: &Context<'_>, arguments: &[String])
           -> Result<(PreparedAgent, Option<ProcessResult>), Setup>;
   }

   impl PreparedAgent {
       /// Archive what the attempt was prepared with.
       pub(crate) fn archive(&self, archive: Archive<'_>, result: &mut AttemptResult) -> Result<(), String>;
       /// Refuse an identity that differs from what was prepared.
       pub fn check_identity(&self, identity: &Identity) -> Result<(), String>;
   }

   impl Observing {
       /// The stream has ended: faults a closing read finds.
       pub fn finish(&mut self) -> Result<(), String>;
       /// A refusal at startup, read from the stream and standard error.
       pub fn refused(&self, task: &Task, exit: Exit, forced: Forced, stderr: &[u8]) -> Option<End>;
       /// Archive the traces this adapter keeps beside its stream.
       pub(crate) fn archive_traces(&mut self, archive: Archive<'_>, result: &mut AttemptResult,
           process: Traces<'_>) -> Result<(), String>;
       /// The tree has ended: health, this adapter's checks, and its credential proof.
       pub(crate) fn settle(&mut self, archive: Archive<'_>, result: &mut AttemptResult,
           ended: Ended<'_>) -> Result<(), String>;
       /// What the agent persisted (benchmarks.md, 6, collect).
       pub(crate) fn collect(&mut self, archive: Archive<'_>, result: &mut AttemptResult) -> Result<(), String>;
       /// Cache reuse over this attempt's responses.
       pub fn cache_reuse(&self, threshold: f64, miss_limit: u32) -> Measure<bool>;
       /// The check this adapter leaves to a whole triplet, where a cache measure exists.
       pub fn cache_check(&self) -> Option<CheckVerdict>;
       /// The accepted answers come from a program account, not a provider.
       #[must_use]
       pub fn program_source(&self) -> bool;
   }
   ```

   `cache_reuse`'s parameter types are `run/cache.rs`'s; take them from it. `Traces` and `Ended`
   are small `Copy` structs of the borrowed fields a method reads (`trace`, `stdout`, `maximum`;
   `exit`, `forced`, `task`, `workspace`), declared in `agent/mod.rs`, for
   `clippy::too_many_arguments`'s sake; fewer fields is better.

3. What moves where:

   | From | To |
   |---|---|
   | `run.rs`'s `collect_observation`, each arm | `Observing::collect`; the runner's function goes |
   | `observe`'s second `match` on `Observing` | `Observing::settle` |
   | `observe`'s `trace.jsonl` branch, and `run/traces.rs`'s `collect` | `Observing::archive_traces`; `traces.rs` moves to `agent/smith/traces.rs` |
   | `run/startup.rs`, with its tests | `agent/smith/startup.rs`, behind `Observing::finish` and `refused` |
   | `run/cache.rs`'s match on `Observing` | `Observing::cache_reuse`; the file keeps its reading of the task |
   | `observe`'s `Agent::Codex && result.cache.is_some()` | `Observing::cache_check`, pushed when `result.cache` is `Some` |
   | `run/outcomes.rs`'s `agent != Agent::Smith` | `Agent::reports_events` |
   | `run/outcomes.rs`'s `Observing::Smith` with `program_source` | `Observing::program_source` |
   | `archive_preparation` | `PreparedAgent::archive` |
   | `source_arm` | `Agent::source_arm`, with `raw`, `succeeded` and `Work` `pub(crate)` |
   | `invocation`'s `smith_home` | `Agent::home` |
   | `invocation`'s argument check | `Agent::takes_arguments`, the digest of it `!arguments.is_empty() && !takes` |
   | `invocation`'s `match` on the agent to prepare | `Agent::prepare_checked` |
   | `invocation`'s face check | `PreparedAgent::check_identity` |
   | `prepare`'s `Agent::ClaudeCode` check | `Agent::refusal` |

4. The texts left in the runner become neutral, and nothing matches on them
   (`rg` over `tests/` lists none):
   - `run.rs`, 445: "prepared model differs from the authored model override".
   - `run.rs`, 43: "The harness's read-only login; each adapter selects its directory from it."
   - `run.rs`, 931: "the adapter's check refused setup with {:?}".
   - `run.rs`, 1106: "a checked adapter needs its observer".
   - The Claude Code text moves with `Agent::refusal`, as it is.

5. Delete the item and its row; edit the two status files. Run the citations check with `--all`.
6. `cargo fmt`; `python3 -I -B smith/scripts/old_forms.py` from the root; `heavy cargo clippy -p
   smith-bench --all-targets -- -D warnings`; `heavy cargo nextest run -p smith-bench`. One commit
   for the code, one for the debt and status text. Push.

## Report

What the search reads, line by line; the net lines; the structs you added and why; any order you
kept with a second method.
