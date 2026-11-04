# examples: status

2026-11-03. Where skein's code lags or departs from [examples.md](examples.md).

- **Not yet:** 6: the echo's referee reads the echo's address from its
  state (`tests/echo/src/referee.rs`), not from its startup fact; the
  end-to-end kit starts its binary in a cgroup through the shell
  (`start_binary_in_cgroup`), not as an io tree.
- **Departs:** 3.4: `skein/docs/foundation/programming-model.md`, 5.4
  asks for one explicit state enum per lifecycle, each state holding
  exactly its data, no lifecycle in flags or `Option`s beside it,
  exhaustive matches and one handler per cell; the echo that
  applications copy keeps its stop in `Protocol`'s `stopping` and
  `aborting` flags (`skein/examples/echo/protocol/src/layer.rs`), and
  `conn.rs` keeps `ended` flags in `State::Admitting` and
  `State::Answering` and `closed` and `call` flags in `Closing`, rather
  than explicit enums for those lifecycles; no reason is recorded (the
  code's side is
  `skein/docs/development/debt/D024-echo-template-flags.md`).
