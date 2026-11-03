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

## Slices

1. subagents.

## Critical path, as it stands

To come with the designers' batches: appsettings' surface, then its core, then the kinds'
routing through the root.

## Overlaps with other plans

- `subagents-ctl` changes `temper-client-tools`' chat start (`jig/rows.rs`, `runner/acts.rs`,
  `docs/help/reference/ctl-chat.md`), which configuring-p1's `ctl-chat-template` changed and
  which is already on this trunk: no overlap left in flight.
