+++
id = "jig-clients-host"
part = "service-clients"
kind = "migration"
needs = ["jig-store-host"]
files = ["jig/crates/jig-protocol-engine-clients/src/host.rs", "jig/crates/jig-protocol-engine-clients/src/host/tests.rs", "jig/crates/jig-protocol-engine-clients/src/lib.rs", "jig/crates/jig-protocol-engine-clients-http/src/lib.rs", "jig/crates/jig-protocol-config/src/clients.rs", "jig/testing/jig-test-service/src/clients.rs", "jig/testing/jig-test-service/src/clients/public.rs", "jig/docs/design/protocol/service.status.md"]
effort = "high"
minutes = 90
+++

# jig: the clients' side as product code

## Goal

`J/clients.rs` and `T/clients.rs` (`J` and `T` are the paths `notes/service-parts.md` gives) hold the same bridge between the
component's secrets and the store's, the same sign-in endings, the same printed link and the same settings. The bridge
and the endings move to `jig_protocol_engine_clients::host`, the printed link's URL to `jig_protocol_engine_clients_http`,
beside `link_path`, and the settings to `jig_protocol_config::clients`, for the reason below.
`J`'s `Clients` keeps its fields' owner, its pass, and how each part's output reaches its root. `temper-clients-host`
does the same.

**The pass stays.** `iterate`, `advance`, `server_requests`, `up`, `down`, `work_pending` and `new`'s order stay in `J/clients.rs`,
whole. temper's google and provider code has its places in that order, and moving it isn't this task, whatever D001's
measure reads.

**Why the settings aren't in `host`.** `jig-protocol-config` depends on `jig-protocol-engine-clients`, so that crate can't name
`config::Engine`. A `host` function taking values would need each of `Settings`' fields as an argument, and each service
would still read them from configuration: no line would go. `kit::Settings` stays a value, and what both fill alike
goes beside `config::clients::models`, where configuration is already turned into the component's values.

Acceptance:

- `jig_protocol_engine_clients::host` and `jig_protocol_config::clients` hold the outer layer below.
- `Asked`, `secret_request`, `place`, `kept`, `owner`, `carry`, `ended`'s `people::Key` arms, `address_text`, `HOST_BYTES`,
  `public_bounds`, `public_kinds`, `endings_ready`, `ending_commit_durable` and the fields `endings`' map, `durable`,
  `secret_owners`, `next_secret` are gone from `J/clients.rs`.
- `Settings` is read from `config::clients::settings`; `J/clients.rs` no longer writes its literal.
- Every test passes, changed only as step 7 says. `tests::clients::sign_in_endings_follow_their_own_commits_under_load`
  keeps its name and module. `jig/docs/design/domain/root.md`, 5 names `ending_commit_durable`: the shipped `Endings` keeps
  a private method of that name.
- `jig/docs/design/protocol/service.status.md` no longer lists the clients' side.
- **Size cap:** the branch's net lines are at most +80, tests included: about 325 lines ship and `J/clients.rs` falls by about
  290. `temper-clients-host` then falls by about 245. Measure it, and stop and ask if it's passed.

## Inputs

- `temper/docs/plan/debt-7/notes/common.md`, `temper/docs/plan/debt-7/notes/service-parts.md`, `AGENTS.md`, `jig/AGENTS.md`.
- `jig/docs/design/protocol/service.md`, 11 and 11.6.
- `jig/docs/design/domain/root.md`, 5, on an ending's commit; `jig/docs/design/protocol/client.md`, 8.
- `temper/crates/temper-service/src/clients.rs`, for what the shape must also serve: its `secret_answer`, `Endings`.

## Owns and may change

- Owns: `jig-protocol-engine-clients`'s new `src/host.rs` and `src/host/tests.rs`, its `pub mod host;`;
  `jig-protocol-config`'s `src/clients.rs`, the additions alone; `J/clients.rs`; `J/clients/public.rs`.
- May change: `service.status.md`, as step 8 says, which departs from the implementer's text for that file alone.
- Checked by reading, needing no edit: `J/route.rs` (`clients.commit`), `J/engine.rs` and `J/stop.rs`, `J/tests/clients.rs`
  (`clients.commit`, `committed`, `secrets`, `up`, `iterate`, 19 private reads, none of a moved field), `J/startup.rs`. No manifest changes, and
  `jig-boundary`'s exact lists for the two crates hold: the clients' crate already links `jig-protocol-store`,
  `jig-domain-people` and `jig-domain-core`.
- Not yours: `T`.
- `jig/crates/jig-protocol-engine-clients-http/src/lib.rs` gains `printed_link`, beside `link_path`, and nothing else: the
  clients' crate speaks no HTTP (`jig/docs/design/protocol/README.md`, 5). Its `jig-boundary` list needs no edit.

## What differs, and which holds

From `notes/service-parts.md` and the survey. Every line keeps each service's behaviour.

- **`endings`:** the bound is a value: `Endings::new(u32::MAX)` here, as `Map::with_capacity(u32::MAX)` is today.
- **Secrets before the mark:** `PublicNames` parks in `J/clients.rs`, as it does; `Bridge` only takes what the owner gives it.
- **Component birth:** `settings`, then `start`, stay. **Hosts:** `up` builds them.
- **`not_ready`, `root_event`, `to_root`, `Routed`:** application-typed: stay.
- **The listener's failure and `Closed`, `advance`, the application's calls, `kit::Request`'s match, the bound checks:** in the
  pass: stay, as they are.
- **`moved::bound`'s check:** stays in `new`. `jig-protocol-config` can't name `jig-protocol-accounts`, so the bound comes in
  as a value.
- **`kit::Served`'s second row:** a value. Its `.or(Err(invalid(What::Bound)))?` goes: two pushes into a list of two.

`Bridge` holds no secret: `Asked` names an owner or an exchange, so it derives `Debug`. `Endings` holds numbers.
`Clients`' hand-written `Debug` stays, pinned by `debug_of_the_clients_shows_the_links_count_never_a_link`.

## Steps

1. `host.rs`, with a module doc citing `jig/docs/design/protocol/service.md, 11.6`, and `use jig_protocol_store as store;`:

   ```rust
   /// The bridge between the component's secrets and the store's (protocol/service.md, 11.6).
   #[derive(Debug)]
   pub struct Bridge { .. } // `owners: Map<Token, Asked>` with room `u32::MAX`, `next: u64` from 1, `namespace: u8`
   impl Bridge {
       pub fn new(namespace: u8) -> Self;
       /// Names a request and keeps what it was for; the owner calls its store with what this returns.
       pub fn ask(&mut self, request: SecretsRequest) -> (Token, store::Secrets);
       /// The component's answer to a store's terminal for a token `ask` named.
       pub fn answer(&mut self, answer: store::Event) -> SecretsAnswer;
       /// No request is in flight.
       pub fn idle(&self) -> bool;
   }
   /// Where the store keeps an owner's value; a carried value is the application's, under the number
   /// the store drew (store.md, 9).
   pub fn place(owner: SecretOwner) -> (store::SecretOwner, u64);

   /// Sign-ins' endings, each held until its own commit is durable (domain/root.md, 5).
   #[derive(Debug)]
   pub struct Endings { .. }
   impl Endings {
       pub fn new(commits: u32) -> Self;
       /// A commit ends this sign-in; a sign-in it already ends is one ending.
       pub fn ended(&mut self, commit: u64, sign_in: u64);
       pub fn committed(&mut self, number: u64);
       /// Room for one more commit's endings.
       pub fn room(&self) -> bool;
       /// An ending's commit is durable.
       pub fn ready(&self) -> bool;
       /// The endings held.
       pub fn waiting(&self) -> u32;
       /// The next ending whose commit is durable, in commit order.
       pub fn next(&mut self) -> Option<u64>;
   }
   /// The sign-in a core key's erasure ends.
   pub fn ending(key: &core::Key) -> Option<u64>;

   /// jig's own client protocol, then the application's family.
   pub fn served(application: Served) -> List<Served>;
   /// A bound address as text, `None` for a non-loopback IPv6 one.
   pub fn address_text(address: ::core::net::SocketAddr) -> Option<Box<[u8]>>;
   ```

   And in `jig_protocol_engine_clients_http`, beside `link_path`, using `kit::host::address_text`:

   ```rust
   /// The sign-in URL for a bound address, with the path `link_path` gave.
   pub fn printed_link(address: ::core::net::SocketAddr, path: &[u8]) -> Box<[u8]>;
   ```

   The bodies are `J/clients.rs`'s: `Asked`, `secret_request`, `place` and `kept` as they stand. `ask` is
   `advance_secrets`' loop's naming, with `Token::tagged(self.namespace, self.next)`. `answer` is `secret_answer`, with
   `self.owner` and `self.carry`, returning what it passed to `down`; where `kept` refuses, it returns
   `SecretsAnswer::Failed`. `ending` is the `core::Key` part of `ended`. `Endings` keeps a map from commit to a list and
   a `next` index, as `T/clients.rs`'s `Endings` does, with a private `ending_commit_durable(number)`; `next` takes the
   first entry and stops when its commit isn't durable; `ended` keeps the first error's text, `"bounded commits in flight"`.
2. `jig-protocol-config/src/clients.rs`, beside `models`:

   ```rust
   /// What an application adds to the clients' settings (protocol/service.md, 11.6).
   #[derive(Debug)]
   pub struct Application {
       pub link: Option<jig_domain_people::Identity>,
       pub served: List<kit::Served>,
       /// Its own public kinds, after jig's.
       pub public: List<kit::PublicKind>,
       pub moved_bytes: u64,
   }
   /// The provider of kind link, for the printed link's identity.
   pub fn link_provider(config: &Engine) -> Option<u16>;
   pub fn public_bounds(config: &Engine) -> jig_client_protocol_wire::PublicIndexBounds;
   /// The clients' settings, with no provider and no addresses (`client.md`, 18).
   pub fn settings(config: &Engine, application: &Application, seed: Secret, stream: u64) -> kit::Settings;
   ```

   `settings` is `J/clients.rs`'s literal, the providers' loop of `new`, `public_bounds` and `public_kinds`, with
   `kit::jig_kinds` first and `application.public` after, as `T/clients.rs`'s `client_settings` orders them. It clones
   `link`, `served` and the kinds, because a service may build it twice. Use `match` on `ProviderKind`, as `new` does.
3. `J/clients.rs`, the fields: `bridge: kit::host::Bridge` (`pub(super)`) for `secret_owners` and `next_secret`,
   `Bridge::new(crate::tokens::CLIENTS)`; `endings: kit::host::Endings` for `endings` and `durable`, `Endings::new(u32::MAX)`.
   `Debug`: `.field("endings", &self.endings)` and `.field("bridge", &self.bridge)` in their place. `work_pending` calls
   `self.endings.ready()`.
4. `J/clients.rs`, the passes:

   ```rust
   fn advance_secrets(&mut self, store: &mut crate::store::Store) {
       /* `public_names` as it stands */
       for _ in 0..self.secrets.len() {
           let request = self.secrets.pop().expect("client secrets requests");
           let (token, request) = self.bridge.ask(request);
           if let Some(answer) = store.host.secrets(token, request) {
               self.secret_answer(answer);
           }
       }
   }
   fn secret_answer(&mut self, answer: jig_protocol_store::Event) {
       let answer = self.bridge.answer(answer);
       self.down(kit::Event::Secrets(answer));
   }
   fn endings_step(&mut self, events: &mut Queue<root::Event>, bound: .., rows: u32, accounts: ..) {
       // Only the sign-ins retained when this stage starts can end here.
       for _ in 0..self.endings.waiting() {
           let Some(sign_in) = self.endings.next() else { return };
           self.down(kit::Event::SignInEnded { sign_in });
           self.advance(events, bound, rows, accounts);
       }
   }
   ```

   `commit` keeps its signature and its exhaustive `match write`, and its `Erase` arm calls `self.endings.ended(number, sign_in)`
   for what `ended(key)` gives. `committed` calls `self.endings.committed(number)`. `ended` is one arm for `root::Key::Core(key)`,
   calling `kit::host::ending(key)`, and one for the rest.
5. `J/clients.rs`, `new`: `link_provider` from `config::clients::link_provider`, then

   ```rust
   let application = config::clients::Application {
       link,
       served: kit::host::served(kit::Served {
           family: Some(1),
           first: jig_client_test_protocol_wire::VERSION,
           last: jig_client_test_protocol_wire::VERSION,
       }),
       public: List::with_capacity(0),
       moved_bytes: jig_protocol_accounts::moved::bound(&config.limits.driver.client.document),
   };
   let settings = config::clients::settings(config, &application, seeds.crypto.clone(), seeds.seed);
   ```

6. `J/clients.rs`, the link: `printed` stays, as `http::printed_link(address, &http::link_path(&http_settings(), secret))`;
   `up` calls `kit::host::address_text`.
7. Tests:
   - `public_names_are_kept_as_a_key_without_an_expiry` can't compile where it is: it moves to `src/host/tests.rs`, its
     assertions unchanged, its imports what the crate gives.
   - `the_public_key_lookup_waits_for_the_stores_durable_startup_mark`'s `clients.secret_owners.is_empty()` reads
     `clients.bridge.idle()`.
   - Add one test in `src/host/tests.rs`, `endings_leave_in_commit_order_behind_their_own_commit`: two commits of two
     sign-ins, the second durable first by number; `next` gives nothing before `committed`, gives the first commit's
     endings in order, then the second's. Break: `ending_commit_durable` always true.
8. `service.status.md`: remove `the clients' side` from **Not yet**, 11, and the `11:` sentence with it if it's then empty; date line.
9. Checks, one at a time: `cargo fmt`; `python3 -I -B jig/scripts/old_forms.py`; `heavy cargo clippy -p jig-protocol-engine-clients
   -p jig-protocol-engine-clients-http -p jig-protocol-config -p jig-test-service --all-targets -- -D warnings`; `heavy cargo nextest run -p jig-protocol-engine-clients
   -p jig-protocol-config -p jig-test-service -p jig-test -p jig-boundary -p jig-one-process-world -p jig-channel-world -p
   jig-engine-clients-world -p jig-client-protocol-world -p jig-ops-world`; `heavy cargo check -p temper-service --tests`; the
   citations check.
10. Commit and push.

## Report

The common note's items, with the test's break, and net lines against the cap.
