# tui: status

2026-11-08. Where the design lags or departs from [tui.md](tui.md).

- **Design lags:** 3 and 7: the later surfaces, approval between hosts
  and managing sign-ins await the user's refinement after the first
  pass; `../client/README.md`, section 11 defers them. Sections 4 and 5
  have no lag: 4.2's TUI side is `../client/README.md`, 6.13 and 7.1,
  and 5 is `../client/tui/journeys.md`. Section 8's journey-format
  default is settled by that design: the browser's versioned syntax and
  common records, with the TUI's own records and codec, sharing the
  browser's redaction and last scan (its sections 3 and 5).
- **Design departs:**
  - 4.1: a local box signs the TUI in with nothing to paste. The running engine keeps its live printed link at rest in a private file, under `$XDG_RUNTIME_DIR` on Linux and the data directory on macOS, keyed by the deployment's data directory; a fresh `temper`, or one whose sign-in expired, signs in by it with no key pressed, and bare `temper`'s address is that file's before the one its directory keeps. A sign-in ended before its expiry is noted in the TUI's directory, and only a person's act signs it in again. The risk of a live owner link at rest is recorded as accepted, with its way back (`../client/README.md`, 6.11, 6.12 and 6.14): the user's decision of 2026-11-07, designed by the cut of `tui-link-colours`.
  - 4.4 and 5: the TUI draws on the terminal's own colours. Its base is the terminal's default text and background, the palette kept for the accent, dim, faint and good colours, and the focus shown by the accent and its mark with no background of its own; dark or light comes from `COLORFGBG`, else the terminal's answer to an OSC 11 query at start, else dark (`../client/README.md`, 6.15; skein's `ui-screen.md`, sections 3 and 8). Frames and goldens keep the terminal's default as a colour of its own, which a journey's page draws in the theme's stand-ins (`../client/tui/journeys.md`, 6.4): the user's decision of 2026-11-07, designed by the cut of `tui-link-colours`.
  - 4.1: on the sign-in screen `q` is a character of the pasted link's field, which has the focus there, so Ctrl+C closes it and `q` closes the first screen (`../client/README.md`, 6.14): decided by the cut of `tui-connected`, 2026-11-07.
- **Open:**
  - 3: all of it, refined on recorded frames once the first pass runs.
  - 4.2: the code carried in the tab's address on a box, a risk taken so that one press approves: the user's to confirm.
  - 8: bare `temper` as a person's command: the cut of `tui-connected` took the section's default, the TUI in a terminal and the usage otherwise (`../client/README.md`, 6.11; `../protocol/README.md`, section 4), as `tui.execution.md` asks; it waits for the user's word.
  - 8: the code's length and lifetime, and the approval's page, took section 8's defaults (`tui-approval`, as `tui.execution.md` asks): eight characters of twenty consonants, ten minutes (jig's `protocol/client.md`, section 8), and the web client's page at `/terminal` (`../client/README.md`, 3.5; `../client/browser.md`, 2.3), for the user to refine on the first pass.
