# ui-welcome: the plan

From `inputs/request.md`: the first run, whole, as the greenfield mock draws it. States and acts, no
restyling of what matches. The design is cut ahead of the tasks; the rest of this plan comes in batches.

## Parts and tasks, so far

| # | Task | Request | Needs | Effort, minutes | What it lands |
|---|---|---|---|---|---|
| 1 | `frame` | 2 | | high, 110 | steps as links, `?at=welcome`, "Go on setting up", brand, title |

## Critical path, as it stands

`frame`, then the request's order. Outstanding: Forge, Model, Claude, Ready, the engine's intent and the
sheet, Home, the terminal, the journey and proof.

## Overlaps with other drafts

- `claude-always/fetching-words` touches `temper-client-view/src/accounts/words.rs` and `connect_model`'s
  tests: the Model and Claude tasks, later in this chain, meet it.
- `temper.css`, `tools/gallery/**` and `docs/design/client/journeys/**` meet every look plan: the markers,
  the regenerated prints and one writer at a time here keep it small.
- Formats and protocols change in place, as the request allows: no compatibility is kept.
