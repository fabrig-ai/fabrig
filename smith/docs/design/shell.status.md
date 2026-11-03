# shell: status

2026-11-03. Where smith's code lags or departs from [shell.md](shell.md).

- **Not yet:**
  - 2.5: `smith-protocol-events`, `smith-protocol-llm`,
    `smith-protocol-program`, `smith-agent-service` and
    `smith-agent-shell` are not in fab's gate list
    (`.config/gate/wasm32.txt`); their wasm32 clippy is unmeasured with
    rustls's `std` native only (skein's `tls.md`, 4), which takes
    rustls's `E0599` at `src/ticketer.rs:56` out of their tree.
  - 4, 5.2: the `--account` flag and the `account` setting.
    `smith-local-shell/src/command.rs` parses no `--account`, and
    `settings/schema.rs` has no `account` key, so 4.1's refusal that
    names `--account` points at nothing yet.
  - 5.2: the `anthropic` preset names an account, its endpoint's
    `account = 'anthropic'` and `[accounts.anthropic]` with `variable =
    'ANTHROPIC_API_KEY'` (`smith-local-shell/src/presets.rs`), where no
    preset names one.
