# D010: Test fixtures are shared by `#[path]` across crates

- **Kind:** tests
- **Where:** `temper/testing/client_documents.rs`, `temper/testing/first_run.rs`, `temper/testing/memory_literal.rs`, `temper/tests/real-loop/src/turn_bounds.rs`, `temper/tests/real-loop/src/calls/browse.rs`, `temper/tests/engine/domain/tests/suite/forge_routes/tier_task_show.rs`
- **Seen:** 2026-10-31 at `4a6683c72`, in the skein review of that day (`skein/docs/development/debt/reviews/2026-10-31-skein.md`), theme T5 ("`#[path]` reaching across crates"), for skein; temper's own instances are new here
- **Checked:** 2026-11-03 at `41d63da8c3`
- **Value:** low (each fixture compiled once, no `dead_code` excuses, a fixture's users visible in `Cargo.toml`)
- **Size:** S
- **Status:** open
- **Paid when:** no `#[path]` in temper names a file of another crate, or of `temper/testing/` outside a crate; the shared fixtures are a test kit crate the others depend on (the measure is 0).
- **Measure:** `rg -c --type rust '#\[path = "[^"]*(\.\./testing/|real-loop/|engine/domain/)' temper | awk -F: '{s+=$2} END {print s}'`: 13

## What

Three files under `temper/testing/` belong to no crate and are compiled
into others by `#[path]`: `client_documents.rs` into six
(`temper-client-domain` twice, `temper-client-view`, `temper-client-tools`,
`temper-client-shell-terminal`, `temper/tools/gallery`), `first_run.rs`
into `temper-client-view` and `temper-client-shell-terminal`,
`memory_literal.rs` into `temper-client-view`. Worlds reach into each
other the same way: `temper/tests/real-loop/src/turn_bounds.rs` into
`temper/tests/client-browser` and the `temper` crate's process harness,
`temper/tests/real-loop/src/calls/browse.rs` into `temper/tests/one-process`, and an
engine world's test file (`temper/tests/engine/domain/tests/suite/forge_routes/tier_task_show.rs`) into the
client world's suite.

Each including crate then excuses what it does not use: seven of
temper's nine `#[expect(dead_code, ...)]` sit on these includes, five
with the reason "shared document fixtures also supply the other client
suites".

## Why it matters

A fixture's users cannot be found from `Cargo.toml`; each copy is
compiled and linted again, and a change to one shows its breakage only
in another crate's build. One world's test file compiled into another
ties the two suites together.

## What pays it back

Make `temper/testing/` a test kit crate (it already holds
`temper-github-app`), with the three files as modules, and depend on it;
give the real loop's `turn_bounds` and `browse` to the kit or to a
library the worlds share; move `tier_task_show`'s shared assertions to a
kit both worlds use. The `dead_code` expectations go with them.
