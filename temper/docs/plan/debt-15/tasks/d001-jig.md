+++
id = "d001-jig"
part = "d001-kept"
kind = "feature"
needs = []
files = ["jig/crates/jig-protocol-config/src/clients.rs", "jig/crates/jig-protocol-config/src/tests/**", "jig/crates/jig-protocol-store/src/host/tests.rs", "jig/testing/jig-test-service/src/clients.rs", "jig/testing/jig-test-service/src/store.rs"]
effort = "medium"
minutes = 30
+++

# jig: what both services repeat of the clients' start and the read check's test

## Goal

The survey's small moves for `temper/docs/development/debt/D001-reference-service-copy.md` (the request's part 2), jig's
half: two helpers join `jig_protocol_config::clients`, the read check's test moves to the crate that holds the check,
and `jig-test-service` uses them. `d001-temper` is temper's half, built on this.

Acceptance:

- `jig_protocol_config::clients` has `invalid` and `link_owner`, as sketched, with a test each.
- `jig/testing/jig-test-service/src/clients.rs` has no `fn invalid` and no loop over `config.owners`.
- `a_store_whose_reads_are_below_its_page_is_refused` is in `jig-protocol-store/src/host/tests.rs`, with the same
  assertions, and not in `jig-test-service/src/store.rs`. `drivers_read_a_whole_page_in_every_admitted_store` stays: it
  checks that service's composition.
- `heavy cargo nextest run -p jig-protocol-config -p jig-protocol-store -p jig-test-service` passes, with the one test
  moved and the new ones added.

## Inputs

- `temper/docs/plan/debt-15/notes/common.md`
- `jig/docs/design/protocol/service.md`, 11.5 and 11.6: what stays each application's.
- `temper/docs/development/debt/D001-reference-service-copy.md`, **What pays it back**.

## Owns and may change

- **Owns:** `jig-protocol-config/src/clients.rs` and that crate's tests; `jig-protocol-store/src/host/tests.rs`;
  `jig-test-service/src/clients.rs` and `store.rs`, those lines alone.
- **Not to touch:** `temper-service`, which reads neither name until `d001-temper`; the pass in `clients.rs`.
- **Options:** if `crate::What` isn't reachable under that name in `clients.rs`, import it the way the module's `Path` and
  `Problems` are.

## Steps

1. `jig-protocol-config/src/clients.rs`, beside `link_provider`:

   ```rust
   /// The problem of the engine's clients' settings, at `engine.clients`.
   #[must_use]
   pub fn invalid(what: What) -> Problems;

   /// The owner the printed link signs in, the first with an identity at `provider`.
   #[must_use]
   pub fn link_owner(config: &crate::Engine, provider: u16) -> Option<&jig_domain_people::InitialOwner>;
   ```

2. `jig-test-service/src/clients.rs`: its `fn invalid` goes and its callers name `config::clients::invalid`; the loop
   becomes:

   ```rust
   let owner = config::clients::link_owner(config, provider).ok_or(config::clients::invalid(What::Refers))?;
   Some(people::Identity { key: owner.identity.clone(), login: ..., name: ... })
   ```

3. `a_store_whose_reads_are_below_its_page_is_refused` moves to `jig-protocol-store/src/host/tests.rs`, on that file's
   `fixtures::store_limits()`; `check_read_page` is that crate's. Trim `sizing_tests`' `use` to what stays.
4. Tests for `invalid` (the path it names) and `link_owner` (the first of two matching owners; none at another
   provider). Break each by hand.
5. `cargo fmt`; `heavy cargo clippy -p jig-protocol-config -p jig-protocol-store -p jig-test-service --all-targets -- -D
   warnings`; the three packages' tests; `heavy cargo check -p temper-service --tests`; the citations check. Commit and
   push.

## Report

The common note's items. The D001 measure's command, before and after your branch.
