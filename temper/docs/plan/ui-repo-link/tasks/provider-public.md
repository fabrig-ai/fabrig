+++
id = "provider-public"
part = "issuer"
kind = "interface"
needs = []
files = [
  "skein/crates/skein-oauth-issuer/src/**",
  "skein/tests/oauth-issuer/**",
  "skein/docs/design/oauth-issuer.status.md",
  "jig/crates/jig-protocol-config/src/clients.rs",
  "jig/crates/jig-protocol-config/src/clients/**",
  "jig/crates/jig-protocol-config/src/files.rs",
  "jig/crates/jig-protocol-config/src/members.rs",
  "jig/crates/jig-protocol-config/src/tests/provider.rs",
  "jig/docs/design/protocol/configuration.status.md",
]
effort = "high"
minutes = 60
+++

# The issuer's public client, and jig's configuration of it

## Goal

skein's issuer takes a client with no secret, client by client, with S256 and the exact redirect standing in for it, and jig's `engine.clients.provider` configures one, `"secret": null`. A box's owned Forgejo will use it (`temper/docs/design/protocol/sign-in.md`, 9.9). jig's `addresses` also takes a provider of kind link, since a box's owner is one.

This task departs from "a branch stays in its tree": it carries skein's mechanism and jig's one consumer together, since the field runs straight through, and temper's lines that read the field alone.

Acceptance:

1. `Client::secret` is `Option<Secret>`. Section 5's rules hold for a public client by the tests of its section 9: Basic with an empty password, Basic with any password, the body with no `client_secret` and with one; both methods at once `InvalidRequest`; an unknown id `InvalidClient`.
2. **Refused for a public client, each by a test:** a wrong redirect URI at the authorization and at the token request, no challenge at the authorization, no verifier at the token request, a wrong verifier. These tests fail against a build whose check you remove (see the implementer's text, *A new test must bite*).
3. **No other client is weakened:** with a public client and a confidential one in one issuer, the confidential one still refuses a wrong secret, an empty one and none, and a code drawn for one is refused to the other.
4. Discovery says `"none"` after the two methods when a client is public and is byte for byte today's otherwise; `new` refuses an empty `Some` secret as it refuses an empty one today.
5. jig: `"secret": null` parses to `None` and adds no file (`files.rs`); a missing member still refuses, as a required member does. `provider_issuer` gives `None` through. `addresses` takes a provider of kind link, and still refuses any other kind but application. The references it makes stale are regenerated.
6. `temper forge sign-in` for a public client refuses, saying that its client has no secret and a box adds its own source: exit as its other refusals, and it makes no file.

## Inputs

- `skein/docs/design/oauth-issuer.md`, 1, 2, 3, 5 and 9.
- `jig/docs/design/protocol/configuration.md`, 3, `clients`.
- `temper/docs/design/protocol/sign-in.md`, 9.9, *The client is public*.
- `temper/docs/plan/ui-repo-link/notes/contracts.md`; `notes/common.md`.

## Owns and may change

- Owns: this spec's `files`.
- Readers, field's type alone: `Client {` literals in `skein/crates/skein-oauth-issuer/src/tests/**`, `skein/tests/oauth-issuer/tests/support/mod.rs`, and any other `skein_oauth_issuer::Client` literal `rg` finds in `jig/crates/jig-protocol-engine-clients/src/tests/**`, `jig/tests/engine-clients/**`; `ProviderClient` readers: `jig/testing/jig-test-service/src/startup.rs`, `temper/crates/temper/src/forge_sign_in.rs` (acceptance 6), `temper/crates/temper-service/src/tests/**` where a literal needs it. `temper/docs/help/reference/configuration-*.md`: regenerated.
- Options:
  - The world (`skein/tests/oauth-issuer`) runs `skein_oauth::Client` as the issuer's peer, confidential. If that client has a public registration, add one story of the whole flow with it; if it doesn't, test the flow by hand in the same world and say so. Don't change `skein-oauth`.
  - `provider_issuer`'s probe client keeps its probe secret.

## Steps

Failing tests first, in `skein/crates/skein-oauth-issuer/src/tests/` and `jig-protocol-config/src/tests/provider.rs`. Then:

```rust
// requests.rs, the client's step: find the client by the id, whichever way it came; then
match &client.secret {
    None => {}                                       // read and dropped
    Some(secret) => /* the constant-time compare as today, a missing body secret InvalidClient */,
}
```

Then the narrow tests, `cargo fmt`, old forms for skein, jig and temper, `heavy cargo clippy -p skein-oauth-issuer -p jig-protocol-config -p temper --all-targets -- -D warnings`, `heavy cargo nextest run -p skein-oauth-issuer -p skein-oauth-issuer-world -p jig-protocol-config` (the world's package name as its `Cargo.toml` says), `heavy cargo check --tests -p jig-protocol-engine-clients -p temper-service`. Drop this task's **Not yet** from `oauth-issuer.status.md` (back to "No gap." if it's the last) and `configuration.status.md`. Commit and push.

## Report

As the implementer's text says.
