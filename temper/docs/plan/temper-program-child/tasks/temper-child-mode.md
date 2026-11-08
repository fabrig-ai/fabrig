+++
id = "temper-child-mode"
part = "temper-modes"
kind = "feature"
needs = []
files = [
  "temper/crates/temper/src/main.rs",
  "temper/crates/temper/tests/suite/agent_modes.rs",
  "temper/crates/temper/tests/suite/main.rs",
  "temper/crates/temper/Cargo.toml",
  "Cargo.lock",
  "temper/docs/design/protocol/README.status.md",
]
effort = "high"
minutes = 45
+++

# temper's binary serves the agent's `--program-child` mode

## Goal

On a temper worker the agent is `temper internal agent AGENT.json`, so smith's agent shell starts every Claude Code process, its `auth status` check included, as `temper --program-child PARENT PROGRAM ARGS...`. temper's `main` dispatches `--mcp-relay` but not that flag, which falls through to the binary's command parser and is refused: every program-backed run fails at its start, read as not signed in. First a failing table test that runs the built `temper` binary in smith's agent modes, then the dispatch beside the relay's.

Acceptance:

1. **The table test,** `the_binary_serves_smiths_agent_modes`, in a new module `temper/crates/temper/tests/suite/agent_modes.rs` of the `temper` package's `suite` binary (the focused tier), runs `skein_scratch::paths::binary("temper")` once per row of `notes/contracts.md`, 2: `relay usage`, `child refused`, `signed in` (its scripted program's status reads `Status::Usable`), `not signed in`. Each row asserts the exit code and the stdout and stderr bytes exactly, and the program rows `smith_protocol_program::status` too.
2. **It fails first:** run on the task's base, before the dispatch, `child refused`, `signed in` and `not signed in` fail because the binary's parser refuses `--program-child`; the report quotes that failure (`notes/common.md`).
3. **The dispatch,** in `temper/crates/temper/src/main.rs`, as `notes/contracts.md`, 1 says: after the fake git and the Claude Code launcher (both on `argv[0]`), beside the relay's,
   ```rust
   if env::args_os().nth(1).as_deref() == Some(std::ffi::OsStr::new(smith_agent_shell::PROGRAM_CHILD_FLAG)) {
       return smith_agent_shell::program_child::main();
   }
   ```
   and the four rows pass.
4. **Status:** delete the **Not yet** entry `4: smith's agent modes` from `temper/docs/design/protocol/README.status.md`; no other design line.

## Inputs

- `temper/docs/plan/temper-program-child/notes/common.md` and `notes/contracts.md` (sections 1 and 2: the dispatch and the rows).
- `temper/docs/plan/temper-program-child/inputs/request.md`, items 1 and 2.
- `temper/docs/design/protocol/README.md`, 4: *smith's agent modes* (written by this cut), and *`temper internal agent AGENT.json`*.
- `smith/docs/design/protocol/programs.md`, 5.1 (*Started to die with its agent*) and 5.3 (the relay); read only.
- `smith/src/bin/smith-agent.rs` (the model dispatch); `smith/crates/smith-agent-shell/src/program_child.rs` (the mode and its refusal lines); `smith/crates/smith-agent-shell/src/relay.rs` (`USAGE`, `main`); `smith/crates/smith-protocol-program/src/status.rs` (`status`, `Status`).
- `temper/crates/temper/tests/suite/relay.rs` and `startup.rs` for the suite's idiom: `support::Scratch::new(case)`, `scratch.path()`, `scratch.remove()`, `skein_scratch::paths::binary("temper")`. (`suite/relay.rs` is the GitHub relay, not smith's.)

## Owns and may change

Owns `main.rs`'s dispatch and the new module `tests/suite/agent_modes.rs`.

Reader and neighbour files, each edited only so:

- `temper/crates/temper/tests/suite/main.rs`: the line `mod agent_modes;`, beside `mod relay;`.
- `temper/crates/temper/Cargo.toml`: `smith-protocol-program.workspace = true` under `[dev-dependencies]`, for `status`; `Cargo.lock`: the line that dependency adds to `temper`'s entry, made by cargo.
- `temper/docs/design/protocol/README.status.md`: the one entry of acceptance 4.

Options, each decided:

- **The relay row's stderr differs from the note's line** only because smith's `USAGE` reads otherwise on the trunk: pin what the binary prints and say so in the report.
- **The scripts:** one per program row, written 0700 with `std::fs` (closing the file before the spawn) in the case's scratch directory; if a shared helper in the module is shorter, use it. `std::os::unix::fs::PermissionsExt` for the mode.
- **`main.rs` reads `env::args_os().nth(1)` twice:** keep it beside the relay's line as the model does; folding both flags into one `match` on the first argument is fine if it reads as plainly, and the report says which.
- **A lint on the new test** (`clippy::disallowed_*` under `temper/tests/clippy.toml`): none is expected (`std::process::Command` is in use in `startup.rs`); if one fires, a scoped `#[expect(..., reason = "...")]` on the test, named in the report.

## Steps

1. Write `agent_modes.rs` with its table and the `mod` line; add the dev-dependency. Run it alone and see it fail (acceptance 2):
   `heavy cargo nextest run -p temper --test suite -E 'test(=agent_modes::the_binary_serves_smiths_agent_modes)'`.
2. Add the dispatch (acceptance 3); run the same command and see it pass, one test selected.
3. Delete the status entry (acceptance 4).
4. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `python3 -I -B tools/citations/citations.py --base origin/main`; `heavy cargo clippy -p temper --all-targets -- -D warnings`; `heavy cargo nextest run -p temper --test suite -E 'test(/^agent_modes::/) | test(/^startup::/)'` (the module and the startup tables beside it; the count in the report). A departure from the implementer text's focused tests of the whole package: the dispatch changes only a first argument of `--program-child`, which no other test gives, and the trunk's end check runs the rest.
5. Commit (why: a program-backed run's status check on a temper worker reads its program's real status) and push with `git push origin HEAD:refs/heads/<your branch>`.

## Report

fab's implementer report, with: the failure seen in step 1 (rows and lines), the passing run's count, the relay row's stderr as pinned, and the choices of the options above.
