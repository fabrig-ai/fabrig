# The landing model

Draft, 2026-10-30, revision 4 (one spec: the work around a landing, the
checks before ready, after landing, findings and QA, and what temper has
today). The north star for how temper runs a project's development: one
model, generic over forges and projects, expressive enough that fabrig's
own pipeline (its coordinators, leads and implementers, its merge queue,
its tiers after landing and its QA) is one of its configurations, and
the forge's native machinery serves a step only where its semantics
match. It supersedes `process-model.md`, whose tree of work section 3
carries, and `porting.md`, whose mapping section 13 replaces. It records
conversations with the user on 2026-10-29 and 2026-10-30, and is kept
current as fabrig's pipeline changes (the user, 2026-10-29). The model is
the target, whatever temper or fabrig runs today: section 13 says what
fabrig runs, section 15 what temper's code has and what the model asks
of temper's and jig's designs. Its decisions are the user's; section 17
lists the choices of primitives still open.

Unqualified, `landing.md`, `after-landing.md`, `findings.md`, `forge.md`,
`worker.md` and `testing.md` are temper's `../domain/` documents; jig's
are named as jig's.

## 1. In one page

- **One model:** ready landings are **prepared** into candidates,
  **proved** on runners into receipts kept by key, and **landed** by one
  serializer per project, fast-forward only, at exactly the commit that
  was proved. Batching, speculation, priority and the number of provers
  are only how many candidates are in flight, in what order and where
  they prove; they never change what proves a commit or who lands it.
- **Mechanisms are temper's and jig's; choices are settings.** Who hears
  an outcome, whether a goal waits for its tiers, where a finding goes,
  how a repair is let through, how much is checked before ready: each is
  a project's setting over one mechanism (section 14), which gives
  fabrig's values beside the defaults.
- **temper's landing is the contract.** What lands, when, what proves it
  and what a failure means are temper's, never a forge's queue's. A
  project moving between GitHub and Forgejo, and temper's own tests,
  meet one behaviour.
- **The work is a tree of goals** (section 3): the person and their
  chat, coordinators, the goals that own an area of the code for a part
  of a design, and their delegates. Holds have owners, modes and scopes;
  a path has one writer across goals; workspaces are claimed and lent;
  a long-lived goal hands over to a fresh run of itself; news goes to
  roles in the tree.
- **Proved before ready, by the same keys** (section 4): a change's
  deterministic gates (formatting, lints, builds, counted baselines, the
  tools' suites) run in its own workspace, at its head merged onto main,
  before it is ready. Their receipts are the candidate's too, so a
  candidate runs only the keys that main or the other members changed
  since. Freshness is key equality.
- **Keys per package, so selection is receipts** (section 5): a change
  re-proves only the packages it reaches, safely by construction;
  receipts outlive candidates; full proofs audit the keys.
- **Candidates carry landings: one, a batch, or a train** (section 6),
  each landing one change in one repository (landings across
  repositories are Later's, 6.6), in readiness order, with a repair let
  through first, alone and in full, when the project's operator says
  so.
- **Provers are machines with exclusive, warm use** (section 7) for the
  gates that measure (tests); any runner temper dispatches and certifies
  may prove a deterministic gate.
- **Every outcome is classed before it is anyone's** (section 8), and
  routed by role: a red to the change's goal, a fault to the project's
  operator, a suspected key gap to both.
- **One serializer per project** (section 9): the only writer of its
  repositories' mains, its decisions persisted before their effects, its
  effects at least once under stable ids, its recovery decided by
  reading main.
- **After landing, tiers follow main** (section 10): one run at a time
  on the newest tip, a failure rerun alone, new reds bisected within a
  bound and sent to their suspects' coordinators. Whether a goal waits
  for them is a setting.
- **Findings are temper's records** (section 11), routed to a
  coordinator who triages them. QA walks exercise a build on demand; QA
  passes exercise what landings marked.
- **A forge's native queue is an accelerator, never a dependency**
  (section 12). Switched off, nothing changes but the forge's display.
- **fabrig's pipeline is one configuration** (section 13): each of its
  operations has its term here, and what fabrig runs live, has built or
  has only designed is marked.

## 2. What fabrig's pipeline showed

### 2.1 The queue's first night (10-28 22:13Z to 10-29 02:10Z)

- **The cost is builds, not tests:** a full check was 7 to 11 minutes,
  of which tests ran about 41 seconds; building and linking about 1,366
  test executables was the rest, until each package had one (then a
  warm full check took 2.5 to 3.5 minutes).
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
- **Throughput doubled, then the queue saturated:** 23 merges an hour
  at midnight, 47 an hour by 01:08Z with about eleven implementers at
  once, 41 to 47 an hour after, with about seventeen; the queue was
  busy 96 to 98% of the time and the ready branches waiting grew from
  36 (01:21Z) to 58 (02:10Z). The builders were not the limit: builds
  waited a median of 2 to 7 seconds for a slot in the builder's build
  cap, and averaged about two of eight cores. So capacity moved to the
  queue: batches grew from 8 to 16 (01:34Z), and a builder was drained
  to become the second prover (01:30Z to 01:56Z).
- **A red costs a big batch most:** one member's lint failure in a
  batch of 16 took five bisection rounds before the good members
  landed. Bisection keeps the good members, but every round is a full
  proof; the train (6.3) overlaps proofs, it does not shorten them.
- **The checker's own rules break it as the tree changes:** a selector
  that names a test binary fails the whole run the day the last such
  binary goes (nextest refuses a `binary()` matcher that matches
  nothing); selection by test names that a refactor cannot empty
  stays valid.
- **Starved is not the only congestion:** a CPU-bound test took 2.5
  times its idle CPU on the loaded prover with no time spent waiting
  for a CPU (shared cores, cache, clock), which no runnable-wait rule
  sees. The remedy was the testing pyramid's own: the costly story
  moved down a tier, where it belongs (`testing.md`, 2.4).

### 2.2 The days after (10-29, 10-30)

- **Checks before ready pay for themselves:** since a red in a big
  batch costs whole proofs, each branch is merged with main in its own
  warm builder, the queue's deterministic steps run there (counted
  baselines, the tools' suites, formatting, lints, a test build), and
  it is marked ready only once they pass and main has changed none of
  the files they checked since. The builders had cores to spare (2.1).
- **Readers run tests, not only builds:** a change of what a shared
  item means, not only its signature, also runs the tests of every
  package that reads it, merged with main, before ready: a build alone
  missed it.
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

## 3. The work around a landing

### 3.1 The tree

- **The person** directs the work, sets budgets and priorities, and
  answers what is theirs to answer. **Their chat** is a coordinator: an
  agent conversation whose charter lets it make goals and steer them
  (jig's `domain/people.md`, 7). No hold over the project holds it
  (3.5).
- **A coordinator** is a goal that designs and coordinates goals of its
  own: from a draft it writes the design, cuts it into parts, and starts
  a goal per area for each part, at once for the parts that are certain
  while it designs the rest. Several coordinators run in one project at
  once, each on its own drafts. Depth is policy (authority's depth), not
  a fixed set of layers.
- **An area goal** owns one area of the code (a tree, a set of packages)
  for one part: it turns the part into changes, starts their producers,
  reads their reports, runs the checks before ready (section 4) and
  marks their changes ready; it hears their landings and repairs what
  comes back. It is short-lived: what outlasts it is in its notes and in
  the design, so a successor starts warm from them (3.6).
- **Delegates** are agents (producers, reviewers, investigators,
  advisors, testers), procedures (a change, a landing, a check, a QA
  pass) or people (a question, an approval). Each runs under a charter
  that names its model and effort; a delegate's charter is one its
  parent's authority allows.
- **Briefs, notes and results:** a goal starts from its brief, keeps
  notes while it works, and ends with a result that says what it did
  and what is left, which can be the next goal's brief.
- **Checks on a period:** a coordinator wakes on its delegates' results
  and on news, and a timer or a recurring task wakes it on a period as
  well (jig's `domain/tasks.md`, 3 and 8.1).
- **Standing rules are instructions:** waves that land an interface step
  first and then fan out one writer per package, no change across trees
  but in steps, a report's diff skimmed, an advisor after two misses.
  Such rules live in the charters, are judged by the agents that read
  them, and reach live tasks with a charter's new revision (jig's
  `MoveToNewest`, `domain/engine.md`, 17.4). Only what section 14 lists
  is enforced by the engine.

### 3.2 Workspaces

- **A workspace** is a warm checkout on a builder, with its own build
  cache, claimed by a goal for one change's life: the change is produced
  in it, checked before ready in it (section 4), and its readers' tests
  run in it; the claim is released once the change is ready or
  withdrawn. A check temper runs at admission (4.2) claims one of its
  own (gap J12).
- **A claim never expires.** A workspace whose use is uncertain (a run
  lost, a check's end unknown, a checkout not restored) stays claimed
  until it is reconciled; an absent process proves nothing.
- **Shares:** a project's workspaces are divided into shares, per
  coordinator or per area, as an authority limit; a goal claims from its
  share or from the free pool, and from another's share only by its
  holder's lend.
- **Housekeeping is the workspace's own:** stale build outputs are
  trimmed before a producer starts and when the builder's disk nears its
  floor, never while a build reads them.
- **A workspace is not a prover:** a builder runs several workspaces at
  once, so what it measures is not a measurement (section 7).

### 3.3 One writer of a path

- **A lease** reserves a path (a package, or a design document with its
  companions) for one goal for a wave, across goals and coordinators: a
  change that touches a path leased to another goal is refused at ready,
  unless the holder lent that path to the change (gap L12).
- **Waiters queue first come;** no lease expires or is taken over; a
  released lease tells its first waiter.
- **Within a goal,** its delegates divide paths by its own rule (one
  writer per package for a wave).

### 3.4 Asking

- **Any task may ask the person** if its authority lets it make a
  person task addressed to them. A question carries its options and a
  recommendation (an instruction, not a mechanism).
- **A question to the requester keeps the asker** until it is answered,
  and moves with it.
- **Questions to the chat stay with the chat** (gap J7): a delegate that
  asks its requester, the chat, is asking the chat, which answers itself
  or asks the person in its own words (gap J8). Where person answers are
  on, the person's answer goes straight to the asking task and the chat
  hears a notice.
- **An answer after its asker ended** goes to the asker's successor
  (3.6); with none, to its requester, as news naming the question.

### 3.5 Holds

- **A hold has an owner,** the party or task that placed it: only the
  owner lifts it, or a party whose role stands above the owner's; a task
  above a party's hold may only propose its lift, which waits in the
  owner's inbox (gap J1).
- **Two modes:** **stop** cancels runs at once and holds the tasks;
  **drain** lets running runs finish their turn and running changes
  finish, landing included, starts nothing new (no delegate made, no due
  task run, no new candidate admitted), and holds each task as it goes
  idle (gap J2).
- **Three scopes:** a task's subtree; a project, which tasks made later
  inherit and the person's chat is exempt from (gap J3); and a kind of
  procedure's admission in a project, such as the landing pipeline's
  (gap J9). A drain of admission lets the candidates in flight be
  decided and admits no new one; it is how the operator changes the
  serializer's tooling or provers, and what a priority repair may place
  after itself (6.4).
- **Parked until an event** (gap J4): a task-owned hold that lifts itself
  when a named event comes: a landing on a branch, a task's end, a time.
- **Pausing a coordinator's work** is a drain over its subtree, owned by
  the person: its runs end their turn, nothing new starts, its wakes
  wait; the person's chat stays live.
- **Lifting a project hold** lifts it from the project; a task that went
  idle under it is released only when its owner lifts that task too, or
  every hold at once, so goals can resume one at a time.
- **One task released under a standing hold:** the hold's owner may
  release a single task from a project hold while it stands (a revert's
  change, say); tasks made later still inherit it.

### 3.6 Handing over

- **A long-lived goal starts afresh from its notes** (gap J11): when a
  coordinator's or an area goal's context nears its charter's bound (or
  after a compaction), its run ends with a handover note, and its next
  run starts from its brief, its notes and the handover rather than its
  transcript. Its identity, address, subscriptions, claims and leases
  stay with it: nothing moves, and no second session warms up beside it.
- **A successor** (gap J6): a goal that ends can name a live goal as its
  successor, or its requester names one after it ended; the successor
  then receives its open questions' answers, its subscriptions and kept
  news, its claims and leases, and its notes by reference. Until one is
  named, its leases and claims stay held in its name. Its requester
  still hears its result.
- **A live goal moved under another task** (gap J5): anyone who may
  amend both makes the second the first's requester; its reservation
  moves to the new parent's funding, and its results, questions and
  escalations go there from then on. A part moved from one coordinator
  to another is this move.

### 3.7 Who hears what

- **News goes to roles in the tree,** resolved when the news is made
  (gap J10): the member's goal (the area goal that owns the change),
  the nearest coordinator above it, the project's operator, the person's
  chat. Which outcome goes to which role is the project's routing table
  (section 14); sections 8 and 10 give the defaults.
- **The operator** is the party or goal the project names for its
  machines, provers, serializer and tools: it hears faults, suspected key
  gaps, stuck holds and capacity refusals, and by default it alone sets
  a priority (6.4) or drains admission (3.5).
- **A delivery never changes an outcome:** news is an effect through the
  outbox (section 9), sent at least once; a role that resolves to nobody
  goes to the operator.

## 4. Before ready

### 4.1 A change's own gates

- **Ready** when its goal marks it so: its producer's report is in, and
  its own deterministic gates hold at its **own candidate**, its head
  merged onto main's tip, uncommitted, in its workspace (gap L11).
  Judgement gates (a review, a person's approval) are blocking gates
  like any (`landing.md`, 7.2): their verdicts may come before ready,
  and are met at the candidate.
- **Its own deterministic gates** are the project's gates of the
  deterministic class (section 5) over the packages the change reaches,
  and the repository gates whose paths it touches. They run as a check
  job in the workspace, a job beside `worker.md` 4.4's prepare and key,
  dispatched by temper like any gate; their results are receipts by
  key, made only from a certificate temper accepted (section 7).
- **Measured gates wait for the prover:** tests run before ready (the
  producer's focused tests, a change of meaning's readers' tests, chosen
  from the reverse dependency graph) are advisory there: reported, never
  a receipt, since a builder shared by several workspaces is no
  measurement.
- **What the check meets** is the change's: a conflict merging onto
  main is a resolution under its goal (`landing.md`, 3), a red a repair,
  a fault of the workspace the operator's, never the change's.
- **The workspace is left as it was:** the check's merge is never
  committed, and the checkout's sources, index and refs are restored and
  verified before the claim moves on; a checkout not restored stays
  claimed (3.2).

### 4.2 Fresh by key

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

## 5. Keys per package

- **A gate per package, or a gate whose key is per package:** a test
  gate's key for a package is a digest of the package's own inputs
  (its directory's tracked files but the paths the gate ignores), its
  dependencies' keys, the toolchain, the gate's definition and the
  workspace's shared inputs (the lockfile entries it resolves, the
  profiles). A package's key changes exactly when something its build
  or tests read changes.
- **Repository gates beside package gates:** a gate whose subject is
  the whole repository (a tools' suite, a counted baseline's ratchet,
  formatting of the changed files) has a key of its own: the paths it
  reads and its definition. It runs when its key changes, whatever the
  package selection says: a tools-only batch runs the tools' gate and
  no package's, a documentation batch may still run a tree's ratchet.
- **A candidate is proved** when every member's blocking gates hold at
  exactly its candidate: each test gate's keys by passed receipts, each
  judgement gate by its verdict (`landing.md`, 7.2). Only keys with no
  receipt run, as one dispatch with the set (one build for the set). An
  advisory gate is run and reported, and neither holds a landing nor
  starts a bisection or a repair.
- **Receipts outlive candidates, once certified:** a receipt is a
  key's, not a candidate's, made only from a certificate the serializer
  accepted (section 7). A candidate discarded later (a bisection half, a
  train's successor whose ancestor failed, a stale base) leaves its
  certified receipts good for every key they name, so its next
  preparation runs only the keys the change of parent reached. What an
  invalid or incomplete certificate carried is discarded whole.
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
  on the machine (tests and their time bounds), so only a prover makes
  its receipt. Either key names the environment: the toolchain, its
  targets and profile.
- **What a key cannot see** is named by the project and checked: a test
  that reads files by path or runs another package's binary joins the
  keys it depends on.
- **Full proofs audit the keys:** every N proof attempts (reds
  included), hourly, and when the provers have idled with main moved
  since the last, a full proof runs every gate whatever its receipts
  say. An idle audit proves main itself (parent and candidate both
  main's tip; nothing lands). A full proof that fails is a red like any
  other, bisected with every half proved in full; where the selections
  before it passed, the operator is also told of a suspected key gap.
- **Paths no gate reads** (documentation outside every package and
  every repository gate's paths, a gate's ignored paths) reach no key:
  a landing of them is proved on receipts alone (`landing.md`, 7.2),
  and runs no build.

## 6. Candidates: one landing, a batch, a train

### 6.1 A candidate

- **A landing is one change's, in one repository,** in v0.1.0 (6.6):
  a project with several repositories proves and lands each on its own.
- **A parent and an ordered list of landings,** each pinned at its
  head, prepared once into one exact commit whose history holds them.
  Its identity is the parent, the pinned heads, the merge commits the
  prepare made, and the prepare's tooling at its revision.
- **What is proved is what lands:** the prepare's commits are made
  once and travel to the provers and to the turn as they are; nothing
  is merged, rebased or amended again after proof.
- **On the forge, on temper's branches:** a candidate of several
  landings lives on a branch of temper's, `temper/<root>/c<n>`, made by
  temper and deleted once the candidate is decided, so that any runner
  the forge dispatches can fetch it (gap L3). A landing of one may keep
  the forge's rebase of its own branch (`landing.md`, 5.2). A prover
  reached outside the forge may take the same commits as a bundle.
- **Pinned heads:** a landing joins a candidate at the head it had when
  selected, and its branch's name is reserved across every live
  candidate. A head that moves later enters only a later candidate,
  once the one it was pinned in is decided; it does not send the live
  candidate back to prepare (an exception to `landing.md`, 5.1).
- **A conflict is staged with its attempt:** a conflict the prepare
  meets is evidence of that attempt, adjudicated (a resolution, a
  rejection) only once the attempt's parent is known good; if the
  attempt errors or is invalidated, its pending conflicts are dropped
  and the member is prepared again.
- **The prepare's tooling may write** what the project's rules make
  from the members (regenerated files, lowered counted baselines that
  refuse a raise) as commits of the candidate's own, named for what
  they are; no member's branch carries them.
- **Merge or rebase:** a candidate of several landings is made by
  merging each member's head onto the parent in order, which forces no
  push and keeps each member's commits as they were; a landing of one
  may keep the forge's rebase.

### 6.2 Batches

- **Side by side while proofs keep up:** `landing.md`'s model,
  unchanged: each landing proves at its candidate; the turn
  fast-forwards the first proved; the others' stale proofs keep what
  still holds (their unchanged keys).
- **One candidate when they don't:** a project whose turns make more
  proofs stale than finish, or whose ready landings wait longer than a
  proof (measures the serializer keeps), groups ready landings into one
  candidate, first ready first, up to a batch size.
- **A red batch is bisected by halves,** the left half first, each half
  prepared again from the parent it will land on (after a green left
  half, on that half's candidate), recursively, until each red is one
  landing's: that landing's red is its own (a repair), and every green
  member lands. A conflict in preparing a batch leaves that member out
  and the rest goes on.
- **Members stay their own landings:** a batch is a grouping of the
  serializer's, not a landing; each member keeps its own task, news and
  outcome, so one member's red rejects only that member.
- **The size is measured, not fixed:** larger batches land more per
  proof while most are green and cost more rounds per red (2.1); the
  serializer keeps the red rate and the rounds spent bisecting, and the
  batch size follows them.

### 6.3 Trains

- **Depth follows the provers:** with N provers, up to N candidates are
  in flight, each prepared on the one ahead of it: candidate k+1's
  parent is candidate k's exact commit, not main's tip.
- **Proved out of order, landed in order:** a successor whose proof
  ends first waits, with no effect: no landing, no rejection, no
  report. The serializer lands only the train's head, once main equals
  that head's parent.
- **An ancestor's failure discards its successors' candidates:** a red,
  a conflict, a fault or a recovery doubt at candidate k invalidates
  every candidate prepared on it; their results, arriving later, decide
  nothing (no landing, no rejection, no report). Their landings return
  to the ready set unchanged and are prepared again on the new parent,
  where the keys a certified result already proved need no run
  (section 5).
- **No speculation during a bisection:** from a red until its
  bisection's whole worklist is decided, nothing is prepared on any
  candidate of it; the halves are proved one at a time against main as
  it is, each carrying the full-proof requirement it inherited, and the
  worklist is persisted so a restart cannot change the outcome. Nor is
  anything prepared on a candidate whose preparation left a landing
  out until that is decided.
- **An invalidated prover is not free:** a prover whose candidate was
  invalidated stays busy until its commands and cleanup have ended;
  only then does it take another candidate.
- **Landing a successor needs its whole chain:** a successor lands only
  after every candidate ahead of it has landed, at exactly the parent
  its proof certified; a successor's green never lands an ancestor.

### 6.4 A repair first

- **A landing may carry a priority** (gap L10), `repair`, with a reason,
  set by the operator or a party whose authority allows it: when the gate
  itself or main is broken, its repair must land before batches can be
  green.
- **Pinned, alone and in full:** it is pinned at its head (a moved head
  refuses the priority rather than moving it), enters the next
  candidate alone, ahead of every ready landing, and is proved in full,
  every gate whatever its receipts say, since a broken gate is what it
  repairs. The candidates in flight are decided first, as under a drain;
  nothing is prepared on it.
- **Spent once decided:** green, it lands; red, it is a repair like any
  other. With its decision it may place a hold over admission (3.5),
  owned by whoever set the priority, who lifts it when the gate is
  trusted again.
- **It passes its setter's hold:** a hold over admission owned by the
  party that set the priority admits that landing, so a broken gate can
  be held shut while its repair goes through.

### 6.5 Fairness

- **First ready, first prepared; first proved, first landed**
  (`landing.md`, 9): within a train, landing order is the train's
  order, which is preparation order. A batch is proved when all its
  members are. A priority (6.4) goes before both.
- **The reservation** (`landing.md`, 9) holds across batches and trains,
  unchanged: a landing whose proof went stale `stale_proofs` times
  reserves the next turn for its repository; while it proves again,
  no other candidate there may take the turn, and the reservation
  lapses as `landing.md`, 9 says.

### 6.6 Across repositories (Later, not v0.1.0)

A sketch, so that v0.1.0 builds nothing in the way; its details are
for when a project needs it.

- **In v0.1.0,** a change that spans repositories lands in steps: each
  step is a change of its own, whose landing is parked until the one
  before lands (3.5).
- **Kept open now:** every identity names its repository (a
  candidate's parent and commit, a key's inputs, a receipt, the
  serializer's state), and the serializer is one per project with a
  train per repository. A joint landing is then a set of per-repository
  candidates that one serializer prepares, proves and decides together,
  and a downstream package's key can hold an upstream's pinned commit.
- **Later adds:** joint landings and their admission, the turn's order
  across repositories (upstream first), a partial turn's rule
  (`landing.md`, 8), pins kept at an upstream's main, `with` across
  goals, and batches that take a joint landing whole.

## 7. Provers and runners

- **A prover runs one candidate's gates at a time, with the machine to
  itself:** a measured gate is a measurement, and a congested prover
  makes tests starved or slow that are neither (`testing.md`; skein's
  `testing-strategy.md`, 2.10). Nothing else builds there.
- **Any certified runner proves a deterministic gate:** a workspace's
  check job (section 4), a forge's runner, or a prover; the receipt is
  the same, and names the runner (section 5).
- **Warm by affinity:** a prover keeps its own checkout and build cache
  across candidates, never shared with another machine, so a candidate
  goes to the prover whose cache it reuses most, where several are
  free.
- **A forge's runner or the project's own,** reached the same way: by
  dispatch (`landing.md`, 7.1). A project's own prover is a self-hosted
  runner with a label of its own, one per machine, which fetches the
  candidate from temper's branch (6.1).
- **Dispatch is recorded before it starts, and starting is
  idempotent:** the serializer persists the request (attempt, candidate,
  runner) before asking the runner to start it; asking again for the
  same attempt answers that attempt's state, never a second run. A
  runner's state for an attempt is running, ended (with its result) or
  unknown; unknown is never read as idle.
- **A runner answers with a certificate,** accepted only when it
  matches the request in full: the generation, the attempt and the
  runner; the parent, the pinned heads in order and the prepare's
  merge commits; the policy and the selection; every gate expected and
  its result, with its complete log returned to the serializer's own
  storage; the checkout clean before the checks and unchanged after
  them (for a workspace's check, before its merge and after its
  restore). An empty plan is accepted only where the request expected one.
  Anything else is a fault.
- **Capacity is the runner's to check:** a runner below its disk floor,
  missing a tool, or unable to fetch the candidate refuses it as a fault
  (section 8) before it runs a gate; a refusal its tools give mid-gate
  is a fault too. It keeps its own cache within its disk (stale test
  executables trimmed), on its own checkout, after its commands end.
- **How many provers** is a project's setting, measured by the share
  of time proofs run, the ready landings waiting, the candidates
  invalidated and the serializer's preparation lag; a second prover is
  worth it while the first is busy most of the hour with landings
  waiting.

## 8. Outcomes and who hears them

| Outcome | What it is | What follows | Who hears it, by default | For example |
|---|---|---|---|---|
| green | every blocking gate holds at the candidate (advisory gates reported only) | the turn (section 9) | each member's goal: its landing, with the commit | a batch landed |
| red | a blocking gate's own step failed, and no fault was recognized | a batch is bisected; one landing's red is a repair of the failing member (`landing.md`, 7.3), its brief the failing step and the log | the failing member's goal | a lint, a test, a raised count, a tools' suite's nonzero exit or timeout |
| conflict | the prepare could not settle a member | conflicts the project's rules know are settled by its tooling; any other leaves that member out for a resolution (`landing.md`, 3) | the member's goal | a counted baseline recounted; a textual conflict to resolve |
| fault | the runner's, not the change's: set-up failed, a refusal for capacity, a missing tool, a transport or certificate failure, a crash | retried with nobody rejected and no member marked tried, up to `gate_retries`; past it the landing is held (`landing.md`, 7.4) | the operator, once per episode | a disk below its floor; a runner restarted without its tools |
| starved | a test that the prover starved of time it was owed | neither a pass nor a fail: recorded apart in the receipt, the gate's verdict unchanged by it (skein's `testing-strategy.md`, 2.10) | recorded; the operator when it recurs past its bound | a timed story on a loaded machine |
| stale | at the train's head, its parent is not main's tip; or an ancestor in the train failed | prepared again; certified keys keep their receipts | nobody | a successor whose ancestor failed; a main moved by hand |
| key gap | a full proof fails where selections passed | a red as any other (bisected in full); the missing input joins its key | the operator, beside the red's own | a test reading another package's files |

- **A fault outranks a red:** where a gate failed and the runner also
  recognized a fault (a refusal for capacity, a lost runner), the
  outcome is the fault. A suite that ran and failed, or timed out, is
  red; a test step whose selection was empty is green.
- **A red main already has is main's:** where a full proof of main
  left a key red, a candidate's failure in the same tests is not its
  members': nobody is rejected, its landings return to the ready set,
  and the operator hears it, until a repair (6.4) turns main's key
  green.
- **A successor is not stale while it waits:** its parent is compared
  with main only when it reaches the train's head; before that it waits
  on its ancestors' decisions.
- **A flake is a red until shown otherwise:** a failure whose tests
  all match open flake findings is rerun once (11.1); a failure the
  change cannot reach, passing alone, is a finding, never a silent
  rerun.
- **The routing is a table** (3.7, section 14): a project may also tell
  the member's coordinator of a red, or the person's chat of a fault
  that holds a landing.

## 9. The serializer and the turn

- **One per project, the only writer of its mains:** `landing.md`'s
  turn, held by one serializer that alone prepares candidates,
  dispatches them, adjudicates certificates, lands and reports, with a
  train per repository. Runners never write the forge's refs or report.
  One authority lock covers every mode that can move a main (the train,
  a single prover, a priority, a restart), and a mode refuses to start
  while another's state or a runner's checkout is unresolved.
- **It lands the train's head only,** after reading main afresh: main
  must equal the certified parent, then main is fast-forwarded to
  exactly the certified commit. No force, no merge at the turn; a
  fast-forward check alone is not enough, since main might be another
  ancestor of the candidate.
- **Decisions before effects:** the serializer's state is versioned (a
  version it does not know is refused, never guessed) and holds the
  requests, the reserved names, the certificates, the invalidations, the
  bisection worklist, the priorities and each decision under a stable id
  with a mark once applied, so an accounting step is applied once. A
  landing is written as an intent before its push.
- **Effects at least once, through an outbox:** each effect (news, a
  status, a branch's release) is written as pending, delivered, then
  marked delivered; an uncertain delivery is sent again under the same
  id, and its reader may see it twice. A failed delivery never changes
  an outcome. A branch is released by a deletion that names the head it
  landed (an expected-head lease), so a head pushed since, or a branch
  made again, survives; a failed release stays pending work.
- **Recovery reads main and the intents:** after a crash, under the
  lock, the serializer reconciles each runner first, then each landing
  intent in train order: main at the candidate means the landing
  happened, to be finished once; main at its parent means it did not,
  to be tried again under the same decision; main anywhere else stops
  promotion until it is explained. A runner's result for an invalidated
  candidate is ignored, and a runner that cannot be reached stays
  unavailable.
- **Draining is a hold on admission** (3.5): it closes admission (no new
  candidate, no new bisection group), lets admitted proofs end in order
  and decides them, persists any recovery work left, and only then may
  the serializer's tooling or the runners' checkouts change; a drain
  that times out stays in force. Restarting resumes from the persisted
  state, with the settings it had.
- **Nothing runs inside the turn:** no gate, no prepare (`landing.md`,
  8). The turn is seconds; the train is how proofs overlap.

## 10. After landing

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
  (11.1); unable to run, a fault, retried and the operator told.
  Failures the previous run had are standing, not new.
- **New reds are bisected** over the commits of the landings in the
  run's range, within a bound of time and runs, to the landing whose
  commit turned them red, to a range, or to no answer; the project may
  "tell all" suspects instead.
- **Where a red goes:** by default to the nearest coordinator above each
  suspect landing's change (3.7), with the tests, the log's end and the
  bisection's answer; the coordinator decides the repair. Standing reds
  and recoveries are recorded and reach nobody; a new flake's finding
  is routed as findings are (11.1), and a flake seen again is counted on
  it and reaches nobody. A project may
  send a red to the landing's goal as well, or hold admission into that
  repository until the red is repaired (3.5).
- **Cleared, where a project asks for it** (gap L14): a project may hold
  each change's result until its tiers clear it (a run whose range holds
  it blames it nothing), up to a verification bound; its goal then ends
  only once its work is verified, and a red before then goes to it as a
  repair. Off by default.
- **No test twice on a tree:** a tier's key with a receipt is not run
  again; a bisection's runs are on other trees.

## 11. Findings and QA

### 11.1 Findings

- **A finding** is something found that is not the finder's to fix now:
  a QA case that failed, a flake, a red its landing's goal no longer
  holds, a review's follow-up.
- **A record in temper's store** (gap L15): its kind, severity and area;
  its reproduction and commit; its signature (the failing tests' ids and
  the failure's first line, where it has one); its state (`new`,
  `routed`, `backlog`, `dropped`, `closed`) and who holds it. The client
  and `ctl` show it.
- **Matched by signature:** a flake seen again is counted on its
  finding, not filed again; a gate's failure whose tests all match open
  flake findings has its failed jobs rerun once rather than repaired.
- **Routed by role:** to the coordinator its landing's change or its
  area names (3.7), which triages it to a goal, to a new goal, to its
  backlog, or drops it with a reason; the person's chat takes what has
  no coordinator.
- **Closed** when a landing whose change names it lands, or clears where
  cleared is on (section 10).
- **On the forge, optionally:** a projection of findings as labelled
  issues, their labels read back as their state, is a project's setting,
  off by default (`findings.md`).

### 11.2 QA

- **A walk** is a goal under a tester charter, started by a coordinator
  or the person on a build at a named commit, or once named commits have
  landed (gap L16). It claims a workspace (3.2), deploys the build there,
  holds an account's sign-in in custody for its length (exclusive,
  returned at its end, never copied), and exercises stories as a user
  would, with no forge effects. It writes down what it expects before
  each case, and reports each case's outcome and each finding with its
  reproduction. A walk has a cap; a pause keeps its workspace and its
  state, and a resume carries its findings on.
- **Marks:** a landing marks what a user can now do with a `QA:` trailer
  in its commits; landing news carries the trailers of each commit it
  brought to main (`QA:`, `Taught:`), and a goal's wake policy may name
  them (gap L17).
- **A pass** is a procedure that starts on the newest main whose tiers
  ran green and that holds chosen marks: it batches up to a number of
  marked landings, runs a walk over them within a cap, and records a
  verdict per landing. Its findings go to the coordinator of the landing
  that carried the mark; its exceptions to the person's chat. Marks are
  chosen by default from a starting main; the QA chooser (a coordinator)
  moves exceptions to its backlog or runs them.

## 12. Each forge, by what it offers

| Step | temper's own (every forge) | GitHub native | Forgejo native |
|---|---|---|---|
| Prepare a candidate | the forge's rebase for a landing of one; a merge chain on a branch of temper's for a batch or a train; the project's tooling (`landing.md`, 5.3) | the same | the same |
| Prove | dispatch-only gate workflows, receipts by key (`landing.md`, 7); a project's own provers as self-hosted runners | the same; the merge queue's `merge_group` runs would be gate runs temper did not dispatch, so a queue-backed turn reports receipts already made | the same |
| Speculate (a train) | temper's own: candidates on temper's branches, landed in order | the merge queue builds each group on the groups ahead of it: a train of GitHub's sizing | none: temper's own |
| The turn (check base, then land) | read main, fast-forward it (`landing.md`, 8): Forgejo by a fast-forward merge, GitHub by a ref update with an `Always` bypass | the merge queue: temper enqueues a proved candidate; its required check is temper's status, already met; GitHub merges the group it built, fast-forward | none: temper's own |
| Show it | the candidate on its pull request, `temper/gate/<name>` statuses (`landing.md`, 10) | plus the queue's "queued" and position | the same as temper's |
| Batching | 6.2 | the queue's groups, sized by GitHub: temper caps them by its own batch | temper's own |

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

## 13. fabrig's pipeline in the model's terms

The test of the model's reach: each operation fabrig runs (the roles and
tools of its factory repository, and its own `tools/`), the model's term
for it, where the model has it, and whether fabrig runs it today:
**live**, **built** (and not running) or **designed**. As of 2026-10-30.
fabrig's queue tests (`tools/queue/test_mq.py`, 117 of them) list cases
temper's worlds should carry.

### 13.1 The tree, workspaces and leases

| fabrig | The model | Where | Today |
|---|---|---|---|
| the user, and the director, a Claude session | the person and their chat | 3.1 | live |
| coordinators, one per draft (`s/dynamic-memory`, `s/qa`, `s/upgrades`), Claude at xhigh, in the factory's coordinator registry | coordinators, several in a project | 3.1 | live |
| leads, one per area and part (`lead/<area>`, `lead/<h>/<area>`), ephemeral, Claude at high (Codex for upgrades), carried by `state.md` and `common.md` | area goals; their notes | 3.1, 3.6 | live |
| implementers: Codex tasks, one task, branch and slot each, at high effort (medium for mechanical work) | producers under a charter naming model and effort | 3.1 | live |
| reviews, surveys and an advisor after two misses: read-only Codex tasks | delegates; standing rules | 3.1 | live |
| the operator, `ops`, a Codex session | the operator | 3.7 | live |
| waves: an interface step alone, then one writer per crate; no branch across trees but in three steps, a required field excepted | standing rules; leases | 3.1, 3.3 | live |
| slots: `claim`, shares in `tools/slots/hosts`, lends, claims that never expire, lifetime holds across a run, its precheck and its readers' tests | workspaces, claimed, shared and lent | 3.2 | live |
| `tools/slots/trim`, the disk floor before each heavy command | a workspace's housekeeping | 3.2 | live |
| `lease` on packages and design documents, first-come waiters, lends to a branch | leases | 3.3 | live |

### 13.2 Before ready

| fabrig | The model | Where | Today |
|---|---|---|---|
| the implementer's focused tests, formatting, lints and dependents' tests | advisory checks; the change's own gates | 4.1 | live |
| `precheck SLOT TIP`: the tip merged with main uncommitted, ratchets, tools' suites, fmt, clippy, a test build, the checkout restored and verified | own deterministic gates at its own candidate, in its workspace | 4.1 | live |
| a change of meaning's readers' tests, from Cargo's reverse dependencies, merged with main | advisory measured checks | 4.1 | live |
| `ready-check`: the precheck's files against main's first-parent changes since its base, by file, not by what the checks depend on | freshness by key (`before_ready: keys`), which also sees a dependency changed | 4.2 | live; the queue does not check its evidence |
| a lead pushes `ready/<area>/<task>` at the checked tip | a change becomes ready; its landing of one | `landing.md`, 3, 4 | live |

### 13.3 The merge queue

| fabrig | The model | Where | Today |
|---|---|---|---|
| `leads.txt`: a branch's area to a post address; `ops` for errors and suspected gaps | routing by role | 3.7, 8 | live |
| tips tried, pinned at selection, oldest tip time first | pinned heads; readiness order | 6.1, 6.5 | live |
| batch of 16 (8 by default), oldest tip commit first, then by name | a batch, first ready first | 6.2 | live |
| `git merge --no-ff -m "Merge <branch>"` per member | prepare by a merge chain | 6.1 | live |
| `old_forms.py --write` lowers baselines; ratchet-only conflicts recounted | the project's prepare tooling, its candidate's own commits; conflicts its rules know | 6.1; `landing.md`, 5.3 | live |
| selection: owners, reverse dependencies, lockfile entries, `always.txt`, `read-by-tests.txt` | keys per package; what a key cannot see | 5 | live |
| docs-only batches skip cargo (a touched tree's ratchet still runs); tools-only batches run the tools' suites and no cargo | repository gates keyed by their own paths; paths no package gate reads | 5 | live |
| steps: ratchets, tools, fmt, clippy, build, focused, fuzzy | repository and package gates, deterministic (ratchets, tools, fmt, clippy, build) and measured (focused, fuzzy), each with its key | 5; 17 | live |
| `heavy --gate`: the gate machine builds nothing else | a prover's exclusive use | 7 | live |
| a full run every 5 attempts, hourly, or idle with main moved | full proofs that audit the keys; an idle audit of main | 5 | live |
| a red batch halved, left before right | bisection | 6.2 | live |
| a rejection posted with its step and log | a red receipt; the member's repair brief | 8; `landing.md`, 7.3 | live |
| below the disk floor; an exception | faults, retried, nobody rejected | 8 | live |
| a missing tool: an exception in the repair lane, elsewhere a step's exit 127, rejected as red | a fault, always | 8 | live, as a red |
| a full run red where selections passed | a red bisected in full, and a suspected key gap to the operator | 5; 8 | live |
| test executables trimmed after a batch | the prover's own housekeeping | 7 | live |
| `--repair` at a pinned tip, singleton and full; `--repair-hold` | a priority; a hold over admission | 6.4; 3.5 | live, by hand |
| `queue.stop`, `tools/queue/restart`, one authority lock, fail-closed recovery | draining and restart under one authority lock | 9 | live |
| `queue.state.json`, `queue.jsonl` | the serializer's persisted state; its audit stream | 9; `landing.md`, 12 | live |
| `git push origin HEAD:main` | the turn's fast-forward | 9 | live |
| the ready branch deleted after landing under an expected-tip lease | the member's branch released by an expected-head deletion, through the outbox | 9 | live |
| the train: one controller, a worker per gate machine, bundles, certificates, invalidation, an outbox | a train of depth two; provers; certificates; the serializer | 6.3; 7; 9 | built, not active |
| batch size 8 then 16; a builder drained to prove, then retired | settings measured: batch size, provers | 6.2; 7 | live |

### 13.4 After landing, QA and the rest

| fabrig | The model | Where | Today |
|---|---|---|---|
| `after-landing` on its own builder slot: main polled every 60 s, one tip at a time, more than two waiting coalesced into the newest | tiers following main | 10 | live |
| its profiles: exhaustive, QA, process and browser suites per tree | tiers | 10 | live |
| a failure rerun once alone; red, flake or fault; a bisection bounded at 45 minutes and 16 runs | the verdict; bisection | 10 | live |
| new reds posted to each suspect branch's coordinator; standing reds, flakes and recoveries as events | where a red goes | 10; 3.7 | live |
| `qa walk`: a slot, a build at a commit, a sign-in in custody, a Codex walker, a cap; pause and resume | a QA walk | 11.2 | live |
| `QA:` trailers, passes on a green after-landing main, five marks a pass, 120 minutes, findings to the landing's coordinator | marks; a QA pass | 11.2 | designed |
| findings in a walk's report; the backlog's waiting list | findings, routed and triaged | 11.1 | live, by hand |
| records (events, drafts, priorities, questions, decisions); the status page; the pipeline watch (silent sessions, unread reports, hung runs, idle claims, stalls) | temper's store, the client's views and `ctl` | 15.4 (L19) | live |
| a draft paused: its coordinator and leads take no wakes; the director never pauses | a drain over a coordinator's subtree; the chat exempt | 3.5 | live |
| a session stopped by hand; heavy cancelled by its token | a stop | 3.5 | live |
| a handover near 75% of context: a `-next` session warms up beside a coordinator or the director and takes its mailbox; a lead ends and the next starts from its files | a fresh run from notes and a handover | 3.6 | live |

## 14. Settings

| Setting | Scope | Default | fabrig's |
|---|---|---|---|
| `before_ready`: `keys`, `advisory` or `none` | project | `keys` | by files, before ready, not at admission |
| gates: workflow or check, class (deterministic, measured), ignored paths, blocking or advisory | project, repository | the repository's adoption | ratchets, tools, fmt, clippy, build deterministic; focused, fuzzy measured |
| keys: by the project's tooling (per package) or by tree | project | by tree | per package, from cargo |
| prepare: tooling command; merge or rebase for a batch | project | temper's rebase; merge | `old_forms.py`; merge |
| `batch`: a size, or by measure | project | by measure | 16 |
| `provers`, `train_depth` | project | 1, 1 | 1, 1 (2 built) |
| `full_proofs`: every N attempts, hourly, idle after | project | 5, 1 hour, 10 minutes | the same |
| `gate_retries`; the starved bound | project | 2; skein's | — |
| priority: who may set it; a hold after it | project | the operator; no | the operator; at the operator's choice |
| `stale_proofs` before a reservation | project | 2 | — |
| tiers: per repository, report format, coalesce beyond N waiting | repository | none | nine profiles; 2 |
| a tier's red: bisect within a bound, or tell all | project | bisect, 45 minutes, 16 runs | the same |
| a tier's red goes to: suspects' coordinators, also their goals, holds admission | project | coordinators | coordinators |
| cleared: change results wait for tiers, verification bound | project | off | off |
| findings: on the forge as issues | project | off | off |
| QA: marks from a starting main, marks a pass, a pass's cap; walks' caps | project | off | designed: on, 5, 120 minutes; walks, 180 minutes |
| routing: outcome to role | project | sections 8, 10 | the same |
| the operator | project | the person's chat | `ops` |
| workspace shares; leases | project | none; on | per area; on |
| handover: context threshold | charter | — | 75% |
| a forge's merge queue backs the turn | project | no | no |

These join the landing rules as settings (`forge.md`, 8.9; `landing.md`,
13.1).

## 15. What temper has today, and what to build

### 15.1 The code (fabrig 94c0f7648, 2026-10-30)

- **Landing by turn, on Forgejo, partly built,** opt-in by
  `landing.by = turn` (the default is `queue`,
  `temper/crates/temper-service/src/startup/application/landing.rs:417`;
  `landing.md`, 13.1): the landing as a procedure task the deployment
  makes; members joined across repositories and goals by a dependency
  amendment, at most one per repository; the forge's rebase as prepare
  and a conflict as a resolution; keys from the forge's tree reads,
  without ignored paths (`temper-engine-domain-forge-landing/src/key.rs`);
  receipts in families 23 `Landing` and 24 `Receipt`, the oldest evicted
  but main's; dispatch-only gates with `temper_commit` and `temper_key`,
  runs found by polling and followed, a runner's fault retried twice
  then held; `temper/gate/<name>` statuses; the turn: first proved,
  bases checked, fast-forward-only merges upstream first, partial
  landings kept; restart mid-proof; `ctl landing show` and `landing
  list`. GitHub's turn is refused.
- **The `queue` way** (`forge.md`, 8 to 10), the default: CI waited on at
  each pushed head, queues per repository and base, aging, a red base
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
  never evicted) and **the tree key** as the default (section 5); its
  version byte leaves room for keys per package.
- **Proof by dispatch:** run matching, the runner's faults and their
  retries, the run's classification
  (`temper-engine-domain-forge-landing/src/verdict.rs`) as the first cut
  of section 8's classes.
- **The forge's rebase** as the prepare of a landing of one (6.1).
- **`temper/gate/<name>` statuses, `ctl landing`, the turn-restart and
  owned-turn process stories.**
- **The goal issue projection,** a display.
- **skein's Forgejo calls; GitHub's status write, enqueue and
  `merge_group` decoding,** the last two for section 12's accelerator.
- **`Check::Tier`** in family 24's codec, for tiers' receipts (section
  10).
- **jig's J1** and its designs for J2 to J8.

### 15.3 Delete

- **Today's joining across repositories** (6.6, H9): the dependency
  closure and its upstream-first order
  (`temper-engine-domain-forge/src/landing.rs:169` to 271), the root's
  join by the amendment
  (`temper-engine-domain/src/route/forge_route/landing_route.rs:118` to
  137), `kept_landings` (`:227`) and the partial turn across
  repositories. The rule of one change per repository in a landing
  (`landing.rs:236`;
  `temper-engine-domain-forge-landing/src/domain.rs:18`) goes with it:
  a landing is one change, and batching groups landings. The designs
  of pin-only members, `with`, stacks and pins kept at main, with their
  settings `dependencies_join` and `pins` (`landing.md`, 4.1, 5.3,
  13.1), move to Later (6.6).
- **The `queue` way** (`forge.md`, 8 to 10), once the turn lands every
  project, GitHub's included: its states (`Checking`,
  `CancellingGates`, `Queued`, `First`, `Updating`, and the change's own
  `Landing`; `temper-engine-domain-forge-change/src/boundary.rs:157`),
  its CI and semantic repairs, its ranking and `queue_window`, the
  merge-style update and the merge in the repository's style, and
  `landing.by`. It is woven through the change procedure, the forge's
  top and the simulation world: rework, not a module's deletion.
- **Findings as forge issues by default,** their labels read back, the
  forge's closing keywords, and questions projected as issues: to a
  setting that is off (11.1) or to Later.
- **A change that lives to cleared by default** becomes a setting, off
  (section 10).
- **The drafts `process-model.md` and `porting.md`,** carried here.

### 15.4 What to build

Gaps by id; process-model's ids in brackets where one carries over. The
grades are *design* (to design, and it fits) or *conflict* (to design,
and it changes something designed), as before.

| Id | Gap | Grade | Changes |
|---|---|---|---|
| L1 | Batches: a candidate of several landings, bisection by halves, the size by measure | design | `landing.md`, 1, 4, 9 |
| L2 | Trains: a candidate on a proving one, the head landed on its certified parent, successors invalidated, no speculation in a bisection | design | `landing.md`, 8, 15 |
| L3 | Candidates on temper's branches by a merge chain; the prepare's own commits; pinned heads and reserved names; conflicts staged with their attempt [T5] | conflict | `landing.md`, 5 (a moved head no longer sends a live candidate back) |
| L4 | The prepare and key job on a worker: the project's tooling, no model, no credentials [T2] | design | `landing.md`, 5.3, 6.1; `worker.md`, 4.4 |
| L5 | Keys per package and repository gates' keys, versioned; receipts kept from discarded candidates; what a key cannot see; full proofs that audit [T3] | design | `landing.md`, 6.1, 6.2 |
| L6 | Gate classes: deterministic receipts from any certified runner, measured ones from provers; the environment in the key | design | `landing.md`, 6, 7 |
| L7 | Provers and runners: labelled runners with affinity, idempotent dispatch, certificates, faults before a gate [T4] | design | `landing.md`, 7 |
| L8 | Outcomes: starved recorded apart, key gaps, faults outranking reds, a red main already has [T9 in part] | design | `landing.md`, 6.2, 7 |
| L9 | The serializer's durability: one authority lock, versioned state, decisions applied once, intents, the outbox, recovery by main, drain as a hold [T6] | design | `landing.md`, 8, 11, 12, 15 |
| L10 | A priority: a repair pinned, alone and in full, ahead of every ready landing, passing its setter's hold | conflict | `landing.md`, 9 (priorities ordered only gate dispatch) |
| L11 | Checks before ready: a change's own deterministic gates in its workspace at its own candidate; `before_ready` | design | `landing.md`, 3; `forge.md`, 8.1, 8.3; `worker.md`, 4.4 (a check job) |
| L12 | Leases: one writer of a path across goals, refused at ready, lendable, first-come waiters | design | `forge.md`, 8.1 |
| L13 | Tiers following main: one run at a time, the newest tip, coalescing, the rerun, bisection within a bound, reds routed by role [T8, T9] | design | `after-landing.md` |
| L14 | Cleared as a setting: change results held to a verdict, the bound [T10] | design | `after-landing.md`, 6 |
| L15 | Findings as temper's records, matched by signature, routed and triaged; the forge projection a setting [T11] | conflict | `findings.md` (the store moves from issues to temper) |
| L16 | QA walks and passes: a tester charter, a workspace, a sign-in in custody, marks, a pass procedure [T12] | design | `findings.md`, 8 |
| L17 | Trailers in landing news and wake policies on them [T13] | design | `forge.md`, 7.2 |
| L18 | The reservation [T7] | design | `landing.md`, 9 |
| L19 | Views and acts: candidates, batches, trains, receipts, tiers, findings, holds, leases, workspaces, in the client and `ctl` [T15] | design | `landing.md`, 14; client |
| L20 | A landing is one change in one repository in v0.1.0; joint landings across repositories Later (6.6) | conflict | `landing.md`, 4.1, 5.3, 8, 13.1 |
| J1 | Owned holds and proposals of their lift | built | jig's `domain/tasks.md`, 5.9 |
| J2 | Drain | design | jig's `domain/tasks.md`, 5.9 |
| J3 | A project's holds, inherited, the chat exempt | design | the same |
| J4 | Parked until an event | design | jig's `domain/tasks.md`, 5.9 |
| J5 | A live task moved under another task | design | jig's `domain/tasks.md`, 7 |
| J6 | A successor, named late too, with claims and leases | design | jig's `domain/tasks.md`, 7 |
| J7 | A question to the chat stays with it (person answers are built) | conflict | jig's `domain/tasks.md`, 8.5; temper's `agent.md`, 8.2 |
| J8 | The chat asks its own person | design | the same |
| J9 | A hold over a kind of procedure's admission | design | jig's `domain/tasks.md`, 5.9; `landing.md`, 4.2 |
| J10 | An address by role in the tree: the nearest coordinator above, the project's operator | design | jig's `domain/tasks.md`, 8; `people.md` |
| J11 | A fresh run from a goal's notes and handover at its context's bound | design | jig's `domain/engine.md`, 7.1, 10 |
| J12 | Workspaces as resources a goal claims and lends, shares as authority | design | jig's `domain/authority.md`, 4; temper's `worker.md`, 5 |
| F1 | skein: read a run's artifacts; update a ref fast-forward only [B1] | design | skein's `forge.md`, 3.2 |
| F2 | GitHub's turn: tree and run reads, dispatch, rerun, the ref update under `Always` [B2] | design | skein's `forge-github.md`; `landing.md`, 8 |
| F3 | GitHub App permissions: statuses and actions written, the `Always` bypass, `workflows` [B3, B5] | conflict | temper's `protocol/setup.md`, 17.1 |
| F4 | Adoption checks: dispatch-only workflows, Forgejo's "autodetect manual merge" [B4] | design | `landing.md`, 13.2 |
| F5 | GitHub's merge queue backing the turn | design | `forge.md`, 9.1; section 12 here |

## 16. Later

- Landings across repositories (6.6): joint candidates decided
  together, the turn's order and partial rule, pins, `with`.
- Keys per package for languages other than Rust (the project's tooling
  names a package's inputs).
- Trains deeper than two when one serializer's preparation keeps up.
- Elastic provers: more when proofs queue, fewer when they idle, by
  the measures of section 7.
- Partitioned full proofs: one full proof split across provers.
- Findings as forge issues by default, with the forge's closing keywords
  (process-model's B6).
- The person's questions projected as issues and answered by comment
  (process-model's T14).

## 17. Open questions

H1 to H9 are choices of temper's and jig's primitives, each with the
recommendation this draft is written to; the rest are details.

- **H1, who makes a receipt:** any certified runner for a deterministic
  gate, a prover only for a measured one (sections 5, 7: recommended, so
  that the checks before ready are never repeated), or provers only,
  with checks before ready advisory.
- **H2, where keys are computed:** by the prepare job on a checkout,
  trusted and audited (section 5: recommended), or by the engine from
  forge reads, which cannot see a build's graph.
- **H3, where a candidate lives:** on temper's branches at the forge
  (6.1: recommended, since dispatch needs the commit on the forge) or
  only on the provers, as bundles (13.3's train).
- **H4, routing by role:** an address in jig for the nearest goal of a
  role above a task (3.7: recommended), or subscriptions each goal makes.
- **H5, hold scopes:** subtree, project and a kind of procedure's
  admission (3.5: recommended), or admission held by the serializer's
  own switch, outside jig's holds.
- **H6, leases:** enforced at ready by temper (3.3: recommended), or
  advisory and shown.
- **H7, workspaces:** claimed resources of jig's placement, owned by a
  goal and lent to its delegates (3.2: recommended), or temper's worker
  checkouts with no claim.
- **H8, handing over:** the same task's fresh run from its notes (3.6:
  recommended), or a successor task with a warm-up overlap (13.4).
- **H9, today's joining across repositories,** with joint landings
  Later's (6.6): deleted now, its designs moved to Later
  (recommended: it is built on one change per repository in a landing,
  which batching breaks, so it would be rewritten anyway), or kept
  dormant until joint landings come.
- **A gate per step or one gate of steps:** fabrig's steps (fmt,
  clippy, build, focused, fuzzy) as gates of their own, each with a
  receipt (a lint red then re-proves nothing else), or one gate whose
  receipt names its failing step. Recommended: a gate per step whose
  result a key can carry.
- **A package's key from the build tool or from the project:** cargo's
  metadata gives the Rust graph; other projects name their packages.
- **The batch's size and when to batch:** a fixed size, or from the
  measures of 6.2.
- **Receipts from a speculative candidate:** kept for their keys once
  the certificate is accepted (section 5, recommended: a key's result
  does not depend on which candidate carried it), or held until the
  candidate's ancestors land (13.3's train).
- **GitHub's merge queue and dispatch-only gates:** whether a
  queue-backed turn can always report temper's receipts as the queue's
  required check without a run of its own.
