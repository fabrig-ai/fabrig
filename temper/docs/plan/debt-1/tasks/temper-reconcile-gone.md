+++
id = "temper-reconcile-gone"
part = "temper-reconcile"
kind = "migration"
needs = []
files = [
  "temper/crates/temper/src/forge.rs",
  "temper/crates/temper/src/forge/tests.rs",
  "temper/crates/temper/src/commands.rs",
  "temper/crates/temper/src/tests.rs",
  "temper/crates/temper/src/tests/commands.rs",
  "temper/crates/temper/tests/support/owned.rs",
  "temper/crates/temper/tests/suite/forge.rs",
  "temper/crates/temper/tests/suite/startup.rs",
  "temper/crates/temper/tests/suite/fake.rs",
  "temper/crates/temper-forge-setup-shell/**",
  "temper/crates/temper-forge-setup/src/limits.rs",
  "temper/crates/temper-forge-setup/src/given.rs",
  "temper/crates/temper-forge-setup/src/setup.rs",
  "temper/crates/temper-forge-setup/src/machine.rs",
  "temper/crates/temper-forge-setup/src/tests/**",
  "temper/crates/temper-service/src/limits.rs",
  "temper/crates/temper-service/src/limits/names.rs",
  "temper/crates/temper-service/src/setup/settings.rs",
  "temper/crates/temper-service/src/tests/**",
  "temper/docs/help/reference/commands.md",
  "temper/docs/help/reference/limits.md",
  "temper/docs/design/protocol/README.status.md",
  "temper/docs/design/protocol/setup.status.md",
  "temper/docs/design/protocol/configuration.status.md",
  "temper/docs/development/debt/D006-forge-reconcile-dead.md",
  "temper/docs/development/debt/README.md",
  "Cargo.lock",
]
effort = "high"
minutes = 120
+++

# temper D006: `temper forge reconcile`, its shell host and `setup.orgs` removed

## Goal

Remove what only served the managed and connected forms that are gone
(`temper/docs/design/protocol/setup.md`, 5 and 6.5): the `temper forge reconcile` verb, which
only refuses; `temper-forge-setup-shell`'s `reconcile` module (524 lines with no product
caller) and the `exchange` module that serves it alone; and the `setup.orgs` limit with its
`orgs` places. The engine's setup host keeps its own `SetupPlan::Reconcile` through skein's
machine: that is live and untouched. Behaviour is otherwise unchanged. This pays
`temper/docs/development/debt/D006-forge-reconcile-dead.md`.

Acceptance:

- `temper forge reconcile 1` is refused with the forge verbs' usage line, as any unknown
  verb is; `temper --help` and `docs/help/reference/commands.md` show no `reconcile` row.
- `temper-forge-setup-shell/src/` has no `reconcile.rs` and no `exchange.rs`.
- `rg -n 'setup\.orgs|Place::Orgs\b|Place::Org\(|OrgsLimit' temper -g '!**/docs/plan/**' -g '!**/docs/design/draft/**'`
  finds only the arms that map skein's `Place` variants (below), and `rg -n -w orgs
  temper/crates -g '*.rs'` finds only skein's `Given.orgs` and `Limits.orgs` that temper
  fills, and Forgejo API paths (`/api/v1/orgs/`).
- The three status entries D006 names are gone; D006's file and its README row are gone;
  `tools/citations/citations.py --all` reads 0; the tests in Steps pass, each count above
  zero; clippy is clean.

## Inputs

- `temper/docs/plan/debt-1/notes/common.md`.
- `temper/AGENTS.md` (its `docs/help/` is compiled into its help tests).
- `temper/docs/development/debt/D006-forge-reconcile-dead.md`, whole.
- `temper/docs/design/protocol/setup.md`, 5.3: its given values' table, whose limits row this
  cut rewrote: no `setup.orgs`; temper gives no org, and gives skein's orgs bound as 1, the
  least the kit takes (`skein_forge_setup::limits::check` refuses 0). 5 and 6.5 for what the
  verb served; 11 for the crates.
- `temper/docs/design/protocol/README.md`, 4 (the binary's commands) and
  `temper/docs/design/protocol/configuration.md`, 7 (the limits, which name no `setup.orgs`).

## Owns and may change

**The verb** (`temper/crates/temper`):

- `src/forge.rs`: the `reconcile` parse arm, its refusal branch and its form in the usage
  line go. `src/forge/tests.rs` holds only the verb's test: delete it and its `mod tests;`.
- `src/commands.rs`: the `forge reconcile` row goes (`COMMANDS` takes 20 rows). The `Row`
  field `transition` goes too, with its two renderings (" (a transition)" and "A transition:
  it goes once the forms it serves do."): this row was its only `true`, and its doc names the
  verb. `src/tests/commands.rs` drops the verb's row and the `transition` check (~136);
  `src/tests.rs` drops `["forge", "reconcile", "1"]`.
- `tests/support/owned.rs`: `reconcile()` and `refused`'s `reconcile` arm go;
  `tests/suite/forge.rs`'s `reconcile_refuses_a_forge_the_setup_has_no_entry_for` and
  `tests/suite/fake.rs`'s `the_prepared_fake_directs_reconcile_to_the_service_and_keeps_its_token`
  go; `tests/suite/startup.rs`'s `every_command_checks_setup_before_reading_files` loses
  `"reconcile"` from its verbs and keeps the rest.
- `temper/docs/help/reference/commands.md`: written by
  `SKEIN_CODEGEN_WRITE=1 heavy cargo nextest run -p temper --lib -E 'test(=tests::commands::the_page_commands_is_what_the_table_renders)'`
  (check the test's path with `--list` first), never by hand.

**The shell's host** (`temper-forge-setup-shell`, owned whole):

- `src/reconcile.rs` and `src/exchange.rs` (which uses `reconcile::Settings` alone) go, with
  their `pub mod` lines; `tests/suite/reconcile.rs` and `tests/suite/exchange.rs` go with
  their `mod` lines; `tests/support/mod.rs` loses what only they used (`settings`, the
  reconcile runner, their fakes) and keeps what the other suites use.
- A dependency in its `Cargo.toml` that only the removed modules and tests used goes, and
  `Cargo.lock` follows (`cargo metadata` or the build updates it). Find them by `rg` of each
  dependency's crate name over `src/` and `tests/`.

**`setup.orgs` and the `orgs` places**:

- `temper-forge-setup/src/limits.rs`: `Host.orgs` goes; `setup.orgs` leaves `NAMES` (4 rows)
  and `HOST_NAMES` (7 rows) and `apply`; `small()` and `Host::plan()` give skein's
  `Limits.orgs` one constant, `pub const ORGS: u32 = 1`, documented "temper gives no org
  (protocol/setup.md, 5.3): the least bound the kit takes". The module doc stops naming
  managed orgs.
- `temper-forge-setup/src/setup.rs`: `Entry.orgs` goes (nothing outside the removed shell
  module reads it); `machine.rs` (~216) and every literal drop the field. skein's
  `Given.orgs` stays, which `given::values` fills with `Box::new([])` as today.
- `temper-forge-setup/src/given.rs`: `Place::Orgs`, `Place::Org(_)` and `Place::OrgsLimit`
  join the `unreachable!` arm of the places temper never gives, in `refusal` and in its
  `other` match, its reason saying temper gives no org and a fixed bound.
- `temper-service/src/limits/names.rs`: the `setup.orgs` row (`HOST_NAMES[4]`) goes and the
  rows after it index `HOST_NAMES` one lower (check every `HOST_NAMES[`/`NAMES[` index in
  temper by `rg`). `temper-service/src/limits.rs`: `host_setup` drops `setup.orgs`, and its
  doc stops naming the old shell's overrides.
- `temper-service/src/setup/settings.rs` (`check`, ~146-185): `pure.orgs == 0` goes; the arm
  that names `b"setup.orgs"` names each place as `temper-forge-setup/src/given.rs`'s
  `refusal` does (`forge.client.repositories` for the repository places), and the places
  `skein_forge_setup::limits::check` never returns (it checks limits alone) join its
  `unreachable!` arm.
- `temper/docs/help/reference/limits.md`: written by
  `SKEIN_CODEGEN_WRITE=1 heavy cargo nextest run -p temper --lib -E 'test(/the_pages_limits_are_what_the_tables_render/)'`.
- Tests, each edited only for these items: `temper-forge-setup/src/tests/limits.rs` (the two
  `setup.orgs` rows go; the unknown name `setup.orgs.extra` becomes `setup.deadline_ms.extra`),
  `tests/given.rs` (the three orgs places' rows go; skein `Limits` literals take `ORGS`),
  `tests/fixtures.rs` and `tests/machine.rs` (`Entry.orgs`); `temper-service/src/tests/`
  (`setup.rs` ~635 drops `setup.orgs` from its names; `startup.rs` ~724-731 uses another
  override of the same family that stays, `setup.backoff_ms` with a value its check takes,
  keeping what the test asserts; `setup_create.rs`, `setup_admin.rs`, `setup_adoption.rs`,
  `sign_in_configuration.rs`, `fixtures.rs` drop `Entry.orgs` lines; assertions on skein's
  `given.orgs` being empty stay).

**Status and debt text**:

- `temper/docs/design/protocol/README.status.md`: delete "4: the binary keeps `temper forge
  reconcile`, which always refuses, because the managed and connected forms it served are
  gone; a code task removes it." and keep the entry's other 4 and 15.5/15.6 words.
- `temper/docs/design/protocol/setup.status.md`: delete "3, 5, 6.5: `temper forge reconcile`
  stays and always refuses, and the `setup.orgs` limit stays, though no managed or connected
  form uses them (`temper/src/forge.rs`, `temper-forge-setup/src/limits.rs`); a code task
  removes them."
- `temper/docs/design/protocol/configuration.status.md`: delete "7: the `setup.orgs` limit
  stays, though no setup entry names orgs (`temper-forge-setup/src/limits.rs`); a code task
  removes it."
- Each of the three dated today. `temper/docs/design/STATUS.md` names neither: leave it.
- `temper/docs/development/debt/D006-forge-reconcile-dead.md` (delete) and its row in
  `temper/docs/development/debt/README.md`; D005 and D008 name D006 by ID only: leave them.

Pre-approved:

- skein's `skein_forge_setup::given::Place` keeps its orgs variants and its `Limits` its
  `orgs` field: whether skein drops them is skein's. Change no skein file.
- If a removed module leaves another module of the shell with no caller but its own tests,
  stop and ask before removing it: the item names `reconcile` alone.
- If a test outside this list pins `reconcile`'s verb, its help row or `setup.orgs` (a golden,
  a help test of `temper-docs`, a page-size figure), update it the same way, by its write
  mode where it has one, and list it.
- The draft `temper/docs/design/draft/agent-docs.md` mentions the verb: drafts stay
  untouched.

## Steps

1. The verb; then the shell's modules; then `setup.orgs`, the places and `Entry.orgs`; then
   the two help pages by their tests.
2. Narrow tests, one at a time, each count above zero:
   - `heavy cargo nextest run -p temper --lib`
   - `heavy cargo nextest run -p temper --test suite -E 'test(/^(forge|startup|fake)::/)'`
     (the default profile may leave some out; say which ran)
   - `heavy cargo nextest run -p temper-forge-setup-shell`
   - `heavy cargo nextest run -p temper-forge-setup --lib`
   - `heavy cargo nextest run -p temper-service --lib -E 'test(/setup|startup|limits/)'`
   - `heavy cargo nextest run -p temper-docs --lib`
   - `heavy cargo check -p temper-box --tests` and `heavy cargo check -p temper-fake --tests`,
     the other direct dependents of `temper-forge-setup` (`cargo tree -i temper-forge-setup
     -e normal,dev --depth 1` lists them).
3. `cargo fmt`; `python3 -I -B scripts/old_forms.py` from `temper/` (with `--write` if a
   count fell); `heavy cargo clippy -p temper -p temper-forge-setup-shell -p temper-forge-setup -p temper-service -p temper-docs --all-targets -- -D warnings`.
4. Status and debt text as above; `python3 -I -B tools/citations/citations.py --base
   origin/main` and `--all` from the repository root; the acceptance's `rg`.
5. Commits: the verb; the shell's host; `setup.orgs`; the status and debt text. Pushed.

## Report

As the implementer's text says, and also: each removed file and its lines; each dependency
removed; what the acceptance's `rg` leave, by file; any test beyond the list.
