# llm: status

2026-11-03. Where skein's code lags or departs from [llm.md](llm.md).

- **Departs:** 5: five of the live suite's provider stories are ignored, manual
  stories run by the owner; how the owner runs them is to be decided later.
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
