+++
id = "redirect-bound"
part = "bound"
kind = "feature"
needs = []
files = ["skein/crates/skein-oauth-accounts/src/limits.rs", "skein/crates/skein-oauth-accounts/src/lib.rs", "skein/crates/skein-oauth-accounts/src/redirect.rs", "skein/crates/skein-oauth-accounts/src/listener.rs", "skein/crates/skein-oauth-accounts/src/component/sign_in.rs", "skein/crates/skein-oauth-accounts/src/paste_tests.rs", "skein/crates/skein-oauth-accounts/src/tests.rs", "skein/crates/skein-oauth-accounts/src/openid_tests.rs", "skein/crates/skein-oauth-accounts/src/ports_tests.rs", "skein/tests/oauth-accounts/**", "skein/docs/design/oauth.status.md"]
effort = "high"
minutes = 120
+++

# ui-long-redirect: the returning redirect has a bound of its own

## Goal

A ChatGPT sign-in's redirect whose code is 1,900 or 3,900 bytes is taken, from the
loopback listener and pasted, where today both refuse it: `redirect.rs` bounds the
returning redirect by the client's `url_bytes`, which the small profile holds at
`ADDRESS`, 2,048, the bound of the address the client sends out. The driver gets its own
bound, `redirect_bytes`, and the crate says how much a full answer and a browser's
request for it take (`redirect_room`, `listener_head`, `BROWSER_FIELDS`), for jig's
profile to size and check (a later task).

Acceptance:

- `skein_oauth_accounts::Limits.redirect_bytes`, `BROWSER_FIELDS`, `redirect_room` and
  `listener_head` exist as `contracts.md`, section 1 gives them, exported from `lib.rs`,
  citing `skein/docs/design/oauth.md`, 6.7 at item level. Both functions are `const fn`.
- `redirect::parse` and `redirect::from_head` bound the address by `redirect_bytes`; no
  code in the crate reads the client's `url_bytes` for a returning redirect. The paste
  (`component/sign_in.rs`) and the listener (`listener.rs`) pass the same bound.
- Every `skein_oauth_accounts::Limits` literal compiles with the value
  `contracts.md`, section 1's table gives it.
- The rows below fail on the base for the rule's reason (quoted in the report) and pass
  on the branch; every other test of `skein-oauth-accounts` and
  `skein-oauth-accounts-world` passes as before.
- `redirect_room` and `listener_head` are pinned by a row each: under `answer_profile()`,
  13,407 and 24,593; and `None` for a sum past `u32`.
- Delete from `skein/docs/design/oauth.status.md` the bullet "**Not yet:** 6.5, 6.7, 6.8:
  the driver's own redirect bound: ..." and nothing else (a stated departure from the
  implementer's text, which closes `docs/design/`).

## Inputs

- `temper/docs/plan/ui-long-redirect/notes/common.md`: the standing rules.
- `temper/docs/plan/ui-long-redirect/notes/contracts.md`, sections 1, 4 and 5: the shapes,
  values and synthetic fixtures, binding.
- `skein/docs/design/oauth.md`, 6.5 (*A redirect is taken only when it is the sign-in's*;
  *A pasted redirect*), 6.7 (the table's `redirect bytes` row; *The redirect bound holds a
  full answer*; *The listener's head holds a browser's request*), 6.8 (*Focused rows*).
- `skein/docs/foundation/programming-model.md`, 10: the strict subset the crate keeps.

## Owns and may change

- Owns: `skein-oauth-accounts`' `limits.rs`, `lib.rs`, `redirect.rs`, the bound's
  passing in `listener.rs` and `component/sign_in.rs`; its tests `paste_tests.rs`,
  `tests.rs` (`limits()` gains the field), and the call sites of `parse`/`from_head` in
  `openid_tests.rs` and `ports_tests.rs`; `skein/tests/oauth-accounts/**` (its
  `Limits` literals, `src/sign_in/paste.rs`' `Wrong::Long`, which now goes one byte past
  `redirect_bytes`, and `tests/suite/sign_in.rs`' comment `// Past redirect_bytes.`).
- May change, each only the new field's line (`redirect_bytes: <value>`, with the
  comment `contracts.md` gives for small), as the producer's branch carrying another
  tree's literal (the lab's rule for a required field):
  `jig/crates/jig-protocol-config/src/profile/small.rs` (`DRIVER`),
  `jig/crates/jig-protocol-accounts/src/tests/fixtures.rs`,
  `jig/testing/jig-protocol-accounts-kit/src/fixtures.rs`,
  `smith/crates/smith-local-shell/src/credentials/product.rs`,
  `smith/crates/smith-local-service/src/credentials/tests.rs`. Any other literal the
  compiler finds gets its client's `url_bytes`, named in the report.
- Not yours: `small.rs`' `server.head` and every check in `jig-protocol-config`
  (task `driver-profile`); the listener's pages, `Failure` and `Component::new`
  (task `long-redirect`).
- Options, pre-approved:
  - `parse` and `from_head` take the bound as a `u32` beside `&ClientLimits`, or take
    `&Limits`: yours; say which.
  - If `answer_profile()` needs more than the listed bounds raised for the exchange to
    carry a 3,900-byte code, raise them in that fixture alone and list them.
  - If a memory ceiling in `skein-oauth-accounts-world` moves because a fixture's
    `redirect_bytes` reads differently, it should not: each takes its `url_bytes`'
    value. A ceiling that moves anyway is a `Question:` with the measurement.

## Steps

1. **The failing rows first,** in `paste_tests.rs`, beside the redirect rows, with
   `contracts.md`, section 5's `synthetic` and `answer_profile()`:
   - a table test `a_long_issuer_answer_is_taken_from_the_listener_and_pasted`, rows
     `(name, code length, path)` for 1,900 and 3,900 bytes, each through the listener
     (the browser's request of section 5, its target the answer's path and query; the
     listener answers `HTTP/1.1 200` with `Sign-in received.`) and pasted (`Taken`);
     each then signs in, and where the harness reads the token request
     (`Owner::sent`), its body carries the code whole;
   - a row at the bound: the listener's address exactly `redirect_bytes` long (the
     answer padded by an unread field, `&pad=<synthetic>`), from the browser's request,
     taken under `server.head` = `listener_head(redirect_bytes)`; and one byte past it,
     refused with the refusal page (`HTTP/1.1 404`), the sign-in still waiting.

   Write them first against today's `Limits`: `answer_profile()` without
   `redirect_bytes`, its `server.head` the literal 24,593, the at-bound rows left for
   step 3. Run them (`heavy cargo nextest run -p skein-oauth-accounts --lib -E
   'test(=paste_tests::...)'`): each long row is refused today (the listener's `404`,
   the paste's `Refused(Redirect)`). Quote one of each in the report.
2. Add the field, the constant and the two functions (section 1), and their pinning rows
   in `tests.rs` beside the limit rows; `answer_profile()` takes `redirect_bytes` 16,384
   and `server.head` `listener_head(16_384)`.
3. Bound `parse` and `from_head` by `redirect_bytes`; pass it from both callers. Add the
   at-bound rows.
4. Fill every literal (section 1's table); rename the world's `Past url_bytes.` story
   comment and make `Wrong::Long` go past `redirect_bytes`.
5. Make each new row bite: bound `from_head` by `url_bytes` again, see the listener rows
   fail; then `parse`, see the paste rows fail; restore.
6. Delete the status bullet. `cargo fmt`; `python3 -I -B skein/scripts/old_forms.py`;
   `heavy cargo clippy -p skein-oauth-accounts -p skein-oauth-accounts-world
   -p jig-protocol-config -p jig-protocol-accounts -p jig-protocol-accounts-kit
   -p smith-local-shell -p smith-local-service --all-targets -- -D warnings`;
   `heavy cargo nextest run -p skein-oauth-accounts -p skein-oauth-accounts-world`;
   `heavy cargo check --tests` for each jig and smith package above. Commit, push.

## Report

The implementer's report, with: the failures seen on the base; the bound's signature
choice; the literals filled and their values; anything `answer_profile()` raised.
