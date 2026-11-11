# agent-binary: `temper-agent`, the deployment's agent (deployment-binaries M4)

`temper/docs/design/draft/deployment-binaries.md` splits the deployment into four binaries. This plan is its M4, the agent (`deployment-binaries.execution.md`, 1): the draft's sections 4 and 9.3, with the acceptance of section 10 that concerns the agent. It writes its own design from the draft as it runs, in the designs the draft names, and lands whole. **Run, unless marked read.**

## What's wanted, ranked. Drop from the bottom if it passes six tasks, and say what you dropped

1. **smith's crate for the agent's document without the agent service** (9.3, third item): the agent-document reader and the program helpers the worker uses today through `smith-agent-shell`, moved into a smith crate that doesn't link smith's agent service, so that a worker can read the document without linking the agent. `smith-agent-shell` re-exports or uses it, so nothing else changes. That's smith's change: its design and status say so.

2. **The fetched Claude Code launcher and its fetch** move out of `temper-forge-setup` into a crate that the worker's `program-login` and the agent's `claude` mode both link (9.3, second item).

3. **A `temper-agent` package** on smith's agent shell and temper's thin agent glue (9.3, first item), holding every mode the agent runs in today: `temper-agent AGENT.json` for one run, its list mode, `--mcp-relay`, `--program-child`, and running as `claude`, the fetched launcher. Plus `--version` and its help. Each mode behaves as it does today from temper's binary (`temper/crates/temper-box-shell/src/binary.rs` and `internal agent`).

4. **Step tests of the package** (section 10): it prints `--version` and its help, it refuses an unknown command, and each mode starts as today's does. A row per mode in one table.

5. **The dependency rule's row for the agent** (sections 4 and 10): the agent's closure holds smith's agent and temper's thin glue alone, by `cargo tree` for its package, in the repository's boundary tests, with any exception named and its reason. If the shared test doesn't exist yet, write the agent's part as its first row.

6. **The design says so:** temper's and smith's designs where the agent's binary and its modes are written, and the draft's execution file's M4 row marked as landed.

## Not in this plan

The worker naming `temper-agent` as its program (M3), the box's `temper` linking it (M5), the release job (M6), and any figure. The box's `temper` keeps every command it has, so the box and the tests are unchanged.

**Tests stay cheap:** a new case of one behaviour is a row of an existing table test. Failing tests first; no test reaches a real host or a real LLM. **A plan runs the focused and fuzzy suites alone and is complete when it lands (`AGENTS.md`, 5).** A stored format or protocol change is a question first. An interface change in skein's `shell` goes to hearth/core first. smith names neither jig nor temper. `walk = false`.
