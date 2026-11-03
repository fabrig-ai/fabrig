+++
id = "kinds-document"
part = "kinds-root"
kind = "migration"
needs = ["kinds-switch-charters", "kinds-ctl-reads", "reference-charters"]
files = ["temper/crates/temper-service/src/startup.rs", "temper/crates/temper-service/src/startup/**", "temper/crates/temper-service/src/stock.rs", "temper/crates/temper-service/src/system.rs", "temper/crates/temper-service/src/tests/**", "temper/crates/temper-engine-domain/src/translate.rs", "temper/crates/temper-engine-domain/src/lib.rs", "temper/crates/temper-engine-domain/src/domain.rs", "temper/crates/temper-engine-domain/src/route/policy_translate.rs", "temper/crates/temper-engine-domain/src/tests/**", "temper/crates/temper-init/template/temper.json", "temper/crates/temper-init/src/**", "temper/docs/examples/owned.json", "temper/docs/examples/reference/landing.json", "temper/docs/examples/reference/settings.json", "temper/crates/temper/tests/support/**", "temper/crates/temper/tests/fixtures/cluster/**", "temper/crates/temper/src/tests/help.rs", "temper/crates/temper/src/fake/tests.rs", "temper/crates/temper/tests/suite/fake.rs", "temper/crates/temper/src/setup.rs", "temper/crates/temper/src/lib.rs", "temper/crates/temper-fake/src/**", "temper/tests/one-process/src/**", "temper/tests/real-loop/src/**", "temper/tests/client-browser/src/configuration.rs", "temper/tests/engine/domain/src/**", "temper/tests/engine/domain/tests/suite/**", "temper/crates/temper/src/tests/reference_settings.rs", "temper/docs/help/landing-rule.md", "temper/docs/help/charter.md", "temper/docs/help/policy.md", "temper/docs/help/repository.md", "temper/docs/design/protocol/configuration.status.md", "temper/docs/design/domain/root.status.md"]
effort = "high"
minutes = 150
+++

# kinds-document: `application.landing`'s rules and `charter` leave the document

## Goal

13.6's second step for the document (`configuration.md`, 13.5 and section 3, as this cut wrote
them): temper reads no landing rule and no `charter` from its document, and refuses them
saying where they went; the binary gives the forge 13.4's stock (kind 1 rule 1, kind 3's
charters 2 and 4); a test world gives its own, as it gives jig's kinds; every writer of the
members moves (13.5, **Their writers name none of them**). `application.landing` keeps its
projects' ways, checks and tiers (kinds 4 and 5 are not M3's).

Acceptance:

- temper-service's document reading (`startup/application/landing.rs` and where `charter` is
  read): `application.landing.deployment`, any `application.landing.projects[N].rules` and
  `application.charter` are refused as unknown members are, each naming its path and where it
  went: "`application.landing.deployment`: landing rules live in the store; read them with
  `temper ctl settings show landing-rule`", a project's rules likewise with `--project`, and
  "`application.charter`: the forge's charters live in the store; read them with `temper ctl
  settings show forge-charter 1`". A project entry keeps `project`, `by`, `gates` and the
  turn's and the tiers' settings; the checks that read a rule's approvals' roles go.
- The bridge goes: `Config.landing_rules`, `stock::forge_defaults` and the root's derived
  stock. temper-service's `stock::forge(limits) -> Box<[forge::settings::Stock]>` gives kind 1
  rule 1 (`forge` open "", enforced, CI passed, up to date, gate 1 blocking at a clean head, no
  approvals; 13.4's row) and kind 3 number 1 charter 2, number 2 charter 4, sized by
  `kinds::size`. `stock::Given::Own` carries a world's own forge stock beside jig's defaults
  (`Own { jig: Defaults, forge: Box<[Stock]> }`, its five users moved: the one-process and
  real-loop fixtures, `crates/temper/tests/support/case.rs`, temper-service's
  `tests/stock.rs`, `startup.rs`). The domain world, which builds `Config` itself, gives
  `kinds_stock` through a world helper `temper_engine_domain::landing_stock(deployment,
  projects, change, review)` that translates its `LandingRule` values as the bridge did (the
  type stays for the worlds' builders alone; say so in its doc line); every domain world
  file that sets `Config.landing_rules` (`src/forge_routes.rs`, `src/fixtures.rs`,
  `src/walking.rs`, `src/oracle/config.rs`; `tests/suite/forge_routes/landing.rs`,
  `light_rules.rs`, `goal_branch.rs`, `goal_changes.rs`, `change_tool.rs`, `saved_retry.rs`;
  `tests/suite/forge_routes.rs`, `panels.rs`, `oracle.rs`, `settings/b2.rs`) gives the same
  rules through it, each story's outcome unchanged.
- Writers moved: `temper-init`'s template (`temper.json`: no `landing` rules, no `charter`;
  `landing` left out where nothing else remains) and its tests; `docs/examples/owned.json`;
  `docs/examples/reference/landing.json` goes, its two rules becoming kind 1's rules 1 and 2 in
  `docs/examples/reference/settings.json` (13.9), and
  `temper-service/src/tests/reference_landing.rs` decoding them through `temper-settings-form`
  instead, and `crates/temper/src/tests/reference_settings.rs` (which decodes that file through
  `settings_form::file::decode`) counting its two more items in `ITEMS` (6 after
  `reference-charters`' 4) and asserting the two rules (kind 1, numbers 1 and 2, as 13.9 has
  them); the cluster fixture's `engine.json` and the GitHub support's `engine.json` (their
  rules, if other than the stock's, into the stories' `settings.json` applied after sign-in);
  the fake backends' document rewriter (`temper-fake`); every Rust builder of the members
  (the survey in the header: `tests/client-browser/src/configuration.rs`,
  `tests/one-process/src/fixtures.rs`, `tests/real-loop/src/fixtures.rs`,
  `crates/temper/tests/support/case.rs`, `configuration.rs`, temper-service's
  `tests/fixtures/startup.rs`, `tests/startup.rs`, `tests/stock.rs`,
  `tests/welcome_policy.rs`, `tests/root_checks.rs`, the domain world's `change_tool.rs` and
  `goal_changes.rs`, `crates/temper/src/tests/help.rs`, `src/fake/tests.rs`,
  `tests/suite/fake.rs`).
- Help, named here as a stated departure from the implementer's text: `docs/help/
  landing-rule.md` (the rules are settings of kinds 1 and 2, the deployment's and a
  project's, changed with `temper ctl settings apply` and judged by the forge, reaching the
  next merge with no restart; enforced by requirements 5 and 6; lines 24, 40 and 60 today
  name `application.landing` and a restart), `charter.md` (the forge's charters are kind 3,
  numbers 1 and 2; lines 28 and 75), `policy.md` (line 98) and `repository.md` (line 25) where
  they name the document's members; their tests: `heavy cargo nextest run -p temper-docs` and
  `heavy cargo nextest run -p temper --lib -E 'test(/help/)'`.
- Status: `configuration.status.md`'s entry "13.2, 13.3, 13.6 step 2: ..." narrows to "13.2,
  13.6 step 2: kinds 4 and 5: nothing erases a deleted project's kinds 4 and 5" (or goes
  whole if `kinds-switch-rules` left only this task's words); `root.status.md`'s first **Not
  yet** entry goes where nothing of it is left.
- Packages pass (a change of meaning of a start): `-p temper-service --lib`; `-p temper-init`;
  `-p temper-fake`; `-p temper-engine-domain --lib`; `-p temper-engine-domain-world --test
  suite`; `-p temper --lib`; `heavy cargo check --workspace --tests`.

## Inputs

- `temper/docs/design/protocol/configuration.md`, section 3 (`landing`, `charter`, as this cut
  wrote them), 13.4 (the kind 1 and kind 3 rows; **Built at each start**), 13.5, 13.6 step 2
  and 13.9 (**The landing rules**); `temper/docs/design/protocol/setup.md`, 8.2 and 8.3 (what
  `temper init` writes).
- `temper/docs/plan/reference-m3/notes/common.md`; `notes/contracts.md`, C3.

## Owns and may change

- Owns the files in the header for the edits above.
- Options: if a writer the header does not list names a refused member (the compiler or a
  failing start names it), move it and list it in the report. If a live story's fixture
  (`live*`, `proof_live`) names one, move its file but never run it. If the reference's walk
  document (outside this repository) is a writer, say so in the report: fabrig's generated
  document is the lead's.
- Not this task's: kinds 4 and 5 (`landing.projects[]`'s ways, checks and tiers stay).

## Steps

1. The refusals and their words; the stock; `Given::Own`; the bridge's removal; the world
   helper.
2. The writers, file by file; the reference's settings file and its test.
3. The help pages.
4. Tests: temper-service `tests/startup.rs` (each refused member's line, a project's ways
   still read), `tests/stock.rs` (the forge's stock as 13.4 has it). Narrow first, each count
   checked; then the packages in Goal.
5. fmt, old forms, clippy over every package changed.
6. Commit ("Move landing rules and the forge's charters out of the document") and push.

## Report

The implementer's text's report, plus: every writer moved, and any the header missed.

## Rebase (2026-11-03, step merge, foreman reference-m3/kinds-document/foreman-1)

rebase the document migration onto current integration, preserving newer setup, merge-job and settings-read behavior.
Reapply `6d930264e3` onto `origin/reference-m3/integration`; do not replay its older dependency merges.
The landing failed with content conflicts and a modify/delete conflict; no test result establishes a retry.

Resolve the conflicts as follows:

- Preserve integration’s versions of `temper/crates/temper-client-protocol-wire/src/{generated/v1.rs,tests.rs}` and `temper/crates/temper-client-tools/src/jig/{render/document/proposal.rs,render/document/settings.rs,rows.rs,runner.rs,runner/reads.rs}`. These dependency changes include `23eb57bdbb` and `6a4d123705`; retain their host-hook extraction and newer wire behavior.
- In `temper/crates/temper-init/src/tests/documents.rs`, retain `5d0c252709`’s empty `engine.projects`, including the owned-example transformation, alongside this task’s assertions that landing rules and charter are absent.
- In `temper/crates/temper-service/src/tests/welcome_policy.rs`, retain integration’s explicit first-project creation, stock-policy snapshot and two successful adoptions followed by third-project refusal. Apply this task’s `kinds_stock` construction and remove all `landing_rules`/document-charter accesses.
- In `temper/crates/temper/tests/support/github_land.rs::configure_engine`, retain integration’s explicit `[{"number":1,"policy":1}]` project fixture. Keep this task’s settings-file migration; do not copy projects from init’s now-empty template.
- Reconcile `temper/docs/examples/reference/landing.json` with `936cefb055`: remove its deployment/project rules, but preserve project 1’s `merge_job: ".temper/merge"` as a rule-free landing fixture. This narrowly supersedes the spec’s deletion instruction because the file now carries surviving project configuration.
- In `temper/crates/temper-service/src/tests/reference_landing.rs`, retain the new settings-file rule decoder and preserve main’s merge-job parser assertion against that rule-free fixture. Keep rules 1 and 2 in `reference/settings.json`.
- In `temper/tests/conformance/src/lib.rs::Application::build`, retain `9723a21a03`’s direct configuration construction. Delete the entire `ChangedJudge` rule-disabling block; do not translate it to mutate `kinds_stock`.

Inspect the auto-merged landing parser to ensure `merge_job` parsing and validation survive alongside the new rule refusals.
Run the spec’s focused checks for the reconciled behavior, then its required package/workspace checks through `heavy`; never run live fixtures.
