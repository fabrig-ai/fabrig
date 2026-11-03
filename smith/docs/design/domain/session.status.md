# session: status

2026-11-03. Where smith's code lags or departs from [session.md](session.md).

- **Departs:** 1: a session knows a conversation and nothing of runs,
  charters, results or hosts, but `smith-domain-session/src/session.rs`,
  `unrun`, and `session/restore.rs`, `resumed_tail` and `validate_results`,
  match the run's `b"wait"` tool name to resume or restore a held call;
  no reason is recorded (the code's side is
  `smith/docs/development/debt/D004-session-knows-wait.md`).
