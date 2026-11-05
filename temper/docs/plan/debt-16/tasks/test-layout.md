+++
id = "test-layout"
part = "test-layout"
kind = "feature"
needs = ["own-exchange"]
files = ["skein/crates/skein-forge-connection/src/tests/**", "skein/crates/skein-forge-connection/src/component/github/tests.rs", "skein/crates/skein-forge-connection/src/component/runtime.rs", "skein/crates/skein-forge-connection/src/connection.rs", "skein/crates/skein-forge-connection/src/connection/forgejo.rs", "skein/crates/skein-forge-connection/src/connection/github.rs", "skein/crates/skein-forge-connection/src/connection/tests.rs", "skein/crates/skein-forge-connection/src/connection/tests/**", "skein/crates/skein-forge-connection/src/connection/forgejo/tests.rs", "skein/crates/skein-forge-connection/src/connection/github/tests.rs"]
effort = "medium"
minutes = 50
+++

# skein: the forge connection's tests lose their dialect mismatch and their layout tricks

## Goal

Finding 8 and the last of finding 10. Tests rebuild the dialect mismatch that `side-out` took out of production: a test
enum of dialects, dispatched on, with `unreachable!` arms. Two test helpers keep an ignored `_at`, three test modules sit
behind `#[path]`, and `remove` in `component/runtime.rs` uses the closure the crate's `find` says is excluded.

Acceptance:

- `rg -n 'FixtureHeard|enum Exchange' skein/crates/skein-forge-connection/src/tests/fixtures.rs` finds nothing.
- `rg -n 'unreachable!\("(Forgejo fixture|Forgejo harness answer|GitHub harness answer)"\)'
  skein/crates/skein-forge-connection/src` finds nothing.
- `rg -n '_at\b' skein/crates/skein-forge-connection/src/component/github/tests.rs` finds nothing.
- `rg -n '#\[path' skein/crates/skein-forge-connection/src` finds nothing.
- `rg -n '\.position\(' skein/crates/skein-forge-connection/src/component/runtime.rs` finds nothing, and `remove` carries
  no `#[expect]` unless clippy still asks for one.
- `heavy cargo nextest run -p skein-forge-connection` selects the same number of tests as your base, and all pass.
- Tests net about -50.

## Inputs

- `temper/docs/plan/debt-16/notes/common.md`
- `skein/docs/foundation/testing-strategy.md`, 1.

## Owns and may change

- **Owns:** `skein-forge-connection/src/tests/fixtures.rs` and every reader of what it changes; `component/runtime.rs`
  (`remove`'s position, nothing else).
- **Readers of `ConnectionHarness::start` and `forgejo_exchange`, spelling only:** `tests/connection.rs`,
  `tests/dialects.rs`, `tests/reads.rs`, `tests/github_start.rs`, `connection/tests.rs`, `connection/tests/transport.rs`.
- **Readers of `github_progress` and `github_local`, spelling only:** `component/github/tests.rs`.
- **The moves, content unchanged:** `connection/tests.rs` to `connection/github/tests.rs`; `connection/tests/forgejo.rs`
  to `connection/forgejo/tests.rs`; `connection/tests/transport.rs` to `connection/tests.rs`. Each parent's `mod tests;`
  loses its `#[path]`. Their module paths don't change, so `super::*` still reads. Do these moves in a commit of their own,
  with no edit, so `git log --follow` reads them.
- **Not yours:** `skein-fake-forge-http`'s own `#[path]`, another crate.
- Every assertion stays as it is. If a test depends on the harness holding both exchanges, keep them both.

## Steps

1. `ConnectionHarness`:

   ```rust
   pub fn start_forgejo(
       &mut self,
       request: skein_forge::forgejo::request::Request,
       range: Option<skein_forge::forgejo::plans::plan::Range>,
   ) -> Result<(), &'static str>
   pub fn start_github(
       &mut self,
       exchange: skein_forge::github::plans::plan::Exchange,
       authorization: Option<Secret>,
   ) -> Result<(), &'static str>
   ```

   `start_forgejo` builds the authorization `forgejo_exchange` builds. A test that needs another authorization may take a
   `start_forgejo_with(.., authorization)`. `forgejo_exchange`, the `Exchange` enum and `start` go.

2. `FixtureHeard` goes, and `heard`'s and `answer_dialect`'s dispatch on it. `answer` and `github_answer` each read their
   own exchange's `heard`, around a shared loop that drives the wire and holds no event. `pump` and `ready` may keep their
   dispatch on `forge.kind`: they yield no dialect's type.

3. `request_bytes` calls `start_forgejo`, and its `let .. else unreachable!` goes.

4. `github_progress` and `github_local` lose `_at`, with their call sites.

5. The three moves.

6. `component/runtime.rs`: a `for` loop finds the position, as `find` does. If clippy has no more objection, the
   `#[expect]` on `remove` goes.

7. `cargo fmt`; `python3 -I -B skein/scripts/old_forms.py`; `heavy cargo clippy -p skein-forge-connection --all-targets --
   -D warnings`; the citations check. Commit and push.

## Report

The common note's items. Tests only move or change spelling, so nothing to break by hand; give the test count before and
after.
