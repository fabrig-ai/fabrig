# settings: status

2026-11-10. Where the tree's code lags or departs from [settings.md](settings.md).

- **Not yet:**
  - 1.2: the `--account` flag and the `account` setting.
    `smith-local-shell/src/command.rs` parses no `--account`, and
    `settings/schema.rs` has no `account` key, so `commands.md`, 1's refusal that
    names `--account` points at nothing yet.
  - 1.2: the `anthropic` preset names an account, its endpoint's
    `account = 'anthropic'` and `[accounts.anthropic]` with `variable =
    'ANTHROPIC_API_KEY'` (`smith-local-shell/src/presets.rs`), where no
    preset names one.
