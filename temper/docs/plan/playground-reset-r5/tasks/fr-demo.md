+++
id = "fr-demo"
part = "forge-records"
kind = "migration"
needs = ["fr-domain", "fr-root"]
files = ["temper/crates/temper-engine-domain-forge/src/demo.rs", "temper/crates/temper-engine-domain-forge/src/domain.rs", "temper/crates/temper-engine-domain-forge/src/restore.rs", "temper/crates/temper-engine-domain-forge/src/reconcile.rs", "temper/crates/temper-engine-domain-forge/src/tests/demo.rs", "temper/crates/temper-engine-domain/src/route/demo.rs", "temper/crates/temper-engine-domain/src/domain.rs", "temper/tests/engine/domain/tests/suite/demo.rs"]
effort = "high"
minutes = 90
+++

## Goal

The demo's forge becomes a record made at the demo's start, by the
smallest change that keeps today's playground working (the forge part's
brief; `notes/forge-records.md`, section 0): with a playground template
(`configure_playground(Some(..))`), `StartDemo`'s first decision makes the
fake forge through `make_forge(NewForge::Playground)` and the demo runs on
its number; the document's demo forge is used only while no template is
given (until `fr-demo-doc` removes it). The playground part
(`pg-start`) later replaces the demo whole; this task leaves it the
`Demo` row, `DemoAct` and the end as they are.

Acceptance:

- `demo::start`, no `Demo` row and a playground template: `make_forge(
  NewForge::Playground)` in the start's decision (a refusal answers the
  start `NotOffered`), then the `Demo { forge: n, state: Running }` row
  with that number, the demo's adoption (`DemoConfig.adoption`) taking
  `forge: n` and the template's git host (`family::host`'s rule: the
  host of the template's `git`), kept in the domain beside `DemoConfig`
  (a `demo_forge: Option<u16>` set by the start and by restore). A
  restored `Demo` row is admitted when its forge is a kept record of
  mode `Fake`, or, with no template, the configured demo forge (today's
  check in `restore.rs` / `domain.rs`'s restore and `reconcile.rs`'s
  `Need::Demo`).
- Everything that reads `demo.adoption.forge` (`needs_group`, `restart`,
  `connected`, `forge_ended`, `demo_adopted`, and the root's
  `route/demo.rs`: the adoption it sends, the end's selection) reads the
  domain's demo forge instead, through one accessor
  (`Domain::demo_forge() -> Option<u16>`).
- "Another forge connects ends the demo" (`demo::connected`) counts any
  configured or record forge other than the demo's.
- Tests: forge domain `src/tests/demo.rs`
  `a_start_with_a_template_makes_the_fake_forge_record_and_asks_its_group`,
  `a_restart_restores_the_demo_on_its_record_forge`,
  `a_start_with_the_fake_port_taken_answers_not_offered`; the engine
  domain world's `tests/suite/demo.rs`
  `the_demo_adopts_widget_on_its_made_forge_and_ends_on_another_forges_connect`.
  Every existing demo test passes (they give no template, so they run on
  the configured forge).

## Inputs

- `temper/docs/plan/playground-reset-r5/notes/common.md`,
  `temper/docs/plan/playground-reset-r5/notes/contracts.md`,
  `temper/docs/plan/playground-reset-r5/notes/forge-records.md`
  (sections 0 to 2).
- `temper/docs/design/protocol/setup.md`, 8.4 (the demo's start and end).
- Survey `temper/docs/plan/playground-reset-r5/notes/surveys/s3-playground-today.md`,
  sections 1 and 2 (every demo reader, engine and root).

## Owns and may change

- Owns: the files listed in `files`: in `temper-engine-domain-forge`
  `src/demo.rs`, its tests, and the demo's lines of `src/domain.rs`,
  `src/restore.rs`, `src/reconcile.rs`; in `temper-engine-domain`
  `src/route/demo.rs` and the demo's lines of `src/domain.rs`; the world's
  demo suite.
- Not yours: the service's `settings::Demo`, `application.demo`, the
  template, `temper-fake` (`fr-demo-doc`).
- Options: if keeping `DemoConfig.adoption.forge` as a placeholder is
  smaller than removing it, keep it and say so (the playground part
  replaces `DemoConfig`).

## Steps

1. The domain's demo forge and its accessor; the start's make.
2. Restore and reconcile on a record; the root's readers.
3. Tests. Narrow: `heavy cargo nextest run -p temper-engine-domain-forge
   --lib -E 'test(/demo/)'`; `heavy cargo nextest run -p
   temper-engine-domain-world --test suite -E 'test(/demo/)'`.
4. `cargo fmt`; `heavy cargo clippy -p temper-engine-domain-forge -p
   temper-engine-domain -p temper-engine-domain-world --all-targets -- -D
   warnings`; focused tests of the three; `heavy cargo check -p
   temper-service --tests`. Old-forms from `temper/`. Commit and push.

## Report

As `notes/common.md` says; what of the demo you left for `pg-start`.
