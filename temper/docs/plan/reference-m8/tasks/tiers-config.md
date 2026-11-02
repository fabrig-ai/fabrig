+++
id = "tiers-config"
part = "tiers-config"
kind = "feature"
needs = ["tiers-policy"]
files = ["temper/crates/temper-service/src/startup/application/landing.rs", "temper/crates/temper-service/src/startup/checks.rs", "temper/crates/temper-service/src/limits.rs", "temper/crates/temper-service/src/limits/names.rs", "temper/crates/temper-engine-domain/src/translate.rs", "temper/crates/temper-engine-domain-forge/src/limits.rs", "temper/crates/temper-engine-domain-forge/Cargo.toml", "temper/crates/temper-engine-domain/Cargo.toml", "temper/crates/temper-service/Cargo.toml", "temper/crates/temper-engine-domain/src/lib.rs", "temper/tests/engine/forge/Cargo.toml"]
effort = "high"
minutes = 150
+++

# Tiers and their project settings, read from the configuration

## Goal

A project's repositories name their tiers in the configuration's `landing` member, each with
the branch it follows, and the project its `tier_retries` and `verification_bound_ms`; the
service reads and checks them, and they reach the forge connector, which keeps them per
project beside the gates. The forge gains the tiers' limits. Nothing runs a tier yet.

Acceptance:

- `temper/docs/design/protocol/configuration.md`, section 3 holds for tiers: a `gates` entry's
  `tiers`, each `{"name", "workflow", "ignored", "artifact", "format", "branch"}` (`branch`
  optional), and the project's `tier_retries` (u8, 2 when absent) and `verification_bound_ms`
  (u64, 7,200,000 when absent) beside `by`. Refused, naming the member, as the gates' refusals
  do: a tier name not a gate name's form or past 32 bytes, a name a gate or another tier of the
  repository already has, an empty `artifact` or one with `/` or past a gate name's bytes, a
  `format` other than `junit` or `lines`, an empty `branch`, more tiers than
  `forge.tiers.tiers`.
- The typed forms of `notes/contracts.md`, 3 exist: `TierCheck`, `RepositoryChecks::tiers`,
  `AfterLandingCheck`, `ProjectLanding::after` in `temper-engine-domain`'s `translate.rs`;
  `Tier`, `RepositoryGates::tiers`, `AfterLanding` in the forge's `boundary.rs`;
  `Domain::project_landing(project, way, gates, after)` keeping them per project, and
  `Domain::after_landing(project)` reading them back; `Limits::tiers: TierLimits` with the small
  profile's 2, 16, 256 and the three limit names, checked by `Limits::valid` (non-zero,
  `test_id_bytes` at least 1, `failing_tests` at most what one store row can hold, as the old
  code's bound says or a value you measure).
- `temper/crates/temper-service/src/tests/startup.rs` gains stories beside
  `landing_configuration_reads_the_way_and_bounded_repository_gates`: tiers and settings read
  whole, defaults when absent, and each refusal above; the forge's lib tests show
  `project_landing` keeping a repository's tiers and its project's settings.
- Packages pass: `heavy cargo nextest run -p temper-service --lib`; `-p temper-engine-domain-forge
  --lib`; `-p temper-engine-domain --lib`; and the readers below build and pass their focused
  tests (`-p temper-engine-domain-world --test suite -E 'test(/forge_routes/)'`,
  `-p temper-engine-forge-world --test suite`).

## Inputs

- `temper/docs/plan/reference-m8/notes/common.md`; `notes/contracts.md`, 1 and 3.
- `temper/docs/design/protocol/configuration.md`, section 3 (the `landing` member; the
  limits' naming, section 7: a duration's last segment `_ms`); `temper/docs/design/domain/landing.md`,
  13.1; `temper/docs/design/domain/after-landing.md`, sections 2, 10 (the limits) and 11.
- Today's path, by a survey: `startup/application.rs` calls `landing::parse`
  (`startup/application/landing.rs`: `parse`, `check`, `repository_checks`, `project_landing`;
  `tiers` and the settings are refused today as unknown members by `read::finish`) →
  `settings::Engine.landing` → `startup/checks.rs` `landing_config` → the engine's
  `Config.landing` (`temper-engine-domain/src/domain.rs`, about 396-401 and 1304-1308) →
  `translate::project_landing` → the forge's `Domain::project_landing`
  (`temper-engine-domain-forge/src/landing.rs`, about 113, with `landing.projects` at 22).
- The old code: `temper/docs/plan/reference-m8/notes/old/temper-2a-service.diff`
  (`startup/application/landing.rs`: tiers read with `check()` plus `artifact` and `format`,
  names unique across gates and tiers; it has no `branch`, used `verification_bound_ms` and
  `pause_on_red`, and checked forges this tree no longer configures).

## Owns and may change

- Owns the files in the header, and in the forge crate: `boundary.rs` (the new types and
  `RepositoryGates::tiers`), `landing.rs` (`project_landing`, its per-project storage,
  `after_landing`) and `lib.rs` (exports). `temper-engine-domain/src/domain.rs`: passing
  `after` through, only.
- Root `Cargo.toml` is not touched: the forge, engine and service crates, and the forge world
  (`temper/tests/engine/forge/Cargo.toml`), gain `temper-engine-domain-forge-tiers.workspace = true`
  in their own `Cargo.toml`; `Cargo.lock` as cargo writes it.
- `temper-engine-domain/src/lib.rs`: `TierCheck` and `AfterLandingCheck` join the `translate`
  export, only.
- Readers, each only the new field in a literal (`tiers: Box::new([])`, the default settings) or
  the new argument: the forge's `Limits` literal in `temper-service/src/limits.rs`,
  `temper-engine-domain-forge/src/tests.rs`, `temper-engine-domain/src/tests/resume_fixture.rs`,
  `temper-fake/src/tests/check.rs`, `temper/tests/engine/forge/src/world.rs`,
  `temper-protocol-store/src/tests/fixtures.rs`, `temper/tests/codecs/tests/support/store.rs`
  (or wherever the compiler finds them); `ProjectLanding`/`RepositoryChecks`/`RepositoryGates`
  literals in `temper-service/src/tests/{startup.rs,root_checks.rs,fixtures.rs,welcome_policy.rs}`,
  `temper-service/src/tests/fixtures/startup.rs`, `temper-engine-domain/src/route/policy_translate.rs`,
  the forge's `src/tests.rs`, `src/tests/{views.rs,landing.rs}`,
  `temper/tests/engine/domain/src/{forge_routes.rs,oracle/config.rs}`,
  `temper/tests/engine/forge/src/{landings.rs,world.rs}`, `temper/tests/engine/forge/tests/suite/landing.rs`,
  `temper/tests/system/src/forge.rs`, `temper/tests/conformance/src/lib.rs`.
- Not this task's: `change-verifying`'s files (the change crate, the store's schema and change
  records); the forge's `domain.rs` facts constant (`tiers-forge` replaces it).

Pre-approved options:

- If a figure under `temper/crates/temper/tests/figures/` or a service limits test pins a number
  the new limits move, regenerate or update it as its test says, and list it.
- If the owned example (`temper/docs/examples/owned.json`) or `temper-init`'s template is read
  by a test that refuses the new members' absence, leave the documents unchanged and report it
  (they keep `projects` without tiers).

## Steps

1. The typed forms and the forge's storage and limits (contracts.md, 3).
2. The service's parse and checks, then the stories.
3. `cargo fmt`; clippy over `temper-service`, `temper-engine-domain`, `temper-engine-domain-forge`
   and every reader package you touched; the tests in Goal, each count in the report.
4. Commit (why: tiers are configured per repository, following a named branch); push.

## Report

As the implementer's text says, and also: the refusals' words; the readers touched; the
`failing_tests` bound you set and why.

## Answers

2026-11-02. May the task also change `temper-service/Cargo.toml` (the tiers crate as a
dependency) and `temper-engine-domain/src/lib.rs` (export `TierCheck`, `AfterLandingCheck`)?
Yes, both, and nothing more: the service names `temper_engine_domain_forge_tiers::Format`
directly through its own dependency, as the old code did, with no re-export of `Format`
elsewhere; the engine's `pub use translate::{...}` gains `TierCheck` and `AfterLandingCheck`,
the spec's own types, so no public item beyond the spec. Both files are now in the header and
under Owns.

2026-11-02. May `temper/tests/engine/forge/Cargo.toml` gain `temper-engine-domain-forge-tiers`
so the forge world can build the `AfterLanding` argument? Yes, that one line and nothing more
in the manifest: the world is a reader of the forge's boundary, and `AfterLanding::verdict` is
the tiers crate's `Settings` as `notes/contracts.md`, 3 has it, so no re-export or helper is
added to the forge for it. In `world.rs` the world's own `project_landing` keeps its
signature; its two calls (about 140 and 531) pass the default settings, `AfterLanding {
verdict: Settings { retries: 2, suspects: Suspects::Bisect }, bound: 7,200,000 ms }`, the
configuration's defaults, so its callers do not change. The manifest is now in the header and
under Owns.
