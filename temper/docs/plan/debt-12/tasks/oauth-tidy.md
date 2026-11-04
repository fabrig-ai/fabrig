+++
id = "oauth-tidy"
part = "owners"
kind = "migration"
needs = []
files = ["skein/crates/skein-oauth-accounts/src/exchange.rs"]
effort = "medium"
minutes = 10
+++

# oauth's `link_env` is one `Env::with`

## Goal

`link_env` in `skein-oauth-accounts/src/exchange.rs` is seven lines `Env::with` says in three. No behaviour
changes.

Acceptance:

- `rg -n 'wall: env.wall' skein/crates/skein-oauth-accounts/src/exchange.rs` no longer lists `link_env`.
- `heavy cargo nextest run -p skein-oauth-accounts -p skein-oauth-accounts-world` passes, no test changed.
- `heavy cargo clippy -p skein-oauth-accounts --all-targets -- -D warnings` passes.

## Inputs

- `temper/docs/plan/debt-12/notes/common.md`
- `temper/docs/plan/debt-12/notes/contracts.md`, 2 and 3

## Owns and may change

- **Owns:** `link_env` in `skein/crates/skein-oauth-accounts/src/exchange.rs`, and nothing else there.
- **Leave alone:** `if !self.link.encrypted()` in that file: the request leaves it, because removing it retimes
  requests. `due(web.next_deadline(), now)` in `component.rs`: see the contracts' section 3.

## Steps

1. `link_env`, as the contracts' section 2 says, with this file's `link::Limits`.
2. Fmt, clippy, tests. Commit and push.

## Report

The report of your implementer text, and net production lines.
