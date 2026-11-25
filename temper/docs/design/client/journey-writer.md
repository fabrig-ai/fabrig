# The journeys' writer

Provisional, 2026-11-21. Who writes main's journey files, and when.
One writer, run after each landing, prints every journey in a world
with `temper-client-walk pages` and offers what differs as a change
of its own; nothing else writes those files. The journeys are the
browser's (`web/journeys.md`) and the TUI's (`tui/journeys.md`); the
look requests drawn on their pages are `web/look-requests.md`'s.

## 1. In one page

- **Main's pages have one writer** (section 2): each page and the files
  printed beside them. A journey's `walk`, `README.md` and
  `scripts.json` stay sources, which any change edits.
- **One engine prints main's tip** (section 3): `temper-client-walk
  pages` walks every journey in a world, in one process, with no box,
  browser, terminal, recording or network (`journey-walks.md`, 5.2).
- **A script runs it one at a time** (section 4): at each landing of
  main, `scripts/journey-pages.sh`, through the journeys tool's
  `writer` command, prints the tip and offers what
  differs as one change, opened from a pushed branch, its message ending
  `Printed: COMMIT`, or, given no project, as a commit on a branch of
  its own for its runner to land. While an offer of its own is open it
  starts no other, and one run at a time holds the checkout. Once it
  has run on a tip and found nothing to write, main's pages are that
  tip's print.
- **Goals are kept off** (section 5): the delivery check refuses a
  delivery that changes a written file beside any other file, and a
  nonempty overlay; the merge job settles a written file's conflict with
  the landing branch's copy; no test at the gate holds a journey to a
  committed recording or page.
- **A request is checked against the print** (section 6):
  `look-refresh.sh` drops a request once the last `Printed:` covers its
  landing, and whoever lands one prints into scratch in its own tree
  and compares, committing no written file.
- **After a landing, never inside one:** a walk that stops holds no
  landing. It fails the writer's run, an after-landing red of its
  runner's (the user, 2026-11-18).

## 2. The written files

Under `web/journeys/` and `tui/journeys/`:

| Written by the writer alone | Sources, any change's |
|---|---|
| a journey's pages: `NN.html`, and the TUI's `NN.txt`, NN two digits or more | a journey's `walk`, `README.md` and, where it has one, `scripts.json` |
| beside the browser's journeys, `skein-ui.css` and `journey.js`; beside the TUI's, `theme.css` and `tui-journey.js` | each root's `overlay.css`, empty on main; the `README.md` beside each root |
| a leftover `recording`, removed once its journey prints | |

- **A source changes with what a person does:** a change that renames a
  control a `walk` names edits that line in the same change, as it edits
  a test (`journey-walks.md`, 2.2). The writer's next run walks it.
- **A new journey** lands as its sources. Its pages follow from the
  writer, printed in the world (`journey-walks.md`, 6).
- **Pages are replaced whole:** a successful print removes surplus
  numbered pages and its journey's leftover `recording`. A directory
  with no `walk` loses its pages (`journey-walks.md`, 5.2).

## 3. The command

`cargo run -p temper-client-walk -- pages`, the writer's command of
`crates/temper-client-walk-tool`, on the checkout it is built from
(`journey-walks.md`, 5.2 and 5.3):

1. **Which:** every journey of both clients, the browser's then the
   TUI's, each in name order. A journey is a directory whose `walk`
   starts with exactly `walk 2` (`journey-walks.md`, 2.1 and 3.1).
2. **Walked in a world:** each chain's lines, with its joined scripts,
   and its own marks printed as pages. No box is walked and no
   recording is replayed.
3. **Printed in place:** a completed walk writes section 2's files only
   where their bytes differ, removing surplus pages and the journey's
   leftover `recording`. A walk that stops leaves every file of its
   journey as it was, and the others still print.
4. **Its lines,** on stdout, one a journey in that order: `printed
   NAME`, files changed; `fresh NAME`, no byte differed; `stopped NAME:
   WHY`, the journey and line that stopped it (`journey-walks-world.md`, 1.3).
   Every other word goes to stderr.
5. **Its exit:** 0, every walk printed; 1, a walk stopped, once the
   others are printed; 2, usage; 3, the print could not finish, a root
   unreadable or unwritable, the pages as it left them.

On a current tree it starts no child and changes no byte. A plan uses
`cargo run -p temper-client-walk -- print --out DIR` instead, which
prints into scratch and writes no journey file in the checkout
(`journey-walks.md`, 5.1).

## 4. The run

`scripts/journey-pages.sh [--project PROJECT (--budget AMOUNT |
--as-allowed)] [--branch NAME] [--push REMOTE] [--prepared DIR]
[--within DURATION]`, under `temper/scripts/`, run by the
after-landing check's owner at each landing of main, in a checkout of
its own whose `origin` is the repository. With `--project` it opens a
change, and needs `temper` on its `PATH` signed in; without it, it
opens nothing, and needs no ctl directory and no way to push.

- **The script runs the command:** `scripts/journey-pages.sh` runs
  `cargo run --locked --quiet --manifest-path ROOT/Cargo.toml -p
  temper-client-journeys -- writer` with its arguments, ROOT the
  repository holding the script, in the directory it was run in; its
  exit is the command's. The command, `temper-client-journeys writer`
  (`../testing/tools.md`, 2), is the run: this section's words, lines
  and exits are its own, under the script's name (`journey-pages:
  WHY`). It asks for each effect as a value, a child (`git`, `cargo`,
  `temper`) with its arguments, a file or a directory, or whether a
  process is live, and decides on the answer, so that its decisions are
  tested on canned answers (section 7). Every child runs with
  `LC_ALL=C` and its stdin closed. Building the command reads no
  `TEMPER_INTERNAL_BUNDLE`: the journeys tool builds no bundle.
- **Its arguments:** a value option needs a nonempty value, not another
  option; repeated options and unknown arguments are refused. With a
  project, exactly one of `--budget` and `--as-allowed` is needed;
  neither is taken without a project. `--branch` must be a git branch
  name and an explicit `--push` must name a configured remote.
- **`--prepared DIR` and `--within DURATION` have no effect:** each
  given option warns on stderr, `journey-pages.sh: --OPTION has no
  effect: temper-client-walk pages walks no box`. `--within` still
  requires digits then `s`, `m` or `h` (`90m`); neither option bounds or
  changes the print. `--help` prints the command's page
  (`../testing/tools.md`, 4.2), which says this, and exits 0.
- **Its branch:** NAME, `journey-pages` where none is given, the local
  branch its commit is made on.
- **Its push:** with `--project`, to REMOTE, `origin` where none is
  given; without it, only with `--push REMOTE`. With `--project` and no
  `--branch` the push is forced: `journey-pages` is the script's alone,
  and no change of its own still reads it, a change reading its branch
  once (`../domain/forge/change/opened.md`). Otherwise it is never
  forced, and a branch REMOTE holds at another commit is refused,
  naming the branch and that commit.
- **Its stdout,** which a script reads: the print's lines, one a
  journey (section 3, item 4), then one last line: `main's pages are
  TIP's print`, `wrote NAME COMMIT`, or a waiting line, `waiting on
  change N` or `waiting on COMMIT`. An early refusal prints nothing
  there; every other word goes to stderr.

1. **Alone:** a lock, the directory `journey-pages.lock` in the
   checkout's git directory holding the run's process number, made
   with `mkdir`. A second run while a live process holds it is refused,
   naming the lock and the process, exit 2; a lock whose process has
   gone is taken over. A missing or invalid process number is refused.
   The run removes its lock as it exits; a run a signal ends leaves it,
   and the next run takes it over, its process gone.
2. **One at a time:** the git directory keeps `journey-pages`, the last
   offer: `task N`, the change the script opened, or `commit COMMIT`,
   the commit it wrote where it opened none. While `temper ctl task
   show N --json` says the task has not ended, the script prints
   `waiting on change N`, exit 0, having fetched and started nothing.
   For a commit, it fetches `main` and waits, `waiting on COMMIT`,
   exit 0, until main holds that print: COMMIT reachable from
   `origin/main`, or any commit reachable from it whose message carries
   `Printed:` with COMMIT's TIP, a print merged by hand or squashed.
   Once the offer has ended or is held, the file goes and the run goes
   on. Whoever drops an offer by hand removes the file. An unreadable
   offer or task answer fails the run.
3. **On main's tip:** a checkout with tracked changes, or a rebase, a
   merge, a cherry-pick or sequencer in progress, is refused. The script
   fetches `main` from `origin` and checks out NAME at `origin/main`, TIP.
4. **Prints:** `TEMPER_INTERNAL_BUNDLE=none cargo run --locked -p
   temper-client-walk -- pages` (`journey-walks.md`, 5.2), once,
   regardless of which directories the roots hold. It passes stdout on
   as it comes. Exit 2 or 3 ends the run with nothing committed or
   offered; exit 1 still offers the other journeys' changes.
5. **Nothing differs:** its last line is `main's pages are TIP's
   print`, its exit the print's, including 1 where a walk stopped.
6. **Written files alone:** a changed path that is not section 2's
   written file ends the run, naming it, nothing committed.
7. **Its commit,** on TIP, of the written files: the subject
   `Journey pages printed at TIP12`, TIP's first twelve hexadecimal
   digits; a paragraph of the `stopped` lines where there are any; and
   a last paragraph of one line, `Printed: TIP`, TIP's whole hash.
8. **Pushed** as above, where it pushes.
9. **Opened,** with `--project`: `temper ctl change open --project
   PROJECT --branch NAME --at COMMIT --title SUBJECT --body BODY` with
   the budget given and `--key journey-pages-TIP`, so that retrying on
   the same tip after a lost answer opens nothing twice.
10. **Kept:** `task N` from the answer, or with no `--project`
    `commit COMMIT`, goes to step 2's file, and the last line is `wrote
    NAME COMMIT`.
11. **Its exit:** 0, nothing to write, a commit written, or waiting; 1,
    a journey left unprinted (the others written and offered), git,
    cargo or `temper ctl` failed, a push refused, or a path outside the
    written files changed; 2, its arguments, its checkout or its lock
    refused.

- **It settles:** its change lands as any opened change does, and that
  landing, as any other, runs the script on the new tip, which finds
  nothing to write. Without `--project`, whoever runs it lands the
  commit, as it is, by a merge or squashed, and step 2 waits for that.
- **How far behind:** between a landing and the writer's change, main's
  pages are an earlier tip's, and the last `Printed:` on main says
  which. Nothing reads them in between but a person.
- **A walk that stops** stops on every later tip, until a change repairs
  the walk or the client: one after-landing red, its runner's. The other
  journeys are printed and offered all the same. A new journey whose
  walk stops has no page until then.
- **Its change holds written files alone,** so it meets no conflict a
  goal made; the rules covering main ask of it what they ask of any
  change.

## 5. Goals kept off

- **The delivery check** (`tools/pre-pr/design.md`, 4, check 8): a
  delivery whose own changes hold a written file and any other path is
  refused, naming `cargo run -p temper-client-walk -- print --out DIR`
  for every journey, and how to restore the files; so is a tree whose
  `overlay.css`, under either root, holds a byte. A delivery of written
  files alone passes it. The written set includes leftover recordings.
- **The merge job** (`../protocol/tooling.md`, 4.1;
  `tools/merge/README.md`): a conflict at a written file's path is
  settled with the landing branch's copy, the commit merged in, with no
  model: its file where that commit holds the path, none where it does
  not. A deletion or a rename there asks no judgement, since only the
  writer writes the path, so it is settled too, unlike any other file's
  (`../domain/forge/change/repairs.md`). The writer's next run replaces
  it with the tip's print.
- **No test** replays or compares a recording or a page of main's, or
  walks main's journeys to their end (section 7).

## 6. A request against the print

- **Drawn on main's pages as they stand** (`web/look-requests.md`, 2):
  the writer's last print, where a journey shows the state. Where none
  does, the request adds a walk, which the person prints with
  `temper-client-walk print`; it lands as its sources, and the writer
  prints its pages after the landing (section 2).
- **The refresh goes by the print** (`web/journeys.md`, 7):
  `look-refresh.sh`, which runs the journeys tool's `look-refresh`
  command as the writer's script runs `writer` (section 4), reads
  PRINTED, the value of the newest valid
  `Printed:` trailer in topological order among every commit reachable
  from `origin/main`, not its first-parent line alone: a print reaches
  main as its own commit, as a merge's second parent made by hand, or
  squashed into another commit. It drops a request once the commit
  carrying its `Look-request` trailer is PRINTED or an ancestor of it.
  A request whose trailer is on main and not yet printed stays, on the
  pages main still holds. Its report opens with `pages printed at
  PRINTED12, N landings since`, N main's first-parent landings after the
  print's carrier is included, or `pages not printed yet` where main
  holds none; then, for each request, `kept NAME`, `kept NAME, done on
  main by C12, not yet printed`, or `dropped NAME, done on main by C12,
  printed at PRINTED12`.
- **Whoever lands a request** prints every journey into scratch in its
  own tree, `cargo run -p temper-client-walk -- print --out DIR`, and
  compares each page the request changes with the request's commit's,
  read from `docs/look/NAME`, equal where their facts agree
  (`journey-walks.md`, 5.1; `web/look-requests.md`, 3). It commits
  no written file, and its last commit carries `Look-landed: NAME`.
  The print needs no box and reads no committed recording or page.
- **The render check follows the writer's change** that prints the
  landing (`web/look-requests.md`, 4).

## 7. Its tests

Focused and native: no box, browser or terminal starts.

- **The sources and recorded-file tools:** each journey's README and
  each overlay's emptiness stay held as sources. Redaction, the last
  scan and the proof keep their cases on recordings the tests make in
  the host's world (`web/journeys.md`, 6; `tui/journeys.md`, 7), not on
  main's files.
- **The command,** in `crates/temper-client-walk-tool`'s suite, on
  journeys of the tests' own in scratch roots: current pages change no
  byte; one page changed by hand is the only page written; a completed
  print removes a leftover `recording` and surplus numbered pages,
  including the TUI's `NN.txt`; a stopped walk keeps all its journey's
  files and lets the others print. Its lines and exits are held there
  (`journey-walks.md`, 7).
- **The run's decisions,** step tests in
  `crates/temper-client-journeys-tool`'s library: the `writer` command
  on canned answers, each request it makes, in order, with its answer
  (git's output and exit, the print's lines and exit, `temper ctl`'s
  answers, the lock's and the offer's files, a process's liveness),
  then its stdout, stderr and exit. Nothing to write, nothing pushed or
  opened; pages changed, one commit of them alone whose last paragraph
  is `Printed: TIP`, pushed, and one `change open` asked with its
  branch, commit and key, for each budget form and remote; a second run
  while that task is live starts no print, push or open; once it has
  ended, the next run prints; with no project, a commit written on its
  branch, pushed only where asked and never forced, a branch held
  elsewhere refused, and the next run waiting until main holds the
  print, merged or squashed; its last line in each case; a held lock
  refused, exit 2, and one whose process has gone taken over; a path
  outside the written files changed, nothing pushed or opened, exit 1;
  a checkout with tracked changes, or a rebase, merge, cherry-pick or
  sequencer in progress, refused before fetching, exit 2; each refused
  argument, exit 2; an unreadable offer or task answer kept, exit 1.
  One `temper-client-walk pages` call, with or without a walk in the
  roots; its removed `recording` committed and its `stopped` lines in
  the paragraph; exit 2 or 3 offering nothing. `--prepared` and
  `--within` are validated and warn, but affect neither the print nor
  its commit.
- **The refresh's decisions,** step tests in the same library, the
  `look-refresh` command on canned answers: a request whose trailer is
  on main and not yet printed kept; dropped once a `Printed:` names a
  commit that contains it, the print reaching main as its own commit, a
  merge's second parent or squashed; the newest valid `Printed:`
  deciding, each invalid one warned of and passed over; nothing dropped
  while main holds no `Printed:`; the report's lines; a first push
  making the branch; a branch moved elsewhere left as it is, exit 3; a
  request without its trailer, with a malformed or repeated one, or a
  merge commit refused before rebasing, exit 2; an unsuitable checkout
  refused before fetching, exit 2.
- **Their witnesses,** in `temper-client-journeys`'s suite, the
  shipped scripts in scratch repositories with real git, a fake `cargo`
  first on `PATH` that runs the built tool for `-p
  temper-client-journeys` and answers the print for `-p
  temper-client-walk`, and a fake `temper`: changed pages offered on the
  fetched tip, with each budget form; a lock whose process is live
  refused, exit 2, then one whose process has gone taken over; the
  requests following main, rebased and pushed with the lease; and a
  conflict stopping for the person, exit 1, with the rebase in
  progress, the next refresh pushing once it is settled.
- **The delivery check** (`tools/pre-pr/design.md`, 9.2) and **the
  merge job** (`tools/merge/README.md`), in their own tests.

## 8. Later

- **temper runs the writer:** a command the project names, run on a
  landing branch's new tip as `close` is run on a goal's
  (`../protocol/tooling.md`, 4.2), its staged files landing as a change
  of their own, one at a time. The script then goes.
- **The playground's `story-a.log`**
  (`temper/tests/playground/tests/fixtures/`), another recording on main
  that the merge job regenerates (`tools/merge/README.md`): the same
  rule would have a test make it and no file keep it.
