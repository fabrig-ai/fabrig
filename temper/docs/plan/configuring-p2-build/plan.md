# configuring-p2-build: plan

Phase P2 of `temper/docs/design/draft/configuring.execution.md`: settings through ctl, built
from the design `configuring-p2` released, on the reference's M3 as landed on main. 14 tasks
(1,535 implementer-minutes), all released in five batches; P1 (a charter's documents reaching a run's brief), which the
walk also exercises, landed with `configuring-p1`.

## Parts

| Part | Slice | Design | Tasks |
|---|---|---|---|
| `policy-item` | 1 | jig's `domain/engine.md` 17.2 and 17.6; `domain/people.md` 5.2; temper's `protocol/configuration.md` 13.1; `client/tools.md` 5.3 | `policy-surface`, `policy-change`, `policy-ctl`, `policy-story` |
| `restore` | 2, **the walk** (`walk = true`) | jig's `domain/engine.md` 17.5 and 17.10; `domain/connectors.md` 17; `client/domain/contract.md` 3.2, 5.1, 6; temper's `client/tools.md` 5.2 and 5.3 | `restore-surface`, `forge-replaced`, `connector-replaced`, `restore-connector`, `restore-forge`, `settings-store`, `settings-wire`, `restore-request`, `restore-ctl`, `restore-story` |

The cross-task shapes are `notes/contracts.md` (B1 on), which takes configuring-p2's
`notes/contracts.md` (P1, P2) as binding; the standing rules, `notes/common.md`.

## The trunk

This trunk was made from `configuring-p2/integration`, which sits on
`reference-m3/integration` at 3db9a0e69a, before M3 landed. The tick's catch-up with main
waits while the cut is in flight, so the cut's first commit merges `origin/main` (M3 landed,
224965929f) into its branch, and the trunk takes it with the first batch: every task starts on
M3's `apps-*` and `kinds-*`. One conflict, `temper/docs/design/client/tools.status.md`: main
built `forge replace`, which leaves the line; P2's 5.3 entry stays.

## The design this cut wrote (batch 2)

The addenda the request names, after M3, in their sections and no other:

- jig's `domain/engine.md`, 17.5 "History": the connector keeps, beside each change, the value
  it replaced, by revision, while the history keeps the record (17.10).
- jig's `domain/connectors.md`, 17: "What its items replaced, for a restore": the keeping, the
  erasure with the record, a restore's items handed over and answered with sizes and charters.
- jig's `client/domain/contract.md`, 3.2 (`ChangeSettings`' and `Propose`'s items naming a
  project's policy; `Restore`'s answer and what it leaves out), 5.1 (the review's item of a
  project's policy), 6 (`revision{n}`'s items: an application's with the state it replaced, a
  project's policy with the one it replaced): compatible detail.
- temper's `protocol/configuration.md`, 13.2 "Never pinned" and 13.4 "A release that changes a
  default", in their own words; `client/tools.md`, 5.2's `revision show` printing what each
  item replaced.
- Statuses: jig's `engine.status.md` (17.5, 17.10), `connectors.status.md` (17),
  `contract.status.md` (3.2, 5.1, 6); temper's `tools.status.md` (5.2); the draft's
  `configuring.status.md`, its 3.3 entries closed and 13.2/13.4's citation narrowed to P3's
  charter 3.

## Batches

1. `policy-surface` (interface, 60): the setting named, refused `Shape` until its check.
2. `restore-surface` (interface, 90; needs `policy-surface`): B2 to B4's types, each item's
   `replaced` recorded for real; `forge-replaced` (75-90) and `connector-replaced` (75), no
   needs: temper's forge and jig's test connector keep the values their changes replaced, on
   the `KeepSettings` ask that already carries the revision.

3. `policy-change` (150; needs `restore-surface`): the core's check, commit, proposal and
   review of a project's policy; `restore-connector` (120; needs `restore-surface`,
   `connector-replaced`) and `restore-forge` (120; needs `restore-surface`, `forge-replaced`):
   the restore's asks answered and routed, in jig's test domain and temper's root.

4. `settings-store` (120; needs both surfaces): settings-v1 and people-v14 in place, a
   project's policy, each item's `replaced`, a restore's stored answer; `settings-wire` (170;
   the same needs): client-v19 in place and jig's client crates; `policy-ctl` (90; needs
   `settings-wire`): the settings file's `project-policy` item and `proposal show`;
   `restore-ctl` (90; needs `settings-wire`, `policy-ctl`): `revision restore` and `revision
   show`'s `was:`; `restore-request` (150; needs `policy-change`, `settings-store`,
   `restore-connector`, `restore-forge`): the core's restore decision and `ForgetReplaced`.

5. `policy-story` (90; needs `policy-change`, `settings-store`, `settings-wire`,
   `policy-ctl`): jig's core world and temper's joined client world, a project's policy
   through a change and a proposal; `restore-story` (120; needs `restore-request`,
   `restore-forge`, `restore-connector`, `restore-ctl`, `policy-story`): the walk's story in
   the gate, jig's core world, temper's domain world and a settings file of each kind through
   ctl in the joined client world; it closes the status entries left.

## Critical path

`policy-surface` (60) -> `restore-surface` (90) -> `settings-wire` (170) -> `policy-ctl` (90)
-> `restore-ctl` (90) -> `restore-story` (120): about 620 implementer-minutes. The core's line
(`restore-surface` -> `policy-change` (150) -> `restore-request` (150)) ends at about 450, and
`policy-story` at about 500, both before `restore-ctl`. One writer per crate keeps
jig-domain-core's four tasks in a line and temper-client-tools' two; the store, connector and
forge tasks run beside them.

## Decided along the way

- **The ctl stories run in temper's joined client world** (`temper/tests/client`), ctl against
  the engine root, not in the one-process world: temper's `domain/testing.md`, 6.4 gives the
  one-process world no new story.
- **Too large is counted before anything is left out** (contracts B4), bounding `left_out`.
- **Status lines one task at a time:** jig's `engine.status.md` 17.10 line is
  `restore-request`'s, not `settings-store`'s, which runs beside `policy-change` (whose 17.2
  line it borders); temper's `tools.status.md` 5.3 restore-items line is `restore-story`'s.
- **Not this plan's:** `settings show landing-rule|forge-charter` and a settings file's items
  of temper's kinds are M3's, landed; the walk exercises them.

## Overlaps with other plans

Checked at the cut against `reference-m10/integration` and the queue's `ready/` branches of
v0.1.0-ladder, sign-in-wait and reference-walks:

- `temper/docs/design/client/tools.status.md`: reference-m10 edits the same 5.2/5.3 line this
  cut's 5.2 `revision show` entry extends (a template's `resources`); and
  `jig/docs/design/domain/connectors.status.md`, its 6.2 entry, beside this cut's 17 entry. A
  catch-up conflict there is a status merge, both entries kept.
- v0.1.0-ladder's `finding-4` (ready) changes `client-v19-part1.schema`,
  `temper-client-tools/src/jig/rows.rs` and `render/document/proposal.rs`, which
  `settings-wire`, `policy-ctl` and `restore-ctl` change later; those tasks start after it
  lands or meet it at a catch-up. Not avoidable: they are the only homes of the wire and ctl.
- No other file of this plan's tasks is changed by a draft beside it, as far as their trunks
  showed.
