+++
id = "jig-link-secret"
part = "jig-given-secret"
kind = "interface"
needs = []
files = ["jig/crates/jig-protocol-engine-clients/src/boundary.rs", "jig/crates/jig-protocol-engine-clients/src/component.rs", "jig/crates/jig-protocol-engine-clients/src/lib.rs", "jig/tests/engine-clients/tests/suite/sign_in.rs"]
effort = "high"
minutes = 35
+++

# jig's sign-in component takes a given first secret

## Goal

jig's one surface change: the component's settings can carry the first printed link's secret. Where one is given, the component takes it at start in the draw's place and says no `Request::Link`, because the application's startup holds it already.

Acceptance:

- `jig_protocol_engine_clients::Settings` has `pub link_secret: Option<Secret>`, and the crate exports `pub const LINK_SECRET_BYTES: usize = 32`.
- With one given, the secret signs in as the first link does, and the start says no `Request::Link`.
- Its expiry runs from `env.now` at `Component::new`, as a drawn one's does.
- Spent, expired or tried with another secret, it behaves as a drawn link: a fresh one is drawn and said, once. Not ready, busy and full spend nothing.
- `Component::new` keeps no copy in `self.settings`: it takes the value out as it starts.
- A given secret of any other length: `Component::new` answers `Err(Unusable::LinkSecret)`.
- Given where `settings.link` is `None`: no link is live and nothing is said.
- With none given, every existing test passes unchanged.
- `heavy cargo check -p temper-service --tests` passes without an edit in temper.

## Inputs

- `jig/docs/design/protocol/client.md`, section 8, its point *A secret given*.
- `temper/docs/design/draft/box-link-first.md`, section 3.
- `temper/docs/plan/box-link-first/notes/common.md`.

## Owns and may change

Owns `jig/crates/jig-protocol-engine-clients/src/boundary.rs`, `component.rs`, `lib.rs` (the `Settings` field, the constant, the `Unusable` variant), and the tests in `jig/tests/engine-clients/tests/suite/sign_in.rs`.

Reader and neighbour files, each only for the new field, `link_secret: None`, in a literal:

- `jig/crates/jig-protocol-config/src/clients.rs` (`settings`: temper reaches it by struct update, and so needs no edit).
- `jig/crates/jig-protocol-engine-clients/src/tests/fixtures.rs`.
- `jig/testing/jig-engine-clients-kit/src/fixtures.rs`: both literals; the one that copies a `Settings` copies the field with `.clone()`.
- `jig/testing/jig-client-protocol-kit/src/fixtures.rs`.

Options, each pre-approved:

- If `Peer::printed_link` in `jig/testing/jig-engine-clients-kit/src/peer.rs` panics when no link was said, keep it so, and read the absence from the peer's `links` (a one-line accessor there is allowed).
- If a test needs the URL of a given secret, use `fixtures::printed(&secret)`; if it is private, make it `pub`.
- If another `Settings` literal turns up in jig, add the field there, and say so in the report.

## Steps

In `component.rs`, the outer shape, inside which the choices are yours:

```rust
pub fn new(mut settings: Settings, env: &Env<Limits>, out: &mut Queue<Request>) -> Result<Self, Unusable> {
    let given = settings.link_secret.take();
    // a given secret of another length: Err(Unusable::LinkSecret), checked with the other refusals
    // ...
    // where it calls fresh_link today:
    // given and settings.link is some: self.link = Some(Link { secret, until: env.now + link_span }), no Request::Link
    // otherwise: value.fresh_link(env, out)
}
```

Write the tests first, in `jig/tests/engine-clients/tests/suite/sign_in.rs`, each with `Settings { link_secret: Some(Secret::new(Box::new([9; 32]))), ..fixtures::settings() }`:

- `a_given_secret_signs_in_as_the_first_link_and_no_link_is_said`
- `a_given_secret_spent_draws_a_fresh_link`
- `a_given_secret_expires_a_span_from_its_taking`
- `a_given_secret_answered_not_ready_is_not_spent`
- `a_given_secret_of_another_length_is_refused_at_start`
- `a_given_secret_without_the_printed_link_is_no_link`

Narrow runs with `heavy cargo nextest run -p jig-engine-clients-world --test suite -E 'test(given_secret)'` (read the package's name from `jig/tests/engine-clients/Cargo.toml`). Then fmt, clippy and the focused tests as your text says, with `heavy cargo check -p temper-service --tests` as the cross-tree check. Commit and push.

## Report

As your text says: the tests and what each found, the rule broken for each test that pins one, and any literal found beyond the list.
