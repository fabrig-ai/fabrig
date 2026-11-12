# commands: status

2026-11-11. Where the tree's code lags or departs from [commands.md](commands.md).

- **Not yet:**
  - the whole document: the `--account` flag and the `account` setting.
    `smith-local-shell/src/command.rs` parses no `--account`, and
    `settings/schema.rs` has no `account` key, so 1's refusal that
    names `--account` points at nothing yet.
  -  the preset account gap remains `settings.status.md`, 1.2.
- **Departs:**
  -  the portable-config departure remains `README.status.md`, 2.5.
  - 5: `smith login` tells the sign-in deadline from other time-outs
    by the clock alone, because `skein_oauth::Failure::TimedOut` does
    not say which deadline passed: a redirect taken in the deadline's
    last `request_time` (20 s) whose code exchange is not answered
    before the deadline ends with the browser's words, though the
    browser answered.
