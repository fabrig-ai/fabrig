+++
id = "jig-sign-in"
part = "jig-sign-in"
kind = "feature"
needs = []
files = [
  "jig/crates/jig-protocol-engine-clients/src/boundary.rs",
  "jig/crates/jig-protocol-engine-clients/src/component.rs",
  "jig/crates/jig-protocol-engine-clients/src/mint.rs",
  "jig/crates/jig-protocol-engine-clients/src/sign_in.rs",
  "jig/crates/jig-protocol-engine-clients/src/tests/**",
  "jig/testing/jig-engine-clients-kit/src/peer.rs",
  "jig/tests/engine-clients/tests/suite/sign_in.rs",
  "jig/docs/design/protocol/client.status.md",
]
effort = "high"
minutes = 75
+++

# jig's clients' component takes a given local secret and says when the printed link signed in

## Goal

Two additions to jig's clients' component, for a box that starts its TUI in
front (`temper/docs/design/protocol/setup.md`, 7.6): the startup may give
the first local link's secret, as it gives the printed link's, so that it
can hand that link to the TUI it starts; and the component says
`LinkSignedIn` each time the printed link signs a person in, so that the
application can show that a browser used it.

Acceptance, by `jig/docs/design/protocol/client.md`, 4.2 and section 8
(*The printed link*, its *Signed in by it, said*; *The local link*, *Its
first, given or drawn*; *A secret given*), and `notes/contracts.md`,
section 2:

1. **`local_secret`:** `Settings` gains `pub local_secret: Option<Secret>`
   beside `link_secret`; `Unusable` gains `LocalSecret`. `Component::new`
   takes it as it takes `link_secret`: refused `LocalSecret` for any length
   but `LINK_SECRET_BYTES`, or when it equals the given `link_secret`,
   before anything is said; where `settings.link` is none it is dropped;
   otherwise it is the local slot's live link, its `until` `link_span` from
   `env.now`, and no `LocalLink` is said for it (in `fresh_link`'s place for
   `Which::Local`). Spent, the next local link is drawn and said as today.
2. **`LinkSignedIn`:** `Request` gains `LinkSignedIn` (no fields), after
   `LocalLink`. In `mint.rs`' `sign_in_reply`, the `people::Reply::SignedIn`
   arm says it once when the reservation it spends was the printed link's
   (`spent` holding `Which::Printed`), whether the sign-in was made or a
   held one kept, before `spend_link`. Nothing says it for the local link, a
   `Mint`, a service's sign-in, an approval's take, or any other reply.
3. **Every exhaustive `match` on `Request`** gains its arm, in jig and in
   temper (compiler-forced, allowed in both trees by the implementer's
   text): `jig/testing/jig-engine-clients-kit/src/peer.rs` counts it (a
   `link_signed_in_count()` beside `local_link_count()`, for the tests
   below); `jig/testing/jig-test-service/src/clients.rs`,
   `jig/testing/jig-client-kit/src/joint/component.rs`,
   `jig/testing/jig-client-protocol-kit/src/engine.rs`,
   `jig/tests/engine-clients/tests/suite/provider.rs`,
   `jig/tests/ops/src/joint/bytes.rs`, `temper/tests/client/src/component.rs`
   and `temper/crates/temper-service/src/clients.rs` drop it (temper-service's
   body is task `serve-sign-in`'s). `Settings` literals that the compiler
   forces gain `local_secret: None` (jig's fixtures, `temper-service`'s
   `clients.rs`, `temper/tests/client/src/component.rs`, and any other
   `Settings { .. }` built without `..fixtures::settings()`).
4. **Tests** in `jig/tests/engine-clients/tests/suite/sign_in.rs`, a failing
   test first for each behaviour:
   - one new test, `a_given_local_secret_signs_in_as_the_first_local_link_and_no_local_link_is_said`:
     started with both secrets given, `local_link_count()` is 0 and
     `link_count()` 0; the given local link signs the first owner in; then
     one `LocalLink` is said, a link differing from both given; the given
     printed link still signs in after it; the given local link sent again
     is answered `LinkUsed`; it expires `link_span` from its taking (as
     `a_given_secret_expires_a_span_from_its_taking` reads, one step
     through).
   - rows in `a_given_secret_of_another_length_is_refused_at_start`: a
     `local_secret` of each wrong length, and one equal to `link_secret`,
     refused `Unusable::LocalSecret` with nothing said.
   - a row (or assertion) in `a_given_secret_without_the_printed_link_is_no_link`:
     `local_secret` given with `link: None` is dropped, no local link.
   - `LinkSignedIn`: in `the_printed_and_the_local_link_each_sign_in_once_and_leave_the_other`,
     each case asserts `link_signed_in_count()` grows by one at the printed
     link's sign-in and never at the local link's;
     `a_live_credential_keeps_the_sign_in_without_minting_and_spends_the_link`
     asserts it said once for a kept sign-in by the printed link;
     `a_spent_link_is_used_and_a_fresh_one_printed` asserts the `LinkUsed`
     refusal says nothing more; `a_mint_for_an_application_provider_signs_in_as_the_link_does`
     asserts a mint says nothing.
5. `jig/docs/design/protocol/client.status.md`: delete the **Not yet**
   bullet naming `jig-sign-in`. No other design text changes.

## Inputs

- `temper/docs/plan/tui-box-front/notes/common.md`; `notes/contracts.md`,
  section 2.
- `jig/docs/design/protocol/client.md`, 4.2 (the startup's messages) and
  section 8, written by this plan's cut.
- The code: `boundary.rs:82-112` (`Request`), `:213-217` (`Settings`,
  `link_secret`); `component.rs:32-36` (`Unusable`), `:53-138`
  (`Component::new`: `given`, the length check, the printed slot's live
  link, `fresh_link` for both), `:182-193` (`fresh_link`), `:216-224`
  (`spend_link`); `mint.rs:52-80` (`sign_in_reply`, `take_reservation`, the
  `SignedIn` arm, `held`); `sign_in.rs` (`Which`, `Slot`, `Link`). The
  world: `jig/tests/engine-clients/tests/suite/sign_in.rs:1-104` and
  `:696` on; its kit `jig/testing/jig-engine-clients-kit/src/peer.rs:175-176,
  243-244, 452, 747-751`.

## Owns and may change

Owns the component's `local_secret` and `LinkSignedIn`, the kit's count,
the tests above and the status edit. The other `match` arms and `Settings`
literals of acceptance 3 are reader edits, each the arm or the field alone.
Not `jig-client` or the wire: nothing on the wire changes. Not temper's
design or its behaviour: temper-service's arm drops the request.

Options, each decided:

- **Which `spent` means printed:** `take_reservation` already returns the
  slot's kind; if it doesn't, carry `Which` with the reservation where
  `spend_link` reads it, and say so.
- **A held sign-in kept** (`held == Some(sign_in)`) says `LinkSignedIn` too:
  a browser that opened the link with a live cookie has used it.
- **The kit's peer** records `LinkSignedIn` as a count; no other kit
  exposes it.

## Steps

1. The failing tests, then `boundary.rs`, `component.rs`, `mint.rs`, then the
   arms and literals; the narrowest test after each:
   `heavy cargo nextest run -p jig-engine-clients-world --test suite -E 'test(/sign_in::/)'`
   (check the package's name with `cargo metadata`; a run that selects
   nothing proves nothing).
2. `heavy cargo nextest run -p jig-protocol-engine-clients`, then
   `-p jig-engine-clients-world`; `heavy cargo check --tests` for each crate
   whose arm or literal you touched (`-p jig-test-service -p jig-client-kit
   -p jig-client-protocol-kit -p jig-ops-world -p temper-service
   -p temper-client-world`, names checked with `cargo metadata`), and
   `heavy cargo nextest run -p temper-service -E 'test(/clients/)'` since its
   settings literal changed. `cargo fmt`;
   `python3 -I -B jig/scripts/old_forms.py` and
   `python3 -I -B temper/scripts/old_forms.py`; clippy naming every package
   changed; `python3 -I -B tools/citations/citations.py --base origin/main`.
3. The status edit; commit (why: a box can hand its TUI the first local
   link, and its TUI can show that a browser used the printed link) with the
   trailer `Look-request: tui-box-front`; push.

## Report

As the implementer's text says, with each test added or extended and the
rule broken by hand for it, and every reader arm and literal changed, by
file.
