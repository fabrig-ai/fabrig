# settings: status

2026-11-27. Where the tree's code lags or departs from [settings.md](settings.md).



- **Not yet:**  engine slots, hello and placement remain `fleet.status.md` (Not yet, the whole document).  the point-load fences remain `vocabulary.status.md`, 1.2, and `decisions.status.md`, 1.3.
   the brief's missing sections remain `briefs.status.md`, 1.  turn-call objects remain `executors-transcripts.status.md`, 1.  the missing tools remain `executors-tools.status.md`, 1.
   the restart-oracle gaps remain `../testing.status.md`, 9.
  10: the core world's eleven restore stories
  (`jig/tests/core/tests/suite/settings_restore.rs`) are all ignored,
  each naming `restore-request`'s `Restore` admission or its
  `ForgetReplaced`, both on main since (`routing/party/people.rs`,
  `settings/restart.rs`, `forget_replaced`), and the story of a deleted
  project's items also naming a project-scoped default that survives
  the project's deletion (`../connectors.md`, 17); whether they pass
  unignored is unknown. The core's own tests of a restore run
  (`jig-domain-core/src/tests/settings/restore.rs`).
- **Departs:**  the unstarted-core departure remains `restart.status.md` (Departs, the whole document).
   the optional-answer facts departure remains `jig/docs/design/client/domain/contract/answers.status.md`, 1.
   the undocumented root methods remain `vocabulary.status.md`, 1.
   the child-timer departure remains `vocabulary.status.md`, 1.1.
  2, 8: accepting proposed settings rechecks the change and
  refuses it as full, leaving it pending, if held revisions or charter
  versions would pass their limits. With one charter slot, the acceptance
  path (`jig-domain-core/src/routing/settings/dispatch.rs`, `accept` and
  `decide`) answers `ProposalDecided { Accepted }` rather than `Full`;
  whether a capacity check is missing or the expectation is wrong, and
  whether this departs, is the designer's decision: it is a bug if the
  design requires `Full` in this case; no reason is recorded.
