# ui-web: status

2026-11-12. Where skein's code lags or departs from [README.md](README.md).


`elements.md` and `README.md`, section 5 have no known gap in a field's options: listbox and
option mapping, retained active selection, arrows, Enter without a
paired Submit, prefix completion on Tab and pointer presses retaining
the field's focus. `README.md`, section 8's kept store is separate from the small
store, with the same encoding and bounds and refusal where local
storage is unavailable. `README.md`, section 9's full-screen reports, Escape-only
lock and unlock are present. Real-key checks remain `README.md`, section 12's
after-landing work; no browser result is established here.

`elements.md` has no known gap in mapping a node's hint to an escaped
`title` attribute, including its replacement and removal by a patch.

`testing.md`, section 2 has no known implementation gap in `Step::Pressed`: it
finds the named target again at the face's poll interval and ends within
the step's bound, including without a new event. The script table holds
pressed at the first or a later look, unpressed until the bound, and
absent until the bound, with the latter two failing for their reasons.

`testing.md`, section 2 has no known gap in the three `folds_away` stories' arrival:
each starts with a `Go` and awaits the fixture's `Editor` disclosure
within 2 s before any act or read. Every folding, focus and outside-press
check stays in order, with the existing deadlines. This arrival changes
no page-part behaviour; `README.md`, section 5's departures below remain.

Unlisted sections, including 7's requested focus bringing its node into
view even when already focused, have no known gap. A bound continuing
list reports its foot near through the page's scroll and apply paths,
without log following. Temper's retained sheet story exercises this path;
no passing runtime result for its repaired version is supplied here.

Sections 6 and 10 have no known gap in the lazy scene, its stated
height, the SVG in its figure's flow, its stated width with
`data-scene-width`, `skein-ui.css` scaling its SVG to the figure's
width and a press's point scaled to the scene's pixels, or the flow
tokens' default look.
The host table reads the Markdown cases and compares their SVG bytes.

Sections 6, 10 and 11 have no known gap in a fade: `svg::draw` puts a
`<linearGradient>` for each faded mark in the SVG's first `<defs>`, its
stops' tokens classes, the mark `stroke="url(#ID)"` and `data-fade`,
its id from the stem `ids` and the page part's `skein-ui-fade-<node
id>`; `skein-ui.css`'s default stroke skips a faded mark. The golden
scene and `two_drawings_paint_their_fades_by_ids_of_their_own` hold
it; no browser has painted one on record.

- **Departs:** 5: a press outside an open menu returns the focus to its button
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
- **Not yet:** 5: no run on record shows that tier within its budget with
  `log_reading`'s seven stories, from the plan `chat-reading-c`, which waits for its run
  after landing likewise.
