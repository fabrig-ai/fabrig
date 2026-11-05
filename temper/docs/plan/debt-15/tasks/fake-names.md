+++
id = "fake-names"
part = "d011-names"
kind = "feature"
needs = ["github-media-type"]
files = ["skein/crates/skein-fake-forge-github/src/**", "skein/crates/skein-fake-forge-domain/src/object.rs", "skein/crates/skein-fake-forge-domain/src/lib.rs", "skein/crates/skein-fake-forge-domain/src/tests/**", "skein/crates/skein-fake-forge-protocol/src/routes.rs"]
effort = "medium"
minutes = 45
+++

# The fake GitHub names no `forgejo::` item

## Goal

`skein/docs/development/debt/D011-fake-github-on-forgejo-fake.md`'s measure goes to 0: each `forgejo::` name in the GitHub
fake becomes the neutral root's item (`skein/docs/design/forge.md`, 4.4), skein-lib's, or the domain's. The commit's
bytes stop going through the Forgejo fake's `routes`.

Acceptance:

- `rg -c 'forgejo::' skein/crates/skein-fake-forge-github/src` finds nothing, so the item's measure reads 0 (the item says
  26; 28 at this plan's base).
- `rg -n 'routes::' skein/crates/skein-fake-forge-github/src` finds no `skein_fake_forge_protocol::routes`.
- `commit_bytes` and `commit_count` are in the domain as `notes/contracts.md`, 4 has them, each tested, and `routes::object`
  and `routes::commit` are built on them with their signatures unchanged.
- `heavy cargo nextest run -p skein-fake-forge-domain -p skein-fake-forge-protocol -p skein-fake-forge-github -p
  skein-fake-peers-tests -p github-conformance` passes, the same tests as at base, plus yours.

## Inputs

- `temper/docs/plan/debt-15/notes/common.md`
- `temper/docs/plan/debt-15/notes/contracts.md`, 4
- `skein/docs/design/forge.md`, 4.4; `skein/docs/design/fake-forge.md`, 12

## Owns and may change

- **Owns:** `skein-fake-forge-github/src/**`; `skein-fake-forge-domain/src/object.rs`, its `lib.rs` export line and its
  tests.
- **May change:** `skein-fake-forge-protocol/src/routes.rs`: the bodies of `object` and `commit`, those alone.
- **Not to touch:** `G/git.rs`' `protocol::Limits` and `errors::answer`, which `route-limits` takes, but for its one
  `skein_forge::forgejo::limits::Limits`, which is this task's.
- **Options:** other copies of the padding, in `skein/tests/forge/client/src/translate.rs` and
  `skein/tests/forge/github/src/fake.rs`, stay: they're not in the item.

## Steps

The items, each with its home:

| Item | Where | Becomes |
|---|---|---|
| `forgejo::document::Body` | `delivery.rs`, `reads/reviews.rs`, `reads/issues.rs`, `comment_edit.rs`, tests | `skein_forge::text::Body`, the same type |
| `forgejo::time::write`, `read` | `rates.rs`, `answers.rs`, `tests/rates.rs` | `skein_forge::time::*` |
| `forgejo::limits::Limits` | `git.rs` | `skein_forge::limits::Limits` |
| `forgejo::id::unsigned` | `delivery.rs`, `routes.rs`, `logs.rs`, `mutations.rs`, `reads.rs`, `comment_edit.rs` | `skein_lib::bytes::decimal`, which it wraps |
| `forgejo::id::{ObjectFormat, ObjectId}` | `reads.rs` | `skein_fake_forge_domain::{commit_bytes, commit_count}` |
| `forgejo::id::write` | `tests/inline.rs` | `skein_forge::github::id::write_commit` |
| `skein_fake_forge_protocol::routes::{object, commit}` | `reads.rs`, `tests/service_reads.rs`, `tests/read_support.rs`, `tests/inline.rs` | the domain's two functions |

1. The domain's two functions, moved in substance from `P/routes.rs`, and `routes` on them.
2. Each line of the table. `reads::commit` and `reads::count` keep their signatures.
3. `cargo fmt`; `heavy cargo clippy -p skein-fake-forge-domain -p skein-fake-forge-protocol -p skein-fake-forge-github
   --all-targets -- -D warnings`; the tests; the citations check. Commit and push.

## Report

The common note's items. The measure before and after. Break: make `commit_count` accept a value with its mix wrong, and
name the failure.
