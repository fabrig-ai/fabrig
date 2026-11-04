+++
id = "forge-exchange"
part = "forge-exchange"
kind = "feature"
needs = ["forge-calls", "forge-tail"]
files = ["skein/crates/skein-forge-connection/src/connection.rs", "skein/crates/skein-forge-connection/src/connection/**", "skein/crates/skein-forge-connection/src/component.rs", "skein/crates/skein-forge-connection/src/component/**", "skein/crates/skein-forge-connection/src/tests/**", "skein/crates/skein-forge-connection/src/tests.rs", "skein/docs/development/debt/D013-forge-connection-parallel-lists.md", "skein/docs/development/debt/D020-forge-connection-states.md", "skein/docs/development/debt/D022-oauth-states.md", "skein/docs/design/forge-connection.status.md"]
effort = "high"
minutes = 150
+++

# A connection speaks no dialect, and D013 is paid

## Goal

`Connection` is a stack that carries an exchange, and six two-variant enums wrap that exchange. The mismatch of a plan
and a connection is a panic. Here the connection keeps the stack alone and each holder keeps its exchange, typed in its
own dialect, so that mismatch can't be written. This is the last of D013: the item goes in this branch.

Acceptance:

- `rg -n 'Forgejo|GitHub|github|forgejo|Kind\b' skein/crates/skein-forge-connection/src/connection.rs` shows only
  imports and calls of what's neutral (`skein_forge::plan::Failure`, `decode`, `github::redirect::Origin` in
  `at_origin` by way of `settings`). Name each in your report.
- `rg -n 'enum (Exchange|Heard|Refusal|Request|Decoder|Body)\b' skein/crates/skein-forge-connection/src/connection.rs`
  finds nothing.
- **Paid when** holds. `rg -n 'unreachable!\(.*(Forgejo|GitHub|forgejo|github|dialect)' skein/crates/skein-forge-connection/src -g '!**/tests/**' -g '!tests.rs'` finds nothing.
- The item's **Measure** reads 0 or nothing.
- `skein/docs/development/debt/D013-forge-connection-parallel-lists.md` is gone, and `python3 -I -B tools/debt/debt.py
  skein` no longer lists it. `rg -n 'D013' skein/docs/development/debt/D020-forge-connection-states.md
  skein/docs/development/debt/D022-oauth-states.md` finds nothing.
- `skein/docs/design/forge-connection.status.md`'s **Not yet** entry is gone.
- `heavy cargo nextest run -p skein-forge-connection -p skein-forge-machine-world -p skein-forge-protocol-world -p
  github-conformance -p forgejo-conformance -p skein-fake-peers-tests -p temper-engine-protocol-forge-world` passes,
  the same tests as at base. These are what would show a moved order, so run them before you trust a green crate.
- The crate's production lines end at most 7,400 (7,517 at this plan's base; the same command as `forge-entry`'s).
- `python3 -I -B tools/citations/citations.py --all` reads 0.

## Inputs

- `temper/docs/plan/debt-13/notes/common.md`
- `temper/docs/plan/debt-13/notes/contracts.md`, 3
- `skein/docs/development/debt/D013-forge-connection-parallel-lists.md`
- `skein/docs/development/debt/D020-forge-connection-states.md`, for what's left alone
- `skein/docs/design/forge-connection.md`, 4.5, 7.1 and 8.1
- `skein/docs/design/forge.md`, 4.4
- `skein/docs/foundation/programming-model.md`, 5.4

## Owns and may change

- **Owns:** `skein-forge-connection/src/connection.rs` and `src/connection/` (new `forgejo.rs`, `github.rs`), and every
  place in `src/component/` that starts, pumps or hears a connection.
- **May change:** `src/tests/fixtures.rs`, `src/tests/connection.rs`, `src/connection/tests.rs`, `src/tests/dialects.rs`
  and any test that drives a `Connection` directly: a harness holds an exchange beside its connection, and patterns on
  `Heard::Forgejo(plans::Heard::Done(..))` lose the wrapper. `test_github_handoff` moves to where the request is. What
  each test asserts stays.
- **May change, and only so:** `skein/docs/design/forge-connection.status.md`, the **Not yet** entry goes, and its
  **Departs** text if a name it cites moved. `D020`: "Do it with D013, and with D016 if the shared transport comes
  first." drops the first clause. `D022`: "The same shape as D013:" drops the reference, and the sentence stands.
- **Not to touch:** where deadlines are set, `Phase` and `draining`, `closing` and `written`: that's D020, and this
  branch carries them over as they are. `D030`'s tests. `Connection::role`.
- **Change of meaning:** `pump` keeps its 64 movements and its order, moved to the holder's call. So a run of tokens
  and rows stays within one pump, and calls don't interleave differently. If a world's order moves, stop and ask.
- **Options:**
  - If `Ended` can reach a holder before its queue is empty, `heard` gives the queue first, as the old queue's order did.
  - A connection pumped with no holder (`drain_redirect`, the close in `drive_calls`) has no exchange to feed. A
    `Moved` it reaches is dropped. If it needs more, it's `Connection::drain`, and the report says why.
  - If the two `pump`s share more than their loop's shape, the shared part goes on `Connection`, not a third type.
    Generics and traits are out (`skein/docs/foundation/programming-model.md`, 10).

## Steps

1. `Connection::step` by `notes/contracts.md`, 3: one iteration of today's `pump`. It takes `held`, what `answer_pending`
   said, and answers `Moved`. `http_event` splits: what's neutral stays (`Bytes`, `End`, the raw and tail bodies,
   `Failed`), and `http::Event::Response` comes up as `Moved::Head`, and `Done` as `Moved::Finished`. JSON tokens come up
   as `Moved::Token`. `Connection` exposes to the exchanges, `pub(crate)`: `json_request`, `set_json_next`,
   `send` (the call, its headers and its upload), `begin_json`, `begin_raw(framing, keep, cap, failed_step)`, `discard`
   and `finish(env, reuse, closes)`. Names are yours. `Range` at base is Forgejo's, so the connection keeps its own
   `Keep::{Head(u32), Tail(u32)}`, each exchange maps to it, and the `Range` header is built in Forgejo's `start`.
2. `Ended::{Failed(Failure), Malformed(decode::Malformed)}`, kept on the connection in `heard`'s place: `failed` and
   `malformed` set it, and `ready` reads it.
3. `connection/forgejo.rs` and `connection/github.rs`, each an `Exchange` with `new`, `start`, `pump`, `heard` and
   `ready`, holding today's `request`, `decoder`, `done`, `heard` and, for Forgejo, `after`. `forgejo_response`,
   `raw_request`, `response`'s GitHub arm, `github_raw_request` and `json_event`'s arms move to them. `keep_after` and
   `cut` go on Forgejo's `Exchange`. `Exchange`'s Forgejo and GitHub arms in `start` become the two `start`s: each
   returns its own refusal, so the two `Refusal` arms in `calls.rs` and `github_calls.rs` go. Response dispatch passes the
   request in, so `Request::GitHub(_) => unreachable!` goes.
4. Holders, by `notes/contracts.md`, 3: `exchange` on `Dialect::Forgejo`, `GitHubForge`, `ForgejoCall` and `GitHubCall`.
   Made new at `open_own`, at `assign` and at a re-assign after a redirect.
5. Each `c.pump(env, io)` then `c.heard()` in the component becomes the holder's. `component.rs`'s `drive_forgejo`,
   `github.rs`'s own drive, `calls.rs`'s `pump_call`, `github_calls.rs`'s. The four `Heard` `unreachable!`s go.
6. Tests and fixtures follow. Run `src/tests/connection.rs` first: it's the narrowest check of the movement order.
7. Delete the item, edit D020, D022 and the status entry. Commit that last, on its own.
8. `cargo fmt`, `heavy cargo clippy -p skein-forge-connection --all-targets -- -D warnings`, the tests above, the
   citations, commit, push.

## Report

What changed, the commit, what ran with counts, each exemption of the acceptance searches, the final measure, the
`unreachable!` count (54 at this plan's base), and the net lines, production and tests apart.
