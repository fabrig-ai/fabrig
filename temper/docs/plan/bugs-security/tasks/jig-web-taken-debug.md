+++
id = "jig-web-taken-debug"
part = "jig-holders"
kind = "feature"
needs = []
files = [
  "jig/crates/jig-client-shell-web/src/sign_in.rs",
  "jig/crates/jig-client-shell-web/src/tests.rs",
]
effort = "medium"
minutes = 25
+++

# jig-client-shell-web's taken sign-in secret prints as its length

## Goal

`jig/crates/jig-client-shell-web/src/sign_in.rs`'s `Taken` (~9) derives `Debug, PartialEq, Eq`,
and its `secret: Box<[u8]>` is the sign-in link's secret, decoded from `?secret=`. Give it a
hand-written `Debug` that prints the secret as its length; `PartialEq` and `Eq` stay derived.

Acceptance:

- `Taken` prints `Taken { secret_len: N, address: .. }`; no byte of the secret.
- The new test fails on the old code and passes on the new;
  `the_link_route_s_secret_is_taken_and_the_address_is_the_home` passes unchanged.
- clippy clean (for the host; the crate's wasm32 code is unchanged); the crate's lib tests pass.

## Inputs

- `temper/docs/plan/bugs-security/notes/common.md` and `notes/debug.md`.
- `jig/AGENTS.md`.
- `temper/docs/plan/bugs-security/inputs/request.md`, item 4.
- `jig/docs/design/protocol/browser.md`, 11 (the printed link taken out of history, the
  module's citation); `skein/docs/design/lib.md`, 6, `Secret`.
- Readers, surveyed: only `src/tests.rs` ~53, an `assert_eq!` on a `Taken` (its `Debug` is read
  only on a failure).

## Owns and may change

- `jig/crates/jig-client-shell-web/src/sign_in.rs`: `Taken`'s derive and its impl.
- `jig/crates/jig-client-shell-web/src/tests.rs`: the test below, appended.

Nothing else.

Pre-approved options:

- If clippy's `missing_fields_in_debug` fires, a scoped expect citing lib.md, 6.
- Formatting in the test: `alloc::fmt::format(format_args!(..))`, `std::format!` or a scoped
  `#[expect(clippy::disallowed_macros, reason = "...")]`, whichever the crate's lints accept.

## Steps

1. The test first, appended to `src/tests.rs` (the crate's lib tests, `tests`):

   ```rust
   #[test]
   fn debug_of_a_taken_link_shows_the_secrets_length_never_its_bytes() {
       // sign_in::Taken { secret: <32 made-up canary bytes>, address: b"/" }: formatted, it is
       // exactly "Taken { secret_len: 32, address: <derived> }", the canary absent in both forms
   }
   ```

   Run `heavy cargo nextest run -p jig-client-shell-web --lib -E 'test(/debug_of_a_taken_link/)'`
   on the old code: it fails. Keep the output.
2. The impl, a line above it saying what it hides.
3. Narrow tests, one at a time:
   - `heavy cargo nextest run -p jig-client-shell-web --lib -E 'test(/debug_of_a_taken_link|link_route/)'`: 2 passed.
   - `heavy cargo nextest run -p jig-client-shell-web --lib`.
4. `cargo fmt`; `python3 -I -B scripts/old_forms.py` from `jig/`;
   `heavy cargo clippy -p jig-client-shell-web --all-targets -- -D warnings`.
5. One commit, the impl and its test, saying why; push.

## Report

As the implementer's text says, and: the failing run on the old code, the passing runs and their
counts.
