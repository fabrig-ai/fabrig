# ui-screen: status

2026-11-09. Where skein's code lags or departs from [ui-screen.md](ui-screen.md).

- **Not yet:** 3, 5.4 and 9: a token's focused value's paint,
  `StyleLook::focused_value`; a focused field's value and caret are laid
  in the focus's paint whatever its token says (plan `tui-box-polish`).

Sections 3, 5 and 6.2 include rules without
sides, border-end titles, end-only titles two glyphs across from a
rule's end, end alignment, bands, unlabelled fields, marks,
description paints, opt-in active-name paints, child spacing and options-menu selection. Paging
uses the visible body's rows less one, at least one; paging up stops
following the log's end and reaching the end resumes it.

Sections 6.1 and 8 release a held Escape when the background wait ends;
a later read-final Escape is a key, while a whole late OSC answer is
consumed. Section 9's renderer, key and simulated-terminal rows cover
these paths. Section 10 keeps log-edge reporting and the other deferred
rendering and input extensions.
