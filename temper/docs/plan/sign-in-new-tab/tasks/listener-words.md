+++
id = "listener-words"
part = "listener-words"
kind = "feature"
needs = []
files = ["skein/crates/skein-oauth-accounts/src/listener.rs", "skein/tests/oauth-accounts/src/sign_in/browser.rs"]
effort = "medium"
minutes = 15
+++

## Goal

The public client's loopback listener answers a redirect it takes, with no return given, with the fixed page "Sign-in received. You can close this tab.", because a new tab now ends there and the listener answers before the exchange ends.

Acceptance:

- `PAGE` in `skein-oauth-accounts`'s `listener.rs` is `b"Sign-in received. You can close this tab.\n"`.
- A test in `skein-oauth-accounts-world` holds the whole body, and fails on the old words before the change.
- No file in the repository's code or tests holds the old words. `temper/docs/design/client/README.md`, 3.5 still has them and is not yours; a later task's design replaces them.

## Inputs

- `skein/docs/design/oauth.md`, 6.5.
- `temper/docs/design/draft/sign-in-new-tab.md`, 4.
- `temper/docs/plan/sign-in-new-tab/notes/common.md`.

## Owns and may change

- Owns `skein/crates/skein-oauth-accounts/src/listener.rs` and `skein/tests/oauth-accounts/src/sign_in/browser.rs`.
- May change, each only in the literal's value:
  - `smith/tests/local-process/src/oauth.rs`, line 113, the listener's words among its two.
  - `temper/tests/real-loop/tests/suite/sign_in.rs`, line 41.
- Remove this draft's **Not yet** line for 6.5 from `skein/docs/design/oauth.status.md`. This task may make that one edit, against the implementer's text, because the status follows the code.
- `skein/crates/skein-oauth-accounts/src/paste_tests.rs` finds "Sign-in received." alone and needs no edit.

## Steps

1. Change the assertion in `skein/tests/oauth-accounts/src/sign_in/browser.rs` to the new words. Run `heavy cargo nextest run -p skein-oauth-accounts-world --test suite -E 'test(sign_in)'` and see it fail on the old words. Read the count: the run must select tests. If the test binary is named otherwise, take its name from the package's `Cargo.toml`.
2. Change `PAGE`. See it pass.
3. Edit the two literals in smith and temper.
4. `cargo fmt`; `heavy cargo clippy -p skein-oauth-accounts -p skein-oauth-accounts-world --all-targets -- -D warnings`; `heavy cargo nextest run -p skein-oauth-accounts`; `heavy cargo check -p smith-local-process-world -p temper-real-loop-world --tests`.
5. `heavy cargo nextest run -p temper-real-loop-world --test suite -E 'test(a_terminal_sign_in_returns_to_the_real_listener)'` and `heavy cargo nextest run -p smith-local-process-world --test suite -E 'test(a_sign_in_uses_the_page_and_redirect_before_lending_the_saved_token)'`. Read each count. If a binary is named otherwise, take its name from the package's `Cargo.toml`.
6. Commit and push.

## Report

As the implementer's text asks, and say which smith and temper tests ran.

## Answers

2026-11-06. **May `smith/tests/local-process/src/oauth.rs` line 138's accepted length change from 43 to 42, beyond the literal's value?** Yes. The native page's length is the literal's length: the new words are 42 bytes where the old were 43, so the assertion follows from the words the request fixes, and step 5's smith test cannot pass without it. Change `length == 43` to `length == 42` on that line, and nothing else there; the 18 stays. Then run step 5's smith test, and the rest of steps 4 and 5 that have not run on the final tree.
