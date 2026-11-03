# configuring-p4: plan

Phase P4 of `temper/docs/design/draft/configuring.execution.md`: stage 1 of configuring's
acceptance (`temper/docs/design/draft/configuring.md`, 8.1), built on main (P1 to P3 and the
reference's M1 to M10 landed). Released in batches; this file is brought up to date with each.

## Parts

| Part | Slice | Design | Tasks |
|---|---|---|---|
| `golden` | 1 | temper's `domain/testing.md` 6.5 | `stage-one-surface` (batch 1) |

The cross-task shapes are `notes/contracts.md` (K1 the crate's layout, K2 the golden, K3 the
evidence, K4 the check's report, K5 the commands, K6 the judge, K7 collect); the standing rules,
`notes/common.md`.

## The design this cut wrote (batch 1)

- temper's `domain/testing.md`, **6.5** (new, beside 6.4, the execution file's default home):
  the golden as `reference-code-production.md`, section 14 cut into obligations (a setting by
  one of eight tests, the box's, a gap), the profile `docs/examples/reference/` as its values;
  the roles and how the check binds them to the proposal's numbers; the box (6.5.2); the
  evidence a pass collects (6.5.3); the check, the judge and the stage's verdict (6.5.4); and
  section 7's layout line for `tools/stage-one`.
- Statuses: `testing.status.md`, a 6.5 Not yet; the draft's `configuring.status.md`, its 8.1
  Design lags entry closed (8.2's kept), its stale 2 entry closed (N1, N3, N4, N19 and N23 are
  designed since M1, M3 and M5: `domain/agent.md` 4.3.1, `domain/forge.md` 8.11), and two
  **Open** entries proposing the figures that are the user's (the box's limits, the walk's
  bounds); `reference-code-production.status.md`, a 14 Design lags entry: the coordinator's
  decide, propose, subscribe and recall families, which 13.9 does not grant.

## Decided along the way

- **The check is a Rust tool crate,** `temper/tools/stage-one`, not a script: the gate runs its
  step tests (a Python tool's tests run in no gate step), it reads the settings form's `FIELDS`
  to hold the golden's members to the form, and nothing of it reaches temper's binary.
- **The golden's values are the profile's;** its obligations say which fields count and by what
  test. A member no obligation names is free (budgets and times stay the box's).
- **Roles, not numbers:** the proposal numbers its charters, so the check binds roles by anchors
  (the goal template's executor, forge charter 1, the foremen's fixed charter 6, the goal-branch
  resource, the landing rules' last segments) and the rest by the assignment meeting most.
- **The survey's corrections:** forge charter 3 is not built (the forge's foremen run under the
  fixed charter 6); temper's kinds 4 to 6 are not built, so the tiers, the merge job and the
  question bound are the box's document; "Claude Code" is the document's model's program, the
  box's; no gap is left with M1 to M10 landed.
- **The walk's evidence is collected through ctl's reads** (`temper-stage-one collect`), the
  documents and the person's files by the pass from the forge; the judge is a session of its
  own reading a brief the tool prints, its verdict a JSON file the tool decodes.

## Batches

1. `stage-one-surface` (interface, 150): the crate, the golden's types and decoder, the golden
   file whole, the settings file's and the evidence's readers, `obligations`; stubs for the rest.

## Critical path (as it stands)

`stage-one-surface` (150), then the check, collect and the judge (batch 2).

## Overlaps with other plans

- The root `Cargo.toml` (one workspace member line) and `Cargo.lock`:
  `playground-browser/browser-walk` edits both. One line; a conflict is a lock regeneration.
- `.config/gate/always.txt`: one line.
- `temper/docs/design/domain/testing.md` and `testing.status.md` are this plan's alone among
  the drafts beside it; `configuring.status.md` and `reference-code-production.status.md` too.

## Left for others

- `temper/docs/design/protocol/configuration.md`, 13.2 designs the forge's charter 3, its
  foremen's, which the code fixes at charter 6 (`temper-engine-domain/src/route/forge_route.rs`,
  `FOREMAN_CHARTER`) with kind 3's `per_scope` 2; its status does not record it. Left to
  `configuration.status.md`'s owner (fabrig-scale edits that file now).
- 13.9's table of the profile's limits differs from `docs/examples/reference/limits.json`
  (`tasks.delegates`, `people.operations`, `tasks.result_bytes`, `tasks.batch`, `tool_payload`);
  the golden takes the file as the box's.
