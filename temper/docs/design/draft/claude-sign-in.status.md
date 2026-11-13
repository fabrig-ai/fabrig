# claude-sign-in: status

2026-11-12. Where the design lags or departs from [claude-sign-in.md](claude-sign-in.md).

The design carries the draft in full: the ask, its offer, cancel and
ends in jig's `domain/terminal-login.md`; the terminal lent in skein's
`ui/screen.md`, 8, `runtime/shell.md`, 6.3 and `runtime/io/processes.md`,
1; the TUI's side in `../client/tui/box.md`, *Lending its terminal*; the
browser's in `../client/pages/welcome.md`, 1, *Claude Code's sign-in*,
and `../client/pages/accounts.md`, 1; section 8's findings in jig's
`domain/providers.md`, 3, smith's `protocol/programs.md`, 4.2 and 5.3,
and `../protocol/setup/owned.md`, 6. Section 7's choices are settled
there: the ask is a request and a watch (`terminal-login.md`, 2 and 3);
the TUI in front of the box, signed in as the person, lends; the TUI
gives `program-login` the directories it started the box with.

- **Design departs:** 2, *Signed in (04)*: the engine, not the page,
  declares the source's name or checks the person's source of that name
  once the TUI reports exit 0 (jig's `domain/terminal-login.md`, 4), and
  the page reads the login's end, because the sign-in goes on with the
  page closed (7, *The page closed*) and only the TUI sees the exit;
  goal 1905's design, 2026-11-12, within 7's "the pick is the
  implementer's".
