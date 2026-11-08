# ui-screen: status

2026-11-08. Where skein's code lags or departs from [ui-screen.md](ui-screen.md).

Unlisted sections have no known gap. Sections 5.3 and 5.4 have each role's
words, marks and properties, focus columns and paint, including disabled
select options. Section 9's every-role, every-state and every-property
fixture has goldens at 80 by 24 and 80 by 144; the latter holds the whole
tree. Section 10 keeps the deferred rendering and input work.

- **Not yet:** 3 and 5.2: a border with no sides, framing a region by
  two rules; a title's part at the border's end. 3 and 5.3: `Align` and
  `band` on a token's look, a line aligned to its end and a band.
- **Not yet:** 3, 5.3 and 5.4: the theme's active paint; an options menu
  laid one row an item with its names' column and its active item; the
  active item kept on the menu's enabled items. 5.2: the body following
  a log's end (`Focus::follow`, `Focus::ended`) and saying its rows
  (`Focus::rows`). 6.2: a field's keys while an options menu follows it;
  PageUp and PageDown by the body's rows less one, clearing and setting
  `follow`, where today they move it by nine rows.
- **Not yet:** 3 and 5.4: `label` on a token's look, a text field or
  area laid with no name.
- **Not yet:** 3 and 5.3: `mark`, `described` and `spaced` on a token's
  look: an item's mark, a description's own paint, one blank row
  between children.
