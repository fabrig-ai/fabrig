# D020: Services hand-set token tags and route agent operations through a table

- **Kind:** primitives
- **Where:** `smith-agent-service/src/service/`, `smith-agent-service/src/service.rs`, `smith-local-service/src/credentials/transport.rs`, `smith-local-service/src/service.rs`
- **Seen:** 2026-11-05 at `35f444e8a9`, while moving multiplexing worlds onto io's stage
- **Checked:** 2026-11-05 at `35f444e8a9`
- **Value:** low (one token namespace rule and completion routing by lane)
- **Size:** S
- **Status:** open
- **Paid when:** smith's token tags use `Token::tagged`, with the bit-61 socket tag removed first, and the agent service routes completions by lane as `HostIo` does instead of through its `operations` table.

## What

The local service hand-sets `ACCOUNT_OPERATION_BIT` and
`EFFECT_OPERATION_BIT` in `smith-local-service/src/service.rs`.
Its `credentials/transport.rs` adds `SOCKET_OPERATION` at bit 61 to
separate accounts' socket and file completions. The agent service
hand-sets `COMPONENT_OWNER_BIT` in `smith-agent-service/src/service.rs`
and maps completion tokens through its `operations` table. Its
`next_operation` counter also seeds `files.seed_randomness` in
`service/down.rs` and `service/program.rs`.

## Why it matters

Each hand-set bit repeats the namespace rule that `Token::tagged`
provides (`skein/docs/design/lib.md`, 4). The agent's table adds another
routing rule beside io's lanes.

## What pays it back

Remove bit 61 first: `Token::tagged` refuses values from bit 60 upward,
so the nested socket tag cannot be its value. Accounts have one `Io`
and one `FileIo`, which `file_layer::owns` already separates. Use
`Token::tagged` for the remaining namespaces and route the agent's
completions by lane as `HostIo` does. Preserve the randomness seeded
through `next_operation` when removing the operations table.
