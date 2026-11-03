# configuring-p2: plan

Phase P2 of `temper/docs/design/draft/configuring.execution.md`: settings through ctl, on the
reference's M3. Written by the cut as it releases; each batch brings it up to date.

## Parts

| Part | Slice | Design | Tasks |
|---|---|---|---|
| `policy-item` | 1 | jig's `domain/engine.md` 17.2 and 17.6; `domain/people.md` 5.2; temper's `protocol/configuration.md` 13.1; `client/tools.md` 5.3 | none released (below) |
| `restore` | 2, **the walk** (`walk = true`) | jig's `domain/engine.md` 17.10; temper's `client/tools.md` 5.3 | none released (below) |

The cross-part shapes are `notes/contracts.md` (P1, P2); the standing rules,
`notes/common.md`; the code batch as sketched, `notes/next-cut.md`.

## The design, released with the first push

All of it by the lead, in the sections the request names and no other:

- jig's `domain/engine.md`, 17.2: a project's own policy as an item of a change, set alone,
  at most one, decided by the project's owners and, beside the deployment's items, by one
  party who owns both; checked as a policy projects are made from and as its owners' edits
  are; committed in the change's decision, recorded whole with what it replaced. 17.6: such a
  proposal waits for the parties who own both, its review shows the policy now and proposed,
  a deletion withdraws it. 17.9 and 17.10: their rules in their own words.
- jig's `domain/engine.md`, 17.10: restore takes temper's kinds too (closing the draft
  status's entry one way, the draft's "a new proposal of that set" winning): a connector keeps
  what its items replaced, while the history keeps the revision; a project's policy is
  restored as the rest; a deleted project's items are left out and named.
- jig's `domain/people.md`, 5.2: a project's policy changed whole by a change of settings; a
  deletion withdrawing a proposal naming it.
- temper's `protocol/configuration.md`, 13 (its opening), 13.1 and 13.7: the draft's rules in
  their own words; 13.1's project policy through a change.
- temper's `client/tools.md`, 5.3: the settings file's `project-policy` item; `revision
  restore` taking temper's kinds and projects' policies; `settings propose` to its items'
  owners.
- Statuses: jig's `engine.status.md` (17.2, 17.6, 17.10 Not yet), `people.status.md` (5.2),
  temper's `tools.status.md` (5.3); the draft's `configuring.status.md` entries narrowed to
  what is left (below).

## Why the code waits

P2's code builds on M3's application settings and temper's kinds as settings (jig's `apps-*`,
temper's `kinds-*`). This trunk was made from `reference-m3/integration` at 3db9a0e69a, which
holds 5 of M3's 30 tasks whole (`kinds-values`, `change-call-llm`, `change-jig-check`,
`reference-charters`, `subagents-profile`) and an early commit of `subagents-ctl`; at 03:20Z
M3's `apps-change`, `apps-surface`, `kinds-store` and `subagents-ctl` were running,
`apps-connector` and `kinds-codec` done but not on this trunk, and the rest waiting. fab gives a
task no need on another plan's task, and this trunk takes M3's later commits only by catch-up
once M3 is on main (fab's `docs/design/integration.md`, 3, `after`). Every file P2's code
changes is one M3's in-flight tasks change in place (`jig-domain-core/src/settings/`,
`settings-v1`, client-v19, `jig-domain-people`'s `Setting` and boundary,
`temper-client-tools/src/jig/`), so specs released now would start without the types they
build on and collide with M3's schema and golden changes. The report's Question asks when the
code batch is cut; `notes/next-cut.md` sketches it.

## Critical path, as sketched

`policy-surface` (60) -> `policy-store` (90) -> `restore-records` (120) -> `restore-request`
(150) -> `restore-connector` (150) -> `restore-forge` (150) -> `restore-story` (120): about 840
implementer-minutes from the code batch's release; `restore-connector` also waits for 17.5's
part, written after M3 lands.

## Decided along the way

- **Restore and temper's kinds:** closed one way, not a departure: the draft (3.3, "a new
  proposal of that set") and the walk (each kind restored) settle it. What that asks of 17.5
  and `domain/connectors.md`, 17 (M3's) is written after M3 lands, as the entry says; the
  draft's status names it.
- **A project's policy is no setting:** it keeps no default, revision or pin; a change
  carries it as an item of the project's scope, `set` alone, at most one per change, priced
  at one policy beyond a whole set.
- **jig's `client/domain/contract.md`, 3.2 and 5.1** (a change's items, `Restore`'s answer,
  the review) lag the design: not among the sections the request names; the report asks.
- **Not P2's:** `settings show landing-rule|forge-charter` and a settings file's and a
  review's items of temper's kinds are M3's `kinds-ctl-form` and `kinds-ctl-reads`; the
  walk exercises them, and no task of this plan copies them.

## Overlaps with other plans

- jig's `engine.status.md`: M3's `apps-change`, `apps-proposals` and `apps-start` edit its
  17.5 entry; this cut's lines sit at 7.1's, away from theirs.
- temper's `tools.status.md`: M3's `subagents-ctl` (line 5) and `kinds-ctl-reads` (5.2 and
  5.3's lines) edit it; this cut's 5.3 entry sits before 6.1, away from both.
- No code file of this plan's is changed by this batch.
