# temper-program-child: temper's binary serves the agent's `--program-child` mode

`subagent-burst` (a79eb23785, `smith: start native agent programs through the child mode`) starts a program-backed agent's Claude Code program, its `auth status` check included, through the agent's own executable in `--program-child` mode (`smith-protocol-program/src/backend/start.rs`, `executable`): `RELAY --program-child PARENT claude ...`, where the relay is the agent's current executable. `smith-agent`'s `main` dispatches that flag to `smith_agent_shell::program_child::main` (`smith/src/bin/smith-agent.rs`), but temper's binary, which is the agent on a temper worker, dispatches only `--mcp-relay` (`temper/crates/temper/src/main.rs`). On a real deployment at `d5d1962c62` every Claude Code run then failed at startup in under a second, `unauthorized: smith: the program source is not signed in through a subscription`, with logins that were usable. **Run, unless marked read.**

## What's wanted, ranked

1. **The failing test first:** at the lowest tier that runs temper's binary as an agent's relay (where its `--mcp-relay` mode is exercised, or a world below the real loop with a scripted program), `temper --program-child PARENT PROGRAM ARGS...` runs `PROGRAM ARGS...` as `smith-agent`'s mode does, passing its exit code and standard output through, so a program-backed run's status check reads `Usable` for a signed-in scripted program.

2. **The fix:** temper's `main` dispatches `smith_agent_shell::PROGRAM_CHILD_FLAG` to `smith_agent_shell::program_child::main()`, beside the relay. Find any other binary that serves as an agent's relay and give it the same mode.

3. **The design says so:** wherever temper's binary's agent modes are listed (`--mcp-relay`), `--program-child` beside it.

## Not in this plan

Any figure, and smith's program start itself.

**Tests stay cheap:** a new case is a row of an existing table test; no test reaches a real host or a real LLM. Failing tests first. **A plan runs the focused and fuzzy suites alone and is complete when it lands (`AGENTS.md`, 5).** `walk = false`.
