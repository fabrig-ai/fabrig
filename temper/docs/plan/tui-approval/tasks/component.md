+++
id = "component"
part = "jig-approval"
kind = "feature"
needs = ["surface"]
files = ["jig/crates/jig-protocol-engine-clients/src/**"]
effort = "high"
minutes = 210
+++

## Goal

The clients' component holds sign-ins approved by a browser, whole: asked, read and decided by a browser alone, taken with the secret, bounded, expiring and remembered, as `jig/docs/design/protocol/client.md`, section 8, *A sign-in approved by a browser*, and `temper/docs/plan/tui-approval/notes/contracts.md`, 3.2's table say. Only the take makes a sign-in, through people's `Whose` and the core's ordinary `SignIn`; jig's people child and core are untouched.

Acceptance:

- With `Settings.approval` true, every row of contracts 3.2 is answered as the table says, each pinned by a test row; with it false, the `surface` task's not-enabled answers stand.
- A take of an approved approval sends people `Whose { sign_in }` naming the approving sign-in, then the core's `SignIn { identity, held: None }` with the `Signer`'s identity, and answers `signed_in` with a fresh 40-byte credential only after its digest is kept, as the link's sign-in does; a second take is `used`.
- An approval never taken, past its span, ends expired with no core event ever sent for it.
- Not ready, busy and full leave the approval approved with its expiry; full's `until` is that expiry's wall time.
- No `Debug` of the component, its pending exchanges, its requests out or its core events shows an approval's secret or a drawn credential (a canary test).
- `next_deadline` includes the earliest live approval's end; `fire` moves it to the past as expired.
- jig's people, core and their tests are unchanged.

## Inputs

- `jig/docs/design/protocol/client.md`, section 8: *A sign-in made*, *The printed link* (its reservation, its past links, *Not ready, busy and full spend nothing*, *Full*), *A sign-in approved by a browser* whole; 4.2; 17.3, *The credential's carrier, told to the component*.
- `jig/docs/design/domain/people.md`, section 3: *Whose sign-in*, and the third of *Three ways of signing in are jig's own*.
- `temper/docs/plan/tui-approval/notes/contracts.md`, sections 1 to 3 (binding).
- `temper/docs/plan/tui-approval/notes/common.md`.
- The precedents in this crate: the printed link (`component.rs`, `body`'s `Route::Link` arm, `remember_link`, `link_answer`; `sign_in.rs`'s `Past` and `PAST_LINKS`; `mint.rs`'s `sign_in_reply`) and the OpenID provider's codes (`provider.rs`: `Whose`, `Waiting::Authorize`, its codes' map by digest and their `until`).

## Owns and may change

- Owned: `jig/crates/jig-protocol-engine-clients/src/**`: a new `approval.rs` (its state, its constants `WAITING`, `SPAN`, `PAST`, its handlers), and the arms and hooks it needs in `component.rs`, `sign_in.rs` (new `Waiting` and `Answer` variants), `mint.rs`, `services.rs` (the kept digest releasing an approval's answer), `routes.rs` (the encoders `surface` wrote), `closing.rs`, `limits.rs`, and the crate's tests under `src/tests/` (a new `approval.rs` and its `mod` line; helpers in `tests/fixtures.rs`).
- Readers: none outside the crate; `surface` made the boundary. If a pattern elsewhere breaks on a new `Waiting` or `Answer` variant (they are `pub(crate)`), it is inside the crate.
- Options:
  - The live approvals: a small fixed-capacity list or a `Map` by the secret's digest, the code unique among them; your choice, bounded by `WAITING`. The past: a `Queue` of `PAST` entries, each the secret's digest, the code and taken, refused or expired, the oldest dropped, as `past_links`.
  - Drawing the code: eight letters, each a byte from the component's `CryptoRng` taken uniformly into the alphabet by rejection (a byte below 240, modulo 20), then `ApprovalCode::from_indexes`; draw again while a live approval holds it.
  - Where the deciding exchange's credential is checked: through `Waiting::Get`/`Waiting::Checked` as every authenticated route's is, with `browser` checked first, in `body`, before anything is read (the `surface` task keeps `browser` on the admission; if it didn't, carry it into `Waiting::Get` and say so).
  - The take's wait for people: new `Waiting` variants for `Whose` and for the core's sign-in, reusing `sign_in_reply`'s token draw and `Waiting::Keep` with a new `Answer::Approval(w::SignedIn)` (or `Answer::SignedIn` with a mark) whose kept digest releases an `ApprovalSignInAnswer::SignedIn` record; your choice, named in the report.
  - If the engine's close or abort meets an approval's take in flight, it ends as the link's reservation does (`closing.rs`), the approval returned or dropped with the component.

## Steps

1. **Tests first, red, each refusal before its code** (`src/tests/approval.rs`, on the `Exchange` harness of `tests/fixtures.rs`, with `Settings { approval: true, ..f::settings() }`; add a harness helper to send a message with a given `browser` and credential, and to answer `Whose` and `SignIn` on the core's queue). Commit them red; the report says what each failed on. One table per behaviour, a row per case:
   - `an_ask_is_answered_by_the_room_and_the_engine`: asked (the secret 32 bytes, the code parses, `expires` the wall plus 10 minutes, nothing on `out`); refused where approval is off; not ready while stopping; busy at the fifth live, and asked again once one has ended.
   - `only_a_browser_reads_or_decides`: `browser` false, with a bearer credential, is `not_a_browser` with nothing on `out`; no credential, and a credential whose digest the secrets answer refuses, are `signed_out`; for each of read and decide.
   - `a_read_says_what_the_approval_is`: waiting with its `asked` wall time; approved; taken; refused; expired (past its span, and after `fire`); unknown (a code never drawn; the ninth ended, past `PAST`).
   - `the_first_decision_spends_the_approval`: approve, then approve again (`used`), refuse after approve (`used`), refuse then approve (`used`); a decision past its span (`expired`); a code never drawn (`unknown`).
   - `a_take_says_why_it_signs_no_one_in`: waiting; refused; expired; used (after a take); unknown (32 other bytes; 13 bytes; empty); the same secret again while its take waits on people (`busy`); while stopping (`not_ready`); the approver's sign-in ended (`Whose` answered `Signer(None)`): `refused`, and the approval spent (a second take `refused`).
   - `busy_not_ready_and_full_spend_nothing`: the core's `Again(Busy)`, `Again(NotReady)` and `Full { .. }` to the take's `SignIn`: busy, not ready, full with `until` the approval's expiry; then the same take signs in.
   - `an_approved_take_signs_in_the_approvers_party_once` (a story): ask; read and approve from a browser exchange signed in as sign-in `f::NUMBER`; take: `Whose { sign_in: f::NUMBER }` on the core's queue, answered with a person's `Signer`; then `SignIn { identity, held: None }` with that `Signer`'s identity exactly; answered `SignedIn`; the digest `Keep` before any answer; then `signed_in`, its token 40 bytes, its party the person's. A second take is `used`; a read is `taken`.
   - `an_approval_never_taken_makes_no_sign_in`: approved, the clock past `SPAN`, `fire`: no core event at any point; the take is `expired`; `next_deadline` named that moment before.
   - `no_print_of_the_component_shows_a_secret`: with a canary seed, after an ask, an approve and a take in flight and after its answer: the component's `{:?}`, `out`'s and each core event's hold neither the secret's bytes nor the credential's, as text or as a byte list (as `jig/tests/engine-clients/tests/suite/sign_in.rs`, `no_core_event_holds_a_token_or_digest`).
   - `a_restart_forgets_every_approval`: a fresh component answers a live approval's take and read `unknown`.
2. **Then `approval.rs`** and its hooks, until each row passes. Keep the secret only as its digest; compare digests in constant time (`sign_in::constant_time_eq`).
3. Break each rule once by hand (let a bearer decide; take without `Whose`; spend on busy; keep the secret whole and print it) and see its row fail, then restore.
4. `cargo fmt`; `python3 -I -B jig/scripts/old_forms.py`; `heavy cargo clippy -p jig-protocol-engine-clients --all-targets -- -D warnings`; `heavy cargo nextest run -p jig-protocol-engine-clients`; its direct dependents' focused tests: `-p jig-protocol-engine-clients-http`, `-p jig-engine-clients-world`, `-p jig-protocol-config`, `-p jig-client-kit`, `-p temper-service`, `-p temper-client-world` (`heavy cargo nextest run -p ...`, one at a time).
5. Commit and push.

## Report

As the implementer's text says, and: the choices among the options; each red test and what it failed on; each rule broken by hand and the failure seen; the new tests' time.
