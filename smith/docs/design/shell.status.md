# shell: status

2026-11-03. Where smith's code lags or departs from [shell.md](shell.md).

- **Not yet:**
  - 2.5: these crates fail the gate's wasm32 clippy, so fab's gate list
    (`.config/gate/wasm32.txt`) leaves them out; each at its first error:
    `smith-protocol-events`: `E0599` at `rustls 0.23.41 src/ticketer.rs:56`;
    `smith-protocol-llm`: `E0599` at `rustls 0.23.41 src/ticketer.rs:56`;
    `smith-protocol-program`: `E0599` at `rustls 0.23.41 src/ticketer.rs:56`;
    `smith-agent-service`: `E0599` at `rustls 0.23.41 src/ticketer.rs:56`;
    `smith-agent-shell`: `E0599` at `rustls 0.23.41 src/ticketer.rs:56`.
  - 4, 5.2: the `--account` flag and the `account` setting.
    `smith-local-shell/src/command.rs` parses no `--account`, and
    `settings/schema.rs` has no `account` key, so 4.1's refusal that
    names `--account` points at nothing yet.
  - 5.2: the `anthropic` preset names an account, its endpoint's
    `account = 'anthropic'` and `[accounts.anthropic]` with `variable =
    'ANTHROPIC_API_KEY'` (`smith-local-shell/src/presets.rs`), where no
    preset names one.
