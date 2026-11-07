+++
id = "scope-input"
part = "scope-input"
kind = "feature"
needs = []
files = [
  "smith/benchmarks/src/run.rs",
  "smith/benchmarks/src/run/passive.rs",
  "smith/benchmarks/src/isolate.rs",
  "smith/benchmarks/src/fixture/request.rs",
  "smith/benchmarks/src/probe.rs",
  "smith/benchmarks/src/probe/command.rs",
  "smith/benchmarks/src/probe/plan.rs",
  "smith/benchmarks/src/probe/plan/tests.rs",
  "smith/benchmarks/src/main.rs",
  "smith/benchmarks/README.md",
  "smith/benchmarks/tests/suite/runner.rs",
  "smith/benchmarks/tests/suite/pipeline.rs",
  "smith/benchmarks/tests/suite/probe_command.rs",
  "smith/docs/design/benchmarks.status.md",
]
effort = "medium"
minutes = 60
+++

# The directory the bench's scope gate lists is an input of the attempt

## Goal

`run::attempt` lists every entry directly under the machine's real `/tmp`, twice per attempt, for the `nothing-outside` check. A builder's `/tmp` held about 20,000 entries, so the tests that reach an attempt cost what the machine has run (`inputs/request.md`, 1). Make the directory an input, as `smith/docs/design/benchmarks.md`, 10, now says: a request's `shared_temporary`, `/tmp` for every real run, a test's own directory in the tests. A real run's behaviour doesn't change.

Acceptance:

1. `run::attempt` lists `prepared.shared_temporary` and no other directory. `isolate.rs` names no `/tmp`.
2. No test in `smith-bench` lists the machine's `/tmp`: the six that reach an attempt are named under `Owns and may change`, and each lists its own directory or none.
3. `runner::a_file_left_in_tmp_by_an_attempt_is_a_write_outside` still proves what it says, against the directory it names: the attempt's file there makes `nothing-outside` observed false, with that file's path in its evidence. The test writes nothing under `/tmp`, and it no longer starts itself as a second process.
4. Tests hold, without listing `/tmp`, that a planned request names `/tmp`, that a document naming none means `/tmp`, and that `suite`, `calibrate` and `fix` refuse `--shared-temporary`.
5. Every other assertion of the six tests stands as it is. A real run's result is byte-for-byte what it was: its evidence still reads `User-owned /tmp entries listed before and after the attempt`, and `User-owned /tmp listing unavailable: ...`. Its archived `prepared.json` gains the one field.
6. `smith/docs/design/benchmarks.status.md` no longer holds the `10, 14` entry under **Not yet**.

## Inputs

- `smith/docs/plan/smith-test-cost/notes/common.md`.
- `smith/docs/plan/smith-test-cost/inputs/request.md`, 1.
- `smith/docs/design/benchmarks.md`, 10 (*Out of sight, nothing shared*), 11.2 and 14 (*An attempt against a replaying agent*).
- `smith/docs/design/benchmarks.status.md`, the `10, 14` entry under **Not yet**.

## Owns and may change

- **Owns:** this spec's `files`.
- **Departs from your text in one way:** you delete the status entry of acceptance 6 and change nothing else under `docs/design/`.
- **Readers, read at the cut; nothing else names what changes:**
  - `run::Request` literals: `src/probe/plan.rs` and `src/fixture/request.rs` take the field. `tests/suite/runner.rs`, `tests/suite/pipeline.rs`, `src/batch/agent_task_tests.rs`, `src/batch/credentials/tests.rs`, `src/batch/native_suite_pin_tests.rs` and `src/batch/cache_order_tests.rs` all spread a fixture request, so they need no edit but where a step below says.
  - `Order` literals: `src/probe/command.rs` (two) and `src/probe/plan/tests.rs` (one).
  - `RequestInputs` literals: `src/probe/plan.rs` and `src/probe/plan/tests.rs`.
  - `temporary_entries`, `temporary_changes` and `passive::temporary`: only `src/run.rs` calls them.
  - `smith/tests/agent-process` depends on this crate for `guard::write_path` alone, so no other package reads what changes.
- **The tests that reach an attempt, read at the cut:** `pipeline::grading_and_archives_keep_agent_commits_and_untracked_files_outside_the_timer` (through `smith-bench attempt`), `pipeline::login_shells_keep_machine_path_in_setup_agent_and_grading`, `pipeline::an_existing_scratch_root_is_refused_and_never_deleted`, `runner::a_file_left_in_tmp_by_an_attempt_is_a_write_outside`, `probe_command::a_replay_arm_runs_through_the_probe_command_and_prints_one_line` (through `smith-bench probe`) and `fault::tests::a_refused_attempt_keeps_the_seed_step`. `probe_command::a_missing_smoke_login_refuses_before_calibration` refuses before planning. The `batch` tests dispatch a fake program, never an attempt.
- **Options, each pre-approved:**
  - `isolated_replay` in `runner.rs` re-runs the test binary with `HOME` and the `XDG_*` variables set. Nothing `run::attempt` reaches reads them: at the cut, `std::env` reads in `src/` sit in `arm.rs`, `guard.rs`, `isolate.rs` (`Machine::from_environment`), `credential/logins.rs`, `batch.rs` and `main.rs`, and `attempt` takes its guard, machine and state as arguments. So remove the helper and its re-execution. If you find a read the attempt does reach, keep the helper and say so in the report.
  - Should `clippy::too_many_arguments` fire on a function you changed, use a scoped `#[expect]` over a new struct.
  - A second `Request` field, a `Machine` field or an environment variable for this is out: the sketch below is the decision.

## Steps

1. **`src/run.rs`:**

   ```rust
   /// The directory the scope gate lists for every real run (benchmarks.md, section 10).
   pub const MACHINE_TEMPORARY: &str = "/tmp";

   fn machine_temporary() -> PathBuf { MACHINE_TEMPORARY.into() }

   pub struct Request {
       // ...
       /// The directory whose user-owned entries the scope gate lists before and
       /// after the attempt: `/tmp` for a real run, and for a document naming none.
       #[serde(default = "machine_temporary")]
       pub shared_temporary: PathBuf,
   }
   ```

   `attempt` passes `&prepared.shared_temporary` to both listings and to `passive::temporary`.

2. **`src/isolate.rs` and `src/run/passive.rs`:**

   ```rust
   pub(crate) fn temporary_entries(shared: &Path, private: &Path) -> Result<BTreeMap<PathBuf, TemporaryEntry>, Fault>
   pub(crate) fn temporary_changes(
       before: Result<BTreeMap<PathBuf, TemporaryEntry>, Fault>,
       shared: &Path,
       private: &Path,
   ) -> Result<Vec<(PathBuf, String)>, Fault>
   pub(super) fn temporary(result: &mut crate::AttemptResult, shared: &Path, changes: Result<Vec<(PathBuf, String)>, Fault>)
   ```

   The two evidence lines format `shared.display()` where they now say `/tmp`.

3. **`src/fixture/request.rs`:** the fixture's value is `"/nonexistent-shared-temporary".into()`, with a line saying why: a test that forgets to name a directory gets an unavailable listing, never the machine's `/tmp`. `fault::tests::a_refused_attempt_keeps_the_seed_step` then needs no edit.

4. **The plan:** `Order` gains `pub shared_temporary: PathBuf`, `RequestInputs` gains `shared_temporary: &'a Path`, and `prepare` gains a last parameter `shared_temporary: &Path`.
   - `task_requests` is at seven parameters. Build `RequestInputs` once in `prepare`, before its loop, drop the `task_requests` wrapper and rename `task_requests_in` to `task_requests`. It takes the inputs, so `plan/tests.rs` keeps calling it.
   - `plan.rs`, beside `RequestInputs`:

     ```rust
     /// The directory a planned request's scope gate lists: `/tmp`, unless `probe` was told another.
     pub(super) fn planned_temporary(named: Option<&str>) -> PathBuf
     ```

   - `command.rs`: `--shared-temporary` joins the `probe` row of `allowed` alone. `command` computes `planned_temporary(options.get("--shared-temporary").copied())` for its `Order`. `suite_command` passes `Path::new(crate::run::MACHINE_TEMPORARY)` to `prepare`. The `Order` at `one` in `comparison` clones its parent's value.
   - Add ` [--shared-temporary DIRECTORY]` to the `probe` line of the usage in `src/main.rs` and of the `text` block in `smith/benchmarks/README.md`.

5. **Tests:**
   - `runner.rs`, `a_file_left_in_tmp_by_an_attempt_is_a_write_outside`: a `skein_scratch::Scratch` root as `isolated_replay` made it, and `root.join("shared-tmp")` created, named in `prepared.shared_temporary`, with the attempt's file inside it (quote the path in the shell line). Keep both assertions, and add that the evidence holds `format!("User-owned {} entries listed before and after the attempt", shared.display())`. The scratch removes the file. Drop `isolated_replay` and its environment variable, per the option above.
   - `pipeline.rs`, `fixture`: `root.join("shared-tmp")` created and named in the `Request`, so the listing still runs in all three stories.
   - `probe_command.rs`, the replay test: create `directory.join("shared-tmp")` after `replay_arm`, and pass `--shared-temporary` with it.
   - `plan/tests.rs`: `Fixture::requests` takes its `shared_temporary` from `planned_temporary(None)`. `supplied_arm_plans_each_attempt_with_pin_and_tier` asserts both `request` and `persisted` name `/tmp`. Add to the same test that the persisted document with that key removed reads as `/tmp`, and add a table test of `planned_temporary` over `None` and a named directory.
   - `probe_command.rs`, new: `only_the_probe_command_names_another_shared_temporary`, in-process over `smith_bench::probe::command` for `suite`, `calibrate` and `fix` with `--shared-temporary`, each refused with `FaultKind::Input` before anything is planned. Read how `command` splits a `suite`'s flags first (`parity::change::Inputs::parse`).
   - The rules that must bite are in the implementer's text: the listing of the named directory, the `/tmp` default, and the refusal.
6. Delete the `10, 14` status entry. Then `cargo fmt`, `python3 -I -B smith/scripts/old_forms.py`, `heavy cargo clippy -p smith-bench --all-targets -- -D warnings`, and `heavy cargo nextest run -p smith-bench`. Check `smith-agent-process-world` compiles only if you changed `guard` (you shouldn't). Commit and push.

## Report

Per the implementer's text, and for each of the six tests: what it lists now, and any reading of its time you took. Name the `isolated_replay` outcome and the rules you broke to see the new tests bite.
