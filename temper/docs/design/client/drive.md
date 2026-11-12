# Driving the clients a line at a time

Provisional, 2026-11-12. QA's hands on the web client and the TUI
(`../draft/landed-qa.md`, 7 and 10): `temper-client-journeys drive`, a
session that starts a fake box from a given build in a given scratch
directory, then takes lines on standard input, each a step a person
takes in the browser, the TUI or ctl, or a look at what they are shown,
and answers each. It runs no build and writes nothing outside its
scratch directory. Recipes are walked through it (`recipe.md`). The
journeys' `again` (`web/journeys.md`, 8.1; `tui/journeys.md`, 8.2) stays
a person's command that builds from the checkout: it records into the
checkout, which a session never touches. Section 2 is the session, 3
its lines, 4 the lines it keeps, 5 its code, 6 its tests, 7 what is
open.

## 1. In one page

- **A session is one process and one box:** started from a given build
  (`--bin`, `--bundle`), in a given scratch directory, the box's
  directories kept there across sessions, so that a place's restored
  copy starts as a fresh box does (2.1 and 2.2).
- **Signed in by the session, three ways:** ctl with the box's printed
  link, the browser with the fresh link serve prints when the spent one
  is opened, and the TUI, when a line first needs it, by the browser's
  approval (2.3 and 3.3). No line signs in.
- **Lines are the recipes' words:** `web` and a journeys' setup record,
  `tui` and a walk's record, `ctl` and a ctl act, and `look`, which
  answers the page's accessible text and a screenshot, or the TUI's
  rows (section 3).
- **Each line answered,** `ok` or `failed: WHY`, what it shows prefixed
  `| `; a failed line never ends the session (3.6).
- **Kept lines:** every line but `look`, in order, in `session/lines`,
  so a case's steps become a recipe's and a finding's reproduction
  (section 4).
- **Its end stops the box cleanly,** so that its directories are a
  stopped box's, ready to copy; its exit status says whether the box
  started, was refused by its store, or failed (2.4 and 2.5).
- **Its logic has no io,** as `again`'s walk has none, and its tests are
  focused: no browser, no box (sections 5 and 6).

## 2. The session

### 2.1 Its command

```text
temper-client-journeys drive --bin DIR --bundle DIR --scratch DIR [--scripts FILE]... [--strict]
```

- **`--bin DIR`** holds `temper-box`, `fake-claude` and `temper-client`,
  each an executable regular file. The client is the TUI's recording
  build (`tui/journeys.md`, 2), whose saves the TUI's lines read
  (3.3). The box finds `fake-claude` beside itself
  (`../protocol/setup/box.md`, 5).
- **`--bundle DIR`** is the loopback bundle (`scripts/bundle.sh
  --loopback`), given to the box as its `--bundle`.
- **`--scratch DIR`** is an existing directory, the user's own and not
  open to others; every directory and file the session or what it
  starts writes is under it (2.2).
- **`--scripts FILE`, `--strict`** are given to the box as they are
  (`../protocol/setup/box.md`, 5), its fake model's scripts and strict
  switch; a recipe's walk gives its chain's scripts (`recipe.md`, 5).
- **Chromium** comes from `SKEIN_TEST_CHROMIUM`, as the browser tier and
  `again` take it.
- **Each path** is taken relative to the working directory and made
  absolute before anything starts. A file of `--bin` missing, a bundle
  without its `index.html`, a scratch directory that is not a
  directory, or no Chromium refuses the session, naming it, before
  anything starts.
- **It runs no build:** the programs it starts are the box, the client
  and Chromium, and what they start; never `cargo`, `scripts/bundle.sh`
  or another program of the checkout.

### 2.2 Its scratch directory

| Path | What | Kept |
|---|---|---|
| `config/`, `data/`, `cache/` | the box's three directories (`../protocol/configuration/directories.md`), given as `--config`, `--data` and `--cache` | across sessions: absent, the box prepares a new sandbox; present, it starts the one they hold, as a place's restored copy (`../draft/landed-qa.md`, 5) |
| `session/` | the session's own, removed and made again, 0o700, at its start | until the next session |
| `session/home/`, `session/state/`, `session/tmp/` | `HOME`, `XDG_STATE_HOME` and `TMPDIR` of every program the session starts; `XDG_CONFIG_HOME`, `XDG_CACHE_HOME` and `XDG_DATA_HOME` under `home/` | |
| `session/runtime/`, `session/tui-runtime/` | `XDG_RUNTIME_DIR` of the box and of the TUI, apart, as `tui/journeys.md`, 8.2 keeps them | |
| `session/ctl/` | ctl's directory (`ctl/README.md`, 6), which its sign-in makes | |
| `session/browser/` | Chromium's profile and downloads | |
| `session/bin/` | `xdg-open` and `open`, links to `true`, first in the TUI's `PATH` (`tui/journeys.md`, 8.2) | |
| `session/box.log` | the box's stdout and stderr, each line as it comes | |
| `session/looks/` | each look's screenshot, `NNNN-WIDTH.png`, NNNN the line's number from 1 | |
| `session/lines` | the kept lines (section 4) | |

The box's own environment is the session's: the `PATH` the session was
given, and the directories above. The TUI's is `tui/journeys.md`, 8.2's,
in the session's directories.

### 2.3 Its start

In order, each within its bound; a failure is 2.5's.

1. **The box:** `BIN/temper-box --fake-backends --bundle BUNDLE --config
   SCRATCH/config --data SCRATCH/data --cache SCRATCH/cache`, then each
   `--scripts FILE` and `--strict` as given, its stdin closed, its
   stdout and stderr read a line at a time into `session/box.log`. Its
   first link is the first stdout line starting `http://`
   (`../protocol/setup/box.md`, 2), within 60 seconds.
2. **ctl:** `BIN/temper-client ctl SCRATCH/session/ctl sign-in -`, the
   link on its stdin and then stdin's end (`ctl/README.md`, 6.1 and 8),
   exit 0 within 60 seconds. It waits on serve's start as a link opened
   early waits (`../protocol/setup/box.md`, 2).
3. **The browser:** headless Chromium, as the browser tier drives it
   (`web/testing.md`; skein's `ui/web/testing.md`, 2), opens the first
   link, now spent, so that serve prints a fresh link on stdout
   (`../protocol/setup/box.md`, 2) within 30 seconds; it opens that one,
   then loads `/` on the box's origin and waits until the page shows the
   client's main region, within 60 seconds. So the browser holds the
   session's sign-in, a copy holds none (`../draft/landed-qa.md`, 5),
   and every session's first line meets the box's root page.
4. **The TUI** starts at the session's first `tui` line (3.3), not
   before.
5. **`ready`** on stdout, and the lines are read.

While any step waits, the box's exit, or a line on its stderr saying a
step is restarting or the box gives up (`../protocol/setup/box.md`, 2),
ends the wait as the box's failure.

### 2.4 Its end

At a line `end`, or at stdin's end: the TUI ended, its process group
killed, as `again` ends it; the browser closed; then the box stopped by
`SIGTERM` and waited on, within 60 seconds, and its tree killed past
them. After a clean stop the box's directories hold a stopped box,
which a place copies (`../draft/landed-qa.md`, 5). The session's
directory stays until the next session.

### 2.5 Its exit

| Status | When | Its last lines on stdout |
|---|---|---|
| 0 | the session reached its end and the box stopped cleanly, whatever its lines answered | `ended` |
| 3 | the box's store was refused by name: serve's log, under the box's logs (`../protocol/setup/box.md`, 2), holds a line starting `temper serve: ` that names a directory under the scratch directory, as every refusal of a store does (`../protocol/store.md`, 5; jig's `protocol/store/records.md`, 3); or a fake forge's log there holds a line starting `temper-box fake forge: state ` that names a file under the scratch directory, its kept state refused (`../testing/fakes.md`, 2) | `refused: ` and that line |
| 4 | the box did not start or stop otherwise: it exited, gave up or panicked, a link or the page did not come within its bound, ctl's sign-in failed, or its stop took past its 60 seconds | `box failed: WHY`, then the last 20 lines of `session/box.log` and of serve's log, each prefixed `\| ` |
| 1 | the session's own failure: a refused command line (2.1), a scratch directory it cannot make its own | `failed: WHY`, on stderr |

A place makes a copy refused by its store again from its recipe
(`../draft/landed-qa.md`, 5): status 3 is that signal. Status 4 on a
fresh box, or on a copy past its closing check, is a finding against
temper (`../draft/landed-qa.md`, 7).

## 3. Lines

### 3.1 Their form

- **One line each,** a recipe's step (`recipe.md`, 3): its client first,
  `web`, `tui` or `ctl`, then its record and its fields, as the
  journeys' file writes fields (`web/journeys.md`, 3.2): each after one
  space, a word bare, bytes in double quotes. Or a look (3.2 to 3.3), or
  `end`.
- **An empty line** is answered `ok` and not kept.
- **A line that does not parse** is answered `failed: line N: WHY`,
  naming what it expected, and is not kept: it was never carried out.
- **Numbered** from 1, every line read, empty ones among them.

### 3.2 The browser's lines

| Line | What the session does | Bound |
|---|---|---|
| `web press ROLE NAME [ORDER]`, `web submit ...`, `web toggle ...` | finds the control by its role, its accessible name and its order from 0, as `again`'s setup finds it (`web/journeys.md`, 8.2), and acts on it as a person | 60 s to find it |
| `web input ROLE NAME VALUE [ORDER]` | the field's value selected and VALUE typed | 60 s |
| `web see TEXT` | waits until the page shows TEXT | 60 s |
| `web reload PATH` | loads PATH, with its query, on the box's origin, and waits for the client's main region | 60 s |
| `web look` | answers the page's accessible text, the browser kit's snapshot, a line per node (skein's `testing/browser.md`, 6) | 10 s |
| `web look WIDTH` | first the path of a screenshot of the page at WIDTH CSS pixels wide, its height the page's viewport's, written to `session/looks/`, then the text as `web look`; the viewport restored after | 10 s |

A role is a word of skein's roles, lowercase, as `web/journeys.md`, 8.2
has it. WIDTH is 320 to 3840.

### 3.3 The TUI's lines

The TUI starts at a `tui` line when none runs: `BIN/temper-client
http://HOST:PORT`, at the box's origin, in a pseudo-terminal of 80
columns by 24 rows, its environment `tui/journeys.md`, 8.2's in the
session's directories, its output read and dropped. The session signs
it in at once, as a walk's `approve` does (`tui/journeys.md`, 8.1): a
save that holds a `browse`, whose address the browser opens and whose
"Approve" it presses, then the browser loads again the address it was
on; within 30 seconds, before the line is carried out. So no line
approves, and a recipe holds none.

| Line | What the session does | Bound |
|---|---|---|
| `tui key KEY` | KEY as the journeys' file writes a key (`tui/journeys.md`, 3.2), given through the terminal as a person types it | — |
| `tui see TEXT` | waits until the TUI's frame shows TEXT: a save, replayed natively to its end, gives the frame, as a walk's `see` does (`tui/journeys.md`, 8.1) | 30 s |
| `tui stop` | ends the TUI as the session's end does, `ok` when none runs; the next `tui` line starts it again, signed in afresh | 10 s |
| `tui look` | answers the frame's rows, from a save replayed, a row a line, trailing spaces cut | 10 s |

A save that writes no recording within 10 seconds fails its line,
saying that the session needs the client's recording build
(`tui/journeys.md`, 2).

### 3.4 ctl's lines

`ctl ARGS...` runs `BIN/temper-client ctl SCRATCH/session/ctl ARGS
</dev/null`, each field after `ctl` one argument: an act as the
signed-in person, the deployment's first owner (`ctl/README.md`, 7). It
answers ctl's stderr and then its stdout, each line prefixed `| `, then
`ok` on exit 0, else `failed: exit N`. An act and the wait on it are two
lines, as ctl waits (`ctl/tools.md`, 4):

```text
ctl task show 3 --until ended
```

A ctl line's bound is its own: a wait's timeout, and 10 seconds past
it, and a call with none 60 seconds.

### 3.5 What a line answers

For each line, in order, on stdout: what it shows, each line prefixed
`| `, then one line, `ok` or `failed: WHY`. A caller writes a line and
reads to that last one. WHY names the line's number, what it sought and
what the page or the frame showed instead, where it can: for a control
not found, the controls of that role on the page with their names; for
a `see`, the page's text or the frame's rows, cut at 4096 bytes.

### 3.6 A failed line

A failed line never ends the session: the next line is read
(`../draft/landed-qa.md`, 7, a finding never stops a pass). A session
whose box has exited answers every later line `failed: the box
exited`, and ends with status 4 (2.5).

## 4. The kept lines

- **Every line the session parsed,** but a look, `end` and an empty
  line, in order, as it was given, appended to `session/lines` once
  answered, so that a session ended by its bound or a crash keeps what
  it reached.
- **A failed line is kept,** since a reproduction ends at the line that
  failed; the answers say which failed.
- **Replayed,** they reach the same page: given to a new session on the
  same start, the same copy or a fresh box, they take the same steps.
  So a case's steps become a recipe's (`recipe.md`, 3) and a finding's
  reproduction (`../draft/landed-qa.md`, 8).

## 5. Its code

- **In `temper-client-journeys-tool`,** beside `again`: a `drive`
  module for the session and a `line` module for the lines' grammar,
  `Line`'s parse and print, which `recipe.md`'s reader shares. A line
  prints as it parses, field for field, so a kept line is the line
  given.
- **Its logic without io,** as `again`'s walk has it
  (`web/journeys.md`, 8.1): a state machine that takes each event, a
  line read, a face's answer, a save's bytes, a process's line or exit,
  a deadline, and answers what to do: start this program with these
  arguments and this environment, find this control and act, take a
  snapshot, a screenshot or a save, write this answer, stop. The io is
  a thin loop around it over skein's harness, the fake person's browser
  face, the pseudo-terminal and the processes, as `again`'s is.
- **The browser kit's snapshot, viewport and screenshot** (skein's
  `testing/browser.md`, 5 and 6) are asked through the fake person's
  harness, which gains a read for each where it has none (skein's
  `ui/web/testing.md`, 2).
- **The TUI's saves and replay** are the walk's (`tui/journeys.md`, 8.1
  and 8.2), shared, not copied.

## 6. Its tests

Focused, with no browser, no box and no Chromium:

- **A line and its effect:** each line given to the logic answers its
  act; `web press button "Start"` asks the face for the first button
  named "Start" and then presses it.
- **The grammar:** every line form parses and prints back to its bytes;
  a line with an unknown client or record, a missing field or a field
  too many is refused, naming it.
- **Kept lines replayed:** a session over a fake face that models a few
  pages, given lines with looks among them; its kept lines, given to a
  fresh session over a fresh face, reach the same page.
- **Nothing built, nothing outside:** the programs the logic starts,
  their arguments and their environments name no `cargo` and no
  `scripts/bundle.sh`, and every path they write is under the scratch
  directory.
- **Its exits:** a serve log holding a store's refusal ends with status
  3 and its line; a box that exits before its link, with 4.

## 7. Open questions

- **The TUI's size:** 80 columns by 24 rows, as the journeys'. Default:
  fixed until a case needs another.
- **The box with the TUI in front** (`tui/journeys.md`, 8.3): not
  driven. Default: none until a case needs it.
- **The TUI's approval as a case:** the session approves the TUI
  itself (3.3), so no case walks the approval's own frames; the TUI
  journey `sign-in-approved` keeps them recorded (`tui/journeys.md`, 9).
  Default: so, until a case asks for them.
- **A newer build's Forgejo:** a build that pins another Forgejo
  version than a copy's cache holds needs it fetched before an owned
  forge starts. Default: the place keeps its cache across copies and
  builds, and a session's start fails with status 4 when the box
  cannot start its forge, naming the version.
