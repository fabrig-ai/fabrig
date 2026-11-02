# shell: status

2026-11-02. Where smith's code lags or departs from [shell.md](shell.md).

- **Not yet:** 4, 5.2: the `--account` flag and the `account` setting.
  `smith-local-shell/src/command.rs` parses no `--account`, and
  `settings/schema.rs` has no `account` key, so 4.1's refusal that
  names `--account` points at nothing yet. 5.2: `smith check` gives a
  program account's model left out of the run's models the reason "a
  program runs no sub-agent" (`smith-local-shell/src/settings/resolve.rs`),
  where a sub-agent runs on main's program account.
