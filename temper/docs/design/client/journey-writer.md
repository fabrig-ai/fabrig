# The journeys' writer

Provisional, 2026-11-18; the run's options and lines, 2026-11-19, from
the after-landing owner who runs it. Who writes main's journey files, and when:
`../draft/journey-walks.md`, sections 6, 7 and 8.3, its first step. One
writer, run after each landing, walks again on a fake box each journey
whose recording no longer replays, reprints every journey, and offers
what differs as a change of its own; nothing else writes those files.
The journeys are the browser's (`web/journeys.md`) and the TUI's
(`tui/journeys.md`); the look requests drawn on their pages are
`web/look-requests.md`'s.

## 1. In one page

- **Main's journey files have one writer** (section 2): each
  `recording`, each page and the files printed beside them. A journey's
  README, setup and walk stay sources, which any change edits.
- **A command prints main's tip** (section 3): `temper-client-journeys
  pages` walks again each journey whose recording no longer replays, and
  no other, then reprints every journey in place. A moved journey, one
  whose `walk` is of version 2, is `temper-client-walk pages`' instead,
  printed in a world (`journey-walks.md`, 5.2).
- **A script runs it one at a time** (section 4): at each landing of
  main, `scripts/journey-pages.sh` prints the tip and offers what
  differs as one change, opened from a pushed branch, its message ending
  `Printed: COMMIT`, or, given no project, as a commit on a branch of
  its own for its runner to land. While an offer of its own is open it
  starts no other, and one run at a time holds the checkout. Once it
  has run on a tip and found nothing to write, main's pages are that
  tip's print.
- **Goals are kept off** (section 5): the delivery check refuses a
  delivery that changes a written file beside any other file, and a
  nonempty overlay; the merge job settles a written file's conflict with
  the landing branch's copy; no test at the gate holds a journey to its
  recording.
- **A request is checked against the print** (section 6):
  `look-refresh.sh` drops a request once the last `Printed:` covers its
  landing, and whoever lands one compares by a reprint in its own tree,
  committing no journey file.
- **After a landing, never inside one:** a walk that stops holds no
  landing. It fails the writer's run, an after-landing red of its
  runner's (the user, 2026-11-18).

## 2. The written files

Under `web/journeys/` and `tui/journeys/`:

| Written by the writer alone | Sources, any change's |
|---|---|
| a journey's `recording` | a journey's `README.md` |
| its pages: `NN.html`, and the TUI's `NN.txt`, NN two digits or more | a browser journey's `setup`, or its `walk` once it has moved; a TUI journey's `walk` and `setup` |
| beside the browser's journeys, `skein-ui.css` and `journey.js`; beside the TUI's, `theme.css` and `tui-journey.js` | each root's `overlay.css`, empty on main; the `README.md` beside each root |

- **A source changes with what a person does:** a change that renames a
  control a `setup` or a `walk` names edits that line in the same
  change, as it edits a test (`../draft/journey-walks.md`, 3.2). The
  writer's next run walks it.
- **A new journey** lands as its sources. Its recording and pages are
  the writer's runner's to make, walked by hand and added, or with
  `again`, in a change of written files alone
  (`../draft/journey-walks.md`, 8.3).

## 3. The command

`cargo run -p temper-client-journeys -- pages [--prepared DIR]
[--within DURATION]`, an internal command of
`crates/temper-client-journeys-tool` (`web/journeys.md`, 5.1), on the
checkout it is built from. A moved journey, a directory under either
root whose `walk` is of version 2 (`journey-walks.md`, 3.1, `moved`),
is none of its: no item below reads, walks, prints or removes a file of
it, nor does the bare reprint, `again` or `add`, which refuse its name,
naming `temper-client-walk`. Its pages are `temper-client-walk pages`'
(`journey-walks.md`, 5.2), which removes its `recording`.

1. **What is stale:** every journey of both clients not moved, a directory under
   either root that holds a `recording`, the browser's then the TUI's,
   each in name order, replayed through the current build as the
   reprint replays it (`web/journeys.md`, 4.1; `tui/journeys.md`, 4.1).
   A replay that meets a stale turn marks its journey for a walk. No
   process starts for this.
2. **Walked again:** each marked journey, and no other, as `again NAME`
   walks it (`web/journeys.md`, 8.1; `tui/journeys.md`, 8.2), on a
   fresh fake box, its inputs `--prepared DIR`'s or built from the
   checkout once for the run. Each walk has a fresh scratch directory,
   under DIR or the platform's temporary directory as `again`'s is: it
   goes when the walk ends in a recording, and is kept when the walk
   stops. A walk that stops leaves its journey's files as they were,
   and the run goes on.
3. **Forgejo from DIR:** `DIR/cache`, where it is a directory, is each
   walk's box's cache (`web/journeys.md`, 8.1), so a Forgejo and runner
   it holds where `temper-box forge fetch` puts them, `forgejo/VERSION/`
   and `forgejo-runner/RUNNER_VERSION/` (`../protocol/configuration/directories.md`,
   the cache row), are taken from it, and the box fetches only what it
   lacks (`../protocol/configuration/directories.md`, 1: a disconnected
   walk uses a populated cache). This is the box's way today; the
   command adds nothing for it.
4. **The bound:** `--within DURATION`, digits then `s`, `m` or `h`
   (`90m`), bounds the walks from the command's start. A walk it cuts,
   or one not started before it passes, is a stopped walk, its WHY
   naming the bound: `cut at --within 90m`, `not walked, past --within
   90m`. The reprint after the walks is not bounded.
5. **Reprinted:** then every journey of both clients, in place, with the
   files beside them, as the bare command reprints (`web/journeys.md`,
   5.1). A journey still stale keeps its pages.
6. **Pages with no recording go:** a directory under either root that
   holds pages, no `recording` and no `walk` of version 2 loses its
   pages, and keeps its other files.
7. **Its lines,** on stdout, one a journey, in item 1's order: `fresh
   NAME`, current and printed; `walked again NAME`, a walk that ended in
   a recording and printed; `stopped NAME: WHY`, a journey left
   unprinted, WHY `again`'s refusal with its step or line, the bound, or
   `still stale after its walk`, then `; scratch kept at PATH` where its
   walk left one. NAME is the journey's directory name; the two roots
   hold no name twice. The reprint's own lines, and every other word,
   go to stderr.
8. **Its exit:** 0, every journey printed; 1, a journey left unprinted,
   once the others are printed; 2, usage; 3, the print could not
   finish, a root unreadable or unwritable, the roots as it left them.
9. **What it writes:** section 2's written files under the two roots,
   and nothing else; a walk's scratch is its own (`web/journeys.md`,
   8.1).

`pages --check`, with no other argument, is item 1 alone: it prints
`fresh NAME` or `stale NAME` for each journey, in item 1's order, writes
nothing and starts no child; exit 0, every journey fresh; 1, one stale
or more; 2, usage; 3, a root unreadable.

On a current tree it starts nothing and changes no byte. With a stale
journey it starts a box, a browser and, for the TUI, a terminal: it is
the writer's command, never a plan's. A plan reprints with the bare
command alone.

## 4. The run

`scripts/journey-pages.sh [--project PROJECT (--budget AMOUNT |
--as-allowed)] [--branch NAME] [--push REMOTE] [--prepared DIR]
[--within DURATION]`, under `temper/scripts/`, run by the
after-landing check's owner at each landing of main
(`../draft/journey-walks.md`, 6), in a checkout of its own whose
`origin` is the repository. With `--project` it opens a change, and
needs `temper` on its `PATH` signed in; without it, it opens nothing,
and needs no ctl directory and no way to push.

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
  journey (section 3, 7), then one last line: `main's pages are TIP's
  print`, `wrote NAME COMMIT`, or a waiting line, `waiting on change N`
  or `waiting on COMMIT`. A refusal prints nothing there; every other
  word goes to stderr.

1. **Alone:** a lock, the directory `journey-pages.lock` in the
   checkout's git directory holding the run's process number, made
   with `mkdir`. A second run while a live process holds it is refused,
   naming the lock and the process, exit 2; a lock whose process has
   gone is taken over. The run removes it as it exits.
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
   on. Whoever drops an offer by hand removes the file.
3. **On main's tip:** a checkout with tracked changes, or a rebase, a
   merge or a cherry-pick in progress, is refused. The script fetches
   `main` from `origin` and checks out NAME at `origin/main`, TIP.
4. **Prints:** first, where a directory under either root holds a
   `walk` whose first line is `walk 2`, `cargo run --locked -p
   temper-client-walk -- pages` (`journey-walks.md`, 5.2), unbounded;
   then `cargo run --locked -p temper-client-journeys -- pages`,
   with `--prepared DIR` where given, `--within` the bound's rest where
   one is given; each with `TEMPER_INTERNAL_BUNDLE=none` (`AGENTS.md`,
   5). It passes each print's stdout on as it comes, as one report:
   step 7's paragraph takes the `stopped` lines of both.
5. **Nothing differs:** its last line is `main's pages are TIP's
   print`. Either print exiting 2 or 3 ends the run, nothing written.
6. **Written files alone:** a changed path that is not section 2's
   written file ends the run, naming it, nothing committed.
7. **Its commit,** on TIP, of the written files: the subject
   `Journey pages printed at TIP12`, TIP's first twelve hexadecimal
   digits; a paragraph naming the journeys walked again and those
   stopped, each with why, where there are any; and a last paragraph of
   one line, `Printed: TIP`, TIP's whole hash.
8. **Pushed** as above, where it pushes.
9. **Opened,** with `--project`: `temper ctl change open --project
   PROJECT --branch NAME --at COMMIT --title SUBJECT --body BODY` with
   the budget given and `--key journey-pages-TIP`, so that a run again
   on the same tip after a lost answer opens nothing twice.
10. **Kept:** `task N` from the answer, or with no `--project`
    `commit COMMIT`, goes to step 2's file, and the last line is `wrote
    NAME COMMIT`.
11. **Its exit:** 0, nothing to write, a commit written, or waiting; 1,
    a journey left unprinted (the others written and offered), git,
    cargo or `temper ctl` failed, a push refused, or a path outside the
    written files changed; 2, its arguments, its checkout or its lock
    refused.

- **The bound:** `--within DURATION`, as the command takes it, is the
  whole run's: the script gives the print what is left of it once it
  reaches step 4, and a walk the bound cuts is a stopped walk.

- **It settles:** its change lands as any opened change does, and that
  landing, as any other, runs the script again on the new tip, which
  finds nothing to write. Without `--project`, whoever runs it lands the
  commit, as it is, by a merge or squashed, and step 2 waits for that.
- **How far behind:** between a landing and the writer's change, main's
  pages are an earlier tip's, and the last `Printed:` on main says
  which. Nothing reads them in between but a person.
- **A walk that stops** stops again on every later tip, until a change
  repairs the walk or the client: one after-landing red, its runner's.
  The other journeys are printed and offered all the same.
- **Its change holds written files alone,** so it meets no conflict a
  goal made; the rules covering main ask of it what they ask of any
  change.

## 5. Goals kept off

- **The delivery check** (`tools/pre-pr/design.md`, 4, check 8): a
  delivery whose own changes hold a written file and any other path is
  refused, naming the reprint to compare with, and for a moved journey
  `cargo run -p temper-client-walk -- print --out DIR`, and how to
  restore the files; so is a tree whose `overlay.css`, under either
  root, holds a byte. A delivery of written files alone passes it.
- **The merge job** (`../protocol/tooling.md`, 4.1;
  `tools/merge/README.md`): a conflict at a written file's path is
  settled with the landing branch's copy, the commit merged in, with no
  model: its file where that commit holds the path, none where it does
  not. A deletion or a rename there asks no judgement, since only the
  writer writes the path, so it is settled too, unlike any other file's
  (`../domain/forge/change/repairs.md`). The writer's next run writes it
  again.
- **No test** replays, reprints or compares a recording or a page of
  main's, and a recording gone stale fails none (section 7).

## 6. A request against the print

- **Drawn on main's pages as they stand** (`web/look-requests.md`, 2):
  the writer's last print, never a walk of the person's.
- **The refresh goes by the print** (`web/journeys.md`, 7):
  `look-refresh.sh` reads PRINTED, the value of the newest `Printed:`
  trailer among every commit reachable from `origin/main`, not its
  first-parent line alone: a print reaches main as its own commit, as a
  merge's second parent made by hand, or squashed into another commit.
  It drops a request once the commit carrying
  its `Look-request` trailer is PRINTED or an ancestor of it. A request
  whose trailer is on main and not yet printed stays, on the pages main
  still holds. Its report opens with `pages printed at PRINTED12, N
  landings since`, N main's first-parent commits after the one that
  holds that `Printed:`, or `pages not printed yet` where main holds
  none; then, for each request, `kept NAME`, `kept NAME, done on main
  by C12, not yet printed`, or `dropped NAME, done on main by C12,
  printed at PRINTED12`.
- **Whoever lands a request** reprints in its own tree, `cargo run -p
  temper-client-journeys`, compares each page the request changes with
  the request's commit's, equal where their facts agree, and restores
  the journeys' files to its base's: it commits no written file. A
  journey the reprint finds stale is the writer's to walk after the
  landing; its comparison is the render check's
  (`web/look-requests.md`, 3). A moved journey is compared by a print
  into scratch, `cargo run -p temper-client-walk -- print --out DIR
  NAME`, DIR's pages against the request's (`journey-walks.md`, 6),
  which is never stale.
- **The render check follows the writer's change** that prints the
  landing (`web/look-requests.md`, 4).

## 7. Its tests

Focused and native: no box, browser or terminal starts.

- **The gate's:** the tests that reprinted main's journeys and held
  each recording to its normalisation are gone, with
  `tools/tiers/budgets.toml`'s lines that named a committed recording.
  Redaction, the last scan and the proof keep their cases on walks the
  tests record in the host's world (`web/journeys.md`, 6;
  `tui/journeys.md`, 7). Each overlay's emptiness and each journey's
  README stay held, as sources.
- **The command,** in `crates/temper-client-journeys-tool`'s tests, on
  roots in scratch holding walks recorded in the host's world: on
  current journeys, `pages` changes no byte and asks for no walk; with
  one recording made stale by hand, the walks it asks are that
  journey's alone; a directory with pages and no recording loses its
  pages; its lines are one a journey; `--check` names each journey
  fresh or stale and writes nothing; a bound already past stops each
  walk asked, naming it. The walk is `again`'s, compiled and never run
  in a test. A moved journey, with a `recording` left and with pages
  alone: `pages`, `--check` and the bare reprint neither name it nor
  touch a byte of it, and `again` and `add` refuse it.
- **The run,** in the same crate's suite beside the refresh's tests, in
  scratch repositories with a fake `cargo` and a fake `temper` first on
  `PATH`: nothing to write, nothing pushed or opened; pages changed, one
  commit of them alone whose last paragraph is `Printed: TIP`, pushed,
  and one `change open` asked with its branch, commit and key; a second
  run while that task is live starts no print, push or open; once it
  has ended, the next run prints; with no project, a commit written on
  its branch, pushed only where asked and never forced, a branch held
  elsewhere refused, and the next run waiting until main holds the
  print, merged or squashed; its last line in each case; a held lock
  refused, exit 2; a path outside the written files changed, nothing
  pushed or opened, exit 1; a dirty checkout refused, exit 2. With a
  `walk` of version 2 under a root, `temper-client-walk pages` asked
  first, then the journeys tool, a `stopped` line of either in the
  commit's paragraph, its removed `recording` committed, and either
  exiting 2 or 3 writing nothing; with none, the walk's print never
  asked.
- **The refresh:** a request whose trailer is on main and not yet
  printed kept; dropped once a `Printed:` names a commit that contains
  it, the print reaching main as its own commit, a merge's second
  parent or squashed; nothing dropped while main holds no `Printed:`;
  the report's first line.
- **The delivery check** (`tools/pre-pr/design.md`, 9.2) and **the
  merge job** (`tools/merge/README.md`), in their own tests.

## 8. Later

- **temper runs the writer** (`../draft/journey-walks.md`, 11): the
  script then goes.
- **One engine:** `again`, the reprint and section 3's command go with
  the last journey that moves, and the run prints with
  `temper-client-walk pages` alone (`journey-walks.md`, 8).
