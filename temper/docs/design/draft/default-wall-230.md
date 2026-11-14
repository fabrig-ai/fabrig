# The default suite's wall: 200 s to 230 s, for now

Draft, 2026-11-14, on the user's word: "raise the budget to 230s … this is band aid until temper/tiers looks at it".

## 1. Why

The default profile's suite takes about 199 s of its 200 s `global-timeout`, on about 1,200 s of CPU, even with nothing else running. On the CI runner, one core's difference cuts it: nextest cancels every remaining test, and the gate reports "tests unobserved twice". Today that failed five landings' CI runs, each needing a hand rerun.

## 2. Wanted

- In `.config/nextest.toml`, the default profile's `global-timeout = "200s"` becomes `"230s"`. Above it goes one comment line: a band-aid by the user's word, 11-14, until the default suite is brought back under 200 s by the testing strategy's order of repairs (`skein/docs/foundation/testing-strategy.md`, 8.1).
- Wherever a document states the default's wall as 200 s, it says 230 s, with the same note.
- Nothing else changes: no other profile, no per-test timeout, no test moved between tiers.

## 3. Tests

Cheap: focused and fuzzy suites only; walk = false. No new test: the change is a figure in the configuration. Its own landing's CI runs under the new figure.
