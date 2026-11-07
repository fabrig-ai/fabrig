+++
id = "http"
part = "jig-approval"
kind = "feature"
needs = ["surface"]
files = ["jig/crates/jig-protocol-engine-clients-http/**"]
effort = "high"
minutes = 75
+++

## Goal

jig's default HTTP mapping, engine end, carries the approval: its four routes to the component's four messages, each exchange told to the component as a browser's when, and only when, its credential came by the cookie, and an approval's take answered to a browser as a link's sign-in is, the cookie set and no credential in the body (`jig/docs/design/protocol/client.md`, 17.1 and 17.3; `temper/docs/plan/tui-approval/notes/contracts.md`, 3.1 and 5).

Acceptance:

- `POST <prefix>/v1/approval/ask`, `/v1/approval/read`, `/v1/approval/decide` and `/v1/sign-in/approval` reach the component as `Message::ApprovalAsk`, `ApprovalRead`, `ApprovalDecide` and `SignInApproval`, each with its body; any other method on them is `404`, as for every POST route.
- `FromTransport::Exchange.browser` is true for a credential read from the cookie (the browser header present, else `403` as today), false for a bearer, with the header or without, and false with no credential; on every route, the existing ones included.
- A browser's `signed_in` on `/v1/sign-in/approval` sets the cookie, as `cookie()` makes it for a link, and the body's `SignedIn` carries no token; a native one keeps the token in the body; busy, full and the approval's other answers set no cookie.
- The guards hold for the new routes as for every route (rows of the existing tables).

## Inputs

- `jig/docs/design/protocol/client.md`, 17.1, 17.3 (*The credential's carrier, told to the component*) and section 8, *A sign-in approved by a browser* (for the take's answer).
- `jig/docs/design/protocol/browser.md`, 12.2 and section 19's table, its row *an approval read and decided under the cookie alone*.
- `temper/docs/plan/tui-approval/notes/contracts.md`, sections 2.1, 3.1 and 5 (binding).
- `temper/docs/plan/tui-approval/notes/common.md`.

## Owns and may change

- Owned: `jig/crates/jig-protocol-engine-clients-http/**`: `dispatch.rs` (`Route`'s four variants, `route`, `body_bound`, `message`, and `reply` decoding `w::ApprovalSignInAnswer` for the take's route), `lib.rs` (where the credential is read: keep its source, cookie or bearer, on the call), `call.rs` (`browser` set from that source where `surface` left `false`), `credential.rs` if its `Read` needs no change but a helper, and `src/tests.rs`, `src/tests/fixtures.rs`.
- Readers: none; the component's boundary is `surface`'s.
- Options:
  - `call.browser` (the header) stays what decides the cookie's answer; a new field (say `cookie: bool`, the credential's source) is what `browser` on the exchange takes. If one field can carry both without changing an existing answer, use one, and say so.
  - If `reply`'s decoding is easier as a second function for the take's answer than as a branch, write it so; the cookie's attributes stay `cookie()`'s.

## Steps

1. **Tests first, red** (rows of the existing tables in `src/tests.rs` where the behaviour is theirs):
   - `SIGN_IN_PATHS` gains `/jig/v1/sign-in/approval`, so `a_browser_s_link_sign_in_sets_the_cookie_and_answers_no_token` and `a_native_sign_in_answers_its_token_in_the_body` cover the take; where those tests decode a `SignInAnswer`, add the path's own fixture reply (`ApprovalSignInAnswer::SignedIn`) beside `signed_in_reply`.
   - `busy_and_full_answer_200_without_setting_a_cookie` gains the take's busy and full rows (`ApprovalSignInAnswer::Busy`, `Full`).
   - A new table, `the_approval_routes_reach_the_component_as_their_messages`, rows `(&[u8], fn(Box<[u8]>) -> kit::Message)`: the four paths, a POST of a short body each, the `FromTransport::Exchange` it gives.
   - A new table, `an_exchange_is_a_browser_s_only_by_its_cookie`, rows `(&str, Fields, Option<bool>)`: the cookie with the header (`Some(true)`); a bearer with the header (`Some(false)`); a bearer alone (`Some(false)`); a bearer and the cookie with the header, the bearer read first (`Some(false)`); no credential with the header (`Some(false)`); the cookie without the header (`None`: `403`, nothing reaches the component); each on `/jig/v1/approval/decide` and on `/jig/v1/read`.
   - `each_guard_refuses_with_nothing_reaching_the_component` gains a row for `/jig/v1/approval/decide` per guard it holds rows for (Host off the list, content type, OPTIONS).

   Commit them red; the report says what each failed on.
2. The routes, the source of the credential carried to `browser`, and the take's cookie, until each row passes.
3. Break each rule once by hand (mark a bearer a browser's; drop the take's path from `reply`) and see its row fail, then restore.
4. `cargo fmt`; `python3 -I -B jig/scripts/old_forms.py`; `heavy cargo clippy -p jig-protocol-engine-clients-http --all-targets -- -D warnings`; `heavy cargo nextest run -p jig-protocol-engine-clients-http`, then its dependents' focused tests one at a time: `-p jig-engine-clients-world`, `-p jig-test-service`, `-p temper-service`; `heavy cargo check -p jig-engine-clients-kit -p jig-client-protocol-kit --tests`.
5. Commit and push.

## Report

As the implementer's text says, and: how the credential's source is kept on the call; each red row and what it failed on; each rule broken by hand and the failure seen.
