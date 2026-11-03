# shell: status

2026-11-03. Where smith's code lags or departs from [shell.md](shell.md).

- **Not yet:**
  - 4, 5.2: the `--account` flag and the `account` setting.
    `smith-local-shell/src/command.rs` parses no `--account`, and
    `settings/schema.rs` has no `account` key, so 4.1's refusal that
    names `--account` points at nothing yet.
  - 5.2, 5.3, 6.2, 7.1: the product still runs on the withdrawn forms
    (protocol/agent.md, section 4). The built-ins hold a third preset,
    `codex` (`smith-local-shell/src/presets.rs`, `Preset::Codex`, merged
    in `settings/layers.rs`): the endpoint `codex` at `chatgpt.com`,
    `dialect = 'codex'`, the account `codex` and the models
    `codex/<name>`; and the `anthropic` preset sets `identity =
    'claude-code'` and an endpoint `account`. Settings take an
    endpoint's `identity`, `account` and `dialect = "codex"`
    (`settings/schema.rs`), so 5.3's refusal of them is not built. An
    `env` account on a Codex endpoint reads its token's
    `chatgpt_account_id` claim (`credentials.rs`, `codex_selector`), and
    the local service sends it with the token in the withdrawn
    id-and-bearer grant (`smith-local-service/src/service/accounts.rs`,
    `GrantValue::Withdrawn`; protocol/channel.md, section 6).
