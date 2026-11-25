# browser: status

2026-11-25. Where skein's code lags or departs from [browser.md](browser.md).

Unlisted sections have no known gap. Section 5's retained terminal boxed
matches, truncation count and scope resolution without geometry are present.
Its `States.pressed` reads Chromium's text or boolean `true`; false,
mixed and absent properties read unpressed. The parser table holds
those five forms. No implementation gap remains for that state.

- **Not yet:** 9: the real-Chromium viewport story is compiled; its runtime
  check awaits the browser tier after landing.
- **Not yet:** 3, 5: a browser that ends on its own reports no `Ended`
  and its `Ending`, and the command lacks `--use-mock-keychain`.
  [in flight: change 18056]
- **Design departs:** 9: the real-Chromium story's leaving document sends
  its request after the reload's document request, signalled over its
  still-open stream, in place of a `keepalive` request at `pagehide`: on
  Chromium 147.0.7727.101, in three runs by name, the origin received the
  `pagehide` request but no event of the page's session reported it, nor
  its handler's console message. The check stays: the old load, after the
  new document's `Sent`. free decided, 2026-11-19. [in flight: change 9768]
