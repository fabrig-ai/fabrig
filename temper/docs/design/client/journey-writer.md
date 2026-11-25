# The journeys' writer

Provisional, 2026-11-21, revised 2026-11-25. Who writes main's journey
files, and when. One writer, run by a timer after landings, prints
every journey in a world with `temper-client-walk pages`, renders each
newly landed look request beside the print, and offers what differs,
pages and verdicts, as a change of its own; nothing else writes those
files. The journeys are the browser's (`web/journeys.md`) and the
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
  since (6.2), and offers what differs as one change, opened from a
  pushed branch, its message ending `Printed: COMMIT`, or, given no
  project, as a commit on a branch of its own for its runner to land.
  While an offer of its own is open it starts no other, and one run at
  a time holds the checkout. Once it has run on a tip and found nothing
  to write, main's pages are that tip's print.
- **Goals are kept off** (section 5): the delivery check refuses a
  delivery that changes a written file beside any other file, and a
  nonempty overlay; the merge job settles a written file's conflict with
  the landing branch's copy; no test at the gate holds a journey to a
  committed recording or page.
- **A request is checked against the print** (section 6): whoever
  lands one runs `look-compare.sh` in its own tree, which prints into
  scratch and writes the account's rows, committing no written file
  (6.1). After it lands, the writer's run renders it in Chromium beside
  the print with `look-render` (6.2) and writes the verdict into
  `looks.html` (6.3); `look-refresh.sh` drops the request once main's
  `looks.html` holds it.
- **After a landing, never inside one:** a walk that stops holds no
  landing. It fails the writer's run, an after-landing red of its
  runner's (the user, 2026-11-18).

## 2. The written files

Under `web/journeys/` and `tui/journeys/`:

| Written by the writer alone | Sources, any change's |
|---|---|
| a journey's pages: `NN.html`, and the TUI's `NN.txt`, NN two digits or more | a journey's `walk`, `README.md` and, where it has one, `scripts.json` |
| beside the browser's journeys, `skein-ui.css` and `journey.js`; beside the TUI's, `theme.css` and `tui-journey.js` | each root's `overlay.css`, empty on main; the `README.md` beside each root |
| beside each root's journeys, `looks.html`, the render check's verdicts (6.3) | |
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
5. **Checks the requests landed since** (6.2, 6.3):
   - **Which:** each request NAME whose `Look-landed: NAME` a commit
     reachable from `origin/main` carries, L its newest such commit, read
     as PRINTED is (section 6), where its root's `looks.html` as TIP
     holds it has no entry standing for L (6.3); oldest L first.
   - **Its request:** read from `origin`'s `docs/look/NAME` as 6.1,
     step 2, reads it, C its tip and P its parent. A branch `origin`
     lacks, a tip that is a merge or a root, or a tip without exactly
     one `Look-request: NAME`, gives the entry `not checked`, with why;
     a fetch that fails otherwise is a check that can't finish.
   - **Pending:** where a journey whose pages C changes from P printed
     `stopped` in step 4, the request gets no entry until a run prints
     it; its line is `pending NAME: JOURNEY stopped`.
   - **Rendered:** otherwise `cargo run --locked --quiet -p
     temper-client-journeys -- look-render NAME C` (6.2), in the
     checkout, on the print in place. Its exit 0 makes the entry from
     its lines, and its line is `checked NAME: RESULT`.
   - **A check that can't finish** (`look-render`'s exit but 0, a
     fetch, git or a file failing) restores both roots as TIP holds
     them, untracked files under them removed, so that no written file
     is left changed. The run names why, exits 1, commits and offers
     nothing and finishes no run, so the next tick starts again.
   - Each root's `looks.html` gains its new entries at its top, newest
     first, and is written only where it gains one.
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
  squashed into another commit.
  - **A request that landed** has a `Look-landed: NAME` on main, L its
    newest, read as PRINTED is. It is dropped once the `looks.html` of
    the branch's root (`look-requests` the browser's, `tui-requests`
    the TUI's), as `origin/main` holds it, has an entry standing for L
    (6.3), whatever its result, and kept until then: a difference comes
    back as a repair, under the request's trailer, carrying a new
    `Look-landed`.
  - **A request with no `Look-landed`** on main, as one landed before
    the trailer was, is dropped by the print: once the commit carrying
    its `Look-request` trailer is PRINTED or an ancestor of it. A
    request whose trailer is on main and not yet printed stays, on the
    pages main still holds.

  Its report opens with `pages printed at PRINTED12, N landings since`,
  N main's first-parent landings after the print's carrier is included,
  or `pages not printed yet` where main holds none; then, for each
  request, `kept NAME`, `kept NAME, done on main by C12, not yet
  printed`, `dropped NAME, done on main by C12, printed at PRINTED12`,
  `kept NAME, landed by L12, not yet checked`, or `dropped NAME, landed
  by L12, checked: RESULT`.
- **Whoever lands a request** compares it with
  `scripts/look-compare.sh NAME C` (6.1), which prints its journeys
  into scratch in the landing's own tree (`web/look-requests.md`, 3).
  It commits no written file, and its last commit carries
  `Look-landed: NAME`. The print needs no box and reads no committed
  recording or page.
- **The render check is the writer's run's** (section 4, step 5): it
  renders each newly landed request beside the print with
  `look-render` (6.2) and writes the verdict into `looks.html` (6.3),
  offered with the pages (`web/look-requests.md`, 4).

### 6.1 The comparison

`scripts/look-compare.sh NAME C`, beside `look-refresh.sh`, run by
whoever lands the request NAME in the landing's own tree, C the
request's commit on `docs/look/NAME` (`web/look-requests.md`, 3 and
7). It runs `cargo run --locked --quiet --manifest-path ROOT/Cargo.toml
-p temper-client-journeys -- look-compare NAME C` as `look-refresh.sh`
runs `look-refresh` (section 4): the command (`../testing/tools.md`, 2)
asks for each effect as a value, a child with its arguments or a
file, and decides on the answer. Its words go to stderr as
`look-compare: WHY`, and its exit is the script's. It starts no
browser, terminal or box, so a plan runs it. Its rows are the
account's evidence; the account's judgement stays the landing's
(`web/look-requests.md`, 5).

1. **Its arguments:** NAME, `[a-z0-9][a-z0-9-]*`, and C, 7 to 64
   lowercase hexadecimal digits. Any other argument or count is
   refused.
2. **Reads the request:** it fetches `docs/look/NAME` from `origin`,
   with no tags. The fetched tip must be C, its hash starting with C's
   digits, with one parent, P, and exactly one `Look-request: NAME`
   trailer. P holds the requests below, so P to C is this request's
   edit alone.
3. **Sorts what C changes** from P, under the two journeys roots:
   - a page, `NN.html` or the TUI's `NN.txt`, of a directory whose
     `walk` in C starts with exactly `walk 2`: a journey's page,
     compared (step 5);
   - any file of a directory with no such `walk` in C: a hand-made
     directory, one row, not compared, since no print meets it;
   - a root's `overlay.css`: its rules (step 6).

   Any other path, a walk's sources among them, gives no row.
4. **Prints** the journeys of step 3's pages, once, into `print` under
   `look-compare` in the checkout's git directory, which each run makes
   afresh and leaves for reading: `TEMPER_INTERNAL_BUNDLE=none cargo
   run --locked --quiet -p temper-client-walk -- print --out DIR
   JOURNEY...` (`journey-walks.md`, 5.1).
   - A journey whose directory in the landing's tree has no `walk` of
     version 2 is a row, missing, and is not printed.
   - A TUI journey whose name has a `walk` under the browser's root
     too can't be printed by its name, which finds the browser's first
     (`journey-walks.md`, 5.1): the comparison stops before printing,
     exit 3, naming both directories, for the landing to rename one.
   - A `stopped NAME: WHY` line is a row, and that journey's pages get
     none. A stopped walk holds no landing (section 1): the landing
     repairs one its own change stopped, and the account names any
     other.
   - A request that changes no journey's page prints nothing.
5. **Compares each page three ways, with git:** the page as P holds it
   (what the person drew on), as C holds it (what they drew), and as
   DIR holds it (what landed).
   - Each is first made comparable: its generated ids are numbered
     afresh by `temper-client-walk-lines`' `print::numbered_ids`
     (`web/journeys.md`, 5.3), so that an element moved on either side
     renumbers no reference. The three texts are written under
     `look-compare`.
   - `git merge-file -p` applies the person's edit, P's page to C's,
     to DIR's page. Where the result equals DIR's page, the page
     landed. Otherwise each hunk between DIR's page and the result,
     by `git diff --no-index`, is a row: an edit that didn't land, or
     landed otherwise.
   - Each conflict, where both the person and the landing changed one
     tag, is a row, to account for by reading.
   - A page P doesn't hold, such as a page of a walk the request adds,
     is compared whole: each hunk between C's page and DIR's is a row. The
     world prints the same bytes from the same tree, so each difference
     is this landing's or another's since M, never the world's.
   - A page C holds and DIR lacks is a row, missing. A page C removes
     has landed where DIR lacks it too.
6. **Compares the rules** each overlay of C adds, changes or removes
   from P's. A rule is its selector, its declarations and the
   conditional at-rules around it, `@media` and `@supports`,
   outermost first. In one sheet, two rules of one selector and
   condition are one, the later declaration winning. Selectors,
   conditions and values compare with comments dropped and whitespace
   collapsed, property names lowercased but for custom properties.
   - A browser rule is matched against the rules of its selector and
     condition in the landing's `temper.css` and `task-pages.css`
     (`temper/crates/temper-client-shell-web/`), taken together: landed,
     each of its declarations there with its value; differing, naming
     those that aren't; or missing, where no rule has its selector and
     condition.
   - A removed rule has landed when no landed rule of its selector and
     condition keeps its declarations.
   - Listed for the landing to read: the order the browser's rules
     cascade in, one row, which the render check sees
     (`web/look-requests.md`, 4); each of the TUI's rules, which land
     in its theme's code (`tui/journeys.md`, 10); and each other
     at-rule C changes, by its prelude.
7. **Writes the account's rows** on stdout, as `web/look-requests.md`,
   5's table: its header, then a row for each directory and page, the
   browser's root first, each in path order; then the rules, the
   browser's first, in their overlay's order, removed rules after;
   then one row to fill for the states C's message names in words.
   Each hunk and conflict a row names goes to stderr under that row's
   item. A request of rules alone prints nothing, and gets rule rows
   alone beside the row to fill.

   | Item | Result |
   |---|---|
   | `` `web/journeys/J/NN.html` ``, a page by its path below `temper/docs/design/client/` | `landed`, or `missing` where DIR lacks it |
   | `` `PAGE`, lines A-B ``, DIR's lines; `line A` for one; `after line A` where DIR has none | `differs` or `conflict` |
   | `` `web/journeys/J` ``, a directory | `stopped: WHY`, `missing: no walk`, or `not compared: hand-made` |
   | `` rule `SELECTOR` ``, or `` rule `SELECTOR` in `CONDITION` `` | `landed`, `differs: PROPERTY, ...` or `missing` |
   | `` removed rule `SELECTOR` ``, with its condition as above | `landed`, or `differs: kept` |
   | `the order the rules cascade in`; `` TUI rule `SELECTOR` ``; `` at-rule `PRELUDE` `` | `to read` |
   | `each state the message names in words` | `to fill, a row each` |

   A `|` within a cell is written `\|`.
8. **Its exit:** 0, every row landed, to read or to fill; 1, a row
   differs, is missing, conflicts or stopped; 2, its arguments, a
   fetch that fails, a tip that isn't C, or a request it can't read: a
   merge, a root commit, or not exactly one `Look-request: NAME`; 3,
   the comparison couldn't finish: another git child, the print (an
   exit but 0 or 1) or a file failed. On 2 and 3 no row is written.

### 6.2 The render check

`temper-client-journeys look-render NAME C`, the journeys tool's
command (`../testing/tools.md`, 2), which the writer's run asks after
its print (section 4, step 5), in its checkout, on the print in place;
C is the request's commit at `docs/look/NAME`. A person may run it in
any checkout whose roots hold a print. As the comparison does (6.1), it
asks for each effect as a value, Chromium's start and reads among them,
and decides on the answer, so that its decisions are tested on canned
answers (section 7). Its words go to stderr as `look-render: WHY`. It
needs `SKEIN_TEST_CHROMIUM`, Chromium's path, and starts no box or
terminal.

1. **Its arguments and its request** as 6.1, steps 1 and 2: NAME and
   C; `docs/look/NAME` fetched from `origin`, its tip C, with one
   parent P and exactly one `Look-request: NAME`.
2. **Which pages,** under the two roots in C, a journey being a
   directory whose `walk` in C starts with exactly `walk 2`:
   - each `NN.html` of a journey that C changes from P, a page of a
     walk the request adds among them;
   - each `NN.html` of any journey under a root whose `overlay.css` C
     changes, where a rule C adds, changes or removes there selects an
     element (step 5).

   A page C changes in a directory that is no journey is `uncompared`,
   `hand-made`; a journey's page the checkout's print lacks at its path
   is `uncompared`, `nothing printed beside it`. The TUI's `NN.txt` is
   not rendered: its text was compared at the landing (6.1).
3. **Two copies,** made afresh under `look-render` in the checkout's git
   directory and left for reading: `request`, C's two journeys roots and
   `temper/crates/temper-client-shell-web/` from git, and `print`, the
   checkout's same files as they stand, each root's `overlay.css` empty.
   Each keeps their paths below the repository's root, so that a page's
   links resolve within its copy (`web/journeys.md`, 5.3). In each
   copy's pages every `details` and `dialog` start tag gains `open`
   where it lacks it, so that every disclosure and dialog shows; no other
   byte changes, and nothing is clicked. Nothing else is copied: no
   credential, ctl directory or home is in either.
4. **One Chromium,** started by the kit's command
   (`skein/docs/design/testing/browser.md`, 3), its profile under the
   scratch directory, with one person. Each page of step 2 is opened
   from each copy as a `file://` page kept to files (browser.md, 5), so
   that every request outside `file://` fails. A page's `Trouble` goes
   to stderr under the page, and the check judges none.
5. **The elements the request touched,** in the request's copy:
   - **Its start tags are its elements:** the check numbers the page's
     start tags in source order, each a `<` then an ASCII letter outside
     comments, the doctype and raw text. The page's `Elements` must give
     the same local names in the same order, in each copy, else the page
     is `uncompared`, `its markup doesn't parse as written`. So the k-th
     start tag is the k-th element, and its line is the line it starts
     on.
   - **By its edit:** each element whose start tag starts on a line that
     P's page to C's changes, by `git diff --no-index -U0` of their
     comparable texts (6.1, step 5); every element where P lacks the
     page.
   - **By its rules:** each element that a rule C adds, changes or
     removes in the page's root's overlay selects (`Elements` with its
     selector), the selector read as 6.1, step 6, reads it, with its
     pseudo-elements and its user-action pseudo-classes (`:hover`,
     `:active`, `:focus`, `:focus-visible`, `:focus-within`) dropped,
     since the element they style is the one touched. A selector
     Chromium refuses selects nothing.
   - Each touched element brings its descendants. A request whose edits
     and rules touch no element on any page is `nothing to render`.
6. **Paired by the pages' alignment:** the request's page and the
   print's, each made comparable as 6.1, step 5, makes it, diffed by
   git. A start tag on a line the diff keeps pairs with the same start
   tag of that line on the other side; within a hunk, the two sides'
   start tags pair in order while their names agree, and the rest have
   no pair. A touched element with no pair is one difference, `pair`.
7. **Compared at each width,** 1440, 1000 and 420 CSS pixels by 900,
   both copies' pages set to it: each touched element's and its pair's
   `Style` and `Rect`. A property differs where its two values differ as
   bytes, custom properties (`--*`) aside, since the properties they
   feed are compared; a box differs where its x, y, width or height
   differs by more than 1 px, or where one side has a box and the other
   none.
8. **Its lines,** on stdout once the check has finished, fields
   separated by a tab, a `\`, tab, CR or LF within a field written
   `\\`, `\t`, `\r` or `\n`:

   | Line | Says |
   |---|---|
   | `identical PAGE` | no touched element of PAGE differs at any width |
   | `differs PAGE WIDTH ELEMENT PROPERTY REQUESTED LANDED` | one difference: WIDTH `1440`, `1000` or `420`, and `-` for a pair; PROPERTY a computed property, `box` (each value `X,Y WxH`, or `none`), or `pair` (`present` against `none`) |
   | `more PAGE N` | N of PAGE's differences past its first 50, not listed |
   | `uncompared PAGE WHY` | `hand-made`, `nothing printed beside it`, or `its markup doesn't parse as written` |
   | `result RESULT` | last, once: `identical`, `differs`, `not compared` where no page was compared, or `nothing to render` |

   PAGE is the page's path below `temper/docs/design/client/`, as 6.1's
   rows name it (`web/journeys/J/NN.html`); ELEMENT is `LINE TAG`, its
   start tag's line in the request's page and its local name with its
   classes (`41 button.temper-chip`). Pages go in path order, the
   browser's root first; a page's differences by width, widest first,
   then element, then property. A value is as Chromium gives it, cut to
   the kit's `Limits::text`, and is escaped for HTML by the writer
   (6.3).
9. **Its exit:** 0, its lines written, whatever its result; 2, as 6.1,
   step 8: its arguments, a fetch that fails, a tip that isn't C, or a
   request it can't read; 3, the check couldn't finish: a git child, a
   file, `SKEIN_TEST_CHROMIUM` unset, Chromium's start, a read refused
   or the browser gone. On 2 and 3 it writes nothing on stdout.

- **States in words aren't rendered:** the landing's account covers
  them (`web/look-requests.md`, 5), and the entry says so (6.3).
- **The TUI's** themed frames are rendered as the browser's pages are,
  under `theme.css` and the overlay, and the entry says these are the
  browser's renders, not a terminal's. A real terminal's look stays the
  after-landing owner's (`tui/journeys.md`, 11).
- **Why by source lines:** the comparison's hunks are the page's lines
  (6.1), and the print writes a tag a line (`web/journeys.md`, 5.3), so
  a start tag's line names the element a person edited; Chromium's
  order of elements, checked against the source's, maps each to its
  node with no script and no mark in the page. Turned down: marking
  each element with an attribute in the copies, which a rule could
  select, and pairing by Chromium's tree alone, which can't say which
  element the person edited.

### 6.3 The verdicts

`looks.html`, beside each root's journeys (`web/journeys/looks.html`,
`tui/journeys/looks.html`), is a written file (section 2): the
writer's run alone writes it (section 4, step 5), and the person opens
it from a checkout as a page (`web/journeys.md`, 5.4). A request's
entry is in its root's page: the TUI's for a name starting `tui-`
(`web/look-requests.md`, 2), the browser's otherwise.

- **The page:** `<!doctype html>`, `html` with `lang="en"`, the title
  `Look requests rendered · temper`, no script and no stylesheet; a tag
  a line, as a print (`web/journeys.md`, 5.3); an `h1`, then its
  entries, newest first. A root with no entry has no page.
- **An entry** is a `section` whose start tag carries, in this order,
  `data-look-request` NAME, `data-look-commit` C, `data-look-landed`
  L, the `Look-landed` commit it checks, `data-look-tip` TIP, the print
  it rendered against, and `data-look-result`, `look-render`'s result
  (6.2) or `not checked`. Each value is a name or a whole hash, so none
  needs escaping. Within it, in order:
  - an `h2`, NAME;
  - a paragraph naming C, L, TIP and the result;
  - for each page, a link to the print's page by its path from the
    root, then `identical`, why it was not compared, or a table of its
    differences (width, element, property, the request's value, the
    landed one), and `N more` past them;
  - for `not checked`, why, in place of the pages;
  - `States the request names in words aren't rendered: its landing's
    account covers them.`;
  - under the TUI's root, `These are the browser's renders of the TUI's
    frames, not a terminal's.`
- **Escaped:** every value from `look-render`'s lines, its own escapes
  undone first, is written with `<`, `>`, `"` and `&` escaped, as
  `dom::print` writes text (`web/journeys.md`, 5.3), so that no value a
  page computes becomes markup.
- **Which entry stands:** for a request, the entry whose
  `data-look-landed` is its newest `Look-landed` commit on main, read
  as PRINTED is (section 6). A newer `Look-landed` adds a newer entry
  above the older, which stays. An entry lost is checked again at the
  next run. A `section` start tag missing one of the five attributes,
  or holding a malformed value, is no entry: a reader passes over it
  with a warning, and the writer keeps its lines as they are.
- **Read by** the writer's run (section 4, step 5), from its checkout
  at TIP, and the refresh (section 6), from `origin/main`.

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
  an `uncompared` page, hand-made, is listed as not compared; a
  landing that changes no page still offers its verdict, with
  `Printed:`; a check that can't finish restores the written files,
  exit 1, nothing committed or offered; a value holding `<`, `&`, `"`
  and an escaped tab is written escaped; a TUI request's entry goes
  beside the TUI's journeys; the commit's dates are TIP's, and a new
  offer at the same tip, another commit, gets a new key.
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
  stderr and exit (6.2): equal pages, `identical`; one property
  differing at each width, three `differs` lines; a box 1 px off equal,
  2 px off differing; a touched element with no pair, `pair`; an element
  touched by a rule's selector, its `:hover` dropped; a page whose
  elements don't follow its start tags, a hand-made page and a page
  with nothing printed beside it, each `uncompared`; nothing touched,
  `nothing to render`; a value holding a tab and a `\` escaped; past 50
  differences, `more`; the copies' `details` and `dialog` opened and
  each page opened kept to files; a read refused, exit 3; a tip that
  isn't C, exit 2. No Chromium starts.
- **The kit's reads** of Chromium, in skein's browser suite
  (`skein/docs/design/testing/browser.md`, 9), in the session its
  viewport story starts: written and compiled in the plan, run after
  landing. None is in temper's browser tier.
- **The comparison's decisions,** step tests in the same library, the
  `look-compare` command on canned answers (git's output and exit, the
  print's lines and exit, the files read), then its rows, stderr and
  exit (6.1): every edit landed, exit 0; an edit missing, its hunk a
  row, exit 1; one tag changed by both sides, a conflict row;
  generated ids renumbered by a moved element, no row; a rule landed,
  one differing, one missing, one removed; a request of rules alone,
  rule rows alone and no print; a new walk's page compared whole; a
  stopped walk, a row, exit 1; the TUI's rules listed to read; a
  hand-made directory not compared; a tip that isn't C, and each
  refused argument, exit 2; a print that can't finish, exit 3. Each of
  its effects is a value these steps answer, so its script has no
  witness.
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
