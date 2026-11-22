# The services in one process

This document holds the services' composition in one process, split from [`README.md`](README.md).

## 1. One process

Standalone is jig's one process (jig's `protocol/service.md`, section 10):

- **One service, one loop.** `temper-service` owns the engine's
  protocol layer and the worker's as components, each with its own root
  and journal, sharing io and nothing else; nothing crosses between them
  but bytes. Each half is a crate of its own, which the engine's and the
  worker's binaries link alone (`services.md`).
- **Joined by the in-process name `workers`:** the engine's workers'
  listener and the worker's link (configuration/README.md, section 2). The
  link's opening, authentication, liveness, relays and grants run as
  between hosts, in plaintext, with no TLS.
- **A client on loopback,** over TCP to the clients' listener: `temper-box
  open`, signing in by the link `temper-box serve` prints at start, or
  `temper box` before it starts serve (`setup/box-supervision.md`, 1).
- **Agents and git as child processes,** each in a process group of its
  own, which a stop signals whole (jig's `protocol/README.md`, section
  10); containment after a forced stop remains open (`README.md`, section 11). An
  agent's program, `claude`, is the agent's child in a group of its own
  (smith's `protocol/programs.md`, section 2), so killing an agent's
  group leaves its program running: only the agent's own stop ends it.
- **The forge outside,** reached over HTTP, and over TLS when it is not
  on loopback; its deliveries reach the webhooks' listener on loopback,
  so a forge on another host polls alone (forge/README.md, section 4).
- **The core knows its workers are in process:** the engine's workers'
  listener is the in-process name, from which jig's configuration tells
  the core so (jig's `protocol/configuration.md`, section 3;
  `../domain/root.md`, section 2).
- **Starting and ending together:** configuration for both, then the
  engine's start, then the worker's. A signal drains the engine; once it
  has drained, the service stops the worker, with no grace for its turns
  and answers and no drain of its own (`../domain/worker/README.md`,
  section 2): its runs are cancelled at once, each agent process ending its
  program before it exits, within smith's cancel, then kill, and the
  service closes the worker once no agent process is left, its git
  invocations' groups killed as io closes them (jig's
  `protocol/service.md`, section 10). The service exits only after. A
  second signal closes the worker at once, its agent processes' groups
  killed, and a program they started outlives them until agents run in
  contained trees (`README.md`, section 11). No worker outlives the engine, so the
  engine, on hearing the signal, settles the worker's agent claims
  as interrupted (jig's `domain/engine/restart.md`, 1): a stop spends no try
  of a run's task, a coding run's among them, and moves none of its
  change's counts; the task is due again at the next start, its next
  attempt resuming from the turns it committed and starting from the
  work it saved last (`../domain/worker/hosted-runs.md`, 1.2 and `../domain/worker/checkouts.md`). After a crash,
  the next start settles them at once, lost or not started (jig's
  `domain/engine/restart.md`; `../domain/root-restart.md`, 1.1). Job claims
  remain for restart recovery even after a drain, since their effects
  may have happened.
- **Its world** is `tests/one-process` (`../testing/tiers.md`, 1.5).

### 1.1 A long pass

One thread runs every stage of a pass, so a stage that takes long holds
all the others, and a client sees only that nothing answered. The service's
log says when that happened; nothing else about the loop changes.

- **The stages,** in the order a pass runs them, are `io` (io's resumes,
  completions and timers), `route` (its events to their owners), `engine`,
  `worker` and `out` (closes, the box's pipe, the components' requests,
  printed links, the publish and the reclaim). `temper-service`'s
  `iterate` runs them, and `iterate_stage` runs one. The binary's adapter
  adds its own two: `names` (a kept forge's name resolved before
  serving, a push's host, and, once serving, at most one of GitHub's
  names a pass, `forge/README.md`, 2.6) and `drain` (handover
  files, operator lines and links), the only stages that may block in a
  system call.
- **The clock.** Every stage still sees the pass's one `now`. Only the
  binary's adapter reads the monotonic clock between stages, and those
  times reach `temper-service`'s `long_pass::Watch` alone, a step that
  takes them as arguments and decides nothing but whether to write a line.
  A world that calls `iterate` has no watch.
- **The bound is 1 s,** `long_pass::LONG_PASS`, on a whole pass: no stage
  can pass it without its pass passing it. It is a thirtieth of `temper
  ctl`'s deadline (`../client/ctl/bounds.md`, section 1), so a stall big enough
  to cost a call that deadline shows at once; and across 1,543 `temper ctl
  me` reads on running boxes under a load of up to 27 on 8 CPUs, the longest
  whole read was 1.95 s, so a healthy box writes few of these lines or none.
  A pass of exactly the bound writes nothing.
- **The line** is one stderr line, which the box keeps in `serve.log`
  (`setup/box-supervision.md`, 1), naming the pass, its longest stage and the bound,
  with no payload:
  `temper-box serve: long pass: 1250 ms, engine 1200 ms of it, past the 1000 ms bound`.
- **At most one line in 10 s,** `long_pass::LINE_EVERY`. The first long pass
  writes at once. Those after it within the 10 s are held as the longest
  of them and a count, and the first pass past the 10 s, long or not,
  writes that, ending `; 3 more since the last line`, the count being the
  long passes it does not describe. So a long stall never hides behind a
  shorter one. A held line left at the service's end is written at its exit.
- **What it costs when nothing is slow:** seven clock reads and a few
  comparisons in a pass, no allocation and no write.
- **Its tests** hold the bound, the rate and the words on supplied times:
  `temper-service`'s `tests/long_pass.rs`, and the line's words beside
  `note_line` in `temper-service-shell`'s `host.rs`.

### 1.2 When the core does not answer

The engine's service hands a party's input to the root only while its
gate is open: the root started and its decisions open
(`../domain/root-restart.md`, 1.1), the forges' records loaded and connected,
the first periods given (configuration/startup.md, section 1) and the forges'
names resolved (forge/engine.md, 2). While the gate is shut, the clients'
component answers a party's input not ready and a public read busy, at
once. A watch refused while the gate is shut says not ready, distinct
from a full watch room. An input the root hands back at its watermark
waits in the service for room (`../domain/root.md`, section 4). The
client names not ready when its answer carries that refusal; the
service's log names what the gate waits on. No gate reason travels on
the client wire (`README.md`, section 11).

- **The line.** When the gate has kept a party's input from the root for
  1 s or more, answered not ready or held, the service writes one stderr
  line, which the box keeps in `serve.log` (setup/box-supervision.md, 1):
  `temper-box serve: the core is not answering: REASON`. It writes it again
  only when the reason changes, and once the gate passes a party's
  input again after such a line, `temper-box serve: the core answers
  again`. The 1 s is 1.1's bound, a thirtieth of `temper ctl`'s
  deadline. The time is the service's clock, judged as each party's
  input arrives, so a deployment no client calls writes nothing, and
  the line needs no timer of its own.
- **Its reasons,** in the order the gate tests them, the first that
  holds:

  | The gate | REASON |
  |---|---|
  | the root not yet opened | `starting: STEP`, the start's step in 9.1's words: `loading the core's records`, `loading the inbox`, `loading the forge's records`, `linking the forge's records`, `sweeping`, `switching the store`, `committing the wires`, `adopting runs`, `opening decisions`; the forge's catching up comes after the opening and holds no client (`../domain/forge/catching-up.md`) |
  | the forges' records not yet loaded and connected | `loading the forges' records` |
  | the first periods not yet given | `opening the first periods` |
  | the forges' names not yet resolved, GitHub's apart (`forge/README.md`, 2.6) | `resolving the forges' names` |
  | the root's commits past its window, waiting on the store | `waiting for the store to make commit N durable, the oldest of M in flight` |
  | the root at its watermark | `at its watermark: B of W in flight` |

  A root whose journal has stopped is not among them: it ends the
  engine (below).
- **A stopped journal ends the engine.** The root's `Stop` with what
  stopped its journal (`../domain/root.md`, section 4) ends `temper-box
  serve` with exit 1 and one line, `temper-box serve: the engine's journal
  stopped: WHY`, WHY one of `the store answered commit N, with S sent
  and D durable`, `a decision past the store's E writes`, `the commit
  counter at its end`. A commit the store failed keeps its own line,
  `temper-box serve: commit N failed: WHY` (`store-versions.md`, 1), and a start's refusal its own lines
  (`binaries.md`).
- **Bounded, and free when the core answers:** at most 240 bytes, no
  request's words and no secret; one reason held, and a comparison as
  each party's input arrives.
- **Its tests** hold the 1 s, the change of reason, the line on the gate
  passing again and each reason's words, on a service stepped with
  supplied times, in `temper-service`'s tests; the journal's stop and
  its line beside `note_line` in `temper-service-shell`'s `host.rs`.

### 1.3 A wait that runs long

A start that does not open, a forge that keeps catching up, or a table
that stays full says what holds it (`../draft/reconciling-start.md`, 5). 1.2's line names the gate
alone, and only as a party asks; these name the waiters, with no party
asking. On 2026-11-20 dev.fabrig.ai's start logged "reading the forge
afresh" alone, while six goal branches waited for places that eight
cancelled landings held.

- **The report bound is 30 s,** `WAIT_REPORT` (`configuration/derived.md`,
  5.6): three times dev.fabrig.ai's whole start once that cycle was cut
  (ten seconds, 2026-11-20), so a healthy start writes none of these
  lines. It is no deadline: a report fails, ends and slows nothing.
- **Its age, in words:** below 2 min in seconds, `30 s`, `60 s`; from
  2 min on in whole minutes, rounded down, `2 min`, `17 min`.
- **A step of the start past it.** Once one step of the start
  (`../domain/root-restart.md`, 1.1) has lasted the bound, counted from the
  core's asking for it, the service writes one stderr line, then one
  each time the step's age doubles (60 s, 2 min, 4 min, ...), until the
  step ends; the next step counts from its own beginning:
  `BINARY serve: starting: STEP for AGE: WAITS`. STEP is 1.2's words for
  the step, and WAITS what it waits for, by kind, joined by `, `, each
  `N KIND (NAME, NAME, NAME)`: the count, the kind's words, singular for
  one, and the three oldest by name, oldest first, fewer where fewer
  wait; `nothing named` where it waits on none of these:

  | Step | KIND | NAME |
  |---|---|---|
  | a load or the sweep | `store page`, `store pages` | its range: `the core's records`, `the inbox`, `the forge's records`, `the sweep's rows` |
  | any step | `commit in flight`, `commits in flight` | `commit N` |
  | switching the store | `store switch` | none: `1 store switch` |

  Kinds come in the table's order, and a kind with none is left out.
  The oldest is the one that began first: by its own time where it keeps
  one, else by the order it was made in (a read's owner token, an
  entry's number, which follow commit order), and those made together
  in their table's order.
- **The forge catching up past it.** Counted from the opening's
  `CatchUp` (`../domain/forge/catching-up.md`, 2), the service writes one
  line at the bound, then one at each doubling until both halves have
  ended: `BINARY serve: the forge catching up for AGE: WAITS`. After
  such a line, the caught-up moment writes one more:
  `BINARY serve: the forge has caught up`. WAITS names these kinds,
  counts and three oldest, as above, or `nothing named` if all are empty:

  | KIND | NAME |
  |---|---|
  | `fresh read`, `fresh reads` | what it reads: `OWNER/NAME branch B`, `OWNER/NAME pull N`, `OWNER/NAME issue N`, `OWNER/NAME listing` |
  | `landing gather`, `landing gathers` | `task T` |
  | `CI read`, `CI reads` | `OWNER/NAME at HEAD`, HEAD its first 12 hex digits |
  | `goal-file read`, `goal-file reads` | `task T` |
  | `lost writer's read`, `lost writers' reads` | `task T` |
  | `goal branch`, `goal branches` | `task T` |
  | `uncertain write`, `uncertain writes` | `entry E of task T` |

  Example:
  `temper-engine serve: the forge catching up for 60 s: 6 goal branches (task 10084, task 10085, task 10086), 1 fresh read (work/hello branch main)`.
- **A full table past it.** A table is full while it has no free place:
  the forge's outbox, its free room zero (`../domain/forge/room.md`, 2),
  and its read admission, its call slab full (skein's
  `forge/client.md`, 5). Both are the forge connector's, one for every
  forge. Once one has been full for the bound, the service writes one
  line, then one at each doubling of its age while it stays full, and,
  after such a line, one when it has a free place again:
  - `BINARY serve: the forge's outbox full for AGE: USED of LIMIT, held by HOLDERS; W waiting for room`,
    the last part left out with no waiter in the room line
    (`../domain/forge/room.md`, 3);
  - `BINARY serve: the forge's read admission full for AGE: USED of LIMIT, held by HOLDERS`;
  - `BINARY serve: the forge's outbox has room again`, and
    `BINARY serve: the forge's read admission has room again`.

  HOLDERS are its holders, oldest first, joined by `, `, at most eight,
  then `and N more`: an entry, `entry E (task T)`, by number; then its
  reservations as they were made (`../domain/agent/room.md`, 2), `room asked for task
  T`, `room for task T's call`, `room for goal T's projection`; a read,
  `READ of OWNER/NAME`, READ what it reads (`branch B`, `pull N`, `issue
  N`, `the listing`, `a file`, `CI at HEAD`), and a write `a write of
  OWNER/NAME`, never what it writes, each with ` for task T` where a
  task's procedure owns it, oldest admitted first. Example:
  `temper-engine serve: the forge's outbox full for 2 min: 8 of 8, held by entry 1201 (task 10061), entry 1202 (task 10061), entry 1207 (task 10070), entry 1209 (task 10072), entry 1210 (task 10072), entry 1214 (task 10075), entry 1215 (task 10075), entry 1216 (task 10077); 6 waiting for room`.
- **The census shows the same words.** The forges' document carries
  each table full past the bound: which table, since when (a wall time),
  and its words after the line's `AGE: `. The deployment's forges, on
  the client's Deployment page and `temper ctl forge list`, show each
  above the forges' rows (`../client/forge/deployment.md`, *Its load*;
  `../client/ctl/areas.md`). A start's steps have no census: no client
  is admitted before decisions open (1.2). The pages' "Catching up: ..."
  counts are `../domain/forge/catching-up.md`, 5.
- **Made by the domain, written by the service.** The root keeps the
  time its step began, and the forge the opening's `CatchUp` and the
  time each table filled; each
  asks a deadline for its next report, so a start no client calls still
  writes. At it, the root's fact carries the line's words after
  `serve: `, and the service writes each as it writes the root's other
  facts. One clock, `Report` beside `WAIT_REPORT`, gives every one of
  them its reports: the first at the bound, then at each doubling, one
  report where a pass skipped several.
- **Bounded:** a line is at most 1,024 bytes, with no payload and no
  secret: a waiter is named as its table names it, never by its words.
- **Its tests** hold the bound, the doublings, a step's end, each kind's
  words and the order of the oldest on supplied times in the root's and
  the forge's worlds, and the line beside `note_line`.

## 2. Open questions

- **The gate's reason on the client wire** (`service-process.md`, 1.2): a distinct not-ready
  watch refusal lets a client distinguish startup or an unanswering core
  from watch capacity, but carries no reason. Sending the start's step
  or the gate's other reason needs jig's vocabulary, wire and client
  link to retain it. Default: the client names not ready and points to
  serve's log; the log alone names why (`../client/ctl/README.md`, section 11).
- **The kernel's share of a pass.** A long pass (`service-process.md`, 1.1) times what the
  service runs, not the reap and the submit around it, since the submit
  holds the loop's wait, and `skein-shell`'s `drive` keeps both. A stall
  there writes no line. Default: so, until a walk's log leaves a stall
  unexplained.
- **Supervision** (jig's `protocol/service.md`, section 9): what
  restarts `temper-box serve` after a failed commit or a crash. Default:
  nothing of `temper-box serve`'s own; a box restarts it (setup/box-supervision.md, 1),
  and otherwise the person or the machine's
  service manager starts it again, and it recovers from the last
  durable commit.
- **Agents in the worker's process** (`smith-inline-agent`), with no
  containment, for one person's machine (jig's `protocol/service.md`,
  section 10). Default: agent processes, as `temper-box internal agent`.
- **Programs after a forced agent stop** (`service-process.md`, section 1): killing an agent's
  group leaves a program in its own group running, even in standalone.
  Ending both needs contained trees or a parent-death signal for the
  program (setup/README.md, section 9). Default: the orderly stop lets agents
  end their programs; a second signal or forced close has no such guarantee.
- **A whole stop's bound** (`service-process.md`, section 1; `cluster.md`, 4): an engine's stop ends
  within what its parts' own deadlines give, the forge's settling
  (`../domain/root-restart.md`, 1.2), the drain, and its workers' channels
  within the stall deadline of the close (jig's `protocol/channel/README.md`,
  9.2), with no deadline of its own, while a service manager kills it
  at its own stop timeout. Default: so; a stop deadline over the whole
  is a figure, the user's to set, and a test of a stop within it
  follows.
- **An engine restarted under live workers.** The engine announces its
  identities' values at generation 1 at each start, sound only while no
  worker outlives it (credentials-git-identities.md, section 1). Default: the cluster
  restarts whole, its workers stopped before its engine (`cluster.md`, 4);
  generations kept across starts come with hosts apart (credentials.md,
  section 8).
