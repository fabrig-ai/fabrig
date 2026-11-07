+++
id = "long-redirect"
part = "long"
kind = "feature"
needs = ["redirect-bound"]
files = ["skein/crates/skein-oauth/src/machine.rs", "skein/crates/skein-oauth/src/machine_tests.rs", "skein/crates/skein-oauth-accounts/src/**", "skein/tests/oauth-accounts/**", "skein/docs/design/oauth.status.md"]
effort = "high"
minutes = 150
+++

# ui-long-redirect: an over-long redirect says what it is and ends its sign-in

## Goal

A sign-in's own redirect past the driver's redirect bound gets, at the listener, the
page "This sign-in's answer is longer than temper accepts. Go back to temper and start
the sign-in again." (temper being the name its owner gives the component), and the
sign-in ends failed at once, naming the address's length and the bound, rather than
waiting for its deadline. A redirect that doesn't match keeps "This is not the
registered redirect.", and the sign-in still waits; so does an over-long one of
another state.

Acceptance:

- `Failure::LongRedirect { length, bound }`, `APPLICATION_BYTES`,
  `Component::new(accounts, limits, crypto, application)` and `skein_oauth`'s
  `Client::awaits_state` exist as `contracts.md`, section 3 gives them, each citing
  `skein/docs/design/oauth.md` (6.5, 6.7; 3 and 6.5 for `awaits_state`) at item level.
- The listener's pages, the over-long reading and the judgement are as `contracts.md`,
  section 3 says: the long page and `414` only for the sign-in's own state, the
  terminal in the same step, the listener closing once the page is sent; everything
  else past the bound the `REFUSAL` page, the sign-in still waiting. A paste past the
  bound stays `Refused(Redirect)`.
- An application name off its bound (empty, past 64 bytes, a byte outside visible ASCII
  and space) makes `Component::new` answer `Err(Unusable::Limits)`.
- The readers outside skein compile with only the edits `contracts.md`, section 3 lists,
  and their rows below pass.
- Rows below fail on the base for the rule's reason (quoted in the report) and pass;
  every other test of `skein-oauth`, `skein-oauth-accounts`,
  `skein-oauth-accounts-world`, `jig-protocol-accounts`, `smith-local-shell` and
  `smith-local-service` passes as before.
- Delete from `skein/docs/design/oauth.status.md` the bullet "**Not yet:** 6.3, 6.5,
  6.7, 6.8: an over-long redirect: ..." and nothing else (a stated departure from the
  implementer's text, which closes `docs/design/`).

## Inputs

- `temper/docs/plan/ui-long-redirect/notes/common.md`: the standing rules.
- `temper/docs/plan/ui-long-redirect/notes/contracts.md`, sections 3, 4 and 5: the
  shapes, the page's bytes, the readers' edits and the fixtures, binding.
- `skein/docs/design/oauth.md`, 6.3 (the `failed` row), 6.5 (the listener's first
  bullet; *An over-long redirect*), 6.7 (*The application's name*), 6.8 (*Focused
  rows*); section 3 for where `awaits_state` sits beside `admits`.
- `skein/docs/foundation/programming-model.md`, 10: the strict subset.

## Owns and may change

- Owns: `skein-oauth`'s `machine.rs` (`awaits_state` alone) and `machine_tests.rs` (its
  rows); `skein-oauth-accounts/src/**` (`boundary.rs`, `lib.rs`, `component.rs` and
  `component/*`, `listener.rs`, `redirect.rs`, `exchange.rs` where the page reaches the
  listener, and every test file's `Component::new` call); `skein/tests/oauth-accounts/**`
  (its `Component::new` calls, each passing a name of the world's own).
- May change, each only what `contracts.md`, section 3's *Readers outside skein* lists,
  as the producer's branch carrying another tree's lines (a stated departure: a new
  argument and a new variant force them, as a required field does):
  `jig/crates/jig-protocol-accounts/src/translate.rs` (the arm),
  `jig/crates/jig-protocol-accounts/src/moves.rs` (the accessor),
  `jig/crates/jig-protocol-accounts/src/component.rs` (`opened`'s call),
  `jig/crates/jig-protocol-accounts/src/tests/translate.rs` (one row),
  `smith/crates/smith-local-service/src/credentials.rs` (the call),
  `smith/crates/smith-local-shell/src/credentials/login/provider/driver.rs` (the arm),
  `smith/crates/smith-local-shell/src/credentials/login/provider/driver/tests.rs` (one
  row). Another call of `skein_oauth_accounts::Component::new` the compiler finds
  outside skein passes the application name its crate already holds, or is a
  `Question:` where it holds none.
- Not yours: `jig-protocol-config` (task `driver-profile`), the client wire, jig's
  domain `Failure`, temper.
- Options, pre-approved:
  - How the long page reaches the callback (a page the component holds and lends, an
    enum of the three pages, a `Box<[u8]>` on the callback) is yours; `PAGE` and
    `REFUSAL` keep their bytes.
  - How the over-long read is handed up (a field beside `read`, a variant of an enum
    `from_head` returns) is yours; `parse`, the paste's path, keeps refusing past the
    bound.
  - If the listener cannot send its page once the sign-in's terminal closes it, keep
    the exchange's listener settling until the page is sent, as a taken redirect's does;
    a design that needs the terminal later than the judging step is a `Question:`.

## Steps

1. **The failing rows first**:
   - `skein-oauth`'s `machine_tests.rs`, beside the redirect-admission rows: a table of
     `awaits_state`: the sign-in's state while it waits (true); another state, the
     deadline reached, before the visit and after the redirect is taken (false each).
   - `paste_tests.rs`, with `contracts.md`, section 5's fixtures and `answer_profile()`:
     a table test `an_over_long_listener_redirect_ends_only_its_own_sign_in`, each row
     from the browser's request one byte past `redirect_bytes`:
     - the sign-in's state on the bound path: `HTTP/1.1 414`, the body the long page
       naming the harness's application, and `Failed { account: 0, ends: Ends::SignIn,
       failure: Failure::LongRedirect { length: redirect_bytes + 1, bound:
       redirect_bytes } }` without the clock reaching the deadline; the listener's
       socket closes after the page;
     - another state: `HTTP/1.1 404`, `This is not the registered redirect.`, no
       terminal; a fitting redirect then signs in;
     - another path, the sign-in's state: the same as another state;
     - the state twice: the same.

     The `redirect-bound` row one byte past the bound becomes the first row here.
     Keep a row of the existing listener tests, a fitting redirect of another path, and
     assert its body is `This is not the registered redirect.`.
   - `tests.rs`' validity table: an empty name, 65 bytes, a byte `0x7f` →
     `Err(Unusable::Limits)`; 64 bytes of letters and a space → made.
   - jig's `other_needs_sign_in_failures_do_not_request_record_erasure`: the cause
     `driver::Failure::LongRedirect { length: 16_385, bound: 16_384 }`; smith's
     `other_credential_failures_keep_their_words`: the same failure, "account work: not
     signed in".

   Run the skein rows on the base: the long row gets the `404` refusal and no terminal.
   Quote it. (The new names don't compile on the base; write those rows' bodies against
   today's API where you can, and say which you could not run first.)
2. Add `awaits_state`; `Failure::LongRedirect`; the application name through
   `Component::new` and the long page; the over-long read in `redirect.rs`, handed up by
   the listener; the judgement in `component.rs`.
3. Fix every call and arm (section 3's readers, then skein's tests and world).
4. Make each row bite: compare the state as accepted always (the other-state row fails);
   drop the terminal (the first row fails); restore.
5. Delete the status bullet. `cargo fmt`; `python3 -I -B skein/scripts/old_forms.py`,
   and `jig/` and `smith/`'s; `heavy cargo clippy -p skein-oauth -p skein-oauth-accounts
   -p skein-oauth-accounts-world -p jig-protocol-accounts -p smith-local-shell
   -p smith-local-service --all-targets -- -D warnings`; `heavy cargo nextest run
   -p skein-oauth -p skein-oauth-accounts -p skein-oauth-accounts-world
   -p jig-protocol-accounts -p smith-local-shell -p smith-local-service`. Commit, push.

## Report

The implementer's report, with: the failure seen on the base; how the page reaches the
callback and how the long read is handed up; how the listener settles after its page;
every edit outside skein, by file and line.
