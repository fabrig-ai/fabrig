# configuring-p2: the code batch, sketched

Written by the first cut (2026-11-03) for the cut that releases this plan's code once the
reference's M3 code is on this trunk (`plan.md`, "Why the code waits"). A sketch, never a
spec: that cut checks every file and name below against the trunk it then finds, surveys the
readers, and writes the specs. The design it builds is on this trunk already: jig's
`domain/engine.md`, 17.2, 17.6 (a project's policy), 17.9, 17.10; `domain/people.md`, 5.2;
temper's `protocol/configuration.md`, 13 and 13.1; `client/tools.md`, 5.3. The shapes are
`notes/contracts.md`, P1 and P2.

## What M3 builds, which this plan never copies

`kinds-ctl-form` (a settings file's `landing-rule` and `forge-charter` items),
`kinds-ctl-reads` (`settings show landing-rule|forge-charter`, the forge's `Setting` and
`ProposalItems` reads, `proposal show`'s items of temper's kinds), `apps-*` (an application's
kinds judged and kept by their connector, proposed, stored as state and size, on the wire).
The entry's first two Builds ("the items of temper's kinds in a settings file and in a
proposal's review; `settings show landing-rule` and `forge-charter`") are those tasks': this
plan's walk exercises them, and its tasks extend them only for a project's policy and for
restore.

## Design still to write, once M3 lands

The entry's "what that asks of 17.5, M3's, written after M3 lands", and the citations left in
M3's sections (the draft's status, `temper/docs/design/draft/configuring.status.md`):

- jig's `domain/engine.md`, 17.5 ("History"): a connector keeps, beside each change it
  saves, the value it replaced, by the revision that made it, while the history keeps that
  record (17.10, "A connector keeps what its items replaced"); `domain/connectors.md`,
  section 17: the asks, a change's replaced values kept with `KeepSettings`, the record's
  erasure, and a restore's items handed to the connector, answered with their sizes and
  charters as a proposal's items are.
- jig's `client/domain/contract.md`, 3.2 (`ChangeSettings`' and `Propose`'s items, a
  project's policy among them; `Restore`'s answer, no application's kinds left out) and 5.1
  (the review's item of a project's policy): not named by the entry, a question for the user
  in the first cut's report.
- temper's `protocol/configuration.md`, 13.2 ("Never pinned", 4's always-current authority)
  and 13.4 (3.2's release defaults) in their own words; `client/tools.md`, 5.2's `revision
  show` (temper's kinds' replaced values are kept).

## The tasks, as sketched

Part `policy-item` (slice 1):

| Task | Kind | Needs | Owns, roughly | Minutes |
|---|---|---|---|---|
| `policy-surface` | interface | | `jig-domain-people/src/settings.rs` (`Setting::ProjectPolicy { project }`), the core refusing it `Shape` until `policy-change`; compiler-forced arms in jig and temper | 60 |
| `policy-change` | feature | surface | `jig-domain-core`'s settings check and change (17.1's policy check against the set left, the people child's edit checks whole, *not theirs*, *empty*, unknown project), the people child's whole replacement in the change's decision, 17.6's waiting for owners of both, the deletion's withdrawal, the review's now and would-be; unit tests in `jig-domain-core/src/tests/settings/` | 150 |
| `policy-store` | feature | surface | `jig-protocol-store`'s `settings-v1` `Setting.project_policy` in a revision record's items and a pending proposal's set, codec and goldens (in place, as M3's `apps-store`) | 90 |
| `policy-wire` | feature | surface | jig's client wire `Setting.project_policy` and `ReviewItem` (in place, as M3's `apps-wire`), `jig-protocol-engine-clients` and `jig-client-protocol-link` translations, `jig-client-domain`'s contract | 120 |
| `policy-ctl` | feature | wire | `temper-settings-form` (the `project-policy` item; `edit` and a second refused before sending), `temper-client-tools` (`proposal show` printing the policy now and proposed), `docs/help/settings.md`, `configure.md`, `reference/ctl-settings.md` | 90 |
| `policy-story` | feature | change, store, wire, ctl | jig's core world (an owner of both sets charters and a project's policy in one change; a deployment-only owner refused *not theirs*; a proposal no party owns both of refused at its entrance; a deletion withdrawing one; a restart keeping a pending one); temper's domain world, a settings file through ctl's runner | 90 |

Part `restore` (slice 2, the walk):

| Task | Kind | Needs | Owns, roughly | Minutes |
|---|---|---|---|---|
| `restore-records` | feature | policy-store, policy-change | each record's item keeping what it replaced (jig's kinds whole, a project's policy whole, an application's state and size), a release's revision its defaults; `settings-v1` `Item` | 120 |
| `restore-request` | feature | records | jig's `Restore { revision, reason }`: the records after R loaded a page at a time, the proposal against the newest, its refusals (forgotten, empty, stale, too large), a deleted project's items left out and named; jig's kinds and projects' policies | 150 |
| `restore-wire` | feature | request | the client wire's `Restore` and its answer, the translations, the contract | 90 |
| `restore-connector` | feature | request | jig's side of 17.5's replaced values (once written): the asks, `jig-test-connector` keeping and answering them | 150 |
| `restore-forge` | feature | connector | the forge keeping kinds 1 to 3's replaced values in its family `Setting` and answering a restore's items (`temper-engine-domain-forge`, the root's `route/kinds.rs`, temper's store) | 150 |
| `restore-ctl` | feature | wire | `temper ctl revision restore REVISION [REASON]` (`jig/rows.rs`, `runner/acts.rs`, its render), its help page and golden | 90 |
| `restore-story` | feature | forge, ctl, policy-story | temper's domain world: a settings file of each kind (charter, template, policy, rules, landing rule, forge's charter, a project's policy) proposed, shown, accepted and restored, the walk's story in the gate | 120 |

Critical path as sketched: `policy-surface` (60) -> `policy-store` (90) -> `restore-records`
(120) -> `restore-request` (150) -> `restore-connector` (150) -> `restore-forge` (150) ->
`restore-story` (120), about 840 implementer-minutes; `restore-connector` also waits for the
17.5 design above.

## Old code

The entry's Old code (`configuring-revisions/restore`, `set4-r2`, `replaced`,
`settings-file`, in hel8's attic mirror) was not read by this cut; implementers cannot reach
it, so whatever the next cut takes from it goes into the specs.
