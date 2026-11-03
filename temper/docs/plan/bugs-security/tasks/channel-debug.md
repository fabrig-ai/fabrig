+++
id = "channel-debug"
part = "skein-debug"
kind = "feature"
needs = []
files = [
  "skein/crates/skein-channel/src/boundary.rs",
  "skein/crates/skein-channel/src/frame.rs",
  "skein/crates/skein-channel/src/tests.rs",
]
effort = "medium"
minutes = 45
+++

# skein-channel's opening credential prints as its length

## Goal

skein-channel carries the initiator's opaque credential (`skein/docs/design/channel.md`, 5) in
three records that derive `Debug` and print it: `boundary.rs`'s `Request::Open { credential }`
(~18) and `Event::Opening { credential, .. }` (~78), and `frame.rs`'s `Control::Open { ..,
credential }` (~30). Give the three enums hand-written impls that print the credential as its
length and every other variant and field as derived, as `skein/docs/design/lib.md`, 6 has a
credential print ("a record that holds one prints, logs and traces without it").

Acceptance:

- `Request::Open` prints `Open { credential_len: N }`, `Event::Opening` prints `Opening {
  credential_len: N, lowest: .., highest: .. }`, `Control::Open` prints `Open { magic: ..,
  lowest: .., highest: .., features: .., credential_len: N }`; no byte of the credential.
- Every other variant of the three enums prints exactly as derived, `Request::Send`'s `frame` and
  `Event::Body`'s `body` included (a frame's bytes are this plan's question `byte-carriers`).
- The new tests fail on the old code and pass on the new.
- clippy clean; the crate's lib tests pass.

## Inputs

- `temper/docs/plan/bugs-security/notes/common.md` and `notes/debug.md`.
- `skein/AGENTS.md`.
- `temper/docs/plan/bugs-security/inputs/request.md`, item 4.
- `skein/docs/design/channel.md`, 5 (the opening, "an opaque credential") and the frame table
  (`Open`'s credential, bytes); `skein/docs/design/lib.md`, 6, `Secret`.
- Readers, surveyed: no test asserts these types' `Debug` text, no golden file holds it; the
  crate's tests use `b"key"`; smith sends an empty credential (`smith-host-protocol`
  `component.rs` ~146). `smith-protocol-channel`'s `Component` (~81) derives `Debug` over a
  `Queue<skein_channel::Event>` and needs nothing more.

## Owns and may change

- `skein/crates/skein-channel/src/boundary.rs`: `Request`'s and `Event`'s derives, and their
  hand-written impls, each one exhaustive `match`.
- `skein/crates/skein-channel/src/frame.rs`: `Control`'s derive and its impl.
- `skein/crates/skein-channel/src/tests.rs`: the tests below, appended.

Nothing else: no field changes, so no reader edits.

Pre-approved options:

- If clippy's `missing_fields_in_debug` fires, a scoped `#[expect(clippy::missing_fields_in_debug,
  reason = "lib.md, 6: a credential prints as its length")]`.
- A private helper for the shared arms is yours.

## Steps

1. Tests first, appended to `src/tests.rs` (the crate's lib tests, `tests`), with a made-up
   canary in both forms (`notes/debug.md`), formatting under a scoped
   `#[expect(clippy::disallowed_macros, reason = "...")]`:

   ```rust
   #[test]
   fn debug_of_an_opening_shows_the_credentials_length_never_its_bytes() {
       // Request::Open { credential: CANARY } is exactly "Open { credential_len: 13 }";
       // Event::Opening { credential: CANARY, lowest: 1, highest: 2 } and
       // frame::Control::Open { magic: .., lowest: 1, highest: 2, features: 0, credential: CANARY }
       // contain "credential_len: 13", the canary absent in both forms
   }

   #[test]
   fn debug_of_records_without_a_credential_is_the_derived_form() {
       // Request::Refuse { reason: 1, text: b"busy" }, Event::Ready { .. } and Control::Accept { .. }
       // each equal the literal the old derived impl prints, written from a run on the old code
   }
   ```

   Run `heavy cargo nextest run -p skein-channel --lib -E 'test(/debug_of_/)'` on the old code:
   the first fails, the second passes. Keep the output.
2. The three impls, a line above each citing lib.md, 6 and channel.md, 5.
3. Narrow tests, one at a time:
   - `heavy cargo nextest run -p skein-channel --lib -E 'test(/debug_of_/)'`: 2 passed.
   - `heavy cargo nextest run -p skein-channel --lib`.
4. `cargo fmt`; `python3 -I -B scripts/old_forms.py` from `skein/`;
   `heavy cargo clippy -p skein-channel --all-targets -- -D warnings`.
5. One commit, the impls and their tests, saying why; push.

## Report

As the implementer's text says, and: the failing run on the old code, the passing runs and their
counts.
