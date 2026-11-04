+++
id = "oauth-start-refused"
part = "oauth-client"
kind = "feature"
needs = []
files = ["skein/crates/skein-oauth/src/machine.rs", "skein/tests/oauth/tests/suite/world.rs", "skein/docs/design/oauth.status.md"]
effort = "medium"
minutes = 30
+++

# A start outside Idle is refused, never dropped

## Goal

`skein-oauth`'s `Client::step` returns early when `SignIn`, `Refresh` or `Revoke` arrives outside
`Phase::Idle`, with no terminal, so a direct caller waits forever. (Bug 2.)
`skein/docs/design/oauth.md`, 3 now says what happens.

Acceptance:

- A `SignIn` or `Refresh` outside Idle emits `Request::Failed { failure: Failure::Limit }`, a
  `Revoke` outside Idle emits `Request::Revoked { confirmed: false }`, in that step's output.
- `phase` and `active` do not change: a running exchange reaches its own terminal afterwards, and
  Done still needs `Event::Reset`.
- A test shows both, and fails on the old code, which answers nothing.

## Inputs

- `skein/docs/plan/bugs-skein/notes/common.md`.
- `skein/docs/design/oauth.md`, 3 (the paragraph "A start in any state but Idle"), 4 (`Limit`,
  a local admission failure).

## Owns and may change

- Owns: `skein/crates/skein-oauth/src/machine.rs` (`Client::step`'s three start arms);
  `skein/tests/oauth/tests/suite/world.rs` (the new test, or a new module beside it if
  `suite/main.rs` lists them: then that one `mod` line too).
- `skein/docs/design/oauth.status.md`: remove this plan's **Not yet** entry, keeping its
  **Departs** (`notes/common.md`).
- No reader changes. Surveyed: every caller starts a fresh client exactly once:
  `skein-oauth-accounts` (`component/sign_in.rs`, `component/lock.rs`, `component/sign_out.rs`) and
  `temper-service`'s `clients/google.rs`, so none reaches the new output. If a test shows
  otherwise, stop and ask.

## Steps

1. In `skein/tests/oauth/tests/suite/world.rs`, using `skein_oauth_world`'s `client_step`, `begin`
   and `planned`, write `a_start_outside_idle_is_refused_at_once`:

   - with a sign-in waiting for its redirect, a second `SignIn` and a `Refresh` each give
     `Some(Request::Failed { failure: Failure::Limit })`, a `Revoke` gives
     `Some(Request::Revoked { confirmed: false })`; `oauth::Request` has no `Debug`, so `match`;
   - the same sign-in then takes its redirect and ends in `Tokens`, its own terminal;
   - in Done, a `SignIn` is refused the same way, and after `Event::Reset` it is taken.

   Run it on the old code: `client_step` gives `None`.
2. In `Client::step`, push the refusal in each arm's non-Idle path; the rest of the arm is as it
   is. Each arm stays explicit.
3. `heavy cargo nextest run -p skein-oauth-world --test suite -E 'test(a_start_outside_idle)'`,
   then the focused tests of `skein-oauth`, `skein-oauth-world`, `skein-oauth-accounts`,
   `skein-oauth-accounts-world`, `skein-oauth-issuer-world`. `heavy cargo check -p temper-service
   -p smith-local-service --tests` confirms the callers beyond skein. `cargo fmt`,
   `python3 -I -B skein/scripts/old_forms.py`, `heavy cargo clippy -p skein-oauth -p
   skein-oauth-world --all-targets -- -D warnings`.
4. Commit, push.

## Report

As the implementer's text says, with what the test showed on the old code.
