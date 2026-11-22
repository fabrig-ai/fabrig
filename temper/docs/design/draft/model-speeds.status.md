# Model speeds: status

2026-11-21. Where the design lags or departs from [model-speeds.md](model-speeds.md).

- **Design lags:**
  - 3: jig's `protocol/configuration.md`, section 3, and jig's `domain/engine/settings.md`, sections 1 and 11: a model has one set of prices and no speeds.
  - 4: jig's `domain/tasks/README.md`, section 3: a choice is a model and an effort. [areas.md](../client/ctl/areas.md) has no `--speed`.
  - 5: smith's `protocol/charter.md`, section 2: an LLM has no speed. skein's `llm/README.md`, 4.8, sends no `service_tier`.
  - 6: smith's `domain/session.md`, section 6: an LLM has one set of prices. skein's `llm/README.md`, 2.3: a completion carries no served tier.
  - 7: [stock.md](../protocol/configuration/stock.md) has no gpt-6.2-sol.
- **Design departs:** none.
- **Open:**
  - 7: Ultrafast's stock price, the user's figure: six times standard, its list price from 2027-03-01, taken for now, which overcounts by half until then; or three times, its price until 2027-02-28, which matches what's paid now but needs every model carrying it, and the work holding it, repriced on that date.
