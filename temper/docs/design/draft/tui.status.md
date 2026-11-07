# tui: status

2026-11-07. Where the design lags or departs from [tui.md](tui.md).

- **Design lags:**
  - 4: no design document holds the TUI. `../client/README.md`, section 2 lists none of its crates, and 6.9 names only "a later full-screen terminal". The first pass's designer writes its part (`tui.execution.md`, M1 to M3).
  - 4.1: `../protocol/README.md`, section 4, and `../../help/reference/commands.md` give the binary as a table of commands; bare `temper` and `temper ADDRESS` fit no row.
  - 4.2: jig's `client/domain/contract.md`, section 2, jig's `domain/people.md`, section 3, and jig's `protocol/client.md`, section 8, hold two built-in ways of signing in, not an approval by a signed-in browser. `../client/browser.md`, 2.3 and 2.4, have no approval page.
  - 4.4: skein's `ui.md`, section 16 has two shells, the text shell and the browser's parts, and no full-screen one.
  - 5: `../client/journeys.md` records the browser's client alone.
- **Open:**
  - 3: all of it, refined on recorded frames once the first pass runs.
  - 4.2: the code carried in the tab's address on a box, a risk taken so that one press approves: the user's to confirm.
  - 8: bare `temper` as a person's command.
