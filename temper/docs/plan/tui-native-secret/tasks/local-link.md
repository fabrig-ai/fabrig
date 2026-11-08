+++
id = "local-link"
part = "jig-local"
kind = "feature"
needs = []
files = [
  "jig/crates/jig-protocol-engine-clients/src/boundary.rs",
  "jig/crates/jig-protocol-engine-clients/src/component.rs",
  "jig/crates/jig-protocol-engine-clients/src/sign_in.rs",
  "jig/crates/jig-protocol-engine-clients/src/mint.rs",
  "jig/crates/jig-protocol-engine-clients/src/closing.rs",
  "jig/crates/jig-protocol-engine-clients/src/tests/**",
  "jig/testing/jig-engine-clients-kit/src/peer.rs",
  "jig/tests/engine-clients/tests/suite/sign_in.rs",
  "jig/tests/engine-clients/tests/suite/provider.rs",
  "jig/testing/jig-client-protocol-kit/src/engine.rs",
  "jig/testing/jig-client-kit/src/joint/component.rs",
  "jig/testing/jig-test-service/src/clients.rs",
  "jig/tests/ops/src/joint/bytes.rs",
  "jig/docs/design/protocol/client.status.md",
]
effort = "high"
minutes = 150
+++

# jig's clients' component keeps a local link beside the printed link

## Goal

The request's change in jig (`temper/docs/plan/tui-native-secret/inputs/request.md`, *The change*, its first three bullets, and *Tests*, jig's worlds). Today `jig-protocol-engine-clients`' `Component` keeps one live link, the printed link, and a TUI that signs in by the engine's link file spends that very link. Give the component a second one-time link, the local link, live beside the printed link: drawn at start and said to the application as `Request::LocalLink { secret }`, signed in by the same route, drawn again at once when spent, and never making the startup print. temper's serve keeps it in the link file (task `serve-file`); temper's client world reads it (task `world-file`).

Acceptance, by `jig/docs/design/protocol/client.md`, 4.2 and section 8, *The printed link* and *The local link*, and `notes/contracts.md`, sections 1 and 2:

1. **`Request::LocalLink { secret: Secret }`** in `boundary.rs`, beside `Link`, with the doc comment `notes/contracts.md`, section 1 gives. The start's requests, where the printed link is enabled: `Link` (none when `link_secret` was given), then `LocalLink`, then the public names' `Secrets(Get)`; where it isn't, neither link.
2. **Two links, each apart:** every rule of `notes/contracts.md`, section 2, items 1 to 8, holds. In particular: signing in by one leaves the other live and signing in; a reserved link answers its own secret busy while the other signs in; not ready, busy and full give the reserved link back to its own place; a spent local link is drawn again at once (`LocalLink`), never in close or abort; a sign-in by a secret remembered as a local link's never draws a printed link; a sign-in whose secret isn't live draws a fresh local link when none is live or reserved, whatever the secret; `fire` and `next_deadline` cover both.
3. **The tests,** a failing test first for each behaviour (`notes/common.md`, *Tests stay cheap*):
   - `jig/tests/engine-clients/tests/suite/sign_in.rs` (jig's engine-clients world), through `Peer`:
     - one new table test, `the_printed_and_the_local_link_each_sign_in_once_and_leave_the_other`, rows `printed first` and `local first`: at start one printed and one local link said, and they differ; the first signs in; the second still signs in after it; each tried again answers `LinkUsed`; spending the local link says the next local link at once, which differs and signs in, with `link_count()` unchanged; spending the printed link says no local link (`local_link_count()` unchanged). A third row, `crossing`: the printed link's sign-in held reserved (`pending_reply`) while the local link signs in, then the printed one answered and signed in.
     - one new table test, `a_local_links_dead_secret_never_prints_a_link`, rows `used` (a spent local link tried) and `expired` (the local link tried past `link_span`): answered `LinkUsed` / `LinkExpired`; `link_count()` unchanged; for `expired`, a fresh local link said, which signs in.
     - one new test, `a_restart_forgets_both_links`: both links, then `peer.restart(&settings)`; each old link answers `LinkUnknown`; the restarted component said a new printed and a new local link, each differing from the old ones, each signing in.
     - **never printed:** in each of the tests above, no printed link said (`Peer`'s `links`) equals any local link said; and `no_core_event_holds_a_token_or_digest` gains the local link's sign-in, whose core record is the same `SignIn` with no secret in it (a row, or a second assertion in it).
     - the existing tables gain rows where they fit: `busy_and_full_return_the_same_printed_link_and_it_can_sign_in` a local link row (full carries the local link's expiry); `a_link_that_cannot_sign_in_answers_why` is left as it is unless a row is the cheaper home for one of the above.
   - `jig/crates/jig-protocol-engine-clients/src/tests/closing.rs`: `linked_component` takes `Link`, then `LocalLink`, then the public names; `close_and_abort_drop_a_links_reservation_even_before_its_core_answer` (or a row of it) shows that close and abort with a local link reserved say no `LocalLink`. Other unit tests that read the start's requests in order take the new one where they read them; a unit table that pins a drawn value is re-pinned, and the report names each.
4. **Readers, compiler-forced and no more:** `jig-client-protocol-kit`'s `engine.rs`, `jig-client-kit`'s `joint/component.rs`, `jig/tests/ops`' `joint/bytes.rs` and `jig-test-service`'s `clients.rs` drop the secret (`Request::LocalLink { .. } => {}`, or its `Routed` equivalent dropping it, with a comment citing `protocol/service.md`, 7 in `jig-test-service`); `provider.rs` lists `Request::LocalLink { .. }` beside `Request::Link { .. }`. In temper, `temper/crates/temper-service/src/clients.rs` and `temper/tests/client/src/component.rs` take the arm `Request::LocalLink { .. } => {}` (as each spells `Request`), with no other line changed: `serve-file` and `world-file` replace it.
5. **Status:** delete the **Not yet** bullet naming `local-link` in `jig/docs/design/protocol/client.status.md`. No other design line.
6. **Runs,** through `heavy`, counts in the report: `cargo nextest run -p jig-protocol-engine-clients`, `-p jig-engine-clients-world` (`jig/tests/engine-clients`), `-p jig-engine-clients-kit`, `-p jig-client-protocol-kit`, `-p jig-client-kit`, `-p jig-test-service`, `-p jig-ops-world` (`jig/tests/ops`); `cargo check -p temper-service -p temper-client-world --tests`. Then, as readers of a change of meaning (the start now draws twice), `cargo nextest run -p temper-service` and `-p temper-client-world`: a red there is reported under `## Left` with its test and log line, not fixed (it is `serve-file`'s or `world-file`'s).

## Inputs

- `temper/docs/plan/tui-native-secret/notes/common.md` and `notes/contracts.md`.
- `jig/docs/design/protocol/client.md`, 4.2 (the messages to the startup) and section 8, *The printed link*, all its sub-bullets, *The local link* first.
- `jig/docs/design/protocol/service.md`, section 7 (`jig-test` drops the local link).
- The component's link today: `component.rs`, `Component::new` (about line 129, the given secret or `fresh_link`), `remember_link`, `link_answer`, `fresh_link` (about lines 145 to 177), `Route::Link` (about lines 469 to 519), `sign_in_refused` (about line 738), `next_deadline` and `fire` (about lines 812 to 855); `mint.rs`, `return_link` and `sign_in_reply` (about lines 44 to 145); `closing.rs`, lines 124 and 199; `sign_in.rs`, `Past`, `Link`, `PAST_LINKS`.
- The world: `jig/tests/engine-clients/tests/suite/sign_in.rs`, its helpers `finish_given_sign_in`, `pending_sign_in`, `pending_reply`, `decoded`, `fresh_link`, `retry_answers`; `Peer` in `jig/testing/jig-engine-clients-kit/src/peer.rs` (`links`, `link_count`, `printed_link`, `post_link`, `advance_time`, `restart`, about lines 175, 449 and 660 to 745).

## Owns and may change

Owns `jig-protocol-engine-clients`' link state, its `Request::LocalLink`, its unit tests; `Peer`'s `local_links`, `local_link_count` and `local_link`; the new and changed stories of `sign_in.rs`; the status bullet. May change, each only as acceptance 4 says: the readers' arms in jig's kits, `jig-test-service`, the ops world and `provider.rs`, and the one arm in each of `temper/crates/temper-service/src/clients.rs` and `temper/tests/client/src/component.rs` (a variant the producer's branch carries into the other tree, `AGENTS.md`'s exception for a required field). Not the wire, `jig-client`, `jig-protocol-config`, any design text but the status bullet.

Options, each decided:

- **The state's shape inside the component is yours.** A suggested shape: a `Slot { live: Option<Link>, spending: Option<(Token, Link)> }` in `sign_in.rs`, the component holding `printed: Slot` and `local: Slot`, and `Past` entries `(digest, Past, Which)` with `enum Which { Printed, Local }`; `fresh_link` taking the `Which` to draw. If another shape reads better, use it and say why in the report.
- **`fire` takes no `out`:** an expired local link is moved to the past there and drawn again by the next sign-in that finds none live (contracts, section 2, item 5). Don't change `fire`'s signature; if a reader's story needs the next local link before any sign-in, say so in the report rather than drawing in `fire`.
- **A seeded draw moved:** the printed link is drawn first, so its value at a given seed is today's; tokens and codes drawn after the start move by one draw. A jig test pinning one is re-pinned to the new value, and named in the report.
- **`Peer::local_link` with none said** panics `"local link"`, as `printed_link` panics.

## Steps

1. Write the failing stories of acceptance 3 against `Peer` (with `local_links` added first, so they compile), run them, see them fail for the missing link.
2. The variant, the start's order, then the two links' rules in the component, item by item of the contracts' section 2; the unit tests' fallout.
3. The readers' arms (acceptance 4).
4. The narrowest tests after each step; then the runs of acceptance 6; `cargo fmt`; `python3 -I -B jig/scripts/old_forms.py` and `temper/scripts/old_forms.py`; `heavy cargo clippy -p jig-protocol-engine-clients -p jig-engine-clients-kit -p jig-engine-clients-world -p jig-client-protocol-kit -p jig-client-kit -p jig-test-service -p jig-ops-world -p temper-service -p temper-client-world --all-targets -- -D warnings`; `python3 -I -B tools/citations/citations.py --base origin/main`.
5. Commit (why: a local link beside the printed link, so a native client never spends the person's printed link), push.

## Report

As the implementer's text says, with: each test added and the rule broken by hand for it; the state's shape chosen; each unit test whose expected order or drawn value moved; the temper readers' runs and anything under `## Left`.
