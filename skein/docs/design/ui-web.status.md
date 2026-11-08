# ui-web: status

2026-11-08. Where skein's code lags or departs from [ui-web.md](ui-web.md).

Sections 6 and 7 have no known gap in a field's options: listbox and
option mapping, retained active selection, arrows, Enter without a
paired Submit, prefix completion on Tab and pointer presses retaining
the field's focus. Section 10's kept store is separate from the small
store, with the same encoding and bounds and refusal where local
storage is unavailable. Section 11's full-screen reports, Escape-only
lock and unlock are present. Real-key checks remain section 15's
after-landing work; no browser result is established here.

Section 6 has no known gap in mapping a node's hint to an escaped
`title` attribute, including its replacement and removal by a patch.

Section 14.2 has no known implementation gap in `Step::Pressed`: it
finds the named target again at the face's poll interval and ends within
the step's bound, including without a new event. The script table holds
pressed at the first or a later look, unpressed until the bound, and
absent until the bound, with the latter two failing for their reasons.

Section 14.2 has no known gap in the three `folds_away` stories' arrival:
each starts with a `Go` and awaits the fixture's `Editor` disclosure
within 2 s before any act or read. Every folding, focus and outside-press
check stays in order, with the existing deadlines. This arrival changes
no page-part behaviour; section 7's departures below remain.

Unlisted sections, including 7's requested focus bringing its node into
view even when already focused, have no known gap. A bound continuing
list reports its foot near through the page's scroll and apply paths,
without log following. Temper's retained sheet story exercises this path;
no passing runtime result for its repaired version is supplied here.

Sections 8 and 12 have no known gap in the lazy scene, its stated
height, the SVG in its figure's flow, or the flow tokens' default look.
The host table reads the Markdown cases and compares their SVG bytes.

- **Departs:** 7: a press outside an open menu returns the focus to its button
  unless that press took it to another control, where the code asks only whether
  the focus left the menu (`skein-ui-web`'s `page.rs`, `close_menu_outside`, and
  `page/web.rs`, `press_outside`). An element that took the focus by request or by
  a removal, the main among them, keeps its `tabindex="-1"`, so a press on its own
  inert words takes the focus there, on no control, and the button doesn't get it
  back. No reason is recorded; seen by reading, and no test presses past a menu
  with such an element under the press.
- **Departs:** 7, *Near*: a read that failed with the person at a log's head is not
  asked for again when they scroll away and back within one and a half screens, where
  the section has an owner that keeps *busy* through a failure hear again once the
  person moves, and `temper/docs/design/draft/chat-reading.md`, 5 has a failed read try
  again as the head nears. `page.rs`'s `report_near` counts its quarter of a screen from
  where it last reported, and lets that go only once the view has left that reach, which
  a log under two and a half screens can't, so such a log hears nothing more. No reason
  is recorded; seen by reading: `a_log_kept_busy_says_nothing_until_the_person_moves`
  moves nearer from short of the head, temper's `a_failed_head_read_says_so_and_the_next_near_retries`
  gives its `Near` by hand, and no browser story has a read fail.
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
- **Not yet:** 7: no run on record shows that tier within its budget with
  `log_reading`'s seven stories, from the plan `chat-reading-c`, which waits for its run
  after landing likewise.
