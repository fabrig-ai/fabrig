# shell: status

2026-11-03. Where smith's code lags or departs from [shell.md](shell.md).

- **Not yet:**
  - 2.5: fab's gate list (`.config/gate/wasm32.txt`) leaves out
    `smith-protocol-events`, `smith-protocol-llm`,
    `smith-protocol-program`, `smith-agent-service` and
    `smith-agent-shell`, and no listed package reaches them on wasm32,
    so the gate's wasm32 clippy lints none of them. Outside the gate,
    in `temper-playground`'s wasm32 graph, the first four pass it,
    warnings denied, and `smith-agent-shell` fails it: in
    `src/config/`, what only its `cfg(unix)` code reads is unused on
    wasm32: the import `skein_tls as tls` (`native.rs:10`), the
    parameter `errors` (`native.rs:31`), the constant `CERTIFICATES`
    (`native.rs:74`), `TrustDocument`'s fields (`document.rs:185`) and
    those of `EndpointTransport::Tls` (`endpoints.rs:39`).
  - 4, 5.2: the `--account` flag and the `account` setting.
    `smith-local-shell/src/command.rs` parses no `--account`, and
    `settings/schema.rs` has no `account` key, so 4.1's refusal that
    names `--account` points at nothing yet.
  - 5.2: the `anthropic` preset names an account, its endpoint's
    `account = 'anthropic'` and `[accounts.anthropic]` with `variable =
    'ANTHROPIC_API_KEY'` (`smith-local-shell/src/presets.rs`), where no
    preset names one.
