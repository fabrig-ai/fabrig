# fake-forge: status

2026-11-11. Where skein's code lags or departs from [forge.md](forge.md).

Unlisted sections have no known gap. `github.md`, section 1.1's App JWT check uses
the key's public half and verifies without signing; its exact-token/key
cache and the claims checked on every request are present.

- **Not yet:** 4: the runs' listing has no `workflow_id`
  (`skein-fake-forge-forgejo/src/routes.rs`).
- **Not yet:** 4: conditional reads: a raw read answers no `ETag`, and no
  route answers `304` (`skein-fake-forge-forgejo/src/server.rs`,
  `src/response.rs`).
- **Not yet:** 4: an issue's comments ignore `since`, on the Forgejo
  layer (`skein-forge/src/forgejo/server/items.rs`) and the GitHub layer.
- **Not yet:** 4: a refused source, missing or private, leaves no
  repository (`skein-fake-forge-forgejo/src/setup/migrate.rs`,
  `migration`), where Forgejo leaves an empty one, and `DeleteRepository`
  asks `write:repository` (`server.rs`, `missing_scope`), where Forgejo
  asks `write:organization`, so no story shows either.
 
- **Not yet:** 4: a caller who is no site admin and holds no grant reads any repository by its path as `403` (`skein-fake-forge-domain/src/domain.rs`, `authorized_read`), where Forgejo shows a public one to any token and answers a private one `404` (`unreadable-repository-by-path-is-404`), as the fake's read by id does, so on the fake no adoption finds a public repository temper's forge user may only read.  the GitHub push-refusal gap remains `github.status.md`, 1.2.

- **Departs:** 4: `skein/docs/foundation/testing-strategy.md`, 4 asks for no shared types, but `skein-fake-forge-forgejo/src/server.rs` uses `skein-forge`'s request types and server document halves, as this section and `skein/docs/design/forge/README.md`, 4.1 design, and `skein-fake-forge-http/src/git.rs` holds `skein_forge::limits::Limits`, as `github.md`, section 3 designs; `skein-fake-oauth` shares `skein-oauth`'s HTTP and token document types, `skein-fake-channel` shares `skein-channel`'s frames and codec, and the fake CDP peer in `skein-browser-world` shares `skein-browser`'s `wire::decode` types and decoder (`skein/tests/browser/src/world.rs`). No reason is recorded for departing from the foundation; the backfill recommends amending `skein/docs/foundation/testing-strategy.md`, 4, settled when `skein/docs/development/debt/D043-browser-kit-json-and-tables.md` is planned. 4: `skein/docs/design/runtime/lib.md`, 5 has an uncapped queue hold nothing a peer can push in, but `skein-fake-forge-forgejo/src/server.rs`'s `Server` keeps a copy of each decoded request in one for `take_request`, which the setup world alone drains, so under the forge peer (`skein-fake-peers`) a connection's copies stay until it closes; no reason is recorded, and no debt item names it.  the user's request of 2026-11-04 has the fixture's App, accounts and repositories built once, in the fake; that holds in skein alone. Four builders of temper's stand beside it, each starting `Peer::github` from its own `Config` (`temper/crates/temper-fake/src/forge/github.rs`, `temper/tests/client-browser/src/github.rs`, `temper/tests/one-process/src/lower/github.rs`, `temper/tests/real-loop/src/github.rs`), because the item that request paid is skein's and they write no HTTP of their own.  the shared connection-state departure remains `github.status.md`, 3.
