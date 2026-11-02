# reference-m1: plan

Milestone M1 of `temper/docs/design/draft/reference-case.execution.md`: the goal's own
branch g (N3), `change`'s landing branch, dependencies and batch (N22 in part), the light
rules (N4), a result carrying its producer's report (N15), a charter's default wake policy
(N23), the landing rules as settings, and the reference's first editions. Written by the
cut as it releases; each batch brings it up to date.

## Families and parts

| Family | Parts | Design | Written by |
|---|---|---|---|
| bound | `bound` | jig's `domain/tasks.md`, 11 (already designed) | the lead |
| goal | `goal-jig`, `goal-forge` | jig's `protocol/configuration.md` 3.1, `domain/connectors.md` 3.3 and 4.6, `domain/tasks.md` 6.1 to 6.5, `domain/engine.md` 7.1; temper's `domain/forge.md` 2, 11, 11.1 | its designer |
| change | `change` | temper's `domain/agent.md` 4.3.1; `domain/forge.md` 8.1, 8.2, 8.7, 10 | its designer |
| settings | `settings-landing`, `settings-wake`, `settings-reference` | temper's `domain/root.md` 5.5, 5.6, 6, `domain/forge.md` 8.9, `protocol/configuration.md` 13.2, 13.3 (designed); jig's `domain/tasks.md` 8.3 and `protocol/configuration.md` 3; the reference's configuration | its designer |

The cross-family shapes are `notes/contracts.md` (C1 to C8); the standing rules,
`notes/common.md`.

## Tasks so far

- `bound-verdict-fields` (bound): `VERDICT_FIELDS = 512`, a verdict's fields counted apart
  from `result_bytes`; ported from jig's old `upgrades-6/j12-verdict-fields-2`. Needs none.

## Critical path, as it stands

The goal family's jig surface (a template's own resources, a hold's taking effect), then the
forge's g, then the reference's template for drafts and the walk slice. The change family
runs beside it from the trunk; the settings family's rules-as-settings beside both.

## Overlaps with other plans

- reference-m7a changes `jig-domain-core`, `jig-domain-tasks` and `jig-protocol-store` on its
  trunk; `bound-verdict-fields` touches each in a few lines (`limits.rs`, `verdict.rs`,
  `activity.rs`, `codec/settings.rs`): unavoidable, since the bound lives there.
