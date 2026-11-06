# model-catalogue-2: the plan

Step 2 of `temper/docs/design/draft/model-catalogue.md`, section 7: its sections 4 and 6, and the page on Settings. The cut is writing the rest; this file grows with each push.

## Parts and tasks so far

| Part | Task | Needs | Minutes | What |
|---|---|---|---|---|
| `page` | `models-tab` | | 90 | the read-only tab, from `me` |
| `bounds` | `fleet-bounds` | | 120 | a hello names endpoints, not models |
| `agent` | `smith-limits` | `fleet-bounds` | 240 | the agent holds no list |
| `setting` | `model-surface` | | 200 | the kind and its value, inert, everywhere it's matched |
| `catalogue` | `catalogue-core` | `model-surface`, `fleet-bounds` | 300 | the table from settings; a charter's resolution; versions |
| `catalogue` | `catalogue-store` | `model-surface` | 90 | names at open and on each change |
| `catalogue` | `catalogue-clients` | `model-surface`, `catalogue-store` | 100 | the clients' component; a root's route |
| `catalogue` | `config-models` | `model-surface` | 120 | passed models; a provider's endpoint; the bound of 64 |
| `act` | `model-act` | `catalogue-core`, `catalogue-clients` | 150 | `ChangeModel`; `me`'s facts |

## Critical path so far

`model-surface`, then the catalogue going live, then temper's stock, as the rest is released.

## Beside it

- **`look-round-11` isn't on main** at this cut (2026-11-06): its All models link stays on Accounts, and moves at catch-up or in this plan's closing task.
- `model-check` is on its way too: a print conflict is regenerated, not merged.
