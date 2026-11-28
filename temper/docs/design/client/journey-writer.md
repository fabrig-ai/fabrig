# The journeys' writer

Provisional, 2026-11-21, revised 2026-11-25. Who writes main's journey
files, and when. One writer, run by a timer after landings, prints
every journey in a world with `temper-client-walk pages`, renders each
newly landed look request beside the print, lists the journeys requests
left that may be retired, and offers what differs, pages and verdicts,
as a change of its own; nothing else writes those files. The journeys
are the browser's (`web/journeys.md`) and the
TUI's (`tui/journeys.md`); the look requests drawn on their pages are
`web/look-requests.md`'s.

## 1. In one page

- **Main's pages have one writer** (section 2): each page and the files
  printed beside them. A journey's `walk`, `README.md` and
  `scripts.json` stay sources, which any change edits.
- **One engine prints main's tip** (section 3): `temper-client-walk
  pages` walks every journey in a world, in one process, with no box,
  browser, terminal, recording or network (`journey-walks.md`, 5.2).
- **A script runs it one at a time** (section 4): every ten minutes, by
  a timer on the after-landing worker (4.1), `scripts/journey-pages.sh`,
  through the journeys tool's `writer` command, decides whether to run:
  within ten minutes of a commit carrying `Look-landed` reaching main,
  otherwise hourly. It prints the tip, renders the requests landed
  since (`journey-writer-requests.md`, 1.2), and offers what differs as one change, opened from a
  pushed branch, its message ending `Printed: COMMIT`, or, given no
  project, as a commit on a branch of its own for its runner to land.
  While an offer of its own is open it starts no other, and one run at
  a time holds the checkout. Once it has run on a tip and found nothing
  to write, main's pages are that tip's print.
- **Goals are kept off** (section 5): the delivery check refuses a
  delivery that changes a written file beside any other file, and a
  nonempty overlay, and takes a journey's directory deleted whole as
  written files; the merge job settles a written file's conflict with
  the landing branch's copy; no test at the gate holds a journey to a
  committed recording or page.
- **A request is checked against the print** (`journey-writer-requests.md`, section 1): whoever
  lands one runs `look-compare.sh` in its own tree, which prints into
  scratch and writes the account's rows, committing no written file
  (`journey-writer-requests.md`, 1.1). After it lands, the writer's run renders it in Chromium beside
  the print with `look-render` (`journey-writer-requests.md`, 1.2) and writes the verdict into
  `looks.html` (`journey-writer-requests.md`, 1.3); `look-refresh.sh` drops the request once main's
  `looks.html` holds it. Its foot lists the candidates to retire (`journey-writer-requests.md`, 1.3).
- **After a landing, never inside one:** a walk that stops holds no
  landing. It fails the writer's run, an after-landing red of its
  runner's (the user, 2026-11-18).

## 2. The written files

Under `web/journeys/` and `tui/journeys/`:

| Written by the writer alone | Sources, any change's |
|---|---|
| a journey's pages: `NN.html`, and the TUI's `NN.txt`, NN two digits or more | a journey's `walk`, `README.md` and, where it has one, `scripts.json` |
| beside the browser's journeys, `skein-ui.css` and `journey.js`; beside the TUI's, `theme.css` and `tui-journey.js` | each root's `overlay.css`, empty on main; the `README.md` beside each root |
| beside each root's journeys, `looks.html`, the render check's verdicts and the candidates to retire (`journey-writer-requests.md`, 1.3) | |
| a leftover `recording`, removed once its journey prints | |

- **A source changes with what a person does:** a change that renames a
  control a `walk` names edits that line in the same change, as it edits
  a test (`journey-walks.md`, 2.2). The writer's next run walks it.
- **A new journey** lands as its sources. Its pages follow from the
  writer, printed in the world (`journey-walks.md`, 6).
- **Pages are replaced whole:** a successful print removes surplus
  numbered pages and its journey's leftover `recording`. A directory
  with no `walk` loses its pages (`journey-walks.md`, 5.2).
- **A journey retired goes whole:** one change of its own deletes its
  directory, sources and pages together, taken as written files
  (section 5); its `walk` never goes alone, since no run removes the
  pages it leaves. The person says which go, from `looks.html`'s
  candidates (`journey-writer-requests.md`, 1.3).

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
[--within DURATION]`, under `temper/scripts/`, run every ten minutes
by a timer on the after-landing worker (4.1), in a checkout of its own
whose `origin` is the repository; each run decides by step 3 whether
to print. With `--project` it opens a change, and needs `temper` on
its `PATH` signed in; without it, it opens nothing, and needs no ctl
directory and no way to push. Its render check (step 5) needs
Chromium, `SKEIN_TEST_CHROMIUM` naming it, as the browser tiers' does.

- **The script runs the command:** `scripts/journey-pages.sh` runs
  `cargo run --locked --quiet --manifest-path ROOT/Cargo.toml -p
  temper-client-journeys -- writer` with its arguments, ROOT the
  repository holding the script, in the directory it was run in; its
  exit is the command's. The command, `temper-client-journeys writer`
  (`../testing/tools.md`, 2), is the run: this section's words, lines
  and exits are its own, under the script's name (`journey-pages:
  WHY`). It asks for each effect as a value, a child (`git`, `cargo`,
  `temper`) with its arguments, a file or a directory, or whether a
  process is live, or the clock's time in Unix seconds, and decides on
  the answer, so that its decisions are tested on canned answers
  (section 7). Every child runs with
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
  journey (section 3, item 4), then the check's, one a request
  (step 5), then one last line: `main's pages are TIP's print`, `wrote
  NAME COMMIT`, or a waiting line, `waiting on change N`, `waiting on
  COMMIT` or `waiting on the hour after LAST12`. An early refusal
  prints nothing there; every other word goes to stderr.
- **Its state,** two files in the checkout's git directory beside its
  lock: `journey-pages`, the last offer (step 2), and
  `journey-pages.finished`, the last finished run, one line, `TIP
  SECONDS`: the tip it ran on and the time it started. A finished run
  is one that wrote nothing (step 6), or whose offer landed (step 2).
  Waiting on its own offer finishes nothing.

1. **Alone:** a lock, the directory `journey-pages.lock` in the
   checkout's git directory holding the run's process number, made
   with `mkdir`. A second run while a live process holds it is refused,
   naming the lock and the process, exit 2; a lock whose process has
   gone is taken over. A missing or invalid process number is refused.
   The run removes its lock as it exits; a run a signal ends leaves it,
   and the next run takes it over, its process gone.
2. **One at a time:** the git directory keeps `journey-pages`, the last
   offer: its first line `task N`, the change the script opened, or
   `commit COMMIT`, the commit it wrote where it opened none; its second
   `run TIP COMMIT SECONDS`, the tip its run printed, the commit it
   offered and when the run started. While `temper ctl task show N
   --json` says the task has not ended, the script prints `waiting on
   change N`, exit 0, having fetched and started nothing. For a commit,
   it fetches `main` and waits, `waiting on COMMIT`, exit 0, until main
   holds that print: COMMIT reachable from `origin/main`, or any commit
   reachable from it whose message carries `Printed:` with COMMIT's
   TIP, a print merged by hand or squashed. Once the offer has ended or
   is held, the file goes and the run goes on; where main, as step 3
   fetches it, holds the offer's print by that rule, its run is
   finished, and `journey-pages.finished` takes its TIP and SECONDS. An
   offer of the earlier form, its first line alone, is read as before
   and finishes no run. Whoever drops an offer by hand removes the
   file. An unreadable offer or task answer fails the run.
3. **On main's tip:** a checkout with tracked changes, or a rebase, a
   merge, a cherry-pick or sequencer in progress, is refused. The script
   fetches `main` from `origin`, TIP `origin/main`, and asks the clock
   once, NOW.
   - **Whether it runs,** by the last finished run, LAST at SECONDS:
     where LAST is TIP, its last line is `main's pages are TIP's
     print`, exit 0. Otherwise it runs where no run has finished, where
     LAST is not an ancestor of TIP, where a commit of `LAST..TIP`
     carries a `Look-landed:` trailer, or where an hour has passed since
     SECONDS. So a request's verdict comes within ten minutes of its
     landing, and main's other landings cost one run an hour. Else its
     last line is `waiting on the hour after LAST12`, exit 0. An
     unreadable `journey-pages.finished` is warned of and read as none.
   - It checks out NAME at TIP.
4. **Prints:** `TEMPER_INTERNAL_BUNDLE=none cargo run --locked -p
   temper-client-walk -- pages` (`journey-walks.md`, 5.2), once,
   regardless of which directories the roots hold. It passes stdout on
   as it comes. Exit 2 or 3 ends the run with nothing committed or
   offered; exit 1 still offers the other journeys' changes.
5. **Checks the requests landed since** (`journey-writer-requests.md`, 1.2, 1.3):
   - **Which:** each request NAME whose `Look-landed: NAME` a commit
     reachable from `origin/main` carries, L its newest such commit, read
     as PRINTED is (`journey-writer-requests.md`, section 1), where its root's `looks.html` as TIP
     holds it has no entry standing for L (`journey-writer-requests.md`, 1.3); oldest L first.
   - **Its request:** fetched from `origin`'s `docs/look/NAME`, the
     remote the whole run reads, and read as `journey-writer-requests.md`, 1.1, step 2, reads a
     fetched tip, C its tip and P its parent. A branch `origin`
     lacks, a tip that is a merge or a root, or a tip without exactly
     one `Look-request: NAME`, gives the entry `not checked`, with why;
     a fetch that fails otherwise is a check that can't finish.
   - **Pending:** where a journey whose pages C changes from P printed
     `stopped` in step 4, the request gets no entry until a run prints
     it; its line is `pending NAME: JOURNEY stopped`.
   - **Rendered:** otherwise `cargo run --locked --quiet -p
     temper-client-journeys -- look-render NAME C` (`journey-writer-requests.md`, 1.2), in the
     checkout, on the print in place. Its exit 0 makes the entry from
     its lines, and its line is `checked NAME: RESULT`.
   - **A check that can't finish** (`look-render`'s exit but 0, lines
     the writer can't read (`journey-writer-requests.md`, 1.3), a fetch, git or a file failing)
     restores both roots as TIP holds
     them, untracked files under them removed, so that no written file
     is left changed. The run names why, exits 1, commits and offers
     nothing and finishes no run, so the next tick starts again.
   - Each root's `looks.html` gains its new entries at its top, newest
     first.
   - **The candidates to retire** (`journey-writer-requests.md`, 1.3), once the entries are made, from
     each root's page as the run leaves it: each request NAME whose
     entry standing for L, its newest `Look-landed`, has the result
     `identical`, C that entry's `data-look-commit`.
     - **What C added:** each journey J whose `J/walk`, under either
       root, C adds from its parent. Where the checkout lacks C, it
       fetches `docs/look/NAME` as above first; a branch `origin`
       lacks, or a fetch that leaves the checkout without C, lists
       none of NAME's journeys, and stderr says why
       (`journey-pages: cannot read what NAME added: WHY`). A fetch or
       git failing otherwise is a check that can't finish.
     - **A candidate:** such a J that TIP holds as a journey (section
       3, item 1), where no journey's `walk` at TIP, under either root,
       has `after J` for its second line (`journey-walks.md`, 2.3),
       since its going would stop the other.
     - A request whose newest `Look-landed` has no entry yet lists
       none: a repair's landing waits for its own verdict. Where no
       standing entry is `identical`, nothing is read for them.
   - Each root's `looks.html` is written only where its bytes change:
     a new entry, or a foot that changes, appears or goes (`journey-writer-requests.md`, 1.3).
6. **Nothing differs,** no page and no `looks.html`: the run is
   finished, `journey-pages.finished` taking `TIP NOW`; its last line is
   `main's pages are TIP's print`, its exit the print's, including 1
   where a walk stopped.
7. **Written files alone:** a changed path that is not section 2's
   written file ends the run, naming it, nothing committed.
8. **Its commit,** on TIP, of the written files: the subject
   `Journey pages printed at TIP12`, TIP's first twelve hexadecimal
   digits; a paragraph of step 5's lines where there are any; a
   paragraph of the `stopped` lines where there are any; and a last
   paragraph of one line, `Printed: TIP`, TIP's whole hash. Its author
   and committer dates are TIP's committer date, so that the same files
   on the same tip make the same commit. A landing that changes no page
   still gives a commit, its verdict.
9. **Pushed** as above, where it pushes.
10. **Opened,** with `--project`: `temper ctl change open --project
    PROJECT --branch NAME --at COMMIT --title SUBJECT --body BODY` with
    the budget given and `--key journey-pages-COMMIT`, COMMIT's whole
    hash: a retry of the same offer after a lost answer, the same
    commit, opens nothing twice, and a new offer at the same tip,
    another commit, opens anew.
11. **Kept:** `task N` from the answer, or with no `--project`
    `commit COMMIT`, then `run TIP COMMIT NOW`, go to step 2's file, and
    the last line is `wrote NAME COMMIT`.
12. **Its exit:** 0, nothing to write, a commit written, waiting, or no
    run due; 1, a journey left unprinted (the others written and
    offered), a check that couldn't finish, git, cargo or `temper ctl`
    failed, a push refused, or a path outside the written files
    changed; 2, its arguments, its checkout or its lock refused.

- **It settles:** its change lands as any opened change does; the next
  tick finds its offer landed and its run finished (step 2), and the
  next run, at the hour or at a request's landing, prints the tip then.
  Without `--project`, whoever runs it lands the commit, as it is, by a
  merge or squashed, and step 2 waits for that.
- **How far behind:** between a landing and the writer's change, main's
  pages are an earlier tip's, up to an hour, or ten minutes after a
  request's landing, and the last `Printed:` on main says which.
  Nothing reads them in between but a person.
- **A walk that stops** stops on every later tip, until a change repairs
  the walk or the client: one after-landing red, its runner's. The other
  journeys are printed and offered all the same. A new journey whose
  walk stops has no page until then.
- **Its change holds written files alone,** so it meets no conflict a
  goal made; the rules covering main ask of it what they ask of any
  change. Since the run waits on its own offer, its change meets no
  other change to `looks.html`.

### 4.1 Its timer

The after-landing worker's owner installs it once, for the checkout
CHECKOUT that the run keeps, as a user timer of systemd:

```ini
# ~/.config/systemd/user/journey-pages.service
[Unit]
Description=The journeys' writer's run

[Service]
Type=oneshot
WorkingDirectory=CHECKOUT
Environment=PATH=PATH
Environment=SKEIN_TEST_CHROMIUM=CHROMIUM
ExecStart=CHECKOUT/temper/scripts/journey-pages.sh --project PROJECT --as-allowed

# ~/.config/systemd/user/journey-pages.timer
[Unit]
Description=The journeys' writer's run, every ten minutes

[Timer]
OnBootSec=10min
OnUnitInactiveSec=10min

[Install]
WantedBy=timers.target
```

then `systemctl --user daemon-reload` and `systemctl --user enable
--now journey-pages.timer`. PATH holds `cargo`, `git` and `temper`,
signed in as whoever starts look requests' goals, so that its change's
end brings them the verdict (`web/look-requests.md`, 4); CHROMIUM is the Chromium the browser tiers use; PROJECT and the
budget form are the owner's (`--budget AMOUNT` in place of
`--as-allowed`). A tick ten minutes after the last one ends never
overlaps it, and the lock (step 1) holds one run at a time whatever
starts it. Each tick decides itself whether to print (step 3); its
lines go to the user journal (`journalctl --user -u journey-pages`).

## 5. Goals kept off

- **The delivery check** (`tools/pre-pr/design.md`, 4, check 8): a
  delivery whose own changes hold a written file and any other path is
  refused, naming `cargo run -p temper-client-walk -- print --out DIR`
  for every journey, and how to restore the files; so is a tree whose
  `overlay.css`, under either root, holds a byte. A delivery of written
  files alone passes it. The written set includes leftover recordings,
  and every path of a journey's directory deleted whole, its `walk`
  among them, so that a journey retired (section 2) passes alone. A
  `walk` deleted while its directory still holds a written file is
  refused, naming the walk: no run removes those pages, and a merge
  that brings main's print back into a retiring branch shows there.
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

Moved to [`journey-writer-requests.md`](journey-writer-requests.md), section 1.

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
  its commit. Its timer's rule: with no finished run it runs; at the
  finished run's tip, nothing runs; a `Look-landed` since runs it within
  the hour; none, under the hour, `waiting on the hour after LAST12`;
  an hour past, it runs; an ended offer whose print main holds finishes
  its run, one whose print it doesn't finishes none, and an offer file
  of one line is read as before. Its check, the `look-render` child's
  answer canned: a landed request with no entry gets one, and a checked
  one is skipped; a newer `Look-landed` is checked again, the older
  entry kept below; a journey whose print stopped keeps its request
  pending, with no entry; a branch `origin` lacks gives `not checked`;
  an `uncompared` page, hand-made, is listed as not compared; a page
  that is no journey's, by its own name, linked as a printed page is,
  and a path with a `/` below its directory, or a name starting `.`, a
  check that can't finish; a
  landing that changes no page still offers its verdict, with
  `Printed:`; a check that can't finish restores the written files,
  exit 1, nothing committed or offered; a value holding `<`, `&`, `"`
  and an escaped tab is written escaped; a TUI request's entry goes
  beside the TUI's journeys; the commit's dates are TIP's, and a new
  offer at the same tip, another commit, gets a new key. Its
  candidates: a journey a request added, its verdict `identical` and no
  walk `after` it, listed; one another starts `after`, or whose verdict
  differs, not; a C the checkout lacks fetched first, a branch `origin`
  lacks listing none; a foot whose candidate has gone removed, an
  unchanged one keeping the page's bytes.
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
  refused before fetching, exit 2. A request with a `Look-landed` on
  main kept until its root's `looks.html` there holds an entry for its
  newest, then dropped whatever the result; an entry for an older
  `Look-landed` dropping nothing; a request with none dropped by
  `Printed:` as before; their report lines.
- **The render check's decisions,** step tests in the same library, the
  `look-render` command on canned answers (git's output, the files
  read and written, Chromium's start and reads), then its lines,
  stderr and exit (`journey-writer-requests.md`, 1.2): equal pages, `identical`; one property
  differing at each width, three `differs` lines; a box 1 px off equal,
  2 px off differing; a touched element with no pair, `pair`; an element
  touched by a rule's selector, its `:hover` dropped; a page whose
  elements don't follow its start tags, a page with elements past
  `Limits::elements` in either copy, a hand-made page and a page
  with nothing printed beside it, each `uncompared`; nothing touched,
  `nothing to render`; a value holding a tab and a `\` escaped; past 50
  differences, `more`; the copies' `details` and `dialog` opened and
  each page opened kept to files; a read refused, exit 3; a tip that
  isn't C, exit 2. A hand-made page whose `README.md` names a gallery
  state and the page as the request, rendered beside that state's page,
  the gallery printed once for all such pages; a page beside it that
  the `README.md` calls the look before, or doesn't name, `not the
  requested look`, and no difference; one whose `README.md` names no
  state, or two, `hand-made`; a state the gallery's print lacks,
  `nothing printed beside it`; the gallery's print failing, exit 3; a
  page on one line paired a tag a line. On a page drawn on a gallery
  state: an element a rule selects, its declared property and a
  shorthand's longhand differing, each a row, and its box, an
  undeclared property and an element no rule selects differing, none;
  a declared custom property differing, a row; a selected element with
  no pair passed over, and none paired, `nothing printed beside it`. On
  a journey's page, a box and its `width`, `height`, `inline-size`,
  `block-size`, `transform-origin` and `perspective-origin` differing,
  one `box` row, and those properties differing within a box 1 px off,
  none. No Chromium starts.
- **The render check's lines, read back,** so that the two forms
  can't part: the two sides were built apart, each on its own canned
  answers, and a hand-made page's path, printed by one and refused by
  the other, once stopped the writer's run.
  - **Listed** in one file of the library's sources, as the look
    commands' git lists are: each line `look-render` prints in a step
    test, under that test's name, in the order printed. Each of those
    step tests checks that the file's lines under its name are
    exactly its stdout's, and one step test that every name the file
    holds is one of theirs; a line missing or left over fails, naming
    it.
  - **Read** by one step test: each listed stdout, through the
    writer's reader (`journey-writer-requests.md`, 1.3), read whole, its pages and result written
    back as step 8 writes lines are its listed lines byte for byte.
    Across them it requires each line of step 8's table, each result
    the reader accepts, each why the render check gives, and a printed
    page, a page that is no journey's and a page under each root: a
    form one side adds and the other doesn't read fails it. It asks no
    effect.
- **The kit's reads** of Chromium, in skein's browser suite
  (`skein/docs/design/testing/browser.md`, 9), in the session its
  viewport story starts: written and compiled in the plan, run after
  landing. None is in temper's browser tier. The command's file access
  and hidden scrollbars (`journey-writer-requests.md`, 1.2, step 4) are held by the kit's step tests
  of its arguments (browser.md, 9); the tool's host asks for both at
  every start.
- **The comparison's decisions,** step tests in the same library, the
  `look-compare` command on canned answers (git's output and exit, the
  print's lines and exit, the files read), then its rows, stderr and
  exit (`journey-writer-requests.md`, 1.1): every edit landed, exit 0; an edit missing, its hunk a
  row, exit 1; one tag changed by both sides, a conflict row;
  generated ids renumbered by a moved element, no row; a rule landed,
  one differing, one missing, one removed; a request of rules alone,
  rule rows alone and no print; a new walk's page compared whole; a
  stopped walk, a row, exit 1; the TUI's rules listed to read; a
  hand-made directory not compared; a tip that isn't C, and each
  refused argument, exit 2; a print that can't finish, exit 3. Its
  request (`journey-writer-requests.md`, 1.1, step 2): a C the checkout holds read with no fetch;
  one it lacks fetched from `forge` beside `origin`, from `origin`
  beside another, from an only remote, and refused, exit 2, with no
  remote or several and neither. Each of its effects is a value these
  steps answer, so its script has no witness.
- **The look commands' git,** in two halves, so that each argument
  list a step test of `look-compare`, `look-render` or `look-refresh`
  asks is one real git has accepted:
  - **Listed** in one file of the library's sources, each list under
    the name of the step test that asks it. Each of those step tests
    checks that the file's lists under its name are exactly the git
    lists it asked, and one step test that every name the file holds
    is one of theirs. A list missing or left over fails its test,
    naming the line to add or remove. None runs git or another step
    test.
  - **Run** by one test among the witnesses, in
    `temper-client-journeys`'s suite: each listed list, with real git
    in one scratch repository that holds the commits and refs they
    name, since git refuses an unknown revision before it reads the
    options. A list git answers with its usage, exit 129, fails it,
    naming the list: the canned answers can't see git refuse one.

  It starts git once per list, a process test's cost, so it runs where
  the witnesses run, at the user's word of 2026-11-26, and the step
  tests, which every delivery that selects the library runs, stay
  cheap. Turned down: running every look step test again to gather
  the lists, which a test outside the library can't, and which made
  the one test many times a step test's reference.
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
  of their own, one at a time. The script then goes, and its timer
  (4.1) with it.
- **The playground's `story-a.log`**
  (`temper/tests/playground/tests/fixtures/`), another recording on main
  that the merge job regenerates (`tools/merge/README.md`): the same
  rule would have a test make it and no file keep it.
- **The render check's reads kept to its copies:** with file access a
  page reads any file its user may (`journey-writer-requests.md`, 1.2, step 4); the kit's `files`
  could continue only the `file://` URLs below the check's scratch
  directory.
