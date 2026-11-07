+++
id = "list-exchange"
part = "provider-list"
kind = "feature"
needs = ["list-parse", "list-fake"]
files = ["skein/crates/skein-llm-connection/src/list.rs", "skein/crates/skein-llm-connection/src/lib.rs", "skein/crates/skein-llm-connection/Cargo.toml", "skein/crates/skein-llm/src/client.rs", "skein/tests/llm-connection/tests/suite/list.rs", "skein/tests/llm-connection/tests/suite/main.rs", "skein/docs/design/llm-connection.status.md"]
effort = "high"
minutes = 110
+++

# One bounded request on an endpoint

## Goal

`skein_llm_connection::list::Exchange`: the one `GET /v1/models` of a list, on an endpoint's address and transport, with one answer. smith's `--list` holds one and nothing else of this crate.

Acceptance:

- `contracts.md`, 2's names exist, as `skein/docs/design/llm-connection.md`, 4.3 says.
- A story in `skein/tests/llm-connection`, over its simulated sockets and `list-fake`'s peer, with a row for each answer:
  - `Listed`, over plaintext and over TLS, on each route, the request's head observed: its target, the credential as a call sends it, `connection: close`, and on Anthropic its version header;
  - `Refused` for a 401, a 404 and a 500, holding the status alone;
  - `Unreadable` for a body past `Limits.body`, stopped at the bound without reading the rest, and for a body that doesn't parse;
  - `Unreachable` for a refused connection and for a connection lost mid-body;
  - `Deadline` for a peer that never answers, by a virtual clock.
- `answer()` gives one answer, once, and only after the socket has settled. A `cancel` settles with no answer's meaning: it answers `Deadline`.
- `Debug` of an `Exchange` holds no credential, as `Request`'s `missing_debug_implementations` reason has it.
- `llm-connection.status.md`'s 4.3 line goes.

## Inputs

- `temper/docs/plan/model-catalogue-3/notes/common.md`.
- `temper/docs/plan/model-catalogue-3/notes/contracts.md`, 1 to 3.
- `skein/docs/design/llm-connection.md`, 3, 4.2 and 4.3; `skein/docs/design/llm.md`, 2.6 and 4.9; `skein/docs/design/link.md`.
- `skein/crates/skein-oauth-accounts/src/exchange.rs:174-260`: `Web`, the same shape. Read it and write yours beside `skein-link` and `skein-http` the same way. Don't change it: the shared form is `llm-connection.md`, section 9's.

## Owns and may change

Owns the header's files. `skein-llm/src/client.rs:1592,1670` holds how a credential is sent: make what you need `pub(crate)`-to-public as one function, `skein_llm::list::credential_headers`, or the narrowest equal, used by both.

Options, each pre-approved:

- `Deadlines`' `connect`, `handshake` and `whole` are read; `head` and `idle` are ignored here, and the `whole` deadline bounds everything.
- `ExchangeLimits` holds the `skein-http`, `skein-tls` and `skein_llm::list::Limits` bounds it needs, with a `DEFAULT`. A body is at most `list::Limits.body`.
- A response's `content-encoding: gzip` isn't asked for, so isn't read: it answers `Unreadable`.
- A redirect is `Refused`, by its status.

## Steps

```rust
// skein-llm-connection/src/list.rs
//! One list request on an endpoint (skein/docs/design/llm-connection.md, 4.3). Keeps one link
//! and one bounded body; it never knows who asked. `start`, `up` and `fire` drive it;
//! `answer` gives its one answer after its socket has settled.
```

Then `cargo fmt`; `heavy cargo clippy -p skein-llm-connection -p skein-llm -p skein-llm-connection-world --all-targets -- -D warnings`; `heavy cargo nextest run -p skein-llm-connection -p skein-llm -p skein-llm-connection-world`. Commit and push.

## Report

As the implementer's text says.
