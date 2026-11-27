# Places and copies

Provisional, 2026-11-12, revised 2026-11-27, from
`../../draft/archive/landed-qa.md`, sections 2, 5 and 7, milestones Q3, Q4,
Q6 and Q7 of `../../draft/archive/landed-qa.execution.md`. Where a pass's boxes and
deployments run, and what each case starts from: a place, a fixed
directory with its own `bin/`, of one kind; a setup's copy restored
there, or made again from its recipe; and the drive session, held open
between the agent's one-shot commands. The recipes, their walk and the
drive session are Q2's (`../../client/recipe.md`,
`../../client/drive.md`), a deployment's processes Q6's
(`deployment.md`); this document is how a pass uses them.

## 1. In one page

- **A place** (section 2) is a directory `qa.json` names, of a box's
  kind or a deployment's, with its own `bin/`, its bundle, its scratch
  directory, its copies and its holder's files. One pass runs at a place
  at a time, on setups of the place's kind.
- **A build installed** (section 3) into the place's `bin/` and
  `bundle/` from its manifest, at a pass's start and resume, before any
  box starts, so a copy made there names the place's programs.
- **A setup restored** (section 4) by `temper-qa pass setup NAME`: the
  newest copy of the recipe's chain unpacked and its check walked; a
  copy the build refuses, its store by name or its box at its start,
  made again from the nearest ancestor whose copy restores, each
  recipe's copy saved on the way. A copy
  leaves out what a start pins from the place's `bin/`, saved and
  unpacked alike.
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
    bundle/       the pass's build's web bundle (section 3)
    box/          a box's place: the drive session's scratch directory (../../client/drive.md, 2.2): config/, data/, cache/, session/
    deployment/   a deployment's place, in box/'s stead: its scratch directory (deployment.md, 3): config/, data/, cache/, session/
    copies/NAME/  the copy of recipe NAME, one file, HASH-COMMIT.tar (section 4)
    manifest      the completed install's kept manifest (section 3)
    setup         the setup last restored: its name and hash, one line (section 4)
    walk.log      the last walk's output (section 4)
    drive.lock    held by the holder for its life (section 5)
    drive.sock    the holder's socket (section 5)
    drive.log     the holder's own output (section 5)
  ```

- **Its kind,** `qa.json`'s `kind` for it (`README.md`, 3.2), a box's
  when absent (`../../draft/archive/landed-qa.md`, 2): a box's place runs
  `temper-box` on its fake backends, as Q3 to Q5 have it; a
  deployment's runs the deployment's processes (`deployment.md`), needs
  its `ports`, and holds `deployment/` where a box's holds `box/`. Its
  scratch directory below is the one its kind holds. A place runs
  setups of its kind alone: a recipe's kind is its chain's root's
  (`../../client/recipe.md`, 2), and `fresh` is a box's. What differs
  between the kinds, here and in the drive session, is `kinds.md`, 3's
  table, each value read from the kind (`kinds.md`, 6).
- **Private, whatever the umask** the tool runs under
  (`../../draft/archive/landed-qa.md`, 5): every directory the tool makes at a
  place, `PLACE/` itself, its scratch directory, `copies/` and each
  `copies/NAME/`, and `bin/` and `bundle/` with what they hold, is
  0o700, and every file it writes there, `manifest`, `setup`, `walk.log`,
  `drive.log`, `drive.lock`, a copy and the temporary files before
  their renames, 0o600, but the build's files installed in `bin/` and
  `bundle/`, which keep their build's modes (section 3) under their
  0o700 directories. Each mode is set once the directory or file is
  made, never asked of the umask, which can only take bits away; and
  wherever the tool reaches a place, it sets `PLACE/`, its scratch
  directory and `copies/` to 0o700 when they are not already, since a
  place is the tool's own. So the drive session's check that its
  scratch directory is the user's own and open to no one
  (`../../client/drive.md`, 2.1) holds at every place.
- **One pass at a time:** the ledger records the pass running at a place
  (`ledger.md`, 2.7). A place is **free** when no `running` pass is
  recorded at it and its `drive.lock` can be taken at once, so a
  holder left by an ended session still keeps it (section 5).
- **Fixed ports:** the place's own, `qa.json`'s `ports` for it
  (`README.md`, 3.2): every walk and drive session at the place is
  given them as `--ports` (`../../client/drive.md`, 2.1), a new sandbox
  binds them, and the box retains them in its configuration
  (`../../protocol/setup/box-backends.md`, 1), so a copy made at the place binds
  the place's ports, which no other place's box takes. A place with no
  `ports` takes the kernel's when its sandbox is first prepared, as one
  place alone may: two such places can hand a stopped copy's ports to
  each other's new box, which then refuses the copy's restart, naming
  the port. A place's `ports` changed leaves its copies on the old ones:
  the deployment empties its `copies/` with the change. A copy is
  restored only at the place that made it, since its copies live in
  that place's directory.
- **The cache stays:** `box/cache/` is kept across copies and builds; a
  copy holds `config/` and `data/` alone, those of them its box made (a
  box with fake backends makes no `config/`), the fake forges' state among
  them (`../../protocol/setup/box-backends.md`, 1), but for what its processes
  pin at their start (`places.md`, section 4).
- **No Forgejo and no runner:** every setup's forge is the box's fake
  Forgejo (`setups.md`, 2), so no box at a place sets up its own
  Forgejo, and the place's cache holds none of its binaries; a case
  that would set one up is blocked, naming it (`passes.md`, 4;
  `../../draft/archive/landed-qa.md`, 6 and 12).

## 3. Installing a build

At a pass's start and at its resume (`passes.md`, 3 and 7), once the
place is free and before any box starts there:

- **Already installed:** if the place's `manifest` equals the kept build's
  manifest byte for byte, including its commit, nothing is installed and
  `setup` stays. Only a completed install writes this private receipt;
  a home without one installs normally.
- **`bin/` and `bundle/`** removed, then each directory made private
  (section 2); each regular file of `bin/` hard-linked from the kept
  build's, with its build's mode, and each of `bundle/` copied, since the
  box serves a bundle's files as public documents and takes them with one
  link alone. The builds directory and place normally share a
  filesystem. Every installed file must be named by the manifest, and
  every manifest path in those directories must be installed. The
  manifest was checked at keep (`builds.md`, 3 and 4), so the linked
  binaries are not hashed again; the copies are checked as below.
- **Copies, and across filesystems:** a bundle's file, and a binary
  whose link is refused with a cross-device error, is copied to a private
  temporary file in its directory, its build's mode set and then renamed
  into place. Once the files are in place, the tool checks the installed
  `bin/` and `bundle/` lines of the manifest from the place's directory,
  itself, as `sha256sum --check` would: each line read by the manifest's
  one reader (`builds.md`, 4), its file hashed in Rust, streamed, and
  compared, every line checked, with no checksum program run. No other
  link error falls back to copying. A failed check names each file that
  failed, in path order, as one refusal, `checksum check: P1: FAILED;
  P2: WHY`: `PATH: FAILED` for a digest that differs, `PATH: WHY` for a
  file that does not read. It fails the install and leaves no completed
  receipt, so the next tick installs again.
- **Completed:** `setup` removed, so no case runs on a box restored for
  an earlier build until a setup is restored again; then the manifest
  copied to a private temporary file and renamed to the place's
  `manifest`. An earlier receipt is removed before changing any files.
- **A failure** fails the loop's step (`README.md`, 4), retried at the
  next tick, with nothing recorded for the pass (`passes.md`, 3).

So every program a box at the place runs is the pass's build's, from
the place's `bin/`, never a kept build's directory
(`../../draft/archive/landed-qa.md`, 5). Its files stay private under the place's
0o700 directories. Removing a released kept build (`builds.md`, 5)
unlinks its directory's files; installed links remain until the place
installs another build.

## 4. A setup restored

`temper-qa pass setup NAME`, by a pass session's agent (`ledger.md`,
4.9), at its pass's place:

- **Refused while a holder lives** (section 5): `the drive session is
  open at PLACE; close it first: temper-qa pass close`.
- **NAME** is `fresh` or a recipe of the pass's setups,
  `passes/P/setups/` (`passes.md`, 2): `setup S is not in pass P's
  setups; setups: S1, S2, ...`. It is of the place's kind, its chain's
  root holding `config/` at a deployment's place and none at a box's:
  `setup S is a deployment's; PLACE is a box's place`, or `setup S is a
  box's; PLACE is a deployment's place`; `fresh` at a deployment's
  place, `a deployment's place has no fresh setup: its setups start
  from their root's config/`, exit 1.
- **The pass's surfaces:** when neither `web` nor `tui` is configured,
  a setup whose chain holds a web or tui line is refused before any
  restore or walk, naming the recipe, line and surface: the pass's
  surfaces take ctl and forge lines alone. The brief lists the same
  walkable chains as triage (`passes.md`, 4; `triage.md`, 3).
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
- **What a start pins, left out:** a worker writes its agent's program,
  a copy of the place's `bin/` file its agent document names, and that
  document, into its root's `agent/` as it starts, the previous ones
  removed first (`../../protocol/binaries.md`, 5;
  `../../domain/worker/agent-processes.md`, 1). That directory is the build's,
  not the setup's state, so a copy leaves it out: a copy's save never
  archives it, and a restore leaves none of it at the place from any
  copy that holds it, removing it once unpacked and before anything
  else reads the scratch directory (`places.md`, section 8). A copy holds the
  setup's state alone, and every box restored at
  the place pins its agent from the place's `bin/` as it starts, the
  pass's build's. The directory, by its path below the scratch
  directory, matched whole and only there, so that a directory of the
  same name elsewhere, in a checkout or a fake forge's state, is kept:
  - **a box's place:** its fake backends' worker's,
    `data/box/fake-backends/data/workspaces/agent`, the box's isolated
    sandbox under `DATA/box/fake-backends`
    (`../../protocol/configuration/directories.md`, 1;
    `../../protocol/setup/box-backends.md`, 1), whose worker's root is
    `workspaces` under its data directory;
  - **a deployment's place:** its worker's,
    `data/worker/workspaces/agent` (`deployment.md`, 2 and 3).

  Absent, as when a walk stopped before its worker started, nothing is
  left out and nothing fails. A box whose worker's root lies elsewhere
  only makes copies that carry its program, which its next start pins
  again.
- **The walk,** with the chain R1 to Rk, root first, Rk being NAME:
  1. **The nearest restoring copy:** from Rk back to R1, the first Rj
     with a copy: the box's `config/` and `data/` removed and the copy
     unpacked into `box/`, but for what a start pins (above). With none,
     the box emptied as for `fresh`.
  2. **Each recipe after Rj, one walk each,** the first of them walking
     Rj's check before its steps (`../../client/recipe.md`, 5.1):
     `recipe walk Ri --from Ri-1` for each Ri after Rj, in order; where
     Rj is NAME itself, `recipe walk NAME --from NAME`, its check alone;
     from no copy, `recipe walk R1`, then the same from R1. Each is `bin/temper-client-journeys recipe walk ... --bin bin
     --bundle bundle --scratch box --setups S --strict` at the place,
     with `--ports P` where the place has ports, P its nine joined by
     commas (`../../client/recipe.md`, 5.1); its output appended to
     `walk.log`. At a deployment's place, `--scratch deployment`, with
     its ports and `--bundle bundle` as at a box's: the walk starts the
     deployment from the chain's root's `config/` (`deployment.md`, 4),
     its engine serving the place's bundle (`deployment.md`, 2), in a
     ctl session or a full one by the chain's lines
     (`../../client/recipe.md`, 5.1).
  3. **Each walk's exit:**
     - **0:** walked; for each Ri walked past a copy, its copy saved:
       `tar --create` of the scratch directory's `config` and `data`
       (`box/` or `deployment/`), but for what a start pins (above),
       into a temporary
       file in `copies/Ri/`, renamed to `HASH-COMMIT.tar`, and every
       other file in `copies/Ri/` removed. A copy whose check walks is
       kept as it is: one copy per recipe, the newest that restored
       (`../../draft/archive/landed-qa.md`, 5).
     - **3, a store refused by name** (`../../client/drive.md`, 2.5):
       the copy the walk started from is removed, printing `copy R
       removed: ` and the walk's `refused: ` line, and step 1 goes on
       from the next ancestor up; on a box with no copy, 3 is as 4.
     - **4 at a copy's start:** the round's first walk, from Rj's copy
       unpacked in step 1, ended 4 having answered no line: the first
       line it appended to `walk.log` is its failure line, `box failed:
       ` at a box's place and `deployment failed: ` at a deployment's
       (`kinds.md`, 3). The newer build refused, as its box started,
       what the older copy holds by name: a fake-backends sandbox
       without the `temper.toml` it now requires, a configuration
       member that has moved. As for 3, that copy is removed, printing
       `copy Rj removed: ` and that failure line, and step 1 goes on
       from the next ancestor up.
     - **4, the box failed** otherwise, from a fresh box, at a later
       walk of the round or once the walk answered a line: it stops,
       printing `setup NAME: the box failed walking R` and the walk's
       last 40 lines, exit 4. A box that will not start from a fresh
       box, or panics, is a finding (`../../draft/archive/landed-qa.md`,
       7).
     - **5, stopped at a line:** it stops, printing `setup NAME: stopped
       at R:N: WHY`, the walk's last 40 lines and section 7's facts,
       exit 5.
     - **1 or another:** it stops, printing the walk's last 40 lines,
       exit 1.

     Why 4 at a copy's start is the first walk alone, with no line
     answered: a later walk's box is the one this build walked and
     stopped, so its failure is the build's own; and the walk prints
     nothing before its session's first answer
     (`../../client/recipe.md`, 5.1), which comes only once the box is
     ready, so a walk that answered nothing failed at its start, or at
     once at its first line, on what the copy holds. The QA tool reads
     only what any place's build prints: a ready mark added to the
     walk's output, which an older build would not print, and the
     start's words (`box exited N before ready`, a link past its bound,
     a sign-in refused), which differ by cause, were turned down. A
     start that fails for a reason no copy holds, a port in use,
     removes each copy up the chain before the fresh walk stops as a
     finding; the next setup makes them again.
- **Restored:** `setup` written, `NAME HASH`, the box stopped and ready
  for `pass open`; it prints how, one line: `setup NAME: restored from
  its copy made at COMMIT12`, `setup NAME: made again from R's copy` or
  `setup NAME: made from a fresh box`, then a line per copy saved,
  `copy R saved: HASH12 at COMMIT12`, exit 0.

## 5. The drive session held open

A drive session (`../../client/drive.md`, 2) reads lines on its stdin
and answers each; an agent's shell runs one command at a time. A
holder keeps the session between them.

- **`temper-qa pass open [--ctl]`:** refused without `setup` (`no setup is
  restored at PLACE: temper-qa pass setup NAME first`) or while a holder
  lives. `--ctl` opens a ctl session (`../../client/drive.md`, 2.6), no
  browser started, for a pass's ctl cases, which run first
  (`passes.md`, 4), at a place of either kind; without it the session
  is a full one, its browser and TUI on the box's origin or the
  deployment's engine's (`deployment.md`, 5). It starts `temper-qa pass hold` (with `--ctl` where
  given) in a session and process group
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
  setups, none for `fresh`, `--strict` (`../../draft/archive/landed-qa.md`,
  6), `--files` the pass's `files/` (`passes.md`, 2), which its `forge
  commit` lines read (`../../client/drive.md`, 3.5), `--ports P`
  where the place has ports, as a walk is given
  them (section 4), and `--ctl` where `pass open` was given it. At a
  deployment's place it is `bin/temper-client-journeys drive
  --deployment S/ROOT/config --bin bin --bundle bundle --scratch
  deployment`, ROOT the setup's chain's root in the pass's setups S,
  with the same scripts, `--strict`, `--files`, `--ports` and `--ctl`
  where `pass open` was given it: the restored deployment is held, so
  its documents are its copy's (`deployment.md`, 4), and its engine
  serves the place's `bundle/`. It listens on `drive.sock`, a Unix stream socket,
  0o600, any old one removed first, since the lock is its.
  - **One request per connection:** one line, the start's, a drive line,
    or the end; the holder writes a drive line or `end` to the
    session's stdin, reads the session's stdout up to that line's last
    answer line (`ok`, or one starting `failed: `), or to the session's
    end, writes it back with its status, and closes the connection. Its
    wire is its own and the commands' alone.
  - **Its end:** at the end's request, which the loop sends through
    `drive.sock` as soon as its pass session's end is saved
    (`README.md`, 4), or once its pass session is no longer running in
    the ledger, which it reads every 60 seconds as a fallback: `end`
    given to the session, which then stops its box
    (`../../client/drive.md`, 2.4); the session's tree killed past 70
    seconds; the socket removed; the lock released by its exit. The
    loop waits for that release off its own thread and wakes at once
    to reuse the place (section 2). A normal stop adds no ledger-watch
    delay; if the socket request fails, the fallback still stops the
    session, with its 70-second bound, after the next ledger read.
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
- **Looks** stay in the scratch directory's `session/looks/`,
  `box/session/looks/` or `deployment/session/looks/`, until the next
  session; a finding's evidence copies them (`findings.md`, 2).

## 6. A new recipe

`temper-qa pass recipe NAME FILE`, once a case whose state no setup
reached has run (`../../draft/archive/landed-qa.md`, 5):

- **FILE,** a path or `-`, one JSON object: `after`, the setup the case
  started from, `fresh` or a recipe of the pass's setups, of the place's
  kind (section 4); `lines`, the
  steps from there and then the state's check, `web see` or `tui see`
  lines last, or a `check` line and the ctl lines after it
  (`../../client/recipe.md`, 3.1), drive lines with no look (the case's
  kept lines, cut where the state is reached); `readme`, the paragraph
  of its `README.md`;
  `scripts`, optional, the object of its `scripts.json` in the box's
  `--scripts` form (`../../protocol/setup/box-backends.md`, 1).
- **Refused while a holder lives;** NAME a recipe name
  (`../../client/recipe.md`, 2) not among the build's setups: a recipe
  the pass itself added may be written again.
- **Written** to `passes/P/setups/NAME/`: `recipe`, `recipe 1`, then
  `after PARENT` unless `fresh`, then the lines; `README.md`;
  `scripts.json` where given; never `config/`, a root's alone; and
  `files/`, where its `forge commit`, `forge push` or `deployment patch`
  lines name any, each PATH they name copied from the pass's `files/`
  (`passes.md`, 2) to `files/PATH`, so that the recipe holds all it
  gives (`../../client/recipe.md`, 2). A PATH the pass's `files/` lacks
  refuses, naming it, exit 1. A recipe that does not read
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
  (`../../draft/archive/landed-qa.md`, 5):
  - **a flow a landing changed on purpose:** the recipe's to follow; the
    agent edits `passes/P/setups/R/recipe`, as a journey's walk follows
    the controls a landing changes (`../../client/journey-walks.md`, 2.2),
    and runs `pass setup`
    again, which walks it from its nearest ancestor's copy, since its
    hash changed. The edit goes with the pass's next request;
  - **a flow no landing meant to change:** a finding against temper
    (`findings.md`, 2), naming the step, what it showed and the last
    build the recipe walked on; the cases on that setup are failed with
    it, and the others go on (`../../draft/archive/landed-qa.md`, 7).

## 8. Its code and tests

- **In `temper-qa-tool`,** a `place` module for sections 2 to 4 and 6
  and a `hold` module for section 5, beside the ledger's; a place's kind
  answers its values, and one function reads a setup's chain for both
  (`kinds.md`, 6).
- **Its world behind faces:** the drive tool, run as a process from the
  place's `bin/`; the archive (below); the install's checksums, in Rust
  (section 3; `builds.md`, 3 and 4); the holder's socket and its
  process. The drive tool's and the archive's production faces run their
  programs through the tool's processes face and their paths through its
  files face (`README.md`, 6), so that what they decide replays. The QA
  tool links none of the drive tool's code: the place's
  build reads its own recipes and speaks its own lines.
- **The archive** is the system's `tar`, run as a process with the
  captured environment and closed stdin, given only options GNU tar and
  macOS's bsdtar both take, so that a place works on either
  (`../macos.md`): `tar --create --file ARCHIVE --directory SCRATCH
  config data`, the members present (a box with fake backends may make
  no `config/`), and `tar --extract --file ARCHIVE --directory SCRATCH`.
  No `--exclude`: the two tars match its patterns from any `/` of a
  name and read them as wildcards, and anchoring them takes GNU's
  `--anchored --no-wildcards`, which bsdtar refuses. So what a start pins
  (section 4) is left out by the tool, by its path whole:
  - **at a save,** each left-out path present below SCRATCH is moved
    aside by a rename into a private directory beside the archive's
    temporary file, the archive's name with `.left-out` appended
    (`copies/R/copy.tmp.left-out/`, one entry per left-out path, named
    for its place in the list from `0`), before `tar --create` runs, and
    moved back once it has exited, whatever its exit; the directory is
    then removed. One left by an interrupted save is removed before the
    next save moves anything, and a successful save of R removes it with
    `copies/R/`'s other files (section 4), so what it held is lost only
    to the box's next start, which pins its agent again. A rename that
    fails refuses the save, naming the path, `PATH: WHY`;
  - **at a restore,** once `tar --extract` has exited 0, each left-out
    path present below SCRATCH is removed, a directory with all it holds,
    before the restore goes on; a removal that fails refuses it, `PATH:
    WHY`.

  A tar that exits otherwise refuses, `tar --create: STATUS: STDERR` or
  `tar --extract: STATUS: STDERR`. The archive's format is the one the
  system's tar writes by default; a copy is restored only at the place
  that made it (section 2), so by the tar that wrote it. Copies saved before this form hold the same members, those with a
  pinned `agent/` among them, and restore unchanged.
- **Focused tests,** in the QA world (`README.md`, 6), with no box and
  no browser: `temper-client-journeys` in the place's `bin/` a scripted
  program answering `recipe list`, `recipe hash`, `recipe walk` and
  `drive` as the test tells it, `tar` a scripted program for the
  archive's own face, and an archive face that copies directories of
  the world's tree:
  - a copy restored and its check walked; a copy refused by name made
    again from its parent's copy, each copy saved on the way, and the
    refused one removed; a copy whose box fails at its start, its walk
    answering no line, removed and made again from its parent's copy,
    at a box's place and a deployment's, while one whose walk answered
    a line, a later walk's failure and a fresh box's stop with exit 4,
    every copy kept; a chain with no copy walked from a fresh box;
  - a walk stopped at a line exiting 5 with the landings since the
    copy's build;
  - a copy saved without its worker's pinned `agent/`, the rest of
    `data/` with it, and a copy holding one restored without it;
  - the install's checksum refused for a file the manifest disagrees
    with; the checksums' lines equal to `sha256sum`'s for plain and
    escaped paths (`builds.md`, 4), and their check naming each file
    that differs or does not read;
  - the archive's saves and restores through `tar` setting aside,
    moving back and removing pinned paths, and refusing in their words;
  - the real faces, on both systems, in the real loop: the system's
    `tar` saving a copy without a pinned `agent/` and a restore removing
    one an older copy holds, with a directory of the same name elsewhere
    kept, and `copies/R/` and SCRATCH holding nothing set aside once it
    ends; the checksums of a path that is not UTF-8, on Linux alone
    (`../macos.md`, 3);
  - a holder giving lines and answers, `pass line`'s exits, `pass
    close`'s kept lines, and the holder ending its session once the
    ledger ends the pass session, its two halves over scripted peers
    (`README.md`, 6), and through the socket once in the real loop;
  - `pass open --ctl` starting `drive --ctl`; at a deployment's place,
    a restore walking with `--scratch deployment`, its ports and
    `--bundle bundle`, its copy saved from that directory, and the
    holder starting `drive --deployment` with the chain's root's
    `config/` and `--bundle bundle`, with `--ctl` for `pass open --ctl`
    and without it for `pass open`;
    `fresh`, and a setup of the other kind, refused in their words.

## 9. Open questions

- **A copy's archive compressed:** a plain tar. Default: plain, until a
  copy's size tells against it.
