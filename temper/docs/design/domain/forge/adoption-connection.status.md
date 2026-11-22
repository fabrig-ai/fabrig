# adoption-connection: status

2026-11-22. Where the tree's code lags or departs from [adoption-connection.md](adoption-connection.md).

- **Not yet:**
  - 1: a removal without the box, its group's act answered in its
    decision: an owned or fake forge's removal asks the box link for
    `remove N` whether or not a pipe is there, and without one waits for
    an answer that never comes, so it is never whole and the
    playground's reset stops at its forge
    (`temper-engine-domain-forge/src/removal.rs`, `begin`, sets
    `box_removed` by the forge's mode alone;
    `temper-service/src/route/box_link.rs`, `remove`).

- **Not yet:** 1, "What waits on it": a change's step on a forge not connected reads all the same, and a read of its step answered `NotConnected` holds the change `ResourceRefused` (`temper-engine-domain-forge/src/domain/admission.rs`, `read_failed`), never stepped again on `Connected`.
