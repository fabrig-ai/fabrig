+++
id = "proposal-projects"
part = "w3-engine"
kind = "interface"
needs = []
files = [
  "jig/crates/jig-domain-core/src/settings.rs",
  "jig/crates/jig-domain-core/src/inbox.rs",
  "jig/crates/jig-domain-core/src/documents/inbox_deployment.rs",
  "jig/crates/jig-domain-core/src/documents/objects.rs",
  "jig/crates/jig-domain-core/src/reads/rows.rs",
  "jig/crates/jig-client-domain/src/contract/inbox.rs",
  "jig/crates/jig-client-domain/src/contract/proposal.rs",
  "jig/crates/jig-client-protocol-wire/schema/**",
  "jig/crates/jig-client-protocol-wire/src/generated/**",
  "jig/crates/jig-client-protocol-wire/golden/**",
  "jig/crates/jig-protocol-engine-clients/src/translate/**",
  "jig/crates/jig-client-protocol-link/src/translate/**",
  "jig/tests/core/tests/suite/settings_proposals.rs",
  "temper/docs/design/client/journeys/**",
]
effort = "high"
minutes = 120
+++

# A proposal of settings names the projects it belongs to

## Goal

W3's engine half. A proposal of settings waits for owners in ctl, but its inbox row and document say project 0, so no project's Needs you can show it. Have the core work out its `projects` from its items, carry them on the inbox row and the proposal document, and make a read's project filter honour them, as `notes/contracts.md`, 2, has it. The client's use of it is `needs-proposal`'s.

Acceptance:

- An application's item of scope 1 gives a row with `project` `Some(0)` and `projects` `[1]`; the filtered read of project 1 has it and project 2's doesn't.
- Items of scopes 1 and 2, or a project's policy beside an item of the other: `projects` `[1, 2]`, in both filtered reads, one row unfiltered and `cached_needs` up by one.
- A charter's item alone, or no items: `projects` the projects the reader owns, not one where the reader is only a member.
- Both for a party's proposal and a run's (`propose_run`).
- Accepting from `project` 0 works as before; the decided proposal leaves the rows.
- A world over the real wire reads `projects` on a row and a proposal.
- The four journeys are re-recorded and replay.

## Inputs

- `temper/docs/plan/ui-walk-large/inputs/request.md`, W3.
- `temper/docs/plan/ui-walk-large/notes/common.md`; `notes/contracts.md`, 2.
- `jig/docs/design/domain/engine.md`, 17.5, 17.6; its status's 17.6 on the holder, which this task doesn't change.
- `jig/docs/design/client/domain/contract.md`, 5.1.
- `temper/docs/design/client/journeys.md`, 8.1; `skein/docs/design/codec.md`, on `skein-codegen`. Commit `f9f04a41c6` changed this schema in place: its diff shows the files such a change regenerates.

## Owns and may change

Owns the `files` above.

May also change, each only for the new fields:

- The `InboxEntry { .. }` and `Proposal { .. }` literals: in jig, `jig-protocol-engine-clients/src/tests/fixtures/live.rs`, `jig-domain-core/src/tests/inbox/inbox_selection.rs`, `jig-client-view/src/tests/tree/sections/inbox.rs`, `jig-client-view/src/tests/fixtures.rs`, `jig-client-view/src/tests/state/domain/owner_inbox.rs`, `jig-client-protocol-link/src/tests/fixtures/link_deliveries.rs`, `jig/tests/ops/src/client/stories.rs`, `jig/testing/jig-client-kit/src/stories.rs`, `jig/testing/jig-client-kit/src/stories/shared/b_inbox.rs`, `jig/testing/jig-client-scripted-engine/src/tests.rs`, `jig/testing/jig-client-scripted-engine/src/codec/values/inbox.rs`, `jig/testing/jig-client-test-records/src/live_values.rs`.
- In temper, the same field, with an empty list: `temper-client-domain/src/tests/pages/inbox.rs`, `temper-client-domain/src/tests/fixtures.rs`, `temper-client-tools/src/tests/fixtures/digest.rs`, `temper/testing/temper-fixtures/src/client_documents.rs`, `temper/tools/gallery/src/worlds.rs`. A branch stays in its tree but for a required field: these lines name the field alone.
- `jig/tests/conformance/tests/suite/client_settings_proposals.rs`: one story over the wire. `jig/tests/codec-families.rs` if its inventory lists what changes.
- `jig/docs/design/domain/engine.status.md`: remove the 17.6 line that starts "a proposal of settings has no `projects`". `jig/docs/design/client/domain/contract.status.md`: remove the **Not yet** on 5.1. Only those lines.

Don't touch `temper-client-domain/src/pages/**`, `temper-client-view/src/**`, `temper-fake/**` or `stock.rs`: other tasks own them.

Options, each pre-approved:

- **Wire:** v19, in place, no compatibility and no new version.
- **Where `touched_projects` lives:** beside `Item` in `settings.rs` by default; a submodule of `settings/` if that file's line limit asks.
- **A start check or `document_limits` world says a document's bound is now short** by what 32 u32s add to a row: raise that bound the least the check takes, where the profile says it moves with what it carries (`temper/docs/design/protocol/configuration.md`, 13.4, on coupled limits), and name both figures in your report. No test budget or timeout moves.
- **The mark:** a role's change marks a party's inbox already (`jig/docs/design/client/domain/contract.md`, 7.2). If a test shows a new project owned by the reader leaves a row of the deployment's stale, mark the inbox in that decision.
- **A proposal's scope that names a closed project:** keep it in `projects`; a client never opens it, and `project_allowed` refuses that filter.

## Steps

1. The core: `touched_projects`, `settings_row_projects`, as `notes/contracts.md`, 2. A party's roles as `documents/small.rs`'s `me` walks them, `Role::Owner` and `open_project`. Rows in `documents/inbox_deployment.rs` take it for a `PROPOSAL_WITHOUT_TASK` proposal, and every other row an empty list. `reads/rows.rs`: `inbox_filter` uses `shows_in`; `person_proposal_need`, for `PersonAction::Settings`, tests the filter with `settings_row_projects` and fills the row. `documents/objects.rs`'s `proposal`: `projects` from `core.settings_proposals`' set.

2. The test first, in `jig/tests/core/tests/suite/settings_proposals.rs`, sketch:

   ```rust
   #[test] fn a_proposal_of_one_projects_item_shows_in_that_project_alone()
   #[test] fn a_proposal_touching_two_projects_shows_in_both_and_counts_once()
   #[test] fn a_deployments_proposal_shows_in_each_project_its_reader_owns()
   #[test] fn a_runs_proposal_belongs_as_a_partys_does()
   ```

   Each reads the `inbox` document and `Read::Inbox` with a project filter. Break the rule once (`inbox_filter` back to `entry.project`; `touched_projects` ignoring `ProjectPolicy`) and see each fail on its row.

3. The contract, the schema's two lines, `skein-codegen` for `generated/` and goldens, the translators both ways.

4. `jig/tests/conformance/tests/suite/client_settings_proposals.rs`: one story reading `projects` through the wire and link.

5. **A change of meaning runs its readers' tests:** a read's project filter. Run `jig-domain-core`, `jig-client-domain`, `jig-client-protocol-wire`, `jig-protocol-engine-clients`, `jig-client-protocol-link`, `jig-client-view`, `jig-client-kit` and its worlds (`jig/tests/core`, `conformance`, `client-protocol`, `client-wire`, `engine-clients`, `ops`), and in temper `temper-client-tools`, `temper-client-domain`, `temper-client-world`, `temper-engine-domain-world`, with `heavy cargo check -p temper-client-gallery --tests`.

6. **Re-record,** each on a fresh fake box, by `heavy cargo run -p temper-client-journeys -- again NAME`: `welcome-set-up`, `welcome-playground`, `project-landing`, `chat-draft`, then `heavy cargo run -p temper-client-journeys` to reprint. `notes/common.md`, *Recording flakes*, holds.

7. fmt, `old_forms` for jig and temper, clippy, `python3 -I -B tools/citations/citations.py --base origin/main`; commit and push. Small commits: the core, the wire, the recordings.

## Report

As `notes/common.md`, *Your report*, and each journey's tries; any bound you raised; the packages step 5 ran.
