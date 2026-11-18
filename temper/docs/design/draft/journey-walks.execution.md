# journey-walks: execution

2026-11-18. How [journey-walks.md](journey-walks.md) gets built: one milestone of ten goals. Each goal writes its own design from the draft as it runs, brings the designs it changes in line in the same landing, and lands whole, with its own tests, workspace entries and gate rules. Plans use the focused and fuzzy suites alone: no goal walks a box, and the box's walks are written and compiled, never run. A goal may run `temper-client-walk print`, which starts no process (`journey-walks.md`, 5.2).

## 1. The goals

| Goal | Draft parts | Needs | Its walk | State |
|---|---|---|---|---|
| W1, the gate lets go: the tests that reprint main's journeys and hold each recording to its normalisation removed, with the gate's entries that name a committed recording; the tests of redaction, the scan and the proof given recordings built in the host's world; `AGENTS.md`, the journeys' designs and the look requests' saying what holds until the last journey has moved | 8.3, 10 | none | none: the suites | not started |
| W2, the walk in a world, the browser: the lines' grammar and the printers moved to a library free of the browser kit; the world's driver moved out of the playground's test crate, with the page's HTML, a reload and the wait for a control; the `walk` of version 2 and its reader; the walker; `temper-client-walk print` and its scratch layout; the cost of a walk measured. `welcome-playground` gains its walk, its recording staying | 3, 4.1, 5 | W1 | none: the suites | not started |
| W3, one writer: `temper-client-walk pages` and its `Printed:` line; the script that runs it one at a time; the delivery check's two refusals; the merge job's rule for a page; `look-refresh.sh` going by the print; `AGENTS.md`'s line. `welcome-playground` moves: its recording goes, its pages the writer's | 6, 7, 8.3 | W2 | none | not started |
| W4, the box in a world: a peer that reports an owned forge's set-up step by step. `welcome-set-up` moves | 4.1 | W3 | none | not started |
| W5, the browser's longer walks: a tab carried to the fake issuer and back; `scripts.json` along a chain. `project-landing` moves, and `chat-draft` returns by its renewed walk (`../client/web/journeys.md`, 10) | 3.3, 4.1 | W4 | none | not started |
| W6, the TUI on the service: its host's requests, stream, kept records, deadlines and browse carried to the simulated service; `tui` lines, `tui approve` and frames. `sign-in-approved` moves | 3.2, 4.1, 5.1 | W3 | none | not started |
| W7, the TUI's longer walks: `launchpad-chat`, `launchpad-drawn` and `design-session` move, with their scripts | 3.3, 4.1 | W5, W6 | none | not started |
| W8, the old way removed: `again`, `normalise`, the placeholders, the `setup` and version 1 readers; `temper-client-journeys print FILE OUT` in place of `add`; the drive session taking a journey's lines, its box walks written and compiled | 4.2, 4.3, 8.1 | W7 | none: compiled | not started |
| W9, the fixed start removed from the box, the page and the TUI, with `../client/journey-clock.md`, if `journey-walks.md`, 12, keeps its default | 8.1 | W8 | none | not started |
| W10, the words that are left: `../client/web/journeys.md` and `../client/tui/journeys.md` read whole around the walk; the drafts `journey-clock.md`, `journey-placeholders.md`, `real-journeys.md` and `record-anywhere.md` folded or deleted | all | W1 to W9 | none: docs | not started |

## 2. Order

- **W1 first, alone, at once.** It builds nothing and ends the conflicts: from its landing no goal records a journey, and a change to the wire has no journey file to touch.
- **What waits between W1 and a journey's move:** that journey's pages are held current by no one. A look request drawn on them is compared by the reprint while its recording still replays, and by the render check after landing once it does not. No journey is added for a client until that client walks in a world: the browser at W2, the TUI at W6.
- **W2 next:** every later goal stands on the two libraries, the reader and the walker.
- **W3 before any journey moves,** so that a moved journey's pages always have their writer. Its first run, after its own landing, writes `welcome-playground`'s pages.
- **W4, then W5,** each adding what the next browser journeys need. **W6 beside them,** after W3: the TUI's host on the service is the largest new piece, and lands with one short journey before the three long ones.
- **W7 after W5 and W6:** its journeys start `after project-landing`.
- **W8 once the last journey has moved.** **W9 and W10 close the milestone.**

## 3. Acceptance

A moved journey's own proof is the writer's first run on it, after landing. In a plan, each goal holds what its journeys need by tests on small walks of its own, and prints its journeys into scratch as it works.

- **W1:** no test replays, reprints or compares a committed recording or page; a recording made stale by hand fails none; `tools/tiers/budgets.toml` names no committed recording; the scans that read every tracked file still pass.
- **W2:** a walk of the tests' own prints the same bytes twice, in one process with no child and no socket; a control that appears late is found, and one that never does stops the walk by its journey and line, the others still printed; a reload keeps the small store; a page opens from the scratch directory with its stylesheets; the binary's dependencies hold no browser kit; a journey's measured cost is in the goal's account, and its tests sit in their layers within the budgets as they stand.
- **W3:** `pages` on a tree whose pages are current writes nothing; with one word of the view changed it writes exactly the pages that draw it; a second run while a change of its own is open starts none; a directory with no `walk` of version 2 is left as it is, and its pages stay a goal's to change; the delivery check refuses a page changed beside another file, with and without a base, and a nonempty overlay; the merge job settles a page's conflict with no model; the refresh keeps a request whose trailer is on main and not yet printed.
- **W4:** an owned set-up's steps are drawn as the peer reports them, in order, on the simulator's time, with no step through ctl.
- **W5:** a walk with a tab reaches the page the sign-in returns to, its account connected through the client; scripts join along a chain of two.
- **W6:** the TUI's host draws a chat's streamed reply from the simulated service; `tui approve` approves what the TUI browsed, and a TUI not approved stays on its approval; a recipe's walk signs in as before.
- **W7:** each journey's scripts read, and a walk of the tests' own goes through a delegated question and a pane's keys.
- **W8:** no `recording` or `setup` under a journeys' directory; the tool has no `again`, `normalise` or `add`; `print FILE OUT` refuses an OUT under the repository's journeys, and a file recorded at another layout by that layout's name.
- **W9:** no `--fixed-start`, `internal-fixed-start` or `TEMPER_INTERNAL_FIXED_START` in the code or its help.
- **W10:** the citations' gate passes, and each status file says what still lags.
