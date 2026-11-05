+++
id = "migrate-request"
part = "forgejo-requests"
kind = "interface"
needs = []
files = ["skein/crates/skein-forge/src/forgejo/request.rs", "skein/crates/skein-forge/src/forgejo/client.rs", "skein/crates/skein-forge/src/forgejo/client/orgs.rs", "skein/crates/skein-forge/src/forgejo/server/orgs.rs", "skein/crates/skein-forge/src/forgejo/server/writes.rs", "skein/crates/skein-forge/src/forgejo/tests/repositories.rs"]
effort = "medium"
minutes = 35
+++

# `Migrate` and `DeleteRepository` in the Forgejo dialect

## Goal

The Forgejo dialect says the two requests a copy sends (contracts S1): a client encodes each and a
server decodes each. Nothing sends them yet, and the fake answers each as a route it does not serve.

Acceptance:

- `Request::Migrate { at, from }` encodes `POST /api/v1/repos/migrate`, with the body of
  `skein/docs/design/forge-setup.md`, 4.3, in that section's field order, and decodes its `201` as
  `Document::CreatedRepository`.
- `Request::DeleteRepository { at }` encodes `DELETE /api/v1/repos/{owner}/{name}`, each one encoded
  path segment, with no body; a 2xx has no decoder.
- `server` routes both, each round trip equal to the request, and refuses a `Migrate` body whose
  `service` is not `git`, whose `mirror` or any other fixed field is `true`, or whose `private` is not
  `true`, as it refuses `CreateRepository`'s `default_branch` of another value.
- New tests in `skein/crates/skein-forge/src/forgejo/tests/repositories.rs`:
  `migrate_encodes_its_fixed_body_and_round_trips`, `migrate_refuses_a_body_that_asks_for_more`,
  `delete_repository_is_one_segment_each_and_round_trips`, each with its bytes spelled out. No
  transcript yet: `copy-facts` records them and adds the recorded test.
- `heavy cargo nextest run -p skein-forge -p skein-fake-forge-protocol -p skein-forge-connection -p
  skein-forge-setup` pass; `heavy cargo check --tests -p skein-fake-forge-github` passes.

## Inputs

- `temper/docs/plan/repository-copies-1/notes/common.md`; `notes/contracts.md`, S1.
- `skein/docs/design/forge-setup.md`, 4.3, its paragraphs on `CreateRepository`, `Migrate` and
  `DeleteRepository`.

## Owns and may change

- The files of the header.
- **Reader files, each the two variants added to an existing or-pattern, nothing else:**
  `skein/crates/skein-forge/src/forgejo/client/{items,reviews,writes,files,start}.rs`,
  `skein/crates/skein-forge/src/forgejo/server/writes.rs`,
  `skein/crates/skein-forge/src/forgejo/plans/reads/keep.rs`,
  `skein/crates/skein-forge-connection/src/connection.rs`,
  `skein/crates/skein-fake-forge-protocol/src/{response,routes,answers,server,setup,tally}.rs`,
  `skein/crates/skein-forge-setup/src/host/connection.rs`. Follow `Request::DeleteRepoHook` and
  `Request::CreateRepository` through every match: `rg 'Request::CreateRepository|Request::DeleteRepoHook'`.
- **The fake:** where a match must answer, each variant answers as a route the fake does not serve
  (`Unrouted::NotFound` or that file's equivalent, so `404`). `tally.rs` gets `Route::Migrate` and
  `Route::DeleteRepository`, in `Self`'s setup group; the tally's bound of routes grows by 2 if it is a
  constant, and a test pinning it takes the new value. `fake-migrate` makes them answer.
- `skein/crates/skein-forge/src/forgejo/limits.rs`: only if a bound must name the new body.
- Pre-approved:
  - If `WriteKind` needs a member, `WriteKind::Migration`. If `org_target`'s shape does not fit
    `/api/v1/repos/...`, use `hooks::repo_target`'s, or `out.put` and `out.segment` inline.
  - `from` is a JSON string as it is: this layer checks no address.
  - If `client/orgs.rs` grows past its neighbours, a new `client/repositories.rs` arm is fine: that
    file holds `CreatedRepository`'s fields already.

## Steps

1. The variants on `Request`, with one-line doc comments citing forge-setup.md, 4.3.
2. `client.rs`'s `encode` and `decoder`; the target and body in `client/orgs.rs`.
3. `server/orgs.rs`'s route and `server/writes.rs`'s fields. `/repos/migrate` is two segments after
   `v1`; `/repos/{owner}/{name}` is three, and is routed where the other repository routes are.
4. The compiler's arms in the reader files.
5. The tests, each biting: flip `private` in the encoder, and accept `mirror: true` in the server,
   each once.
6. The implementer's checks; commit and push.

## Report

As the implementer's text asks; the arms the compiler asked for, by file.
