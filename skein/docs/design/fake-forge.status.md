# fake-forge: status

2026-11-04. Where skein's code lags or departs from [fake-forge.md](fake-forge.md).

- **Not yet:** 10.2: the GitHub layer refuses no push as `GH013`: a push reaches the domain, whose `Rejected` is its only refusal. So 11's required-signatures fact stays unchecked on the fake. 10.6: `Server`'s `charset` switch still leaves the peer's connections naming bare media types; `Inline` alone turns it on.
- **Departs:** 4: `skein/docs/foundation/testing-strategy.md`, 4 asks for no shared types, but `skein-fake-forge-protocol/src/server.rs` uses `skein-forge`'s request types and server document halves, as this section and `skein/docs/design/forge.md`, 4.1 design; `skein-fake-oauth` shares `skein-oauth`'s HTTP and token document types, `skein-fake-channel` shares `skein-channel`'s frames and codec, and the fake CDP peer in `skein-browser-world` shares `skein-browser`'s `wire::decode` types and decoder (`skein/tests/browser/src/world.rs`). No reason is recorded for departing from the foundation; the backfill recommends amending `skein/docs/foundation/testing-strategy.md`, 4, settled when `skein/docs/development/debt/D043-browser-kit-json-and-tables.md` is planned.
