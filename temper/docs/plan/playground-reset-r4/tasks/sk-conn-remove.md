+++
id = "sk-conn-remove"
part = "skein-forges"
kind = "feature"
needs = ["sk-conn-add"]
files = [
  "skein/crates/skein-forge-connection/src/component.rs",
  "skein/crates/skein-forge-connection/src/component/*.rs",
  "skein/crates/skein-forge-connection/src/connection.rs",
  "skein/crates/skein-forge-connection/src/tests.rs",
  "skein/crates/skein-forge-connection/src/tests/runtime_remove.rs",
]
effort = "high"
minutes = 60
+++

## Goal

skein's forge component removes a forge at once: `Component::remove`
ends the forge's calls, aborts its connections, drops its access, start,
GitHub state and own work, and forgets it, so that nothing of it is
served again, while every other forge goes on untouched. This is the
component's half of a forge's removal, "ended at once ... no call is
served" (`temper/docs/design/draft/playground-reset.md`, section 4).

Acceptance:

- After `remove(forge)`: `start(forge)` is `None`; each call of the
  forge that had no terminal is answered once, at once,
  `api::Error::NotConnected`, with the cost it spent; a new call on it
  is answered as a call naming an unknown forge is today
  (`component/calls.rs`, `drive_calls`: `Unavailable`); its webhook path answers 404, as an unknown forge's does; a
  connect or installations request naming it is ignored, as for an
  unknown number.
- Every connection of the forge (its calls', its own start or mint
  connection, its delivery log's) is aborted; io's later events for
  them are taken without effect and without a panic; the component
  still closes and counts each settled.
- Answers already held for the owner stay held and are offered as
  before (the owner drops what names an ended forge).
- The other forges' starts, calls and connections are unchanged; a
  forge added after a removal (`add`, from `sk-conn-add`) under a new
  number works; removing an unknown number changes nothing.
- The crate's tests and the worlds that build the component pass.

## Inputs

- `temper/docs/plan/playground-reset-r4/notes/common.md` and
  `temper/docs/plan/playground-reset-r4/notes/contracts.md` (D3: removed is
  ended, then gone).
- `skein/docs/design/forge-connection.md`, sections 3.5 (a replacement
  ends what the old access began: the model for ending a forge's work),
  4.1, 6.1, 8.1, 8.2 and 8.5 (ending).
- `temper/docs/plan/playground-reset-r4/notes/surveys/s6-skein-forges.md`,
  section 1: "Position versus number" and "What runtime
  addition/removal needs", "Removal".
- The merged `sk-conn-add` (its report's `## Interfaces`).

## Owns and may change

Owned: `skein/crates/skein-forge-connection/src/component.rs` (the new
`remove`), `src/component/*.rs` (the position-taking helpers, where a
removal must reach them), `src/connection.rs` (absorbing a removed
forge's late events), `src/tests.rs` (one `mod` line), the new
`src/tests/runtime_remove.rs`.

Neighbours: `src/hooks.rs`, `src/hooks/connection.rs`,
`src/installations.rs`, `src/puller.rs`: a helper that drops one
forge's state, if the removal needs one there; no change to what they
do for kept forges.

Pre-approved options:

- Positions: keep the parallel lists aligned by removing the forge's
  entry from all of them in one place (rebuild each list without it,
  since `skein_lib::List` has no `remove`); the default. If a position
  is held across steps anywhere (a connection or a plan keeping `at`),
  key that holder by forge number instead, and say where.
- The dialect tables (`forgejo_tables`, `github_tables`) keep the
  removed forge's cached rows: numbers are never reused, and their
  clocks evict them. If a test shows a row read back for a removed
  forge, drop its rows with a new `Tables` method in `skein-forge`, and
  say so.
- No `Request` or public enum variant is added (temper matches them);
  `remove` is a method.

## Steps

1. **The method** (`component.rs`, cited to forge-connection.md, 8.5,
   and the draft, 4):

   ```rust
   /// Removes a forge at once: its calls answered `NotConnected`, its
   /// connections aborted, its access, start and GitHub state dropped.
   pub fn remove(
       &mut self,
       env: &Env<Limits>,
       forge: u16,
       up: &mut Queue<Event>,
       io: &mut Queue<skein_io::Request>,
   );
   ```

   In order: find the forge's position (none: return); end each open
   call whose `Waiting.repository.forge` is the forge with its one
   terminal, `Err(NotConnected)` and its spent cost, the way `expire`
   and the revoked path end a call (`Request::Abort`'s loop, the
   connect replacement's `revoked`), releasing its connection; abort
   every connection whose `Connection.forge` is the forge and its own
   token's (`own_tokens`), as `abort` aborts them, keeping their
   settle accounting; drop the GitHub state (not `forget()`: the whole
   box) and the own plan; remove the forge's entry from
   `settings.forges`, `starts`, `github`, `own_tokens` and `own_plans`
   together; `drive`.

2. **Late events.** An io event for an aborted connection of a removed
   forge, and a step of a helper that looked the forge up by number,
   find no forge: each such path returns quietly where it asserted the
   forge exists (`expect("configured forge")` on a number lookup). List
   each assert you turned into a quiet return.

3. **Tests**, `src/tests/runtime_remove.rs`, with `tests/fixtures.rs`'
   driver:
   - `removing_a_forge_answers_its_open_calls_not_connected`: two
     Forgejo forges ready, a call on each in flight; remove the first:
     its call answered `NotConnected` once, the second's call answered by
     its forge's response as usual.
   - `a_removed_forges_late_responses_change_nothing`: remove during a
     call's exchange, then deliver the connection's response and its
     close: no second terminal, no panic, the component closes
     settled.
   - `a_call_on_a_removed_forge_is_answered_as_an_unknown_forges`.
   - `a_removed_github_forge_drops_its_installations_and_log_work`: a
     GitHub forge ready with an installation and its delivery log
     polling (after `github_logs.rs` and `puller.rs`' fixtures); remove:
     its own connections aborted, no more requests on them, no
     `Installation` or `GitToken` event after.
   - `a_removed_forges_webhook_path_answers_not_found` (after
     `hooks.rs`' tests).
   - `held_answers_outlive_their_forges_removal`.
   - `a_forge_added_after_a_removal_starts` (remove 1, `add` 3, start
     to ready, call answered).
   - `removing_an_unknown_forge_changes_nothing`.
   Narrow: `heavy cargo nextest run -p skein-forge-connection --lib -E
   'test(runtime_remove::)'`.

4. `cargo fmt`; `heavy cargo clippy -p skein-forge-connection
   --all-targets -- -D warnings`; `heavy cargo nextest run -p
   skein-forge-connection`; `-p skein-forge-machine-world --test suite`;
   `-p skein-forge-protocol-world --test suite`; `heavy cargo check
   --tests -p temper-service -p temper-engine-protocol-forge-world -p
   forgejo-conformance`. Commit and push.

## Report

fab's sections, and `## Interfaces`: `Component::remove`'s signature,
any helper made `pub`, the asserts turned into quiet returns (step 2),
and whether the tables' rows were dropped (second option).
