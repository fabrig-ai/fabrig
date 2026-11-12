# Places and copies

Provisional, 2026-11-12, from `../../draft/landed-qa.md`, sections 2, 5
and 7, milestone Q3 of `../../draft/landed-qa.execution.md`. Where a
pass's boxes run, and what each case starts from: a place, a fixed
directory with its own `bin/`; a setup's copy restored there, or made
again from its recipe; and the drive session, held open between the
agent's one-shot commands. The recipes, their walk and the drive
session are Q2's (`../../client/recipe.md`, `../../client/drive.md`);
this document is how a pass uses them.

## 1. In one page

- **A place** (section 2) is a directory `qa.json` names, with its own
  `bin/`, its bundle, the box's directories, its copies and its holder's
  files. One pass runs at a place at a time.
- **A build installed** (section 3) into the place's `bin/` and
  `bundle/` from its manifest, at a pass's start and resume, before any
  box starts, so a copy made there names the place's programs.
- **A setup restored** (section 4) by `temper-qa pass setup NAME`: the
  newest copy of the recipe's chain unpacked and its check walked; a
  copy the build refuses by name made again from the nearest ancestor
  whose copy restores, each recipe's copy saved on the way.
- **The drive session held open** (section 5): `pass open` starts a
  holder that keeps one drive session on the place's box; `pass line`
  gives it a line and prints its answer; `pass close` ends it and prints
  the kept lines.
- **New recipes** (section 6), from a case's lines, proven by walking
  from their parent on the same build; **a recipe that no longer walks**
  (section 7) is the agent's to follow or to file.
- **Its code and tests** (section 8): the archive, the drive tool and
  the processes behind faces, the place's logic tested with fakes.

## 2. A place

- **Its directory,** as `qa.json`'s `places` names it (`README.md`,
  3.2), made by the tool when absent:

  ```
  PLACE/
    bin/          the pass's build's executables (section 3)
    bundle/       the pass's build's loopback bundle (section 3)
    box/          the drive session's scratch directory (../../client/drive.md, 2.2): config/, data/, cache/, session/
    copies/NAME/  the copy of recipe NAME, one file, HASH-COMMIT.tar (section 4)
    setup         the setup last restored: its name and hash, one line (section 4)
    walk.log      the last walk's output (section 4)
    drive.lock    held by the holder for its life (section 5)
    drive.sock    the holder's socket (section 5)
    drive.log     the holder's own output (section 5)
  ```

- **One pass at a time:** the ledger records the pass running at a place
  (`ledger.md`, 2.7). A place is **free** when no `running` pass is
  recorded at it and its `drive.lock` can be taken at once, so a
  holder left by an ended session still keeps it (section 5).
- **Fixed ports:** the box's own, chosen when its sandbox is first
  prepared and retained in its configuration
  (`../../protocol/setup/box.md`, 5), so a copy restored where it was
  made binds the ports it was made with. A copy is restored only at the
  place that made it, since its copies live in that place's directory.
- **The cache stays:** `box/cache/` is kept across copies and builds, as
  `../../client/drive.md`, 7 has it for a newer build's Forgejo; a copy
  holds `config/` and `data/` alone.

## 3. Installing a build

At a pass's start and at its resume (`passes.md`, 3 and 7), once the
place is free and before any box starts there:

- **`bin/`** emptied, then each file of the build's `bin/` copied in,
  its mode kept; **`bundle/`** removed, then the build's `bundle/`
  copied in. Each file is copied to a temporary name in its directory
  and renamed into place.
- **Checked against the manifest** (`builds.md`, 4): each copied file's
  SHA-256 is the manifest's for its path; a file the manifest lacks, or
  a checksum that differs, fails the install, naming the path.
- **`setup` removed,** so no case runs on a box restored for an earlier
  build until a setup is restored again.
- **A failure** fails the loop's step (`README.md`, 4), retried at the
  next tick, with nothing recorded for the pass (`passes.md`, 3).

So every program a box at the place runs is the pass's build's, from
the place's `bin/`, never a kept build's directory
(`../../draft/landed-qa.md`, 5).

## 4. A setup restored

`temper-qa pass setup NAME`, by a pass session's agent (`ledger.md`,
4.9), at its pass's place:

- **Refused while a holder lives** (section 5): `the drive session is
  open at PLACE; close it first: temper-qa pass close`.
- **NAME** is `fresh` or a recipe of the pass's setups,
  `passes/P/setups/` (`passes.md`, 2): `setup S is not in pass P's
  setups; setups: S1, S2, ...`.
- **`fresh`:** the box's `config/` and `data/` removed, `setup` written
  `fresh`; it prints `setup fresh: an empty box`, exit 0. The drive
  session then prepares a new sandbox (`../../client/drive.md`, 2.2).
- **A recipe:** its chain and each member's hash read with the place's
  own `bin/temper-client-journeys`, `recipe list --setups S` for the
  parents and `recipe hash R --setups S` for each R of the chain
  (`../../client/recipe.md`, 5.2), S the pass's setups. A recipe that
  does not read refuses, `recipe NAME: WHY`, exit 1: the pass's to fix.
- **Its copy** is `copies/R/HASH-COMMIT.tar`, HASH the chain's hash of R
  (`../../client/recipe.md`, 4), COMMIT the build that made it. A file
  there with another hash is no copy of R's chain as it stands.
- **The walk,** with the chain R1 to Rk, root first, Rk being NAME:
  1. **The nearest restoring copy:** from Rk back to R1, the first Rj
     with a copy: the box's `config/` and `data/` removed and the copy
     unpacked into `box/`. With none, the box emptied as for `fresh`.
  2. **Each recipe after Rj, one walk each,** the first of them walking
     Rj's check before its steps (`../../client/recipe.md`, 5.1):
     `recipe walk Ri --from Ri-1` for each Ri after Rj, in order; where
     Rj is NAME itself, `recipe walk NAME --from NAME`, its check alone;
     from no copy, `recipe walk R1`, then the same from R1. Each is `bin/temper-client-journeys recipe walk ... --bin bin
     --bundle bundle --scratch box --setups S --strict` at the place
     (`../../client/recipe.md`, 5.1); its output appended to `walk.log`.
  3. **Each walk's exit:**
     - **0:** walked; for each Ri walked past a copy, its copy saved:
       `tar --create` of `box/config` and `box/data` into a temporary
       file in `copies/Ri/`, renamed to `HASH-COMMIT.tar`, and every
       other file in `copies/Ri/` removed. A copy whose check walks is
       kept as it is: one copy per recipe, the newest that restored
       (`../../draft/landed-qa.md`, 5).
     - **3, a store refused by name** (`../../client/drive.md`, 2.5):
       that copy is removed, and step 1 goes on from the next ancestor
       up; on a box with no copy, 3 is as 4.
     - **4, the box failed:** it stops, printing `setup NAME: the box
       failed walking R` and the walk's last 40 lines, exit 4. A box
       that will not start or panics is a finding
       (`../../draft/landed-qa.md`, 7).
     - **5, stopped at a line:** it stops, printing `setup NAME: stopped
       at R:N: WHY`, the walk's last 40 lines and section 7's facts,
       exit 5.
     - **1 or another:** it stops, printing the walk's last 40 lines,
       exit 1.
- **Restored:** `setup` written, `NAME HASH`, the box stopped and ready
  for `pass open`; it prints how, one line: `setup NAME: restored from
  its copy made at COMMIT12`, `setup NAME: made again from R's copy` or
  `setup NAME: made from a fresh box`, then a line per copy saved,
  `copy R saved: HASH12 at COMMIT12`, exit 0.

## 5. The drive session held open

A drive session (`../../client/drive.md`, 2) reads lines on its stdin
and answers each; an agent's shell runs one command at a time. A
holder keeps the session between them.

- **`temper-qa pass open`:** refused without `setup` (`no setup is
  restored at PLACE: temper-qa pass setup NAME first`) or while a holder
  lives. It starts `temper-qa pass hold` in a session and process group
  of its own, its stdin closed and its output to `drive.log`, with the
  command's own environment, then asks the holder for the session's
  start and prints it: `ready`, exit 0; or the session's last lines and
  its status, 3 for a store refused by name, 4 for the box failed, 1
  otherwise. It waits 300 seconds for the start at most, then stops the
  holder, exit 4.
- **The holder, `temper-qa pass hold`:** takes `drive.lock`, exclusive,
  for its life, or exits 1; starts `bin/temper-client-journeys drive
  --bin bin --bundle bundle --scratch box`, with `--scripts F` for each
  `scripts.json` of the setup's chain, root first, from the pass's
  setups, none for `fresh`, and `--strict` (`../../draft/landed-qa.md`,
  6); and listens on `drive.sock`, a Unix stream socket, 0o600, any old
  one removed first, since the lock is its.
  - **One request per connection:** one line, the start's, a drive line,
    or the end; the holder writes a drive line or `end` to the
    session's stdin, reads the session's stdout up to that line's last
    answer line (`ok`, or one starting `failed: `), or to the session's
    end, writes it back with its status, and closes the connection. Its
    wire is its own and the commands' alone.
  - **Its end:** at the end's request, or once its pass session is no
    longer running in the ledger, which it reads every 5 seconds: `end`
    given to the session, which then stops its box
    (`../../client/drive.md`, 2.4); the session's tree killed past 70
    seconds; the socket removed; the lock released by its exit. So a
    session ended by the loop leaves no box running past the next
    minute and a half, and the place stays busy until then (section 2).
- **`temper-qa pass line LINE`:** LINE one argument, a drive line
  (`../../client/drive.md`, 3). It prints the session's answer as the
  session gave it, `| ` lines and the last line, and exits 0 for `ok`, 5
  for `failed: WHY`, 4 once the session has ended with the box's
  failure; with no holder, `no drive session is open at PLACE:
  temper-qa pass open first`, exit 1.
- **`temper-qa pass close`:** the end's request; it waits for the
  session's end and the holder's exit, prints the session's last line
  and then the kept lines, `kept lines:` and each of
  `box/session/lines` on its own line, so a case's steps become a
  recipe's or a finding's reproduction (`../../client/drive.md`, 4); its
  exit the session's, 0 for a clean stop, 4 for the box's failure.
- **Looks** stay in `box/session/looks/` until the next session; a
  finding's evidence copies them (`findings.md`, 2).

## 6. A new recipe

`temper-qa pass recipe NAME FILE`, once a case whose state no setup
reached has run (`../../draft/landed-qa.md`, 5):

- **FILE,** a path or `-`, one JSON object: `after`, the setup the case
  started from, `fresh` or a recipe of the pass's setups; `lines`, the
  steps from there and then the state's check, `web see` or `tui see`
  lines last, drive lines with no look (the case's kept lines, cut where
  the state is reached); `readme`, the paragraph of its `README.md`;
  `scripts`, optional, the object of its `scripts.json` in the box's
  `--scripts` form (`../../protocol/setup/box.md`, 5).
- **Refused while a holder lives;** NAME a recipe name
  (`../../client/recipe.md`, 2) not among the build's setups: a recipe
  the pass itself added may be written again.
- **Written** to `passes/P/setups/NAME/`: `recipe`, `recipe 1`, then
  `after PARENT` unless `fresh`, then the lines; `README.md`; and
  `scripts.json` where given. A recipe that does not read
  (`recipe hash`) is removed, its refusal printed, exit 1.
- **Proven:** walked from its parent on the same build, as section 4
  restores NAME: its parent's copy restored, or made again, then `recipe
  walk NAME --from PARENT`, or from a fresh box for `fresh`. Walked, its
  copy is saved and it prints `recipe NAME walked from PARENT: HASH12`,
  exit 0; else its directory is removed and the walk's stop printed,
  with section 4's exit.
- **Then in the catalogue:** the pass's later cases and findings may
  name it, and it goes with the pass's next request (`findings.md`, 4).

## 7. A recipe that no longer walks

Exit 5 from `pass setup` (section 4) or `pass recipe` (section 6):

- **The facts printed with it:** the recipe and line that failed and
  what the session answered; the copy the walk started from and the
  build that made it, the last build the recipe walked on; and the
  landings since, those the pass's build holds and that build's record
  does not (`ledger.md`, 2.4), each its number, goal and title. With no
  copy of that recipe, no build is named and every landing the pass's
  build holds is listed.
- **The agent judges** them against those landings' drafts
  (`../../draft/landed-qa.md`, 5):
  - **a flow a landing changed on purpose:** the recipe's to follow; the
    agent edits `passes/P/setups/R/recipe`, as a journey's setup is
    edited (`../../client/web/journeys.md`, 8), and runs `pass setup`
    again, which walks it from its nearest ancestor's copy, since its
    hash changed. The edit goes with the pass's next request;
  - **a flow no landing meant to change:** a finding against temper
    (`findings.md`, 2), naming the step, what it showed and the last
    build the recipe walked on; the cases on that setup are failed with
    it, and the others go on (`../../draft/landed-qa.md`, 7).

## 8. Its code and tests

- **In `temper-qa-tool`,** a `place` module for sections 2 to 4 and 6
  and a `hold` module for section 5, beside the ledger's.
- **Its world behind faces:** the drive tool, run as a process from the
  place's `bin/`; the archive, `tar --create` and `tar --extract` run as
  processes with the captured environment; the holder's socket and its
  process. The QA tool links none of the drive tool's code: the place's
  build reads its own recipes and speaks its own lines.
- **Focused tests,** with no box and no browser: a fake
  `temper-client-journeys` in a test place's `bin/`, a small script that
  answers `recipe list`, `recipe hash`, `recipe walk` and `drive` as the
  test tells it; an archive face that copies directories:
  - a copy restored and its check walked; a copy refused by name made
    again from its parent's copy, each copy saved on the way, and the
    refused one removed; a chain with no copy walked from a fresh box;
  - a walk stopped at a line exiting 5 with the landings since the
    copy's build;
  - the install's checksum refused for a file the manifest disagrees
    with;
  - a holder giving lines and answers through the socket, `pass line`'s
    exits, `pass close`'s kept lines, and the holder ending its session
    once the ledger ends the pass session.

## 9. Open questions

- **More places** (`../../draft/landed-qa.md`, 12): two boxes preparing
  at once may choose the same ports. Default: one place until passes
  queue, then ports given each place in `qa.json`, through the box's
  configuration.
- **A copy's archive compressed:** a plain tar. Default: plain, until a
  copy's size tells against it.
