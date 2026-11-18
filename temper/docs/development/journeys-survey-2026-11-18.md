# Journeys: a survey, 2026-11-18

The figures behind `temper/docs/design/draft/journey-walks.md`. Read on main at `7307b6172b`. Section 1 is what the committed journeys have cost, from the repository's history; section 2 is how close the in-process worlds are to walking a journey, from a read of the code. No test, walk or build was run for either.

## 1. What the committed journeys cost

### 1.1 What main holds

- **Four recordings and fourteen printed pages,** 235 KB in all, under `temper/docs/design/client/web/journeys/` and `temper/docs/design/client/tui/journeys/`: `welcome-set-up`, `welcome-playground`, `project-landing` and the TUI's `sign-in-approved`.
- **Four journeys with no recording:** the TUI's `design-session`, `launchpad-chat` and `launchpad-drawn` keep their `walk`, `setup` and README alone, since their walks stalled on a step when they were last walked. `chat-draft` was removed on 11-13 by the landing of the `chat-bar` look request, and has not been walked since.
- **The sources are 99 lines:** every `walk` and `setup` file together.
- **A recording is the wire's bytes.** `welcome-set-up`'s 949 lines are 296 turns, each with its clocks and its deadline, 266 `heard` records and the asks between them. An ask's path names the wire's version (`/jig/v1/stream?version=34`), and a `heard` record holds the answer's bytes, escaped.

### 1.2 How often they changed

Counted on main's first-parent line since 2026-11-02, 1,704 commits:

| What | Figure |
|---|---:|
| Commits that rewrote a recording | 101 |
| Lines changed in journey files | 416,809 |
| The same, as a share of every line changed on main | 6.7% |
| Of those lines, in recordings | 342,231 |
| In the 101 commits, the median share of the diff that is journey files | 31% |
| Of the 101, commits where journey files are over half the diff | 27 |

- **Versions kept in history,** on main since 10-26: 395 of the eight recordings, 93 MB before compression, between 33 and 76 of each; and 2,957 of printed pages, 16 MB.
- **On every branch,** since 11-01: 184 commits that are not merges rewrote a recording.

### 1.3 Why they changed

- **Landings that draw nothing new.** Among the 101: "Count every open needs row across inbox pages" (`34216f8e77`), "Name tasks waiting after a program starts without its tools" (`20c12e6166`), "A signed-in client asks for a fresh sign-in link" (`5a5f31cb5f`), the store's S1, the merge queue's M2 and M3, the landing history's H1.
- **Their diffs.** In `20c12e6166`, 6,015 of 6,630 changed lines are journey files; in `34216f8e77`, 1,648 of 2,401. In both, the printed pages change by 2 to 4 lines each, a time a page draws, while each recording is rewritten nearly whole.
- **The wire's version.** jig's client wire went from version 19 on 11-10 (`temper/docs/design/draft/layouts-both-ways.md`, 1) to 34 on 11-18. A recording's asks name the version, so each move stales every journey (`temper/docs/design/client/web/journeys.md`, 4.2).
- **What a page asks, and when.** Each milestone of `temper/docs/design/draft/page-round-trips.md` changed the exchanges of the pages it touched, and staled the journeys that open them.
- **Commit subjects on goal branches** say the same: "Refresh journeys for the account fact wire", "Refresh journeys for the dismissal watch exchange", "Resolve journey conflicts with normalized streamed-me recordings", "Refresh journey goldens after resolving the stale-update merge".

### 1.4 What walking again takes

- **A box, a browser and the network.** `again` builds `temper-box`, `fake-claude` and the bundle, starts a fake box and headless Chromium, and for the journeys that set a forge up fetches Forgejo from codeberg.org (`temper/docs/design/client/web/journeys.md`, 8.1).
- **11-18:** codeberg.org's name did not resolve from the builders for some hours. One goal's walk failed there; R4 of `page-round-trips.md` removed six stale journeys in place of walking them (`40728f6131`); and a bug fix left out a client-wire change so as not to stale two.
- **Not measured:** how long a walk takes.

### 1.5 What the tooling is

| Part | Lines of Rust, tests among them |
|---|---:|
| `temper/crates/temper-client-journeys-tool`, without its drive session and recipes | 6,141 |
| `temper/bin/temper-client-journeys` | 594 |
| `temper/crates/temper-client-shell-web/src/journey` | 4,929 |
| `temper/crates/temper-client-shell-tui/src/journey` | 4,530 |
| `temper/crates/temper-client-redaction` | 1,116 |
| **The journeys' own** | **17,310** |
| The tool's drive session and recipes (`src/drive*`, `src/recipe*`), QA's | 9,915 |

About half of the journeys' own lines are tests. Their design is five documents, 147 KB: `client/web/journeys.md`, `client/tui/journeys.md`, `client/journey-clock.md`, `client/journey-placeholders.md` and `client/web/journey-redaction.md`.

### 1.6 Three languages for one walk

- **The browser's `setup`** (`temper/docs/design/client/web/journeys.md`, 8.2): `press link "Set up temper"`, `see "..."`, `reload "/projects/1"`. What follows the reload is no file: `again` reads the acts back from the recording.
- **The TUI's `walk`** (`temper/docs/design/client/tui/journeys.md`, 8.1): `key enter`, `see "..."`, `mark`, `approve`.
- **QA's recipes** (`temper/docs/design/client/recipe.md`; `temper/docs/design/client/drive.md`, 3): `web press link "..."`, `tui key enter`, `ctl project make Hello`, which wrap both of the others' records behind a client's name.

`welcome`, `project-landing` and `chat-draft` exist as a recipe under `temper/docs/development/qa/setups/` and as a journey.
