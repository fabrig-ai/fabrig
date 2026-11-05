+++
id = "github-media-type"
part = "github-media-type"
kind = "feature"
needs = []
files = ["skein/crates/skein-fake-forge-github/src/server.rs", "skein/crates/skein-fake-forge-github/src/service.rs", "skein/crates/skein-fake-forge-github/src/inline.rs", "skein/crates/skein-fake-forge-github/src/tests/inline.rs", "skein/tests/fake-peers/tests/forge/github.rs", "skein/docs/design/fake-forge.status.md"]
effort = "medium"
minutes = 30
+++

# skein: the fake GitHub names one media type, github.com's

## Goal

The user's decision (the request's part 1): `Server`'s charset switch goes, and every reply of the fake GitHub names
github.com's spelling, with `; charset=utf-8`. The user approved exactly three test changes, listed below; no other.

Acceptance:

- `rg -n 'charset' skein/crates/skein-fake-forge-github/src/server.rs skein/crates/skein-fake-forge-github/src/inline.rs`
  finds nothing, and `ContentType` has one spelling function, `bytes`.
- A `Server` built by `Server::new` or `Server::blob` answers as `Inline` does.
- `github_peer_serves_the_manifest_and_conversion_pages` reads `text/html; charset=utf-8` and
  `application/json; charset=utf-8`.
- `the_peers_server_names_a_bare_media_type` is gone; `an_app_request_is_answered_whole` pins the one spelling.
- The readers' tests of step 5 pass, with no other assertion moved.
- `fake-forge.status.md`'s **Not yet** sentence for 10.6 is gone.

## Inputs

- `temper/docs/plan/debt-15/notes/common.md`
- `skein/docs/design/fake-forge.md`, 10.6, the **Media types** entry.
- For reading, on an older main: `git show origin/debt-9/github-media-type` is most of this change. Don't merge that
  branch.

## Owns and may change

- **Owns:** `skein-fake-forge-github`'s `server.rs` (the `charset` field, `fn charset`, the one `if` in the reply's head),
  `service.rs` (`ContentType`), `inline.rs` (the `.charset()` call).
- **The three approved test changes, and no other:**
  1. `skein/tests/fake-peers/tests/forge/github.rs`: `page.header(b"content-type")` reads `text/html; charset=utf-8`;
  2. the same test: `conversion.header(b"content-type")` reads `application/json; charset=utf-8`;
  3. `skein-fake-forge-github/src/tests/inline.rs`: `the_peers_server_names_a_bare_media_type` is deleted, with any import
     only it used.
- **May change:** `skein/docs/design/fake-forge.status.md`: the 10.6 sentences of **Not yet** go, nothing else but the
  date. This departs from the implementer's text for those sentences alone.
- **Options:**
  - With `charset` gone `Server` has three `bool` fields: if clippy calls its
    `#[expect(clippy::struct_excessive_bools, ...)]` unfulfilled, remove the expectation.
  - `Inline::exchange` takes `Server` by value only for the switch: make the parameter `mut server: Server`.
- Any other test whose outcome moves (a head that no longer fits a tight limit, a byte offset in a fault test) is a stop
  and ask.

## Steps

1. `service.rs`: `ContentType` keeps one function, `bytes`, with `charset`'s spellings: `Json`, `Html` and `Plaintext`
   with `; charset=utf-8`, `Octets` as `application/octet-stream`. `charset` goes.
2. `server.rs`: the field, its initializer, `fn charset` and the `if` go:
   `value: bytes::copy_of(reply.content_type.bytes())`.
3. `inline.rs`: the call goes.
4. The three test changes.
5. A longer head is a change of meaning for every world on the fake GitHub, so run the readers:
   `heavy cargo nextest run -p skein-fake-forge-github -p skein-fake-peers -p skein-fake-peers-tests -p
   skein-forge-machine-world -p skein-forge-protocol-world -p skein-forge-setup-world -p github-conformance`, then `-p
   temper-fake -p temper-engine-protocol-forge-world -p temper-one-process-world`; `heavy cargo check --tests -p
   temper-fake-git-hosted -p temper-playground`.
6. `cargo fmt`; `heavy cargo clippy -p skein-fake-forge-github --all-targets -- -D warnings`; the citations check. Commit
   and push.

## Report

The common note's items. Break: put `bytes` back to the bare `Json` spelling and name the failure of
`an_app_request_is_answered_whole`. Say how many assertions moved and which.
