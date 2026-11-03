# shell: status

2026-11-03. Where smith's code lags or departs from [shell.md](shell.md).

- **Not yet:**
  - 4, 5.2: the `--account` flag and the `account` setting.
    `smith-local-shell/src/command.rs` parses no `--account`, and
    `settings/schema.rs` has no `account` key, so 4.1's refusal that
    names `--account` points at nothing yet.
  - 5.2, 5.3, 5.4, 6.2, 7.1: the product still runs on the withdrawn
    forms (protocol/agent.md, section 4). The built-ins hold a third
    preset, `codex` (`smith-local-shell/src/presets.rs`, `Preset::Codex`,
    merged in `settings/layers.rs`): the endpoint `codex` at
    `chatgpt.com`, `dialect = 'codex'`, the account `codex` and the
    models `codex/<name>`; and the `anthropic` preset sets `identity =
    'claude-code'`. Settings take an endpoint's `identity`, `dialect =
    "codex"` and the `codex` endpoint (`settings/schema.rs`), so 5.3's
    refusal of them is not built, and the agent's configuration carries
    dialect 1 and the identity (`settings/resolve/document.rs`). An `env`
    account, or a `sign_in` account without a provider, on a Codex
    endpoint reads its token's `chatgpt_account_id` claim
    (`credentials.rs`, `codex_selector`; `credentials/sign_in.rs`;
    `smith-local-service/src/credentials.rs`, `claim_identifier`), and
    the local service sends it with the token in the withdrawn
    id-and-bearer grant (`smith-local-service/src/service/accounts.rs`,
    `GrantValue::Withdrawn`; protocol/channel.md, section 6). An `env`
    account's value is lent as a bearer on any endpoint, so on
    `anthropic` as `Authorization: Bearer`, not in `x-api-key`.
  - 5.2: the `anthropic` preset names an account, its endpoint's
    `account = 'anthropic'` and `[accounts.anthropic]` with `variable =
    'ANTHROPIC_API_KEY'` (`smith-local-shell/src/presets.rs`), where no
    preset names one.
