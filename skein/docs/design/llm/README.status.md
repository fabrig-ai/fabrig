# llm: status

2026-11-13. Where skein's code lags or departs from [README.md](README.md).

- **Not yet:** 2.5, *A 403 that is not the provider's own*, 4.7 and 4.8: a 403 whose body did not decode as the dialect's error document is `Invalid`, not `Unavailable` (`skein-llm/src/wire/failure.rs`, `classify`) [in flight: change 29].

- **Not yet:** 4.7: `off` refused at admission, so no request carries disabled thinking; a focused test holding each shape current Claude models refuse absent; the fake peer refusing disabled thinking and sampling settings [in flight: change 914].

Section 4.9 has no known implementation gap in the route-specific list
request, bounded parsing, visibility filtering or optional figures.
Later pages remain section 6's deferred work.

- **Not yet:** 5: `skein-live`, the live suite's logins on the routes that
  stay, has no design: the suite calls the public route with a lent login's
  access token and Anthropic with a lent key file, but five of its provider
  stories are ignored, manual stories run by the owner; how the owner runs
  them is for later (section 6).
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
- **Departs:** 3: `skein/docs/foundation/programming-model.md`, 5.4 asks for one
  explicit state enum per lifecycle, each state holding exactly its
  data, no lifecycle in flags or `Option`s beside it, exhaustive matches
  and one handler per cell; `skein-llm/src/client.rs` handles each event
  in one function that matches the state inside, for which no reason is
  recorded, and keeps the SSE reader, which 3's table has under Streaming,
  on `Client` with the other machines, because `close_sse` closes them
  from any state. Beside `State` it also keeps the request's body in `Upload`,
  which the table has under Prepared and Head, because an early response
  leaves the upload live in Streaming and ErrorBody, and `outcome`, because
  ErrorBody stays after its terminal has gone. 4.8:
  the request fields have no text on verbosity, but
  `skein-llm/src/translate.rs`'s `write_request` sends
  `"text":{"verbosity":"low"}` on every Responses request, using
  `openai/request.rs`'s `VERBOSITY`; no reason is recorded. 2:
  `skein/docs/foundation/programming-model.md`, 10.3 puts generic items out
  of step code, but `skein-llm/src/wire/output.rs`'s `Output<P>` is one, as
  this section has it, for the reason it gives. 2: Anthropic's modules name
  nothing of `openai`, but `skein-llm/src/anthropic/request_tests.rs` spells
  `openai::Limits` and `openai::json`, reaching `wire` through `openai`'s
  re-exports, because the request of 2026-11-05 keeps the tests as they are.
  3: an owner may also ask one prepared call to keep its rendered body
  (`MeasuredRequest::keep_body`, `Client::take_kept_body`), a copy held until
  taken and made before any compression, because a development deployment must
  read what left; it comes out when that need ends. Unasked, nothing is kept.
  The body's encoder takes the prompt and limits alone, so a kept body holds no
  header, credential or URL.
