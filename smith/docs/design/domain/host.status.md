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
  - 8: the local domain holds none of the person's words:
    `smith-local-domain/src/person.rs`, `turn_text` and `tool_end`, and
    `smith-local-domain/src/boundary.rs`, `Older::words`, compose them,
    and `smith-local-service/src/service.rs`, `effort_notice`, patches
    a notice's (the code's side is
    `smith/docs/development/debt/D018-domain-formats-text.md`).
