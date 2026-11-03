+++
id = "jig-http-call-debug"
part = "jig-holders"
kind = "feature"
needs = []
files = [
  "jig/crates/jig-client-protocol-http/src/call.rs",
  "jig/crates/jig-client-protocol-http/src/tests.rs",
]
effort = "medium"
minutes = 35
+++

# jig-client-protocol-http's call prints header values and its body as lengths

## Goal

`jig/crates/jig-client-protocol-http/src/call.rs`'s `Call` (~11) derives `Debug, PartialEq, Eq`;
its `headers: List<skein_http::Header>` hold `Authorization: Bearer <credential>` (built at
~50-57), and `skein_http::Header` derives `Debug` over its value, so a debug print of a call
shows the bearer credential. Its `body` is the encoded message, a sign-in link's secret among
them (`Message::SignInLink`). Give `Call` a hand-written `Debug` that prints each header's name
and its value's length, and the body as its length; `PartialEq` and `Eq` stay derived.

Acceptance:

- `Call` prints `Call { method: .., path: .., headers: [Header { name: .., value_len: N }, ..],
  body_len: N }`: method and path as derived; no byte of a header's value or of the body.
- A keyed call (`exchange(&mapping(), Some(&credential()), ..)`) formats without the credential
  or its base64 form in either form (`notes/debug.md`), and with the header's name.
- The new test fails on the old code and passes on the new; every existing test passes unchanged.
- clippy clean; the crate's lib tests pass.

## Inputs

- `temper/docs/plan/bugs-security/notes/common.md` and `notes/debug.md`.
- `jig/AGENTS.md`.
- `temper/docs/plan/bugs-security/inputs/request.md`, item 4.
- `skein/docs/design/lib.md`, 6, `Secret`; `skein/docs/design/oauth.md`, 4 ("HTTP request
  bodies and headers that contain them are treated as secrets by the owner").
- For the shape of a list printed through a private wrapper: `skein/crates/skein-io/src/kernel.rs`'s
  `EnvNames` (~566).
- Readers, surveyed: `src/tests.rs` ~22-35 compares header values (`headers.contains(&bearer)`),
  not `Debug` text; `jig-client-shell-native`'s `Pending` (`connections.rs` ~43), which holds
  calls, has no `Debug`.

## Owns and may change

- `jig/crates/jig-client-protocol-http/src/call.rs`: `Call`'s derive, its impl and a private
  wrapper for the headers.
- `jig/crates/jig-client-protocol-http/src/tests.rs`: the test below, appended.

Nothing else: `skein_http::Header`'s own `Debug` is skein's, and this plan's question
`byte-carriers` (`plan.toml`); leave it.

Pre-approved options:

- If clippy's `missing_fields_in_debug` fires, a scoped expect citing oauth.md, 4.
- Formatting in the test: `alloc::fmt::format(format_args!(..))` or a scoped
  `#[expect(clippy::disallowed_macros, reason = "...")]`.

## Steps

1. The test first, appended to `src/tests.rs` (the crate's lib tests, `tests`):

   ```rust
   #[test]
   fn debug_of_a_keyed_call_shows_lengths_never_the_bearer_or_body() {
       // a keyed call of a message whose body holds a canary (Message::SignInLink(<canary>)),
       // with fixtures::credential(): formatted, "value_len: " and "body_len: " present, the
       // header's name present, neither the credential's bytes nor its base64 nor the canary
       // present, in either form
   }
   ```

   Run `heavy cargo nextest run -p jig-client-protocol-http --lib -E 'test(/debug_of_a_keyed_call/)'`
   on the old code: it fails. Keep the output.
2. The impl, a line above it citing oauth.md, 4.
3. Narrow tests, one at a time:
   - `heavy cargo nextest run -p jig-client-protocol-http --lib -E 'test(/debug_of_a_keyed_call/)'`: 1 passed.
   - `heavy cargo nextest run -p jig-client-protocol-http --lib`.
4. `cargo fmt`; `python3 -I -B scripts/old_forms.py` from `jig/`;
   `heavy cargo clippy -p jig-client-protocol-http --all-targets -- -D warnings`.
5. One commit, the impl and its test, saying why; push.

## Report

As the implementer's text says, and: the failing run on the old code, the passing runs and their
counts.
