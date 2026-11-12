# browser: status

2026-11-08. Where skein's code lags or departs from [browser.md](browser.md).

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
- **Not yet:** 9: the real tab story reads the tabs once after each press,
  rather than asking again each poll until they settle. [in flight: change 1331]
