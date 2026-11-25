# Driving the clients a line at a time

Provisional, 2026-11-12, revised 2026-11-21. QA's hands on ctl, the web
client and the TUI (`../draft/archive/landed-qa.md`, 7 and 10):
`temper-client-journeys drive`, a session that starts a fake box, or a
deployment's processes (`../testing/qa/deployment.md`), from a given
build in a given scratch directory, then takes lines on standard input,
each a step a person takes in ctl, the browser, the TUI or the forge, or
a look at what they are shown, and answers each. A ctl session starts
no browser (2.6). It runs no build and writes nothing outside its
scratch directory. It serves QA's recipes (`recipe.md`) and a journey's
walk on a box or deployment after landing (2.7), for what only a browser
or terminal shows. It writes no page of main's: those are printed in a
world (`journey-walks-world.md`, 1). `drive.md`, section 2 is the session, 3
its lines, 4 the lines it keeps, 5 its code, 6 its tests, 7 what is
open.

## 1. In one page

- **A session is one process and one box,** or one deployment's
  processes: started from a given build (`--bin`, `--bundle`), in a
  given scratch directory, the box's or the deployment's directories
  kept there across sessions, so that a place's restored copy starts as
  a fresh one does (2.1 and 2.2).
- **Signed in by the session:** ctl from a file holding the
  box's first link, the browser with the fresh link serve prints when
  the spent one is opened, and the TUI, when a line first needs it, by
  the browser's approval (2.3 and 3.3), except in a journey's session:
  there `tui approve` signs the TUI in, and no root load is implied
  (2.7). No link is logged, answered or kept.
- **A ctl session** (2.6), `--ctl`, takes ctl and forge lines alone and
  starts no browser and no TUI, on the box or at a deployment. A
  deployment's session (`--deployment`) starts the deployment's
  processes in the box's stead, and, but with `--ctl`, its browser and
  TUI reach the engine's origin as a box's reach the box's
  (`../testing/qa/deployment.md`, 5 and 6).
- **Lines are the walks' words:** `web` and a browser act,
  `tui` and a terminal act, `ctl` and a ctl act, with the box's values
  named rather than written, `forge` and a commit or a wait on the box's
  fake Forgejo as the person, and `look`, which answers the page's
  accessible text and a screenshot, or the TUI's rows (section 3).
  `web requests` answers the requests the page has sent since it last
  loaded, each with its wave, so that a page's request budget is
  checked on a landed build (3.2).
- **Each line answered,** `ok` or `failed: WHY`, what it shows prefixed
  `| `; a failed line never ends the session (3.7).
- **Kept lines:** every act but a look or mark, in order, in `session/lines`,
  so a case's steps become a recipe's and a finding's reproduction
  (section 4).
- **Its end stops the box cleanly,** so that its directories are a
  stopped box's, ready to copy; its exit status says whether the box
  started, was refused by its store, or failed (2.4 and 2.5).
- **Its logic has no io,** and its tests are
  focused: no browser, no box (sections 5 and 6).

## 2. The session

### 2.1 Its command

```text
temper-client-journeys drive --bin DIR [--bundle DIR] --scratch DIR [--scripts FILE]... [--strict] [--ports LIST] [--files DIR] [--ctl] [--deployment DIR] [--journey [--widths LIST]]
```

- **`--bin DIR`** holds `temper-box`, `fake-claude` and `temper`,
  each an executable regular file, and with `--deployment` also
  `temper-engine`, `temper-worker` and `temper-agent`. The client is
  the TUI's recording build (`tui/journeys.md`, 2), whose saves the
  TUI's lines read (3.3). The box finds `fake-claude` beside itself
  (`../protocol/setup/box-backends.md`, 1).
- **`--bundle DIR`** is the web bundle (`scripts/bundle.sh OUT`), given
  to the box as its `--bundle`, required on a box. At a deployment it is
  optional: the bundle the engine's document names as `{place.bundle}`
  (`../testing/qa/deployment.md`, 2), which a document naming it
  requires; one given is checked as a box's is.
- **`--ctl`:** a ctl session (2.6), on the box or at a deployment.
- **`--journey`:** a journey's session (2.7), refusing `--ctl`.
- **`--widths LIST`:** distinct screenshot widths from 320 to 3840,
  joined by commas, in the given order; requires `--journey`. With none,
  `web mark` answers text alone (3.6).
- **`--deployment DIR`:** a deployment's session, DIR a deployment's
  root recipe's `config/`, its processes started, its lines taken and its
  end as `../testing/qa/deployment.md` has them, in place of the box's
  (2.3 to 2.5): with `--ctl` a ctl session (2.6), without it a full
  one, its browser and TUI on the engine's origin. It needs `--ports`.
- **`--scratch DIR`** is an existing directory, the user's own and not
  open to others; every directory and file the session or what it
  starts writes is under it, apart from Chromium's temporary files
  and sockets (2.2).
- **`--scripts FILE`, `--strict`** are given to the box as they are
  (`../protocol/setup/box-backends.md`, 1), its fake model's scripts and strict
  switch; a recipe's walk gives its chain's scripts (`recipe.md`, 5).
- **`--ports LIST`,** nine ports, each nonzero and distinct, joined by
  commas: given to the box as `TEMPER_INTERNAL_FAKE_BOX_PORTS` in its
  environment (`../protocol/setup/box-backends.md`, 1), so a new sandbox binds
  them and a retained one keeps its own; a QA place's
  (`../testing/qa/places.md`, 2). A list that is not nine distinct
  nonzero ports refuses the session, naming it, before anything starts.
- **`--files DIR`,** an existing directory that `forge commit` and `forge push` lines
  read the files they commit from (3.5), and a `ctl settings` line its
  settings files (3.4), read only: a QA pass's
  (`../testing/qa/places.md`, 5). A recipe's walk gives each recipe's
  lines its own `files/` instead (`recipe.md`, 5.1).
- **Chromium** comes from `SKEIN_TEST_CHROMIUM`, as the browser tier
  takes it, for a full session on a box or at a deployment; a
  ctl session needs none and reads no such variable.
- **Each path** is taken relative to the working directory and made
  absolute before anything starts. A file of `--bin` missing, a bundle
  without its `index.html`, a scratch directory that is not a
  directory, or no Chromium refuses the session, naming it, before
  anything starts.
- **It runs no build:** the programs it starts are the box or the
  deployment's processes, the client and Chromium, and what they start; never `cargo`, `scripts/bundle.sh`
  or another program of the checkout.

### 2.2 Its scratch directory

| Path | What | Kept |
|---|---|---|
| `config/`, `data/`, `cache/` | the box's three directories (`../protocol/configuration/directories.md`), given as `--config`, `--data` and `--cache` | across sessions: absent, the box prepares a new sandbox; present, it starts the one they hold, as a place's restored copy (`../draft/archive/landed-qa.md`, 5) |
| `session/` | the session's own, removed and made again, 0o700, at its start | until the next session |
| `session/home/`, `session/state/` | `HOME` and `XDG_STATE_HOME` of every program the session starts; `XDG_CONFIG_HOME`, `XDG_CACHE_HOME` and `XDG_DATA_HOME` under `home/` | |
| `session/tmp/` | `TMPDIR` of every program the session starts but Chromium | |
| `session/runtime/`, `session/tui-runtime/` | `XDG_RUNTIME_DIR` of the box and of the TUI, apart, as `tui/journeys.md`, 8.2 keeps them | |
| `session/ctl/` | ctl's directory (`ctl/command-line.md`, 1), which its sign-in makes | |
| `session/browser/` | Chromium's profile and downloads | |
| `session/bin/` | `xdg-open` and `open`, links to `true`, first in the TUI's `PATH` (`tui/journeys.md`, 8.2) | |
| `session/box.log` | the box's stderr, each line as it comes, and never a link: its stdout, which carries its sign-in links alone (`../protocol/setup/box-supervision.md`, 1), is read for them and written nowhere (2.3) | |
| `session/looks/` | each look's screenshot, `NNNN-WIDTH.png`, NNNN the line's number from 1 | |
| `session/lines` | the kept lines (section 4) | |

The box's own environment is the session's: the `PATH` the session was
given, the directories above, and `TEMPER_INTERNAL_FAKE_BOX_PORTS` where
`--ports` gives it. The TUI's is `tui/journeys.md`, 8.2's,
in the session's directories.

Chromium takes the session's directories but for `TMPDIR`, left unset
as in the browser tier. It puts its singleton socket in the host's
temporary directory: under `session/tmp/` in a deep scratch directory,
the Unix socket's path can be too long. Its profile and downloads stay
in `session/browser/`.

### 2.3 Its start

In order, each within its bound; a failure is 2.5's.

1. **The box:** `BIN/temper-box --fake-backends --bundle BUNDLE --config
   SCRATCH/config --data SCRATCH/data --cache SCRATCH/cache`, then each
   `--scripts FILE` and `--strict` as given, its stdin closed, its
   stdout and stderr each read a line at a time: its stderr into
   `session/box.log`, its stdout for its links (below). Its first link
   is the first stdout line starting `http://`
   (`../protocol/setup/box-supervision.md`, 1), within 60 seconds.
2. **ctl, signed in from a file:** the link written to
   `SCRATCH/session/link`, 0o600, then `BIN/temper ctl
   SCRATCH/session/ctl sign-in SCRATCH/session/link </dev/null`
   (`ctl/command-line.md`, 1.1, and `ctl/README.md`, 8), exit 0 within 60 seconds, then
   `session/link` removed; the link is never a program's argument and
   never in an answer. It waits on serve's start as a link opened early
   waits (`../protocol/setup/box-supervision.md`, 1).
3. **The browser,** but in a ctl session (2.6): headless Chromium, as the browser tier drives it
   (`web/testing.md`; skein's `ui/web/testing.md`, 2), opens the first
   link, now spent, so that serve prints a fresh link on stdout
   (`../protocol/setup/box-supervision.md`, 1) within 30 seconds; it opens that one,
   waits until that page shows the client's main region so its sign-in
   is answered. Outside a journey's session, it then loads `/` on the
   box's origin and waits for its main region, all within 60 seconds:
   loading `/` sooner cancels the sign-in's exchange and leaves `/`
   signed out. In a journey's session (2.7), it does not load `/`: its
   first line meets the page the fresh link opens. Otherwise the first
   line meets the root page. The browser holds the session's
   sign-in; a copy holds none (`../draft/archive/landed-qa.md`, 5).
4. **The TUI** starts at the session's first `tui` line (3.3), not
   before.
5. **`ready`** on stdout, and the lines are read.

While any step waits, the box's exit, or a line on its stderr saying a
step is restarting or the box gives up (`../protocol/setup/box-supervision.md`, 1),
ends the wait as the box's failure.

**At a deployment** the start is `../testing/qa/deployment.md`, 5's:
the deployment's processes in place of step 1, ctl signed in from the
engine's first link as step 2 has it, then, but in a ctl session, step
3 on the engine's origin, the fresh link the engine's stdout, and the
TUI at its first line as step 4 has it. Its links and the page's
troubles are judged as below, a failure there the deployment's.

**Its links.** Every line of the box's stdout is a sign-in link, its
first or a fresh one serve prints (`../protocol/setup/box-supervision.md`, 1). The
session takes its first for ctl (step 2) and, but in a ctl session, the
fresh one the spent first brings for the browser (step 3); it drops
every other line of the box's stdout as it reads it, as a deployment's
session drops the engine's later links (`../testing/qa/deployment.md`,
5). No line of the box's stdout is written to `session/box.log`, and no
link is answered or kept (`drive.md`, section 4), so `session/box.log`, the tail a
`box failed:` answer prints from it (2.5) and a finding's evidence that
copies it (`../testing/qa/findings.md`, 2) hold no link. The box's
opening lines, which the box's values are read from (3.4), are on its
stderr and so in `session/box.log`.

**The page's trouble.** The browser's harness reports each trouble the
browser kit sees in the page as the kit gives it, its kind, its text
and its resource's URL, apart from its own (skein's `ui/web/testing.md`,
2), and the session judges it:

- **The spent link's refusal is expected.** Opening the spent first
  link, the shell `POST`s its secret to the link's route,
  `<prefix>/v1/sign-in/link` (jig's `protocol/client/default-http.md`,
  1), at the box's `/jig/v1/sign-in/link`, which refuses it, 401, as a
  link a sign-in spent (jig's `protocol/client/sign-in.md`, its printed
  link); that refusal is what makes serve print the fresh link. Chromium
  logs it as a `Log` trouble, `Failed to load resource: the server
  responded with a status of 401 (Unauthorized)`, its URL the route's on
  the box's origin, with no query. From the spent link's opening
  until `ready`, such a trouble is never counted: it may come in any of
  step 3's waits, before or after the fresh link's line, since the
  browser's log and the box's stdout are not ordered.
- **Any other trouble at the start** ends the wait as the box's failure,
  `box failed: WHY` (2.5), WHY the trouble as `KIND: TEXT (RESOURCE)`:
  its kind as the kit names it, its text, which for a refused resource
  holds its status, and the resource, its URL with the query and the
  fragment dropped, since a link's URL carries its secret
  (`/sign-in?secret=…`, or at the route as a build before printed it) and a link is never in an answer (step
  2). For example `Log: Failed to load resource: the server responded
  with a status of 404 (Not Found) (http://127.0.0.1:8367/app.js)`. A
  trouble with no URL is `KIND: TEXT`.
- **After `ready`,** a trouble fails the line being carried out (3.7)
  with the same words, and one that comes between lines is not
  counted.

### 2.4 Its end

At a line `end`, or at stdin's end: the TUI ended, its process group
killed; the browser closed; then the box stopped by
`SIGTERM` and waited on, within 60 seconds, and its tree killed past
them. After a clean stop the box's directories hold a stopped box,
which a place copies (`../draft/archive/landed-qa.md`, 5). The session's
directory stays until the next session. At a deployment the TUI and the
browser end the same way, first; then the deployment's processes stop
in their order (`../testing/qa/deployment.md`, 7).

### 2.5 Its exit

| Status | When | Its last lines on stdout |
|---|---|---|
| 0 | the session reached its end and the box stopped cleanly, whatever its lines answered | `ended` |
| 3 | the box's store was refused by name: serve's log, under the box's logs (`../protocol/setup/box-supervision.md`, 1), holds a line starting `temper serve: ` that names a directory under the scratch directory, as every refusal of a store does (`../protocol/store-restart.md`, 1; jig's `protocol/store/records.md`, 3); or a fake forge's log there holds a line starting `temper-box fake forge: state ` that names a file under the scratch directory, its kept state refused (`../testing/fakes.md`, 2) | `refused: ` and that line |
| 4 | the box did not start or stop otherwise: it exited, gave up or panicked, a link or the page did not come within its bound, the page showed a trouble at the start that 2.3 counts, ctl's sign-in failed, its stop took past its 60 seconds, or the browser ended or Chromium exited, at any line, WHY then the harness session's own trouble as it words it (skein's `ui/web/testing.md`, 2) | `box failed: WHY`, then the last 20 lines of `session/box.log` and of serve's log, each prefixed `\| `: neither holds a link (2.3; `../protocol/setup/box-supervision.md`, 1); where the browser ended or Chromium exited, then the last 20 lines of Chromium's stderr's tail, prefixed the same way |
| 1 | the session's own failure: a refused command line (2.1), a scratch directory it cannot make its own | `failed: WHY`, on stderr |

A place makes a copy refused by its store again from its recipe
(`../draft/archive/landed-qa.md`, 5): status 3 is that signal. Status 4 on a
fresh box, or on a copy past its closing check, is a finding against
temper (`../draft/archive/landed-qa.md`, 7).

### 2.6 A ctl session

A session of ctl and forge lines alone, which starts no browser and no
terminal and needs neither (`../draft/archive/landed-qa.md`, 4 and 7): `--ctl`,
on the box or at a deployment (`--ctl --deployment DIR`).

- **Its start** is 2.3's first two steps, the box started, or the
  deployment's processes, and ctl signed in from its file, then
  `ready`: no Chromium is started, sought or required, and no TUI.
- **Its lines:** ctl and forge lines as 3.4 and 3.5 have them, and at a
  deployment its own (3.8). A web or tui line fails, `failed: line N:
  this session takes ctl and forge lines alone`, at a deployment in its
  words (`../testing/qa/deployment.md`, 6.1), and is not kept: it was
  never carried out. A look of the browser or the terminal fails the
  same way.
- **Its end and exits** are 2.4's and 2.5's, with no browser and no TUI
  to close.

So a pass runs its ctl cases first in such a session
(`../testing/qa/passes.md`, 4), and a recipe whose chain has no web and
no tui line walks in one (`recipe.md`, 5.1).

### 2.7 A journey's session

`drive --journey` takes the journey's client lines with nothing implied:
no root load after the browser's sign-in (2.3), and no TUI approval at
its first line (3.3). `tui approve` signs it in explicitly; `web mark`
and `tui mark` answer looks and are not kept (3.6). This mode refuses
`--ctl`; outside it, those three lines fail without being kept or
starting a TUI.

```text
temper-client-journeys journey walk NAME --bin DIR [--bundle DIR] --scratch DIR [--strict] [--ports LIST] [--widths LIST] [--deployment DIR]
```

- **One journey session,** on a fake box or a deployment, from the given
  build: no build is run. Its options are 2.1's, with `--journey`
  selected by the command. It refuses `config/` or `data/` in scratch,
  and session writes that would overlap either repository journey root.
- **The chain read first** (`journey-walks.md`, 2.2 and 2.3): its lines
  and `scripts.json` files root first, skipping ancestors' marks, on
  one session. No reset, root load or `tui stop` is added between
  journeys. The file's `walk 2` selects these rules; `recipe 1` keeps
  the recipe's own sign-in and resets (`recipe.md`, 5.1).
- **Each answer carries `NAME:N`,** the journey and its file line, before
  each shown line and its `ok` or `failed`. A failed line stops the
  walk before the next; the session is ended before the walk exits.
- **Its exit:** 0, `walked NAME`; 5, `stopped at NAME:N: WHY` for a failed
  line. The session's 3, 4 and 1 (2.5) pass through instead when its own
  start, end or operation fails.
- **After landing:** this walk checks the rendered look or a real
  program's set-up, outside every plan (`web/look-requests.md`, 4).
  Its marks write no page of main's.

## 3. Lines

### 3.1 Their form

- **One line each,** a recipe's step (`recipe.md`, 3): its client first,
  `web`, `tui`, `ctl`, `forge` or `deployment`, then its record and its fields, as the
  journeys' file writes fields (`web/journeys-recordings.md`, 1.2): each after one
  space, a word bare, bytes in double quotes. Quoted bytes take printable
  ASCII alone; write other bytes as `\xHH`, with `\"` and `\\` for
  quote and backslash. Or a look (3.2 to 3.3), or
  `end`.
- **An empty line** is answered `ok` and not kept.
- **A line that does not parse** is answered `failed: line N: WHY`,
  naming what it expected, and is not kept: it was never carried out.
- **Numbered** from 1, every line read, empty ones among them.

### 3.2 The browser's lines

| Line | What the session does | Bound |
|---|---|---|
| `web press ROLE NAME [ORDER]`, `web submit ...`, `web toggle ...` | finds the control by role, exact accessible name and order from 0 (`web/journeys.md`, 8.2), and acts on it as a person | 60 s to find it |
| `web input ROLE NAME VALUE [ORDER]` | the field's value selected and VALUE typed | 60 s |
| `web see TEXT` | waits until the page shows TEXT | 60 s |
| `web reload PATH` | loads PATH, with its query, on the box's origin, and waits for the client's main region | 60 s |
| `web look` | answers the page's accessible text, the browser kit's snapshot, a line per node (skein's `testing/browser.md`, 6) | 10 s |
| `web look WIDTH` | first the path of a screenshot of the page at WIDTH CSS pixels wide, its height the page's viewport's, written to `session/looks/`, then the text as `web look`; the viewport restored after | 10 s |
| `web mark` | in a journey's session alone, answers a look (3.6), text and screenshots at `--widths` | 10 s per screenshot, then 10 s for text |
| `web requests` | answers the requests the page has sent since it last loaded, a line each with its wave, then `N requests in W waves`, once the page is quiet (below) | 30 s, quiet for 2 s |

A role is a word of skein's roles, lowercase, as `web/journeys.md`, 8.2
has it. In a journey's session, `textbox` finds a text field or text
area (`journey-walks-world.md`, 1.2). WIDTH is 320 to 3840. At a deployment the origin is the
engine's clients' listener, its first link's
(`../testing/qa/deployment.md`, 5 and 6.1).

**The page's requests,** what `web requests` answers, come from the
requests the browser kit reports of the session's page and their first
answers (skein's `testing/browser.md`, 5), which the harness passes on
as it passes the page's troubles (2.3; skein's `ui/web/testing.md`, 2).
The session takes them from its start, whatever line runs or none:

- **Since it last loaded:** the page's last load is the latest request
  the kit reports as a `document`, and the page's requests are those it
  reports with that load. So what the leaving page sends as a reload
  begins, its `POST /jig/v1/read` among them, carries the leaving page's
  load and is left out wherever it comes in the order. A move the client
  draws in the same document is no load, and the list goes on.
- **On its origin:** a request whose URL is not on the origin of the
  load's document, its scheme, host and port, is left out: another
  origin's, the browser's own, a `data:` or a `blob:` one.
- **After its document and its bundle's files:** the load's own document,
  and each `GET` under `/bundle/` or of a root file, `/robots.txt`,
  `/sw.js` or `/manifest.webmanifest` (`web/serving.md`, 1, steps 4 to
  6), or of `/favicon.ico`, the tab's icon, which the browser asks for by
  itself, are not listed and count in no wave. Each still counts toward
  the page's quiet (below).
- **A line each, in the order sent:** its method, its path with the
  query and the fragment dropped, since a sign-in link carries its secret
  in its query (2.3), and its wave: `GET /jig/v1/stream, wave 1`.
- **Answered** at its first answer as the kit reports it, the first of
  its answer's first body bytes, its end and its failure. So a stream,
  which never ends, is answered at its first event, the hello (jig's
  `protocol/client/default-http.md`, 2), and a batch at its first reply.
- **Its wave** is counted as `../draft/page-round-trips.md`, 2 counts
  waves, every exchange the client has sent that waits for an answer,
  for answers that come apart rather than all at once: a request's wave
  is one more than the greatest wave among the listed requests answered
  before it was sent, and 1 where none was. So requests sent together
  share a wave, and one sent once an answer has come is in the wave
  after that answer's. A request's wave is the length of the chain of
  round trips it may have waited on, and the last wave the page's count
  of round trips; an engine that answers each wave at once, as the
  budget's test has it, gives the same waves.
- **Then the last line,** `N requests in W waves`, N the requests listed
  and W their greatest wave, 0 with none, its words the same whatever
  the numbers: `0 requests in 0 waves`.
- **Once the page is quiet:** every request of the load on its origin
  answered, its document and its bundle's files among them, and none
  sent or answered for 2 seconds; at once where that already holds. The
  bound is 30 seconds from the line's start. Past it the line fails
  (3.6), showing the requests listed so far and the last line, its WHY
  naming each request still unanswered as its line shows it, or, with
  none, that the page kept sending.
- **Kept** as every line but a look is (section 4): it acts on nothing,
  so a replay reaches the same page, and a finding's reproduction ends
  at the line whose answer the finding compares.

The two cold opens of Work that a pass by hand counted from Chromium's
net log on 2026-11-18, on a project with a milestone of two goals and
three goals outside it, each sent the stream and, after its hello, one
batch, and nothing more. This line answers such an open, the
browser's `GET /favicon.ico` before the stream left out:

```text
| GET /jig/v1/stream, wave 1
| POST /jig/v1/batch, wave 2
| 2 requests in 2 waves
ok
```

A reload whose small store kept keyed requests lists them too, beside
the batch, though `../draft/page-round-trips.md`, 2 puts them outside
its budget.

### 3.3 The TUI's lines

The TUI starts at a `tui` line other than `tui stop` when none runs:
`BIN/temper http://HOST:PORT`, at the box's origin, or the engine's at a
deployment, in a pseudo-terminal of 80 columns by 24 rows, its
environment `tui/journeys.md`, 8.2's in the
session's directories, its output read and dropped. Outside a journey's
session, the session signs it in at once: a save holds a `browse`, whose
address the browser opens and whose "Approve" it presses, then the
browser returns to the address it was on; within 30 seconds, before the
line is carried out. A recipe keeps that sign-in. In a journey's session
(2.7), the TUI starts without approval, even after `tui stop`; its walk
says `tui approve` where it needs to sign in (`tui/journeys.md`, 8.1).

Outside that mode, `tui approve`, `tui mark` and `web mark` each answer
`failed: line N: tui approve is a journey's line`, with its own words,
are not kept and start no TUI.

| Line | What the session does | Bound |
|---|---|---|
| `tui key KEY` | KEY as the journeys' file writes a key (`tui/journeys.md`, 3.2), given through the terminal as a person types it | — |
| `tui see TEXT` | waits until the TUI's frame shows TEXT: a save, replayed natively to its end, gives the frame, as a walk's `see` does (`tui/journeys.md`, 8.1) | 30 s |
| `tui stop` | ends the TUI as the session's end does, `ok` when none runs; the next `tui` line starts it afresh, with approval only outside a journey's session | 10 s |
| `tui approve` | in a journey's session alone, opens the last `browse` address from a save, presses "Approve", then returns the browser to its prior address | 30 s |
| `tui mark` | in a journey's session alone, answers the frame's rows as `tui look` | 10 s |
| `tui look` | answers the frame's rows, from a save replayed, a row a line, trailing spaces cut | 10 s |

A save that writes no recording within 10 seconds fails its line,
saying that the session needs the client's recording build
(`tui/journeys.md`, 2).

### 3.4 ctl's lines

`ctl ARGS...` runs `BIN/temper ctl SCRATCH/session/ctl ARGS
</dev/null`, each field after `ctl` one argument: an act as the
signed-in person, the deployment's first owner (`ctl/README.md`, 7). It
answers ctl's stderr and then its stdout, each line prefixed `| `, then
`ok` on exit 0, else `failed: exit N`. An act and the wait on it are two
lines, as ctl waits (`ctl/tools-waiting.md`, 1):

```text
ctl task show 3 --until ended
```

A ctl line's bound is its own: a wait's timeout, and 10 seconds past
it, and a call with none 60 seconds.

**The box's values.** A field of a ctl line whose bytes are exactly one
of these names, braces and all, bare or quoted as the grammar allows,
is replaced by the box's value when the line runs, before ctl starts.
Each value is read from the newest of the box's opening lines that
names it (`../protocol/setup/box-backends.md`, 1), which the box writes to its
stderr and the session to `session/box.log` (2.2), so a restored copy's
box gives its own afresh. The kept line keeps the name (`drive.md`, section 4), so
a recipe names no place's path, port or credential and walks the same
at every place (`../draft/archive/landed-qa.md`, 5).

| Name | Its value | From the line |
|---|---|---|
| `{forgejo.api}` | `HOST:PORT` of the box's fake Forgejo elsewhere | `box: fake forgejo http://HOST:PORT` |
| `{forgejo.git}` | `http://HOST:PORT`, that forge's git base, which the box's fake git reaches | the same |
| `{forgejo.admin}` | the path of that forge's site admin's token file | `box: Forgejo elsewhere admin token file FILE` |
| `{git}` | the box's fake git program, `D'/demo/bin/git` | `box: effective data D'` |
| `{model.key}` | the path of the box's fake model's key file, which `ctl account add-key` takes | `box: model key FILE` |

A field in braces that names none of them does not parse (3.1); a name
whose line the box has not printed fails the line, naming the line it
lacks. At a deployment each name takes the deployment's value, from the
place and its rendered documents rather than a log
(`../testing/qa/deployment.md`, 6.1). So the person connects the box's fake Forgejo as its admin, with
its first repository, the box's fake git its git program, in one line,
and no line holds the token, only its file's name:

```text
ctl forge connect forgejo-admin {forgejo.admin} --first work/hello --project Hello --api {forgejo.api} --git {forgejo.git} --git-program {git} --key qa-hello
```

**A settings file.** In a `ctl settings apply` or `ctl settings propose`
line, each FILE field is a PATH, as in `forge commit` (3.5): every field
after `apply` or `propose` and before the first flag, except `-`. The PATH
is read from the recipe's `files/` or from `--files DIR`, and replaced by
the file's absolute path when the line runs, as a box value is. The kept
line keeps the PATH. A missing PATH fails the line, naming it, before ctl
starts. stdin stays closed.

### 3.5 The forge's lines

A person's own act on their forge that no client makes: a commit
through the forge's API, as `../../development/acceptance.md`, 7 commits
its gate, and the wait on that commit's checks. The forge is the box's
fake Forgejo elsewhere (`../protocol/setup/box-backends.md`, 1), the person's at
a QA place, keeping its state in the box's data, its CI decided by what
a head holds (`../draft/archive/landed-qa.md`, 6; `../testing/fakes.md`, 2).

| Line | What the session does | Bound |
|---|---|---|
| `forge commit forgejo REPOSITORY MESSAGE PATH [PATH]...` | one commit to REPOSITORY's default branch, MESSAGE its message, of each PATH's file of the files directory, at PATH: created where the branch lacks PATH and replaced where it holds it; answers the new commit's id | 60 s |
| `forge push forgejo REPOSITORY BRANCH MESSAGE PATH [PATH]...` | one commit as the person to BRANCH of REPOSITORY, of each PATH's file as for commit; makes an absent BRANCH at the default branch's head; answers the new commit's id | 60 s |
| `forge checks forgejo REPOSITORY` | waits until the combined status at the default branch's head is `success`; answers the head and each status's context and state | 300 s |

- **`forgejo`,** a word, names the box's fake Forgejo elsewhere; another
  word does not parse. **BRANCH** is a word. **REPOSITORY** is `OWNER/NAME`, a word; MESSAGE
  and each PATH are bytes. One to 16 PATHs, each relative, 1 to 256
  bytes, with no empty, `.` or `..` segment and no two alike. A line out
  of these bounds does not parse (3.1).
- **Its files** are read when the line runs, each PATH a regular file
  of the files directory at most 64 KiB: the session's `--files DIR`
  (2.1), or, on a recipe's walk, the recipe's own `files/` (`recipe.md`,
  2 and 5.1). No files directory, or a PATH missing there or past the
  bound, fails the line, naming it. So the line names what it commits,
  and the recipe holds it (`../draft/archive/landed-qa.md`, 5).
- **The forge's hands** are the box's, read at each line from the
  newest of its opening lines in `session/box.log`
  (`../protocol/setup/box-backends.md`, 1): the API, `http://HOST:PORT/api/v1`,
  from `box: fake forgejo http://HOST:PORT`, and the person's token from
  the file `box: Forgejo elsewhere admin token file FILE` names: the
  forge's site admin's, the token the person connected temper with
  (3.4). A box log with no such line, or no such file, fails the line,
  naming what it lacks.
- **As the person:** every request carries `Authorization: token TOKEN`,
  TOKEN the file's content with one final newline trimmed, read from the
  box's file at each request and never printed, kept, logged, written
  elsewhere or given as an argument; only the file's path is named. So
  no line, no recipe and no answer holds a credential
  (`../draft/archive/landed-qa.md`, 11).
- **The commit:** the repository read (`GET /repos/OWNER/NAME`) for its
  default branch; each PATH read at that branch (`GET
  /repos/OWNER/NAME/contents/PATH?ref=BRANCH`), absent (404) making it a
  `create` and present an `update` with its blob's `sha`; then one
  request (`POST /repos/OWNER/NAME/contents`) with the branch, MESSAGE
  and every file, its content in base64, so that the branch gains one
  commit, the person its author. It answers `| ` and the commit's id
  from the response.
- **The push:** the repository read gives its default branch, then
  `GET /repos/OWNER/NAME/branches/BRANCH` tests the target branch. Present,
  each PATH is read there and the contents batch names `branch` BRANCH.
  Absent (404), each PATH is read at the default branch and the batch
  names `branch` the default and `new_branch` BRANCH, making the branch
  with the commit in that one request. Other refusals fail the line.
  Its files, bounds, credentials, answer and keeping are the commit's.
- **The checks:** the combined status (`GET
  /repos/OWNER/NAME/commits/BRANCH/status`) read every 2 seconds until
  its state is `success`, answering `| ` the head, then a line per
  status, `CONTEXT STATE`; `failure` or `error` fails the line at once,
  naming each status; still pending, statuses or none, at the bound,
  fails it, naming what the last read held.
- **A request refused** fails the line, naming the method, the path,
  the HTTP status and the response's body, cut at 4096 bytes; a forge
  that does not answer within the line's bound fails it as the bound.
- **Its HTTP** is skein's: `skein-http`'s client in the session's io
  loop, with `skein-forge`'s Forgejo requests where they serve, never a
  new dependency and never another program.

### 3.6 What a line answers

For each line, in order, on stdout: what it shows, each line prefixed
`| `, then one line, `ok` or `failed: WHY`. A caller writes a line and
reads to that last one. WHY names the line's number, what it sought and
what the page or the frame showed instead, where it can: for a control
not found, the controls of that role on the page with their names; for
a `see`, the page's text or the frame's rows, cut at 4096 bytes.

A journey's mark is a look, not a kept line: `web mark` answers each
screenshot's path, in `--widths` order, then the page's accessible text;
with no widths, text alone. Each screenshot uses the look's path and
restores the viewport (3.2). `tui mark` answers the frame's rows as
`tui look`, not a browser screenshot. `tui approve` is kept as an act.

### 3.7 A failed line

A failed line never ends the session: the next line is read
(`../draft/archive/landed-qa.md`, 7, a finding never stops a pass). A session
whose box has exited answers every later line `failed: the box
exited`, and ends with status 4 (2.5).

### 3.8 A deployment's lines

| Line | What the session does | Bound |
|---|---|---|
| `deployment restart` | the deployment started again with its documents unchanged, as `deployment patch` does: the TUI ended first and the page loaded again after, except in a ctl session; a setting taken at the next start (`../domain/landing-settings.md`, 1.1) then holds | 300 s |
| `deployment patch DOCUMENT PATH` | DOCUMENT `engine` or `worker`, a word (temper writes the agent's configuration, `../protocol/configuration/figures.md`, 6); PATH bytes, as a `forge commit` line's (3.5): the deployment's document patched by the file and the deployment started again, the TUI ended first and the page loaded again after, but in a ctl session (`../testing/qa/deployment.md`, 6.2) | 300 s |

On a box, a `deployment` line fails, `failed: line N: a box takes no
deployment line`, and is not kept.

## 4. The kept lines

- **Every line the session parsed,** but a look, a mark, `end` and an empty
  line, in order, as it was given, appended to `session/lines` once
  answered, so that a session ended by its bound or a crash keeps what
  it reached.
- **A failed line is kept,** since a reproduction ends at the line that
  failed; the answers say which failed.
- **Replayed,** they reach the same page: given to a new session on the
  same start, the same copy or a fresh box, they take the same steps.
  So a case's steps become a recipe's (`recipe.md`, 3) and a finding's
  reproduction (`../draft/archive/landed-qa.md`, 8).

## 5. Its code

- **In `temper-client-journeys-tool`,** `drive` is the session,
  beside `journey_walk.rs`, which feeds a journey's chain to it,
  `recipe` for QA's recipes, and the recorded file's print. The lines'
  grammar, `Line`'s parse and print, is `temper-client-walk-lines`' `line` module
  (`journey-walks.md`, 3.1), which `recipe.md`'s reader and a journey's
  walk share. A line prints as it parses, field for field, so a kept
  line is the line given.
- **One place, of either kind,** behind one interface
  (`../testing/qa/kinds.md`, 5): the box or the deployment's processes
  started, given their lines, asked for their values and stopped, and
  their logs classified, by the place's kind; the session's own steps,
  ctl's sign-in, the browser's start, the TUI, the lines and the
  clients' end, written once for both.
- **Its logic without io:** a state machine that takes each event, a
  line read, a face's answer, a save's bytes, a process's line or exit,
  a deadline, a forge's response, and answers what to do: start this
  program with these arguments and this environment, find this control
  and act, take a snapshot, a screenshot or a save, read this file, send
  this request to the forge, write this answer, stop. The io is
  a thin loop around it over skein's harness, the fake person's browser
  face, the pseudo-terminal and the processes.
- **The browser kit's snapshot, viewport and screenshot** (skein's
  `testing/browser.md`, 5 and 6) are asked through the fake person's
  harness, which gains a read for each where it has none (skein's
  `ui/web/testing.md`, 2).
- **The page's requests** are the harness's reports (skein's
  `ui/web/testing.md`, 2), each an event to the logic as a trouble is: a
  request sent, with its id, its load, whether it is a document, its
  method and its URL, and a first answer, by its id. The logic keeps the
  last load's from the session's start, and counts their waves, and the
  quiet spell by its events' times, with no io (3.2).
- **The TUI's saves and native replay** (`tui/journeys.md`, 2 and 4)
  give its frames and browse address; the terminal and its environment
  are `tui/journeys.md`, 8.2's.

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
- **The forge's lines:** given a box log with the fake Forgejo's lines,
  a token file and a files directory, `forge commit forgejo` reads its
  files from the directory, asks for the repository, each path, then
  one change of files holding `create` for a path the branch lacks and
  `update` with its `sha` for one it holds, its contents in base64, and
  answers the commit's id; `forge checks forgejo` waits through pending
  to `success`, fails on `failure`, and fails at its bound; a box log
  with no `box: fake forgejo` line, a missing token file, no files
  directory or a missing file fails the line, naming it; the token is in
  no answer, no kept line and no program's argument.
- **The box's values:** a ctl line naming `{forgejo.admin}`,
  `{forgejo.api}`, `{forgejo.git}` and `{git}` runs ctl with the box's
  values from the newest lines in their place, and its kept line holds
  the names; an unknown name in braces does not parse; a name whose
  line the box has not printed fails the line.
- **Its exits:** a serve log holding a store's refusal ends with status
  3 and its line; a box that exits before its link, with 4.
- **The browser's start and the page's trouble** (2.3): the logic given
  the box's links and the faces' answers in order; the 401 `Log`
  trouble at the link's route, given in any of step 3's waits, before
  or after the fresh link's line, is not counted and the start reaches
  `ready`; a 401 at another resource, a 404, or an exception fails the
  box, status 4, its `box failed: ` line naming the trouble's kind, its
  text and its URL with the query dropped; a trouble at the link's own
  URL names no secret; a trouble during a web line fails that line with
  the same words.
- **ctl from a file:** the box's first link written to `session/link`,
  0o600, ctl given its path and no stdin, the file gone once ctl has
  answered; the link in no answer and no program's argument.
- **No link logged:** given box output with links on its stdout, the
  first, the fresh one and a later one, among lines on its stderr, the
  session's `session/box.log` holds each stderr line, in order, and no
  link; a box that then gives up ends with status 4, its answer showing
  those stderr lines in its tail and no link.
- **A ctl session:** its start asks for no browser and needs no
  `SKEIN_TEST_CHROMIUM`; it is `ready` once ctl has signed in; a web or
  tui line, or a look of either, fails and is not kept; ctl and forge
  lines answer as in any session; `{model.key}` takes the box's `box:
  model key` line.
- **The page's requests** (3.2): the logic given a recorded sequence of
  requests and answers, with their times. A reload's document, its
  bundle's files and `/sw.js`, the leaving page's `POST /jig/v1/read`
  with the old load after the new document, `GET
  /jig/v1/stream?version=…` answered at its first bytes, then `POST
  /jig/v1/batch` and a request to another origin: `web requests`
  answers `GET /jig/v1/stream, wave 1`, `POST /jig/v1/batch, wave 2`
  and `2 requests in 2 waves`, no line holds a query, and the other
  origin's request and the leaving page's are left out. Requests sent
  apart (two together, the first answered, a third sent, the second
  answered, a fourth sent) take waves 1, 1, 2 and 2. The answer waits
  until 2 seconds pass with every request answered and none sent, and a
  request sent inside the spell starts it again. A request unanswered at
  30 seconds fails the line, naming it, and a page that sends every
  second fails it, saying so. `web requests` parses and prints back to
  its bytes, `web requests 1` is refused, and the line is kept.
- **A journey's session:** `drive/tests.rs` holds the fresh link's page
  without a root load, option conflicts and bad widths, ordered mark
  screenshots then text with no kept mark, and the journey's `textbox`
  matching a text area, including on retry. `drive/tests/tui.rs` holds
  no implied approval before or after a stop, explicit approval kept,
  its single 30-second bound and return on failure, and refusal of the
  journey-only lines outside that mode.
- **A journey's chain:** `journey_walk/tests.rs` holds one session,
  scripts and lines root first, only the named journey's marks, no
  added resets, answers carrying file lines, stop at the first failure,
  session exits passed through, and refusal of a missing journey or
  held scratch before a session starts. Its page is one an ancestor's
  press changes: the child's `see` and mark read the page that press
  left, and the walk takes one screenshot, at the child's mark, at its
  one width. A TUI journey's walk answers each line under its file
  line, the launchpad's rows after an explicit approval, and keeps
  `tui approve` but not `tui mark`, with no screenshot.
- **Above them:** the browser tier's journey test
  (story 17 of `web/testing.md`, 6) walks a scratch journey in Chromium on the
  shipped bundle, its mark's screenshot a PNG on disk and its kept
  lines the file's; in the same test, story 18 walks a scratch TUI
  journey on the native TUI, its `tui approve` pressing Approve in the
  session's Chromium, the one witness of that sign-in.
- **A deployment's session:** `../testing/qa/deployment.md`, 9's;
  `--ctl --deployment` a ctl session, `--deployment` alone a full one
  needing Chromium, and `--bundle` optional there.

## 7. Open questions

- **The TUI's size:** 80 columns by 24 rows, as the journeys'. Default:
  fixed until a case needs another.
- **The box with the TUI in front** (`tui/journeys.md`, 8.3): not
  driven. Default: none until a case needs it.
- **The quiet spell and its bound** (3.2): 2 and 30 seconds, for a box
  or a deployment on the session's own machine. Default: fixed until a
  case needs another.
- **A load that leaves the page where it was:** a navigation the
  browser turns into a download commits no document, but its request is
  reported as one, and the list starts again from it. Default: so; no
  case's page navigates to a download before `web requests`.
- **The fake GitHub's commit:** a `forge` line on the box's fake GitHub
  would commit through its git data API, as
  `../../development/acceptance.md`, 3 makes GitHub's gate one commit,
  as an App that the person installed. Default: none until a setup
  needs GitHub; the fake Forgejo serves every setup
  (`../testing/qa/setups.md`).
