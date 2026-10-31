+++
id = "lib-forge"
part = "lib-sites"
kind = "migration"
repo = "fabrig"
needs = ["ratchets", "kit-lib", "abort-connection"]
files = ["skein/crates/skein-forge/src/**", "skein/crates/skein-forge-connection/src/**", "skein/scripts/old-forms.txt"]
effort = "medium"
minutes = 40
+++
# lib's primitives in skein-forge and skein-forge-connection

Read first: `skein/docs/plan/component-kit/notes/common.md`, `skein/docs/plan/component-kit/notes/lib-common.md`, `skein/docs/plan/component-kit/notes/decisions.md`.

Draft, 3.4. After abort-connection, which shares
skein-forge-connection's files.

## Sites

- skein-forge: `src/forgejo/id.rs:64`, `src/github/id.rs:22` and
  `src/marker.rs:67` `hex` → `hex::digit` / `hex::value`.
- skein-forge-connection: the inline earliest-deadline merges
  (`rg -n 'min\(|Some\(.*\) if' src/` around its deadline code; a
  `match` on two `Option<Time>` taking the smaller) → `skein_lib::earliest`;
  the discarding pop loops in `src/connection.rs` (two),
  `src/installations.rs`, `src/tests.rs` → `clear()`.

## Acceptance

- `hex_decimal` falls by three, `pop_loop` by four; report each inline
  merge moved (path:line).
- `heavy cargo nextest run -p skein-forge -p skein-forge-connection` and
  the forge worlds that read them, with counts.
