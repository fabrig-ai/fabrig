# reference-m3: plan

Milestone M3 of `temper/docs/design/draft/reference-case.execution.md`: designers and the cut
(N12, N13, N16, N19, N20). Written by the cut as it releases; each batch brings it up to date.

## Families and parts

| Family | Parts | Design | Written by |
|---|---|---|---|
| subagents | `subagents` | temper's `protocol/setup.md` 8.1, `client/tools.md` 5.5 | the lead |
| appsettings | `appsettings` | jig's `domain/engine.md` 17.5 to 17.7, `domain/connectors.md` 17 (designed) | its designer |
| kinds | `kinds-forge`, `kinds-root` | temper's `protocol/configuration.md` 13.2, 13.3, 13.6 step 2; `domain/root.md` 5.5, 5.6, 6; `domain/forge.md` 8.9 (designed) | its designer |
| change | `change-jig`, `change-temper` | temper's `domain/agent.md` 4.3.1, `domain/forge.md` 8.1, 8.9, `protocol/configuration.md` 13.2; jig's `domain/tasks.md` 4 | its designer |
| reference | `reference` | temper's `protocol/configuration.md` 13.9; jig's `domain/core.md` 7.1 | the lead, last |

The cross-family shapes are `notes/contracts.md` (C1 to C6); the standing rules,
`notes/common.md`.

## Tasks

All 30 released (needs in brackets):

- subagents: `subagents-profile` [] (the owned example's agent declares `conversations` 3);
  `subagents-ctl` [] (`chat start --sub-agent MODEL[:EFFORT]`).
- appsettings: `apps-surface` [] (C1's and C2's types, `Config.kinds`, the asks and events;
  compiler-forced arms only, temper's among them); `apps-change` [surface] (a change of an
  application's items judged by its connector and kept); `apps-connector` [surface]
  (`jig-test-connector` owns a kind); `apps-proposals` [change]; `apps-start` [proposals]
  (defaults at a start, the release's revision); `apps-store` [proposals] (settings-v1,
  people-v14, tasks-v9 in place); `apps-wire` [proposals] (client-v19 in place);
  `apps-world` [connector, start, store] (the core world's story); `apps-proposal-key`
  [proposals, store, wire] (a settings proposal's deployment-unique key, sent as C2's
  `proposal`: a revision of what `apps-proposals`, `apps-store` and `apps-wire` build).
- kinds-forge: `kinds-values` [] (the forge's values of kinds 1 to 3, `ForgeCharters`, its
  limits); `kinds-store` [values] (the forge's family `Setting`, store-v11 in place);
  `kinds-held` [store] (judging, keeping, holding, `charters()`); `kinds-requirements` [held]
  (requirements 5 and 6 over the held rules).
- change-jig: `change-jig-check` [] (a batch's LLM choice against its charter's models, every
  creator).
- kinds-root: `kinds-codec` [kinds-values, change-call-llm] (a kind's value in temper's codec);
  `kinds-route` [kinds-held, kinds-codec, apps-surface, change-call-llm] (the root routes C2's
  asks to the forge, typed); `kinds-start` [kinds-route, apps-start] (the stock declares kinds
  1 to 3 and gives the forge its defaults at a start); `kinds-switch-rules` [kinds-start,
  kinds-requirements, change-root, change-story] (the stock rules' requirements 5 and 6 read the held
  rules, nothing compiled); `kinds-switch-charters` [kinds-switch-rules, change-root,
  change-story] (`Config.forge_charter` gone, the change start reads `charters()`, a change
  held when a role's charter or choice is no longer offered: the change family's
  `change-held` folded in); `kinds-ctl-form` [kinds-route, kinds-codec, apps-wire] (the
  settings form's `landing-rule` and `forge-charter` items, the entrances' typed values);
  `kinds-ctl-reads` [kinds-ctl-form, apps-proposal-key, kinds-switch-charters] (the forge's
  family reads and ctl's `settings show`); `kinds-document` [kinds-switch-charters, kinds-ctl-reads,
  reference-charters] (the document refuses `application.landing`'s rules and `charter`;
  every writer moved, `landing.json`'s rules into the reference's `settings.json` as kind 1;
  the help pages); `kinds-story` [kinds-document, change-story, apps-proposals] (the domain
  world's stories: a rule change reaching the next merge, a refused rule, a restart, a
  project's rules erased).
- reference: `reference-charters` [] (charter 6, the designer's; charter 5's second edition;
  their documents and sub-agents), released last as the request asks.
- change-temper: `change-call-llm` [] (`ChangeCall.producer` and `llm` decoded, refused until
  the root checks them); `change-row-llm` [kinds-store, kinds-route, change-call-llm] (the row's
  `producing_llm` and `review_llm`, store-v11 in place); `change-root` [change-jig-check,
  change-call-llm, change-row-llm] (the root checks `producer` and `llm`, the procedure's
  authority, parameter 11); `change-story` [change-root] (the engine world's story).

configuring-p1, whose charter documents the reference names, landed on main at 01:39Z and is
on this trunk.

## Slices

1. subagents.
2. appsettings.
3. kinds-forge.
4. change-jig and change-temper.
5. kinds-root.
6. reference, **the walk** (`walk = true`): reference-case.md steps 1 to 6 and 22, and M4's
   walk (a chat's two sub-agents side by side), through `temper ctl`.

## Critical path, as it stands

`kinds-values` (90) -> `kinds-store` (150) -> `kinds-held` (120) -> `kinds-route` (120) ->
`change-row-llm` (90) -> `change-root` (110) -> `change-story` (90) -> `kinds-switch-rules`
(150) -> `kinds-switch-charters` (120) -> `kinds-ctl-reads` (150) -> `kinds-document` (150) ->
`kinds-story` (120): about 1,460 implementer-minutes. jig's side (`apps-surface` 90 ->
`apps-change` 150 -> `apps-proposals` 100 -> `apps-start` 110 / `apps-wire` 140, about 480)
reaches it at `kinds-start` and `kinds-ctl-form`, well before they are needed. The walk waits
for the whole.

## For the walk's document

- Charter 6 is the deployment's fifth charter: the walk's document sets jig's
  `tasks.charters` to 5 (temper's `protocol/configuration.md`, 13.9), with
  `authority.executors` and `tasks.executor_kinds` at 8 as M1's walk had them.
- The box's agent configuration declares `conversations` 3 (`subagents-profile`); a box made
  before keeps 1 until its `agent.json` is edited.
- Once `kinds-document` lands, the landing rules are kind 1's items of the reference's
  `settings.json`, applied with `temper ctl settings apply`; the document's
  `application.landing` keeps its projects' ways alone, and `landing.json` is gone.
- The documents name the forge's host and repository as placeholders (`127.0.0.1:3000`,
  `acme/widget`), which the walk fills in as it does for the landing rules.

## Decided along the way

- **A settings proposal's identity** (C2's `proposal`): jig keeps settings proposals by
  proposer and number, so the number is not unique across proposers; `apps-proposal-key`
  gives each a deployment-unique key. It and `apps-world` may run at once with one-line field
  edits in `jig-test-domain` and `jig/tests/core`: accepted.
- **`change-held`** (forge.md, 8.9's hold when a role's charter or LLM choice is no longer
  offered or allowed) folds into the kinds family's switch of the forge's charters.

## Overlaps with other plans

- `subagents-ctl` changes `temper-client-tools`' chat start (`jig/rows.rs`, `runner/acts.rs`,
  `docs/help/reference/ctl-chat.md`), which configuring-p1's `ctl-chat-template` changed and
  which is already on this trunk: no overlap left in flight.
- configuring-p1 landed on main before this cut began; `notes/common.md` names it among the
  plans beside this one, which no longer holds: its files are on this trunk.
- `kinds-store` and the change family's `change-row-llm` both change temper's store-v11 in
  place and the forge's `boundary.rs`: `change-row-llm` follows `kinds-store`.
- `kinds-switch-rules` renames `Config.landing` in the domain world's builders, two of which
  `change-story` owns: it follows `change-story` (a `needs` line added before it started),
  about 90 minutes on the critical path.
- `kinds-route` and `change-root` both change `temper-engine-domain`'s `route/forge_route.rs`
  and may run at once: `kinds-route` one ask arm, `change-root` `change_batch`,
  `chat_change` and `start_change`; accepted as disjoint hunks.
- `apps-surface` (one arm) and `change-call-llm` (`change_batch`) both change `forge_route.rs`
  and run at once: disjoint hunks, accepted.
- `apps-surface` puts compiler-forced arms in temper's `temper-engine-domain`
  (`route/forge_route.rs`, `store/settings_bytes.rs`) and `temper-engine-smith`: arm-only hunks,
  beside the change family's `forge_route.rs` work and reference-m2's.
