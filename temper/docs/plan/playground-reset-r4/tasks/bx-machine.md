+++
id = "bx-machine"
part = "box-groups"
kind = "interface"
needs = []
files = [
  "temper/crates/temper-forge-setup/src/setup.rs",
  "temper/crates/temper-forge-setup/src/tests.rs",
  "temper/crates/temper-forge-setup/src/tests/template.rs",
  "temper/crates/temper-fake/src/demo.rs",
  "temper/crates/temper-fake/src/tests/demo.rs",
  "temper/crates/temper-service/src/startup/application/demo.rs",
  "temper/crates/temper-service/src/startup/application/owned.rs",
  "temper/crates/temper-service/src/tests/machine_members.rs",
  "temper/crates/temper/src/setup.rs",
  "temper/crates/temper/src/setup/tests.rs",
]
effort = "high"
minutes = 50
+++

## Goal

The document's two machine members (contracts, section 6) are read, checked
and typed, beside the forges the document still names (D1):
`application.owned`, the box's owned Forgejo, and `application.demo.fake`,
the playground's fake forge, each a setup entry less `forge` and `start`.
Nothing acts on them yet: `SetUp` (`fr-make-forge`), the playground's start
(the playground part) and the box's kinds (`bx-wire`) read them later.
`temper init`'s template is not changed here (`bx-init` does it, once a
first run can make its forges).

Acceptance:

- `temper_forge_setup::setup::Template` exists with `read` and `entry`
  (below); for an owned template, `entry(N)` equals what `Setup::read`
  gives for the same members with `"forge": N, "start": "asked"`; for a
  fake one, with `"forge": N` (paths resolved alike).
- `application.owned` is optional (absent or `null`: none); present, it is
  read by temper-service into `settings::Engine::owned` and by the binary's
  setup check into its `Deployment::owned`; a mode other than `owned`, or a
  `forge` or `start` member, is refused at its path.
- `application.demo.fake` is optional beside `application.demo.forge`, which
  stays required while the demo stands; present, it is read into
  `settings::Demo::fake` (temper-service) and `temper_fake::demo::Demo::fake`
  (the box's view); a mode other than `fake` is refused at its path.
- Every document that passes today passes unchanged and reads both as none:
  the template, `temper/docs/examples/*.json` and every fixture.
- The tests below pass.

## Inputs

- `temper/docs/plan/playground-reset-r4/notes/common.md` and
  `temper/docs/plan/playground-reset-r4/notes/contracts.md` (sections 1, D1,
  and 6 are this task's shape).
- `temper/docs/design/protocol/setup.md`, section 8 (the setup entry's
  members) and 8.4 (`application.demo`).
- `temper/docs/design/protocol/configuration.md`, the section on the
  `application` members (find `application.setup`).
- `temper/docs/plan/playground-reset-r4/notes/surveys/s1-forges-document.md`,
  "Parsing and typed representations" and "Setup entries".
- The code: `temper/crates/temper-forge-setup/src/setup.rs` (`entry`,
  `mode`, `Setup::read`'s path resolution), `temper/crates/temper-fake/src/demo.rs`
  (`read`, `unique_listen`), `temper/crates/temper-service/src/startup/application.rs`
  (`parse`, `Members`), `temper/crates/temper-service/src/startup/application/demo.rs`,
  `temper/crates/temper/src/setup.rs` (`check_with`, `Deployment`).

## Owns and may change

- Owns the files in the header. `owned.rs` and the three test files are new.
- Neighbours, each for this task's lines only:
  - `temper/crates/temper-service/src/startup/application.rs`: `mod owned;`;
    in `parse`, the `owned` member taken (`read::nullable`, as `sign_in` is)
    and given to `owned::parse`; `Members`'s new field `owned`. Nothing else.
  - `temper/crates/temper-service/src/settings.rs`: `Engine`'s new field
    `owned: Option<temper_forge_setup::setup::Template>` and `Demo`'s new
    field `fake: Option<temper_forge_setup::setup::Template>`, with their doc
    lines. `temper/crates/temper-service/src/startup.rs`: the `Engine`
    literal's new field. `temper/crates/temper-service/src/tests.rs`: the
    `mod machine_members;` line.
  - Every literal of `settings::Engine`, `settings::Demo`, `Members` and
    `temper_fake::demo::Demo` in any temper file (tests, worlds,
    `temper/tests/**`): the new field, `None`. These are compiler-forced.
- Not changed: `temper/crates/temper-init/template/temper.json`,
  `temper/docs/examples/`, `engine.acts`, anything that reads a forge.
- Options:
  - `Template::read`'s problems: jig's `config::Problems` (default, as
    `Setup::read`'s inner readers use it; the service maps each to
    `What::Jig`, the binary prints them through `problem::lines` after
    `append`). If the service needs its own `What` for a refusal's words,
    use `What::Jig(config::What::Form)` for a wrong mode or a refused member.
  - Listener uniqueness: `owned`'s `listen` and `fake`'s `listen` differ from
    each other and from the clients', webhooks' and demo LLM's listeners,
    refused as `must be unique` at the `listen` path; each may equal the
    `listen` of a setup entry of its own kind (a seeded forge on the
    machine's port, D1). Check it where the other listeners are at hand:
    the binary's `check_with` (default; it has them all) and temper-fake's
    `read` for `fake`. If temper-service also needs the check to refuse at
    serve's start, add it in `owned.rs`/`demo.rs` with the clients' and
    webhooks' listeners passed in, and say so.

## Steps

1. **`Template`** (`temper-forge-setup/src/setup.rs`), a name this task
   creates:

   ```rust
   /// A setup entry less `forge` and `start` (contracts, 6): the box's owned
   /// Forgejo, `application.owned`, or the playground's fake forge,
   /// `application.demo.fake`. Paths are resolved as `Setup::read` resolves
   /// an entry's.
   #[derive(Clone, PartialEq, Eq, Debug)]
   pub struct Template {
       pub directory: Box<[u8]>,
       pub bot: Account,
       pub orgs: Box<[Box<[u8]>]>,
       pub mode: Mode,
   }
   /// The mode a template must have.
   #[derive(Clone, Copy, PartialEq, Eq, Debug)]
   pub enum Want { Owned, Fake }
   impl Template {
       /// Reads the object at `at`, its problems rooted at `path`; refuses a
       /// `forge` or `start` member and a mode other than `want`.
       pub fn read(
           d: &Document,
           at: u32,
           want: Want,
           limits: &Limits,
           directories: &config::Directories,
           path: &Path,
           problems: &mut config::Problems,
       ) -> Option<Template>;
       /// Forge `forge`'s entry: `asked` for an owned Forgejo (contracts, 3).
       #[must_use]
       pub fn entry(&self, forge: u16) -> Entry;
   }
   ```

   Share the entry reader: split `entry`'s member reading so that both read
   `directory`, `bot`, `orgs` and `mode` by one path, and `Setup::read`'s
   per-entry resolution into a function both call. Tests,
   `src/tests/template.rs`:
   - `an_owned_template_is_its_entry_less_forge_and_start`: the template's
     forge-3 entry of `temper/crates/temper-init/template/temper.json`, as
     `application.owned` less `forge` and `start`, read; `entry(7)` equals
     `Setup::read`'s entry of the same object with `"forge": 7,
     "start": "asked"`, binaries resolved against the cache and the directory
     against the data directory.
   - `a_fake_template_is_its_entry_less_forge`: the template's forge-1 entry,
     likewise, `entry(9)` with `asked` false.
   - `a_template_refuses_forge_start_and_the_other_mode`: each at its path.
2. **The service** (`startup/application/owned.rs`, `demo.rs`):
   `pub(super) fn parse(document, at: Option<u32>, limits, directories,
   problems) -> Option<Template>` for `owned` (absent or `null`: `None`);
   in `demo.rs`, the optional `fake` member read with `Want::Fake` into
   `settings::Demo::fake`. `application.rs` gives `owned` to `Members` and
   `startup.rs` to `settings::Engine::owned`. Tests,
   `src/tests/machine_members.rs`, on the service's startup fixtures
   (`src/tests/fixtures/startup.rs`'s documents): `the_machine_members_reach_the_engines_settings`
   (a document with both: `settings::Engine::owned` and
   `settings::Demo::fake` are `Some`, their `entry(N)` as step 1's);
   `a_document_without_them_reads_none`; `a_wrong_mode_is_refused_at_its_path`.
3. **The binary and the box's view** (`temper/src/setup.rs`,
   `temper-fake/src/demo.rs`): `Deployment` gains `pub owned:
   Option<temper_forge_setup::setup::Template>`, read in `check_with` from
   `application.owned` with the setup's checked limits; `temper_fake::demo::Demo`
   gains `fake: Option<Template>` and `pub fn fake(&self) -> Option<&Template>`,
   read in `read` beside `forge`. The listeners' rule (Options). Tests:
   `temper-fake/src/tests/demo.rs`, `a_demo_reads_its_fake_beside_its_forge`
   and `a_demo_fake_on_another_ones_port_is_refused`; `temper/src/setup/tests.rs`
   (`#[cfg(test)] mod tests;` in `setup.rs`), `the_setup_check_reads_the_owned_member`
   and `the_init_template_reads_no_machine_member`.
4. `cargo fmt`; `heavy cargo clippy -p temper-forge-setup -p temper-fake
   -p temper-service -p temper --all-targets -- -D warnings`; focused tests
   of those four (`heavy cargo nextest run -p temper-forge-setup --lib`,
   `-p temper-fake --lib`, `-p temper-service --lib`, `-p temper --lib`);
   direct dependents build: `heavy cargo check -p temper-forge-setup-shell
   -p temper-box --tests` and every package whose literal you touched
   (`--tests`). Commit, push.

## Report

Fab's sections; under `## Interfaces`: `setup::Template`, `setup::Want`,
`Template::read`, `Template::entry`, `settings::Engine::owned`,
`settings::Demo::fake`, `Deployment::owned`, `temper_fake::demo::Demo::fake`
with their signatures, and where the listeners' rule is checked.
