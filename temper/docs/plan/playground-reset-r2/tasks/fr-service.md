+++
id = "fr-service"
part = "forge-records"
kind = "feature"
needs = ["fr-domain", "fr-connections", "fr-workers", "fr-setup", "fr-clients", "fr-document"]
files = [
  "temper/crates/temper-service/src/family.rs",
  "temper/crates/temper-service/src/startup.rs",
  "temper/crates/temper-service/src/startup/**",
  "temper/crates/temper-service/src/settings.rs",
  "temper/crates/temper-service/src/limits.rs",
  "temper/crates/temper-service/src/limits/**",
  "temper/crates/temper-service/src/system.rs",
  "temper/crates/temper-service/src/forge/stock.rs",
  "temper/crates/temper-service/src/forge/registry.rs",
  "temper/crates/temper-service/src/tests/forge_records.rs",
  "temper/crates/temper-service/src/tests/**",
]
effort = "high"
minutes = 90
+++

## Goal

temper-service builds every forge from records at start and when one
is made, and reads a document that names no forge
(`temper/docs/design/draft/playground-reset.md, 3`): the welcome's
SetUp and Connect make a forge's record from the stock definitions;
a deployment with no forge starts and lists none; the playground's
fake forge is a seed while the playground stays today's demo; limits
are counts (`forge.forges`). Forges the document still names stay
seeds, a bridge `fr-retire` ends.

Acceptance, new tests in `temper-service` (`src/tests/forge_records.rs`):

- `a_deployment_without_forges_starts_and_lists_none`: a document with
  no `application.forges`, `identities` or `setup` starts; the forges'
  rows are empty.
- `set_up_makes_an_owned_record_and_asks_its_group`: a client's
  `SetUp` (wire, `cl-wire`'s shape) on that deployment under the box
  answers `setting_up` naming forge 1, the store holds forge 1's record
  (owned, its definition `stock::owned`'s), and the box link is asked
  `start 1`.
- `connect_github_makes_a_record`: `Connect { forge: None, how:
  github_manifest }` answers a link naming the new forge; its record is
  GitHub's.
- `a_kept_offered_forge_is_set_up_before_a_new_one`: with a seeded
  owned Forgejo not yet set up (the document's forge 3, as the template
  has it), `SetUp` goes on with it (forge 3, no new record); a kept
  GitHub not connected is gone on by `Connect { forge: None, github }`.
- `forgejo_admin_without_a_forge_is_refused`: `Connect { forge: None,
  how: forgejo_admin }` is answered as a forge not kept (no stock
  definition has an address).
- `a_restart_builds_made_forges_from_records`: a forge made, the
  service restarted over its store, the forge is in the rows again,
  configured, its connection added (fr-connections' path).
- `the_playground_seed_is_forge_one`: a document with `playground` and
  no fake seed seeds `stock::playground` as forge 1; today's
  `StartDemo` runs on it.
- `forges_limit_bounds_the_kept_forges`: with `forge.forges` = 1 and
  one kept, a SetUp is answered as `Full` maps (`busy`).
- Every existing `temper-service` test passes, those built on the
  document's forges unchanged (the bridge).

## Inputs

- `temper/docs/plan/playground-reset-r2/notes/common.md` and
  `temper/docs/plan/playground-reset-r2/notes/scope.md`.
- `temper/docs/plan/playground-reset-r2/notes/contracts.md`, sections 2,
  4 and 5.
- `temper/docs/design/draft/playground-reset.md`, sections 3 and 6.4.
- `temper/docs/design/protocol/configuration.md`, sections 3 and 7
  (forges, limits).
- `tasks/fr-ask.md`, `tasks/fr-domain.md`, `tasks/fr-registry.md`,
  `tasks/fr-document.md`, `tasks/cl-wire.md` (the wire's bridges this
  task replaces in `family.rs`).

## Owns and may change

Owns, after the tasks it needs (`fr-retire`, then D4's `rm-service`
and `pg-`, own these next): `src/family.rs`'s request translation,
`src/startup.rs`, `src/startup/**`, `src/settings.rs`, `src/limits.rs`,
`src/limits/**`, `src/system.rs`, `src/forge/stock.rs`,
`src/forge/registry.rs`, the service's tests and fixtures.

May change: every `temper-service` file a limit's rename or a count's
new source reaches (`forge.rs`, `setup/settings.rs`, `workers.rs`,
`engine.rs`: the expression only); the limit's name in any document
fixture or template that overrides `forge_component.forges` (none in
`temper-init`'s template today); anything `notes/scope.md` lists as
forced.

## Steps

1. **Forges optional:** `startup/application.rs` no longer refuses an
   empty or absent `application.forges` (`:812`), `identities` or
   `setup`; what joins them (the identity count equal to the forge
   count, `:815`; landing's and sign-in's references, `landing.rs`,
   `sign_in/forges.rs`) holds over what is given, none included. The
   forge list's cap by `forge.client.repositories` (`:788`) becomes
   `forge.forges`.
2. **SetUp and Connect make records** (`family.rs`, the translation
   `cl-wire` bridged and `fr-ask` numbered):
   - `SetUp`: the kept forge whose offer is `Offered` (mode owned,
     asked, not set up: the domain's `offer`), when there is one,
     `Target::Kept(n)` (a bridge while the document names one);
     otherwise `Target::New(stock::new_forge(stock::owned(machine)?,
     machine)?)`; a stock that cannot be made (no `ports`) is
     `Target::Kept(0)`, refused as a forge not kept.
   - `Connect { forge: Some(n) }`: `Target::Kept(n)`.
   - `Connect { forge: None }`: for `github_app`/`github_manifest`, the
     kept GitHub not connected when there is one (`playground-reset.md,
     6.4`'s Go on), else `Target::New(stock::github)`; for
     `forgejo_admin`, the kept managed or connected Forgejo not
     connected when there is one, else `Target::Kept(0)`.
   Remove cl-wire's bridge comments these replace.
3. **The playground's forge** (the bridge until `pg-`): at startup,
   when `application.playground` is given and no seed's setup entry is
   fake, `stock::playground(machine)` joins the seeds as forge 1 (a
   seed of 1 already given: refused, naming both), resolved as seeds
   are, and today's demo names it.
4. **Limits as counts** (`limits.rs`, `limits/names.rs`): `forge.forges`
   (fr-families' domain limit) is the forges kept at most and sizes the
   forge connection component (`forge_component.forges` goes; its
   readers: `engine.rs:137` `connection_limits`, `setup/settings.rs:15,86`,
   `forge.rs:167`, the installations' product check `limits.rs:101`),
   the registry, the GitHub token map (`forge/tokens.rs:37`), the
   workers' credential bound and the startup record-check worklist
   (`startup/checks.rs:1023`), and the notes' cohort (`system.rs:1097`),
   each today sized by the configured forges' count. Startup refuses
   more seeds than `forge.forges`.
5. Tests (Goal); `cargo fmt`; `heavy cargo clippy -p temper-service
   --all-targets -- -D warnings`; one at a time: `heavy cargo nextest
   run -p temper-service`, then `-p temper` (default profile), then `-p
   temper-one-process-world -p temper-real-loop-world -p
   temper-system-world -p temper-client-world -p
   temper-engine-domain-world`; `python3 -I -B scripts/old_forms.py`
   from `temper/`.
6. Commit (optional forges, translation, playground seed, limits,
   tests) and push.

## Report

The standard report, plus: each reader of `forge_component.forges`
moved; how SetUp picks a kept forge over a new one; what a stock
definition lacked at runtime.
