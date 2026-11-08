# tui: execution

2026-11-07. What of [tui.md](tui.md) gets built first, and in what order:
its first pass (sections 4 and 5), and nothing past it. It designs
nothing: each milestone writes its own design from the draft as it runs
(`DESIGN.md`, section 3). The draft isn't settled in detail, so a
milestone that meets a question the draft leaves open takes its section
8 default, or the nearest thing, and says so in the draft's status.

## 1. In one page

- **Three milestones,** M1 to M3 (section 3): connected, approved,
  recorded.
- **M1 and M2 may run side by side:** M1 is the TUI's shell, host, view
  and first screen, signed in by the printed link, pasted; M2 is the browser's
  approval, jig's and the web's, and the TUI's way through it, which
  alone waits for M1.
- **M3 follows both:** the TUI's journeys, its first one walking M2's
  approval to M1's first screen. Its host seam is M1's, shaped for it
  from the start (`tui.md`, 4.4).
- **Not in the first pass:** a deployment at a name, Google, a chat, the
  side pane, the launchpad, needs from elsewhere, the hand-offs
  (`tui.md`, sections 3 and 7).
- **Then the user takes over:** they use the TUI and refine it on its
  frames; the next parts are drafted from what they ask.

## 2. How a milestone runs

- **Its entry here is its input,** with `tui.md` its source.
- **Its design and its build together,** landed whole: the design
  documents' sections it needs (`tui.status.md`'s lags), then the code
  and its focused and fuzzy tests.
- **Its walk** follows its landing and blocks nothing.
- **Its state** is section 3's table, kept current. When M3 has landed
  and the user has taken its frames, `tui.status.md` holds none of
  the lags of `tui.md`, sections 4 and 5, and this file goes with the draft
  (`DESIGN.md`, section 3).

## 3. The milestones

| | Milestone | Executes (`tui.md`) | Needs | State |
|---|---|---|---|---|
| M1 | Connected | 4.1, 4.3, 4.4 | | done: landed in `tui-connected`, walked on 2026-11-07 |
| M2 | Approved | 4.2 | M1, for the TUI's flow alone | done: jig, the engine and the web landed in `tui-approval`, the TUI's flow in `tui-recorded`, walked on 2026-11-07 |
| M3 | Recorded | 5 | M1, M2 | done: landed in `tui-recorded`, its first journey committed; the user took its frames on 2026-11-08 |

### M1. Connected

- **Executes:** `tui.md`, 4.1 (the command, its address, the pasted
  link, its directory, closing), 4.3 (the first screen) and 4.4 (its
  parts).
- **Design:** the TUI's part of `../client/README.md` (its crates, its
  host, its terminal, its directory) and of `../protocol/README.md`,
  section 4 (bare `temper`); the full-screen shell in skein's design
  beside `ui-text.md`; its theme, from the browser's chat terminal
  (`../client/browser.md`, 2.5).
- **Builds:** skein's full-screen shell, its renderer and its terminal;
  the TUI's view and host on temper's client root; bare `temper` and
  `temper ADDRESS`; the sign-in screen with its pasted link; the first
  screen, live.
- **Tests:** the renderer's frames for the first screen at two sizes,
  light and dark; the host's seam in the client's joint world, a TUI
  signed in by a link drawing `me` (`../domain/testing.md`, 6.3); the
  engine stopped and started again under it; the terminal left as found
  on each way out.
- **Walk:** on a fresh `temper box`, `temper` in another terminal, the
  box's link pasted, the first screen.
- **Done** when it lands.

### M2. Approved

- **Executes:** `tui.md`, 4.2.
- **Design:** jig's third built-in way, in jig's
  `client/domain/contract.md`, section 2, jig's `domain/people.md`,
  section 3, and jig's `protocol/client.md`, section 8; the web's
  approval page in `../client/README.md` and `../client/browser.md`, 2.3
  and 2.4; the TUI's signing in, with its own lines.
- **Builds:** jig's approval, asked, approved, refused, expired and
  taken, its credential drawn only when taken, and its bounds; the
  approval page, with its code given in its address or typed; the TUI's
  flow, its tab opened or its address and code printed.
- **Tests:** in jig's worlds, the approval's ends, a code used twice, a
  secret never in a print or a log, an approval giving the approving
  party's sign-in alone, one from a client that isn't a browser
  refused, one never taken making no sign-in; in temper's client world,
  a TUI's approval approved by a second client, signed in as a browser
  is (`../domain/testing.md`, 6.3); the address the TUI opens made from
  its own parts alone; the browser's page in the gallery's states
  (`../client/browser.md`, section 7).
- **Walk:** on a fresh `temper box`, its link opened in a browser, then
  `temper` in another terminal, approved, the first screen.
- **Done** when it lands.

### M3. Recorded

- **Executes:** `tui.md`, section 5.
- **Design:** the TUI's journeys, beside `../client/journeys.md`: the
  recorder, the replay, the print, its tests, recording again, and the
  look requested on frames.
- **Builds:** the recorder in a build of its own, the replay through the
  shipped host, the frames' print as text and pages, the reprint's
  test, recording again in a pseudo-terminal with the approval given in
  headless Chromium, the redaction's TUI originals, the theme's
  overlay; and the first journey, committed with its README.
- **Tests:** as the browser's journeys' (`../client/journeys.md`,
  section 6), within their cost; the redaction and the scan shared with
  them.
- **Walk:** the user opens the first journey's frames.
- **Done** when it lands and the user has taken the frames.
