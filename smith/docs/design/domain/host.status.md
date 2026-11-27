# host: status

2026-11-27. Where smith's code lags or departs from [host.md](host.md).


Section 4 has no known gap in retaining a panic's closing line and
message when they fit the detail bound. The host-protocol detail table
has a report with a backtrace longer than 4 KiB, retained at 1,024
bytes, with delivery chunks of 1, 20 and 4,096 bytes. It checks the
closing location and message within the existing bound; the keeper
remains a tail.

- **Not yet:**
  - 2, 9.1: live content in one process. The inline agent's LLM deltas
    never reach its owner: `smith-protocol-llm` hands deltas over only
    for a channel's ask, and `smith-local-service`'s `pop_live()` reads
    a spawned agent's alone.
  - 4: an agent spawned in a contained tree, in its view, and gone once
    the tree is proved empty: `smith-host-protocol/src/process.rs`
    spawns a plain child in a process group.
  - 7: a missing grant while the local chat still starts. A grant that
    fails for an account the starting chat already holds, as a refresh
    that fails (skein's `oauth/accounts.md`, 3), reaches the starting chat as
    the answer to an account request
    (`smith-local-domain/src/domain/chat_machine.rs`, `no_credential`):
    with no request waiting it aborts ("one terminal per account
    request"), and with another account's waiting it is counted as that
    one's answer.
- **Design departs:** 4, from goal 16297's request of 2026-11-24, decided
  by its design: the request ends an agent whose host "stays silent past
  its bound", and section 4 counts silence only while the agent waits on
  its host, so a host that owes nothing is never silent. Over pipes a host
  sends no keepalive (protocol/channel.md, section 7): a healthy host
  keeping a run waiting for its next message would read as gone, and host
  pings would change the channel's wire.
