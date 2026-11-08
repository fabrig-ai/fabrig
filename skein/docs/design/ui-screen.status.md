# ui-screen: status

2026-11-07. Where skein's code lags or departs from [ui-screen.md](ui-screen.md).

Unlisted sections have no known gap. Sections 5.3 and 5.4 have each role's
words, marks and properties, focus columns and paint, including disabled
select options. Section 9's every-role, every-state and every-property
fixture has goldens at 80 by 24 and 80 by 144; the latter holds the whole
tree. Section 10 keeps the deferred rendering and input work.

- **Not yet:** 6.1, *The terminal's answers*, and 8, *Entering* and *The
  terminal's background*: `Decoder::answers` and `Decoder::background`,
  `Settings.background`, `ASK_BACKGROUND`, `Background` and
  `Terminal::background` are not built; section 9's rows for them and
  for a cell drawn in the terminal's defaults are not written.
- **Departs:** 6.2: PageUp and PageDown move the body's scroll by nine
  rows, rather than its visible rows less one: the key reader has no body
  height in its inputs, and the screen-keys task permits a fixed ten-row
  page without adding a field to `Focus`.
