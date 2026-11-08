+++
id = "jig-test-agent-modes"
part = "jig-test-modes"
kind = "feature"
needs = []
files = [
  "jig/testing/jig-test/src/main.rs",
  "jig/testing/jig-test/tests/process.rs",
  "jig/testing/jig-test/Cargo.toml",
  "Cargo.lock",
  "jig/docs/design/protocol/service.status.md",
]
effort = "high"
minutes = 40
+++

# jig-test serves smith's agent modes

## Goal

`jig-test agent AGENT.json` runs smith's agent shell in its own process (`jig/testing/jig-test/src/lib.rs`, `run`), and its agent configuration takes a `programs` source, so a program-backed agent starts its programs as `jig-test --program-child PARENT PROGRAM ARGS...` and its MCP relay as `jig-test --mcp-relay 127.0.0.1:PORT`. jig-test's `main` dispatches neither; its command parser refuses both. It is the reference binary an application copies for its agent command, and temper copied it without them. First a failing table test that runs the built `jig-test` in smith's agent modes, then both dispatches.

Acceptance:

1. **The table test,** `jig_test_serves_smiths_agent_modes`, in `jig/testing/jig-test/tests/process.rs` (the `jig-test` package's `process` binary, in the focused tier), runs `skein_scratch::paths::binary("jig-test")` once per row of `notes/contracts.md`, 2: `relay usage`, `child refused`, `signed in` (its scripted program's status reads `Status::Usable`), `not signed in`. Each row asserts the exit code and the stdout and stderr bytes exactly, and the program rows `smith_protocol_program::status` too. Its scratch directory is `support::scratch()`.
2. **It fails first:** run on the task's base, before the dispatch, every row fails because jig-test's parser refuses both flags (`jig-test: usage: ...`); the report quotes that failure (`notes/common.md`).
3. **The dispatch,** in `jig/testing/jig-test/src/main.rs`, as `notes/contracts.md`, 1 says: after `skein_fail_stop::install()` and before `jig_test::configure`, the first argument compared with `smith_agent_shell::relay::FLAG` and `smith_agent_shell::PROGRAM_CHILD_FLAG`, returning `smith_agent_shell::relay::main()` and `smith_agent_shell::program_child::main()`; and the four rows pass. `jig_test::configure` and `run` are unchanged.
4. **Status:** delete the **Not yet** entry `7: smith's agent modes` from `jig/docs/design/protocol/service.status.md`; no other design line.

## Inputs

- `temper/docs/plan/temper-program-child/notes/common.md` and `notes/contracts.md` (sections 1 and 2).
- `temper/docs/plan/temper-program-child/inputs/request.md`, item 2 (*any other binary that serves as an agent's relay*).
- `jig/docs/design/protocol/service.md`, 7: *The reference binary, `jig-test`* (its agent modes, written by this cut); `jig/docs/design/protocol/README.md`'s table row *an `agent` command* (section 6's table).
- `smith/docs/design/protocol/programs.md`, 5.1 and 5.3; read only.
- `smith/src/bin/smith-agent.rs` (the model dispatch); `smith/crates/smith-agent-shell/src/program_child.rs`; `smith/crates/smith-agent-shell/src/relay.rs` (`USAGE`, `main`); `smith/crates/smith-protocol-program/src/status.rs`.
- `jig/testing/jig-test/src/main.rs`, `src/lib.rs` (`configure`, `run`); `tests/process.rs` and `tests/support/mod.rs` (`scratch`).

## Owns and may change

Owns `main.rs`'s dispatch and the new test in `tests/process.rs`.

Reader and neighbour files, each edited only so:

- `jig/testing/jig-test/Cargo.toml`: `smith-protocol-program.workspace = true` under `[dev-dependencies]`, for `status`; `Cargo.lock`: the line that dependency adds to `jig-test`'s entry, made by cargo. `smith-agent-shell` is already a dependency.
- `jig/docs/design/protocol/service.status.md`: the one entry of acceptance 4.

Options, each decided:

- **The relay row's stderr differs from the note's line** only because smith's `USAGE` reads otherwise on the trunk: pin what the binary prints and say so in the report.
- **The scripts:** one per program row, written 0700 with `std::fs` (closing the file before the spawn) in the scratch directory; a shared helper in `process.rs` if shorter.
- **`main.rs`'s shape:** `env::args_os().nth(1)` compared with each flag, as `temper/crates/temper/src/main.rs` does for the relay, or one `match`; the report says which.
- **The `process` binary's `#[expect(dead_code)]` on `mod support`** stays as it is; if the new test makes a support item live and the expectation unfulfilled, leave the expectation and use none of support but `scratch`.
- **A lint on the new test** under `jig/tests/clippy.toml`: a scoped `#[expect(..., reason = "...")]` on the test, named in the report.

## Steps

1. Write the test and add the dev-dependency. Run it alone and see it fail (acceptance 2):
   `heavy cargo nextest run -p jig-test --test process -E 'test(=jig_test_serves_smiths_agent_modes)'`.
2. Add the dispatch (acceptance 3); run the same command and see it pass, one test selected.
3. Delete the status entry (acceptance 4).
4. `cargo fmt`; `python3 -I -B jig/scripts/old_forms.py`; `python3 -I -B tools/citations/citations.py --base origin/main`; `heavy cargo clippy -p jig-test --all-targets -- -D warnings`; `heavy cargo nextest run -p jig-test` (its focused tests; the count in the report).
5. Commit (why: the reference binary an application copies serves the modes smith's agent runs its programs through) and push with `git push origin HEAD:refs/heads/<your branch>`.

## Report

fab's implementer report, with: the failure seen in step 1, the passing run's count, the relay row's stderr as pinned, and the choices of the options above.
