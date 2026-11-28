# tui: the reasons behind it

This holds the reasons for `tui.md`, moved from the shared notes file.

The reasons behind `tui.md`, from the user, 2026-11-07.

- **Two jobs, not one in two shells.** "It should not be a replacement for
  the web dashboard, I think the TUI should be more 'working with
  agents'-oriented, and the web UI more 'dashboard overview of where things
  stand, what needs me, progress, etc'." The TUI's pages are its own, not
  the browser's drawn in a terminal, which would be a lesser web client.
- **Unattended once started.** "The expectation is that once a work starts
  it goes on unattended." Steering a goal or a delegate from the TUI was
  offered and left to the web, which may show enough for a quick look.
- **Agents off the person's machine.** Unlike Claude Code, temper's agents
  work on the forge; the user: "that's a big plus".
- **A side pane, no diffs.** A draft being written shows "a side pane like
  the web UI, showing the draft. Pane can be folded, half-screen or full
  screen", where an inline diff was offered.
- **Full screen, not inline.** Claude Code keeps its past in the terminal's
  own scrollback and redraws a live region below it. A pane beside the
  conversation needs the whole screen, and a recorded frame is well defined
  only as a whole screen, so the TUI takes the screen, as `chat-reading.md`
  already scrolls by keys.
- **The browser approves.** "Ideally I start temper and it opens a tab in
  the browser, where I accept and then I can close the tab. Over ssh a cut
  & paste code will be needed." It is jig's built-in, not temper's own
  sign-in ending in `Mint`, because jig mints for an application's
  providers alone and a box's person is signed in by jig's printed link;
  letting `Mint` reach jig's own providers would hand jig's rule to an
  application's code. A browser alone approves, since agents drive
  `temper ctl` as the owner. The code shown on both sides lets a person
  check that the tab is their own terminal's, but on a box it rides the
  tab's address, so that one press approves: a risk taken because
  whoever could ask for an approval is on the machine already. A
  deployment at a name will have the code typed.
- **The bare minimum first.** "First just the bare minimum for e2e, it will
  get us the foundations laid out in the code", on a box alone, with no
  Google and no deployment at a name: "we start same machine, but we'll
  move to named deployment soon". The user then uses it and refines the
  design on its recordings, so the TUI records from the start.
- **In v0.1.0.** The user put the first pass in v0.1.0, which had left the
  terminal client out.
