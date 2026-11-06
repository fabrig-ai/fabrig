+++
id = "debug-rows"
part = "debug-rows"
kind = "feature"
needs = []
files = ["skein/tests/oauth-accounts/tests/suite/debug.rs"]
effort = "high"
minutes = 40
+++

# The credential Debug rows go red when a raw-bytes field is printed

## Goal

`debug_hides_oauth_secret_and_hand_written_views`'s `accounts::Component` and `fake::Issuer` rows sit behind `Secret` alone, and the raw-bytes fields they hold are empty, so printing those fields stays green. Put the canary in a raw-bytes field of each, so that the realistic break goes red.

Acceptance:

- The `fake::Issuer` row is an issuer holding a delayed answer: `Pending.body`, a `Box<[u8]>`, holds the canary.
- The `accounts::Component` row is a component with an exchange whose `Web.document`, a `List<u8>`, holds the canary, as the issuer's answer arrives.
- With each hand break below in turn, the test fails with `canary bytes in Debug` on that row. Restored, it passes. The report says what you broke and what failed.
- No production file changes in your commit. The test stays one test over the one table, and grows about 35 lines at most: helpers, not copies.

## Inputs

- `skein/docs/design/oauth.md`, 4, and `skein/docs/design/lib.md`, 6.
- `temper/docs/plan/debt-26/inputs/debt24-review.md`, second half, finding 1.
- `temper/docs/plan/debt-26/notes/common.md`.

## Owns and may change

Owns `debug.rs`. The hand breaks are throwaway: they never reach your commit.

**The review's other suggestion doesn't bite, and the cut checked it:** its `Locked` path fills `lock_candidate`, a `SavedToken` of `Secret`s and a `missing` of scope names, so it adds no raw credential bytes. `Web.upload` holds the request body, but `skein_link::Upload`'s own `Debug` prints only its length. `Web.document` is the field that holds an answer's raw bytes.

Options, each pre-approved:

- Reach an open exchange as `tests/suite/sign_out.rs`'s `ExchangeOwner::refreshing` does: a `SignIn` account with a kept record, plaintext endpoint, `Request::Grant`, `fire`, `Connecting`, `Connected`, `fire`. A helper from the world's library is fine if one fits. `exchange.rs`'s own test around `web.document.len()` shows how a body arrives in pieces.
- If `Web.document` can't be reached in the 35 lines through the component's public boundary, ask with what you measured: the request says these rows must go red, so don't weaken the row.

## Steps

**Issuer.** `post()`'s body is the bare canary, which the issuer refuses before it takes a plan. So queue the plan and post a form the issuer accepts, using the fixture's own config:

```rust
issuer.queue(fake::Plan { status: 200, body: fake::Body::Raw(secret()), delay: Duration::from_secs(1), retry_after: Duration::ZERO })?;
issuer.step(fake::Event::Post { request: /* grant_type=refresh_token&refresh_token=<canary>&client_id=client&client_secret=<canary> */, now: Time::ZERO, wall: Wall::EPOCH }, &mut out);
```

Read `post` and `accepted` in `skein-fake-oauth/src/lib.rs` for the fields; assert that nothing came out, so that the row can't quietly lose its pending answer, and that `posts()` is 1.

**Component.** The answer's head says a `content-length` longer than what you deliver, so that the document stays partial and holds the canary: `{"access_token":"<canary>` is enough. Assert that the exchange hasn't finished, by the absence of `Granted` or `Failed` events.

**The hand breaks,** one at a time. The realistic break is a `Debug` that starts printing what it holds:

- `skein-fake-oauth/src/lib.rs`, `impl Debug for Issuer`: add `.field("body", ...)` for `self.pending`'s `body`.
- `skein-oauth-accounts/src/component.rs`, `impl Debug for Component`: print each live exchange's `web`'s `document`, through a temporary `pub(crate)` accessor in `exchange.rs`.

Narrow test: `heavy cargo nextest run -p skein-oauth-accounts-world --test suite -E 'test(=debug::debug_hides_oauth_secret_and_hand_written_views)'`. Check the name with `heavy cargo nextest list -p skein-oauth-accounts-world --test suite` if your filter selects nothing. Then the package's tests, fmt, and `heavy cargo clippy -p skein-oauth-accounts-world --all-targets -- -D warnings`. Commit, push.

## Report

As your text says: both hand breaks with the failure each printed, secret left out, and the lines the test grew by.
