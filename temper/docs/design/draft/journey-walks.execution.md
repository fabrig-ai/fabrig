# journey-walks: execution

2026-11-18. How [journey-walks.md](journey-walks.md) gets built: one milestone of nine goals. Look requests, the browser's and the TUI's, are drawn and landed all the while (`journey-walks.md`, 8.3), so the one writer comes first and each journey moves to the world under it. Each goal writes its own design from the draft as it runs, brings the designs it changes in line in the same landing, and lands whole, with its own tests, workspace entries and gate rules. Plans use the focused and fuzzy suites alone: no goal walks a box, and the box's walks are written and compiled, never run. A goal may run `temper-client-walk print` and the journeys' reprint, which start no process (`journey-walks.md`, 5.2).

## 1. The goals

| Goal | Draft parts | Needs | Its walk | State |
|---|---|---|---|---|
| W1, one writer, by the means there are: the tests that reprint main's journeys and hold each recording to its normalisation removed, with the gate's entries that name a committed recording; the writer's command over `again` and the reprint, its `Printed:` line and the script that runs it one at a time; `look-refresh.sh` going by the print; the delivery check's refusals and the merge job's rule, for every journey file; the tests of redaction, the scan and the proof given recordings built in the host's world; `AGENTS.md` and the look requests' landing saying who writes a journey's files | 6, 7, 8.3, 10 | none | none: the suites, the writer's box walks compiled | not started |
| W2, the walk in a world, the browser: the lines' grammar and the printers moved to a library free of the browser kit; the world's driver moved out of the playground's test crate, with the page's HTML, a reload and the wait for a control; the `walk` of version 2 and its reader; the walker; `temper-client-walk print` and `pages`, the writer taking them for a moved journey; the cost of a walk measured. `welcome-playground` moves | 3, 4.1, 5, 6 | W1 | none: the suites | not started |
| W3, the box in a world: a peer that reports an owned forge's set-up step by step. `welcome-set-up` moves | 4.1 | W2 | none | not started |
| W4, the browser's longer walks: a tab carried to the fake issuer and back; `scripts.json` along a chain. `project-landing` moves, and `chat-draft` returns by its renewed walk (`../client/web/journeys.md`, 10) | 3.3, 4.1 | W3 | none | not started |
| W5, the TUI on the service: its host's requests, stream, kept records, deadlines and browse carried to the simulated service; `tui` lines, `tui approve` and frames. `sign-in-approved` moves | 3.2, 4.1, 5.1 | W2 | none | not started |
| W6, the TUI's longer walks: `launchpad-chat`, `launchpad-drawn` and `design-session` return, with their scripts | 3.3, 4.1 | W4, W5 | none | not started |
| W7, the old way removed: `again`, `normalise`, the placeholders, the `setup` and version 1 readers, the writer's first engine; `temper-client-journeys print FILE OUT` in place of `add`; the drive session taking a journey's lines, its box walks written and compiled | 4.2, 4.3, 8.1 | W6 | none: compiled | not started |
| W8, the fixed start removed from the box, the page and the TUI, with `../client/journey-clock.md` | 8.1 | W7 | none | not started |
| W9, the words that are left: `../client/web/journeys.md` and `../client/tui/journeys.md` read whole around the walk; the drafts `journey-clock.md`, `journey-placeholders.md`, `real-journeys.md` and `record-anywhere.md` folded or deleted | all | W1 to W8 | none: docs | not started |

## 2. Order

- **W1 first, alone, at once.** It ends the conflicts: from its landing no goal records a journey or commits a journey file, and a change to the wire has none to touch. The writer keeps main's pages current after each landing, walking boxes as `again` does today.
- **Look requests go on through every goal.** A request is drawn on main's pages. Whoever lands it compares by the reprint in its own tree, or, for a journey that has moved, by `temper-client-walk print`. Nothing in this order holds a request back.
- **What W1 does not mend:** the four journeys with no pages today stay without until they move, at W4 and W6, unless a repair of the box's walks lands sooner.
- **W2 next:** every later goal stands on the two libraries, the reader and the walker. From it the browser's journeys leave the box one at a time.
- **W3, then W4,** each adding what the next browser journeys need. **W5 beside them,** after W2: the TUI's host on the service is the largest new piece, and lands with one short journey before the three long ones.
- **W6 after W4 and W5:** its journeys start `after project-landing`.
- **W7 once the last journey has moved.** **W8 and W9 close the milestone.**

## 3. Acceptance

A moved journey's own proof is the writer's first run on it, after landing. In a plan, each goal holds what its journeys need by tests on small walks of its own, and prints its journeys into scratch as it works.

- **W1:** no test replays, reprints or compares a committed recording or page; a recording made stale by hand fails none; `tools/tiers/budgets.toml` names no committed recording; the scans that read every tracked file still pass. The writer's command, given a tree whose journeys are current, writes nothing; given a stale recording, it asks for that journey's walk alone; a second run while a change of its own is open starts none. The delivery check refuses a journey file changed beside another file, with and without a base, and a nonempty overlay; the merge job settles a journey file's conflict with no model; the refresh keeps a request whose trailer is on main and not yet printed, and drops it once a `Printed:` commit covers it.
- **W2:** a walk of the tests' own prints the same bytes twice, in one process with no child and no socket; a control that appears late is found, and one that never does stops the walk by its journey and line, the others still printed; a reload keeps the small store; a page opens from the scratch directory with its stylesheets; `pages` with one word of the view changed writes exactly the pages that draw it, and leaves a directory with no `walk` of version 2 to the other engine; the binary's dependencies hold no browser kit; a journey's measured cost is in the goal's account, and its tests sit in their layers within the budgets as they stand.
- **W3:** an owned set-up's steps are drawn as the peer reports them, in order, on the simulator's time, with no step through ctl.
- **W4:** a walk with a tab reaches the page the sign-in returns to, its account connected through the client; scripts join along a chain of two.
- **W5:** the TUI's host draws a chat's streamed reply from the simulated service; `tui approve` approves what the TUI browsed, and a TUI not approved stays on its approval; a recipe's walk signs in as before.
- **W6:** each journey's scripts read, and a walk of the tests' own goes through a delegated question and a pane's keys.
- **W7:** no `recording` or `setup` under a journeys' directory; the tool has no `again`, `normalise` or `add`; `print FILE OUT` refuses an OUT under the repository's journeys, and a file recorded at another layout by that layout's name.
- **W8:** no `--fixed-start`, `internal-fixed-start` or `TEMPER_INTERNAL_FIXED_START` in the code or its help.
- **W9:** the citations' gate passes, and each status file says what still lags.
