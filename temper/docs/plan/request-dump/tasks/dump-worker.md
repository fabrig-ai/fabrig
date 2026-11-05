+++
id = "dump-worker"
part = "dump-worker"
kind = "feature"
needs = ["dump-kit"]
files = [
  "temper/crates/temper/src/lib.rs",
  "temper/crates/temper/src/worker_alone.rs",
  "temper/crates/temper/src/tests.rs",
  "temper/crates/temper-service/src/system.rs",
  "temper/crates/temper-service/src/worker.rs",
  "temper/crates/temper-service/src/tests/worker.rs",
]
effort = "medium"
minutes = 35
+++

# `TEMPER_DUMP_LLM` on a worker reaches each agent and turns the dump on

## Goal

Read `TEMPER_DUMP_LLM` once at a worker's start, as `TEMPER_WORKSPACE_REUSE` is, carry it to each agent process as the one variable in the environment the worker gives it, and there turn on the kit's dump with a bound of 4 GiB. Unset, an agent's environment stays empty and nothing is written.

Acceptance:

1. A worker told a directory launches every agent with `env` exactly `[TEMPER_DUMP_LLM=<directory>]`; its program and arguments are as they were. Told nothing, `env` is empty: the existing test `the_agent_runs_as_temper_agent_with_no_credential_in_its_environment` still passes unchanged.
2. On an engine alone, `Service::dump_llm` does nothing.
3. The value's rule: absent and empty are unset, a relative value becomes absolute, an absolute one stays.
4. The bound passed to `Dumps::new` is exactly 4 GiB, `4_294_967_296`, as one named constant.

## Inputs

- `temper/docs/plan/request-dump/notes/contracts.md`, its `temper` section and the `smith_agent_shell` signatures.
- `temper/docs/plan/request-dump/notes/common.md`.
- `temper/docs/design/draft/dogfood-hacks.md`, 2.10; `temper/docs/design/domain/worker.status.md`, the `TEMPER_DUMP_LLM` lines under **Departs**; `temper/docs/design/domain/worker.md`, 6.

## Owns and may change

**Owns:** the `files` above.

- `temper/crates/temper/src/lib.rs`: beside `workspace_reuse`; at its `if workspace_reuse()` in `prepare_with_limits`; and `Command::Agent`'s arm.
- `temper/crates/temper/src/worker_alone.rs`: at its `if crate::workspace_reuse()`.
- `temper/crates/temper-service/src/system.rs`: beside `reuse_workspaces`.
- `temper/crates/temper-service/src/worker.rs`: one field, set to empty in `new` and used in `process_spawn`, whose `environment: Box::default()` it replaces.

**No other reader changes:** `Worker` is built in one place, and nothing else reads this environment. Commands and the program take their environment from the agent's configuration, not the process's, so the variable reaches neither.

**Options, each with its default:**

- A value `std::path::absolute` refuses counts as unset, with one line on the worker's standard error, in the form its other start lines have.
- Under `temper box`, do nothing special: whether the variable reaches a boxed worker is what its host's environment does. Say what you found in the report.

## Steps

1. In `lib.rs`:

   ```rust
   /// Where a worker's agents keep each model request (docs/design/draft/dogfood-hacks.md, 2.10).
   pub(crate) fn dump_llm() -> Option<PathBuf> {
       dump_directory(std::env::var_os("TEMPER_DUMP_LLM"))
   }
   fn dump_directory(value: Option<OsString>) -> Option<PathBuf> { ... }
   const DUMP_BOUND: u64 = 4_294_967_296;
   ```

   `dump_directory` holds the rule, so that a test needs no process environment.
2. At both starts, beside the reuse branch:

   ```rust
   if let Some(directory) = dump_llm() {
       service.dump_llm(directory.as_os_str().as_bytes());
   }
   ```
3. `temper-service`: `Service::dump_llm` as `reuse_workspaces` is (`if let Some(worker) = ...`), and `Worker::dump_llm`, which sets the one entry with `skein_lib::bytes::concat`.
4. `Command::Agent`: inside the existing `and_then`, with `dump_llm()`: `Some(directory)` calls `smith_agent_shell::run_dumping(decoded, smith_agent_shell::dump::Dumps::new(directory, DUMP_BOUND), Box::new(std::io::stderr()))`; `None` calls `run_decoded` as today.
5. Tests:
   - `temper-service/src/tests/worker.rs`: `a_dump_directory_reaches_the_agent_as_its_one_variable`, after the existing test's shape, with `launched.env` compared whole, and `launched.program` and `launched.args` as there.
   - `temper-service/src/tests/worker.rs`: `a_dump_directory_on_an_engine_alone_does_nothing`, on `fixtures::engine_alone_document()`, after `workspace_reuse_on_an_engine_alone_does_nothing`.
   - `temper/src/tests.rs`: `the_dump_directory_is_set_by_a_value_and_made_absolute`, rows for absent, empty, relative and absolute; and that `DUMP_BOUND` is 4 GiB.
6. Break each new test's rule once by hand and see it fail.
7. `cargo fmt`, `old_forms.py` for temper, `heavy cargo clippy -p temper -p temper-service --all-targets -- -D warnings`.
8. Narrow runs: `heavy cargo nextest run -p temper-service --lib -E 'test(/dump_directory/)'`, `heavy cargo nextest run -p temper --lib -E 'test(/dump_directory/)'`, and the existing `the_agent_runs_as_temper_agent_with_no_credential_in_its_environment`. Then the focused tests of both packages.
9. Push `request-dump/dump-worker`.

## Report

Beside what your text asks: whether the variable reaches a worker under `temper box`, and that no test runs it from a worker through a real agent process to a file, as the entry says. The next comparison's run is that proof.
