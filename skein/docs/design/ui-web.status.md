# ui-web: status

2026-11-06. Where skein's code lags or departs from [ui-web.md](ui-web.md).

- **Departs:** 7: a press outside an open menu returns the focus to its button
  unless that press took it to another control, where the code asks only whether
  the focus left the menu (`skein-ui-web`'s `page.rs`, `close_menu_outside`, and
  `page/web.rs`, `press_outside`). An element that took the focus by request or by
  a removal, the main among them, keeps its `tabindex="-1"`, so a press on its own
  inert words takes the focus there, on no control, and the button doesn't get it
  back. No reason is recorded; seen by reading, and no test presses past a menu
  with such an element under the press.
- **Departs:** 4.3, *The loader*: a stall before the module's answer draws no words,
  where the line has `data-waiting` after 15 s with no byte. `loader.js`'s `draw`
  returns until the glue and that answer are both in, as the same section's "the line
  says nothing" has it, so a server holding the answer's head leaves the mark and an
  empty bar with no line, and `#boot-note` empties at 15 s. The draft that asks the
  loader has a stall say it is waiting
  (`temper/docs/design/draft/bundle-loading.status.md`). No reason is recorded; seen by
  reading, and no test stalls an answer: the proof (the `bundle-loading` plan's figures
  on its render check, at `668f7cd3e5`) paused one part-way.
- **Departs:** 4.2: `Cargo.toml` builds `flate2`, `miniz_oxide`, `crc32fast` and
  `adler2` at `opt-level` 3 in development, as it does the bundler, because the tiers
  and the build scripts run the dev bundler. `../foundation/testing-strategy.md`, 8 lets
  a tool build optimized, and has a dependency's override show a profile and the user's
  sign-off, with a comment citing both. Neither is recorded, and `flate2` also reaches
  `skein-ui-web-tests` as a dev dependency.
- **Not yet:** 4.3: no run on record shows skein's browser tier with the loader, which
  its stories on the example load, within `tools/tiers/budgets.toml`'s `browser-skein`, nor
  `browser_stories`'s `a_placeholder_gives_way_to_the_client` and
  `a_module_not_found_says_so_and_offers_to_try_again` passing there: the plan that wrote
  them ran focused and fuzzy alone, the tier waiting for its run after landing
  (the `bundle-loading` plan's figures on the browser tier, at `668f7cd3e5`).
