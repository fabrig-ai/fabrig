# model-catalogue-2: the plan

Step 2 of `temper/docs/design/draft/model-catalogue.md`, section 7: its sections 4 and 6, and the page on Settings. The cut wrote the design: jig's `domain/engine.md`, 17.1, 17.7, 17.8 and the new 17.11; jig's `protocol/configuration.md`, 3, `protocol/store.md`, 5, `protocol/channel.md`, 5.1, `domain/hosts.md`, `domain/tasks.md`, 3, `client/domain/contract.md`, 3.2 and 5.1; smith's `protocol/agent.md`, 4, `charter.md`, 2, `limits.md`, 2.1, 2.2, 3.9 and 4; temper's `protocol/configuration.md`, 2, 13.1, 13.4 and 13.8, `protocol/setup.md`, 8.3, `domain/agent.md`, 8.1, and `client/README.md`, 3.5, *The `Models` tab*. The contracts between parts are `notes/contracts.md`.

## Parts and tasks

| Part | Task | Needs | Minutes | What |
|---|---|---|---|---|
| `page` | `models-tab` | | 90 | the read-only tab, from `me` |
| `bounds` | `fleet-bounds` | | 120 | a hello names endpoints, not models |
| `agent` | `smith-limits` | `fleet-bounds` | 240 | the agent holds no list; `agent.json`'s models go |
| `setting` | `model-surface` | | 200 | the kind and its value, inert, everywhere it's matched |
| `catalogue` | `catalogue-core` | `model-surface`, `fleet-bounds` | 300 | the table from settings; a charter's resolution; versions |
| `catalogue` | `catalogue-store` | `model-surface` | 90 | names at open and on each change |
| `catalogue` | `catalogue-clients` | `model-surface`, `catalogue-store` | 100 | the clients' component; a root's route |
| `catalogue` | `config-models` | `model-surface` | 120 | passed models; a provider's endpoint; the bound of 64 |
| `act` | `model-act` | `catalogue-core`, `catalogue-clients` | 150 | `ChangeModel`; `me`'s facts |
| `stock` | `stock-models` | `catalogue-core`, `catalogue-store`, `catalogue-clients`, `config-models`, `smith-limits` | 300 | seven and the scripted one in the binary; `temper.json`'s models go |
| `stock` | `process-fixtures` | `stock-models` | 80 | the slow tiers' documents, compiled, not run |
| `configuring` | `configuring-learns` | `stock-models`, `model-act` | 110 | the chat's tools, instructions, card and `temper ctl` |
| `page` | `models-switch` | `models-tab`, `model-act` | 110 | the switch; "added by you"; "estimated prices"; left out |
| `page` | `models-add` | `models-switch` | 80 | "Add a model by name" |
| `help` | `models-help` | `stock-models`, `models-add` | 45 | "Offering another model" |
| `page` | `models-story` | `models-add`, `stock-models` | 45 | switch and add stories in the client world, by the task's answer |
| `proof` | `models-proof` | everything it closes | 80 | reprints, the account and status files; render check after landing |

## Critical path

`model-surface` (200), `catalogue-core` (300), `stock-models` (300), `configuring-learns` (110), `models-proof` (80): about 990 implementer-minutes. `fleet-bounds` then `smith-limits` runs beside it and joins at `stock-models`. Beside them, `models-tab` is on its own from the first push.

## What the cut decided

- **A number is a position.** Model `i` is the setting numbered `i + 1`, for the deployment's life, within `MODELS`, 64. It keeps the store's and the wire's indices whole across a change, and so spares every holder of an index. `Settings.models` holds a hole for a number no setting has.
- **A person's model counts down from 64,** so a release's new default, counting up, never meets one.
- **A removal withdraws.** The setting stays, out of the offer and every resolution, so a number is never given to another name and a task keeps its model. This departs from jig's 17.1 for this kind, and `draft/model-catalogue.status.md` says so.
- **A charter resolves.** Its value keeps its first choices and `rest`; what a run reads, the fleet needs and the tasks child checks is the resolution, recomputed with the catalogue. A change of a resolution makes a charter version, though no item names the charter, so a pinned task keeps its model. A picker's flip makes none.
- **`ChangeModel`,** one request, for the switch and for adding: the core has every whole value and the newest revision, so a client holds neither and can't go stale. Its decision is `ChangeSettings`'.
- **An endpoint's `limits`** replace an agent's list: one declaration per endpoint, so a model added while an agent runs reaches it with no restart. A saved window's model the charter doesn't name takes main's quantities for its compaction. `charter_models` bounds what `run.models` was derived from.
- **A hello names endpoints,** and the fleet judges a model by its endpoint's bound, so a worker needs no list either.
- **A provider names its `endpoint`,** since `Endpoint.provider` is a dialect's backend and `Provider.models` goes. The stock keys its rows by provider number, so a fake or an owned document's endpoint serves the stock's names.
- **The store reads the `model` changes at open,** since a stored vocabulary would otherwise read a person's model as gone, and it remaps nothing it holds no name for.
- **The alternatives' bound rises to 63** in both profiles and the wire, so that the rest reaches `MODELS`. `run_bytes` can still stop a resolution first; that's what `left_out` counts until step 3.
- **The address is under the project,** `/projects/<project>/settings/models`, since the offer is a project's; 07 draws `/models`. "the default without Claude" isn't drawn: `me` names one model for a template. Both are the account's rows.
- **A stored format, a protocol and a public interface** all change, in place, with no version bump (the request): the settings, people and tasks families' schemas, the client's wire, the workers' wire, `agent.json`, `temper.json`.

## What waits for step 3

`model-catalogue-3`, section 5, reading each provider's list: "Read again", "Couldn't read OpenAI's list · 2 hours ago", "new", a section's clause on where its list came from beyond temper's own, and the per-account record of names. The seams left plain: `ModelSource::List`, `CatalogueModel.withdrawn`, `settings::catalogue::new_model`, `left_out`. `model-catalogue.status.md` says so.

## Overlaps with drafts beside this one

Not avoidable. The order keeps early work off `look-round-11`'s and `ui-proposal-card`'s files where it can: the jig tasks touch their files only for a literal's new field.

- **`look-round-11`** is on main when `models-proof` starts. That task moves All models' link to the picker's `Models { project }`, with "Choose what the list shows, in Settings", and updates its view test and client design. `temper.css` and `choice.rs`: `models-tab` makes two functions `pub(crate)`, and new rules are under the settings marker. The gallery conflicts are regenerated.
- **`ui-proposal-card`:** `client-v19.schema`, `settings.rs` of the client's contract, `tasks-v9.schema`, `cards/proposal.rs`, `words.rs`, `README.status.md`. `model-surface` and `configuring-learns` are the hunks; generated code and goldens are regenerated at catch-up.
- **`model-check`:** `stock.rs` (its trial charter takes `rest: true` at catch-up), `client-wire-v1.schema`, `pages/accounts.rs`, `words.rs`, and `chat-draft`'s prints, regenerated, not merged.
- **`browser-budget`:** `browser_pages.rs`; `models-story` has a file of its own.
- **`ui-public-history`:** `words.rs`, `temper/tests/client-browser`.

## Slices

No walk: `walk = false` throughout, since only focused and fuzzy run in the plan. The `models-story` answer places switches and adding across reloads in the client world; the existing browser suite is compiled, not run. `models-proof` accounts for the page and reprints the journeys without a render driver. The user's answer of 2026-11-06 defers page 07's render comparison and screenshots at 1440, 1280, 1000 and 420 px to temper/ui after landing.
