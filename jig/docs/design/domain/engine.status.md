# engine: status

2026-11-06. Where jig's code lags or departs from [engine.md](engine.md).
Unlisted provisions have no known gap, including the saved-window load
and unread-tail retry guard in 7.1 and 7.2.

- **Not yet:** 2, 8: runs on the engine's own slots under
  `smith-inline-agent`; the engine hub's hello before placement; the
  two-claim placement balance. 4.2, 5.3: the remaining point-load fences.
  7.1, 9: the brief's lineage, calls and waiting sections. 7.2: what a
  turn's calls made, written beside the turn. 7.3: the tools `move`, `succeed`,
  `hold` and `lift`.
  11.1: `me`'s
  `resolved_default`. 15: core-world stories that opt out of the restart
  oracle with a named finding (`broken()` in `jig/tests/core`), most for
  a claim the start settles (6, step 3) changing a task's `chip`, `acts`
  or run `settings`, which the projection keeps durable, others for a
  frame's `needs` with a person proposal left pending and
  `Deployment.messages` advancing on a retried deadline; the core
  world's `world` and `direct` drivers run no oracle.
  17.6: pending reasons admitted past `people.words` are cut at that bound
  in the review, with loss stated; no continuation is served.
  17.6: a
  proposal of settings waits for the deployment's owners whatever it
  names, its holder always the deployment's policy
  (`jig-domain-core/src/admission/proposals.rs`, `holder`): one naming
  an application's items of a project's scope is accepted by any of the
  deployment's owners, not by that project's; one naming a project's
  policy is admitted only where one party owns both the deployment and
  the project, and accepted only by such a party, even with no item of
  the deployment's beside it, where 17.6 has that project's owners
  decide it (`routing/settings/dispatch/project_policy.rs`, `owns`). Its
  row goes to the deployment's owners alone
  (`documents/inbox_deployment.rs`), so a project's owner who doesn't own
  the deployment reads none in that project.
  17.10: the core world's eleven restore stories
  (`jig/tests/core/tests/suite/settings_restore.rs`) are all ignored,
  each naming `restore-request`'s `Restore` admission or its
  `ForgetReplaced`, both on main since (`routing/party/people.rs`,
  `settings/restart.rs`, `forget_replaced`), and the story of a deleted
  project's items also naming a project-scoped default that survives
  the project's deletion (`connectors.md`, 17); whether they pass
  unignored is unknown. The core's own tests of a restore run
  (`jig-domain-core/src/tests/settings/restore.rs`).
- **Not yet:** 17.5: validation of declared application kinds against
  configured limits before construction is missing, because
  `jig-domain-core/src/settings.rs`, `Current::new`, expects the capacity to
  be representable instead of returning a named configuration refusal; the
  bounds check takes limits without declared kinds. No reason is recorded.

- **Not yet:** 5.1, 17.6: the whole project-deletion commit has no
  established admission price against `journal.writes`, because the
  established allowance covers settings-proposal withdrawals alone,
  excluding the remaining state writes, closing the project and cancellation
  operations. No reason is recorded.

- **Departs:** 9.1: a transcript's load refused for want of a load slot
  counts as transient, though the section pauses a task for room that frees
  by itself and the request `brief-failure` of 2026-11-05 keeps a true capacity
  refusal's pause: temper's root answers no slot as `TranscriptLoad::Failed`
  (`temper-engine-domain/src/restart.rs`, `loads::begin`;
  `archive.rs`, `transcript_failed`), one word for that and a page it cannot
  take. So two conversations loading together can spend a transient try, and
  the next run starts fresh from the tail rather than resuming (7.2). Shipped
  `root.loads.loads` is 2 (`temper-service/src/limits.rs`); the two stories that
  met it set 3 in their fixtures (`temper/tests/system/src/composed/agreement.rs`,
  `temper/tests/real-loop/src/green_light.rs`). No reason is recorded.
- **Design departs:** 10, from the request `note-parts` of 2026-11-05, decided
  by its plan's cut: a range names the revision it read in `recalled`, the tool's
  member that already says it, where the request has `revision`, so that one fact
  keeps one name and a call without `first` stays as it is.
- **Design departs:** 10, from the request `ui-chat-draft` of 2026-11-05, decided by
  its plan's cut: `core::Held::NotesWritten` keeps its owner, name and revision
  (`jig-domain-core/src/vocabulary.rs`), where the request gives it `lines`, `added`,
  `removed` and `first` too, because nothing makes it: a write's answer goes by the
  call's own part, which has them.
- **Design departs:** 17.6, from the request `ui-walk-large` of 2026-11-06, decided by
  its plan's cut: that request has a proposal of settings' project set when it is
  made, and the core works its `projects` out at each read, because its items never
  change once it is made, so the value is the same and no stored format moves. That
  request has it show in each project it touches and in every project whose owner can
  decide it; a row names the lowest 32, the bound of that list on the client's wire.

- **Departs:** 6: `Core::new` without a start is ready, because the
  core's unit fixtures need no start; every root starts it. 8: the
  client wire drops the failing charter's number of a
  shortfall's turns, time and waiting
  (`jig-protocol-engine-clients/src/translate/deployment.rs`), because its
  shortfall arms carry no charter; a client wire version adds it.
  4.2: the core's answers to parties cross the engine's client translation
  with optional refusal and ending facts always empty; the fields and
  the choice to fill them or drop them in a client wire version are in
  `jig/docs/design/client/domain/contract.status.md`; no reason is recorded.
  4: the root interface has no text for public `Core::effect_owner` and
  `application_part_request`
  (`jig-domain-core/src/routing/flight.rs`), `call_room`
  (`jig-domain-core/src/runs/calls.rs`), or `next_finishing_deadline` and
  `finishing_due` (`jig-domain-core/src/routing/timers.rs`), which the test
  service uses; no reason is recorded.
  4.1: children's timers arrive through `fire`, but the section has no
  text on how many due children a pass fires.
  `jig-domain-core/src/routing/timers.rs`, `due`, returns one `Timer`, the
  first due child in people, tasks, fleet, brief and accounts order;
  `temper-engine-domain/src/domain.rs`, `fire_routed`, fires it once per
  pass, and `jig-test-service/src/engine.rs`, `fire`, also fires one;
  no reason is recorded.
  17.2, 17.6, 17.8: accepting proposed settings rechecks the change and
  refuses it as full, leaving it pending, if held revisions or charter
  versions would pass their limits. With one charter slot, the acceptance
  path (`jig-domain-core/src/routing/settings/dispatch.rs`, `accept` and
  `decide`) answers `ProposalDecided { Accepted }` rather than `Full`;
  whether a capacity check is missing or the expectation is wrong, and
  whether this departs, is the designer's decision: it is a bug if the
  design requires `Full` in this case; no reason is recorded.
