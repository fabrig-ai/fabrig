# tui-design-session: status

2026-11-11. Where the design lags or departs from [tui-design-session.md](tui-design-session.md).

- **Design lags:** none: `../client/tui/pages.md`, *A design
  session*, and skein's `ui/screen.md`, 5.2, *A side region*, hold it,
  and the journey `design-session` is `../client/tui/journeys.md`,
  section 9.
- **Design departs:**
  - *What it builds*, the side pane: the terminal had no way to lay two
    columns, so skein's full-screen shell gains a side region beside the
    body (skein's `ui/screen.md`, 5.2), which shows its first rows and
    scrolls with nothing; at half the pane shows the draft from its
    first section marked new, and all is where it is read whole. Half
    shows as all below 80 columns (`temper_client_tui::HALVES`). The
    cut's call, 2026-11-09.
  - *What it builds*, the "new" marks: a section is a `#` or `##` part
    of the document, as the domain splits it, and "new" stands two
    spaces after its heading, not flush with the pane's end. The cut's
    call, 2026-11-09.
  - *What it builds*, `/draft`: on a chat it is offered only where the
    chat has a document or writes one, so it shows or hides the draft
    and never adds one to a chat that has none, as the request names
    it. The cut's call, 2026-11-09.
  - *What it builds*, one question at a time: the reply prints as
    written, its list included, and the prompt stands under the box,
    its last choice the TUI's "Tell it something else"; a choice's words
    are cut from its reason at the first dash or colon, since the
    drafter's instructions fix no form for the reason. The cut's call,
    2026-11-09.
  - *What it builds*, the quiet line: the words after the pane's state
    say what the next Ctrl-P does ("Ctrl-P opens", "widens", "folds"),
    where the mockup has "Ctrl-P folds" at half; at 80 columns
    "/goal starts a goal from the draft" is left out for room; and the
    link's state shows while it is not Live. The cut's call, 2026-11-09.
  - *What it builds*, `/goal`: a goal made returns to the chat, since
    the TUI has no goal's page; the prompt offers the ways and "Cancel",
    and the sheet's name and asking keep their defaults. The cut's call,
    2026-11-09.
- **Open:** none.
