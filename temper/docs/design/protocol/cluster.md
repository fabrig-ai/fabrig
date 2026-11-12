# The cluster

The cluster is temper on several processes of one machine: one engine
alone, its workers alone, and the fakes, each a process of its own,
joined over loopback. It is a primitive of temper's tests, and the
shape of the case of `../testing/reference-case.md` on fakes: an engine and
its workers, one of them added later. It is a deployment's shape too: a
deployment runs its engine alone on `temper-engine serve` and each
worker alone on `temper-worker serve`, `temper-agent` its agents'
program (`binaries.md`, 1), while `temper-box init` writes standalone and a box keeps its
one worker inside `temper-box serve` (setup/box.md, 1). The cluster's stories
run the box's `temper-box`, and one runs the deployment's binaries (6).
TLS, containment and other hosts are later (7).

## 1. Its shapes

`temper-box serve` takes its shape from its document's members, and from
nothing else: no flag, no command of its own. A deployment's binary
serves its own shape alone and refuses the others by their nearest
shape, below (`binaries.md`, 1): `temper-engine serve` an engine alone (`binaries.md`, 2), and
`temper-worker serve` a worker alone (`binaries.md`, 5).

| Shape | `engine` | `workers` | `application` |
|---|---|---|---|
| standalone | jig's engine part; `hosts.listener` `[{"in_process": "workers"}]` | one, whose `engine` is `{"in_process": "workers"}` | every member; `workers`, one entry |
| engine alone | jig's engine part; `hosts.listener` one socket address on loopback, in plaintext, its port 0 or fixed; `hosts.workers` the cluster's workers, each with its digest | `[]` | every member; `workers` `[]` |
| worker alone | absent | one, whose `engine` is a socket address on loopback, in plaintext | `workers`, one entry, and `limits`, alone |

- **Never mixed.** A document of none of these shapes is refused, naming
  the path that departs from the nearest: an engine with both a worker
  and a socket listener, a listener list with an in-process name and a
  socket, a worker alone with any other member. An in-process listener
  tells jig's core that no worker outlives the engine, so it settles the
  claims of every worker at once on a restart (jig's `domain/engine/restart.md`
  ): a socket's workers among them would lose their runs.
  The nearest is read from two members: a document without `engine` is
  nearest a worker alone; one whose `engine.hosts.listener`'s first
  entry is a socket address, an engine alone; any other, standalone.
- **Each shape's checks** (configuration/README.md, section 2): standalone's,
  as configuration/README.md, section 2 gives them; an engine alone's the same, but for its
  listener and its workers, by the table's rule, and the root's for
  every shape with an engine; a worker alone's, its worker part's, its
  entry of `application.workers` (configuration/README.md, section 3) and the
  limits' checks its side holds (configuration/limits.md).
- **The service** has an optional engine and an optional worker:
  standalone both, an engine alone no worker, and a worker alone no
  engine, whose loop, ending and exit are its worker's alone, with no
  store (`README.md`, sections 4 and 5; jig's `protocol/service.md`, section 6).
- **Three directories for each process,** its own (configuration/directories.md
  ): an engine alone's data holds the deployment's directory,
  the store and the secrets' store among it, and each worker's digest; a
  worker alone's, its secret, its root, where its workspaces are, and
  its programs directory. No two processes share one.
- **Memory by its service:** `engine.memory_bytes` meters the engine's
  held transcripts and turns (`../domain/root.md`, 4.1). Memory budgets
  are not startup heap sums (configuration/limits.md); io's sockets
  belong to their own service.
- **`temper-box program-login`** takes a worker alone's directories as it
  takes standalone's, and reads only the worker's part: each worker
  signs its programs in on its own, and none is copied to another
  (credentials.md, section 3). `temper-worker program-login` takes a
  worker alone's alone (`binaries.md`, 5).

## 2. Its processes

```
fake forge   temper-box fake forge ... --ready PREFIX    (setup/playground.md)
fake llm     temper-box fake llm ... --ready PREFIX
engine       temper-box serve DIRS     after both fakes: an engine alone
w1, w2       temper-box serve DIRS     after the engine: each a worker alone
  agents, git                      its children, each a process group
```

- **Started in order, stopped in reverse:** the fakes; the engine, once
  both fakes are ready; each worker, once the engine is (3). A worker
  may start later than the others, as s2 does in the case of
  `../testing/reference-case.md`.
- **Every address from a ready line:** each listener binds port 0 on
  127.0.0.1, and its process prints what it got; the description's
  templates carry it to the processes after it.
  The engine's document names the fakes' addresses as their ready lines
  gave them, and each worker's names the engine's workers' listener as
  its `listening workers` line gave it.
- **Workers are the engine's configured positions,**
  `engine.hosts.workers`, each with its own name and secret (jig's
  `domain/engine/fleet.md`). A worker started later is a position
  the engine knows from its start, empty until that worker links; a
  position added is a new engine document, so the cluster's restart
  (4).
- **The fakes are setup/playground.md's,** each a process of its own with its
  ready line. A worker's git is the program an assignment's repositories
  name, `temper-box fake git` for a fake forge (channel.md, section 4): the
  workers share the fake forge's machine, so its program's path and its
  loopback listener serve them as they serve standalone's worker.

## 3. Joining, and its lines

- **jig's link, in plaintext over loopback** (jig's
  `protocol/channel/README.md`, section 9): the worker dials the engine's
  workers' listener; the engine takes its name and its secret against
  that worker's digest, then its hello: its capability, its configured
  position, its capacity and a stop bound strictly below the engine's
  grace, `fleet.grace_ms`. The worker redials, jittered and doubling,
  until it links.
- **Secrets are the description's fixtures:** each worker's secret, and
  its digest in the engine's data, a fixed pair the description holds
  and renders into each process's directories (5); a test's, never a
  deployment's, and never copied from one.
- **Short graces:** the description's are seconds, each worker's stop
  bound below the engine's grace, so a lost worker's runs are placed
  again within a test's time. Each worker's `graces.drain_ms` is given,
  at most its link's notice and its grace together, so the drain's
  default never enters its stop bound.
- **Lines on stderr,** each one line, flushed once what it says holds;
  the cluster's contract, read by their literal prefixes:

  | Line | Printed by | When |
  |---|---|---|
  | `listening <listener> <address>` | each shape with an engine, for `clients`, `webhooks` and a socket's `workers` | once that listener is bound, with the address bound (`listening workers 127.0.0.1:41733`; an IPv6 one bracketed, `[::1]:41733`) |
  | `linked <worker> <address>` | a worker alone | at each opening of its link: the engine has taken its secret and its hello is sent, with the engine's address |
  | `lost <worker> <address> <why>` | a worker alone | once each linked link is lost, with its `Loss` (below) |
  | `temper-box serve: worker lost worker=<worker> channel=<n>: <why>` | each shape with an engine | once each worker's channel that said hello ends |
  | `temper-box serve: worker opening ended worker=<worker>: <why>` | each shape with an engine | once a worker's connection ends before its hello: refused, turned away, out of time or cut; `worker=-` before its name is taken |
  | `temper-box serve: worker unsent worker=<worker> channel=<n>: <record> of <bytes> bytes <bound>` | each shape with an engine | once each record the engine's link did not send (jig's `protocol/channel/README.md`, 8): `<record>` the record's kind in words (`assignment`, `job`, `message`, `relayed`, `turn-kept`, `turn-busy`, `acknowledge`, `cancel`, `grant`, `sources`, `check`, `sign-out`, `read-list`), `<bytes>` its frame's, and `<bound>` `past the room of <room> bytes` or `refused: <reason>` (`wrong-direction`, `not-taken`, `too-large`, `closed`) |

  **A link that drops is never silent** (jig's `protocol/channel/README.md`, 8,
  *A lost link says why*). `<why>` is the link's `Loss` in one word or
  two: `silence`, `stall`, `full`, `deadline` (the engine's opening or
  hello), `handshake` (the worker's opening), `stream`, `truncated`,
  `framing`, `owner`, `busy` (the worker's busy table), `fault`,
  `unreachable`, `no-room`, `stopping`, `refused <reason>` for a refusal
  this side sent and `refused-by-peer <reason>` for one it received,
  `<reason>` as the refused line names it (`terms`, `malformed`,
  `unauthorized`, `application`, `busy`, `replaced`, `not-admitted`,
  `stopping`, `code-<n>`). Each loss of a linked link and each lost
  channel prints its line; an opening that ends prints once until that
  worker's reason changes or it links, as a refusal does. A workers'
  listener that fails prints its line, `temper-box serve: worker listener
  <address> failed: <why>`, when it fails, not only once the service has
  ended.

  A refused opening or hello is a note on the worker's stderr with jig's
  reason, and the worker redials, as a worker between hosts would; it
  never exits for one. The note reads `refused <worker> <address>
  <reason>`, the address the engine's as `linked` gives it, the reason
  the engine's refusal as jig's link names it (`unauthorized`,
  `application`, `busy`, `replaced`, `not-admitted`, `stopping`), a
  peer's code jig doesn't name as `code-<n>`, or the worker's own
  (`terms`, `malformed`); the engine's text, which jig does not keep,
  is not in it. A refusal is noted once until its reason changes or
  the worker links: jig's link backs the redials off, and a repeat of
  the same refusal prints nothing. The sign-in link stays on stdout
  (`binaries.md`).

  The engine says why a hello is refused (jig's `domain/engine/fleet.md`
  ): the refusal's text, at most 256 bytes, is the fleet's reason
  in words, and the engine writes the same words on its own stderr,
  `temper-box serve: hello refused channel=<n>: <words>`. An agent's shortfall
  reads as a start's problem for it does, after its charter and model
  where it has them (`charter 11: needs 64 turns; the agent's
  profile.policy.max_turns allows 32`), so a charter set after the start,
  which no start's check saw, says the same. A refusal the fleet gives no
  reason for reads `not admitted`. A repeat of the last words prints
  nothing: a worker redials for good, and one line says it until the words
  change.
- **Each line's address is its listener's bound one:** the clients'
  server keeps its own, as the sign-in link reads it; the workers'
  routing keeps the one io's `Listening` event gives; and the webhooks'
  comes up from skein-forge-connection as its `listening` event (skein's
  `forge/connection.md`, 6.4).
- **Ready**: the engine at its `listening` lines
  and its sign-in link; a worker at its `linked` line. A task created
  before a worker's hello has reached the fleet waits for a free slot,
  so no ready line need say more.

## 4. Stopping, and what fails

- **Stopped in reverse**: each worker first, by
  its first termination signal, which runs its worker's ending: its runs
  drained, then stopped, and done once the engine has every turn and
  answer, or past its grace, or at its stop bound (jig's
  `protocol/service.md`, section 6; `../domain/worker/README.md`,
  section 2), exiting 0; then
  the engine, drained, exiting 0; then the fakes. A process not ended
  within its cost class's `stop_seconds` (5) is killed, its tree with it,
  and the stop fails.
- **No survivors:** each process in a cgroup of its own, its agents and
  git within it; a stop is done once its cgroup is empty, and a process
  left in it is a survivor, killed and reported.
- **A worker lost,** its process killed mid-run, then its tree (the
  worker first, so it cannot report its agent's end as an agent's
  failure): the engine's fleet keeps
  its runs for its grace, then they are lost and their tasks retry, on a
  worker with a free slot (jig's `domain/engine/fleet.md`). The
  lost attempt is fenced, so nothing it may still have sent counts, and
  nothing lands twice.
- **A worker hung,** its tree stopped by `SIGSTOP` past the engine's
  grace, then continued: its run has been placed again; it links again,
  and the attempt it held, fenced, lands nothing.
- **A worker restarted** on its own directories links again at its
  position and takes new runs.
- **The engine ends the cluster.** An engine that exits, or is killed,
  under live workers ends the cluster's run: it is never restarted under
  them (`README.md`, section 11). Its workers, past their grace, cancel their runs
  and are stopped; an agent whose worker has gone sees its pipes close,
  and ends.

## 5. Its description

- **One fixture, `bin/temper-box/tests/fixtures/cluster/`,** whose
  description, `experiment.json`, has the format `lab-experiment/1` and
  the cost label `temper-m1-calibration`. Each process keeps its `argv`, whole `env`, `ready`
  lines (3), `after`, `memory_bytes` (1) and `files`: its
  document rendered from a template, and its synthetic secret or digests.
  Templates name addresses by `{proc.P.ready.C}` and directories by
  `{proc.P.dir}`. The description is canonical JSON, with integers
  within ±(2^53−1); each process's `memory_bytes` matches its document.
- **The process tier's reader,** `bin/temper-box/tests/support/cluster.rs`:
  templates rendered; processes started by skein-world's `Binary`, in
  cgroups, by `after`, ties in name order, with no seed; ready lines
  awaited on streams; processes stopped in reverse, a termination signal,
  then the stop bound, then the tree killed. It reads none of the
  description's `actions`, `faults`, `collect` or `checks`, which are
  empty.
- **The reader's bindings:** `{artifact.temper}` is the absolute
  binary Cargo names to the test (`CARGO_BIN_EXE_temper`);
  `{artifact.git}` is its fake-git link; `{artifact.temper-engine}`,
  `{artifact.temper-worker}` and `{artifact.temper-agent}` are the
  deployment's binaries beside it, bound by the story on them alone
  (6); `{experiment}` is this fixture's absolute directory;
  `{proc.P.dir}` is P's scratch directory, made `0700`; and
  `{proc.P.ready.C}` is a capture once P is ready. A seed, run directory, browser or other unknown binding
  fails the test.
- **Bounds:** readiness is 120 seconds per process from its
  start, stopping 20 seconds per process. The stop bound stays above
  each worker's stop bound and the engine's drain. The reader's
  constants and the fixture's memory declarations are tested.
- **The fake forge's setup** is `temper-box serve`'s handover and reconcile
  at start, from records (setup/owned.md, 5). The fixture holds the fake
  forge, fake LLM, engine and workers; a story supplies its fake setup
  (`../testing/process-tier.md`). Its tasks are chats, its referee checks
  their ends on named workers and no survivor. Each chat keeps a worker's
  slot by an idle threshold and ordinary time past the story's deadline,
  never by timing. The forge starts before the engine.
- **The reader is temper's** until a second product needs one
  (`README.md`, section 11). Shared data keeps its process stories on one fixture.
- **Cgroups, or no cluster:** several processes observed by one test
  need skein-world's cgroup trees, where the user's service manager
  delegates them; walking refuses a second observer. Without them, the
  cluster's tests fail at their start, saying why.

## 6. Its tests

Each where `../testing/README.md` puts it (2.4 to 2.6):

| Where | What it shows |
|---|---|
| step tests | each shape read and checked, a document of none refused, naming its path; each line of 3 as printed |
| the engine's domain | two workers at positions 0 and 1, one slot each: two tasks, one placed on each; one lost, its run placed on the other past the grace |
| the real loop | an engine alone and two workers alone, three services on one ring with their own directories, the workers dialling the engine's listener on loopback: a task on each runs to its end |
| the process tier | `bin/temper-box/tests/fixtures/cluster/` on real processes, as below, and case 4.6's restart boundary (`../testing/reference-case.md` and `../testing/process-tier.md`) |

The process tier's first stories, one slot on each worker: the
cluster's fixtures give each worker one. Two tasks on two workers need
no more; the cluster's workers of several slots are later (7).


- **A task on each worker:** two tasks on the fakes, one run on w1 and
  one on w2, each to its end; then the cluster stopped, each process
  exiting 0, no survivor.
- **A task on each worker, on the deployment's binaries:** the same
  story on the same description, its engine started by
  `temper-engine serve`, each worker by `temper-worker serve`, and each
  worker's agent `temper-agent AGENT.json` (configuration/README.md, section
  4) in place of the box's `temper-box internal agent`; the fakes and the
  client stay the box's `temper-box`. The process tier builds the three
  beside the box's binary (`tools/tiers/design.md`). Written and
  compiled in its plan, and run after landing
  (`../draft/deployment-binaries.md`, section 10).
- **A worker lost:** w1 killed mid-run; its task's run is placed on w2
  past the grace, and lands once. Its process is killed first, then
  its tree, so the engine sees its link close, not an agent's end, and
  no silence is waited on. A waiting
  run's turn is committed only when the run ends, so the killed
  attempt's reply never reaches the engine; the retried run resumes
  from the chat's committed turns and answers anew, the reply is in
  them once, and the lost attempt's run ends as lost in the task's
  activity. The killed process's end is the story's, not a failure.
- **A worker hung:** w1 stopped past the grace, then continued; its run,
  placed on w2, lands once, and w1's attempt lands nothing. First, a
  stop past the engine's silence alone, continued within the fleet's
  grace, heals: w1 links again and the same attempt answers there,
  nothing placed twice. Then a stop held until the run is seen on w2
  (never a length of time): once continued, w1 links again, its
  listing of the fenced attempt is cancelled and what it sends of it
  dropped, and w1 takes a new run on its one slot, which shows it
  acted on the cancel, whose end sent the fenced attempt's held turn;
  the fenced attempt's activity and the chat's turns are as they were
  before the continue, and the reply is in them once.
- **A worker started later:** w2 started once w1's slot is taken; the
  second task waits, due and unplaced, with no attempt (the engine
  admits a run only once a slot is free), then runs on w2 once it
  links. A restarted worker links again at its position: w1, its run
  ended, is stopped alone, exits 0, and is started again on its own
  directories; it prints `linked` again, is the engine's w1 again, and
  takes a new run.
- **Every story waits on what shows each step,** read through the
  client and the processes' lines, never on a clock, within one bound;
  w2 starts only once w1 holds its run, so which worker takes a chat is
  never left to placement's order; and each chat is closed and ended
  before the cluster stops, since a worker's ending under a waiting
  run fails it into a retry.
- **A mixed document,** each shape's departures, refuses its start,
  exiting 1 and naming the path.

These are the case's multi-worker steps at their smallest
(`../testing/reference-case.md`): `binaries.md`, 1's placement by free slots, `binaries.md`, 3's
worker added later, 4.6's hung worker.

- **The case's restart,** a story on the same
  description, the case's workers named ws, s1 and s2, its fixture
  applied to the engine's and the workers' documents, its acts by real
  `temper ctl`, and its oracle over what the processes observed
  (`../testing/process-tier.md`): 4.6's restarted
  workstation, ws stopped under the director's waiting chat and started
  again on its own directories.

## 7. Later

- **A worker stopped apart from its engine** (4): its Shutdown
  cancel's agent answer counts as `Failed(Agent)`, spending an agent
  try while the engine stays up. Giving that restart the standalone
  stop's no-try interruption (`README.md`, section 6) is left for the cluster
  (jig's `protocol/service.md`, section 9).
- **The engine's refusal text on the worker** (3): the text is sent
  over the channel, but jig's worker link retains only the reason code
  for its note. Keeping the words for a worker's stderr needs a change
  through that link. Default: the engine prints the words and the
  worker prints its code, suppressing repeats as 3 says.
- **TLS on the workers' listener, contained process trees, and workers
  on other hosts,** with hosts apart (`README.md`, section 11): a worker alone on
  another machine is the same document, its `engine` in TLS (jig's
  `protocol/README.md`, section 10).
- **An engine restarted under live workers,** with generations kept
  across starts (`README.md`, section 11).
- **The cluster's workers of several slots,** as the case of
  `../testing/reference-case.md` has: a worker hosts an agent per slot
  (`../domain/worker/README.md`, section 5), and the cluster's fixtures and
  stories still give each of its workers one slot.
- **`temper-box init` for the cluster's shapes** (`README.md`, section 11).
