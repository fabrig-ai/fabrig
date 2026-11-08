# ui-screen: status

2026-11-08. Where skein's code lags or departs from [ui-screen.md](ui-screen.md).

Unlisted sections have no known gap. Sections 5.3 and 5.4 have each role's
words, marks and properties, focus columns and paint, including disabled
select options. Section 9's every-role, every-state and every-property
fixture has goldens at 80 by 24 and 80 by 144; the latter holds the whole
tree. Section 10 keeps the deferred rendering and input work.

Sections 3, 4 and 7 have no known gap in preserving and drawing the
terminal's default colours. Sections 6.1, 8 and 9 have the bounded OSC
answer decoder, optional background query and wait, and decoder and
simulated-terminal test rows for readable, unreadable, late and absent
answers. The application's choice of dark or light stays outside this
terminal, as section 8 requires.

- **Not yet:** 3 and 5.3: `mark`, `described` and `spaced` on a token's
  look: an item's mark, a description's own paint, one blank row
  between children.
- **Departs:** 6.2: PageUp and PageDown move the body's scroll by nine
  rows, rather than its visible rows less one: the key reader has no body
  height in its inputs, and the screen-keys task permits a fixed ten-row
  page without adding a field to `Focus`.
