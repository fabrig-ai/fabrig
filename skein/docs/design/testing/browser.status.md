# browser: status

2026-11-18. Where skein's code lags or departs from [browser.md](browser.md).

Unlisted sections have no known gap. Section 5's retained terminal boxed
matches, truncation count and scope resolution without geometry are present.
Its `States.pressed` reads Chromium's text or boolean `true`; false,
mixed and absent properties read unpressed. The parser table holds
those five forms. No implementation gap remains for that state.
Sections 4 and 8 have no known gap in a message's index: `wire::decode`
counts its entries while validating, allocates them once and looks a
field, an element and a value's bytes up in them; its unit tests hold the
lookups, and the memory suite runs it under its ceilings (section 8).

- **Not yet:** 9: the real-Chromium viewport story is compiled; its runtime
  check awaits the browser tier after landing.
- **Not yet:** 5 to 9: a page's requests, `Request::Page`'s `requests`,
  `Sent` and `Answered`, `Limits::requests` and the request's `Trouble`,
  with their step tests, the world's fake browser and referee, and the
  real-Chromium requests story, compiled. [in flight: change 8608]
