# landed-qa: QA of each landing, from kept builds and saved setups

Draft, 2026-11-11, from a session with the user that day; amended
2026-11-12 with the user, for QA on the fakes alone and for stores
patched forward; amended 2026-11-14 with the user, for places of two
kinds, a box and a deployment, and for ctl first. Goals land on
the project's deployment faster than anyone can check them by hand. This
draft makes QA after landing keep up: an agent beside the deployment,
woken by landings, building nothing, starting each check from a saved
setup rather than from a fresh box, and sending what it finds back as one
goal. It covers what a person meets through ctl, the web client and the
TUI, ctl first, on the green path, at a box or a deployment. The user's
words: QA "builds nothing (no slot needed)", is "event driven (no
coordinating agent)", and "a single run [covers] as much ground as
possible and [collects] as many findings as possible, to reduce round
trips".

## 1. In one page

- **Places of two kinds, one framework** (section 2): a box, temper-box
  as a person runs it, which is a product to validate in its own right;
  and a deployment, temper's engine, worker and agent as their own
  processes, each from its configuration as a deployment writes it, on
  the same fakes, for what no box can be set to. The builds, the ledger,
  triage, the lines, findings and resuming are the same for both. The
  deployment's kind comes first in the small, for ctl, then widens, and
  the two are then made one (section 10).
- **A pass judges a kept build of main, not a goal** (section 2): each
  landing queues cases, what it changed for a person; a pass takes the
  newest kept build that holds queued cases' landings and covers them all,
  so landings close together cost one pass.
- **Code for the mechanism, an agent for judgement** (section 2): code
  watches main, keeps the ledger, assigns cases, restores setups, files
  findings and deletes builds; an agent triages each landing and runs
  each pass. No agent coordinates, and none waits.
- **The build is the deployment's** (section 3): its after-landing work
  builds the box, the client, the engine, the worker, the agent and the
  web bundle at a landed tip and keeps
  them with a manifest naming the commit. A build is deleted once no pass
  needs it, at once when triage finds nothing to check.
- **Triage** (section 4), one landing at a time in main's order, reads the
  landing's draft and diff and answers no QA, or cases: what a person now
  sees or does, on which surface, from which setup, and what they expect,
  written before anything is tried. Its ctl cases come first, and a pass
  runs them first, with no browser and no terminal.
- **Setups** (section 5): a recipe, kept in the repository, and a copy of
  the stopped place it reaches, box or deployment, kept as a cache. A
  copy is restored at its place with the pass's build, which patches its
  store forward; one the build refuses is a finding, and is made again
  from its recipe. The catalogue grows as passes need new states.
- **No real backend** (section 6): the fake forges, keeping their state
  across a restart, and the fake model on the setup's scripts, strict, so
  that a turn no script answers fails rather than drifts. No place runs a
  Forgejo, the box's own included.
- **A pass covers all its cases** (section 7), each from its setup's
  restored copy, its ctl cases first, through drivers that speak the
  setups' words to ctl, a browser and a terminal and print what is shown
  as text, recording each outcome as it goes. A finding never stops it.
- **Findings go back as one goal** (section 8): one request listing each
  finding with its setup and the lines that reproduce it from there, so
  that its fix carries a test.
- **The ledger** (section 9) holds every landing, case, finding, pass and
  build. A pass waiting on its goal resumes on a kept build holding that
  goal's landing: its findings checked again first, then its cases.
- **What it needs of temper** (section 10): drivers from built binaries, a
  box restored at its place with a newer build, the fake model's scripts
  given and strict, fake forges that keep their state, a recipe's format
  with its commits to a fake forge; for deeper states, scripts that read
  values from the conversation; for deployments, a place of that kind,
  ctl first, then the web client and the TUI there, then one framework
  for both kinds.

## 2. The pass

- **Its subject is a kept build of main** (section 3), at a commit holding
  one or more goals' landings. A pass checks every queued case whose
  landings that commit holds, so landings an hour apart are one pass when
  the first has not started.
- **One at a time per place:** a place is a fixed directory, fixed ports
  and a `bin/` where one box or one deployment runs (section 5). The
  deployment configures
  how many, on a machine of their own, apart from those that gate
  landings and run the tiers after them; one to start. A pass waiting on
  its findings' goal (section 9) holds no place, and another pass may run
  meanwhile.
- **Two kinds of place:**
  - **a box:** `temper-box` from its document, as a person runs it on
    their machine, the playground's fakes its backends
    (`../protocol/setup/box.md`). Its setups reach what a person can do
    from a fresh box, and it stays a place for QA of the box itself:
    its welcome, its playground and `temper box`;
  - **a deployment:** temper's engine, a worker and the agent, each its
    own process from its own configuration, as the project's deployment
    runs them (`../protocol/configuration/engine.md`), with the fake
    forges and the fake model as processes of their own beside them.
    Every setting a deployment's document takes is a setup's to give: a
    project's landing by `turn`, its `batch` and its gates among them
    (`../domain/landing.md`). It also passes through the wires between
    engine, workers and agents, and restores a copy with a newer build of
    each, as a redeploy does.

  A place's kind is the deployment's choice when it configures the
  place; a case runs at a place of its setup's kind.
- **Beside the deployment, not in it:** a pass takes none of the
  deployment's worker slots and builds nothing. Its agent is a coding
  agent's command line, named with its effort by configuration, on the
  machine that holds the places; triage runs it at a lower effort.
- **No coordinator:** code moves landings, cases, passes and builds
  through their states (section 9) on events: main moving, a build kept,
  an agent's end, a goal's landing. Each agent session is a leaf: given
  its inputs, it records its results through the ledger's commands and
  ends.
- **A tool of temper's,** `temper/crates/temper-qa-tool/`, beside the
  journeys' and the twin's, configured by the deployment: the agent's
  command and efforts, the places, where builds and copies are kept, the
  surfaces' areas, the area its goals are started under, and its bounds.
- **Its own faults are its owner's,** the area that runs the pass: a case
  waiting past its bound for a build, a case blocked twice, an agent that
  records nothing. Each is a line for that area, never a finding against
  temper, unless temper's behaviour is its cause (section 7).
- **Beside `../domain/findings.md`, 8, not instead of it:** that section
  is temper's own pass, for a project's people, not yet built. This pass
  runs beside the deployment until temper's exists, and differs from it:
  it triages every landing rather than marks and a chooser, judges a kept
  build of a landed commit rather than the newest cleared main, may run
  while another waits, and files its findings as one goal's request
  rather than as forge issues for a triager. This draft changes nothing
  of that section. Its report keeps that section's fields, with a case's
  landings where that contract has its mark (section 7).
- **Beside `v0.1.0.md`, 3's QA on real backends,** which stays as it is:
  this pass is the fakes', at every landing.
- **`../../development/qa.md` stays its account of the hands and the
  checks:** ctl as the person, the forge read as an observer, a check
  comparing temper's account with the forge's. This draft adds where a
  pass starts from (section 5) and how it drives the pages and the TUI,
  which `../../development/qa.md`, 8 leaves open.

## 3. Landings and kept builds

- **A landing is seen on main's ref,** not on its first-parent line: the
  watcher keeps the last head it saw and, on a move, lists every commit
  the new head gained, since a later merge can carry a goal's merge off
  the first-parent line. A goal's landing is named by its merge's
  trailers (`squash-in.md`, 3), and until those land, by its subject.
  temper's task states are no signal: a goal landed out of band may end
  cancelled, its landing said in words.
- **The deployment keeps builds,** as part of its after-landing work: at a
  landed tip it builds the box, the client, the engine, the worker and
  the agent, and the web bundle as the journeys take it
  (`../client/web/journeys.md`, 8.1), and keeps
  them in a directory named by the commit, with a manifest naming the
  commit, each file and its checksum. A build is published to the pass
  only once every executable prints that commit as its version. temper
  needs nothing for this: it is the deployment's, on its disk, a tier of
  its own or a part of one (`../domain/after-landing.md`, 2).
- **Not every tip is kept:** after-landing work covers several landings
  at once and a receipt may judge a tip with no run
  (`../domain/after-landing.md`, 3). A case waits for the next kept build
  that holds its landing. One that has waited past the deployment's bound
  is a line for the owner (section 2).
- **A build is deleted** when no pass holds it, triage has reached it
  (section 4), and either a newer build triage has reached is kept, or no
  queued case's landing is in it. Nothing else deletes one. So what is
  kept is the newest triaged build while cases wait for it, the builds
  passes hold, and the builds triage has not reached yet. A debug build's
  set is about 400 MB.

## 4. Triage

- **One landing at a time, in main's order,** as the watcher sees them: an
  agent session given the goal's draft (named in its start), the
  landing's diff and its stat, its commits' `QA:` trailers if any, the
  surfaces (ctl's groups, the web client's pages and the TUI's screens,
  by their design and journeys), the setups' catalogue, a line each, with
  its kind (section 5), and the cases queued. A stat names each generated
  directory once, with its count, rather than every file in it, so that a
  landing of goldens stays cheap to read. Triage has reached a commit
  once every landing it holds is triaged.
- **It answers,** through the ledger's commands:
  - **no QA:** the landing changes nothing a person meets on the
    surfaces in scope, with a line saying why;
  - **or cases,** each with its surface (`ctl`, `web` or `tui`), the
    page or screen, or for ctl the group of tools, the setup it starts
    from (or the nearest one, and what the case must add to it), the
    steps from there in words, and what the person expects at each, from
    the draft's promises, written before anything is tried
    (`../domain/findings.md`, 8).
- **The green path only:** a case is the draft's intended use, as
  v0.1.0's happy path weighs it (`v0.1.0.md`, 4). Edge cases, failures
  and recovery wait (section 12); a security hole or lost data seen on
  the way is a finding wherever it is.
- **Surfaces in scope:** ctl, the web client and the TUI, ctl first (the
  user, 2026-11-14): triage writes a landing's ctl cases first, the
  person's acts and temper's account as `temper ctl` gives them, and a
  pass runs every ctl case it holds before its web and TUI ones, so that
  what a landing changes is judged first where nothing but temper stands
  between the person and it. The help comes next (section 12).
- **Cases merge:** a case for a page, screen or ctl group a queued case
  already covers joins it, and the case keeps every landing that asked
  for it.
- **Cheap:** triage reads and starts no box, within a bound of time and
  tokens the deployment sets.

## 5. Setups

- **A setup is a recipe and a copy,** of one kind of place (section 2).
  The recipe says how a fresh place of its kind reaches a state; the copy
  is that place's data, every process of it stopped, ready to start
  again. The recipe is the record, kept in the repository; the copy is a
  cache, kept beside the places.
- **A recipe** is a directory, `temper/docs/development/qa/setups/NAME/`,
  under `docs/` as the journeys' are, so that a recipe lands as
  documentation:
  - `recipe`: text, one record per line, in the journeys' syntax
    (`../client/web/journeys.md`, 8.2): `recipe 1` first; `after NAME`,
    its parent, a chain of at most eight with no cycle; then its steps,
    each led by its client: `web` and a record of the web journeys'
    setup (`press`, `submit`, `toggle`, `input`, `see`, `reload`), `tui`
    and a record of the TUI journeys' walk (`../client/tui/journeys.md`,
    8.1), `ctl` and a `temper ctl` act as the signed-in person, waited on
    as ctl waits, or `forge` and a commit of named files to the person's
    repository on a fake forge, as the person makes one outside temper,
    waited on until its checks end; and last, its state's check, `web
    see` or `tui see` lines;
  - `files/`, if its `forge` lines commit any: the files they commit, so
    that the recipe holds all it gives. No line names a credential: the
    driver reads the person's from the box and hands it to the forge's
    request in a file only it reads;
  - `scripts.json`, if it has its own: the fake model's scripts it adds
    to its parent's (section 6);
  - `README.md`: one paragraph, the state it reaches and the pages,
    screens and ctl groups it serves;
  - for a deployment's setup, `config/`: the documents its engine,
    worker and agent start from, written as a deployment writes them,
    with its fake forges and the fake model in place of real ones. A
    root recipe starts its place from them, where a box's starts from a
    fresh box; a child changes them only through its lines, as a person
    or an owner would. No document names a credential: the place makes
    them, as a box makes its own.
- **A copy** is the place's whole data and configuration, every process
  of it stopped, as a backup has it (jig's `protocol/store/README.md`,
  2), its fake forges' state with it, the engine's, worker's and agent's
  each for a deployment, archived together and named by the hash of the
  recipe, with its files and its parents', and by the commit whose build
  made it.
- **At its place:** a place has a fixed directory, fixed ports and its own
  `bin/`. A pass installs its build into the place's `bin/` before any
  process starts, so the copies made there name the place's programs,
  never a kept build's directory. A copy is unpacked where it was made,
  its box or its deployment's processes started from `bin/` with the
  recipe's scripts. The place prints a new sign-in link, which the
  drivers follow, or writes it to a file ctl signs in from, never
  printed, so a copy holds no browser's session. The place's directories
  are private, whatever the umask its process runs under. The recipe's
  closing check, `ctl` lines for a recipe with no web or TUI line, runs
  before any case starts.
- **Patched forward, or a finding:** a stored format's change moves its
  version and brings its patch (the user, 2026-11-12; `upgrades.md`, 4
  and 6), so a newer build patches an older copy's store as it starts,
  as a redeploy does. A copy it refuses, by name, or whose patching
  fails, is a finding against temper, naming the store, the copy's commit
  and the pass's build, so that every pass also proves the patches. The
  pass then walks the recipe from its nearest ancestor whose copy
  restores, saves the new copy, and goes on with the setup's cases. Any
  other failure to start, or a panic, is a finding (section 7).
- **A recipe that no longer walks** is judged by the agent against the
  drafts of the landings since it last walked: a flow a landing changed
  on purpose is the recipe's to follow, and the agent edits it, as a
  journey's setup is edited (`../client/web/journeys.md`, 8); a flow no
  landing meant to change is a finding against temper, naming the step,
  what it showed and the last build the recipe walked on. The cases on
  that setup are blocked by the finding; the others go on.
- **Kept small:** one copy per recipe, the newest that restored.
- **The catalogue grows:** a case whose state no setup reaches is brought
  there by the agent from the nearest setup, through the drivers, which
  keep the lines it gave (section 7). Once the case has run, those lines,
  without the observations, become a recipe of their own, its README
  written by the agent, proven by walking it from its parent on the same
  build. A pass's new recipes and edits land with its findings' request,
  or alone when it found nothing (section 8).
- **First recipes:** from the journeys' setups (`project-landing`,
  `chat-draft` and the welcome's), for the web client and the TUI. A
  deployment's first setup is a project on the fake Forgejo landing by
  `turn` with a `batch` above one and a gate the fake Forgejo dispatches,
  so that landing by turn, which no box can be set to, is judged first.

## 6. The fakes

- **No real backend:** no real model, no real account and no real forge,
  the box's own Forgejo included (the user, 2026-11-12: QA on the fakes
  stays the rule). A pass's box or deployment runs on its fake backends:
  the person's forges, GitHub and Forgejo alike, are fake forges
  (`../protocol/setup/box.md`, 5) keeping their state in the place's data
  across a restart (section 10), so that a copy keeps their repositories,
  pull requests and checks. A deployment runs the same fakes as processes
  of their own, as a box runs them as its steps. No place runs a Forgejo
  or a runner. What only a real forge shows, the box's own Forgejo set up
  on the person's machine among it, waits for real backends (section
  12).
- **CI is the fake forge's,** decided by a file's content
  (`../testing/fakes.md`, 2), so that a recipe makes a head pass or fail
  by what its `forge` lines commit; a landing's gate is a workflow the
  fake forge runs when dispatched, decided the same way (skein's
  `testing/fakes/forge.md`).
- **The model is the fake model, on the setup's scripts:** the standalone
  document's scripts, matched by their cues
  (`../protocol/setup/playground.md`; skein's `testing/fakes/llm.md`,
  2.4). The box takes a setup's scripts, its own after its parents', in
  place of the scripts it generates, and keeps its own listener,
  credentials and outputs, so that scripts are the same at every place.
  They are given when the box starts and never change while it runs.
  Matching reads only the conversation, so the scripts answer a restored
  box as they answered the box that made the copy. A recipe's scripts
  cover its own steps and, where its cases go on past its state, theirs.
- **Strict:** on a pass's box, a turn no script answers fails with a
  provider's error naming the conversation's last request, rather than
  taking a seeded answer or `done`. The case it stops is blocked, naming
  the setup's scripts, never a finding against temper (section 7).
- **Twins for deep states:** a goal's design, coding, failed CI, repair and
  review are scripted from a real run's transcripts by `temper-twin`
  (`../testing/real-loop.md`), rather than by hand. B0's and B1's twins
  name the task numbers, pull requests and heads of the run they came
  from, which a fresh box's fake forges do not repeat. A twin joins the
  setups once its calls can take such values from the conversation
  (section 10), with the person's acts and the parent it was recorded
  from in its recipe, and walks there. A short state of a page is
  written by hand.

## 7. The run

- **A pass is one agent session** given its cases, its build, its place,
  the setups and the drivers, within a bound of time the deployment sets.
  It records, through the ledger's commands, each case's outcome as it
  reaches it and each finding as it finds it, so that a session ended by
  its bound or a crash loses nothing recorded.
- **Its report** is `../domain/findings.md`, 8's: each case with its
  landings, what was expected, its outcome (`passed`, `failed` or
  `blocked`) and notes; each finding with its title, reproduction,
  severity, area and the case's landings, and with section 8's fields.
- **Each case from its setup's copy,** restored between cases, so a case
  that breaks its box leaves the next one whole. Cases on one setup share
  a restore where none changes what the next one sees; the agent says
  which.
- **The drivers** speak the recipes' words (section 5), a line at a time
  on standard input, and add `look`: for a browser, the page as its
  accessible text, and a screenshot at a given width; for the terminal,
  the screen's rows. Each driver keeps the lines it was given, so a case's
  steps become a recipe's steps and a finding's reproduction; `look`
  lines are observations, kept in neither.
- **ctl first, then beside them,** as `../../development/qa.md` has it:
  the person's acts and temper's account, and the forge read as its
  observer. A pass's ctl cases run first, in a session of ctl and
  `forge` lines alone, which starts no browser and no terminal and needs
  neither.
- **Code compares where it can:** a page's text against the case's
  expectation, where the case gives it word for word. The agent judges
  the rest: whether the page does and says what the draft promised. Looks
  are the person's, and later a render check's (section 12).
- **As much ground as it can:** a finding never stops the pass; it is
  recorded, and the next case begins.
- **Blocked is not failed:** a case that cannot reach its point for the
  pass's own reasons (a copy not made again, a script missing, an agent
  out of time) is blocked, with why. A case that cannot reach it because
  temper fails on the way (a box that will not start on a fresh place or
  panics, a recipe's step that broke with no landing meaning it to,
  section 5) is failed, with that finding. This is
  `../../development/qa.md`, 6's finding for a start refused or a panic.
  A copy's store refused, or its patching failed, is a finding of its
  own, and the setup's cases go on from the copy made again (section 5).
- **Evidence beside the ledger:** each finding's screenshot or screen
  text, and the box's log where it matters, with the case and the
  commit.

## 8. Findings and their goal

- **A finding** has an id, a title, a severity, its surface and area, its
  case's landings, its setup, the driver's lines from that setup that
  reproduce it, what was expected and what was seen, and its evidence.
  One that matches an open finding (the same setup and lines, or the same
  page and expectation) is linked to it, never filed twice.
- **One request per pass:** its findings, each with its case, as a draft
  put on main as any goal's request is, then one goal started on it under
  the owning area. Its fix carries a test in the lowest world that
  reproduces each finding (`v0.1.0.md`, 4); the driver's lines are where
  that test starts.
- **The pass's new recipes and edits** land with the request, as
  documentation, or alone when the pass found nothing.
- **The green path only:** the request asks for what the cases expected
  and nothing past it. What the agent noticed off the path goes in the
  ledger's notes, unless it is a security hole or lost data.

## 9. The ledger and resuming

- **One ledger per project,** kept by code beside the places, the only
  record of the pass's state: each landing (seen, triaged), each case
  (queued, assigned to a pass, passed, failed, blocked with a count),
  each finding (open, closed, handed on), each pass (running, waiting,
  ended) and each kept build (kept, held by a pass, deleted), with each
  agent session's time and tokens. Its commands are the agents' only
  way to record, and refuse what does not fit: a case with no surface or
  expectation, an outcome for a case not assigned to the session's pass.
- **A pass starts** when a place is free and a kept build that triage has
  reached holds a queued case's landings. It takes the newest such build,
  holds it, and takes every queued case that build holds, in one step, so
  no case is in two passes.
- **A pass ends its run** when its session ends. Cases with no outcome go
  back to the queue, uncounted, and may be taken at once on the same
  build. A blocked case goes back with its count raised; a case blocked
  twice leaves the queue, a line for the owner. With no failed case, the
  pass has ended and releases its build. With failed cases, it files its
  request (section 8) and waits, holding its build, its failed cases and
  its open findings.
- **A waiting pass keeps its page:** a case queued later for a page and
  setup that one of its open findings is on joins the waiting pass, and
  runs at its resume.
- **It resumes** once its goal has landed and a kept build that triage has
  reached holds that landing, not necessarily at the landing's own commit.
  It takes the newest such build and releases the old one; a new agent
  session, given the ledger rather than the old session's conversation,
  checks every open finding first, then runs its failed cases, then the
  cases it took at resume: those that build holds, and none later. A
  finding that holds again stays open and goes in the next request; one
  that no longer holds is closed, naming the commit.
- **A goal that ends without landing** leaves its findings open; the pass
  resumes on the newest kept build triage has reached, and files them
  again.
- **Two rounds at most:** a finding still open after two goals is handed
  on, a line for the owning area to weigh, and leaves the pass's
  requests; a pass with nothing else open then ends.

## 10. What it needs of temper

For the first passes, on the web client and the TUI:

- **Drivers from built binaries:** `temper-client-journeys` walks and
  drives with a given box, client and bundle, in a given scratch
  directory, running no build and writing nothing outside it
  (`../client/web/journeys.md`, 8.1 builds from the checkout and
  scratches under `/tmp`), and drives a line at a time on standard input,
  with `look`, as section 7 has it, for the browser and the terminal.
- **A recipe's format and walk:** section 5's `recipe`, its chain of
  parents, its clients' records, `forge` among them, and its check,
  walked by the drivers from a place's copy or a fresh box.
- **A box at its place:** a stopped box's data and configuration,
  unpacked where they were made, start with a newer build installed in
  the same `bin/`, with its fake forges' repositories, pull requests and
  checks and its tasks as they were, its store patched forward as a
  redeploy patches it; a store the build has no patch for is refused by
  name, never read wrongly.
  A process-tier story proves it: the engine, its agents, its adapters
  and the bundle all run the newer build, and nothing runs the old one.
- **The fake model's scripts and strictness:** `temper box
  --fake-backends` takes scripts in place of those it generates, keeping
  its listener, credentials and outputs, and a strict switch with which a
  turn no script answers fails (section 6).
- **Fake forges that keep their state** across a restart, in the box's
  data, so that a copy holds them, and take a person's commit with its
  checks decided by its files (section 6).

For deeper states:

- **Scripts that read the conversation:** a scripted call may take a value
  from a tool's output by a pattern, so that a twin replays on a box whose
  numbers and heads differ (section 6).

For deployments, in three steps, each usable as it lands:

- **ctl on a deployment, first and small:** a place of the deployment's
  kind, the kept build's engine, worker and agent started from a setup's
  `config/` at the place's ports, the fake forges and the fake model as
  processes of their own; a deployment's copy saved and restored, with a
  newer build of each; a session of ctl and `forge` lines alone, signed
  in from a file and starting no browser; the worker's git reaching the
  fake forge's repositories, fetching, merging and pushing as on a real
  forge, so that landing by turn's merges run there; the `ctl` surface
  and its cases first. It reuses what the box's kind has: the builds,
  the ledger, the recipes' lines, the place's commands and findings.
- **The web client and the TUI on a deployment:** the drivers' browser
  and terminal reach a deployment's engine rather than a box of their
  own, once the drivers' browser starts at the places.
- **One framework:** a place's kinds behind one interface (install,
  start, save, restore, its values for the lines), setups' recipes of
  either kind in one catalogue, and what the first two steps duplicated
  removed. The box stays a kind of its own, for QA of the box.

Nothing for the kept build: it is the deployment's (section 3).

## 11. Acceptance

Focused and fuzzy suites alone, each failing first:

- the watcher: a goal's merge carried off main's first-parent line by a
  later merge is seen, once;
- triage in main's order: a landing's cases recorded only after the one
  before it is triaged; a case refused with no surface or expectation; a
  case for a covered page or ctl group merged, keeping both landings; a
  `ctl` case recorded and a pass running its ctl cases first, with no
  browser started; a stat naming a generated directory once;
- builds: deleted on no QA, when a newer triaged build is kept, on a
  pass's end and on its resume, and never while held or before triage
  reaches them; a case waiting for a build past its bound is a line;
- passes: a case in one pass at a time; cases with no outcome back in the
  queue at a session's end; a case blocked twice leaving the queue;
- the recipe format: its chain read and walked, a cycle or a ninth parent
  refused, its hash changing with its parents';
- a copy whose store a newer build refuses by name: a finding naming the
  store, the copy's commit and the build, and the copy made again from
  its nearest restoring ancestor;
- the fake model: a box's scripts given in place of its own; under the
  strict switch, a turn no script answers fails, naming the request;
- the drivers: a line given and its effect; the kept lines, without
  `look`, replayed to the same page; no build run and nothing written
  outside the scratch directory;
- the `forge` line: its files committed to a fake forge as the person and
  its checks waited on; its files changing the recipe's hash; no
  credential in the recipe, the kept lines or the driver's output;
- a box at its place on its fake backends: its fake forges' repositories,
  pull requests and checks back after a restore with a newer build, and
  no Forgejo or runner started;
- a deployment at its place: its engine, worker and agent started from a
  setup's `config/` on the fakes, stopped, saved, and restored with a
  newer build of each, its store patched forward and its fakes' state
  back; a project there landing by `turn` at a `batch` of two, its gate
  dispatched on the fake Forgejo and the turn merged by the worker's
  git; ctl signed in with no link printed; the place's directories
  private under any umask;
- resuming: on a build holding the goal's landing at a later commit; open
  findings checked first, then failed cases, then cases that build holds,
  and none later; a finding open after two goals handed on.

The process-tier stories of a box and of a deployment at their places are
written and compiled in their milestones, and run in the process tier
after they land.

## 12. Later

- **The help** as a surface, then each part's own bar in `v0.1.0.md`, 5.
- **Edge cases, failures and recovery,** once the green path holds.
- **A landed look request's render check** in a pass, once
  `look-requests.md`, 4's comparison is a command.
- **Real backends** for a case the fakes cannot reach: a real model's
  turn, GitHub itself, or the box's own Forgejo on the person's machine,
  with `real-journeys.md`'s redaction where a page is recorded.
- **temper's own pass** (`../domain/findings.md`, 8), taking what this one
  learns.
- **More places,** when passes queue.

## 13. Open questions

- **A finding beside a case:** a fault on the path the case did not ask
  about, such as a wrong word beside the checked one, is filed or only
  noted. Default: filed, when it is on the landing's surface.
- **A pass's recipes landing unread:** they land with its request, with
  no person reading them first. Default: unread, since a recipe that no
  longer walks is the pass's to judge and repair (section 5).
- **A box's setup that needs nothing of a box:** one whose cases a
  deployment serves as well. Default: kept as the box's until the one
  framework (section 10) weighs each, since the box is a product of its
  own to judge.
