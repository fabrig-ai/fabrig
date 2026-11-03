+++
id = "tokens-stop"
part = "fixes"
kind = "feature"
needs = []
files = [
  "temper/crates/temper-client-domain/src/forge/tokens.rs",
  "temper/crates/temper-client-domain/src/tests/forgejo_tokens.rs",
]
effort = "high"
minutes = 60
+++

# A connected Forgejo's tokens draft is emptied only by its own connect's stop

## Goal

`temper-client-domain`'s `forge/tokens.rs` `stopped` empties the shared tokens draft whenever a
connected-mode Forgejo's row is Stopped, or NotConnected with `ended` set, and it runs on every
forges watch delivery (`forge/forges.rs` ~938). So after a failed connect, tokens pasted again are
lost on the next delivery, another forge's update included; and with no connect sent, any stopped
connected-mode Forgejo empties a draft being prepared for a new forge. The design says the three
fields are emptied as the connect leaves and "a stop empties them again"
(`temper/docs/design/client/forge.md`, 5.4): the sent connect's own stop. The sent connect keeps
what it has seen of its forge's row, and the draft is emptied only on that connect's new stop.

Acceptance:

- `stopped` reads only the row of the sent connect's forge (`Pending::forge`); with no connect
  sent (`side.tokens` is `None`), or one still at forge 0 (a new forge not yet numbered), no row
  empties the draft.
- The draft is emptied, and an unanswered connect answered `Connecting` (as now), only when that
  row shows a stop the connect has not seen: the row is stopped (Stopped, or NotConnected with
  `ended` set, as now) and either the connect last saw it not stopped or its `ended` words differ
  from those last seen. That stop settles the connect: no later row empties the draft for it.
  The row Connected settles it too; so does an answer that starts no connect (any but
  `Connecting` and `Unknown`).
- After a failed connect, a fresh paste survives the stopped row's redelivery, alone or beside
  another forge's row: its value stays and `written` is unchanged.
- A stopped connected-mode Forgejo leaves a draft being prepared for a new forge untouched.
- Kept: the fields emptied at send and at the answer; a new stop of a sent connect empties them
  and returns `ConnectForge` to Details; sign-out empties them.
- The tests in Steps pass, each count not zero; clippy is clean.

## Inputs

- `temper/docs/plan/fab3-fixes/notes/common.md`.
- `temper/AGENTS.md`.
- `temper/docs/plan/fab3-fixes/inputs/request.md`, finding 1.
- `temper/docs/design/client/forge.md`, 5.4: *A connected Forgejo* ("empties the three fields as
  it leaves; a stop empties them again") and the paragraph "A pasted value is a field of its
  own".
- For the shape only: `temper-client-domain`'s `forge/replace.rs` `stopped` (~257), which keeps a
  row draft's last seen `stopped` and `ended` and clears on their change. This task does not
  touch it.

## Owns and may change

- `temper/crates/temper-client-domain/src/forge/tokens.rs`: `Pending`, `send_target`, `told`,
  `stopped`, and a private enum and helper they need.
- `temper/crates/temper-client-domain/src/tests/forgejo_tokens.rs`: the tests below, and its
  helpers `row` and `deliver` reshaped as they need (a forge's number, the `ended` words, several
  rows in one delivery); every existing test keeps its assertions.

Nothing else. `forge/forges.rs` keeps calling `super::tokens::stopped(side)` as it is, and
`Pending`'s public fields (`forge`, `key`, `answer`), which `temper-client-view`'s
`forge/connect.rs` reads, stay as they are.

Pre-approved options:

- `Pending` gains a private field, so it takes `#[expect(clippy::partial_pub_fields, reason =
  "...")]` as `Draft` does (the workspace denies the lint).
- If a test in `temper-client-view` or `temper-client-world` (Steps, 3) fails because it relied
  on a row emptying the draft with no connect sent, or on an unchanged stop's redelivery emptying
  it, stop and ask: that would be a reading of the design this task does not settle.
- A stop the connect cannot tell from the one it last saw (the same `connection` and the same
  `ended` words, no other state delivered between) is not a new stop: the draft was emptied at
  send and at the answer already. Do not add a request id or a counter to the row for it; say so
  in the report if a test shows it.
- The admin form's `forge/admin.rs` `stopped` has the same shape; it is not this task's. Leave
  it and name it under `## Left`.

## Steps

1. In `forge/tokens.rs`:

   ```rust
   /// What a sent connect has seen of its forge's row: the stop that empties the fields
   /// again is its own (temper/docs/design/client/forge.md, 5.4).
   #[derive(Debug, PartialEq, Eq)]
   enum Seen {
       /// Not yet stopped or connected: the row's stop and its last connect's words, as last delivered.
       Open { stopped: bool, ended: Option<Box<[u8]>> },
       /// Stopped once, connected, or answered without a connect: no later row empties the fields.
       Settled,
   }

   #[derive(Debug)]
   #[expect(clippy::partial_pub_fields, reason = "...")]
   pub struct Pending {
       pub forge: u16,
       pub key: Key,
       pub answer: Option<github::Answer>,
       seen: Seen,
   }
   ```

   - `send_target`: a kept forge's `Seen::Open` takes the row it checked (`stop(row)` and
     `row.ended`); a new forge's starts `Open { stopped: false, ended: None }`, so the first
     stopped row of its number, once `forges.rs`'s `bind_new` gives the connect that number, is
     its stop.
   - `told`: where the answer is first set, an answer other than `Connecting` and `Unknown`
     settles `seen` (an exhaustive match over `github::Answer`).
   - `stopped(side)`: with `side.tokens` `None`, or `pending.forge == 0`, or no row of that
     number (`super::connect::row(pending.forge, &side.forges, None)`), or `seen` settled, return.
     Otherwise, with `now = stop(row)`: if `now` and (`!stopped` or `row.ended != ended`), empty
     the draft (`side.tokens_form.clear()`), answer an unanswered connect `Connecting`, and
     settle; else if the row is `Connected`, settle; else `seen = Open { stopped: now, ended:
     row.ended.clone() }`.
   - `fn stop(row: &forges::Row) -> bool`: today's test, one exhaustive match (`Stopped` true;
     `NotConnected`, `ended.is_some()`; the rest false).

   The crate is `no_std` with `alloc`: `use alloc::boxed::Box;` as `replace.rs` does.

2. Tests in `src/tests/forgejo_tokens.rs` (the crate's lib tests, `tests::forges::forgejo_tokens`):

   - `a_stop_clears_all_fields_and_returns_to_details_before_the_answer` (~202), extended: after
     the fresh paste (~218), record `written`, then deliver the same stopped row again, then the
     stopped row beside another forge's row (a second connected-mode Forgejo, any connection);
     after each, `has(Field::Api)` and `written` unchanged. Then paste the git token, send again
     (the row offers it: NotConnected with `ended`), deliver the row with other `ended` words:
     every field empty, `written` raised, Details shown. Then the sign-out assertions, as now.
   - `a_stopped_row_redelivered_keeps_a_fresh_paste`, over both stops: from `opened()` (forge
     NotConnected, no `ended`), send, deliver the stop (a) NotConnected with `ended`, (b) Stopped
     with a reason and no `ended`: emptied; paste the API token; deliver the same row again:
     kept, `written` unchanged.
   - `an_unrelated_stopped_forgejo_leaves_a_new_forges_draft`: `connect::opened(true)`, deliver a
     stopped connected-mode Forgejo (both stops, as above); paste both tokens for a new forge;
     deliver the stopped row again, then with other `ended` words: both tokens kept, `written`
     unchanged; write the place's host and `Act::Forges(forges::Act::ConnectTokens)` sends one
     Connect (as `new_connected_forgejo_uses_the_shared_place_and_needs_an_address` does).
   - `a_stop_of_another_forge_leaves_the_sent_connects_draft`: send to `FORGE`, answer
     `Connecting` (as `the_new_forge_follows_its_number_in_either_watch_answer_order` builds the
     answer), paste the API token, deliver `FORGE` Connecting beside another connected-mode
     Forgejo newly stopped: the token kept; then `FORGE` stopped: emptied.

3. Narrow tests, one at a time:
   - `heavy cargo nextest run -p temper-client-domain --lib -E 'test(/forgejo_tokens/)'`
   - `heavy cargo nextest run -p temper-client-domain --lib`
   - The readers of the draft's clearing (this changes what it means, not a signature; they
     stand for the crate's direct dependents, which see no public change):
     `heavy cargo nextest run -p temper-client-view --lib -E 'test(/tokens|connect_forge|forges/)'`
     and
     `heavy cargo nextest run -p temper-client-world --test suite -E 'test(/forgejo_tokens|forge_replace|connect_forge|forges/)'`.
4. `cargo fmt`; `python3 -I -B scripts/old_forms.py` from `temper/`;
   `heavy cargo clippy -p temper-client-domain --all-targets -- -D warnings`.
5. One commit, the code and its tests, saying why; push.

## Report

As the implementer's text says, and: the shape `Seen` took if it departs from the sketch; each
test's count; whether a reader's test needed anything; `forge/admin.rs` under `## Left`.
