+++
id = "link-secret"
part = "link-secret"
kind = "migration"
needs = []
files = ["jig/crates/jig-client-protocol-wire/schema/client-v19.schema", "jig/crates/jig-client-protocol-wire/src/generated/v19.rs", "jig/crates/jig-client-protocol-wire/src/tests.rs", "jig/crates/jig-client-protocol-wire/src/tests/link_sign_in.rs", "jig/crates/jig-client-protocol-link/src/sign_in.rs", "jig/crates/jig-protocol-engine-clients/src/component.rs", "jig/docs/design/protocol/README.status.md"]
effort = "medium"
minutes = 45
+++

# jig's client wire: the printed link's secret takes the generator's secret type

## Goal

`record LinkSignIn { secret: bytes 64 }` makes the generated `LinkSignIn` derive `Debug` over the printed link's secret. The field takes the generator's `secret` type, regenerated, with `LinkSignInParts` taking a `Secret` at its call sites across jig and temper. No compatibility is needed before v0.1.0, and the wire's bytes don't change.

Acceptance:

- The schema says `record LinkSignIn { secret: secret 64 }`. `family jig-client-wire 19` stays. `link_sign_in_secret` and `Path::LinkSignInSecret` keep their names and bound.
- `generated/v19.rs` comes from the generator alone. `git diff --stat` shows no file under `golden/` changed: a changed golden is a red, as the encoding is the same.
- `LinkSignIn` prints `LinkSignIn { secret: Secret { len: N } }`, its `secret()` returns `&skein_lib::Secret`, and `LinkSignInParts.secret` is a `skein_lib::Secret`.
- The test below passes, and failed before the schema changed.
- The checks in Steps 6 pass.

## Inputs

- `temper/docs/plan/debug-secrets/notes/common.md`, but for its departure on direct dependents: this task changes a public type, so the implementer's text holds on them
- `jig/docs/design/protocol/README.md`, 8 and 10
- `skein/docs/design/codec.md`, 3
- `jig/tests/codec-families.rs`: the drift check and its write switch

## Owns and may change

- **Owns:** the files in `files`.
- **Status entry:** in `jig/docs/design/protocol/README.status.md`, remove the `Departs: 8, 10: the printed link's secret ...` entry and keep the rest.
- **Reader files, jig, the call site only:**
  - `jig/crates/jig-protocol-engine-clients/src/tests/closing.rs`
  - `jig/testing/jig-test-service/src/tests/clients.rs`
  - `jig/tests/client-protocol/tests/suite/referee.rs`
  - `jig/tests/engine-clients/src/peer.rs`
  - `jig/tests/engine-clients/src/peer/closing.rs`
  - `jig/tests/engine-clients/tests/suite/sign_in.rs`
- **Reader files, temper, the call site only, each edited only in the field's type:**
  - `temper/crates/temper-service/src/tests/fixtures/http.rs`
  - `temper/crates/temper/tests/support/forgejo_sign_in.rs`
  - `temper/crates/temper-client-shell-terminal/src/client/tests/fixtures.rs`, where `link.secret()` becomes `link.secret().expose()`
- **`Secret` at a call site:** a test that decodes base64 wraps it with `skein_lib::Secret::new(..)`. `jig-client-protocol-link/src/sign_in.rs` passes the `Secret` it holds, moved if it can be and cloned if not, with no `copy_of`. `component.rs` compares `record.secret().expose()` through the same `constant_time_eq`.
- **If a golden changes, or another schema's generated file changes,** stop: that is a question. Generated output that only follows this field is expected.

## Steps

1. In a new `src/tests/link_sign_in.rs`, with `mod link_sign_in;` in `src/tests.rs`, write `debug_of_a_link_sign_in_shows_the_secrets_length_never_its_bytes`. It builds the record's wire bytes by hand with a `Writer` (the secret's length as a big-endian `u32`, then a made-up canary), so it compiles before and after the change. It decodes them with `LinkSignIn::decode(&CEILINGS, ..)`, then asserts as `notes/common.md` says and the exact form `LinkSignIn { secret: Secret { len: N } }`.
2. Run it: `heavy cargo nextest run -p jig-client-protocol-wire --lib -E 'test(debug_of_a_link_sign_in)'`. See it fail on the canary, and name that in your report.
3. Change the schema's field. Regenerate: `SKEIN_CODEGEN_WRITE=1 heavy cargo nextest run -p jig-client-wire-world -E 'test(drift::)'`. Read `git diff --stat`: only `generated/v19.rs` beside the schema.
4. Change the call sites above, with `heavy cargo check -p PKG --tests` while errors last. Add a round trip to the test if the compiler leaves room: encode, decode, `secret().expose()` is the canary.
5. Remove the status entry.
6. Run, one command at a time:
   - `heavy cargo nextest run -p jig-client-wire-world -E 'test(drift::)'` without the switch
   - `heavy cargo nextest run -p jig-client-protocol-wire -p jig-client-protocol-link -p jig-protocol-engine-clients -p jig-client-protocol-world -p jig-engine-clients-world`
   - `heavy cargo nextest run -p jig-test-service -E 'test(clients)'`
   - `heavy cargo nextest run -p temper-client-shell-terminal --lib`
   - `heavy cargo check -p temper-service -p temper --tests`
7. `cargo fmt`, `old_forms.py` for `jig` and `temper`, and clippy on the packages you changed, as the implementer's text has them. Commit and push.

## Report

As the implementer's text has it. Say what the regeneration changed by `git diff --stat`, and which temper lines you changed.
