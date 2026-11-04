+++
id = "temper-clients-host"
part = "service-clients"
kind = "migration"
needs = ["jig-clients-host", "temper-store-host"]
files = ["temper/crates/temper-service/src/clients.rs", "temper/crates/temper-service/src/clients/printed.rs", "temper/crates/temper-service/src/tests/public.rs", "temper/docs/design/protocol/README.status.md"]
effort = "high"
minutes = 75
+++

# temper: compose jig's clients' side

## Goal

`T/clients.rs` (`T` and `J` are the paths `notes/service-parts.md` gives) drops its copy of the bridge, the sign-in endings, the
printed link's text and the settings it fills as jig does, and composes `jig_protocol_engine_clients::host` and
`jig_protocol_config::clients`, which `jig-clients-host` shipped. Its pass, its playground, its provider, google and
github calls stay exactly where they are: the pass's order is theirs. This is the last task that deletes a copy.

Acceptance:

- `Asked`, `Endings` (temper's), `secret_request`, `place`, `kept`, `owner`, `carry`, `endings_ready`, `PREFIX`, the fields
  `durable`, `writes`, `secret_owners`, `next_secret`, and `client_settings`' literal are gone from `T/clients.rs`.
- `Store::secrets` still counts every secrets' request: `advance_secrets` calls `store.secrets`, the adapter's, not
  `store.host.secrets`.
- `Clients` keeps its `#[expect(missing_debug_implementations, reason = "clients hold pending OAuth secrets")]`.
- Every test passes, changed only as step 6 says. `a_stalled_link_consumer_drops_nothing_and_keeps_order` and
  `fuzzy_route_endings_replay_from_the_seed` keep their names and modules.
- `temper/docs/design/protocol/README.status.md` holds no entry for the copies (step 7).
- **Size cap:** the branch's net lines are at most -200. Measure it, and stop and ask if it's passed.

## Inputs

- `temper/docs/plan/debt-7/notes/common.md`, `temper/docs/plan/debt-7/notes/service-parts.md`, `AGENTS.md`, `temper/AGENTS.md`.
- `jig/docs/design/protocol/service.md`, 11 and 11.6.
- `jig/crates/jig-protocol-engine-clients/src/host.rs` and `jig/crates/jig-protocol-config/src/clients.rs`, and
  `jig-clients-host`'s `J/clients.rs`: the same shape.

## Owns and may change

- Owns: `T/clients.rs`, `T/clients/printed.rs`.
- May change: `T/tests/public.rs`, as step 6 says; `README.status.md`, as step 7 says, which departs from the implementer's
  text for that file alone.
- Checked by reading, needing no edit: `T/route.rs:250` (`clients.commit`), `T/engine.rs` (`commit_room`, `committed`,
  `work_pending_with_secrets`), `T/clients/playground_tests.rs` (`secret_answer`), `T/tests/clients.rs` (`secret_terminals`, `commit_room`, 25 tests),
  `T/tests/sign_in.rs`, `T/tests/clients_unwatch.rs` (a `kit::Settings` literal of its own). `crate::clients::printed::address_text`
  stays a path, by a re-export, so `T/route.rs:231`, `T/clients/google.rs:444`, `T/clients/provider.rs:36` and `T/clients.rs:419` don't change.
- Not yours: anything in jig; `T/clients/{google,provider,github,fetch,bundle,translation}.rs`.

## What differs, and which holds

Every line keeps temper's behaviour.

- **`endings`:** `Endings::new(journal.commits)`, so `commit_room` holds the engine as today. `commit_room` stays a method,
  calling `self.endings.room()`.
- **A carried value for the playground:** stays temper's, in `secret_answer`, on what `Bridge::answer` returns (step 3). It
  covers both the store's `Carried` and its `Stale`, which `answer` both turn into `SecretsAnswer::Carried`.
- **`secret_terminals`, the test trace:** stays in `secret_answer`, after the playground's return, as it is. `Failed` carried
  no trace before and carries none now.
- **Secrets before the mark:** withheld by `T/engine.rs`, unchanged. `Bridge` knows nothing of it.
- **The closure at `T/clients.rs:222`:** stays in `new`, building the settings twice, since the provider's signer can't be
  cloned. It can't move, and this task doesn't reshape it: its last field reads `..config::clients::settings(..)`.
- **The link's identity, `sign_in`, the provider's list, public kinds' bound check:** stay in `new`.
- **`link_public`, `http_settings`, `hosts`:** stay.
- **The listener's failure and `Closed`, `advance`'s room, the application's calls, `kit::Request`'s match, `not_ready`, the
  bound checks:** in the pass: stay, as they are.

## Steps

1. Fields: `bridge: kit::host::Bridge` from `Bridge::new(skein_lib::Token::new(crate::tokens::CLIENTS).tag())`, and
   `endings: kit::host::Endings` from `Endings::new(journal.commits)`.
2. `new`: `link_provider` from `config::clients::link_provider(config)`. Before the closure:

   ```rust
   let application = application(config, link, kit::host::served(kit::Served {
       family: Some(temper_client_protocol_wire::number(temper_client_protocol_wire::Owner::Forge)),
       first: temper_client_protocol_wire::CLIENT_VERSION,
       last: temper_client_protocol_wire::CLIENT_VERSION,
   }));
   ```

   with `client_settings` replaced by this, beside `forge_public_kinds`:

   ```rust
   fn application(
       config: &config::Engine,
       link: Option<people::Identity>,
       served: List<kit::Served>,
   ) -> config::clients::Application {
       let mut public = List::with_capacity(3);
       for kind in forge_public_kinds() {
           public.push(kind).expect("three public kinds");
       }
       config::clients::Application { link, served, public, moved_bytes: moved_bytes(&config.limits) }
   }
   ```

3. The passes:

   ```rust
   fn advance_secrets(&mut self, store: &mut crate::store::Store) {
       /* `playground_carries` as it stands */
       for _ in 0..self.secrets.len() {
           let request = self.secrets.pop().expect("client secrets requests");
           let (token, request) = self.bridge.ask(request);
           if let Some(answer) = store.secrets(token, request) {
               self.secret_answer(answer);
           }
       }
   }
   fn secret_answer(&mut self, answer: jig_protocol_store::Event) {
       let answer = self.bridge.answer(answer);
       if let kit::SecretsAnswer::Carried { exchange, number } = &answer
           && self.owns_playground_read(*exchange)
       {
           self.playground_carried(*exchange, *number);
           return;
       }
       /* the `cfg(test)` trace, on `answer`, as it stands */
       self.down(kit::Event::Secrets(answer));
   }
   ```

   `endings_step`, `commit` and `ended` take `J/clients.rs`'s forms from `jig-clients-host`, with temper's arguments and
   `root::Key::Wires | root::Key::Forge(_) => None`. `commit` keeps `writes: &[root::Write]`.
4. `T/clients/printed.rs`: `address_text` becomes `pub(crate) use jig_protocol_engine_clients::host::address_text;`, and
   `link` calls `jig_protocol_engine_clients_http::printed_link`. `link_public` stays.
5. Check the two paths by eye against `T/clients.rs` at your base: a value carried for the playground, and a `Stale` for it.
6. Tests. `T/tests/public.rs`:
   - `the_public_names_key_is_kept_under_its_owner` calls `kit::host::place`.
   - `the_forge_declares_its_three_public_kinds` builds its settings with `config::clients::settings(&config.jig,
     &application(&config.jig, None, List::uncapped()), seeds.crypto.clone(), seeds.seed)`. Its assertions don't change.
7. `README.status.md`, date line and: from **Not yet**, `2: jig's service parts (jig's `protocol/service.md`, 11): `temper-service`
   holds its own copies.`; from **Departs**, the sentence `6, 7: the services are described in jig's shape, but ...`, up to and
   including `(the code's side is `temper/docs/development/debt/D001-reference-service-copy.md`).` Leave what the worker's
   tasks left about `SourceName`.
8. Checks, one at a time: `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p temper-service --all-targets
   -- -D warnings`. Then, as the last temper task, the direct dependents' tests whole: `heavy cargo nextest run -p temper-service -p temper
   -p temper-client-world -p temper-client-browser-world -p temper-engine-domain-world -p temper-engine-protocol-forge-world -p
   temper-fake -p temper-github-app -p temper-one-process-world -p temper-playground -p temper-playground-world -p
   temper-real-loop-world -p temper-system-world`. The citations check.
9. D001's measure, per file, as `notes/service-parts.md` says.
10. Commit and push.

## Report

The common note's items, the measure by file, and net lines against the cap. No test is added.
