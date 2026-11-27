# pair: status

2026-11-27. Where the design lags or departs from [pair.md](pair.md).

- **Design lags:**
  - 3.1, 3.2: smith's `domain/run.md`, section 1, and `domain/run-messages.md`, section 1: a run is Finite or Open, never both; a Finite run ignores `finish_requested`. `../domain/agent/tools.md`: a change's producer talks to its requester in questions alone, and no producer task holds a conversation.
  - 3.3: smith's `domain/run-delivery.md`, 1.5: a run saves only when its LLM calls `save`.
  - 3.4: `../client/ctl/tools-acts.md`, section 1: `task message` has no `--finish`.
  - 3.6, 4: [../protocol/configuration/stock.md](../protocol/configuration/stock.md) has no `pair` charter, and [ways-recipes.md](ways-recipes.md) no connected producer yet built.
- **Design departs:** none.
