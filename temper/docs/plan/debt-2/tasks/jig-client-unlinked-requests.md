+++
id = "jig-client-unlinked-requests"
part = "jig-client-modes"
kind = "interface"
needs = []
files = [
  "jig/crates/jig-client/src/**",
]
effort = "high"
minutes = 50
+++

# jig D018, step 1: the unlinked client's own request type

## Goal

`jig-client` gains `UnlinkedRequest` and the unlinked mode's own pair, `step_unlinked` and
`fire_unlinked` (`jig/docs/design/client/domain/client.md`, 4.4, written by this cut), as
additions: `step`, `fire` and `Request` are unchanged for now, so every caller keeps
building while the next tasks move them. This is the surface of
`jig/docs/development/debt/D018-native-shell-unlinked-panics.md`; the item is paid by
`jig-linked-requests`.

Acceptance:

- `jig_client::UnlinkedRequest`, `Client::step_unlinked` and `Client::fire_unlinked` exist as
  `temper/docs/plan/debt-2/notes/contracts.md` sketches them; each of the pair asserts the
  client is unlinked.
- The client's unlinked routing writes `UnlinkedRequest`; `step` and `fire` given an unlinked
  client give the same `Request`s, in the same order, as before, through one private mapping
  from `UnlinkedRequest` to `Request` that `jig-linked-requests` deletes.
- `jig-client`'s own tests that step an unlinked client step it with the new pair and match
  `UnlinkedRequest`; a test of `step` on an unlinked client that exists only to pin the old
  form may stay until `jig-linked-requests`.
- No file outside `jig/crates/jig-client` changes. The tests in Steps pass, each count above
  zero; clippy is clean.

## Inputs

- `temper/docs/plan/debt-2/notes/common.md` and `temper/docs/plan/debt-2/notes/contracts.md`
  (its jig section binds the names).
- `jig/AGENTS.md` (step code: no generics or traits of its own; exhaustive matches).
- `jig/docs/development/debt/D018-native-shell-unlinked-panics.md`, whole.
- `jig/docs/design/client/domain/client.md`, 4 and 4.4 (4.4 written by this cut).
- `skein/docs/foundation/programming-model.md`, 5.4 (exhaustive matches, `unreachable!` with
  its reason for impossible cells).

## Owns and may change

- `jig/crates/jig-client/src/client.rs`, `lib.rs` (the export line), and the crate's
  `src/tests*` files that step an unlinked client (`tests.rs`, `tests/unlinked.rs`,
  `tests/sign_in.rs`, and any other the compiler or `rg 'unlinked' jig/crates/jig-client/src`
  finds).
- The private method today named `step_unlinked(&mut self, env, event, out)` takes another
  name (`unlinked_event`, say), since the public one takes its name.

Nothing outside the crate changes: no reader is edited in this task.

Pre-approved:

- Inside, the choices are yours (a private sink that both modes' routing pushes into, or two
  routing paths); no trait and no type parameter (jig's step code rules). Say in the report
  which you chose.
- If the unlinked mode also makes a request today that `UnlinkedRequest` lacks (the survey
  found `Told`, `Authorized(Lost)`, `Down` and `SignOut` only: `client.rs`, ~351, ~355, ~494,
  ~495), stop and ask, naming the site.

## Steps

1. Sketch (the contract's):

   ```rust
   /// What an unlinked client asks of its host (client/domain/client.md, 4.4).
   #[derive(Debug, PartialEq, Eq)]
   pub enum UnlinkedRequest {
       Told { told: Told },
       Authorized(crate::Authorized),
       Down { down: Down },
       SignOut,
   }

   impl Client {
       /// Step an unlinked client (client/domain/client.md, 4.4).
       pub fn step_unlinked(&mut self, now: Time, wall: Wall, event: Event, out: &mut Queue<UnlinkedRequest>) {
           assert!(self.link.is_none(), "a linked client steps with step");
           // ...
       }
       /// Fire an unlinked client's deadlines (client/domain/client.md, 4.4).
       pub fn fire_unlinked(&mut self, now: Time, wall: Wall, out: &mut Queue<UnlinkedRequest>);
   }
   ```

   Tests (in `src/tests/unlinked.rs` or beside it): an unlinked client stepped with the pair
   asks `Down`, `SignOut`, `Told` and `Authorized(Lost)` as today's tests already pin for
   `step`; `step_unlinked` given a linked client panics with "a linked client steps with
   step" (`#[should_panic(expected = ...)]`), and so does `fire_unlinked`.
2. Narrow tests, one at a time, each count above zero:
   - `heavy cargo nextest run -p jig-client --lib`
   - `heavy cargo nextest run -p jig-client-test-domain --lib` (an unlinked root, still on
     `step`: unchanged)
   - `heavy cargo nextest run -p jig-client-shell-native`
3. `cargo fmt`; `python3 -I -B scripts/old_forms.py` from `jig/`;
   `heavy cargo clippy -p jig-client --all-targets -- -D warnings`.
4. `python3 -I -B tools/citations/citations.py --base origin/main` from the repository root.
5. One commit ("jig-client gives the unlinked mode its own requests (client.md, 4.4)"),
   pushed.

## Report

As the implementer's text says, and also: how the routing reaches either queue; the private
mapping's name; the tests moved and added, with counts.
