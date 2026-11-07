+++
id = "world-setup"
part = "world-setup"
kind = "feature"
needs = []
files = [
  "jig/testing/jig-channel-kit/src/fixtures.rs",
  "jig/testing/jig-client-protocol-kit/src/native.rs",
  "jig/testing/jig-client-protocol-kit/src/engine.rs",
  "jig/testing/jig-client-protocol-kit/src/fixtures.rs",
  "jig/testing/jig-client-kit/src/test.rs",
  "jig/testing/jig-client-kit/src/joint/truth.rs",
  "jig/testing/jig-client-kit/src/joint/session.rs",
  "jig/testing/jig-test-profile/src/lib.rs",
  "jig/testing/jig-core-kit/src/world/constructors.rs",
  "jig/testing/jig-core-kit/src/world/store.rs",
  "jig/testing/jig-core-kit/src/oracle.rs",
]
effort = "medium"
minutes = 90
+++

# What every test of a world pays to set up, and then doesn't read

## Goal

A fixture many tests share is cheap to build, or a constant (`skein/docs/foundation/testing-strategy.md`, 8). A test runs in its own process, so a `OnceLock` still pays once per test. Remove the six repeats below, each its own commit, in this order. A world still starts the way the design says, and a story that tests startup still runs it.

Acceptance:

1. Each step below is done, or the report says why it's left. An item that proves harder than it reads is left: this task may land short.
2. No story checks less: no assertion, referee rule or cold restart goes. `Truth` reports the same `Observed`s in the same order, and its tests pass as they stand or with only their constructor changed.
3. The channel kit parses `ENGINE` once per process for `store_limits` and `store_names` together. The service's own startup still parses its document.
4. A `NativeClient` a referee holds builds no view and no phrasebook until something renders. One that renders behaves as it does today.
5. The focused tests of `jig-test-profile`, `jig-core-kit`, `jig-client-kit`, `jig-client-protocol-kit`, `jig-channel-kit`, `jig-core-world`, `jig-channel-world` and `jig-client-protocol-world` pass. `heavy cargo check --tests` passes on `jig-ops-world`, `jig-one-process-world`, `jig-conformance-world` and, read by their manifests' names, `temper/tests/client`, `temper/tests/one-process` and `temper/tests/engine/domain`.

## Inputs

- `skein/docs/foundation/testing-strategy.md`, 8 and 8.1.
- `jig/docs/design/domain/engine.md`, 6 for the ordered start and 15 for the cold restart that closes every story. Neither changes.
- `jig/docs/design/client/domain/testing.md`, 5 and 6, for `Truth`'s external image.
- `jig/docs/plan/jig-world-cost-2/plan.md`, *What setup repeats*.
- `jig/docs/plan/jig-world-cost-2/notes/common.md`.

## Owns and may change

- **Owns:** the spec's `files`.
- **Readers, read at the cut:**
  - `store_limits` and `store_names`: 75 call sites, whose signatures don't change. `providers_store_names` is the `Providers` story's own and stays.
  - `jig_client_kit::test::linked_with`: `native.rs:130` and `jig-client-protocol-kit/src/front.rs:272`. It stays for `front.rs`.
  - `Truth::after`: `jig-client-protocol-kit/src/engine.rs:59`, `jig-client-kit/src/joint/session.rs:101`, the tests in `truth.rs`, and `temper/tests/client/src/front.rs:663` and `:1244`. Its signature stays, so temper's lines don't change.
  - `fixtures::server_limits`, in the protocol kit: `engine.rs`, `jig/tests/ops/src/joint/bytes.rs:172`. `jig-engine-clients-kit` has its own of that name, not this one.
- **Departure:** none from the implementer's text. This task changes no design. `truth.rs`'s comment on `rows` changes with the field, and `jig/docs/design/client/domain/testing.md` says nothing of what it retains.
- **Options:**
  - The same kind of repeat beside these, in these paths (a copy, a rebuild, a value computed and dropped), local and plain from the code: fix it and name it in the report. Don't go looking further: `jig/docs/development/debt/D034-world-replay-cost.md`'s row feed, the observer's rebuild at each commit and the store's first start stay as they are.
  - `Core::new` builds its model table twice, and the first is read by `assert_catalogue` and `fleet_needs::configured`: leave it, as it's product code and not asked for here.
  - If `family_world`'s path (`jig-client-protocol-kit/src/fixtures.rs:169`) can be had from a public item of the profile with no new surface, use it. Otherwise leave it.

## Steps

1. **The channel kit parses once** (`jig-channel-kit/src/fixtures.rs`). One `OnceLock` holds what both read:

   ```rust
   /// What the kit reads of the fixture's engine, parsed once per test process.
   struct Admitted {
       limits: jig_test_service::limits::Limits,
       names: Box<[jig_domain_core::settings::ModelName]>,
   }
   fn admitted() -> &'static Admitted;
   // store_limits() and store_names() read it, with their signatures as they are.
   ```

   `engine_configuration` then has one reader; inline it if that reads better.
2. **A held client builds no view** (`native.rs`, `test.rs`). A client under `hold_for_account_sign_in` reads decoded facts and never renders, and `Test::build`, `Test::tree` and `Test::act` are its only reads of the view:

   ```rust
   // jig_client_kit::test: what `linked_with` makes, apart. It stays, as both of these.
   pub fn linked_root(limits: &domain::Limits, seed: u64) -> domain::Domain;
   pub fn linked_view(limits: &domain::Limits) -> view::View<'static>;

   // NativeClient: the view and its patch queue, built when first needed.
   struct Seen { view: jig_client_test_view::View<'static>, patches: Queue<skein_ui::Patch> }
   seen: Option<Seen>,
   ```

   Every read goes through one `get_or_insert_with`. Nothing renders while `account_sign_in_controlled` holds, and `finish_account_sign_in` clears it, after which it builds and renders as today.
3. **`Truth` keeps what it compares** (`joint/truth.rs`). `applied` compares `self.rows` only for a person's `Answer` record, so that's all it retains: `rows` keeps only those, `applied` ends with a `retain` on `snapshot.rows` before the move, and a new `Truth::seen(rows: &BTreeMap<root::Key, root::Record>) -> Self` makes the same `Truth` from a borrow. `after` stays as `Self::seen(&rows)`, for temper and the tests. `engine.rs:59` and `session.rs:101` call `seen` and lose their `.clone()`. `DurableSnapshot`, `StorePeer::capture` and the `facts` passes stay as they are.
4. **The protocol engine asks for the server's limits once** (`engine.rs`, `fixtures.rs`). `fixtures::server_limits()` builds a whole service profile, and `with_peers` asks twice, through `server()` and for `refused`'s capacity. Call it once there, with `fixtures::server_with(&limits)` beside `server()`. `server()` stays, as `jig/tests/ops/src/joint/bytes.rs` reads it.
5. **The profile's fixtures** (`jig-test-profile/src/lib.rs`, `constructors.rs`).
   - `effect_fixture`: the last `Domain::new(domain.rules().clone(), *domain.limits())...rules().clone()` is `domain.rules().clone()`. `valid_rules` has passed on those same rules and limits, and `new` keeps them as they came. The policy's admission and its `assert_eq!` stay.
   - `World::new`: `projects` is collected only in the `boot == Boot::Caller` branch that reads it. `constructors.rs`'s `fixture(seed, requirement).0` stays.
6. **A restart copies its configuration once** (`oracle.rs`, `world/store.rs`). The oracle's close calls `self.restart_with(configuration(&self.restart_config.0))`, and `restart_with` copies that back over the same value. Split it:

   ```rust
   pub fn restart_with(&mut self, config: root::Config) {
       self.restart_config = (crate::oracle::configuration(&config), self.limits);
       self.restart_configured(config);
   }
   /// A cold start under `config`, which `restart_config` already holds a copy of.
   pub(crate) fn restart_configured(&mut self, config: root::Config);
   ```

   The close calls `restart_configured`. Check first that `self.limits` equals `restart_config.1` on that path, as `restart_with_limits` sets both. If it doesn't, leave this step and say so.
7. No new test for what only moves a cost, except one in `truth.rs`: `an_answer_rewritten_after_seen_is_made_again_and_one_unchanged_is_not`, which breaks if `rows` drops what `applied` compares. Make `retain` keep nothing once, see it fail, and restore it. The existing `after` tests run as they are.
8. Narrow runs by name, then the implementer's closing steps for acceptance 5. Commit and push.

## Report

The implementer's report, plus, per step: done or left, and why. Any repeat you fixed beside them. Claim no saving, since nothing is measured.
