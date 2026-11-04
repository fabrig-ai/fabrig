# host: status

2026-11-04. Where smith's code lags or departs from [host.md](host.md).

- **Not yet:**
  - 2, 9.1: live content in one process. The inline agent's LLM deltas
    never reach its owner: `smith-protocol-llm` hands deltas over only
    for a channel's ask, and `smith-local-service`'s `pop_live()` reads
    a spawned agent's alone.
  - 4: an agent spawned in a contained tree, in its view, and gone once
    the tree is proved empty: `smith-host-protocol/src/process.rs`
    spawns a plain child in a process group.
  - 7: a grant refreshed while the local chat still starts. The oauth
    driver's `granted` again for a held grant (skein's `oauth.md`, 6.3)
    reaches the starting chat as the answer to an account request
    (`smith-local-domain/src/chat/machine.rs`, `granted` and
    `no_credential`): with no request waiting it aborts ("one terminal
    per account request"), and with another account's waiting it is
    counted as that one's answer.
- **Departs:**
  - 8: each kind of failure has its own words, matched exhaustively;
    `smith-local-domain/src/person.rs`, `turn_text` and `tool_end`,
    compose person-facing text, exposed by
    `smith-local-domain/src/domain.rs`, `Domain::tool_verdict` and
    `Domain::result_text`, alongside the protocol's notice rendering.
    No reason is recorded; whether this departs is the designer's
    decision (the code's side is
    `smith/docs/development/debt/D018-domain-formats-text.md`).
  - 8: `skein/docs/foundation/programming-model.md`, 5.4 asks for one
    explicit state enum per lifecycle, each state holding exactly its
    data, no lifecycle in flags or `Option`s beside it;
    `smith-local-service/src/host_io.rs`, `HostIo`, spreads the process
    tree's life and its close over `process`, `reading`, `failed`,
    `terminal_input`, `closing`, `signal_operation` and
    `close_root_pending`, and `smith-local-domain/src/domain.rs`,
    `Domain`, keeps its delivery's save in `saving`, `delivery_owner` and
    `reconciling` beside `delivery/machine.rs`; no reason is recorded
    (the code's side is
    `smith/docs/development/debt/D002-lifecycles-in-flags.md`).
