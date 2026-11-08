# engine: status

2026-11-08. Where jig's code lags or departs from [engine.md](engine.md).

Section 7.2's immutable failed run end retains the invalid-result kind
computed for its attempt. No known gap remains in this field.

Section 7.3's asking-family routing and run proposal read are present.
`reads/settings.rs` checks settings authority and project scope before
reading, and checks them and the current attempt after loading a stored
row. `reads/vocabulary.rs` restricts retained task proposals to the
reader's project and settings actions. Section 7.3 carries the contract
for this read; reading as the person remains deferred in section 16.

Sections 17.9 and 17.11 have no known gap in the stored-policy size
refusal or a charter's selected effort. Size facts include parts and
cross proposal answers, client wire 19 and people store 14 unchanged,
with `StoredPolicy` appended after the existing members. Charter
resolution retains a supported selected effort; models admitted by
`rest` retain their catalogue defaults. No wire or store version changes.

Sections 6 and 6.1 have no known gap in interrupting agent claims on
hosts in the engine's process at an orderly stop. The core commits
`End::Recheck` before reporting drained; the fleet forgets each attempt
without telling its host. The restart table distinguishes a stopped
in-process worker from a crash and a worker elsewhere: committed turns
and saved work remain, no try or backoff is charged by the interruption,
and the task is due again. Jobs retain their claims for restart recovery.

Sections 7.1, 7.2, 7.4 and 9 have no known gap in selecting a charter's
parts from a run's reasons or taking only offered messages at a turn's
fence, beyond the **Not yet** entries below. Later runs
take `later`, reselecting the kept account when its provider changes;
charter documents still reach every run. The core-world per-wake table
covers first and later instructions and choices, a kept question,
undecided escalation, always-offered words and amendment, no parts,
and a question arriving between preparation and claim. Tool access to
`later` remains section 16's later work.

Section 17.2 has no known gap in an out-of-shape refusal's position:
it names the last request item for the failing setting, counted from
0, or 0 when no item names it. The refusal table covers a third item
failing with position 2 and the settings rows left unchanged.

Section 8 has no known gap in derived next-run refusals: standing hello
facts retain their figures, and admitted hosts too small for the run's
charter version give the first shared shortfall and their best offer,
or no named bound when their shortfalls differ. Account and credential
waits keep precedence; a fitting host leaves the ordinary host wait.

Sections 17.1, 17.7, 17.8 and 17.11 have no known implementation gap in
numbered model settings, release-default overlays, resolved charter versions,
withdrawal, catalogue notifications or the owner's newest-revision act.
Unlisted provisions have no known gap, including the saved-window load
and unread-tail retry guard in 7.1 and 7.2, and the prospective templates
query for settings judges in 17.5, and the bounded document refusal
facts carried through proposal answers in 17.9. Section 17.6's atomic
replacement of a run's own pending settings proposal has no known gap.
At `settings.proposals`, replacement is refused with the old set retained;
counting that set as room is deferred in section 16.
Section 11.1's account-driven marks preserve time-expiring offers: a
not-due account timer skips account work and leaves its alarm scheduled;
a due expiry marks and delivers the changed offer.

Section 17.12's held worker reads, filtered catalogue changes decided by
the provider's list, atomic account and settings commits, and retention
of running work have no known implementation gap. The mark table covers
failures, repeats, first empty reads and catalogue changes; account times
alone mark no Frame. The omitted-count mark below remains unimplemented.
Later pages, later changes to a known name's figures
and catalogue-number reuse remain section 16's open questions.


## Gaps and departures

- **Not yet:** 7.4: a finish while the task's
  delegates are live fails its attempt invalid rather than parking it. 8: an
  invalid start other than a transcript's is the invalid class, not the
  permanent one.

- **Not yet:** 17.12: a read changing only the displayed omitted count
  does not mark `me`. `routing/lists.rs`, `answered`, compares good-read
  presence and `names`, but not `left_out`; `documents/small.rs`, `frame`,
  adds the visible accounts' largest `left_out` to each project's count.
  Settings can therefore keep the old count until another Frame mark.
  The mark table has no row for this case; this finding is by reading,
  with no runtime result supplied. No reason for the omission is recorded.

- **Departs:** 11.1: an omitted template's preview uses the first offered
  model, while a task's next-run selection needs a supporting host to call
  even a direct account usable and otherwise takes the first serving one.
  These can differ across a worker restart with Claude and OpenAI accounts;
  `../client/domain/contract.status.md`, 5.1 records the source paths and
  reason. The offer remains available as this section requires.


- **Not yet:** 2, 8: runs on the engine's own slots under
  `smith-inline-agent`; the engine hub's hello before placement; the
  two-claim placement balance. 4.2, 5.3: the remaining point-load fences.
  7.1, 9: the brief's lineage, calls and waiting sections. 7.2: what a
  turn's calls made, written beside the turn. 7.3: the tools `move`, `succeed`,
  `hold` and `lift`.
  15: core-world stories that opt out of the restart
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
  core's unit fixtures need no start; every root starts it.
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
