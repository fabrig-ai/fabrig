+++
id = "run-decoded"
part = "offered"
kind = "interface"
needs = []
files = ["smith/crates/smith-agent-shell/src/agent_shell.rs", "smith/crates/smith-agent-shell/src/config.rs"]
effort = "medium"
minutes = 15
+++

## Goal

smith's agent kit lets its host hand it an agent document it has already read: `run` splits into its read and the rest, and `config::read_decoded` into its read of the bytes and the decoding. temper needs both next, to put its launcher's path in the document before smith reads it (`temper/docs/design/protocol/setup.md`, 6.6).

Acceptance: `smith_agent_shell::run_decoded` and `smith_agent_shell::config::read_bytes` exist as the note has them; `run` and `read_decoded` behave exactly as before, each refusal's words and diagnostics unchanged; smith's own tests pass.

## Inputs

- `temper/docs/plan/claude-always/notes/common.md`
- `temper/docs/plan/claude-always/notes/contracts.md`, "smith's agent kit"
- `smith/docs/design/protocol/agent.md`, 4

## Owns and may change

- Owns: `smith/crates/smith-agent-shell/src/agent_shell.rs`, `smith/crates/smith-agent-shell/src/config.rs`.
- May add tests in that crate's existing test modules.
- Nothing else. No design or Markdown: `smith/docs/design/**` is read by tests, and this adds no behaviour.

## Steps

1. In `config.rs`, move `read_decoded`'s `skein_shell::read_file` and its refusal words into `pub fn read_bytes(path: &Path) -> Result<Box<[u8]>, String>` (cfg(unix), as `read_decoded` is), and have `read_decoded` call it and `decode`.
2. In `agent_shell.rs`, `run` keeps its read and its diagnostic, then calls `pub fn run_decoded(decoded: config::Decoded, mut errors: Box<dyn Write>) -> Result<Exit, String>`, which holds the rest. The first line of `run` is what moves; the order of what follows does not change.
3. One test that `read_bytes` answers a file's bytes and an absent file's refusal as `read_decoded` words it. No test of `run_decoded` beyond `run`'s own, which now pass through it.
4. Narrow tests, `cargo fmt`, `heavy cargo clippy -p smith-agent-shell --all-targets -- -D warnings`, `heavy cargo nextest run -p smith-agent-shell`, and `heavy cargo check -p temper --tests`. Commit, push.

## Report

As your text says; name any caller that did not keep its behaviour.
