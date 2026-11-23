# Ways of working's look

Ways of working's look is split from [the browser's look](look.md).

## 1. Ways of working

- **Ask temper's ways** (`../pages/README.md` and `../pages/chat/conversation.md`, 3) are `ask-temper-ways`'s rules, in
  `temper.css`: the opening's, `.temper-ways` with its tabs, axis, panel, card, legend
  and act, under `/* ==== ui: settings ==== */`, where `.temper-starter`'s stand; a
  path's, `.temper-ways-path` and its marks, under `/* ==== ui: chats ==== */`, which a
  reply's views share. Where skein draws an element otherwise than the request's page (a
  tab an `<a role="tab">`, a scene's stop a `<g>` with no ARIA, an open stop a token, one
  SVG laid out by width where the page has two), the selector names what is drawn and
  keeps the request's declarations.
  The drawing's rules (the path's and the legend's) and the card's hold
  under How work runs' frame as under Ask temper's page
  (`../pages/settings.md`, 2), each selector naming both. Neither page
  draws tabs, their axis line or an act in the path's foot, and their
  rules go with them.
  Ask temper's page takes `ways-current`'s rules (2026-11-16), scoped
  under the session, `#app > .temper-session`, on tokens of their own,
  `temper-way-*`: the region `.temper-way`, 18 px under the lede; its
  head `.temper-way-head`, one row that wraps, the line
  `.temper-way-line` at its left, 15 px in `--gf-ink-2`, its acts
  `.temper-way-acts` at its right, each outlined, 34 px high, 13.5 px
  at 500, "Use this" and "Keep it" filled in ink, at their left on a
  phone; the drop-down `.temper-way-pick`, its button outlined with its
  words in ink (`ways-shown`) and a chevron in `--gf-muted`, its menu
  `.temper-way-menu` the composer's model menu's panel and rows
  (`.temper-model-menu`; `../pages/chat/composer.md`, 1, *The work's
  account and model*), 6 px under its button at its right,
  the way shown ticked at 500 with a "✓" in the good marks'
  green; and the last line `.temper-way-ask` under a `--gf-line` rule,
  14 px in `--gf-ink-2`, its mark `.temper-way-ask-mark` in ink. Where
  skein draws the menu otherwise than the request's page (its button
  made by skein beside the menu, a row a `menuitem`, the tick the state
  skein draws for a chosen item), the selector names what is drawn and
  keeps the request's declarations. The rules for "Keep it" and "Use
  this" in the foot of Ask temper's page go: the foot holds "How does
  temper work?" alone, at the same height in every state.
  Both pages take `ways-shown`'s rules (2026-11-19), under the
  `/* ways-shown */` comment after `ways-current`'s, each selector
  naming both `#app > .temper-session` and `#app > .temper-settings`
  but where one page alone is said. The drop-down's button keeps its
  look, its words in ink. In its menu the way the project runs says
  "current" in the item's description, `[role=menuitem] >
  [id^="skein-ui-description-"]`, at the row's right
  (`margin-left: auto`), `--gf-faint`, 12.5 px; a ticked row holding it
  takes its "✓" 10 px after it. The acts leave the line's row: on Ask
  temper's page, the starter holding the ways is their frame
  (`.temper-starter:has(> .temper-ways)`, `position: relative`), its
  lede `.temper-ways-lede` leaves them `padding-right: 260px`, and
  `.temper-starter > .temper-ways .temper-way-acts` sits at the lede's
  right, `position: absolute; top: -3px; right: 0; margin: 0`. On How
  work runs, the path's section holding the head is their frame
  (`.temper-settings-section:has(> .temper-way-head)`), its head
  `.temper-settings-section-head` leaves them `padding-right: 260px`,
  and `.temper-settings-section > .temper-way-head > .temper-way-acts`
  sits at that row's right, `position: absolute; top: -6px; right: 0;
  margin: 0`, clear of the row's rule. At 760 px and under, each lede or
  section head takes `padding-right: 0` and the acts `position:
  static`, so the head wraps as before. Where skein draws the menu
  otherwise than the request's page (the "current" a description's
  `<span>` with an id where the page has a class, `.temper-way-current`;
  a row's name its own text where the page wraps it in a `<span>`), the
  selector names what is drawn and keeps the request's declarations.
  Ask temper's lede reads "How a change is produced and lands in
  PROJECT.". "How does temper work?" has no style token of its own,
  taking `.temper-ways-acts > button`'s outlined look. Its hint names the
  fixed message it sends (`../pages/ask-temper.md`, 1). This button is
  Ask's opening's alone, never How work runs' (`ask-opening`,
  2026-11-12).
  A path's marks take no stroke but their own rules', as the request's
  page has none: skein's default stroke for a scene's marks (skein's
  `ui/web/README.md`, 10) gives way under `.temper-ways-path` by one rule of the
  path's, scoped to its drawn figure, `stroke: none` on the marks skein's
  rule names, outweighing skein's rule and outweighed by every mark's own
  (the line's, the boundary's, a person's ring, the halo, the caret, a
  focused stop's area); skein's rule stays as it is.
  Who acts is the whole story's marks (`web/whole-story/whole-story.css`,
  `.ways-*`; `../../draft/surfaces.md`, 13), orange the person's alone: a
  person's ring `--gf-paper` stroked `--gf-signal` at 2; an agent's star
  `--gf-ink`, and outlined when needed, `--gf-paper` stroked `--gf-ink`
  at 1.3 with round joins; code's square `--gf-muted`, and outlined,
  `--gf-paper` stroked `--gf-muted` at 1.3; a stop's small words 10 px
  in `--gf-faint`; the legend's agent "✦" in `--gf-ink` and its agent
  when needed the outlined star. The line's base is `--gf-line-2` at
  1.5; each rise's fade (`../pages/chat/conversation.md`, 3) is the look request's, its
  stops coloured by their tokens: a person's `--gf-signal` at .8 at its
  mark, .28 at the dotted line, `--gf-line-2` at the base; an agent's
  `--gf-ink` at .3 and .1, the whole story's lighter tone for an agent
  (`.ways-rise.a`), then `--gf-line-2`. Night takes the same tokens'
  night values.
- **How work runs' look** (`../pages/settings.md`, 2, *The page*) is the whole
  story's `web/whole-story/how.html`, its `whole-story.css` rules taken
  exactly, values and all, in `temper.css` under `/* ==== ui: how ==== */`,
  which stands after the ways' rules under Settings' marker, scoped under
  the project's frame on tokens of its own, `temper-how-*`:
  - *its column,* `temper-how-wide` on Settings' column while How work runs
    is its tab: `.col.wide`'s 1020 px, the frame's other tabs keeping 720;
  - *its head,* Settings' own (`.intro`, `.kicker`, `.headline`, `.lede`),
    "Ask temper to change this" a `.btn.dark`, Settings' primary;
  - *a section's head,* `.shead`: its title and, at its right,
    `temper-how-way`, "the way: NAME", as `.shead span`;
  - *the path,* Ask temper's drawing and legend under their own rules
    (*Ask temper's ways,* above), who acts by the path's marks alone: no
    lanes, no pills, no colour of its own;
  - *the ways' head, card and act* take Ask temper's head's and card's
    rules (above), the same declarations under the frame's scope, on the
    same `temper-way-*` tokens: the head over the drawing, the card under
    it, its caret across; Ask temper's last line is not drawn here;
  - *Roles,* `whole-story.css`' `.rows` and `.role` rules taken exactly:
    a role's row `.role > summary`'s grid, 200 px for the name, its line
    small and muted, the model at its right with a "›" turning down while
    open; opened, `.role > dl`'s terms and `.role-more`'s link under its
    hairline; at 760 px and under the line under the name, the terms
    stacked; on tokens of its own, `temper-how-roles`, `temper-how-role`,
    `temper-how-role-line`, `temper-how-role-model` and
    `temper-how-role-more`;
  - *Budgets' row,* `temper-how-row`, `.row`'s grid with the first column
    200 px, its small words muted and its figure at its right;
  - *What reaches you,* `whole-story.css`' `.grid3`, `.box` and `.who`
    rules taken exactly, on tokens of its own: `temper-how-reaches`, the
    columns' grid, `repeat(auto-fit, minmax(240px, 1fr))` with a gap of
    16, so two boxes share the width and a phone stacks them;
    `temper-how-reach`, a box, its lines `.small` with no bullets' margin
    of their own; `temper-how-who-agent` and `temper-how-who-person`, the
    chip and its mark, `.who.a`'s star (`--star`, `--gf-ink`) and
    `.who.p`'s ring (`--gf-signal`); `temper-how-reach-note`, the box's
    last line, `.small.muted`; its rows under the grid Settings'
    `.setting` rows;
  - *Rules by path,* `whole-story.css`' `.shead a`, `.scroll` and `.tbl`
    rules taken exactly, on tokens of its own: `temper-how-add`, "+ A
    rule" at the head's right; `temper-how-rules`, the `.scroll` box
    that scrolls the table across on a phone; `temper-how-table`, the
    `.tbl` table, its header cells small and muted, its rows on
    hairlines; `temper-how-paths`, a row's paths, `.mono`; a row's
    sign-off What reaches you's chip, `temper-how-who-person`; an empty
    cell's "–" `.muted`; `temper-how-rules-note`, the note under it,
    `.small.muted`;
  - *What follows a landing,* `whole-story.css`' `.grid2`, `.box`, `.box
    h3`, `.chips` and `.who.c` rules taken exactly, on tokens of its own:
    `temper-how-follows`, the boxes' grid, `repeat(auto-fit, minmax(300px,
    1fr))` with a gap of 16; `temper-how-follow`, a box, its title `.box
    h3`'s 17 px at 600, its words and its lag's line `.small`, the lag's
    `.small.muted`; `temper-how-chips`, its chips' row, wrapping with a gap
    of 6; `temper-how-who-code`, code's chip, `.who.c`'s 7 px square with
    corners of 1.5 on `--gf-muted`, beside What reaches you's
    `temper-how-who-agent`;
  - *Budgets' head,* the period's words at the section head's right, as
    `temper-how-way` is;
  - *on a phone,* at 760 px and under, the column takes the frame's phone
    width and the path runs down the page, as on Ask temper's page.

## 2. Open questions

None.
