+++
id = "bx-fixtures"
part = "fs"
kind = "migration"
needs = ["bx-harness", "fe-make", "sv-registry-setup", "sv-family"]
files = ["temper/crates/temper/tests/support/**", "temper/crates/temper/tests/process.rs", "temper/crates/temper/tests/suite/**", "temper/crates/temper/tests/fixtures/**"]
effort = "high"
minutes = 120
+++

## Goal

The process tier's and the suite's deployments name no forge: the document builders of
`temper/crates/temper/tests/support/` write no `application.forges`, `identities` or
`setup` (T4's document: `application.machine`, the new `application.demo` or `null`), and a
story whose deployment had a forge makes it at runtime through `temper ctl`, numbered from
the answer, before its own steps. Callers keep their shape: the move is central.

Acceptance:
- `support::configuration(directory, forge)` (`support/mod.rs`) and the builders in
  `support/configuration.rs` (`documents`, `check_document`, `demo_document`,
  `old_form_check_document`) write T4's document; the `secrets/forge-1.*` files it wrote
  for the document's file credentials go.
- A deployment built with a forge records it beside the document (a test file in the
  scratch directory, inside: the kind, addresses and, for a throwaway Forgejo, the admin
  token's path), and `boxed::run_story`/`run_ctl` make it before the story's steps:
  `forge make forgejo API GIT` (`support::forges::make_forgejo`), and for a throwaway
  Forgejo `forge connect N forgejo-admin FILE` until `forge list --json` reads it connected.
  `Forge::Unreached` makes the forge and connects nothing (the forge stays unreachable, as
  today). The number made is 1 on a fresh store, as the tests assumed.
- The suite's tests of the default profile (`tests/suite/`) that build documents through
  these builders pass; those that assert the document's forge members themselves
  (refusals of `application.setup` limits and entries, forge 3's asked group) stay as they
  are while the old members are still read, and are listed in the report for the task that
  refuses them.
- Every process test that built its deployment through these builders runs as before:
  this task runs the set named in step 4, and lists every other one it changed.

## Inputs

- `temper/docs/design/protocol/setup.md`, 13.4; `temper/docs/design/protocol/configuration.md`
  for the document's members.
- Draft `temper/docs/design/draft/playground-reset.md`, sections 3 and 8.
- `temper/docs/plan/playground-reset-r6/notes/common.md`.
- `temper/docs/plan/playground-reset-r6/notes/contracts.md`, T1, T4.
- `temper/docs/plan/playground-reset-r6/tasks/cl-ctl.md` (the commands), bx-harness's report
  (`support::forges`), bx-init's report (the document's shape).
- `rg -n '"forges"|\["forges"\]|"identities"|\["identities"\]|\["setup"\]|"setup"' temper/crates/temper/tests`:
  the files to move (about 30; `support/configuration.rs`, `support/mod.rs`,
  `support/owned.rs`, `support/forge_land.rs`, `support/first_repository.rs`,
  `support/csv_repair.rs`, `support/forgejo_elsewhere.rs`, `support/case.rs`,
  `support/github/*.json`, `fixtures/cluster/engine.json` among them).

## Owns and may change

- Owns: `temper/crates/temper/tests/support/**`, `tests/fixtures/**`, and the callers in
  `tests/process.rs` and `tests/suite/**` (call sites and the document literals they edit).
- Not here: `tests/qa_all/**` (qa tier; list what reads the old members), `tests/fake_backends/**`
  (bx-walks), `docs/examples/owned.json`.
- Options, pre-approved:
  - A fixture that needs its forge in a way the runtime path cannot give (a records-only
    forge whose file credentials a test reads, a GitHub fixture seeded in the document)
    keeps its document forge for now, still read until the refusal; list each under
    `## Left` with the reason.
  - `support/owned.rs`'s service-level deployments (`startup(document)` builds a
    `temper_service::Service` without a box or ctl): their documents move to T4's shape and
    their forge is registered as the service's tests register one, if
    `sv-registry`'s report gives a test seam; otherwise they keep their document forge and
    are listed.

## Steps

1. The builders and the made-at-start step; `heavy cargo check -p temper --tests`.
2. `heavy cargo nextest run -p temper --test suite` (default profile; whole binary).
3. `heavy cargo nextest run -p temper` (default profile).
4. Process tests, each alone, one at a time:
   `heavy cargo nextest run -p temper --profile process-temper --test process -E 'test(=NAME)'`
   for `rung_b0s_funds_document_applies_through_ctl_and_preserves_the_settings`,
   `temper_box_prints_its_link_and_stops_its_processes`,
   `temper_box_restarts_a_killed_forgejo_while_the_engine_serves`, and two more that use a
   throwaway Forgejo through `support::configuration(.., &forgejo.forge())` (name them).
5. `cargo fmt`; `heavy cargo clippy -p temper --all-targets`. Old forms never raised.
   Commit and push.

## Report

- Under `## Left`: every test or fixture still naming the old members, with the reason.
- The time a story now spends making and connecting its forge (measured on one run).
