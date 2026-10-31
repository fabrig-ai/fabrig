+++
id = "fr-registry"
part = "forge-records"
kind = "interface"
needs = ["fr-ask"]
files = [
  "temper/crates/temper-service/src/forge/registry.rs",
  "temper/crates/temper-service/src/forge/stock.rs",
  "temper/crates/temper-service/src/forge.rs",
  "temper/crates/temper-service/src/settings.rs",
  "temper/crates/temper-service/src/engine.rs",
  "temper/crates/temper-service/src/route.rs",
  "temper/crates/temper-service/src/workers.rs",
  "temper/crates/temper-service/src/setup.rs",
  "temper/crates/temper-service/src/clients.rs",
  "temper/crates/temper-service/src/startup.rs",
  "temper/crates/temper-service/src/startup/application.rs",
  "temper/crates/temper-service/src/tests/registry.rs",
  "temper/crates/temper-service/src/tests.rs",
]
effort = "high"
minutes = 90
+++

## Goal

temper-service learns to build a forge from its record
(`temper/docs/design/draft/playground-reset.md, 3`): a registry that
reads a record's definition with the readers that read the document's
forge members today, the stock definitions per kind (today's template
entries, moved into code), the machine's facts they need, forge seeds
stepped at start, and one install path that configures the domain and
calls each consumer's `add`. The consumers' `add`s are empty here; the
four consumer tasks fill them. Every forge the deployment has today
stays a seed, built at start as today, so nothing changes in behavior.

Acceptance, new tests in `temper-service` (`src/tests/registry.rs`):

- `each_stock_definition_reads_back_to_its_kind`: `stock::owned`,
  `stock::github` and `stock::playground` on a test `Machine` (ports
  set, a person) each give a definition that `registry::read(_, 5, _)`
  reads to a `Built` of the stock's kind and mode (owned Forgejo
  `Owned`, asked, its setup entry numbered 5 in `forge-5`, listening on
  `ports.owned`; GitHub `Connected`, no setup; the playground's fake
  `Fake`, listening on `ports.forge`, owner `playground`, repository
  `widget`), its identity numbered 5.
- `a_definition_naming_a_number_is_refused`: a definition whose forge
  has `number`, whose identity has `number` or `forge`, or whose setup
  has `forge`, is refused naming the member; so is one past
  `forge.definition_bytes`.
- `the_document_forges_become_seeds`: today's startup fixture document
  (`src/tests/fixtures/startup.rs`) gives `settings::Engine.forge_seeds`
  one seed per `application.forges` entry, its number the entry's, its
  definition `{"forge": F, "identity": I, "setup": S}` less their
  numbers, and `registry::read` reads each back to the same kind, mode
  and git as the resolved `settings::Forge`.
- `setup_member_numbers_the_entry`: for a held owned record, the
  member is the definition's `setup` with `"forge": N` and its
  `directory`.
- Every existing `temper-service` test passes unchanged in meaning.

## Inputs

- `temper/docs/plan/playground-reset-r2/notes/common.md` and
  `temper/docs/plan/playground-reset-r2/notes/scope.md`.
- `temper/docs/plan/playground-reset-r2/notes/contracts.md`, sections 2
  (the definition) and 4 (the machine's ports).
- `temper/docs/design/draft/playground-reset.md`, section 3.
- `temper/docs/design/protocol/configuration.md`, section 3 (the forge
  and identity members a definition holds);
  `temper/docs/design/protocol/setup.md`, 8 (the setup entry), 8.3 (the
  default document's three forges, which the stock moves into code).
- `temper/crates/temper-init/template/temper.json`, its
  `application.forges`, `identities` and `setup` (the stock's source).
- `tasks/fr-ask.md` (`NewForge`, `Event::Seed`, `configure_forge`,
  `forge_record`, `Request::ForgeMade`).

## Owns and may change

Owns, after `fr-ask` (the consumer tasks and `fr-document` own their
files next): the new `src/forge/registry.rs` and `src/forge/stock.rs`;
in `src/settings.rs` the new `Machine`, `Ports` and `ForgeSeed` types
and the `Engine` fields `machine` and `forge_seeds`; in
`src/engine.rs` the `registry` field and the call that runs
`registry::restored`; `src/route.rs`'s `ForgeMade` arm; the empty
`add` functions in `src/forge.rs`, `src/workers.rs`, `src/setup.rs`,
`src/clients.rs`.

May change:

- `src/startup/application.rs`: the forge and identity readers
  (`fn forge`, `:355`; `fn identity`, `:502`) made `pub(crate)` and
  given a mode that reads a definition's member (no `number`, `forge`
  or `identity` member allowed, the number given); `configured`
  (`:1291`) factored so one forge's projection can be made alone; the
  seeds composed from the document's members (below). The readers'
  checks and refusals stay as they are for the document.
- `src/startup.rs`: where `settings::Engine` is built, its two new
  fields (`machine` as `Machine::none()`, below; `forge_seeds` from
  the document).
- Every `settings::Engine { .. }` literal in any crate or world: the
  two new fields (`machine: Machine::none()`, `forge_seeds:
  Box::new([])`), per `notes/scope.md`.
- Anything `notes/scope.md` lists as forced.

## Steps

1. **`settings.rs`:**

   ```rust
   /// The machine's facts a forge's definition is composed from
   /// (playground-reset.md, 3: the document keeps the machine's).
   #[derive(Debug)]
   pub struct Machine {
       /// `application.ports`, none until the document gives it.
       pub ports: Option<Ports>,
       /// The person an owned Forgejo makes: the effective user's login, `login@localhost`.
       pub person: Option<temper_forge_setup::setup::Account>,
       /// GitHub's API endpoint, resolved by the binary at start; none when unresolved.
       pub github: Option<crate::forge::ForgeRuntime>,
   }
   impl Machine { pub fn none() -> Self; }
   #[derive(Clone, Copy, PartialEq, Eq, Debug)]
   pub struct Ports { pub owned: u16, pub forge: u16, pub llm: u16 }
   /// A forge given at start under its number: its record's definition.
   #[derive(Clone, Debug)]
   pub struct ForgeSeed { pub number: u16, pub definition: Box<[u8]> }
   // Engine: pub machine: Machine, pub forge_seeds: Box<[ForgeSeed]>,
   ```

2. **`forge/stock.rs`** (cite `playground-reset.md, 3` and `setup.md,
   8.3` at module level): each returns a definition, the JSON object
   `{"forge": F, "identity": I, "setup": S}` of contracts 2, written
   with the crate's JSON writer (whatever `startup` or `temper-init`
   writes JSON with; if the service has none, a small byte writer
   escaping as `temper-init/src/template.rs` does), refused with
   `Err(why)` when a machine fact it needs is none:

   ```rust
   pub(crate) fn owned(machine: &Machine) -> Result<Box<[u8]>, Box<[u8]>>;      // template's forge 3, identity 3, setup 3
   pub(crate) fn github(machine: &Machine) -> Result<Box<[u8]>, Box<[u8]>>;     // forge 2, identity 1, setup null
   pub(crate) fn playground(machine: &Machine) -> Result<Box<[u8]>, Box<[u8]>>; // forge 1, identity 2, setup 1
   /// A definition read to the request's `NewForge` (its number 0 until the decision).
   pub(crate) fn new_forge(definition: Box<[u8]>, machine: &Machine) -> Result<forge::NewForge, Box<[u8]>>;
   ```

   The template's `$owned`, `$forge`, `$owned_git`, `$forge_git`,
   `$login`, `$email` come from `machine`; the owned setup entry has no
   `directory` (step 3 fills it); the playground's keeps
   `demo/forge` and `demo/bin/git` as the template has them.
3. **`forge/registry.rs`:**

   ```rust
   /// A forge built from its record's definition.
   #[derive(Debug)]
   pub(crate) struct Built {
       pub(crate) forge: settings::Forge,
       pub(crate) identity: settings::Identity,
       pub(crate) setup: Option<settings::Setup>,
       pub(crate) configured: forge::Configured,
   }
   #[derive(Debug)]
   pub(crate) struct Registry { /* Map<u16, Built>, capacity limits.forge.forges */ }
   impl Registry {
       pub(crate) fn new(limits: &crate::limits::Limits) -> Self;
       pub(crate) fn get(&self, forge: u16) -> Option<&Built>;
       pub(crate) fn numbers(&self) -> Box<[u16]>;
   }
   pub(crate) fn read(definition: &[u8], number: u16, machine: &Machine) -> Result<Built, Box<[u8]>>;
   pub(crate) fn setup_member(engine: &Engine, forge: u16) -> Option<Box<[u8]>>;
   pub(crate) fn restored(engine: &mut Engine);
   pub(crate) fn made(engine: &mut Engine, forge: u16);
   ```

   - `read`: parse the definition (the JSON reader `startup` reads the
     document with) and read `forge` and `identity` with
     `startup::application`'s readers in definition mode, the number
     given; `setup` (when not null) as one entry of
     `temper_forge_setup::setup::Setup::read`, after inserting
     `"forge": number` and, for an owned entry with no `directory`,
     `"directory": "forge-N"`. The forge is records-only (its access
     from the secrets store, as `settings::Engine.records_only` forges
     are): credential-file members in a definition are refused here
     (seeds carry them, through startup). Its API address: a socket
     address as given; GitHub's `null` from `machine.github`; a host
     name as `HELD_ADDRESS` (held, as an unresolvable host is today).
     `configured` as `startup::application::configured` makes one
     forge's.
   - `setup_member`: the domain's record's definition's `setup`,
     numbered and its directory filled as `read` does, written back as
     JSON; none for `null` or no record.
   - `restored`, run once from `engine.rs`'s iterate at the first pass
     where `engine.store.component.mark_durable()` holds, before
     `crate::forge::records::advance`: (a) for each
     `settings.forge_seeds` entry, queue `forge::Event::Seed { number,
     new }` (`new.configured` from the seeded forge's projection,
     `new.definition` the seed's) to be stepped in `input_pass`'s begin
     phase, before the credentials, as `engine.credentials` are; (b) for
     each held record (`engine.domain.forge_records()`) whose number is
     not a seed: `read`, `engine.domain.configure_forge`, then
     `install`. A record that does not read is reported on the
     diagnostics as a forge record that fails its checks and stays
     unconfigured (its calls refused as a forge not connected).
   - `made`: the made record read, then `install` (the decision
     configured the domain already).
   - `install(engine, built)`: in order `crate::forge::add`,
     `crate::workers::add`, `crate::setup::add`, `crate::clients::add`,
     each `pub(crate) fn add(engine: &mut crate::engine::Engine, built:
     &crate::forge::registry::Built)`, then the `Built` kept in the
     registry. In this task each `add` has an empty body, its doc
     comment saying what it adds once built and that a forge made at
     runtime has no connection (or worker identity, setup host entry,
     client row) yet.
4. **Seeds from the document** (`startup.rs`, `startup/application.rs`):
   each `application.forges` entry gives a `ForgeSeed`, its number the
   entry's, its definition composed from the entry, the identity that
   names it and the setup entry of its number, as the raw JSON members
   less `number`, `forge` and `identity` (and the forge's
   `identity` back-reference). These seeds go on being resolved into
   `forges`/`identities`/`setup` as today.
5. **`route.rs`:** `root::Request::ForgeMade { forge } =>
   crate::forge::registry::made(engine, forge)`.
6. Tests (Goal); `cargo fmt`; `heavy cargo clippy -p temper-service
   --all-targets -- -D warnings`; one at a time: `heavy cargo nextest
   run -p temper-service`, then `-p temper-one-process-world -p
   temper-real-loop-world -p temper-system-world -p temper-client-world`,
   then `heavy cargo check --tests -p temper`; `python3 -I -B
   scripts/old_forms.py` from `temper/`.
7. Commit (settings and stock, registry, seeds, tests) and push.

## Report

The standard report, plus: the JSON writer the stock uses; how the
readers' definition mode is selected; anything of the template a stock
definition could not carry; where `restored` runs in `iterate` and why
there.
