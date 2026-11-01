# The landing pipeline

Draft, 2026-11-01. Nothing here is adopted until the user approves it.
The second half of `landing-model.md` revision 4 (10-30), split off at
the user's word and brought loosely in line with how fabrig's lab runs
today (fab, the agents repository at 1dc102c): how a change ready to
land lands on main, and what follows it. What comes before ready is
`code-pipeline.md`; `reference-case.md` runs both. The landing is
temper's forge connector (`landing.md`, `forge.md`, `after-landing.md`,
`findings.md`) on jig's primitives: procedures for the mechanics, jobs
for checks, agent tasks for judgement, news to the goals concerned.
fabrig's way is one configuration (section 13); gap ids are revision
4's, new ones carrying on from them (section 15). Machines, slots and
sign-ins are left out for now.

Unqualified, `landing.md`, `after-landing.md`, `findings.md`,
`forge.md`, `worker.md` and `testing.md` are temper's `../domain/`
documents; jig's are named as jig's.

## 1. In one page

- **One model:** ready changes are **prepared** into candidates,
  **proved** on runners into receipts kept by key, and **landed** by one
  serializer per project, fast-forward only, at exactly the commit that
  was proved. Batching is only how many changes a candidate holds; it
  never changes what proves a commit or who lands it.
- **Mechanisms are temper's; choices are settings.** jig knows nothing
  of landing; temper keeps its candidates, receipts, verdicts and
  findings in the store, never in a repository.
- **An LLM for judgement, code for mechanisms:** preparing, proving,
  rerunning, bisecting and the turn are code; whether a red is a
  change's fault, whose a red after landing is, and triage are agent
  tasks (verdict tasks, a goal's agent).
- **Fresh by key** (section 3): receipts made before ready serve the
  candidate; only keys main or the other members changed are proved.
- **Keys per package, so selection is receipts** (section 4).
- **Candidates carry changes: one, or a batch** (section 5), in
  readiness order; a red batch is bisected by its suspects.
- **Every outcome is classed before it is anyone's** (section 7): a
  failed measured test is rerun alone before it is red.
- **One serializer per project** (section 8), its decisions persisted
  before their effects.
- **After landing, tiers follow main** (section 9): reds go to their
  suspects' goals as news, whose agents decide the repair.
- **Findings are temper's records; a QA ladder is a goal of walks**
  (section 10).
- **A forge's native queue is an accelerator, never a dependency**
  (section 11).

## 2. What fabrig's pipeline showed (10-28 to 10-30, before fab)

### 2.1 The queue's first night (10-28 22:13Z to 10-29 02:10Z)

- **The cost is builds, not tests:** a full check was 7 to 11 minutes,
  of which tests ran about 41 seconds; building and linking about 1,366
  test executables was the rest, until each package had one (then a warm
  full check took 2.5 to 3.5 minutes).
- **Selection decides throughput:** every batch ran the whole workspace
  while lockfile changes forced it; selecting the packages a change
  reaches (and their reverse dependencies) is what keeps checks short.
- **Most red was not the change's:** a full disk, a missing tool on
  PATH, the queue's own wrong flags, a ratchet file both sides lowered,
  a test that ran another package's binary, a starved test, a cleanup
  race in the test harness. Each was a queue rule or a tool fix, none a
  repair of the change.
- **Mechanical conflicts settle themselves:** a conflict confined to a
  counted baseline merged by re-running its writer, which refuses a
  raise; `landing.md`'s prepare by a project's tooling (5.3) is the
  general form.
- **Throughput doubled, then the queue saturated:** 23 merges an hour at
  midnight, 47 an hour by 01:08Z with about eleven implementers at once,
  41 to 47 an hour after, with about seventeen; the queue was busy 96 to
  98% of the time and the ready branches waiting grew from 36 (01:21Z)
  to 58 (02:10Z). The builders were not the limit: builds waited a
  median of 2 to 7 seconds for a slot in the builder's build cap, and
  averaged about two of eight cores. So capacity moved to the queue:
  batches grew from 8 to 16 (01:34Z), and a builder was drained to
  become the second prover (01:30Z to 01:56Z).
- **A red costs a big batch most:** one member's lint failure in a batch
  of 16 took five bisection rounds before the good members landed.
  Bisection keeps the good members, but every round is a full proof; a
  train (section 14) overlaps proofs, it does not shorten them.
- **The checker's own rules break it as the tree changes:** a selector
  that names a test binary fails the whole run the day the last such
  binary goes (nextest refuses a `binary()` matcher that matches
  nothing); selection by test names that a refactor cannot empty stays
  valid.
- **Starved is not the only congestion:** a CPU-bound test took 2.5
  times its idle CPU on the loaded prover with no time spent waiting for
  a CPU (shared cores, cache, clock), which no runnable-wait rule sees.
  The remedy was the testing pyramid's own: the costly story moved down
  a tier, where it belongs (`testing.md`, 2.4).

### 2.2 The days after (10-29, 10-30)

- **Checks before ready pay for themselves:** since a red in a big batch
  costs whole proofs, each branch is merged with main in its own warm
  builder, the queue's deterministic steps run there (counted baselines,
  the tools' suites, formatting, lints, a test build), and it is marked
  ready only once they pass and main has changed none of the files they
  checked since. The builders had cores to spare (2.1).
- **Readers run tests, not only builds:** a change of what a shared item
  means, not only its signature, also runs the tests of every package
  that reads it, merged with main, before ready: a build alone missed
  it.
- **One prover in production:** the train and its second prover were
  built and proved on disposable gates, then kept out of production
  while their supervised transport and lifecycle are built. A repair of
  the gate itself is let through by hand, alone and in full, before the
  batches.
- **After landing is a loop of its own:** the suites too slow for the
  queue (exhaustive profiles, browser suites, process stories, QA
  suites) follow main on a builder of their own, one tip at a time, and
  their new reds go to the coordinators whose branches are suspects,
  without holding the queue.
- **Several coordinators, one queue:** three coordinators, each on its
  own drafts with its own goals per tree, share the builders, the queue
  and the person; one writer per package or design document across them
  is kept by leases, and each branch's owner hears its outcomes.

## 3. Fresh by key

- **No age and no path rule:** a receipt holds for its key while the key
  stands. When a change joins a candidate, its packages' keys at that
  candidate are looked up: those that main and the other members left
  unchanged keep the receipts made before ready; the rest are proved in
  the candidate.
- **Admission may require it** (setting `before_ready`):
  - `keys`: a change joins a candidate only once its own deterministic
    keys at main's current tip have receipts; when main has moved
    something they read since, temper checks it again first, in a
    workspace it claims for the check;
  - `advisory`: a change joins as soon as it is ready, and the candidate
    proves whatever has no receipt;
  - `none`: no check runs before ready.
- **Why it is faster and as safe:** nothing is checked twice unless its
  inputs changed; a red is caught in the change's own workspace rather
  than in a batch, where it costs every member a proof; and nothing
  proves main but receipts for its exact keys.

## 4. Keys per package

- **A gate per package, or a gate whose key is per package:** a test
  gate's key for a package is a digest of the package's own inputs (its
  directory's tracked files but the paths the gate ignores), its
  dependencies' keys, the toolchain, the gate's definition and the
  workspace's shared inputs (the lockfile entries it resolves, the
  profiles). A package's key changes exactly when something its build or
  tests read changes.
- **Repository gates beside package gates:** a gate whose subject is the
  whole repository (a tools' suite, a counted baseline's ratchet,
  formatting of the changed files) has a key of its own: the paths it
  reads and its definition. It runs when its key changes, whatever the
  package selection says: a tools-only batch runs the tools' gate and no
  package's, a documentation batch may still run a tree's ratchet.
- **A candidate is proved** when every member's blocking gates hold at
  exactly its candidate: each test gate's keys by passed receipts, each
  judgement gate by its verdict (`landing.md`, 7.2). Only keys with no
  receipt run, as one dispatch with the set (one build for the set). An
  advisory gate is run and reported, and neither holds a landing nor
  starts a bisection or a repair.
- **Receipts outlive candidates, once certified:** a receipt is a key's,
  not a candidate's, made only from a certificate the serializer
  accepted (section 6). A candidate discarded later (a bisection half, a
  stale base) leaves its certified receipts good for every key they
  name, so its next preparation runs only the keys the change of parent
  reached. What an invalid or incomplete certificate carried is
  discarded whole.
- **Keys are computed by the prepare job** (gap L4): a package's key
  needs the build's graph (cargo's metadata, or the project's own list
  of packages and their inputs), so the project's tooling computes it on
  a checkout of the candidate, in the job that prepares the candidate,
  and reports each gate's keys with it (`landing.md`, 5.3, 6.1). The
  engine never reads a repository (`worker.md`, 2); it takes the keys
  the job reports, and full proofs audit them. A project with no tooling
  keeps the tree key: the candidate's tree without the gate's ignored
  paths, read from the forge.
- **Deterministic and measured gates** (gap L6): a gate declares its
  class. A deterministic gate's result depends only on its key's inputs
  (formatting, lints, a build, a counted baseline, a suite with no
  clock), so any runner temper dispatches and certifies may make its
  receipt, a workspace included. A measured gate's result also depends
  on the machine (tests and their time bounds), so by default only a
  prover makes its receipt (a project may take a workspace's,
  `tests_before_ready`, section 13). Either key names the environment:
  the toolchain, its targets and profile.
- **What a key cannot see** is named by the project and checked: a test
  that reads files by path or runs another package's binary joins the
  keys it depends on.
- **Full proofs audit the keys:** every N proof attempts (reds
  included), hourly, and when the provers have idled with main moved
  since the last, a full proof runs every gate whatever its receipts
  say. An idle audit proves main itself (parent and candidate both
  main's tip; nothing lands). A full proof that fails is a red like any
  other, bisected with every half proved in full; where the selections
  before it passed, the chat is also told of a suspected key gap.
- **Paths no gate reads** (documentation outside every package and every
  repository gate's paths, a gate's ignored paths) reach no key: a
  landing of them is proved on receipts alone (`landing.md`, 7.2), and
  runs no build.

## 5. Candidates: one landing, or a batch

### 5.1 A candidate

- **A landing is one change's, in one repository,** in v0.1.0 (section
  14): a project with several repositories proves and lands each on its
  own.
- **A parent and an ordered list of landings,** each pinned at its head,
  prepared once into one exact commit whose history holds them. Its
  identity is the parent, the pinned heads, the merge commits the
  prepare made, and the prepare's tooling at its revision.
- **What is proved is what lands:** the prepare's commits are made once
  and travel to the provers and to the turn as they are; nothing is
  merged, rebased or amended again after proof.
- **On the forge, on temper's branches:** a candidate of several
  landings lives on a branch of temper's, `temper/<root>/c<n>`, made by
  temper and deleted once the candidate is decided, so that any runner
  the forge dispatches can fetch it (gap L3). A landing of one may keep
  the forge's rebase of its own branch (`landing.md`, 5.2). A prover
  reached outside the forge may take the same commits as a bundle.
- **Pinned heads:** a landing joins a candidate at the head it had when
  selected, and its branch's name is reserved across every live
  candidate. A head that moves later enters only a later candidate, once
  the one it was pinned in is decided; it does not send the live
  candidate back to prepare (an exception to `landing.md`, 5.1).
- **A conflict is staged with its attempt:** a conflict the prepare
  meets is evidence of that attempt, adjudicated (a resolution, a
  rejection) only once the attempt's parent is known good; if the
  attempt errors or is invalidated, its pending conflicts are dropped
  and the member is prepared again.
- **The prepare's tooling may write** what the project's rules make from
  the members (regenerated files, lowered counted baselines that refuse
  a raise) as commits of the candidate's own, named for what they are;
  no member's branch carries them.
- **Merge or rebase:** a candidate of several landings is made by
  merging each member's head onto the parent in order, which forces no
  push and keeps each member's commits as they were; a landing of one
  may keep the forge's rebase.

### 5.2 Batches

- **Side by side while proofs keep up:** `landing.md`'s model,
  unchanged: each landing proves at its candidate; the turn
  fast-forwards the first proved; the others' stale proofs keep what
  still holds (their unchanged keys).
- **One candidate when they don't:** a project whose turns make more
  proofs stale than finish, or whose ready landings wait longer than a
  proof (measures the serializer keeps), groups ready landings into one
  candidate, first ready first, up to a batch size.
- **Suspects first:** a red names its failing gate and key, and only the
  members whose changes reach that key are suspects. The others are
  prepared again without them and land, running only the keys the
  suspects' absence changed; a single suspect's red is its own, with no
  split.
- **Several suspects are bisected by halves,** the left half first, each
  half prepared again from the parent it will land on (after a green
  left half, on that half's candidate), recursively, until each red is
  one landing's: that landing's red is its own (a repair), and every
  green member lands. A conflict in preparing a batch leaves that member
  out and the rest goes on.
- **Members stay their own landings:** a batch is a grouping of the
  serializer's, not a landing; each member keeps its own task, news and
  outcome, so one member's red rejects only that member.
- **The size is measured, not fixed:** larger batches land more per
  proof while most are green and cost more rounds per red (2.1); the
  serializer keeps the red rate and the rounds spent bisecting, and the
  batch size follows them.

## 6. Provers and runners

- **A prover runs one candidate's gates at a time, with the machine to
  itself:** a measured gate is a measurement, and a congested prover
  makes tests starved or slow that are neither (`testing.md`; skein's
  `testing-strategy.md`, 2.10). Nothing else builds there.
- **Any certified runner proves a deterministic gate:** a workspace's
  check job (`code-pipeline.md`, 3), a forge's runner, or a prover; the
  receipt is the same, and names the runner (section 4).
- **A forge's runner or the project's own,** reached the same way: by
  dispatch (`landing.md`, 7.1). A project's own prover is a self-hosted
  runner with a label of its own, one per machine, which fetches the
  candidate from temper's branch (5.1).
- **Dispatch is recorded before it starts, and starting is idempotent:**
  the serializer persists the request (attempt, candidate, runner)
  before asking the runner to start it; asking again for the same
  attempt answers that attempt's state, never a second run. A runner's
  state for an attempt is running, ended (with its result) or unknown;
  unknown is never read as idle.
- **A runner answers with a certificate,** accepted only when it matches
  the request in full: the generation, the attempt and the runner; the
  parent, the pinned heads in order and the prepare's merge commits; the
  policy and the selection; every gate expected and its result, with its
  complete log returned to the serializer's own storage; the checkout
  clean before the checks and unchanged after them (for a workspace's
  check, before its merge and after its restore). An empty plan is
  accepted only where the request expected one. Anything else is a
  fault.

## 7. Outcomes and who hears them

| Outcome | What it is | What follows | Who hears it, by default | For example |
|---|---|---|---|---|
| green | every blocking gate holds at the candidate (advisory gates reported only) | the turn (section 8) | each member's goal: its landing, with the commit | a batch landed |
| red | a blocking gate's own step failed (a measured one again on its rerun), and no fault was recognized | a batch's suspects are found and bisected (5.2); one landing's red is a repair of the failing member (`landing.md`, 7.3), its brief the failing step and the log | the failing member's goal | a lint, a test, a test over its CPU budget, a raised count, a tools' suite's nonzero exit or timeout |
| conflict | the prepare could not settle a member | conflicts the project's rules know are settled by its tooling; any other leaves that member out for a resolution (`landing.md`, 3) | the member's goal | a counted baseline recounted; a textual conflict to resolve |
| fault | the runner's, not the change's: set-up failed, a refusal for capacity, a missing tool, a transport or certificate failure, a crash | retried with nobody rejected and no member marked tried, up to `gate_retries`; past it the landing is held (`landing.md`, 7.4) | the chat, once per episode | a disk below its floor; a runner restarted without its tools |
| starved | a test that the prover starved of time it was owed | neither a pass nor a fail: recorded apart in the receipt, the gate's verdict unchanged by it (skein's `testing-strategy.md`, 2.10) | recorded; the chat when it recurs past its bound | a timed story on a loaded machine |
| stale | at the turn, its parent is not main's tip | prepared again; certified keys keep their receipts | nobody | a main moved by another landing |
| key gap | a full proof fails where selections passed | a red as any other (bisected in full); the missing input joins its key | the chat, beside the red's own | a test reading another package's files |

- **A fault outranks a red:** where a gate failed and the runner also
  recognized a fault (a refusal for capacity, a lost runner), the
  outcome is the fault. A suite that ran and failed, or timed out, is
  red; a test step whose selection was empty is green.
- **A red main already has is main's:** where a full proof of main left
  a key red, a candidate's failure in the same tests is not its
  members': nobody is rejected, its landings return to the ready set,
  and the chat hears it, until a repair turns main's key green.

- **A measured failure is rerun once before it is red:** where its
  failed tests are few and each passed in every member's receipt, they
  run again, alone, on the same prover at the same candidate, before any
  bisection. Failing again, it is red; all passing, it is a **flake at
  the gate**: nobody is rejected or marked tried, the candidate goes
  again, and the flake is counted on its finding (10.1). A tip with two
  such flakes has its next red stand (gap L23).
- **Whose a red is, is judged:** a verdict task reads the failing gate
  and log and says fix (a repair of the change, with what to change) or
  retry (not the change's: the same tip goes back, once). A conflict
  needs no verdict: it is a resolution, by code's rule.
- **The routing is a table** (`code-pipeline.md`, 2.7; section 13): a
  project may also tell the goal above the member of a red, or the
  person's chat of a fault that holds a landing.

## 8. The serializer and the turn

- **One per project, the only writer of its mains:** `landing.md`'s
  turn, held by one serializer that alone prepares candidates,
  dispatches them, adjudicates certificates, lands and reports, per
  repository. Runners never write the forge's refs or report. One
  authority lock covers every mode that can move a main (proving, a
  restart), and a mode refuses to start while another's state or a
  runner's checkout is unresolved.
- **It lands a proved candidate only,** after reading main afresh: main
  must equal the certified parent, then main is fast-forwarded to
  exactly the certified commit. No force, no merge at the turn; a
  fast-forward check alone is not enough, since main might be another
  ancestor of the candidate.
- **Decisions before effects:** the serializer's state is versioned (a
  version it does not know is refused, never guessed) and holds the
  requests, the reserved names, the certificates, the invalidations, the
  bisection worklist and each decision under a stable id with a mark
  once applied, so an accounting step is applied once. A landing is
  written as an intent before its push.
- **Effects at least once, through an outbox:** each effect (news, a
  status, a branch's release) is written as pending, delivered, then
  marked delivered; an uncertain delivery is sent again under the same
  id, and its reader may see it twice. A failed delivery never changes
  an outcome. A branch is released by a deletion that names the head it
  landed (an expected-head lease), so a head pushed since, or a branch
  made again, survives; a failed release stays pending work.
- **Recovery reads main and the intents:** after a crash, under the
  lock, the serializer reconciles each runner first, then each landing
  intent in order: main at the candidate means the landing happened, to
  be finished once; main at its parent means it did not, to be tried
  again under the same decision; main anywhere else stops promotion
  until it is explained. A runner's result for an invalidated candidate
  is ignored, and a runner that cannot be reached stays unavailable.
- **Draining is a hold on admission** (`code-pipeline.md`, 2.4): it
  closes admission (no new candidate, no new bisection group), lets
  admitted proofs end in order and decides them, persists any recovery
  work left, and only then may the serializer's tooling or the runners'
  checkouts change; a drain that times out stays in force. Restarting
  resumes from the persisted state, with the settings it had.
- **Nothing runs inside the turn:** no gate, no prepare (`landing.md`,
  8). The turn is seconds.
- **What it records serves a project's measures:** per landing, its
  ready, admitted, proved and landed or rejected times, its outcome and
  class; per candidate, its members, each gate's duration and CPU time,
  and its bisection's rounds; per fault, its cause and the minutes lost;
  with the tiers' lag and coverage (section 9) and the goals' own times
  (`code-pipeline.md`, 2.1). A project's standing report is computed
  from them (gap L19), never kept by hand.

## 9. After landing

- **A tier** is a check too slow or too broad for a gate (an exhaustive
  profile, a browser suite, process stories, a QA suite): a measured
  gate defined with its key and its report (its failing tests by id, in
  a declared format), run on a runner of its own, never on the
  serializer's provers (gap L13).
- **Tiers follow main, not landings:** when main's tip moves, by temper
  or by anyone, its tiers run on it, one run per tier at a time; when a
  run ends, the next takes the newest tip, so one run covers every
  landing since the last; while more tips wait than the project's bound,
  the older are coalesced into the newest and the coverage lost is
  recorded. A tip whose key has a receipt is judged by it, with no run.
- **The verdict:** a failed test runs again, alone, at the same commit.
  Failing again it is red; passing, a flake, counted on its finding
  (10.1); unable to run, a fault, retried and the chat told. Failures
  the previous run had are standing, not new.
- **New reds are grouped** per tip and suspect set, the suspects being
  the landings since each test last passed. The project may bisect them
  over the range, within a bound of time and runs, to the landing whose
  commit turned them red; or "tell all" suspects, a verdict task reading
  the evidence and naming the owner or saying it is not the suspects'
  (fabrig's choice), which is news to the chat.
- **Where a red goes:** to each suspect landing's goal as news
  (`code-pipeline.md`, 2.7), with the tests, the log's end and the
  bisection's or verdict's answer; the goal's agent decides the repair,
  a change from main that reproduces it in the lowest tier first. A red
  no open goal owns is news to the chat its template names. Standing
  reds are recorded and reach nobody; a recovery goes where its red
  went, so a fix is confirmed green by the same checks; a new flake's
  finding is routed as findings are (10.1), and a flake seen again is
  counted on it and reaches nobody. A project may send a red to the
  landing's goal as well, or hold admission into that repository until
  the red is repaired (`code-pipeline.md`, 2.4). A covering run that
  still has the test makes it unfixed, news again.
- **Cleared, where a project asks for it** (gap L14): a project may hold
  each change's result until its tiers clear it (a run whose range holds
  it blames it nothing), up to a verification bound; its goal then ends
  only once its work is verified, and a red before then goes to it as a
  repair. Off by default.
- **No test twice on a tree:** a tier's key with a receipt is not run
  again; a bisection's runs are on other trees.

## 10. Findings and QA

### 10.1 Findings

- **A finding** is something found that is not the finder's to fix now:
  a QA case that failed, a flake, a red its landing's goal no longer
  holds, a review's follow-up.
- **A record in temper's store** (gap L15): its kind, severity and area;
  its reproduction and commit; its signature (the failing tests' ids and
  the failure's first line, where it has one); its state (`new`,
  `routed`, `backlog`, `asked`, `merged`, `dropped`, `closed`) and who
  holds it. `temper ctl` shows it.
- **Matched by signature:** a flake seen again is counted on its
  finding, not filed again, whether a tier's or a gate's that passed on
  its rerun (section 7).
- **Routed by role:** to the goal its landing's change or its area names
  (`code-pipeline.md`, 2.7), whose agent triages it: to a repair, to a
  new goal, to its backlog, to the person as a question, merged into
  another, or dropped with a reason; the chat its template names takes
  what has no goal.
- **Closed** when a landing whose change names it lands, or clears where
  cleared is on (section 9); a QA finding, when its rung next passes
  (10.2). A verdict task may say a finding is not the code's: news to
  the chat.
- **On the forge, optionally:** a projection of findings as issues,
  written and never read back as state, is a project's setting, off by
  default; so is a backlog rendered into a document of the project's
  (`backlog_document`).

### 10.2 QA

- **A ladder is a goal** whose rungs are walk tasks, each a story on a
  backend, ordered by dependencies (a rung after the rungs below it). A
  rung's brief is its walk task's spec; an **edition** is a revision of
  that spec, and "rerun unchanged" is a walk at the same edition (gap
  L16).
- **A walk** is an agent task under a tester charter, on a build of a
  named commit, holding the accounts its backend needs for its length
  (exclusive, returned at its end, never copied). It exercises the story
  as a user would, writes down what it expects before each case, and
  reports a verdict (pass, fail, void) and each finding with its
  reproduction, naming an open finding it met again. A walk has a cap.
- **A rung's state** is its tasks': passed, walking, waiting on a lower
  rung, holding on an open finding or a person's action, or due (a first
  walk, or a rerun once its findings' repairs have landed). It passes
  when a walk at its edition passes after an earlier one of that
  edition, nothing holding it; a failed walk reopens it.
- **Triage is the ladder goal's agent,** at once for a blocking finding,
  else at a count or an age (section 13). A brief at fault is a question
  to the person, since an agent must not weaken its own acceptance; on
  the answer, the rung's spec is revised, a new edition.
- **Marks and passes** (Later, section 14): a landing marking what a
  user can now do with a `QA:` trailer, and a pass on the newest green
  main walking the marked landings (gap L17).

## 11. Each forge, by what it offers

| Step | temper's own (every forge) | GitHub native | Forgejo native |
|---|---|---|---|
| Prepare a candidate | the forge's rebase for a landing of one; a merge chain on a branch of temper's for a batch or a train; the project's tooling (`landing.md`, 5.3) | the same | the same |
| Prove | dispatch-only gate workflows, receipts by key (`landing.md`, 7); a project's own provers as self-hosted runners | the same; the merge queue's `merge_group` runs would be gate runs temper did not dispatch, so a queue-backed turn reports receipts already made | the same |
| Speculate (a train: Later) | temper's own: candidates on temper's branches, landed in order | the merge queue builds each group on the groups ahead of it: a train of GitHub's sizing | none: temper's own |
| The turn (check base, then land) | read main, fast-forward it (`landing.md`, 8): Forgejo by a fast-forward merge, GitHub by a ref update with an `Always` bypass | the merge queue: temper enqueues a proved candidate; its required check is temper's status, already met; GitHub merges the group it built, fast-forward | none: temper's own |
| Show it | the candidate on its pull request, `temper/gate/<name>` statuses (`landing.md`, 10) | plus the queue's "queued" and position | the same as temper's |
| Batching | 5.2 | the queue's groups, sized by GitHub: temper caps them by its own batch | temper's own |

- **GitHub's queue is worth it for its display and its protection**
  (required checks, merge rules a person already trusts); it costs that
  GitHub sizes the groups and trains, that landing goes through pull
  requests, and that its runs are not temper's dispatches. A project
  chooses it in its landing settings; temper's turn stays the default
  until a project's queue-backed turn has been walked on real backends.
- **A queue-backed turn lands only what temper proved:** temper reports
  success on a group only when the group's commit, parent and ordered
  members match a certificate of temper's; otherwise temper proves the
  group GitHub built (its keys, its judgement gates) before reporting.
- **Forgejo has no merge queue** (its "merge when checks succeed" is a
  scheduled merge, not a queue): temper's turn is the only one.
- **Members' pull requests are marked merged by containment** once main
  holds their heads: GitHub always; Forgejo with "autodetect manual
  merge" on, which adoption checks (gap F4).

## 12. fabrig's landing pipeline in the model's terms

What fab does today, its term in the model, and where fab departs
(**gap**: by hand, by an agent where the model has code, or temper's
data in the repository; temper does not copy it).

| fab | The model | Where |
|---|---|---|
| `ready/<draft>/<task>` refs (gap: temper's data in the repository) | a change ready, a record in the store | `code-pipeline.md`, 3 |
| precheck's receipt, by tip and main, reused for a batch of one | receipts by key, fresh by key | 3, 4 |
| the merge queue on the gate machine: batches of up to 8, oldest commit first, `git merge --no-ff` per member | candidates, a batch, prepared by a merge chain | 5 |
| the old-forms recount at a batch | the project's prepare tooling | 5.1 |
| selection: reached packages, reverse dependencies, always-run lists; a docs batch runs a touched tree's counted baseline; every fifth batch in full | keys per package; repository gates; full proofs | 4 |
| `heavy --gate`, the gate lane | a prover's exclusive use | 6 |
| a collateral red: up to 5 failed tests rerun alone, passing in every member's receipt; 2 per tip | a measured failure rerun once; a flake | 7 |
| a red batch halved, left first | bisection | 5.2 |
| a batch `error` (disk, a refused push) | a fault | 7 |
| a refusal foreman's fix or retry; `fab queue untry` | a repair, or the tip again, by a verdict task | 7 |
| `git push` expecting the parent; ready and source refs deleted under a lease | the turn; branches released by expected-head deletions | 8 |
| `fab after-landing` on its own slot: the newest tip, its suspects, reruns alone, new, standing, unattributed, recovered; faults; starved never red | tiers following main | 9 |
| no bisection: a red foreman reads the suspects' specs and names the owner; a repair task from main | "tell all" and a verdict naming the owner; the goal's repair | 9 |
| a red with no fab task among its suspects, a fault: signals on the page, read by a session (gap) | news to a chat | 9 |
| the repair goes to the newest suspect's plan, whatever owner the foreman names (gap: code doing a judgement) | the owner's goal | 9 |
| a fabrig plan retires after its verdict | a goal ending after its tiers' verdict | 9 |
| the QA ladder plan, rungs and editions (brief files in the repository: gap), walks by a session (gap) | a ladder goal of walk tasks, spec revisions | 10.2 |
| findings in the log, merged by the walker or triage; a triage call | findings in the store, triaged by the ladder goal's agent | 10 |
| the backlog file, readied (and never taken by the queue: a fault) | the backlog as findings; a rendered document, optional | 10.1 |
| `fab land` for fab's own repository: a rebase, the whole suite, code-style gates | a candidate of one by rebase, its gates a setting | 5, 13 |

## 13. Settings

| Setting | Scope | Default | fabrig's |
|---|---|---|---|
| `before_ready`: `keys`, `advisory` or `none` | project | `keys` | `advisory`: fab readies a tip without a receipt, and the queue checks it in full |
| `tests_before_ready`: advisory, or receipts | project | advisory | receipts |
| gates: class, ignored paths, blocking or advisory | project, repository | the repository's adoption | counted baselines, formatting, lints, the build deterministic; the default and fuzzy tests measured |
| keys: by the project's tooling or by tree | project | by tree | per package, from cargo |
| prepare: tooling; merge or rebase | project | temper's rebase; merge | `old_forms.py --write`; merge |
| a candidate's size | project | by measure | 8 |
| full proofs: every N candidates, hourly, idle after | project | 5, 1 hour, 10 minutes | every 5 candidates only |
| a measured failure's reruns; flakes per tip before a red stands | project | 1; 2 | 1 (up to 5 tests); 2 |
| `gate_retries` | project | 2 | — |
| tiers: per repository; coalesce beyond N waiting | repository | none | the browser tiers of jig, temper and skein, temper's process tier, live skein; the newest tip |
| a tier's red: bisect within a bound, or tell all with a verdict | project | bisect | tell all, a verdict naming the owner |
| a tier's red goes to | project | the suspects' goals | the suspects' goals; with none open, the chat |
| cleared: a goal waits for its tiers | project | off | on |
| findings on the forge; a backlog document | project | off; off | off; on |
| QA: a ladder goal, its backends; triage at a count or an age | project | none | one ladder: ChatGPT with an owned Forgejo or GitHub, Claude Code with an owned Forgejo; 4 findings or 30 minutes |
| fab's own repository | repository | — | candidates of one by rebase, its whole suite under 120 s, code-style gates, no check before ready |
| a forge's merge queue backs the turn | project | no | no |

## 14. Later

- **Trains** (revision 4's 6.3): candidates proved on candidates, with
  invalidation; fab dropped its train at the user's word (10-31).
- **A repair first, fairness and the reservation, and landings across
  repositories** (revision 4's 6.4, 6.5, 6.6).
- **Machines** (the user, 11-01): provers warm by affinity, how many
  provers.
- Keys per package for languages other than Rust; partitioned full
  proofs; deeper trains; elastic provers.
- QA marks and passes on landed changes; a passed rung's story joining
  the live tier.
- Findings as forge issues by default; findings matched by a normalised
  key; for fabrig, bisection after landing when verdicts name the wrong
  owner.

## 15. What temper has today, and what to build

### 15.1 The code (fabrig 94c0f7648, 2026-10-30)

- **Landing by turn, on Forgejo, partly built,** opt-in by `landing.by =
  turn` (the default is `queue`,
  `temper/crates/temper-service/src/startup/application/landing.rs:417`;
  `landing.md`, 13.1): the landing as a procedure task the deployment
  makes; members joined across repositories and goals by a dependency
  amendment, at most one per repository; the forge's rebase as prepare
  and a conflict as a resolution; keys from the forge's tree reads,
  without ignored paths
  (`temper-engine-domain-forge-landing/src/key.rs`); receipts in
  families 23 `Landing` and 24 `Receipt`, the oldest evicted but main's;
  dispatch-only gates with `temper_commit` and `temper_key`, runs found
  by polling and followed, a runner's fault retried twice then held;
  `temper/gate/<name>` statuses; the turn: first proved, bases checked,
  fast-forward-only merges upstream first, partial landings kept;
  restart mid-proof; `ctl landing show` and `landing list`. GitHub's
  turn is refused.
- **The `queue` way** (`forge.md`, 8 to 10), the default: CI waited on
  at each pushed head, queues per repository and base, aging, a red base
  paused for one repair, updates by the forge's merge, a merge in the
  repository's style; on GitHub, the forge's merge queue after a refused
  merge.
- **The goal issue projection** (`forge.md`, 12;
  `temper-engine-domain-forge-issues`): goals shown as issues.
- **jig:** owned stop holds over a subtree, the owner's rule by role
  rank, a run's proposal of a release waiting for the owner (J1); a
  question to the requester keeping its asker until answered; person
  answers, which temper turns on, reaching the asker directly; a move to
  a party carrying open questions.
- **skein's forge client:** on Forgejo, workflow dispatch with inputs,
  rerun of failed jobs, status writes, a fast-forward-only merge at a
  head, tree reads, and Actions statuses resolved to their job attempts;
  on GitHub, status writes, the merge queue's enqueue and `merge_group`
  decoding, and refusals of tree and run reads, dispatch, rerun, the
  rebase update and the fast-forward merge. No artifact read and no ref
  update on either.
- **Nothing yet** of batches, trains, candidates on temper's branches,
  the prepare and key job, keys per package, gate classes, provers and
  certificates, the serializer's durability, priorities, the checks
  before ready, workspaces as claims, leases, after landing, findings,
  QA, trailers in news, holds beyond J1, routing by role, handovers or
  successors. `Check::Tier` and its receipt codecs exist with no user.

### 15.2 Keep

- **The landing task and its phases** (prepare, prove, turn), with one
  member; the turn's fast-forward-only merge at a head, and its base
  check.
- **Receipts by key** (families 23 and 24, flights per key, main's key
  never evicted) and **the tree key** as the default (section 4); its
  version byte leaves room for keys per package.
- **Proof by dispatch:** run matching, the runner's faults and their
  retries, the run's classification
  (`temper-engine-domain-forge-landing/src/verdict.rs`) as the first cut
  of section 7's classes.
- **The forge's rebase** as the prepare of a landing of one (5.1).
- **`temper/gate/<name>` statuses, `ctl landing`, the turn-restart and
  owned-turn process stories.**
- **The goal issue projection,** a display.
- **skein's Forgejo calls; GitHub's status write, enqueue and
  `merge_group` decoding,** the last two for section 11's accelerator.
- **`Check::Tier`** in family 24's codec, for tiers' receipts (section
  9).
- **jig's J1** and its designs for J2 to J8.

### 15.3 Delete

- **Today's joining across repositories** (section 14, H9): the
  dependency closure and its upstream-first order
  (`temper-engine-domain-forge/src/landing.rs:169` to 271), the root's
  join by the amendment
  (`temper-engine-domain/src/route/forge_route/landing_route.rs:118` to
  137), `kept_landings` (`:227`) and the partial turn across
  repositories. The rule of one change per repository in a landing
  (`landing.rs:236`;
  `temper-engine-domain-forge-landing/src/domain.rs:18`) goes with it: a
  landing is one change, and batching groups landings. The designs of
  pin-only members, `with`, stacks and pins kept at main, with their
  settings `dependencies_join` and `pins` (`landing.md`, 4.1, 5.3,
  13.1), move to Later (section 14).
- **The `queue` way** (`forge.md`, 8 to 10), once the turn lands every
  project, GitHub's included: its states (`Checking`, `CancellingGates`,
  `Queued`, `First`, `Updating`, and the change's own `Landing`;
  `temper-engine-domain-forge-change/src/boundary.rs:157`), its CI and
  semantic repairs, its ranking and `queue_window`, the merge-style
  update and the merge in the repository's style, and `landing.by`. It
  is woven through the change procedure, the forge's top and the
  simulation world: rework, not a module's deletion.
- **Findings as forge issues by default,** their labels read back, the
  forge's closing keywords, and questions projected as issues: to a
  setting that is off (10.1) or to Later.
- **A change that lives to cleared by default** becomes a setting, off
  (section 9).
- **The drafts `process-model.md` and `porting.md`,** carried here.

### 15.4 What to build

Revision 4's gaps that the landing pipeline keeps, and new ones (L23
on). Trains (L2), a priority (L10), the reservation (L18) and marks
(L17) are Later (section 14).

| Id | Gap | Grade | Changes |
|---|---|---|---|
| L1 | Batches: a candidate of several landings, suspects by key, bisection by halves, the size by measure | design | `landing.md`, 1, 4, 9 |
| L3 | Candidates on temper's branches by a merge chain; the prepare's own commits; pinned heads and reserved names; conflicts staged with their attempt [T5] | conflict | `landing.md`, 5 (a moved head no longer sends a live candidate back) |
| L4 | The prepare and key job on a worker: the project's tooling, no model, no credentials [T2] | design | `landing.md`, 5.3, 6.1; `worker.md`, 4.4 |
| L5 | Keys per package and repository gates' keys, versioned; receipts kept from discarded candidates; what a key cannot see; full proofs that audit [T3] | design | `landing.md`, 6.1, 6.2 |
| L6 | Gate classes: deterministic receipts from any certified runner, measured ones from provers; the environment in the key | design | `landing.md`, 6, 7 |
| L7 | Runners: idempotent dispatch, certificates, faults before a gate [T4] | design | `landing.md`, 7 |
| L8 | Outcomes: a measured failure rerun once before red, starved recorded apart, key gaps, faults outranking reds, a red main already has [T9 in part] | design | `landing.md`, 6.2, 7 |
| L9 | The serializer's durability: one authority lock, versioned state, decisions applied once, intents, the outbox, recovery by main, drain as a hold [T6] | design | `landing.md`, 8, 11, 12, 15 |
| L13 | Tiers following main: one run at a time, the newest tip, coalescing, the rerun, bisection within a bound or a verdict naming the owner, reds and their recoveries routed to the suspects' goals as news, unfixed, a red no goal owns to a chat [T8, T9] | design | `after-landing.md` |
| L14 | Cleared as a setting: change results held to a verdict, the bound [T10] | design | `after-landing.md`, 6 |
| L15 | Findings as temper's records, matched by signature, routed and triaged; the forge projection a setting [T11] | conflict | `findings.md` (the store moves from issues to temper) |
| L16 | QA: a ladder goal of walk tasks with dependencies, editions as spec revisions, a tester charter, accounts held for a walk, rung states and the pass; marks and passes Later [T12] | design | `findings.md`, 8 |
| L19 | Views and acts in `temper ctl`: candidates, batches, receipts, tiers, findings, holds; the records a standing report is computed from (`code-pipeline.md`, 2.1; section 8) [T15] | design | `landing.md`, 14 |
| L20 | A landing is one change in one repository in v0.1.0; joint landings across repositories Later (section 14) | conflict | `landing.md`, 4.1, 5.3, 8, 13.1 |
| F1 | skein: read a run's artifacts; update a ref fast-forward only [B1] | design | skein's `forge.md`, 3.2 |
| F2 | GitHub's turn: tree and run reads, dispatch, rerun, the ref update under `Always` [B2] | design | skein's `forge-github.md`; `landing.md`, 8 |
| F3 | GitHub App permissions: statuses and actions written, the `Always` bypass, `workflows` [B3, B5] | conflict | temper's `protocol/setup.md`, 17.1 |
| F4 | Adoption checks: dispatch-only workflows, Forgejo's "autodetect manual merge" [B4] | design | `landing.md`, 13.2 |
| F5 | GitHub's merge queue backing the turn | design | `forge.md`, 9.1; section 11 here |
| L23 | A flake at the gate bounded per tip; a verdict task on a rejection (fix or retry) | design | `landing.md`, 7; `findings.md`, 5 |

## 16. Open questions

- **H1, who makes a receipt:** any certified runner for a deterministic
  gate, a prover only for a measured one (recommended), or provers only.
- **H2, where keys are computed:** by the prepare job on a checkout,
  trusted and audited (section 4: recommended), or by the engine from
  forge reads.
- **H3, where a candidate lives:** on temper's branches at the forge
  (5.1: recommended), or only on the provers, as bundles.
- **H9, today's joining across repositories:** deleted now, its designs
  moved to Later (recommended), or kept dormant.
- **A gate per step or one gate of steps** (recommended: a gate per step
  whose result a key can carry); **the batch's size** (fixed, or by
  measure); **GitHub's merge queue and dispatch-only gates.**
