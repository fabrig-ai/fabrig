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

## 2. How close the in-process worlds are

A read of the code, with no test run. A journey walked in one process needs four things: a world that runs the engine, a client's own host in front of it, a person who acts on that client by role and name, and a printer for the page at a mark. Each exists, in separate places.

### 2.1 The pieces

| Piece | Where | What it lacks for a journey |
|---|---|---|
| The whole service on the simulator: `temper-service`, hosted agents over the fake LLM, the fake forge, the fake git program | `temper/tests/one-process`, on `temper-playground`'s composition (`temper/docs/design/testing/README.md`, 2.5) | its person's client is the terminal client, whose view and patches are private (`temper/crates/temper-client-shell-terminal/src/client.rs:49`); a reload panics (`temper/tests/one-process/src/world.rs:847`) |
| The web shell's own host against that service | `temper/tests/playground/src/page.rs:99`: a printed-link start, `Report::Act` and `Report::Place`, fetches and the stream on a memory platform; `story_a_walks_from_the_welcome_to_the_csv_change_landed` (`temper/tests/playground/tests/suite/walk.rs:60`) | its surface is private and gives no HTML (`temper/tests/playground/src/page.rs:39`); a reload panics (`temper/tests/playground/src/page.rs:160`) |
| An owned forge set up from the welcome | `temper/tests/one-process/src/ready_copies.rs:274`: the world plays the box over the service's pipes, and the person presses "Run on this machine", reaches Ready and creates the project | the walk's own repository and words |
| The TUI's shipped host with no terminal | `temper/tests/client/src/tui.rs:126`: keys given, duties carried, frames read as rows (`:218`); a browser's approval joined (`temper/tests/client/tests/suite/tui.rs:68`) | it runs on the client's joint world, where the engine's hosts are the world's; the one-process world has no TUI |
| A person by role, name and order | skein's fake person: `Find::exactly_named(role, name).nth(order)`, `Find::text`, and steps press, type, send, see, gone, go, reload (`skein/testing/skein-fake-person/src/lib.rs:40`, `:83`); the drive session finds the same way (`temper/crates/temper-client-journeys-tool/src/drive.rs:431`) | no step for a key, an approval or a mark; `textbox` is a text field to the lines' reader and a text area in the native chat (`temper/crates/temper-client-journeys-tool/src/line.rs:238`, `temper/tests/one-process/src/world.rs:549`) |
| The lines' reader | `Line::parse` gives typed records with no io (`temper/crates/temper-client-journeys-tool/src/line.rs:90`); the drive session is a machine of events in and actions out (`src/drive.rs:75`, `:246`), its io apart (`src/drive/io.rs:496`) | no trait behind which a second backend sits; the actions name the box's processes and files |
| A page's print | `print::pages` takes each mark's HTML, heading and acted nodes, and needs no recording (`temper/crates/temper-client-journeys-tool/src/print.rs:9`); the TUI's takes each mark's frame (`src/tui/print.rs:15`) | nothing |
| ctl in a world | a one-shot ctl over memory io in both worlds (`temper/tests/client/tests/suite/service_party/machine.rs:100`; `temper/tests/one-process/src/ctl.rs:38`) | a recipe's `ctl` lines, their substitutions and files |

### 2.2 The eight journeys

| Journey | In a world today | What is missing |
|---|---|---|
| `welcome-playground` | walks: the one-process story A's opening reaches the playground's new chat (`temper/tests/one-process/src/stories/playground.rs:33`) | marks and the print |
| `welcome-set-up` | small additions | the welcome's clicks joined to an owned set-up whose progress the world plays (`temper/tests/client/tests/suite/welcome_m3_owned.rs:102`) |
| `project-landing` | real work | the set-up joined to the plan's sign-in in a tab, Ready and a reload at an address |
| `chat-draft` | real work | its renewed walk (`temper/docs/design/client/web/journeys.md`, 10), the fake model's scripts for a draft and its change, the three modes, the goal and the way back |
| `sign-in-approved` | small additions | a deployment with no project, and the frames' marks (`temper/tests/client/tests/suite/tui.rs:68`) |
| `launchpad-chat` | small additions | Hello, its model and the scripted reply as the walk has them (`temper/tests/client/tests/suite/tui.rs:132`) |
| `launchpad-drawn` | small additions | `/change`, the delegated question and `/new` joined (`temper/tests/client/tests/suite/tui.rs:198`) |
| `design-session` | real work | the drafter's scripts, the pane's keys, the answer and `/goal` through the TUI's host |

### 2.3 Time, seed and what varies

- **Each world owns its clock and seed.** The one-process world starts at a wall and a seed its settings give (`temper/tests/one-process/src/world.rs:1232`), and below the real loop the same inputs replay step for step (`skein/docs/foundation/testing-strategy.md`, 2).
- **What a box varies is a world's choice:** the ports are the addresses it configures, the build is the start's `version` fact, and an approval's code comes from the seed. The journeys' pages draw each of them today (`temper/docs/design/client/journey-placeholders.md`, 2).
- **What a world does not show:** an owned Forgejo fetched and run, Chromium's own tab, download and layout, and a pseudo-terminal's bytes. A printed page depends on none of them but the set-up's progress, which a world plays.

### 2.4 Cost

- **The one-process world's focused stories:** 51 tests in 27.9 CPU seconds at `bcddfe1ff1`, about half a second each (`temper/docs/development/limits-worlds.md`).
- **Its layer's budget,** simulated: 470 CPU seconds focused and 196 fuzzy; the client's joint world's, system: 111 and 13 (`tools/tiers/budgets.toml`).
- **Two budget entries name a committed recording** (`tools/tiers/budgets.toml:78` to `:88`).
- **Not measured:** a journey's walk in a world.
