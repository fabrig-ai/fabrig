+++
id = "sk-client-remove"
part = "skein-forges"
kind = "feature"
needs = []
files = [
  "skein/crates/skein-forge-client/src/domain.rs",
  "skein/crates/skein-forge-client/src/lib.rs",
  "skein/crates/skein-forge-client/src/calls.rs",
  "skein/crates/skein-forge-client/src/identity.rs",
  "skein/crates/skein-forge-client/src/outbox.rs",
  "skein/crates/skein-forge-client/src/outbox/*.rs",
  "skein/crates/skein-forge-client/src/keep.rs",
  "skein/crates/skein-forge-client/src/keep/*.rs",
  "skein/crates/skein-forge-client/src/tests.rs",
  "skein/crates/skein-forge-client/src/tests/remove.rs",
]
effort = "high"
minutes = 55
+++

## Goal

skein's forge client (`skein-forge-client`) forgets a forge at once:
`remove_forge` drops its writer, every execution copy, lane, call and
flight on its repositories, its watched resources and repository scans
and their alarms, in one decision, erasing the client's durable rows of
it; each logical read it held gets its one terminal; a late answer for
a forgotten flight changes nothing. This is the client's half of
"its writer and its flights at once"
(`temper/docs/design/draft/playground-reset.md`, section 4).

Acceptance:

- After `remove_forge(forge)`: `Request::Erase { key: Key::Writer(forge) }`
  is out if the forge had a writer, and the writer table no longer
  holds it (a later `writer` for it saves a new row); each `Live` and
  `Repository` row of the forge's repositories is erased
  (`Request::Erase`); each read owner with a call on the forge gets one
  `Request::Read { owner, result: Err(Error::NotConnected) }`; no
  `Call`, `Progress` or `Outcome` names the forge after it.
- An entry of the forge is gone from the outbox with no `Outcome` (the
  parent erases its own durable entry in the same decision); a later
  `Event` naming that entry is taken as for an unknown entry.
- A late `Event::Answered` for a forgotten flight's token is ignored;
  a token the client never issued still panics as today, if the
  sequence makes that check exact (step 2).
- The other forges' writers, entries, calls and watches are unchanged;
  the rate budget is the client's whole, unchanged by the removal but
  for the forgotten calls' unspent reservations, released.
- The crate's tests and its world pass.

## Inputs

- `temper/docs/plan/playground-reset-r4/notes/common.md` and
  `temper/docs/plan/playground-reset-r4/notes/contracts.md` (D3).
- `skein/docs/design/forge-client.md`: the writer (7.1), the outbox and
  its lanes, the calls and their one terminal, the keep's watches and
  scans (read the sections the code's module comments cite).
- `temper/docs/plan/playground-reset-r4/notes/surveys/s6-skein-forges.md`,
  section 3 whole.

## Owns and may change

Owned: the files above: `domain.rs` (the new public function and its
re-export in `lib.rs`), `identity.rs` (dropping a writer), `calls.rs`
(forgetting a forge's calls and flights, the late-terminal rule),
`outbox.rs` and `outbox/*.rs` (dropping a forge's execution copies and
lanes), `keep.rs` and `keep/*.rs` (dropping its live resources, scans
and their alarms), `tests.rs` (one `mod` line), the new
`tests/remove.rs`.

Neighbours: `src/boundary.rs` doc comments only; `src/tests/fixtures*`
builders, if a test needs one.

Pre-approved options:

- No `Event`, `Request`, `Stored` or `Key` variant is added (temper
  matches them); `remove_forge` is a function beside `writer`.
- If `Error::NotConnected` is not the error a read on a forge with no
  access gets in this crate, use the one it gets and say so.

## Steps

1. **The function** (`domain.rs`, cited to forge-client.md, 7.1, and
   the draft, 4):

   ```rust
   /// Forgets a forge at once: its writer, entries, calls, flights and
   /// watches, erasing the client's rows of it (the run's draft, 4).
   pub fn remove_forge(d: &mut Domain, env: &Env<Limits>, forge: u16, out: &mut Queue<Request>);
   ```

   Re-exported from `lib.rs` beside `writer`. It removes the writer
   (`identity`), then every outbox `Writing` whose entry's
   `repository.forge` is the forge and the lanes keyed by its
   repositories, then every logical call of the forge in any state
   (queued, waiting, out) from the calls' slab, queues and flights,
   releasing what each holds of the budget, answering each read owner
   once, then the keep's `live` resources and `repos` scans of the
   forge (erasing their rows), and every alarm naming any of them.

2. **Late terminals** (`calls.rs`, `answered`): a token with no flight
   is ignored when the client issued it (`token <= calls.sequence`,
   tokens being sequenced from it), and still panics otherwise. Say in
   the module's comment why.

3. **Tests**, `src/tests/remove.rs`, with `tests/fixtures.rs` and
   `tests/fixtures/*` builders (the configured client of two forges):
   - `removing_a_forge_erases_its_writer_and_a_new_one_saves_again`.
   - `removing_a_forge_drops_its_entries_and_keeps_the_others`: entries
     queued and in flight on both forges; remove one: its entries make
     no `Call`/`Outcome` after, the other's settle as usual.
   - `a_read_on_a_removed_forge_gets_its_one_terminal`.
   - `a_late_answer_for_a_removed_forges_flight_is_ignored`.
   - `a_never_issued_token_still_panics` (`#[should_panic]`, as the
     crate's existing terminal tests do, if any; else skip and say so).
   - `removing_a_forge_erases_its_watch_rows_and_alarms`: a watched
     pull and a scanned repository on the forge; remove: their `Erase`
     requests out, no `Changed` after a fire past their deadlines.
   - `removing_an_unknown_forge_changes_nothing`.
   Narrow: `heavy cargo nextest run -p skein-forge-client --lib -E
   'test(remove::)'`.

4. `cargo fmt`; `heavy cargo clippy -p skein-forge-client --all-targets
   -- -D warnings`; `heavy cargo nextest run -p skein-forge-client`;
   `heavy cargo nextest run -p skein-forge-client-world --test suite`;
   `-p skein-forge-protocol-world --test suite`; `heavy cargo check
   --tests -p temper-engine-domain-forge -p temper-engine-forge-world`.
   Commit and push.

## Report

fab's sections, and `## Interfaces`: `remove_forge`'s signature and
what it emits, in order; the late-terminal rule as built.
