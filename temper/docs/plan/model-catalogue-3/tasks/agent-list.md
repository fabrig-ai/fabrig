+++
id = "agent-list"
part = "agent-read"
kind = "feature"
needs = ["list-exchange"]
files = ["smith/crates/smith-channel/schema/**", "smith/crates/smith-channel/src/**", "smith/crates/smith-channel/golden/**", "smith/crates/smith-agent-shell/src/list.rs", "smith/crates/smith-agent-shell/src/lib.rs", "smith/crates/smith-agent-shell/Cargo.toml", "smith/src/bin/smith-agent.rs", "smith/tests/agent-process/tests/suite/list.rs", "smith/tests/agent-process/tests/suite/main.rs", "smith/tests/agent-process/src/**", "smith/tests/codecs/tests/suite/**", "smith/docs/design/protocol/agent.status.md"]
effort = "high"
minutes = 170
+++

# The agent makes the one request

## Goal

`smith-agent --list CONFIG.json`: a process of its own that reads an ask from standard input, makes the one request and writes the answer to standard output. It follows `relay.rs`, a small process beside the agent, in shape: kernel and hosted glue with its faces handed in, so a simulated world can host it.

Acceptance:

- `contracts.md`, 4's records and `smith_agent_shell::list`'s `FLAG`, `Process` and `run_decoded` exist, as `smith/docs/design/protocol/agent.md`, 6.1 says.
- The records round-trip through `smith-channel`'s codec with goldens at smallest and full. `ListAsk`'s credential is the channel's own grant form, and a `Debug` holds no value. They have no kind and no place in the channel's order: `kinds.rs` is untouched.
- Stories in `smith/tests/agent-process/tests/suite/list.rs`, the hosted process over `fake::peer_for`'s fake, as its siblings are, one table of rows:
  - each route, with a key and, on OpenAI, a bearer, answering rows;
  - a fake's status answering `Refused`; a peer gone answering `Unreachable`; a body too long answering `Unreadable`;
  - an endpoint the configuration doesn't hold, a bearer on Anthropic, and an ask that doesn't read answering `Unreadable` with no request observed;
  - standard input ending with no ask answers `Unreadable`.
- Every row exits 0 and writes one answer, except a configuration refused at startup, which writes its reason to standard error and exits with failure, writing nothing to standard output.
- A list writes no event stream, even where the configuration names one, opens no machine, and starts no run: the row's observation has one `Observation::List` and no call.
- Nothing of the credential's bytes appears on standard error or standard output: a row checks it for a failing request.
- `agent.status.md`'s 6.1 line goes.

## Inputs

- `temper/docs/plan/model-catalogue-3/notes/common.md`.
- `temper/docs/plan/model-catalogue-3/notes/contracts.md`, 2 and 4.
- `smith/docs/design/protocol/agent.md`, 4 and 6.1; `smith/docs/design/protocol/channel.md`, 6.
- `skein/docs/design/llm-connection.md`, 4.3.
- `smith/crates/smith-agent-shell/src/relay.rs`, its `Process`, `Faces` and `main`.

## Owns and may change

Owns the header's files.

- `smith/crates/smith-agent-shell/src/config.rs`: read `Decoded` and its endpoints as they stand. A `pub(crate)` or a getter it needs is allowed; nothing else there changes. The ask's endpoint is the number a charter's LLM names, `smith/docs/design/protocol/charter.md`, section 2, and `Endpoints` in `smith-protocol-llm/src/endpoints.rs:84` shows how that resolves.
- `smith/src/bin/smith-agent.rs`: `smith-agent --list CONFIG.json`, as `relay::FLAG` is handled.
- `smith/crates/smith-agent-shell/src/agent_shell.rs`: a `pub(crate)` it needs for resolving endpoints' names and trust, no change in what it does.

Options, each pre-approved:

- Endpoint resolution and trust loading are `Decoded`'s, and native: reuse them. If they would need the whole service's `prepare`, factor out the one function they need and leave `prepare` calling it.
- Framing: a record's bytes prefixed by its length as `skein-codec` frames, or read to end of stream, whichever `relay.rs`'s stdin shows simplest. Bound the ask at the schema's largest and the answer at 256 of its largest rows.
- The whole deadline: the host stops the process (`hosts.md`, section 2). The agent passes the endpoint's connect and handshake deadlines and none whole, unless the world needs one, then it may take a fixed 60 s, named in the report.
- Where `Process` needs `sockets`, `names` and `tls` in its `io::Limits`, it sizes them for one connection, as `relay.rs`'s `limits` does.

## Steps

```rust
// smith-agent-shell/src/list.rs
//! Kernel and hosted glue for reading a list (smith/docs/design/protocol/agent.md, 6.1).
//! Keeps its I/O queues and one Exchange; holds the credential until the request is written.
pub const FLAG: &str = "--list";
pub struct Faces { pub stdin: Fd, pub stdout: Fd, pub stderr: Option<Fd>, pub signal: Fd }
pub struct Process { .. }
impl Process { pub fn new(decoded: config::Decoded, faces: Faces) -> Result<Self, String>; }
impl Host for Process { .. }
pub fn run_decoded(decoded: config::Decoded, errors: Box<dyn Write>) -> Result<Exit, String>;
pub fn run(path: &Path, errors: Box<dyn Write>) -> Result<Exit, String>;
```

Then `cargo fmt`; `python3 -I -B smith/scripts/old_forms.py` if the tree has one; `heavy cargo clippy -p smith-channel -p smith-agent-shell -p smith -p smith-agent-process-world --all-targets -- -D warnings`; `heavy cargo nextest run -p smith-channel -p smith-agent-shell -p smith-codecs-world`; `heavy cargo nextest run -p smith-agent-process-world -E 'test(/list/)'`; `heavy cargo check -p temper --tests`. Commit and push.

## Report

As the implementer's text says.
