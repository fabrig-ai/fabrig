# claude-api: status

2026-11-11. Where the design lags or departs from [claude-api.md](claude-api.md).

Nothing of this draft is designed or built yet. smith's direct backend and its Anthropic dialect, jig's keys and `ctl`'s `add-key` are there; what follows is what step 1 changes.

- **Design lags:**
  - 3: `../protocol/setup/init.md`, section 3 writes `anthropic` offering Claude Code's program alone, through one endpoint that names it; jig's `protocol/configuration.md`, section 3 and `../protocol/llm.md`, section 3 give a provider one endpoint.
  - 4: jig's `domain/accounts.md`, 3.4 already orders an Anthropic account's Claude Code then a key, but no design says which endpoint a key's run takes when its model's endpoint names a program, and the code keeps the two kinds apart.
  - 5: `../protocol/configuration/stock.md` prices cached input as input and a cache write not at all.
  - 6: `../client/pages/accounts.md`, section 2 words OpenAI's key alone; `../../help/account.md` names no Claude key.
  - 7: skein's `llm/README.md` says nothing of what Opus 5.5, Sonnet 5.5 and Fable 5.1 refuse, nor of a history that only grows.
- **Design departs:** none yet.
- **Open:** 8.2: which of step 2's parts are in v0.1.0, the user's once step 1 has landed.
