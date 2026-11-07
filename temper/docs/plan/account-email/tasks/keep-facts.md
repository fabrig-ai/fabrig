+++
id = "keep-facts"
part = "email"
kind = "feature"
needs = ["facts-record", "oauth-profile"]
files = ["jig/crates/jig-domain-accounts/src/facts.rs", "jig/crates/jig-domain-accounts/src/signing.rs", "jig/crates/jig-domain-accounts/src/boundary.rs", "jig/crates/jig-domain-accounts/src/domain.rs", "jig/crates/jig-domain-accounts/src/changes.rs", "jig/crates/jig-domain-accounts/src/lib.rs", "jig/crates/jig-protocol-accounts/src/translate.rs", "jig/crates/jig-protocol-accounts/src/component.rs", "jig/crates/jig-protocol-accounts/src/party.rs", "jig/crates/jig-protocol-accounts/src/tests/**", "jig/tests/accounts/tests/suite/sign_in.rs", "jig/crates/jig-domain-core/src/tests/routing/account_signing.rs", "temper/crates/temper-engine-domain/src/store.rs"]
effort = "high"
minutes = 130
+++

# jig keeps what a sign-in and a refresh say

## Goal

Every registration of the accounts' driver asks for `email` and `name`. The child keeps them as kinds 1 and 2, at sign-in and refresh, by section 3.7's rules, and a client watching its accounts sees them change.

Acceptance:

- Section 4 of the contracts note, exactly, but for `Event::ProgramSaid`, which is `program-report`'s.
- `jig-protocol-accounts`, `tests/party.rs`: `party_account`'s terms ask for both claims. `tests/translate.rs`: `SignedIn`'s `profile` becomes `said`, and `Refreshed` becomes `FromComponent::Refreshed`.
- `jig/tests/accounts`, `sign_in.rs`, one table where the case is one behaviour, over what a report says and what the record holds after:
  - a sign-in saying both makes the account and saves `Stored::Facts` with kinds 1 and 2, origin 1, the source's number and `env.wall`;
  - a sign-in again that says neither drops them, and erases `Key::Facts`;
  - a sign-in refused for another holder's subject keeps nothing; one with plan usage declined keeps them;
  - a refresh saying a new email replaces it, origin 2, and leaves the name;
  - a refresh saying what's held writes nothing, `seen` included, and marks nothing;
  - a refresh naming a source that's gone is ignored;
  - a program's fact (origin 5) stands through a sign-in that says no email, and a sign-in that says one replaces it;
  - an unregistered kind's fact stands through a sign-in unless its origin is 1 or 2;
  - a report past 24 facts keeps those held;
  - an empty or 256-byte said is absent, and 255 bytes is kept.
- The same table or `changes.rs`: signing out or removing a source takes the facts that came through it, and rewrites the record in that decision; the last gone erases it; another source's stay.
- `jig-domain-core`, `tests/routing/account_signing.rs`: a refresh with a new email marks its holder's `accounts` document, so a watcher reads the fact, and one saying what's held marks nothing. Where the deployment holds the account, the deployment's document likewise.
- `temper-engine-domain/src/store.rs`'s `Stored::Facts` arm counts each fact's label and text bytes; if `facts-record` left another figure, set it and add a row where that function is tested.
- A change of meaning runs its readers' tests: the focused and fuzzy tests of `jig-domain-accounts`, `jig-protocol-accounts`, the accounts world, `jig/tests/protocol-accounts`, `jig-domain-core` and `temper-engine-domain` pass, and `heavy cargo check --workspace --tests`.

## Inputs

- `jig/docs/design/domain/accounts.md`, 3.7.
- `jig/docs/design/protocol/llm.md`, 5.2.
- `skein/docs/design/oauth.md`, 2.2.
- `temper/docs/plan/account-email/notes/contracts.md`, 1, 3 and 4.
- `temper/docs/plan/account-email/notes/common.md`.

## Owns and may change

Owns the header's files. Readers, each edited only for the field: `accounts::Event::SignedIn` literals gain `said: Said::default()`, in `jig/tests/accounts/tests/suite/{start_restore,moves,finish}.rs`, `jig/testing/jig-core-kit/src/world/account_preparation.rs`, `jig-domain-core`'s tests and any other the compiler names. `FromComponent` arms the compiler names. `jig-domain-people` has an `Event::SignedIn` of its own, and doesn't change.

Options, each pre-approved:

- `program-report` builds on this task and adds `Event::ProgramSaid`. `facts::report` takes a `source` and an `origin`, so that it calls the same function.
- The fuzzy accounts simulation may keep `said` empty: its model doesn't compare facts. If a saved `Stored::Facts` trips its oracle, repair the oracle for that row alone.
- If a route doesn't mark the document for a `FromComponent` event, mark it where that route's others are marked, only when a `Save` or `Erase` of `Key::Facts` went out.
- `model-catalogue-3` may also have changed `signing.rs` or `boundary.rs`; the second to land catches up.

## Steps

```rust
// facts.rs
/// Keep what a report says (accounts.md, 3.7). Saves or erases the record only if it changed.
pub(crate) fn report(domain: &mut Domain, env: &Env<Limits>, account: u32, source: u32, origin: u16, said: Said, out: &mut Queue<Request>);
/// Take the facts that came through a source gone.
pub(crate) fn source_gone(domain: &mut Domain, account: u32, source: u32, out: &mut Queue<Request>);

// signing.rs, signed_in: after make or renew, not after a refusal
crate::facts::report(domain, env, account, source, ORIGIN_SIGN_IN, said, out);
```

A renewal needs no `source_gone`: its sign-in's report drops every fact of origin 1 or 2. `source_gone` is for `changes.rs`'s removal of a source; where the account goes with its last source, `facts-record`'s erase does it.

Translate `oauth::ProfileClaims` to `accounts::Said` in `translate.rs`, the one place that names both. `party.rs`: `profile: oauth::Profile { email: true, name: true }`.

Each new test breaks its rule once by hand. `cargo fmt`; `python3 -I -B jig/scripts/old_forms.py`; `heavy cargo clippy` on the changed packages `--all-targets -- -D warnings`; `heavy cargo nextest run` on them and their direct dependents, one command at a time. Commit and push.

## Report

As the implementer's text says.
