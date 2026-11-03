# examples: status

2026-11-03. Where skein's code lags or departs from [examples.md](examples.md).

- **Not yet:** 3.4: the echo's connection keeps the peer's end in
  Admitting's and Answering's `ended` flags and its closing in Closing's
  `closed` and `call` flags, not in the states AdmittingEnded,
  AnsweringEnded and Shut and what the domain still holds; its layer
  keeps its stop in `stopping` and `aborting` flags beside its
  `Shutdown`, not in the connections' stop
  (`examples/echo/protocol/src/conn.rs`, `layer.rs`). 6: the echo's
  referee reads the echo's address from its state
  (`tests/echo/src/referee.rs`), not from its startup fact; the
  end-to-end kit starts its binary in a cgroup through the shell
  (`start_binary_in_cgroup`), not as an io tree.
