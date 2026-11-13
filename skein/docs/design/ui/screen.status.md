# ui-screen: status

2026-11-12. Where skein's code lags or departs from [screen.md](screen.md).

Beside the entry below, no known implementation gap. Sections 3, 5 and 6.2 include rules without
sides, border-end titles, end-only titles two glyphs across from a
rule's end, end alignment, bands, unlabelled fields, marks,
description paints, opt-in active-name paints, focused-value paints, child spacing and options-menu selection. Paging
uses the visible body's rows less one, at least one; paging up stops
following the log's end and reaching the end resumes it. A token's
focused value's paint lies on a focused text field's or text area's
value and caret's cell, its mark and label keeping the focus's paint,
the last token naming one winning
(`focused_value_paint_leaves_the_focus_paint_on_the_mark_alone`).

Sections 6.1 and 8 release a held Escape when the background wait ends;
a later read-final Escape is a key, while a whole late OSC answer is
consumed. Section 9's renderer, key and simulated-terminal rows cover
these paths. Section 10 keeps log-edge reporting and the other deferred
rendering and input extensions.

