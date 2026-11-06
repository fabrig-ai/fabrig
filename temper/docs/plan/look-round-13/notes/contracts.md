# look-round-13: the shapes that cross tasks

Binding. A task builds to these and needs no other task's text.

## Look tokens (`connect-look`)

`temper-client-view/src/look.rs` gains one `Style` per class, in `NAMES`, the class `temper-connect-X` for `CONNECT_X`:
`CONNECT` (the class `temper-connect`), `CONNECT_CARDS`, `CONNECT_CARD`, `CONNECT_DEFAULT`, `CONNECT_MUTED`, `CONNECT_YES`,
`CONNECT_ACTS`, `CONNECT_PRIMARY`, `CONNECT_SECONDARY`, `CONNECT_QUIET`, `CONNECT_MORE`, `CONNECT_ROWS`, `CONNECT_ROW`,
`CONNECT_PANEL`, `CONNECT_PAIR`, `CONNECT_ONE`, `CONNECT_TITLE`, `CONNECT_CMD`, `CONNECT_WAITING`, `CONNECT_HOW`,
`CONNECT_STEPS`, `CONNECT_LIVE`, `CONNECT_LATER`, `CONNECT_EMPTY`, `CONNECT_CONFIRM`, `CONNECT_LIST`, `CONNECT_BOUNDED`,
`CONNECT_NONE`, `CONNECT_COUNT`, `CONNECT_SHEET`, `CONNECT_SHEET_HEAD`, `CONNECT_SHEET_BODY`, `CONNECT_SHEET_FOOT`,
`CONNECT_SOURCE`, `CONNECT_SOURCES`, `CONNECT_SAYS`, `CONNECT_ADDRESSES`, `CONNECT_ALERTS`, `CONNECT_FAILED`.

## `DismissMaking` (`dismiss-making`)

- Wire, `temper-client-protocol-wire/schema/client-wire-v1.schema`: `record DismissMaking { forge: u16 org: bytes 1024 name: bytes 1024 }`; `Request` gains `dismiss_making: DismissMaking`; the wire's `MakeRefusal` gains `not_failed`, and so does the store's (`temper-protocol-store`, where a kept answer holds it).
- Engine, `temper-engine-domain-forge`: `Ask::DismissMaking { forge: u16, org: Box<[u8]>, name: Box<[u8]> }`, answered with `MakeAnswer` (`Kept` or `Refused(MakeRefusal::NotFailed)`).
- Client, `temper_client_domain::forge::dismiss_making(forge: u16, org: &[u8], name: &[u8], limits: &w::Limits) -> Option<jig_client::Family>`, beside `family.rs`'s `make_repository`.
