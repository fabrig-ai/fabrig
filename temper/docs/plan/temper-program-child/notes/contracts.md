# temper-program-child: contracts

The run's binding calls, for both tasks. Design: `temper/docs/design/protocol/README.md`, 4 (*smith's agent modes*) and `jig/docs/design/protocol/service.md`, 7 (*The reference binary*); smith's side is `smith/docs/design/protocol/programs.md`, 5.1 and 5.3, and is not changed.

## 1. The dispatch

A binary that runs smith's agent shell in its own process (`smith_agent_shell::run`, `run_decoded` or `run_dumping`) is its agents' executable: on Linux the agent starts every program process as `<binary> --program-child PARENT PROGRAM ARGUMENTS...`, and the program starts its MCP relay as `<binary> --mcp-relay 127.0.0.1:PORT`. Such a binary's `main`:

- compares its first argument (`env::args_os().nth(1)`) with smith's two flags, `smith_agent_shell::relay::FLAG` and `smith_agent_shell::PROGRAM_CHILD_FLAG`, before it reads or parses any command;
- returns `smith_agent_shell::relay::main()` for the first and `smith_agent_shell::program_child::main()` for the second, each an `ExitCode`, with nothing printed or read before;
- keeps every dispatch it has on `argv[0]` ahead of them (temper's fake git and its Claude Code launcher).

`smith-agent`'s `main` (`smith/src/bin/smith-agent.rs`) is the model. No smith crate changes, and no new public item: the flags and entry points exist.

## 2. The table of the binary's agent modes

Each binary gets one table test that runs the built binary as a child process, one row per case below. A later mode or case is a row. The rows, the same for both binaries:

| Row | Arguments after the binary | Expected |
|---|---|---|
| `relay usage` | `--mcp-relay` | exit 1; stdout empty; stderr exactly `smith-agent: usage: smith-agent --mcp-relay 127.0.0.1:PORT, its secret in SMITH_MCP_SECRET\n` |
| `child refused` | `--program-child` | exit 1; stdout empty; stderr exactly `smith-agent: program child: missing parent\n` |
| `signed in` | `--program-child PID SCRIPT auth status` | exit 0; stdout exactly the script's line; stderr empty; `smith_protocol_program::status(0, &stdout) == Status::Usable` |
| `not signed in` | `--program-child PID SCRIPT auth status` | exit 1; stdout exactly the script's line; stderr empty; `status(1, &stdout) == Status::NotSignedIn` |

- `PID` is the test process's own ID (`std::process::id()`), the binary's parent: `program_child` execs only while its parent is the one named.
- `SCRIPT` is a scripted Claude Code: a `#!/bin/sh` file the test writes into its scratch directory, mode 0700, one per row. It checks that its arguments are exactly `auth status` (else one stderr line and exit 9), prints one line and exits with the row's code:
  - `signed in`: `{"loggedIn":true,"authMethod":"claude.ai"}`, exit 0;
  - `not signed in`: `{"loggedIn":false}`, exit 1.
- The child runs with a cleared environment (`env_clear`); its stdin a pipe (`Stdio::piped()`) the test closes once the child is spawned, since the relay adopts its stdin as a pipe; its stdout and stderr captured (`Child::wait_with_output`).
- The relay row's stderr is the relay's usage line from `smith_agent_shell::relay` (`USAGE`, private there): if the bytes the binary prints differ from the line above only because smith's line reads otherwise on the trunk, pin smith's line as printed and say so in the report.
- Before the fix the `child refused`, `signed in` and `not signed in` rows fail (the binary's own command parser refuses `--program-child`); that is the failing test the request asks for first.
