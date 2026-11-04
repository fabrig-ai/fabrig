+++
id = "github-media-type"
part = "github-media-type"
kind = "feature"
needs = []
files = ["skein/crates/skein-fake-forge-github/src/server.rs", "skein/crates/skein-fake-forge-github/src/service.rs", "skein/crates/skein-fake-forge-github/src/inline.rs", "skein/crates/skein-fake-forge-github/src/tests/inline.rs", "skein/tests/fake-peers/tests/forge/github.rs", "skein/docs/design/fake-forge.status.md"]
effort = "medium"
minutes = 35
+++

# skein: the fake GitHub names one media type, github.com's

## Goal

The user's decision of 2026-11-04 (the request's part 2, the question `github-media-type` of
`temper/docs/plan/debt-7/plan.toml`): `Server`'s charset switch goes, and every reply of the fake GitHub
names github.com's spelling, with `; charset=utf-8`.

Acceptance:

- `rg -n 'charset' skein/crates/skein-fake-forge-github/src/server.rs skein/crates/skein-fake-forge-github/src/inline.rs`
  finds nothing, and `ContentType` has one spelling function.
- A `Server` built by `Server::new` or `Server::blob` answers as `Inline` does.
- `skein/tests/fake-peers`' `github_peer_serves_the_manifest_and_conversion_pages` reads
  `text/html; charset=utf-8`.
- The readers' tests below pass, with no other assertion moved but as **The assertions** says.
- `fake-forge.status.md`'s entry for 10.6 is gone.

## Inputs

- `temper/docs/plan/debt-9/notes/common.md`, `AGENTS.md`, `skein/AGENTS.md`.
- `skein/docs/design/fake-forge.md`, 10.6, the **Media types** entry (written for this task).

## Owns and may change

- Owns: `skein-fake-forge-github`'s `server.rs` (the `charset` field, `fn charset`, the one `if` in the
  reply's head), `service.rs` (`ContentType`), `inline.rs` (the `.charset()` call).
- Reader: `skein/tests/fake-peers/tests/forge/github.rs`, only as **The assertions** says.
- `skein/docs/design/fake-forge.status.md`: the 10.6 sentence goes.
- With `charset` gone `Server` has three `bool` fields: if clippy calls its
  `#[expect(clippy::struct_excessive_bools, ...)]` unfulfilled, remove the expectation.
- `Inline::exchange` takes `Server` by value only for the switch: if `let mut server = server.charset();`
  leaves a plain rebinding, make the parameter `mut server: Server`.

## The assertions

The request approves **one** assertion moving, and says no other does:
`skein/tests/fake-peers/tests/forge/github.rs`, `page.header(b"content-type")`, `text/html`. Move it.

Two more read the bare spelling, and the request did not count them:

1. the next assertion of the same test, `conversion.header(b"content-type")`, `application/json`;
2. `the_peers_server_names_a_bare_media_type` in `skein-fake-forge-github/src/tests/inline.rs`, which pins
   the switch itself.

Neither is yours to move on this spec. Read the question `github-media-type-assertions` in
`temper/docs/plan/debt-9/plan.toml` as it stands in your slot:

- **It has an `answer`:** do what the answer says.
- **It has none:** make the rest of the change, leave those two as they are, commit and push, and report
  `Question:` with the plan's question and what you ran. Don't edit either to get green.

Any other test whose outcome moves (a head that no longer fits a tight limit, a byte offset in a fault test)
is a stop and ask, the same way.

## Steps

1. In `service.rs`, `ContentType` keeps one function, `bytes`, with `charset`'s spellings: `Json`, `Html` and
   `Plaintext` with `; charset=utf-8`, `Octets` as `application/octet-stream`. `charset` goes.
2. In `server.rs` the field, its initializer, `fn charset` and the `if` go:
   `value: bytes::copy_of(reply.content_type.bytes())`.
3. In `inline.rs` the call goes.
4. The approved assertion; **The assertions** for the rest.
5. A longer head is a change of meaning for every world on the fake GitHub, so run the readers:
   `heavy cargo nextest run -p skein-fake-forge-github`, `-p skein-fake-peers`, `-p skein-fake-peers-world`
   (find `skein/tests/fake-peers`' package name in its `Cargo.toml`), and the worlds under
   `skein/tests/forge/{github,machine,protocol,setup}` by their package names; `heavy cargo check --tests` on
   `temper-fake-git-hosted` and `temper-playground`.
6. `cargo fmt`; `heavy cargo clippy -p skein-fake-forge-github --all-targets -- -D warnings`; the citations
   check. Commit and push.

## Report

The common note's items. Break: put `.bytes()` back to the bare `Json` spelling and name the failure of
`an_app_request_is_answered_whole`. Say how many assertions moved and which.
