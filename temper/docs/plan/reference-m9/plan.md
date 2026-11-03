# reference-m9: plan

M9 of `temper/docs/design/draft/reference-case.execution.md`: a goal branch's own tier, which
judges no landing and sends its reds to the goal as news; a red after landing reaching its goal
as news; the coordinator's triage; and `repairs`, a change naming the red landing it repairs,
which clears it.

## The design this cut wrote

- `temper/docs/design/domain/after-landing.md`: 1 and 2 (a tier following goal branches, a
  tier's span), 3 (on a goal's branch), 5.4 (no verdict on a goal's branch), 6 (cleared at
  once where only such tiers follow), 7 (a member's goal, the news's words, `repairs` and its
  clearing, a goal branch's red), 10 (the store), 11, 12 (the stories); its status's Not yet.
- `temper/docs/design/domain/agent.md`, 4.3.1: `repairs`; its status's Not yet.
- `temper/docs/design/protocol/configuration.md`, 3 (`branch` `goal`, `span_ms`) and 13.9
  (the reference's tiers, charter 5's triage, the notes family); its status's Not yet.
- Closed in the drafts' statuses: `reference-code-production.status.md`'s g's tier entry and
  `repairs` in its `change` entry; `reference-landing.status.md`'s "a repair names its red".

The binding shapes are `notes/contracts.md`; the standing rules `notes/common.md`.

## Parts and tasks

| Part | Task | Needs | State |
|---|---|---|---|
| surface | `store-surface`: v11's `ChangeRow.repairs`, `TierRepair.by`, `TierGoal`, `TierRow.goals/goal/started`; `landing_red` | | released |
| surface | `red-news-surface`: `News::Red`, `RedNews`, the root's words | | released |
| tier-config | `goal-tier-config`: `Follows`, `span`, `"branch": "goal"`, `"span_ms"`; goal tiers inert | | released |
| charter | `charter-triage`: charter 5's triage and notes paragraphs | | released |
| repairs | `repairs-call`: `PLAN_CHANGE`'s `repairs`, the root's check, parameter 12 kept | store-surface | released |
| repairs | `repairs-verdict`: `TierRepair.by`, "repaired by" in the result | store-surface | released |
| red-news | `red-news-tell`: the logs' ends read, each goal of a red landing told | store-surface, red-news-surface | released |

## Critical path, as it stands

`store-surface` → `repairs-call` / `repairs-verdict`; `red-news-surface` → `red-news-tell` →
`goal-tier-forge` → the stories. `goal-tier-forge` is the longest single task.

## Overlaps with the plans beside this one

- `temper/crates/temper-protocol-store/schema/store-v11.schema` and its generated code and
  goldens: reference-m3, reference-m5, reference-m6 and first-run-projects append there too.
  This plan edits the store in one task alone (`store-surface`), first, to keep its own merges
  to one.
- `temper/crates/temper-engine-domain/src/route/forge_route.rs` and `calls.rs`: reference-m3,
  m5, m6 and m7b. This plan's edits there are `news_words`, `change_batch`'s `repairs` check,
  `chat_change`/`start_change`'s parameter 12 and `change_verification_words`.
- `temper/crates/temper-engine-domain-forge/src/boundary.rs`: three tasks of the first batch
  write it side by side (each its own types: `ChangeRow`/`Tier*`, `News`, `Tier`/`Follows`), as
  do other plans; unavoidable, since every forge type lives there.
- `temper/docs/examples/reference/settings.json` and its test: reference-m5's `reference-relay`.
- `temper/docs/help/change.md` and `tier.md`: reference-m3, m5 and m7b edit `change.md`; this
  plan's help task comes last.
