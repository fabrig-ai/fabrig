+++
id = "jig-accounts-host"
part = "service-accounts"
kind = "migration"
needs = ["skein-kernel-routes"]
files = ["jig/crates/jig-protocol-accounts/src/host.rs", "jig/crates/jig-protocol-accounts/src/host_tests.rs", "jig/crates/jig-protocol-accounts/src/lib.rs", "jig/crates/jig-protocol-accounts/src/component.rs", "jig/crates/jig-protocol-accounts/Cargo.toml", "jig/crates/jig-protocol-config/src/files.rs", "jig/crates/jig-protocol-config/src/lib.rs", "jig/testing/jig-test-service/src/accounts.rs", "jig/testing/jig-test-service/src/boundary.rs", "jig/testing/jig-test-service/src/lib.rs", "jig/testing/jig-test-service/src/engine.rs", "jig/testing/jig-test-service/src/clients.rs", "jig/testing/jig-test-service/src/startup.rs", "jig/testing/jig-test-service/src/tests/accounts.rs", "jig/testing/jig-test-service/src/tests/engine.rs", "jig/testing/jig-test-service/Cargo.toml", "jig/testing/jig-test/src/lib.rs", "jig/tests/one-process/src/accounts.rs", "jig/tests/boundary/src/accounts.rs", "jig/docs/design/protocol/service.status.md", "Cargo.lock"]
effort = "high"
minutes = 75
+++

# jig: the accounts' host, shipped as `jig_protocol_accounts::host`

## Goal

`J/accounts.rs` (`notes/service-parts.md` names `J` and `T`) is the accounts' host, written again in temper. Move it to
`jig_protocol_accounts::host` as product code, with the order of a pass inside it, and leave `jig-test-service` an
adapter: its tokens, its root's event and its store's calls. The host takes no store: it hands out its store requests,
and the adapter calls its own `store.secrets` and feeds the answer back. `Kernel` and `earliest` leave for skein's.

Acceptance:

- `jig_protocol_accounts::host` holds the surface below. `J/accounts.rs` holds what step 4 shows and nothing else: no
  `Kernel`, `earliest`, `unkept`, `copy_origins`, `copy_endpoint`, `has_tls`, `provider_templates`,
  `registration_input`, `untag_store` or `impl Files`.
- `jig_protocol_config::Files` exists with a public `contents`, and `jig_test_service::Files` is that type.
- Every `jig-test-service` test and every world that reads it passes, with no test edited but as `Owns and may change`
  lists. Their visible events and outcomes don't change.
- The tests of step 6 exist and each has been seen to fail on its rule.
- `service.status.md` no longer lists the accounts' host.
- **Size cap:** the branch's net lines are at most +130, tests included; about +100 is expected, as the host gains its
  tests and its docs. `temper-accounts-host` takes the part below where it started. Measure it, and stop and ask if it's
  passed.

## Inputs

- `temper/docs/plan/debt-7/notes/common.md`, `temper/docs/plan/debt-7/notes/service-parts.md`, `AGENTS.md`,
  `jig/AGENTS.md`.
- `jig/docs/design/protocol/service.md`, 11 and 11.1: the contract. `jig/docs/design/protocol/llm.md`, 5.
- `jig/docs/development/debt/D005-accounts-host-copied.md`: read it. This task doesn't pay it; `service-items` does.
- `temper/crates/temper-service/src/accounts.rs`, read only: `aborting` and `pending` are the two things you bring
  across. `T/tests/accounts_host.rs` shows what `temper-accounts-host` needs of the surface.
- `skein/crates/skein-io/src/kernel/routes.rs`, from `skein-kernel-routes`.

## Owns and may change

- Owns: the new `jig-protocol-accounts/src/host.rs` and `host_tests.rs`; `J/accounts.rs`.
- `jig-protocol-accounts/src/lib.rs`: `pub mod host;`. `src/component.rs`: `fn unkept` becomes `pub(crate)`, nothing
  else. `Cargo.toml`: `jig-protocol-config.workspace = true` and `skein-kv.workspace = true`. There is no cycle: nothing
  `jig-protocol-config` links names this crate.
- `jig-protocol-config/src/files.rs` and `src/lib.rs`: `Files`, as step 2 says.
- `jig/tests/boundary/src/accounts.rs`. It lists what this crate may link, so this one rule widens, and it departs from
  the implementer's text and the common note's for this file: add `"jig-protocol-config"` and `"skein-kv"` to the list,
  say "its child, store, configuration and skein's drivers" in the module doc, and rename the test
  `the_accounts_part_links_only_its_child_store_configuration_and_skein`. No document cites the old name.
  `accounts_fixture_checks_each_normal_dependency_form` still reports `jig-domain-core` and doesn't change.
- `jig/docs/design/protocol/service.status.md`: delete `the accounts' host, ` from **Not yet**, 11, and update its date
  line. This departs from the implementer's text.
- `jig/testing/jig-test-service/Cargo.toml`: delete `skein-oauth`, which only `J/accounts.rs` names. `Cargo.lock`.
- The readers, each edited only as this table says:

  | File | Edit |
  |---|---|
  | `J/boundary.rs` | the `Files` struct goes; `pub use jig_protocol_config::Files;` in its place |
  | `J/lib.rs` | `kernel: accounts::Kernel` and `accounts::Kernel::new(` name `skein_io::kernel::Routes`; `accounts.startup(source)` in `account_startup` reads `accounts.component().startup(source)`; `pub use accounts::{AccountsRuntime, TrustBounds};`. `accounts::{owns, tag, untag}` stay |
  | `J/engine.rs` | 8 `crate::accounts::earliest` on 7 lines become `skein_lib::earliest`; `accounts.iterate(..)` becomes `crate::accounts::iterate(accounts, ..)`. `accounts.work_pending(now, self.store.component.mark_durable())` stays |
  | `J/clients.rs` | 1 `crate::accounts::earliest` becomes `skein_lib::earliest` |
  | `J/startup.rs` | `Accounts::new`'s third argument `seeds` becomes `&seeds.crypto, &seeds.installation` |
  | `J/tests/accounts.rs` | `accounts.iterate(..)` becomes `iterate(accounts, ..)`; the imports `use super::*` no longer brings |
  | `J/tests/engine.rs` | `crate::accounts::Kernel::new(4)` becomes `skein_io::kernel::Routes::new(4)` |
  | `jig/testing/jig-test/src/lib.rs` | the constructor: `trust: Some(service::TrustBounds { roots: trust_roots, bytes: trust_root_bytes })` |
  | `jig/tests/one-process/src/accounts.rs` | the constructor: `trust: Some(jig_test_service::TrustBounds { roots: 0, bytes: 0 })` |

  Neither `jig-test` nor `jig-one-process-world` links `jig-protocol-accounts`, so they take `TrustBounds` through
  `jig_test_service`, and neither manifest changes. `crate::accounts::owns` in `J/store.rs` and `J/tokens.rs`, `jig/tests/one-process/src/world.rs`, and the `Files { .. }` literals in
  `jig/tests/channel`, `jig/tests/client-protocol`, `jig/tests/one-process` and `jig-test` need no edit. This corrects
  the survey, whose counts mix `earliest` with `Kernel`.
- Not yours: any temper file. `temper-client-shell-cli` names `jig-protocol-accounts` in its manifest and no source of
  it uses the crate; leave that. `Seeds` stays where it is. The workers' family reads `Files`.
- jig's files name no application. Write "an owner".

### Where the copies differ

`notes/service-parts.md` holds; this is how, for the points of the survey's table:

| Point | Holds | How |
|---|---|---|
| Trust-root bounds, `has_tls` | both, as a value | `Runtime::trust`. jig passes its bounds, so its refusal stands |
| `Effect::InvalidSecret` | each owner | `Output::InvalidSecret`; jig's adapter pushes `PageRefused`, as today |
| The output-room hold | temper | `Accounts::hold`. jig's adapter never calls it, and the slot stays empty |
| `aborting` | temper's | in the host: no `Opened` after `abort`, a store effect dropped, the open no work. It changes jig only on abort paths |
| `work_pending` | one form | `(marked && !opened && !aborting)`, then today's terms and `held`. jig passes the mark, as today |
| `unkept` on `HandedIn` | jig's | the component's, which `party_account` can't reach with `HandedIn` |
| Root event, tokens | each owner | the adapter |
| `startup`, `return_to`, reads | both | public on the host, for the owner that reads them |

## Steps

1. Read the item, the sections and temper's copy. Commit each step below that compiles on its own.
2. `Files` moves. In `jig-protocol-config/src/files.rs`, with its derive as it stands, and `Files` in `lib.rs`'s
   `pub use files::{..}`:

   ```rust
   /// The files a configuration names, by path, as a process's shell read them (configuration.md, 7).
   #[derive(Debug)]
   pub struct Files {
       pub entries: List<(Box<[u8]>, Secret)>,
   }

   impl Files {
       /// The bytes read for `path`.
       #[must_use]
       pub fn contents(&self, path: &[u8]) -> Option<&[u8]>;
   }
   ```

   The body is `impl Files`'s in `J/accounts.rs`.
3. `host.rs`, with a module doc citing `jig/docs/design/protocol/service.md, 11.1`. The surface, final. Doc comments
   say what `service.md`, 11.1 says of each, in a line. `no_std`: no `Vec`, closure, `while` or `loop` (the subset of
   `skein/docs/foundation/programming-model.md`, 10).

   ```rust
   /// What a process's shell prepares for sign-ins.
   #[derive(Debug)]
   pub struct Runtime {
       /// LLM account sign-in endpoints and origins, keyed by provider number.
       pub providers: List<(u32, driver::Endpoint, List<driver::Origin>)>,
       /// Bounds of all prepared trust roots, supplied by their shell owner; `None` checks nothing.
       pub trust: Option<TrustBounds>,
   }

   #[derive(Clone, Copy, Debug, PartialEq, Eq)]
   pub struct TrustBounds {
       pub roots: u32,
       pub bytes: u32,
   }

   /// What the host can't decide. The owner takes each before it asks for the next.
   #[expect(missing_debug_implementations, reason = "a store request carries secrets' values")]
   pub enum Output {
       /// An event for the root, in the component's order.
       Account(domain::Event),
       /// A request to the secrets' store, on the host's exchange token; the owner tags it.
       Store { exchange: Token, request: store::Secrets },
       /// A restored secret of the wrong kind: the owner says what that means.
       InvalidSecret,
   }

   #[expect(missing_debug_implementations, reason = "the accounts component holds token records")]
   pub struct Accounts {
       component: Component,
       env: Env<driver::Limits>,
       /// Requests for the owner to tag and submit, as its room allows.
       pub io: Queue<io::Request>,
       effects: Queue<Effect>,
       opened: bool,
       held: Option<domain::Event>,
       aborting: bool,
   }

   impl Accounts {
       /// Checks public provider registrations; restored sources are admitted after the store opens.
       pub fn new(
           application: Box<[u8]>,
           config: &config::Engine,
           crypto: &Secret,
           installation: &[u8; 16],
           runtime: Option<Runtime>,
           secrets: &skein_kv::Limits,
       ) -> Result<Self, Problems>;

       pub fn clock(&mut self, now: Time, wall: Wall);
       pub fn request(&mut self, request: domain::Request);
       pub fn return_to(&mut self, source: u32, location: Box<[u8]>);
       pub fn up(&mut self, event: io::Event);
       pub fn abort(&mut self);
       pub fn take_moved(&mut self, source: u32) -> Option<Secret>;

       /// A pass's first three stages, in order: `Opened` once `marked`, each of `answers`, the component's fire.
       /// Returns the pass's count, for `output`.
       #[must_use]
       pub fn begin(
           &mut self,
           now: Time,
           wall: Wall,
           marked: bool,
           installation: [u8; 16],
           answers: &mut Queue<store::Event>,
       ) -> u32;
       /// One effect, in the component's order; `None` when it stayed inside.
       #[must_use]
       pub fn output(&mut self, table: &mut Table) -> Option<Output>;
       /// A store's answer, from `store::Secrets` within the pass; its namespace comes off.
       pub fn answer(&mut self, event: store::Event);
       /// Gives back an `Output::Account` the owner has no room for: it leaves first in the next pass.
       pub fn hold(&mut self, event: domain::Event);

       #[must_use]
       pub fn is_empty(&self) -> bool;
       #[must_use]
       pub fn next_deadline(&self) -> Option<Time>;
       /// `marked`: the store's start mark is durable. An owner whose own gate waits for it passes `true`.
       #[must_use]
       pub fn work_pending(&self, now: Time, marked: bool) -> bool;

       #[must_use]
       pub fn component(&self) -> &Component;
       #[must_use]
       pub fn opened(&self) -> bool;
       #[must_use]
       pub fn effects(&self) -> u32;
       #[must_use]
       pub fn held(&self) -> bool;
   }
   ```

   The bodies are `J/accounts.rs`'s, with temper's two additions. `begin` is the head of `iterate`:

   ```rust
   self.clock(now, wall);
   if marked && !self.opened && !self.aborting {
       self.component.step(&self.env, Event::Opened { installation }, &mut self.effects);
       self.opened = true;
   }
   for _ in 0..answers.len() {
       self.answer(answers.pop().expect("accounts store answers"));
   }
   self.component.fire(&self.env, &mut self.effects);
   self.effects.len().checked_add(u32::from(self.held.is_some())).expect("accounts effects")
   ```

   `output` is the body of `iterate`'s loop, one effect a call:

   ```rust
   if let Some(event) = self.held.take() {
       return Some(Output::Account(event));
   }
   match self.effects.pop().expect("accounts effects") {
       Effect::Child(event) => Some(Output::Account(event)),
       Effect::Io(request) => {
           self.io.push(request);
           None
       }
       Effect::Store { .. } if self.aborting => None,
       Effect::Store { exchange, request } => Some(Output::Store { exchange, request }),
       Effect::Granted { source, generation, value, expires } => {
           table.granted(source, generation, value, expires).expect("admitted grant");
           None
       }
       Effect::Removed { source } => {
           table.remove(source);
           None
       }
       Effect::SourceRemoved { .. } => None,
       Effect::InvalidSecret => Some(Output::InvalidSecret),
   }
   ```

   - `answer` steps `Event::Store(untagged(event))`, where `untagged` is `untag_store` as `J/accounts.rs` has it, every
     arm kept, with `Token::new(owner.untagged())`.
   - `hold` is `assert!(self.held.is_none(), "one held accounts event");` then the slot. `abort` sets `aborting`, then
     steps `Event::Abort`, as temper's does. `is_empty` and `work_pending` count `held`.
   - `new`'s checks keep their order and their `What`s. `seeds.crypto` becomes `crypto` and `&seeds.installation`
     becomes `installation`. Split the end of `new` into a private `fn made(component: Component, limits:
     driver::Limits) -> Self`, which `host_tests.rs` builds on, since a `config::Engine` is out of its reach.
   - `provider_templates`, `registration_input`, `has_tls` and `problem` move. `use crate::component::unkept`;
     `copy_origins` and `copy_endpoint` go with jig's copy of `unkept`. Repeated
     `Path::root().member(b"engine").member(b"llm").member(b"providers")`s may fold into `problem(what)`.
   - The trust check, in `provider_templates`, is now one private function, so a test can reach it:

     ```rust
     /// Trust roots a shell's bounds refuse; a shell that gives none answers for its own.
     fn untrusted(bounds: Option<TrustBounds>, tls: bool) -> bool;
     ```

     `Some` is today's `(roots == 0 || bytes == 0) && tls`, `None` is `false`, and `tls` is `has_tls(&endpoint,
     &origins)`.

4. `J/accounts.rs` becomes the adapter, whole. This is what D001's measure sees of it: with temper's, 25 distinct lines
   against 247 today, by a draft of both.

   ```rust
   //! The reference service's accounts: jig's host around its tokens, its root and its store
   //! (jig/docs/design/protocol/service.md, 11.1).
   use crate::values::Values;
   use jig_domain_core as core;
   use jig_protocol_accounts::host::Output;
   pub use jig_protocol_accounts::host::{Accounts, Runtime as AccountsRuntime, TrustBounds};
   use jig_protocol_store as store;
   use jig_test_domain as root;
   use skein_io as io;
   use skein_lib::{Queue, Time, Token, Wall};

   /// One pass of the host: each output is routed before the next is asked for.
   pub(super) fn iterate(
       accounts: &mut Accounts,
       now: Time,
       wall: Wall,
       values: &mut Values,
       events: &mut Queue<root::Event>,
       store: &mut crate::store::Store,
   ) {
       let marked = store.component.mark_durable();
       let count = accounts.begin(now, wall, marked, store.installation, &mut store.sources);
       for _ in 0..count {
           match accounts.output(&mut values.accounts) {
               Some(Output::Account(event)) => events.push(root::Event::Core(core::Event::Account(event))),
               Some(Output::Store { exchange, request }) => {
                   if let Some(event) =
                       store.secrets(crate::tokens::token(crate::tokens::ACCOUNTS, exchange.raw()), request)
                   {
                       accounts.answer(event);
                   }
               }
               Some(Output::InvalidSecret) => store.above.push(store::Event::PageRefused {
                   owner: Token::new(0),
                   failure: store::restore::Failure {
                       cause: store::restore::Cause::Decode(store::restore::DecodeFailure::Value),
                       row: None,
                   },
               }),
               None => (),
           }
       }
       values.accounts.expire(now);
   }

   pub(super) fn owns(token: Token) -> bool {
       token.tag() == crate::tokens::ACCOUNTS
   }
   pub(super) fn tag(request: io::Request) -> io::Request {
       crate::tokens::tag(request, crate::tokens::ACCOUNTS)
   }
   pub(super) fn untag(event: io::Event) -> io::Event {
       crate::tokens::untag(event)
   }

   #[cfg(test)]
   #[path = "tests/accounts.rs"]
   mod tests;
   ```

   The pass's order is today's, stage by stage: the clock, `Opened`, `store.sources`, `fire`, then one count of effects.
   `count` is taken once: what an effect's handling makes, the immediate answer's work among it, waits for the next pass.
5. The readers of the table. skein's `earliest` and `Routes` have the copies' signatures, and `Routes` the copies' body.
6. `host_tests.rs`, hung from `host.rs` as `component_tests.rs` is from `component.rs`, built on `made` and
   `crate::tests::fixtures::driver_limits`, as `component_tests.rs`'s `machine` is. Each pins one rule of
   `service.md`, 11.1:

   | Test | The rule | Break it by |
   |---|---|---|
   | `an_answer_taken_at_begin_is_stepped_before_the_fire_that_counts_its_work` | answers before the fire, a tagged exchange's namespace off: `Keep`, `begin` hands out `Get`, tag the exchange with `Token::tagged`, answer it through `answers`, and `begin` hands out `Adopt` | firing first |
   | `opened_reaches_the_component_only_when_marked` | a `Keep` waits for the mark: `begin` with `marked` false counts 0 and leaves `opened()` false; with `true`, `opened()` holds and `output` hands out its `Get` | dropping `marked` |
   | `a_held_event_leaves_first_and_is_work_until_it_does` | `hold`, then `held()`, `!is_empty()`, `work_pending(now, false)`; `begin` counts it with a waiting effect, and `output` gives it first | leaving `held` out of the count |
   | `a_grant_reaches_the_table_before_its_event_leaves` | `Effect::Granted` gives `None` with the value in the table, then `Output::Account` | skipping `table.granted` |
   | `an_aborted_host_opens_nothing_and_drops_its_store_requests` | with a `Store` effect waiting, `abort`, and `output` gives `None`; a host aborted before its mark stays unopened, and `work_pending(now, true)` is false once drained | removing the `aborting` guard |
   | `a_restored_secret_of_the_wrong_kind_is_reported` | a `Get` answered with the wrong `SecretValue` gives `Output::InvalidSecret` | mapping it to `None` |
   | `trust_bounds_refuse_tls_alone_and_none_checks_nothing` | `untrusted`'s four cases | making `None` answer `tls` |

7. Remove the host from the status file.
8. `cargo fmt`, and from `jig/`, `python3 -I -B scripts/old_forms.py`. Then, one command at a time:

   ```
   heavy cargo clippy -p jig-protocol-accounts -p jig-protocol-config -p jig-test-service -p jig-test -p jig-one-process-world -p jig-boundary --all-targets -- -D warnings
   heavy cargo nextest run -p jig-protocol-accounts -p jig-protocol-config -p jig-protocol-accounts-world -p jig-test-service -p jig-test -p jig-boundary
   heavy cargo nextest run -p jig-one-process-world -p jig-channel-world -p jig-engine-clients-world -p jig-client-protocol-world -p jig-ops-world
   TEMPER_BUNDLE=none heavy cargo check -p temper-service -p temper-client-shell-cli -p temper-one-process-world --tests
   ```

   `aborting` changes what a shared item means on abort paths, so the worlds that read it are part of this task. A
   test's visible events changing there is a stop. The citations check. Commit and push.

## Tests

- **Stay, edited as the table says:** `J/tests/accounts.rs`'s 3, among them `a_restored_sources_wrong_kind_stops_the_engine`;
  `kernel_operations_are_routed_back_to_their_owner` in `J/tests/engine.rs`.
- **None moves.** The 7 of step 6 are new, and small, in `jig-protocol-accounts`' `--lib`.
- **Cited tests keep their names and modules:** `tests::engine::the_store_root_closes_after_delayed_driver_completions`
  and `tests::clients::sign_in_endings_follow_their_own_commits_under_load` need no edit.

## Report

The common note's items. The table of breaks: the 7 tests, what was broken, what failed. Net lines against the cap,
split into `jig-protocol-accounts`, tests, and the rest. Any test whose assertion you wanted to change, which is a stop.
