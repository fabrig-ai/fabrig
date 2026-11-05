+++
id = "route-limits"
part = "d011-route"
kind = "interface"
needs = ["fake-names"]
files = ["skein/crates/skein-fake-forge-protocol/src/git.rs", "skein/crates/skein-fake-forge-protocol/src/git/**", "skein/crates/skein-fake-forge-protocol/src/lib.rs", "skein/crates/skein-fake-forge-protocol/src/server.rs", "skein/crates/skein-fake-forge-protocol/src/server/git.rs", "skein/crates/skein-fake-forge-protocol/src/service.rs", "skein/crates/skein-fake-forge-protocol/src/tests/git.rs", "skein/crates/skein-fake-forge-github/src/git.rs", "skein/crates/skein-fake-forge-github/src/server.rs", "skein/crates/skein-fake-forge-github/src/tests/git.rs", "skein/crates/skein-fake-peers/src/forge/shared.rs", "temper/crates/temper-fake-git/src/**", "temper/crates/temper-fake/src/**", "temper/crates/temper-playground/src/**", "temper/crates/temper/src/fake/git.rs", "temper/tests/one-process/**", "temper/tests/real-loop/**"]
effort = "high"
minutes = 90
+++

# The git route is typed by neutral types

## Goal

D011's first clause. The route's functions take `git::Limits`, which holds what the route reads, and nothing of the route
names Forgejo: GitHub's fake and temper stop filling in Forgejo limits they don't use. The route stays in the Forgejo fake
for this task; `route-move` carries it out whole.

Acceptance:

- `notes/contracts.md`, 3 holds in `P/git.rs`: `Limits`, `Unrouted`, `status`; and `protocol::Limits::git`.
- `rg -n 'crate::Limits|forgejo' skein/crates/skein-fake-forge-protocol/src/git.rs
  skein/crates/skein-fake-forge-protocol/src/git/` finds nothing.
- `G/git.rs` names no `protocol::Limits`, `protocol::errors` or `protocol::Route`; `limits()` builds a `git::Limits`, with
  the same `http` and `documents` values as at base.
- A table test in `P/tests/git.rs` pins `git::status` to `errors::answer(.., Route::Git(name), ..).status` for every
  `api::Error` and `Name`.
- `temper-fake-git`'s `Route` and transport take `git::Limits`; its test fixture no longer sets `calls`, `connections` or
  `raw_bytes`.
- The packages of step 6 pass, the same tests as at base, plus yours.

## Inputs

- `temper/docs/plan/debt-15/notes/common.md`
- `temper/docs/plan/debt-15/notes/contracts.md`, 3
- `skein/docs/design/fake-forge.md`, 4 (**The git route**) and 12

## Owns and may change

- **Owns:** `P/git.rs`, `P/git/**`; `Limits::git` in `P/lib.rs`.
- **May change, call sites only (a type path, `.git()`):**
  - `P/server/git.rs`, `P/server.rs`, `P/service.rs`, `P/tests/git.rs`, `skein-fake-forge-protocol/tests/suite/memory.rs`;
    `P/server/git.rs` maps `git::Unrouted::NotFound` to the status it answers today.
  - `G/git.rs`, `G/server.rs`, `G/service.rs`, `G/tests/git.rs`.
  - `skein-fake-peers/src/forge/shared.rs`, `skein/tests/fake-peers/tests/forge/git.rs`.
  - `temper-fake-git`: `route.rs`, `exchange.rs`, `transport.rs`, `program.rs`, `tests.rs`.
  - `temper-fake`: `wire.rs`, `forge.rs`, `forge/github.rs`, `tests.rs`, `tests/github.rs`. `git_limits` stays the full
    type; its readers call `.git()`.
  - `temper/src/fake/git.rs`; `temper-playground`: `fakes.rs`, `programs.rs`, `story.rs`, `tests/world.rs`;
    `temper/tests/one-process`: `world.rs`, `lower/github.rs`; `temper/tests/real-loop`: `git.rs`, `github.rs`, `forge.rs`.
  This is the one branch that edits temper for this type: each line only in its type path or its `.git()`. Any further
  file that fails to compile for this type alone is pre-approved, the same way.
- **Not to touch:** `Service`'s and `Server`'s own use of `protocol::Limits` in Forgejo's layer; `errors::answer` stays
  as it is; `fail` and `json_limits` in `P/server/git.rs` keep the full type if they read its other fields.
- **Options:**
  - temper's route client reads `http.{head,response,headers,read,send}`: they're in `git::Limits.http`, so it needs nothing.
  - If `git::Limits` can't be `const fn`-built, drop `const`.
  - If a test compares a route with `git_limits()`, it compares with `git_limits().git()`: a change of spelling.

## Steps

1. `git::Limits`, `git::Unrouted`, `git::status` in `P/git.rs`; `use crate::Limits` goes from `git/`.
2. `protocol::Limits::git`.
3. Forgejo's entrance and its readers.
4. `G/git.rs`: `limits()` drops `calls`, `connections` and `raw_bytes`; `error` calls `git::status`, so `Time` goes if
   nothing else reads it.
5. skein-fake-peers, then temper's call sites.
6. `cargo fmt`; `heavy cargo clippy` on the packages you changed, `--all-targets -- -D warnings`; `heavy cargo nextest run
   -p skein-fake-forge-protocol -p skein-fake-forge-github -p skein-fake-peers-tests -p skein-forge-protocol-world -p
   temper-fake-git -p temper-fake -p temper-playground`; `heavy cargo check --tests -p temper -p temper-one-process-world
   -p temper-real-loop-world -p temper-worker-checkout-world`; the citations check. Commit and push.

## Report

The common note's items. Break: give `git::status` another status for `Full` on `Commit`, and name the table test's
failure. List every file edited outside this list's names, and why.
