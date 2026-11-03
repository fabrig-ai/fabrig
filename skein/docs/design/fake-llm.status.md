# fake-llm: status

2026-11-03. Where skein's code lags or departs from [fake-llm.md](fake-llm.md).

- **Departs:** 4: `skein/docs/foundation/testing-strategy.md`, 4 asks
  for no shared types, but this section designs a shared codec:
  `skein-fake-llm-protocol/src/documents.rs` uses `skein-llm`'s
  `anthropic` and `openai` document types and codecs. No reason is
  recorded for departing from the foundation; the backfill recommends
  amending `skein/docs/foundation/testing-strategy.md`, 4, settled when
  `skein/docs/development/debt/D043-browser-kit-json-and-tables.md` is
  planned.
