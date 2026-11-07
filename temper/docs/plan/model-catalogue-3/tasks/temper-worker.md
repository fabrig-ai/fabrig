+++
id = "temper-worker"
part = "worker-read"
kind = "feature"
needs = ["host-read"]
files = ["temper/crates/temper/src/lib.rs", "temper/crates/temper/src/commands.rs", "temper/crates/temper/src/tests/**", "temper/crates/temper-service/src/program_sources.rs", "temper/crates/temper-service/src/worker.rs", "temper/crates/temper-service/src/worker_route.rs", "temper/crates/temper-service/src/startup/checks.rs", "temper/crates/temper-playground/src/programs.rs", "temper/crates/temper-playground/src/programs/**"]
effort = "medium"
minutes = 80
+++

# temper's worker reads through its own binary

## Goal

temper's worker gives the host half its launch, `temper internal agent-list CONFIG`, and temper's binary runs it. temper-playground hosts the same `Process`, so the simulated worlds can read a list too.

Acceptance:

- `temper internal agent-list CONFIG` parses, as `internal agent` does at `temper/crates/temper/src/lib.rs:132`, and runs `smith_agent_shell::list::run_decoded` with `agent::decoded(&configuration)`. It's hidden as `internal agent` is in `commands.rs:236`, and a row of `commands`' test holds it. `--help`'s reference, if it lists internals, regenerates.
- `temper-service/src/worker.rs` builds `Resources`' launch from `config.agent.program`, the words `internal agent-list` and the same absolute configuration path `config.agent.arguments` holds, with the same environment and root as `process_spawn`'s at `:664`. A worker's `agent.arguments` that aren't temper's own `internal agent PATH` (`startup/checks.rs:171`) give none: its reads answer `Failed(Agent)`.
- `temper-playground/src/programs.rs:143,277`'s spawn recognises the list's arguments and hosts `smith_agent_shell::list::Process`, with the playground's agent's configuration, in place of `Agent`. A row of its existing spawn test holds it.
- A focused test in `temper-service`'s `src/tests/` hears a `Request::Source`'s sibling for a read and sees the launch's arguments with no credential in them.

## Inputs

- `temper/docs/plan/model-catalogue-3/notes/common.md`.
- `temper/docs/plan/model-catalogue-3/notes/contracts.md`, 4.
- `jig/docs/design/domain/hosts.md`, 5.1; `smith/docs/design/protocol/agent.md`, 6.1.

## Owns and may change

Owns the header's files. `temper-service/src/worker.rs:175,664,698,754` are the places: its program, its spawn, its value and its answer.

Options, each pre-approved:

- If `temper-playground`'s `Process` enum needs a fourth arm for `Harness` code, add it; a row of `H::spawned`'s role may reuse `Role::Agent`.
- If a stock test snapshots `commands`' rows, regenerate it.

## Steps

```rust
// temper/crates/temper/src/lib.rs
["internal", "agent-list"] => match arguments {
    [configuration] => Ok(Command::AgentList { configuration: path(configuration).into() }),
    _ => Err(invalid()),
},
```

Then `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p temper -p temper-service -p temper-playground --all-targets -- -D warnings`; `heavy cargo nextest run -p temper-service -p temper-playground`; `heavy cargo nextest run -p temper --lib`. Commit and push. No process tier runs here.

## Report

As the implementer's text says.
