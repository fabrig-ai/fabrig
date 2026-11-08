# temper-program-child: temper's binary serves the agent's `--program-child` mode

The request: `inputs/request.md`. smith's agent shell, on Linux, starts every program process through its own executable, `RELAY --program-child PARENT claude ...` (`smith/docs/design/protocol/programs.md`, 5.1). On a temper worker the agent is `temper internal agent AGENT.json`, so the executable is temper's, whose `main` dispatches `--mcp-relay` and not `--program-child`: the flag falls through to the command parser, exits 1, and the status check reads not signed in. `notes/contracts.md` holds the dispatch and the test table both tasks build to; `notes/common.md` the rules.

## Parts and tasks

| Part | Task | Needs | What it makes run | Minutes |
|---|---|---|---|---|
| `temper-modes` | `tasks/temper-child-mode.md` | | a table test of the built `temper` in smith's agent modes, failing first; `main` gives `--program-child` to `smith_agent_shell::program_child::main` beside the relay | 45 |
| `jig-test-modes` | `tasks/jig-test-agent-modes.md` | | the same table on the built `jig-test`, failing first; its `main` gives both `--mcp-relay` and `--program-child` to smith's agent shell | 40 |

**Critical path:** `temper-child-mode`, about 45 implementer-minutes. Both start at once; neither needs the other.

## What was decided

- **The failing test runs the built binary, in temper's focused `suite`.** The real loop hosts `/real-loop/temper` as an in-process stand-in for the relay (`temper/tests/real-loop/src/claude.rs`) and gives its agent no parent, so its programs start directly: no world below the process tier reaches temper's `main`, and a test that doesn't can't fail before the fix. The `temper` package's `suite` binary is in the focused tier and already spawns `skein_scratch::paths::binary("temper")`; running `temper --program-child PID SCRIPT auth status` there with a scripted Claude Code (a shell script in scratch) and reading its exit and stdout through smith's own `smith_protocol_program::status` is what the agent's status check does, and reads `Usable`. No real host or LLM.
- **A new table, not a row of an existing one:** no table test of the binary's agent modes exists in temper or jig (smith's tests can't name either binary). Each task's test is that table, four rows, and a later mode or case is a row. Its `--mcp-relay` row pins the dispatch already there in temper.
- **The other binary is `jig-test`.** The binaries that run smith's agent shell in their own process, and so are their agents' executable, are `smith-agent` (both modes already), `temper` and `jig-test` (`jig/testing/jig-test/src/lib.rs`, `run`). jig-test's agent configuration takes a `programs` source, and its `main` dispatches neither mode. It gets both: without `--mcp-relay` it would serve no program-backed run at all, and it is the reference binary an application copies for its agent command (`jig/docs/design/protocol/README.md`, section 6's table), which is how temper came to lack the mode. The `smith` CLI runs a program-backed agent through `smith-agent` beside it (`smith/crates/smith-local-shell/src/compose.rs`, `placement::beside_current_exe`), and the playground's and worlds' agents are hosted, not binaries.
- **The fix is the request's shape,** a dispatch line in each `main` beside the relay's, after temper's `argv[0]` dispatches (fake git, the Claude Code launcher). A smith function folding both modes into one call would keep a third binary from missing one, but changes smith's interface for a two-line fix the request names; left out.
- **Design written here:** `temper/docs/design/protocol/README.md`, 4 had no line on the binary's agent modes (no `--mcp-relay` either); it gains *smith's agent modes*, beside `temper internal agent`. `jig/docs/design/protocol/service.md`, 7 gains the reference binary's two modes, and `jig/docs/design/protocol/README.md`'s table row for an application's `agent` command names them. A **Not yet** entry in `temper/docs/design/protocol/README.status.md` and in `jig/docs/design/protocol/service.status.md`, each deleted by its task. No smith design changes (`smith/docs/design/**` is read by tests, and 5.1 already says it). No draft lags: the request is the plan's own input.
- **Runs:** focused only. The temper task runs its module and `startup`'s tables beside it, not the whole package (stated in its spec); jig-test's runs its package's focused tests.

No stored format, protocol or public interface changes; each task adds one dev-dependency line (`smith-protocol-program`) and its `Cargo.lock` line.

## Overlaps with drafts beside this one

- **`Cargo.lock`:** both tasks add a dependency line to it, as do other drafts' tasks; cargo regenerates it on a merge.
- No draft beside this one writes `temper/crates/temper/src/main.rs`, `temper/crates/temper/tests/suite/main.rs`, `temper/crates/temper/Cargo.toml`, `jig/testing/jig-test/**`, `temper/docs/design/protocol/README.md` or its status, or `jig/docs/design/protocol/service.md` or its status. `walks-lower/walks-go` writes `temper/docs/design/STATUS.md`; this plan leaves it alone (its index already lists `protocol/README.status.md`).
