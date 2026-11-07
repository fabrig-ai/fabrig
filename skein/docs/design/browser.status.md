# browser: status

2026-11-07. Where skein's code lags or departs from [browser.md](browser.md).

- **Not yet:** 9: the real-Chromium viewport story is compiled; its runtime
  check awaits the browser tier after landing.
- **Not yet:** 6: `tests/browser/tests/support/real/tabs.rs` presses its opener's
  cut through a node it found before its tab took the front.
- **Not yet:** 9: `browser_real` and `browser_facts` start a Chromium for each fact,
  where the section has one hold many.
