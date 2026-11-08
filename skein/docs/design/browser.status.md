# browser: status

2026-11-07. Where skein's code lags or departs from [browser.md](browser.md).

Unlisted sections have no known gap. Section 5's retained terminal boxed
matches, truncation count and scope resolution without geometry are present.
Its `States.pressed` reads Chromium's text or boolean `true`; false,
mixed and absent properties read unpressed. The parser table holds
those five forms. No implementation gap remains for that state.

- **Not yet:** 4 and 8: `wire::decode` keeps no index; each field lookup
  and array step parses the subtrees it passes again, and compares each
  key by unescaping it.
- **Not yet:** 9: the real-Chromium viewport story is compiled; its runtime
  check awaits the browser tier after landing.
