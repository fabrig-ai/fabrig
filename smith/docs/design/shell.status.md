# shell: status

2026-11-03. Where smith's code lags or departs from [shell.md](shell.md).

- **Not yet:**
  - 2.5: `smith-agent-shell` and what it depends on in smith are not in the gate's
    wasm32 clippy (fab's gate list, `.config/gate/wasm32.txt`).
  - 4, 5.2: the `--account` flag and the `account` setting.
    `smith-local-shell/src/command.rs` parses no `--account`, and
    `settings/schema.rs` has no `account` key, so 4.1's refusal that
    names `--account` points at nothing yet.
  - 5.2: the `anthropic` preset names an account, its endpoint's
    `account = 'anthropic'` and `[accounts.anthropic]` with `variable =
    'ANTHROPIC_API_KEY'` (`smith-local-shell/src/presets.rs`), where no
    preset names one.
