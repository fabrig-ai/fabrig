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

## Tasks so far

Released (needs in brackets):

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
  asks to the forge, typed).
- change-temper: `change-call-llm` [] (`ChangeCall.producer` and `llm` decoded, refused until
  the root checks them); `change-row-llm` [kinds-store, change-call-llm] (the row's
  `producing_llm` and `review_llm`, store-v11 in place); `change-root` [change-jig-check,
  change-call-llm, change-row-llm] (the root checks `producer` and `llm`, the procedure's
  authority, parameter 11); `change-story` [change-root] (the engine world's story).

Written, held for the change family's batch: `reference-charters` (charter 6, the designer's;
charter 5's second edition; their documents). configuring-p1, whose charter documents it
names, landed on main at 01:39Z and is on this trunk.

## Slices

1. subagents.
2. appsettings.
3. kinds-forge.
4. change-jig and change-temper.
5. kinds-root.

## Critical path, as it stands

`apps-surface` (90) -> `apps-change` (150) -> `apps-proposals` (100) -> `apps-wire` (140) or
`apps-start` (110) -> `apps-world` (120): about 610 implementer-minutes for jig's side; the
kinds' routing through the root (to come) follows `apps-start`. The forge's side
(`kinds-values` 90 -> `kinds-store` 150 -> `kinds-held` 120 -> `kinds-requirements` 100, 460)
runs beside it.

## Left for later in this cut

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
- `kinds-route` and `change-root` both change `temper-engine-domain`'s `route/forge_route.rs`
  and may run at once: `kinds-route` one ask arm, `change-root` `change_batch`,
  `chat_change` and `start_change`; accepted as disjoint hunks.
- `apps-surface` (one arm) and `change-call-llm` (`change_batch`) both change `forge_route.rs`
  and run at once: disjoint hunks, accepted.
- `apps-surface` puts compiler-forced arms in temper's `temper-engine-domain`
  (`route/forge_route.rs`, `store/settings_bytes.rs`) and `temper-engine-smith`: arm-only hunks,
  beside the change family's `forge_route.rs` work and reference-m2's.
