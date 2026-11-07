+++
id = "host-list"
part = "agent-read"
kind = "feature"
needs = ["agent-list"]
files = ["smith/crates/smith-protocol-program/src/command.rs", "smith/crates/smith-host-protocol/src/list.rs", "smith/crates/smith-host-protocol/src/lib.rs", "smith/crates/smith-host-domain/src/**", "smith/tests/hosts/src/**", "smith/tests/hosts/tests/suite/list.rs", "smith/tests/hosts/tests/suite/main.rs", "smith/docs/design/protocol/hosts.status.md"]
effort = "high"
minutes = 110
+++

# The host's half runs it

## Goal

`smith_host_protocol::ListRead`: the host's half spawns `--list`, writes the ask to its standard input and answers what it read, as `ProgramSource` runs a program's status. jig's worker holds one and the worker's own tokens stay as they are.

Acceptance:

- `contracts.md`, 4's `Command::with_input`, `ListLaunch`, `ListRead`, and `smith_host_domain`'s `ListedModel`, `ListFailure` and `ListOutcome` exist, as `smith/docs/design/protocol/hosts.md`, section 2 says. `ListRead`'s methods are `ProgramSource`'s: `owns`, `from_io`, `fire`, `cancel`, `next_deadline`, `answer`.
- A story in `smith/tests/hosts`, beside `program_source.rs`, hosting `agent-list`'s `Process` in place of `fake-claude`, one table of rows, over the fake's list:
  - rows answered, each field carried;
  - a provider's status answered as `Refused`;
  - the deadline passing while the agent hangs, answered `Deadline`, only after the child has settled;
  - an agent that exits with failure at startup, or with nothing or garbage on standard output, answered `Agent`;
  - `cancel` settling with no answer heard by the host beyond its own.
- The credential's bytes reach the child's standard input alone: `Launch`'s arguments and environment hold none, a row observes both, and `Debug` of a `ListRead` and of a `Command` holds none.
- The credential isn't held past the write of the ask: a `Secret` dropped once written.
- `Command::new`'s behaviour for `auth status` and sign-out is unchanged; `program_source.rs` passes untouched.
- `hosts.status.md`'s section 2 line goes.

## Inputs

- `temper/docs/plan/model-catalogue-3/notes/common.md`.
- `temper/docs/plan/model-catalogue-3/notes/contracts.md`, 4.
- `smith/docs/design/protocol/hosts.md`, section 2; `smith/docs/design/protocol/agent.md`, 6.1; `smith/docs/design/protocol/programs.md`, 4.2.
- `smith/crates/smith-host-protocol/src/program.rs`, the shape to copy.

## Owns and may change

Owns the header's files. `smith-protocol-program/src/command.rs:64-100` is where a stdin pipe is added: today's "Spawn with no stdin pipe" stays what `new` does, and `with_input` adds the pipe and takes a bound for standard output, where `new` has `STATUS_BYTES`.

Options, each pre-approved:

- `environment` is what the host passes for an agent, whole; don't filter it in `ListRead`: the agent's own environment already holds no credential (`hosts.md`, section 3).
- `root` and `directory` are the caller's, as `Launch`'s are.
- If a partial write of standard input needs a retry, `Command` retries it as it drains output; the ask's bytes are a few KiB, smaller than a pipe's buffer, and a short write is still handled.
- `ListFailure::Agent` covers an exit with failure, a signal, garbage output and an output past its bound. Its detail isn't carried.

## Steps

```rust
// smith-host-protocol/src/list.rs
//! Reading an endpoint's list through the agent (protocol/hosts.md, section 2; protocol/agent.md, 6.1).
//! Keeps one child command and a single answer right; holds the credential until it is written.
pub struct ListLaunch { pub program: Box<[u8]>, pub arguments: Box<[Box<[u8]>]>, pub environment: Box<[Box<[u8]>]>, pub root: kernel::Fd, pub directory: Box<[u8]> }
pub struct ListRead { .. }
```

Then `cargo fmt`; `heavy cargo clippy -p smith-protocol-program -p smith-host-protocol -p smith-host-domain -p smith-hosts-world --all-targets -- -D warnings`; `heavy cargo nextest run -p smith-protocol-program -p smith-host-protocol -p smith-host-domain -p smith-hosts-world`; `heavy cargo check -p jig-protocol-worker-agents -p temper-service --tests`. Commit and push.

## Report

As the implementer's text says.
