# Passes

Provisional, 2026-11-12, revised 2026-11-25, from
`../../draft/archive/landed-qa.md`, sections 2, 7 and 9, milestones Q3, Q4 and
Q6 of `../../draft/archive/landed-qa.execution.md`. A pass
judges a kept build of main: it takes the queued cases that build holds,
runs one agent session over them all at a place, records each outcome
and finding as it goes, and, with failed cases, waits on its findings'
goal and resumes on a build holding that goal's landing.

## 1. In one page

- **Its subject is a kept build** (section 3): the newest reached build
  holding queued cases' landings, with every case it holds, taken in one
  step when a place is free.
- **One agent session** (section 4), the coding agent's command line at
  the pass's effort, within `bounds.pass`, given a brief of its cases,
  build, place, setups and commands, and a tree of the build's commit to
  read.
- **Recorded as it goes** (section 5): each case's outcome, each
  finding and each note, through the ledger's commands, so a session
  ended early loses nothing recorded. A finding never stops it.
- **Its end** (section 6) is its process's: each case's state decided,
  the report written, and, with an open finding, a request made and the
  pass waiting, holding no build and no place.
- **Its resume** (section 7) on a build holding its goal's landing:
  open findings checked first, then its failed cases, then the cases
  that build holds; two goals at most per finding.
- **Its report** (section 8): `../../domain/findings.md`, 8's fields,
  from the ledger.

## 2. A pass's directory

`passes/P/` in the home:

- **`setups/`:** the pass's recipes, the build's commit's
  `temper/docs/development/qa/setups/` written out through the git face
  (`landings.md`, 6) at its start and again at each resume, from that
  resume's build. The agent walks these (`places.md`, 4), edits them
  (`places.md`, 7) and adds to them (`places.md`, 6).
- **`files/`:** the files the session's `forge commit` lines commit
  (`../../client/drive.md`, 3.5), written there by the agent at their
  repository paths before the line that commits them; the drive
  session's files directory (`places.md`, 5), and what a new recipe's
  `files/` is copied from (`places.md`, 6). Made, empty, at the pass's
  start; kept across its resumes.
- **`report.json`:** its report (section 8), written again at each
  session's end.
- **`requests/R/`:** each request as filed (`findings.md`, 5).

## 3. A pass's start

The loop's third step (`README.md`, 4), for each free place
(`places.md`, 2) in `places` order, once no waiting pass resumes there
(section 7):

1. **Chosen:** the ledger's rule (`ledger.md`, 4.8) read for a build it
   may take; with none, nothing is done.
2. **Prepared:** the build installed at the place (`places.md`, 3), and
   `passes/P/setups/` written from the build's commit, P the number the
   pass will take. A failure here records nothing and is the step's,
   tried again at the next tick.
3. **Recorded:** the ledger's start, in one write (`ledger.md`, 4.8),
   which takes the cases queued at that moment.
4. **Started:** its session (section 4). A start that fails before the
   agent runs ends the session `unstarted`, which undoes the pass's
   start (`ledger.md`, 4.12), its kept tree retained; the failure is the
   step's.

## 4. A pass session

- **Its directory,** `sessions/N/`: `brief` and `output`. Its kept
  tree is `trees/PLACE` under the home's private `trees/`, PLACE the
  place's one-based position in the configuration's `places`. A place
  has its own tree, separate from triage's and every other place's.
  Made once with `git worktree add --detach`, then before each session
  moved to the pass's build's commit with `git checkout --force
  --detach COMMIT` followed by `git clean -ffdxq`, as triage's
  (`triage.md`, 2), and removed and made again if moving fails. It
  starts with exactly that commit's files for the agent to read the
  landings' drafts and the design; the brief names the kept tree.
- **Its command** as triage's (`triage.md`, 5), each `{effort}`
  replaced by `agent.pass`, its working directory the tree, its stdin
  the brief, with `TEMPER_INTERNAL_QA_HOME` and
  `TEMPER_INTERNAL_QA_SESSION=N`; its bound
  `bounds.pass`, then SIGTERM to its group and SIGKILL ten seconds
  later, its exit `bound`; its tokens as triage reads them.
- **Its brief,** written by code, in this order:
  - **What a pass is,** from `../../draft/archive/landed-qa.md`, 7: each case
    from its setup's restored copy, restored between cases, cases on one
    setup sharing a restore where none changes what the next sees, the
    outcome's notes saying which. QA judges the happy path alone:
    v0.1.0's stories and each part's bar, used as intended
    (`../../draft/v0.1.0.md`, 4). Edge cases are off it: malformed,
    out-of-range or oversized input, limits and bounds, refusals of bad
    input and their exit codes, errors, races and recovery. Run a case's
    intended-use steps alone; an edge case met is a note
    (`temper-qa pass note`), never a finding. A security hole or lost
    data still counts wherever it is. A page's words are
    compared where the case gives them (its `see`, as a `web see` line
    for a `web` case and a `tui see` line for a `tui` case), the rest
    judged against the draft's promises. A `web` step whose do is `web
    requests` is that drive line, and its see is compared with the
    line's last shown line, as a `see` is compared
    (`../../client/drive.md`, 3.2), in these words: "A `web` step whose
    do is `web requests` is that drive line: compare its see with the
    line's last shown line, `N requests in W waves`, which must equal it;
    one that differs is a finding, its seen the line's whole answer."
    Ctl cases first
    (`../../draft/archive/landed-qa.md`, 4 and 7): every `ctl` case before any
    `web` or `tui` one, each in a ctl session, `temper-qa pass open
    --ctl`, which starts no browser and no terminal
    (`../../client/drive.md`, 2.6), its steps `ctl` and `forge` lines, or
    `temper help PAGE`, a `see` compared with what ctl printed. A case's
    `temper help PAGE` is read with the place's own program, its whole
    path in the brief's pass part, `Help pages: PATH help PAGE`, as the
    setups' directory is given whole. A help page needs no box; its words are compared
    as a `see` is. At a deployment's place the
    same, its ctl cases first in a ctl session and then its web and tui
    cases in a full one, `temper-qa pass open`, the browser and the TUI
    on the deployment's engine (`deployment.md`, 5 and 6), its owner's
    changes `deployment patch` lines, which end the TUI and load the
    page again; a `tui` case's steps are `tui`
    lines, the TUI started at the first and signed in by the session
    (`../../client/drive.md`, 3.3), and its finding's evidence the
    screen's rows, `pass line "tui look"`'s answer saved to a file;
    looks are a person's;
    each outcome recorded as reached and each finding as found; a
    finding never stops the pass; blocked, for the pass's own reasons,
    is not failed, for temper's; what is off the path a note; a fault
    beside a case on the landing's surface a finding
    (`../../draft/archive/landed-qa.md`, 13); the fakes alone: the person's forge is the box's fake Forgejo, and a
    case no line or setup can bring to its state is a gap, not blocked:
    `temper-qa pass gap CASE KIND WHY` (`../../draft/archive/landed-qa.md`, 6 and
    12).
  - **The pass:** its number; its place and the place's kind; its
    build's commit; for a resume, the goal it waited on and whether it
    landed.
  - **The landings its cases came from:** each its number, goal, merge
    commit and title, and its drafts as triage names them
    (`triage.md`, 3), paths in the tree.
  - **For a resume, first:** each open finding of the pass, to check
    before anything else (section 7), with its setup, lines, expected
    and seen; then its failed cases, ctl's first.
  - **The cases:** each case assigned to the pass, its number, surface,
    page and landings, and each ask's setup, adds and steps, each step's
    do, expect and see; the `ctl` cases first, then `web`, then `tui`,
    each in case order.
  - **The setups:** `fresh` at a box's place, and each recipe of
    `passes/P/setups/` of the place's kind, its
    name and its README's first sentence, as `recipe list` gives them
    (`../../client/recipe.md`, 5.2). When neither `web` nor `tui` is
    configured, offer `fresh` and only recipes whose whole chain has no
    web or tui line, as triage does (`triage.md`, 3); a case whose state
    no setup reaches is brought there from the nearest where its lines
    can reach it, then becomes a recipe (`places.md`, 6). When no line
    or setup can reach it, record a gap (`gaps.md`, 3).
  - **The commands:** the place's (`places.md`, 4 to 6), the drive
    lines in brief (`../../client/drive.md`, 3), among them `web
    requests`, which answers the requests the page has sent since it
    last loaded, each with its wave, then `N requests in W waves`
    (`../../client/drive.md`, 3.2), ctl's box values and the `forge
    commit`, `forge push` and `forge checks` lines on the fake Forgejo. One line says quoted bytes take printable ASCII
    alone, other bytes as `\xHH`, with `\"` and `\\` for quote and
    backslash (`../../client/drive.md`, 3.1). Each file a
    `forge commit` or `forge push` names is written under
    `passes/P/files/` first
    (`../../client/drive.md`, 3.4 and 3.5; section 2), and the recording
    commands (section 5; `findings.md`, 2), each with its form; that
    `pass setup`, `pass recipe` and `pass open` may take minutes, each
    walk or start printing a line as it goes, so the agent runs them
    where its shell's own timeout does not cut them; and that each
    record is kept at once and the session is bounded.

## 5. Recording

Each refused as `ledger.md`, 5 words it, otherwise recorded at once:

- **`temper-qa pass outcome CASE RESULT NOTES [--finding N]...`:** a
  case's outcome (`ledger.md`, 4.10), printed `case N RESULT`. A later
  outcome for the same case in the same session is the one its end
  reads.
- **`temper-qa pass gap CASE KIND WHY`:** CASE assigned to this pass;
  KIND `ctl`, `fake` or `qa`; WHY one nonempty argument saying what is
  missing. Records outcome `gap` with its number, out of the queue at
  once, counted neither failed nor blocked (`gaps.md`, 3). Prints
  `gap N`.
- **`temper-qa pass finding FILE`:** a finding (`findings.md`, 2).
- **`temper-qa pass note WORDS`:** a note, WORDS one nonempty argument,
  printed `note N`.

## 6. A session's end

The loop's third step, for a pass session whose process has exited, or
whose bound has passed:

- **Ended** in the ledger with its exit, tokens and time; the kept
  tree, `brief` and `output` stay.
- **Lost:** a pass session still running when the loop starts is ended
  `lost` (`README.md`, 4) and counts as any end does.
- **Its holder** sees the session ended and stops the drive session
  (`places.md`, 5); the place is free once it has gone.
- **The ledger's end** (`ledger.md`, 4.12), with the setups the pass
  changed: each recipe of `passes/P/setups/` whose `recipe`,
  `scripts.json` or `README.md` differs from the build's commit's, or
  which that commit lacks.
- **`report.json`** written (section 8).

## 7. A resume

- **When:** the ledger's rule (`ledger.md`, 4.13), for a free place:
  its goal landed and held by a reached kept build, or ended without
  landing with a reached kept build to take. Having released its own
  build when it began to wait (`ledger.md`, 4.12), it waits while there
  is none.
- **Prepared** as a start is (section 3): the new build installed at
  the place and `passes/P/setups/` written again from its commit, which
  holds the recipes the pass's request put on main where its goal
  landed; then the ledger's resume, in one write; then a new session,
  whose brief has the resume's parts (section 4).
- **`temper-qa pass check FINDING RESULT NOTES`:** RESULT `holds` or
  `gone`, NOTES one argument; a check of an open finding of the pass
  (`ledger.md`, 4.9), printed `finding F holds` or `finding F gone at
  COMMIT12`. The agent checks every open finding first, from its setup
  and lines, then runs the failed cases, then the cases taken at the
  resume.
- **At its end,** the findings' rounds (`ledger.md`, 4.14): a finding
  gone is closed naming the commit; one that holds stays open and goes
  in the next request; one open after two goals is handed on.

## 8. Its report

`temper-qa report PASS` prints, and each session's end writes to
`passes/P/report.json`, one JSON object, from the ledger, in the QA
report's terms (`../../domain/findings.md`, 8) with a case's landings
where that contract has its mark:

```json
{
  "pass": 3, "place": "places/1", "build": "COMMIT", "state": "waiting",
  "cases": [
    { "case": 7, "surface": "web", "page": "work", "landings": [12, 14],
      "expected": ["the draft is listed under Drafts by its title"],
      "outcome": "failed", "notes": "...", "findings": [4] }
  ],
  "findings": [
    { "finding": 4, "title": "...", "severity": "medium", "area": "AREA",
      "landings": [12], "case": 7, "setup": "project-landing",
      "reproduction": ["web reload \"/projects/1\"", "web see \"Drafts\""],
      "expected": "...", "seen": "...", "evidence": ["0004-1280.png"],
      "state": "open", "goals": [] }
  ],
  "gaps": [],
  "notes": ["..."]
}
```

- **A case** has its last outcome, or `outcome` absent while it has
  none; `expected` is each step's `expect` across its asks, in order.
- **A gap** appears in `gaps` beside findings, with its ledger fields
  (`ledger.md`, 2.11); a case's `gap` outcome also carries `gap`, its
  number. Gaps count neither failed nor blocked.
- **A finding's** `reproduction` is its lines from its setup; a linked
  one carries `linked`, the finding it matched.

## 9. Its tests

Focused and fuzzy, over the loop's fake world (`README.md`, 6):

- **A case in one pass at a time:** under random sequences of triage's
  cases, builds kept, passes started, outcomes recorded and sessions
  ended, no case is ever assigned to two passes, and a held build is
  never deleted (fuzzy).
- **A start:** the newest reached build holding queued cases taken,
  with every case it holds and none it does not; a `tui` case left
  queued under `surfaces` `["web"]`; nothing started with no place
  free.
- **A session's end:** cases with no outcome back in the queue,
  uncounted; a case blocked twice out of the queue with its line; a
  session that recorded nothing blocking its cases; with no failed case,
  the pass ended and its build released and deleted at the next tick;
  with one, a request made and the pass waiting with its build
  released, deleted at the next tick with no queued case holding it and
  kept with one.
- **A resume:** on a build holding the goal's landing at a later commit
  than the landing's, newer than the pass's own; after its goal ended
  without landing, none while no reached kept build is kept, then the
  newest once one is; failed cases, joined
  cases and the cases that build holds assigned, and none queued later;
  a finding gone closed with the commit; a finding open after two goals
  handed on, with its line.
- **The brief** written for a start and for a resume, with its parts in
  section 4's order, holding section 4's words for a `web requests`
  step and `web requests` among its drive lines.

## 10. Open questions

- **A pass's bound:** 7200 seconds by default. Default: so, until the
  ledger's session times say otherwise (`../../draft/archive/landed-qa.execution.md`,
  Q4's tuning).
- **Cases left by an ended pass's bound:** they return to the queue
  uncounted and may be taken at once on the same build
  (`../../draft/archive/landed-qa.md`, 9), so a pass that always runs out of
  time is taken again. Default: so; the session's time in the ledger
  shows it, and Q4 tunes the bound.
