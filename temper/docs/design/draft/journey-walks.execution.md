# journey-walks: execution

2026-11-18. How [journey-walks.md](journey-walks.md) gets built: one milestone of eight goals. Each goal writes its own design from the draft as it runs, and lands whole, with its own tests, workspace entries and gate rules. Plans use the focused and fuzzy suites alone: no goal walks a box, and the box's walks are written and compiled, never run.

## 1. The goals

| Goal | Draft parts | Needs | Its walk | State |
|---|---|---|---|---|
| W1, the gate lets go: the tests that reprint main's journeys and hold each recording to its normalisation removed, with the gate's entries that name a committed recording; the tests of redaction, the scan and the proof given recordings built in the host's world; `AGENTS.md` and the journeys' designs saying that no goal records a journey and that a stale recording is no red. Main's recordings and pages stay as they are | 6, 8, 10 | none | none: the suites | not started |
| W2, the walk in a world, the browser: the `walk` file of version 2 and its reader; the world of 4.1 with the web shell's host, its reload and its facts; `temper-client-walk print`; `welcome-playground` and `welcome-set-up` rewritten as walks and printed, the owned forge's set-up played by the world; the print's cost measured | 3, 4.1, 5 | W1 | none: the suites | not started |
| W3, the browser's longer walks: a tab carried to the fake issuer and back; `scripts.json` along a chain; `project-landing`, and `chat-draft` by its renewed walk (`../client/web/journeys.md`, 10) | 3.3, 4.1 | W2 | none | not started |
| W4, the TUI in a world: the TUI's host in front of the same world, `tui` lines, `tui approve` and frames; `sign-in-approved`, `launchpad-chat`, `launchpad-drawn` and `design-session` | 3.2, 4.1, 5 | W2 | none | not started |
| W5, one writer: `temper-client-walk pages`; the delivery check's two refusals; `AGENTS.md`'s line; every `recording` and `setup` removed, with `again`, `normalise` and the placeholders; main's pages written once by `pages`, in this goal's last change | 6, 8 | W3, W4 | none | not started |
| W6, a recorded file: `print FILE OUT` in place of `add`, refusing a journeys' directory and naming the layout it needs; the drive session taking `mark` and `tui approve`, its box walks written and compiled | 4.2, 4.3 | W5 | none: compiled | not started |
| W7, the fixed start removed from the box, the page and the TUI, if section 12 keeps its default | 8 | W5 | none | not started |
| W8, the words: `../client/web/journeys.md` and `../client/tui/journeys.md` rewritten around the walk; `../client/journey-clock.md`, `../client/journey-placeholders.md` and the drafts they came from deleted; `../client/web/look-requests.md`, 2 and 3; `../client/drive.md`; `../testing/README.md` and `../testing/tools.md`; `real-journeys.md` and `record-anywhere.md` folded or deleted | all | W1 to W7 | none: docs | not started |

## 2. Order

- **W1 first, alone, at once.** It builds nothing and ends the conflicts: from its landing no goal walks a journey, and a change to the wire touches no journey file. Main's pages go unrefreshed until W5.
- **W2 next:** every later goal stands on the world and the reader.
- **W3 and W4 side by side,** after W2: one adds what the browser's longer walks need, the other the TUI.
- **W5 after both,** so that the writer's first print holds all eight journeys, and no journey is left on main without a walk that prints.
- **W6 and W7 after W5,** in either order.
- **The words close the milestone.**

## 3. Acceptance

- **W1:** no test in either suite opens a file under a journeys' directory; a recording made stale by hand fails none; `tools/tiers/budgets.toml` names no committed recording.
- **W2:** the two journeys print the same bytes twice, in one process with no child and no socket; a walk whose line finds no control stops by its journey and line, the other still printed; a reload keeps what the small store held.
- **W3:** `project-landing` and `chat-draft` print; a walk with a tab reaches the page the sign-in returns to.
- **W4:** the four TUI journeys print their text and their pages; `tui approve` approves what the TUI browsed, and a TUI not approved stays on its approval.
- **W5:** no `recording` or `setup` under a journeys' directory; `pages` on main's tree writes nothing; with one word of the view changed it writes exactly the pages that draw it; the delivery check refuses a page changed beside another file and a nonempty overlay.
- **W6:** `print FILE OUT` refuses an OUT under a journeys' directory; a file recorded at another layout is refused by that layout's name.
- **W7:** no `--fixed-start`, `internal-fixed-start` or `TEMPER_INTERNAL_FIXED_START` in the tree.
- **W8:** the citations' gate passes, and each status file says what still lags.
