# llm: status

2026-11-03. Where skein's code lags or departs from [llm.md](llm.md).

- **Departs:** 5: five of the live suite's provider stories are ignored, manual
  stories run by the owner; how the owner runs them is for later (section 6).
  `anthropic_streamed_text_and_history`, `anthropic_tool_round_trip` and
  `anthropic_unauthorized` need an Anthropic API key: none is provided or wanted
  for the automatic suite, by the user's decision of 2026-11-03.
  `openai_streamed_text_and_history` and `openai_tool_round_trip` need a login
  whose access token holds the public route's scope, `api.responses.write`: the
  Codex login lent to the suite lacks it and is refused (`401 Missing scopes:
  api.responses.write`), and no lent login serves the route in a form the suite
  reads, by the director's decision of 2026-11-03, which the user takes.
  `openai_unauthorized` stays in the automatic suite: it sends the lent token
  replaced by an invalid one.
  3: `skein/docs/foundation/programming-model.md`, 5.4 asks for one
  explicit state enum per lifecycle, each state holding exactly its
  data, no lifecycle in flags or `Option`s beside it, exhaustive matches
  and one handler per cell; `skein-llm/src/client.rs` keeps a data-less
  `State` beside `Client`'s state-specific fields and decides
  transitions by state comparisons; no reason is recorded (the code's
  side is `skein/docs/development/debt/D023-llm-client-states.md`). 4.8:
  the request fields have no text on verbosity, but
  `skein-llm/src/translate.rs`'s `write_request` sends
  `"text":{"verbosity":"low"}` on every Responses request, using
  `openai/request.rs`'s `VERBOSITY`; no reason is recorded.
