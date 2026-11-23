# Rate limits' waits

Provisional, 2026-11-22. How long a provider's rate limit says to wait,
as the client reads it into `Failure::RateLimited { retry_after }`
(`README.md`, section 2.5). The client still retries nothing: how its
owner waits is the owner's policy (smith's `domain/session.md`, section
4). OpenAI's tokens-per-minute refusal can come inside a stream whose
head said 200, its only figure in its message's words ("Rate limit
reached for … on tokens per min (TPM): Limit 30000, Used 25000,
Requested 8000. Please try again in 6.5s."), so a delay read from the
head and the error's fields alone is zero, and its owner retried at
once, inside the same minute (dev.fabrig.ai, 2026-11-20).

## 1. In one page

- **A rate limit is read from what its provider said, whatever the
  status.** `RateLimited` is decided by the error's code, else its type,
  before any status (`README.md`, sections 4.7 and 4.8): a 429's body
  and a `response.failed` or `error` event after a 200 head are read
  alike, and the evidence keeps the head's status.
- **Its delay is the first of:**
  1. the head's `retry-after`, in whole seconds;
  2. the error's `resets_in_seconds`, then its `resets_at` less the
     wall's seconds;
  3. the figure its message gives after `try again in` (section 2);
  4. the window its message names its limit by: `per min` is a minute
     (section 2);
  5. else zero.

  Items 1 and 2 keep their order (`RateLimit::reset`, unchanged); 3 and
  4 are read only where 1 and 2 give nothing. Both routes read them
  alike: Anthropic's message names its window too ("tokens per
  minute"), and its `retry-after` comes first where it sends one.
- **Never a guess.** The figure and the window are the provider's own
  words. Text that does not read as one gives none, and a figure the
  detail limit clipped away gives none, so zero still says the provider
  gave nothing (`README.md`, section 2.5).
- **Exhausted is unchanged.** Its reset is still only the response's
  `resets_in_seconds`, `resets_at` or `retry-after`, never a message's
  words (`README.md`, sections 2.5 and 4.8).

## 2. Reading the message

The message is the error's `message` as the dialect decoded it, clipped
to the detail limit (`ProviderError.message`, `README.md`, section 2.5),
when the failure is classed. ASCII letters match in either case; no
other byte matches but itself.

- **The figure** follows the first `try again in` and one or more
  spaces. It is a duration as Go writes one, which is how the Responses
  route writes it: one or more parts, each a decimal number, with an
  optional fraction after a `.`, and its unit, `h`, `m`, `s` or `ms`,
  with nothing between: `6.5s`, `120ms`, `1m30s`, `2h0m4.25s`. A unit
  may also be spelled after one space, singular or plural: `hour`,
  `minute`, `second`, `millisecond` (`20 seconds`), and a figure in
  that form has one part. `m` followed by `s` is `ms`. The figure ends
  at the first byte that begins no part (`.` ending the sentence, a
  space, the message's end). Its parts add, and the sum is rounded up to
  a whole millisecond. A whole or fraction of more than 9 digits, a
  number with no unit, or no part at all reads as no figure.
- **The window** is the period the message's limit is counted over: the
  first `per`, at the message's start or after a space, then one space
  and `sec`, `second`, `min`, `minute`, `hour` or `day`, the word ending
  at a byte that is not an ASCII letter: a second, a minute, an hour or a
  day. `per min (TPM)`, `per minute.` and `per day (TPD)` read; `permit`
  and `per minutes` do not.

The reading is one function beside `RateLimit::delay` in
`skein-llm/src/wire/failure.rs`, called where `delay` finds nothing, so
every route that classes `RateLimited` takes it.

## 3. Verification

Focused, in `skein-llm-world`'s suite (`skein/tests/llm`) and
`skein-llm`'s own tests:

- **The refusal of 2026-11-20, reproduced:** the Responses route's
  `rate_limit_exceeded` inside a stream whose head said 200, as
  `response.failed` and as an `error` event, with no `retry-after` and
  no reset field. With the message above, `RateLimited { retry_after }`
  is 6.5 seconds, its evidence `Response { status: 200 }`, its report's
  code `rate_limit_exceeded`; without its last sentence, 60 seconds;
  with neither the figure nor the window, zero.
- **The order:** a `retry-after` over a figure, a reset field over a
  figure, a figure over a window.
- **The grammar,** as a table: `6.5s` is 6,500 ms; `120ms`; `1m30s` is
  90 s; `2h0m4.25s`; `20 seconds`; `1 minute`; `0.0004s` rounds up to
  1 ms; `6.5`, `s`, a ten-digit whole and a figure cut by the detail
  limit are none; `per day (TPD)` is a day; `permit` is none.
- **What stays:** every classification and reset test as it is, among
  them `a_reset_comes_from_the_first_field_given`,
  `each_code_of_the_table_gives_its_class` and
  `shared_window_headers_are_evidence_but_the_client_reports_zero_delay`,
  whose fake's message names neither a figure nor a window.

## 4. Open questions

- **The head's rate-limit headers** (`x-ratelimit-reset-tokens` and
  `-requests`; `anthropic-ratelimit-*-reset`) say when a window resets,
  closer than its length. The client does not read them, as the fake's
  window test pins (`../testing/fakes/llm.md`, section 2.7); whether it
  should is open.
