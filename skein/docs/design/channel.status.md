# channel: status

2026-11-03. Where skein's code lags or departs from [channel.md](channel.md).

- **Departs:** 5, 8.3: for opening and closing a channel,
  `skein/docs/foundation/programming-model.md`, 5.4 asks for one explicit
  state enum per lifecycle, each state holding exactly its data, no
  lifecycle in flags or `Option`s beside it, exhaustive matches and one
  handler per cell; `skein-channel/src/machine.rs`'s `Machine` keeps
  eleven bools beside `phase` and `read`, including `finish_sent`,
  `closed_sent`, `cancel_requested` and `cancel_sent`; no reason is
  recorded (the code's side is
  `skein/docs/development/debt/D019-channel-machine-flags.md`).
