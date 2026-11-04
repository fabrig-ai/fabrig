# shell: status

2026-11-04. Where smith's code lags or departs from [shell.md](shell.md).

- **Not yet:**
  - 4, 5.2: the `--account` flag and the `account` setting.
    `smith-local-shell/src/command.rs` parses no `--account`, and
    `settings/schema.rs` has no `account` key, so 4.1's refusal that
    names `--account` points at nothing yet.
  - 5.2: the `anthropic` preset names an account, its endpoint's
    `account = 'anthropic'` and `[accounts.anthropic]` with `variable =
    'ANTHROPIC_API_KEY'` (`smith-local-shell/src/presets.rs`), where no
    preset names one.
- **Departs:** 4.5: `smith login` tells the sign-in deadline from
  other time-outs by the clock alone, because
  `skein_oauth::Failure::TimedOut` does not say which deadline passed:
  a redirect taken in the deadline's last `request_time` (20 s) whose
  code exchange is not answered before the deadline ends with the
  browser's words, though the browser answered.
