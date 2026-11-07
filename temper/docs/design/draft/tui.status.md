# tui: status

2026-11-07. Where the design lags or departs from [tui.md](tui.md).

- **Design lags:**
  - 4.2: jig's `client/domain/contract.md`, section 2, jig's `domain/people.md`, section 3, and jig's `protocol/client.md`, section 8, hold two built-in ways of signing in, not an approval by a signed-in browser. `../client/browser.md`, 2.3 and 2.4, have no approval page.
  - 5: `../client/journeys.md` records the browser's client alone.
- **Design departs:**
  - 4.1: on the sign-in screen `q` is a character of the pasted link's field, which has the focus there, so Ctrl+C closes it and `q` closes the first screen (`../client/README.md`, 6.14): decided by the cut of `tui-connected`, 2026-11-07.
- **Open:**
  - 3: all of it, refined on recorded frames once the first pass runs.
  - 4.2: the code carried in the tab's address on a box, a risk taken so that one press approves: the user's to confirm.
  - 8: bare `temper` as a person's command: the cut of `tui-connected` took the section's default, the TUI in a terminal and the usage otherwise (`../client/README.md`, 6.11; `../protocol/README.md`, section 4), as `tui.execution.md` asks; it waits for the user's word.
