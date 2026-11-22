# Testing temper

Provisional, 2026-10-03, revised 2026-11-02. How temper is tested. The
strategy is skein's `../foundation/testing-strategy.md`: the tiers,
faults, what every fake shares, what every world checks, scenarios and
the referee, the two suites, and where a failure is fixed. This document
keeps what is temper's: its tiers (section 2), its neighbours and their
fakes (`README.md`, section 3 and `fakes.md`), its scenarios, referee and its case library
(`README.md`, section 4), what its worlds check beyond the strategy (`checks.md`; the
engine's, `engine-checks.md`; blame and tries for gates, `blame.md`; the worker's
waits, disk and faults, `worker.md`), its
layout (`README.md`, section 5), its tools and fixtures as binaries (`tools.md`), what
macOS tests (`macos.md`), QA after landing beside the deployment (`qa/README.md`), and what
is still open (`README.md`, section 6).

## 1. In one page

- **Behaviour is tested in the domain layer.** The engine and the worker
  are each a root domain with child domains, and each has worlds of its
  own; the system world runs the real engine and worker together, with
  smith's agents.
- **Below the domain, one tier per layer it adds:** step tests of every
  codec and translation with built-value round trips and records drift,
  and each parser against transcripts of real peers; machine worlds of the forge component, the git component
  and the terminal shell; the forge's protocol world; temper's client
  joined to the engine's root; the one-process world on skein's
  simulator; the real-loop world on one ring; reconcile's world; the
  process tier, the real binary with an owned Forgejo, its runner and
  real git; and the live tier, with a real model.
- **Peers across a wire, the machine beneath.** LLM providers, their
  sign-in's issuers and the forge are peers, whose fakes grow into
  services and meet temper at the wire; Claude Code, the program smith's
  agent runs on a program's login, is faked by smith's fake `claude`
  (`fakes.md`, 1); people use temper's client, and their fake is skein's fake
  person, in front of it. The machine (files, processes, and the
  programs they run: git, rg, sh, temper's own) stays one set of state
  machines, with a face for each layer of temper it can sit under.
- **The fake forge speaks Forgejo's dialect and, through its GitHub
  layer, GitHub's** (skein's `testing/fakes/github.md`, section 1), one world per
  forge kind, API and git in one store, so a branch the worker pushes is
  the head the engine reads. Its protocol layer serves the dialect's
  API; its git is reached by the fake git program, through the forge's
  own calls, never git's protocol over HTTP.
- **Checks follow content.** The machine's sh is a small subset of POSIX
  sh, in which fake repositories write their checks (`fakes.md`, 3), and the fake
  forge's CI follows a cue in the pushed tree (`fakes.md`, 2), so a change that is
  wrong fails in the run and on the forge alike.
- **A referee in every engine world,** on skein's world harness, holds
  its scenarios' expectations (4.2).
- **The case library runs in the real loop,** step by step, on its cast,
  its scripts and an oracle over effects that every world running the
  case reuses (`reference-case.md`).
- **No fake of temper's own components:** the system world runs the real
  engine and worker.
- **Two suites, and two tiers outside them.** The focused and the fuzzy
  suites have a CPU budget for each layer, held at the gate
  (`checks.md`, 1; `tools/tiers/budgets.md`); the process
  and live tiers run by their scripts, never at the gate. The process
  tier holds only what processes show, within its 540 CPU s (`process-tier.md`). The
  540 is the tier's 6-minute aim, raised by half by the user's word of
  11-08, for now, to be revisited.

## 2. temper's tiers

| Tier (skein/docs/foundation/testing-strategy.md, section 2) | In temper |
|---|---|
| step tests | every domain crate and every fake; every codec (records drift, built-value round trips, bounds, fuzz targets) and each translation both ways; git's argument and environment builders, and git's output parsers against transcripts of real git (../protocol/git.md, section 9); Forgejo's decoders, skein's, against the conformance's transcripts (skein's `forge/README.md`, section 9), and GitHub's, skein's (skein's `forge/github.md`); the forges' records and a git identity's value, built-value round trips and fuzz targets (../protocol/forge/engine.md, 1; ../protocol/credentials.md, section 4); `temper-service`'s routing of the forges: start, connect, stop, installations, generations and pasted values, and GitHub's three routes on the clients' listener (../protocol/forge/README.md, section 9; 2.1); the setup's machines, files and specs, and the box's machine (../protocol/setup/testing.md, 1); lib's are skein's |
| domain worlds | each child domain with the world as its parent, and each component's root domain with its child domains; most of temper's tests |
| system worlds | the engine, the worker and smith's agents together; temper's client joined to the engine's root (2.1) |
| machine worlds | the git component against scripted process ends and outputs; the terminal shell's loop against a scripted stdin and stdout; skein's, for the machines it ships, the forge component among them (skein's `forge/connection.md`, section 10) |
| protocol worlds | the forge's: the forge component against the fake forge's protocol layer, deliveries included; reconcile's is skein's (2.2) |
| io worlds | skein's: temper does not retest io |
| simulated worlds | the one-process world: `temper-service` on skein's simulator (2.3, 2.5) |
| real loop | the standalone service, hosted agents, native clients and fake peers in one loop over one real ring; the cluster's engine alone and two workers alone, in the same loop (`real-loop.md`) |
| end to end | the process tier: the real binary, an owned Forgejo and its runner, and real git; the cluster's processes on the fakes as processes (`process-tier.md`) |
| end to end, live | the live tier: the process tier's stories with a real model (`live-tier.md`) |

Every child domain has a world of its own but the engine's rules, which
keep no state and are tested by their step tests alone: a departure from
skein/docs/foundation/testing-strategy.md, 2.2, since with no state there is no sequence for a
world to drive. The focused suite has one short path through the shipped binary on fakes
(`process-tier.md`). The process profile runs owned Forgejo stories outside both suites,
since provisioning Forgejo exceeds a suite's budget. Its other stories
need OS process boundaries: a real signal, a process tree, cgroup
supervision, or a binary interrupted during account transfer. These are
explicit process-profile exceptions; their domain behaviour stays below.

The tiers in the two suites keep the budgets of `checks.md`, 1; the process and the
live tiers, outside them, keep their own (`process-tier.md`, `live-tier.md`). The browser tier,
a slow tier outside the gate, proves only what a browser
alone shows; temper's stories stay in the worlds and the real loop
(../client/web/testing.md).

No tier calls github.com or uses a real credential. GitHub's tests run
on the fake's GitHub layer (skein's `testing/fakes/github.md`, section 1). Live
calls to GitHub are only skein's opt-in conformance (skein's
`testing/fakes/github.md`, section 2), run by hand on credentials from the person's private
directory, never in a suite or at the gate. The user's own token is never used.

The fake-backend welcome's configuration and issuer contracts are tested
at their lowest owning layer (../protocol/configuration/directories.md, 1;
../protocol/setup/box.md, 5): CLI and filesystem preparation in temper's unit
tests, document rewriting and issuer plans in temper-fake, and welcome
acts in the client and setup worlds. Compatible retained edits remain
untouched; a changed source requires explicit reset. The focused binary
wiring proof is `suite::wiring::a_scratch_binary_answers_one_chat_and_drains`.
The owned welcome walk combines its key and declared-program variants on
one owned Forgejo. It keeps private fixtures and ordinary teardown. Once
it reaches Ready, a native client with the browser's client bounds
(`../client/README.md`, 2) presses "Create your project" at the welcome's Ready
and reaches the project's Home; a second such client, the reload, still
receives its watches: Live, never Behind, the walk's forge on the Forges
page and its key on the Accounts page.

### 2.1 System worlds

One runs the real engine driving the real worker: the composed system
world (`tests/system`), whose worker hosts smith's real agent domain over
skein's fake LLM, and whose engine and worker's git meet one fake forge.
It aims at what crosses components: the tree an agent left is the tree the
worker pushes, and the outcome the run accepted is the one the engine
records. Between the two roots it uses the protocol layer's own
translation, without bytes (../protocol/channel.md, section 7). With a
GitHub forge on the fake's GitHub layer, it checks the forge connector's
connected state, the picker and GitHub's merge queue as the branch's
landing queue (`../domain/forge/README.md`).

A change's merge job has its story there (`../domain/forge/change/repairs.md`; `../domain/worker/jobs.md`
), one table, `suite::merge_job`: a project naming a `merge_job`
makes the job, the worker that declared the kind takes it, prepares the
head with the base merged in on the fake disk, runs the command, and
delivers or not, and the engine goes on from its branch. The command is
the world's: it reads the job's request and the index on the fake disk,
settles and stages what its row says, writes its result and exits.
Rows: a clean merge (a catch-up of main into a goal's branch); a
conflict it settles; one it leaves, which the change takes to a
resolution; a base commit only on a branch the job's cached checkout has
not fetched; and more files in conflict than the checkout's
`conflicts`, all settled. Each row reads the forge's branch and its
parents, the job's task ended and the change's next step. Rows of the
worker alone stay the worker's world's (`tests/worker/domain`).

A run's live text has its story there: a chat's reply reaches the
party's run watch before its turn commits. The core asks for the chat's
text, and smith's start carries the ask. smith's agent domain gives its
reply; the world, playing smith's process protocol as it plays every
record of the agent's channel, sends that reply up the channel in three
pieces ahead of its turn, and holds the second piece, the third and then
the turn itself until the watch has read all the text before it. The
worker's root hands each piece to its hub, and feeds the agent's
watchdog with it; the engine's root keeps it as the run's text and
streams it to the watch, which the world opens and acknowledges as
temper's client does. Once the turn goes, it commits, and its text is
the text the watch read. smith's own producer, the fake LLM's named
holds and the bytes between the roots are jig's channel story's (jig's
`protocol/channel/README.md`, section 11); the same reply in temper's client is
`checks.md`, 3's and `real-loop.md`'s.

A chat's reply of text alone has its story there, with its Finite
sibling. The chat, Open, is answered by skein's fake LLM with text and
no `finish` call: one model call, one committed reply, the chat parked
with no try spent, and past its idle wait no second call, assignment
or reply (`../domain/agent/charter.md`, 5). A task ended by its result, as
every goal and delegation is, given the same text alone, is nudged
toward `finish` by smith and finishes with its report on its next
call, with no try spent. The parking and the nudge are smith's and
jig's own worlds'; this story holds temper's choice of mode for each
(`temper-engine-smith`'s start) to them.

temper's client joined to the engine's root, `tests/client`
(`temper-client-world`), is a system world too: temper's client root and
view, skein's fake person reading the view's tree, jig's client worlds'
faults and jig's check, and temper's engine root on the engine world's
store, hosts and forge (../client/testing.md). `checks.md`, section 3 says
why it is joined. GitHub's three routes on the clients' listener (the
manifest page, the redirect and the setup address; ../client/forge/served.md,
1) are not tested here: they are `temper-service`'s own tests, as
requests through `fixtures::http::Peer` to the service's routes: each
answer, its headers, and a state used, unknown or past its life
refused.

### 2.2 Protocol worlds

Each component's protocol layer meets the protocol layers of its peers'
fakes, joined by in-memory streams cut and joined at random, and the
machine sits there at its io face (3.2). They test what translation adds
in temper.

- **The forge's,** `tests/engine/protocol-forge`: the engine's forge
  component against the fake forge's protocol layer, running the forge
  connector world's stories (`tests/engine/forge`) through bytes, with
  the cost checks counted by HTTP request, and the fake forge's signed
  deliveries to the engine's webhooks' listener, late, out of order or
  never (../protocol/forge/README.md, section 9). The routing is not here:
  start's load of each forge's records, a connect at runtime, a forge
  stopped, GitHub's installations, git's tokens' generations and pasted
  values are `temper-service`'s own tests (../protocol/forge/README.md, section
  9). On GitHub's layer, a change is checked by its head's check runs,
  pending then passed, merged at that head and landed; a failed run is
  a repair (../domain/forge/keeping-up.md, 3 and ../domain/forge/change/repairs.md). The joint's GitHub stories hold the
  component's half of git's
  tokens under temper's settings: every accepted installation's token
  before the forge's first `Connected`, and each renewed before it
  expires on the fake's clock, at GitHub's hour (../protocol/credentials.md,
  section 4).
- **Reconcile's,** in skein (skein's `forge/setup/README.md`, section 5):
  reconcile's machine against the fake forge's protocol layer serving the
  setup's routes, through bytes: a forge converged from empty and again,
  each thing deleted made again, the refusals, no secret in a message or a
  log (../protocol/setup/testing.md, 2).
- **The worker's link and the client's connection** have their protocol
  worlds in jig (jig's `protocol/channel/README.md`, section 11; jig's
  `protocol/client/README.md`, section 11), and an agent's channel and its LLM
  calls theirs in smith and skein. temper's parts and records cross them
  as bytes those worlds never read; the one-process world carries
  temper's through bytes (2.5).
- **git has no protocol world:** it is a program, not a peer on a stream.
  Its tiers are the git component's machine world, the fake git program
  and real git in the process tier (../protocol/git.md, section 9).

### 2.3 Simulated worlds

temper's simulated world is the one-process world (2.5). Every process
in a scenario runs its `iterate` on skein's simulator: `temper-box serve`'s
service, the agents it spawns, the fake forge as a service, the fake LLM
peer and the person's client. A spawn the worker asks for starts an agent
process or the fake git program, and a kill ends it. The machine sits
behind the simulator (3.2).

### 2.5 The one-process world

`tests/one-process`, after jig's (jig's `protocol/service.md`, sections 8
and 10). Its composition is `temper-playground`'s, the world the
playground runs in a browser (`../playground.md`, 3), which this world
composes with its harness: the referee, the fake person and the
terminal client, the captures and the faults.

- **What runs,** on skein's simulator and skein's fake machine:
  - `temper-service`'s system module, the engine's and one worker's
    services in one `iterate`, started from configuration's bytes as
    `temper-box serve` starts them, joined by the in-process name `workers`;
  - the store, jig's store component on skein-kv, on the simulator's
    crashing disk;
  - the worker's agents, smith's agent processes as `temper-box internal
    agent` runs them, hosted as programs of the world, over skein's fake
    LLM peer;
  - the fake git program for every git invocation (`fakes.md`, 3);
  - the fake forge as a service, its protocol layer over its domain
    (`fakes.md`, 2);
  - people: skein's fake person in front of temper's terminal client,
    its domain, view and jig's native transport (../client/README.md, section 2;
    `fakes.md`, 4).
- **Journeys' prints** use this composition too: the browser's shipped
  host and the TUI's shipped host on the simulated service, with no box
  and no terminal (`../client/journey-walks.md`, 3.2, 3.3 and 4). The
  walker feeds their lines and prints their marks as pages.
- **Stories:**
  - a person signs in by the link `temper-box serve` prints, and a chat is
    answered;
  - a provider's list read by the worker's hosted list program (jig's
    `domain/engine/settings.md`, 12): asked once, no run, its new name
    drawn in `me`, nothing spent;
  - a person still signed in signs in again by a fresh link, with the
    same directory, and keeps their sign-in: its number, token and
    expiry, and the deployment's sign-ins unchanged
    (../client/README.md, section 4);
  - a repository is adopted through temper's client, and a goal's change
    is prepared, committed and pushed on the fake forge through the fake
    git program;
  - a merge in progress is resolved and pushed with both parents;
  - on GitHub's layer (skein's `testing/fakes/github.md`, section 1), its App
    seeded as the real loop's are (`real-loop.md`): a goal's change cloned by its
    installation's token, the token renewed `renew_before` ahead of
    GitHub's hour on the simulator's clock while the run holds its
    workspace, git's generation changing under it, and the push
    authenticated by the renewed token, as the layer's record of its git
    calls shows (../protocol/credentials.md, section 4);
  - a signal mid-run drains the engine, the worker ends, and the next
    start settles the worker's claims at once, on a chat's run and on a
    coding run;
  - a stop while a chat's run holds a turn of smith's fake `claude` on a
    program's source: the turn interrupted, the engine drained, and no
    program left running (`../protocol/setup/box.md`, 2); a program's
    own cancel is smith's agent-process world's;
  - the service is killed, or the machine's power lost, at a drawn
    moment, then started again on the same disk, an adoption pending
    among what it finds;
  - those `../protocol/channel.md`, 9, `../client/testing.md` and
    `../protocol/sign-in.md`, 2.5 place here;
  - **through an upgrade** (`../draft/upgrades.md`, section 13): a
    project, a person and an account; a goal's change landed; a chat
    paused and a task mid-run on the in-process worker; a forge write
    left uncertain by a fault; then a settled stop, and a start of the
    same build handed `CURRENT_SETS` after the story used `first_sets()`
    (`../protocol/floor.md`, 7): the production levels unchanged, a
    registered receipt layout gaining its defaulted `report` and the
    job `deleted-repository-forge-rows` pending. `temper-service`'s
    `Service::owner_sets` hands them. The open commits the registry
    and runs the job in a copy, restores before the switch, switches
    and commits its wires. No fresh mark closes the live windows;
  - the same start rehearsed first (jig's `protocol/store/opening.md`, 5),
    reporting the job and the restores whole and leaving the
    directory as it was, then made; and a start failed by the world
    after `Switched` and before its wires are durable, gone back from
    (`Store::back`), the build before it opening the generation as it
    was;
  - the stop unsettled instead, the service killed with the forge write
    in flight: the upgrade's start registers the layout and runs the
    job, and recovers the write as after any stop,
    made once;
  - power lost at a drawn moment of that start, from the copy to the
    wires' commit: the next start finds one whole generation and goes
    on as above. Focused draws one moment; fuzzy sweeps drawn moments
    and the cuts at the copy's first write, the upgrade's seal commit, the
    switch's rename and the wires' commit, each a whole story. The
    store's own cuts there are jig's store world's (jig's
    `domain/testing.md`, 9). These cases and the settled stop start
    from the settled image (`../protocol/floor.md`, 7).

  The upgrade's referee compares the history at the stop with the
  history once the start's wires are durable, before anything acts:
  record by record (people, projects, accounts, tasks live and ended,
  landings, kept outcomes, each forge's records, each forge's secrets),
  equal but for the registry's new receipt layout, the job's done row,
  the commit position and the start's wires clearing the previous
  worker opening. No receipt or deleted-repository row is written by
  this story. Then, as the world goes on: claims settle; no forge write made
  twice, by the fake forge's ledger, kept across the service's
  restarts; a request sent again under its key answered from its record;
  every number given above every number kept; the mid-run task's next
  attempt continues without a mark closing its window; the paused chat resumed.

  The world's seams for it: `Service::owner_sets`; a fault the world
  injects between `Switched` and the wires' commit; the fake forge's
  ledger outliving a restart. Of the draft's section 13, the faults
  below the store (a full disk, a short write, a failed sync, a
  malformed or refused record, a limit too small) are jig's store
  world's (jig's `protocol/store/README.md`, section 9). An effect in
  flight is not an upgrade refusal. The floor's golden store
  is `../protocol/floor.md`, 5's, kept here; the engine alone (its
  workers kept across the engine's upgrade) waits for wires that can
  differ.
- **Faults:** kills and power loss at drawn submissions; the fake
  forge's and the fake LLM's faults, drawn from the seed; the forge's
  refusals and failures as the fake git program prints them. Joined
  sockets are io's own, out of the simulator's faults (skein's `runtime/io/sockets.md`,
  5), so the link's cuts and stalls are jig's channel world's, not
  this world's.
- **Its referee:**
  - jig's conformance referee, through bytes, as jig's one-process world
    has it: nothing a party, a host or the forge saw depended on a commit
    that was lost, and a drained shutdown ends with nothing waiting for a
    commit;
  - the engine's referee over the fake forge (4.2): nothing lands on a
    default branch without green CI, and a run's change is the tree its
    agent left;
  - no git identity's value or API token in a domain's state, a store
    record or an agent process's environment.
- **Budget:** a story aims at 2 s (`checks.md`, 1); sweeps of drawn kills are fuzzy
  tests. The upgrade module's tests are placed as `../protocol/floor.md`, 7
  lists.
  Each story, each run above among them, is a test of its own;
  Anthropic's keys through `ctl` are `tests/client`'s
  (`../client/ctl/testing.md`, 2), as is the template's paging story; a
  worker's opening with chosen wires is `temper-worker-service`'s
  (`../protocol/wires.md`, 11).

### 2.8 macOS

What macOS tests is `macos.md`'s: one workflow, outside the gate, on
GitHub's `macos-26`, dispatched after landing at a commit of main for a
set of parts, each a job: the process tier's stories, the one story among
them, its Forgejo and runner fetched first through `temper-box forge
fetch`; the browser tiers; each tree's focused and fuzzy suites; and the
live stories on the user's key. One JUnit report names the failing tests
by tier main's ids, a build that fails among them, and fabrig's forge
hands it its macOS tiers' runs. Linux's own tests are listed by name.

## 3. Neighbours and their faces

### 3.1 Peers

A peer's fake grows the layers that the temper component facing it
grows, and each tier joins the two at the lowest layer both have
(skein/docs/foundation/testing-strategy.md, 4.1):

```
smith's agent  domain ─ protocol ─ io   ⇄   io ─ protocol ─ domain   skein's fake LLM
engine         domain ─ protocol ─ io   ⇄   io ─ protocol ─ domain   skein's fake forge
```

People are no peer of temper's: they use temper's client, temper's own,
which reaches the engine over jig's client protocol, and their fake is
skein's fake person in front of it (`fakes.md`, 4). A peer can be a client as well as
a server: the fake forge's protocol layer sends signed deliveries to the
engine's webhooks' listener (../protocol/forge/README.md, section 4; skein's
`testing/fakes/forgejo.md`, section 1); and reconcile meets it as the engine does,
through the setup's routes (../protocol/setup/testing.md, 2).

### 3.2 The machine

The machine's fake grows no protocol or io layers, and shows a face to
whichever layer of temper sits just above it (skein/docs/foundation/testing-strategy.md,
4.3). Its domain face answers the tools' file operations, commands and
searches and the checkout's git operations; its io face, io's records,
a spawn with an argument vector among them. Every component's files and
processes go through it: the worker's workspaces and agent processes,
the agent's tools and checks, and the engine's store.

## 4. Scenarios

### 4.1 Setting one up

A scenario (skein/docs/foundation/testing-strategy.md, section 7) is set up in the terms of
temper's fakes: each conversation's script for the LLM, the repositories
on the forge with their files and checks, the items and the people, and
the faults to inject. A scenario cues its scripts through content: a
repository's files say which script its run follows.

A scenario's limits are the shipped profile's, `small`, built by
`temper-service`'s limits module (../protocol/configuration/limits.md),
in every world, fixture and process-tier configuration: none is set by
hand. A scenario that needs another value changes it by name on the
profile's, with the reason beside it. A limit set by hand hides what the
shipped profile refuses, and the shipped numbers drift toward the
fixtures'.

### 4.2 The referee

The referee is the strategy's (skein/docs/foundation/testing-strategy.md, section 7). temper's
expectations are of this kind: safety, such as nothing landing on a
default branch without green CI, or no forge credential reaching an
agent; liveness, such as a pull request opened, with green CI, within
the hour. What temper adds:

- **It steps on what it observes,** and on its own timers, not on every
  iteration, so its work per iteration stays bounded. Its earliest
  deadline is covered by the shell's one ring timeout, as every layer's is
  (skein's `../foundation/programming-model.md`, section 9).
- **A failure in the real-loop, process or live tier** is reported with
  the trace and the files the run left, since the seed does not replay
  there.
- **Its checks return their findings,** as skein's do (skein's
  `testing/README.md`, 5; skein's `../foundation/programming-model.md`,
  10.4), and nothing in temper's tests catches a panic:
  - every referee and judge of temper's worlds implements `watch`, in
    every tier, and keeps what its `act` or `idle` finds for its next
    `watch`; the real loop fails with the finding after its forced
    cleanup, and a check's own test reads it through the world's
    `try_run`;
  - the one-process world's durable observer returns a current
    generation's row that doesn't decode as its finding, which its
    referee returns; an old generation's rows stay opaque until the
    switch (2.5);
  - the real loop's checks of a call's answer within its bound, of a
    CLI timeout and of an overdue deadline with its diagnostic and
    figures, each a returning form beside the assertion a story calls
    at its top; a starved wait still passes as starved
    (`real-loop.md`);
  - the engine replay's digest and text comparisons (`checks.md`), the
    fake git program's replay order (`fakes.md`, 3), the browser
    stories' attribution (`../client/web/testing.md`, 3), the process
    tier's printed link and cluster members (`process-tier.md`), and
    the GitHub stories' secrets scans and teardown (`live-tier.md`);
  - the engine's domain world's cold restart oracle: its comparisons of
    the rows, the answer and the forges a restart reads back, and the
    erasing close, return the first difference, naming its key, and the
    world's close and the conformance world's violation carry it to
    their top; the walking world's delivery of a result its referee
    refuses, which the world keeps for its run;
  - the worker world's referee of turns (`checks.md`, 2): a replay that
    changes a turn's body or spend, or a turn past one never seen, is a
    finding the world keeps for its run;
  - the one-process world's observer of saved artifacts, whose scan for
    a newly issued token returns its finding through the observer's
    close (2.5);
  - the real loop's `Until` step, which an observation that failed or
    was signalled ends with its finding, whatever its stdout
    (`real-loop.md`);
  - temper-box-kit's ctl checks that a child's prerequisite succeeded
    before its output is read or polled again, and that a refused
    adoption stops what waits on it, kept for its judge's `watch`; and
    the fake box's readiness, which returns the child's exit status and
    stderr when the child ends before it is ready;
  - the port chooser's refusal of an ephemeral range that leaves fewer
    than two blocks, naming the range (`process-tier.md`), which its
    caller panics on;
  - a harness that names a seed, a walk, a case or a story's step
    writes it to stderr before it runs it, and a table of cases stops
    at its first failing case, having named it. The runner prints a
    failed test's output.

  No temper test triggers an assertion: what an assertion guards is
  tested through the outcomes around it (skein's
  `../foundation/programming-model.md`, 10.4).

The referee is built on skein's world harness (`skein-world`; skein's
`testing/README.md`, section 5), as ordinary Rust: each world writes its
scenario's expectations, with the stimuli it may inject, and the
harness judges them.

Every engine world has a referee, the engine's own included, over what
the forge did and what the engine assigned. Its stimuli are an engine
restarting cold: at a drawn moment; at once as it posts an outcome, so
that it restarts while applying it; or at a moment a scenario chooses as
the forge shows it. The worker's worlds add a worker's channel dropping,
a worker vanishing and a worker told to shut down. The engine's referee
meets a story only when its item closes. The system world checks its
stories independently of the roots, against durable task and call
records and smith's own observations (`tests/system`). The conformance
world (`tests/conformance`) runs the engine root on jig's conformance
harness, under its outside referee. Its fuzzy sweep crashes each live
commit of a story once, every cut forked from the one baseline (jig's
`domain/testing.md`, 5). Its stories' and scenarios' heap
ceilings are three times the figure skein's ../foundation/testing-strategy.md,
section 6 gives (the measured peak plus a quarter, rounded up), the
user's, 2026-11-03: an engine running a workload of many goals holds
tens of MB, so a quarter's headroom over one story cost a repair at
each milestone's added state, while three times still catches a leak.
The worker's child domain worlds keep their checks inline.

**Beside the referee, the harness's invariants.** The referee never
reads a domain's state. What must hold of a component's frozen state, its
waits and its bounded tables, is a harness invariant: the world reads the
component through its inspection adapter at its driver's quiet point and
returns the shared checks' findings as its run's (skein's
`testing/inspection.md`; `checks.md`, 6.1). A world that injects faults
records each in its run's fault ledger, draws its workload, faults,
parties and scheduling from independent streams of its seed, and a sweep
over its seeds fails when a fault it configures never falls; its only
exclusions name ignored scenarios (skein's `testing/sweeps.md`;
`checks.md`, 6.2). Every engine driver calls the engine's world kit at
its quiet point, and its referee holds that the goals a scenario names
end, or are held with words, within bounds in simulated minutes and in
steps, and that the engine opens within a bound after every restart
(`engine-checks.md`).

## 5. Layout

```
crates/*/src/tests.rs               step tests
tests/codecs                        temper-codecs-world: the generated codecs and the foreign documents' decoders
bin/temper-box/tests/domain          step and domain harness checks over values, committed fixtures and scripted client-tool reads
bin/temper-box/tests/suite           real scratch files, sockets, locks, clocks, shell children and shipped wiring
crates/temper-forge-setup-shell/tests/step  process-boundary checks over committed source and sample calls; suite keeps real files, locks, commands and TLS peers
bin/temper-box/tests/process.rs      owned Forgejo integration and OS process boundaries (2.6); nextest profile process-temper
bin/temper-box/tests/proof_live.rs   B0's offline driver proofs (2.6, 2.7), nextest profile process-temper; scripts/proof-live.sh rung-b0-offline runs them
testing/temper-box-kit              what temper-box's test binaries share, compiled once for them all (tools/crates/design.md, 3.2): the stories' processes, forges, people and fixtures
testing/temper-box-kit/src/github.rs   GitHub's App as processes (2.6): the fake GitHub, the fake LLM, an engine alone and a worker alone, the App made on the layer and connected through ctl
testing/temper-box-kit/src/cluster.rs  the cluster's reader: `bin/temper-box/tests/fixtures/cluster/` started, awaited and stopped on skein-world's `Binary` (../protocol/cluster.md, 5)
testing/temper-process-guard        the kit's process guard: step observes scripted read/exit results, parent observes real /bin/cat identities
bin/temper-box/tests/fixtures/cluster  the retained cluster's description, read by the process tier
bin/temper-box/tests/fixtures/github   the GitHub App's documents, its key and its workers' secrets, read by the kit's GitHub stories
bin/temper-box/tests/live.rs         the live tier (2.7), nextest profile live-temper
bin/temper-box/tests/live_github.rs  the live tier's GitHub story (2.7), run by scripts/proof-live-github.sh
bin/temper-box/tests/live_github_managed.rs  the managed GitHub sitting: its Ready on the fake, nextest profile process-temper (2.6); its relay against github.com, owner-run (2.7)
tests/client-browser                temper-client-browser-world: the browser tier (../client/web/testing.md), Chromium against temper-box serve on the bundle scripts/browser.sh builds; nextest profile browser-temper
crates/temper-client-shell-terminal/tests  the terminal's machine world (../client/testing.md)
crates/temper-fake-git              the fake git program's core: temper-box fake git (4.3)
crates/temper-fake-git-hosted       the fake git program hosted by skein's simulated world (4.3), which the playground ships; its fixtures with the tests
crates/temper-playground            the world's composition, the playground's and the one-process world's (../playground.md, 3)
crates/temper-playground-serve-tool the browser tier's static host for the playground's bundle, called by bin/temper-playground-serve (tools.md, 3)
testing/temper-github-app           the App on the fake GitHub layer's first fixed key, its records, seeded in jig's secrets' store before a start (2.4)
testing/temper-fixtures             the documents, transcripts, memory literals and local browser that temper's client crates and worlds share
testing/temper-agent-fixtures       the agents' configuration as the worker's writer makes it
testing/temper-landing-fixtures     a project's landing settings, typed, for the engine's and client's worlds
testing/temper-engine-inspect       the engine's inspection adapter, read by its worlds alone (engine-checks.md, 2)
testing/temper-blame                the check of blame and tries for gates, over a world's fault ledger (blame.md)
tests/<component>/<child>           a child domain's world, temper-<component>-<child>-world
tests/<component>/domain            a component's world: the engine's, the worker's
tests/engine/forge                  the forge connector's world
tests/engine/protocol-forge         the forge's protocol world (2.2)
tests/worker/protocol-git           the git component's machine world (../protocol/git.md, section 9)
tests/system                        the composed system world (2.1)
tests/conformance                   the engine root on jig's conformance harness
tests/client                        temper-client-world: temper's client with skein's fake person, joined to the engine's root (2.1, 6.3)
tests/one-process                   the one-process world (2.5)
tests/playground                    the playground's stories walked natively through the client's host, and a session log replayed (../playground.md, 9)
tests/real-loop                     the real-loop world (2.4), in the focused suite
tests/*/*/tests/suite/*.rs                a world's focused tests
tests/*/*/tests/suite/fuzzy/*.rs          its fuzzy tests: random worlds, domains driven at random
tests/fake-checkout                 temper-fake-checkout: working trees' git, over skein's fake checkout
crates/temper-twin-tool             the live run's twin: bin/temper-twin, selected transcripts as a fake LLM document (fakes.md, 1)
crates/temper-qa-tool               QA after landing: bin/temper-qa, main's landings triaged into cases and builds kept for them (qa/README.md); step holds values and seeded ledger sequences, suite keeps real scratch homes, sockets, locks and children
crates/temper-git-transcripts-tool  git's transcripts, recorded by bin/temper-git-transcripts against skein's throwaway Forgejo (../protocol/git.md, section 9)
crates/temper-client-gallery-tool    bin/temper-client-gallery: the browser's pages in named states, printed for screenshots (../client/web/gallery.md), its tests in the view's world
crates/temper-client-journeys-tool   recorded walks replayed through the shipped hosts and printed (../client/web/journeys.md; ../client/tui/journeys.md), run by bin/temper-client-journeys, its tests on walks they record
crates/temper-stage-one-tool        bin/temper-stage-one: configuring's stage 1, the golden, its check, its judge's brief and the evidence's collection (6.5), its step tests in the focused suite
scripts/proof-live.sh               the live tier
```

- **The worlds and the harness** are skein's world harness
  (`skein-world`); each world's crate is a library other worlds may use
  (`fakes.md`, 5).
- **The machine-level fakes are skein's:** the fake machine
  (`skein-fake-machine`), the fake checkout, the fake LLM and its peer,
  the fake issuer. temper's fakes sit on them.
- **The fake forge is skein's** (skein's `testing/fakes/forge.md`): its domain
  (`crates/skein-fake-forge-domain`), its protocol layer
  (`crates/skein-fake-forge-forgejo`), its forge peer (in
  `crates/skein-fake-peers`), the throwaway Forgejo
  (`testing/skein-throwaway-forgejo`), its tests in ordinary Rust, and
  the conformance (`testing/skein-forge-conformance`). Reconcile's world is
  in skein's `forge/setup/README.md`, section 5. temper's fake git program,
  its core in `crates/temper-fake-git` and its hosted half in
  `crates/temper-fake-git-hosted`, sits on the fake forge (`fakes.md`, 3).
- **Machine worlds live beside what they test,** as the layout places
  them (../protocol/forge/README.md, section 9); the forge component's is
  skein's (`forge/connection.md`, section 10).
- **The setup's crates** test as step code does: `temper-forge-setup`'s
  files, specs and steps, and `temper-box-machine`'s machine, by their step
  tests (../protocol/setup/testing.md, 1); reconcile's machine by skein's world;
  the hosts, `temper-forge-setup-shell` and `temper-box-machine`'s own, by the
  process tier (`process-tier.md`), as the binary's are.

## 6. Open questions

- **Real git against the fake forge.** How to connect the real-loop
  world's git subprocesses to the fake forge while retaining one
  authoritative repository state. A candidate is real bare repositories
  in the scratch directory, with an adapter for git access and the fake
  API's reads and merges. The design must keep object identities, refs,
  protection and merge results consistent, and reuse the fake's policies
  and scenario expectations. It is designed against skein's fake forge,
  in skein's `testing/fakes/forge.md`, section 3. What needs it: a real agent on
  the fake forge, which runs git in its working tree as a person does;
  nothing else does, since the fake git program reaches the fake forge
  across processes too (`fakes.md`, 3), and real git's own behaviour is the
  transcripts' and the process tier's. A likely shape: the domain stays
  authoritative over refs and policy, the bare repositories mirror it,
  the protocol layer keeps a table of the domain's counts to git's ids,
  a push's ref updates are decided by the domain before git writes them
  and its tree imported as the domain's commit, and a merge the domain
  makes is written to the mirror with a fixed author and time, so its id
  is the same on every run. Default: the real loop's clone, push and
  landing stories run on the fake git program over the fake forge's git
  route (`real-loop.md`), real git meets a forge in the real-loop and system
  worlds, and no story waits for the bridge.
- **What the real loop leaves out:** the real-git bridge (above),
  CI that runs a workflow's steps in the shell subset (how much sh,
  below), and faults in the loop. Several workers in one world are the
  cluster's services (`real-loop.md`); the cluster's faults, a worker lost or hung,
  are the process tier's (`process-tier.md`).
- **How much sh:** which commands and syntax the subset takes. It stays a
  subset of POSIX, so that real sh agrees.
- **Where the shell's interpreter lives,** so that the machine and the
  forge's CI share it.
- **Differential checks:** whether a scenario must end the same way at
  every tier it runs at, as a check that the layers below the domain
  decide nothing.
- **What the referee sees in the process tier:** the machine is real
  there and reports nothing, so an expectation about it, such as what an
  agent's environment holds, needs facts that say so, or is checked only
  in the tiers below. Default: checked in the one-process world.
- **The process tier's privileges:** the containment it shares with
  production needs user namespaces and a delegated cgroup on the test
  host. Default: process groups, but the cluster's tests, which observe
  several processes and need skein-world's cgroup trees, delegated by
  the user's service manager, and fail at their start without them
  (`process-tier.md`).
- **The machine's git in the process tier.** Default: whatever git the
  test host has, 2.41 or later, which ../protocol/git.md's start refuses below
  (../protocol/git.md, section 3); transcripts are of 2.47.

## 7. Later

- **One-process adoption and suite costs** (`checks.md`, 1): switching the caller's
  focused and fuzzy steps is a separate change, tree by tree
  (`tools/tiers/design.md`, 8). The gate and after-landing runs then
  measure process CPU against the standing budgets. Passing the suites'
  outcome checks establishes no CPU saving or headroom, and historical
  per-test samples do not establish a shared-process suite total.

- **Haiku's live landing proof** (`live-tier.md`): use the after-landing Claude Code
  owned-forge story on a fresh store to verify the catalogue apply
  (`live-model`), then charters 1 and 2 selecting that model in a second
  apply (`live-settings`) against the first's revision, and the real
  launch with
  `--model claude-haiku-5-5 --effort low`, through landing and restart.
  Keep the ChatGPT story on stock row 7, `gpt-6-luna`. The focused
  joined-world apply checks settings acceptance; a real launch and model
  outcome need this live story. Keep the live story compiled during the
  plan; its held login and real execution
  belong to after-landing. Repair any live red there.

- **The plain-change live outcome** (`live-tier.md`): run the existing owned-forge
  landing story after landing to observe the real chat's choice with
  `goal`'s declaration (`../domain/agent/tools.md`, 2). Keep the requirement for the
  chat's own successful `change`; a goal making a change does not satisfy
  this story. The referee's executor and committed-call tables establish
  its diagnostics, not a model's choice. If the live story still fails,
  diagnose the named delegate and committed calls at that red, rather
  than accepting the goal path.

- **The box's approval story on GitHub** (`checks.md`, 4): extend the configuring
  and change-wait stories to a manifest-made personal App on the fake
  GitHub. An owned-Forgejo story does not prove that composition.
  A real human review counted on GitHub (`../draft/forge-identity.md`,
  section 10) needs a user's authorized sign-in and review credential.
  Default: after-landing QA supplies that evidence. The joint world's
  Forgejo card and files-page link (`checks.md`, 3) replace the browser step by the
  plan's decision; render checks and a real owned-Forgejo walk follow
  landing too.
- **Deployment-world costs and surviving stories** (`real-loop.md`, `README.md`, 4.2, `reference-case.md`, `checks.md`, 1):
  the measurements and retained causes are in
  `temper/docs/development/limits-worlds.md`. Decide the named survivors'
  placement by their unique coverage, with lower holders first where a
  behaviour belongs below; the closest candidates are case 5.5's depth
  refusal and note recall, and `checks.md`, 5's inbox, effort and advisor checks.
  Recorded-model twins retain their independent transcript histories.
  Cost grants no exception, and moving a story to escape a budget raises
  it. Whole-suite focused CPU measurement and the gate's fuzzy accounting
  remain necessary; partial package sums and failed-prefix instruction
  profiles prove no suite pass.
  Further cause work needs bounded shared schema representations and
  prompt admission measurements without changing validation or refusal
  order. Engine-domain and one-process fuzzy profiles and their retained
  causes are in `temper/docs/development/limits-fuzzy.md`.
  Required engine steps, deadline queries, configuration admissions,
  restart/readback and token searches stay until a profile supports an
  equivalent algorithm. Real-loop matching and public setup candidates
  below roughly 2% of a story stay. Complete repair-story instruction
  totals need a successful profile; instrumentation failures establish
  none. Budgets, the real-loop ceiling and any hot-leaf build override
  remain the user's decisions.

- **Further fuzzy-suite cost work** (2.5, 6, `checks.md`, 1):
  `temper/docs/development/limits-fuzzy.md` holds the instruction pairs,
  complete-suite CPU samples and remaining causes. Keep all seeds,
  drawn inputs, crash cuts, invariants, endings and independent readbacks;
  neither fewer draws nor a cost-driven move resolves an overage.
  Further work needs:
  - A different admitted allocation contract for eager decoder lists,
    or coalesced scalar writing that preserves field order, whole-put
    refusal and overflow progress (skein's `runtime/lib.md`, sections 5 and 6).
    Required decoding and scalar reads remain; smaller measurement,
    conversion and scratch costs stay below the roughly 2% stop rule.
  - Cheaper independent native sessions and equivalent fake-person and
    service stepping, retaining every deadline, root secret check and
    cold readback. One-process prefixes have different process graphs
    and startup sources, so sharing them needs a mechanism that preserves
    each story and fault. Smaller setup and observation tails stay.
  - Cheaper complete mutable-state traversal and store/restore work,
    retaining both engine roots, exact trace comparisons and the final
    complete text. Required final text stays; the measured streaming
    alternative costs more, and old trace-prefix equality is below 2%.
  - Retained tool-schema admission with designed ownership and
    invalidation by charter, namespace and grants. Independent JSON
    validation, configuration admissions, prompt checks, allocation
    headers and byte searches remain; their inclusive footprints do
    not establish a removable real-loop gain above about 2 CPU s.
  - Separate investigation of instrumented observation-cap failures and
    intermittent raised-patience real-loop nonsettling. The suspected
    release/hold ordering is unproved; ordinary exact-name base and
    changed runs passed. Preserve the cap and ordinary patience.
  Default: fix a profiled cause with the same checks, or take the
  measurements to the user under skein/docs/foundation/testing-strategy.md, 8.1. Serial runs
  including the driver do not replace the gate's suite accounting;
  the request's roughly 15 CPU s saving and reliable headroom require
  further evidence. Budgets and enforcement settings remain separate
  user decisions.

- **The budgets raised for now** (1, `process-tier.md`): the process
  tier's 540 CPU s is the tier's 6-minute aim raised by half by the user's
  word of 11-08, for now.
  Revisiting them, and to what, is the user's, as it is for the browser
  tier's 127.5 (`../client/web/README.md`, 3).

- **The second printed link's cause** (`process-tier.md`): a serve restart draws a new
  link; a retry whose earlier sign-in spent its secret can also ask for
  another when no link is live. Neither is established as the reported
  process failure's cause. Default: keep the one-link assertion strict,
  count complete link lines and carry filtered stderr. The next red's
  diagnostics determine whether a production fault needs a lower-tier
  repair; no cause is declared fixed by the assertion change.

- **Further focused-test cost reductions** (`README.md`, 2.1, 2.2, `process-tier.md`, `checks.md`, 1):
  the row profiles and their remaining causes are in
  `temper/docs/development/limits-heavy.md`. Follow skein/docs/foundation/testing-strategy.md,
  8.1: keep each behaviour in its tier, profile its cause, and retain
  every bound, pattern, shell, ending order, request, token lifetime and
  independent oracle. The next cause's gain is priced against about
  0.2 CPU s for this repair; a larger cause needing another mechanism
  stays named rather than being declared cheap. Further work needs:
  - Shared checked startup and preparation documents, preserving retained
    edits, refusal order and independent fresh provider keys. Required
    RSA generation stays; sharing keys or weakening them changes behaviour.
  - Cheaper Markdown and semantic-tree construction and destruction at
    the byte bounds, and retained-word admission without rebuilding rows.
    Repeated-position pages must keep their pairwise tie order.
  - Gallery phrasebook ownership or view reuse, checked capacity reuse,
    shared world construction and equivalent HTML rendering or batching.
    Each state's bindings, clock, sign-in configuration and every page's
    output still need their checks; cheap rules stay in the state holder.
  - Shared JSON decoding and escaping with exact validation, sizing and
    output; ready-call routing and maintained deadlines in the connection
    driver; streaming secret checks and chunked framing observations with
    the same cut and first-byte accounting. Shortening the simulated hour
    or skipping transport fragments removes coverage.
  - Projection feeds with less copying while retaining the whole-tree
    contract, and cheaper store joins and codecs under the independent
    restart/read oracle. Sharing a restart prefix needs snapshot support
    across the harness, box, I/O, simulator and fake machine, preserving
    all 720 ending orders and each fault and quiescence check.
  - Launcher interpreter startup and executable-loader work, without
    changing build settings or combining distinct behaviours for cost.
  Default: keep the required work and all assertions until a profile
  justifies the representation or algorithm change. Instruction ratios
  for selected tests do not establish a whole-suite CPU total.

- **The running box's lifetime stories after landing** (`process-tier.md`): account
  for the hangup/rebind and killed-box stories within the process tier's
  540 CPU s suite budget (360 raised by half by the user's word of 11-08,
  for now, to be revisited), 60 CPU s per test and wall hard limit.
  The killed-box story starts playground steps and kills their box; the
  existing killed-test story kills their owning test without asking for
  playground steps. Their different ownership links and descendants
  remain separate checks. Default: retain both; any tier overage follows
  skein/docs/foundation/testing-strategy.md, 8.1, without raising a figure for the move.

- **Further fixture and startup cost reductions** (`README.md`, 2.1, 2.3, `checks.md`, 1):
  follow skein/docs/foundation/testing-strategy.md, 8 and 8.1, with unchanged behaviour,
  assertions, suites, build settings and budgets. A process-wide cache
  cannot remove the first construction in each binary's process, or in
  each test process of a slow tier. Work beyond the small fixture repairs
  needs these mechanisms:
  - Configuration: a checked generated stock representation or cheaper
    parsing and validation; sharing a bounded parsed startup document
    between its consumers while preserving diagnostic order; cheaper
    stock edits, init generation and measured turn-bound encoding.
  - Service and engine construction: shared checked admission results
    and cheaper owned policy/configuration snapshots; complete durable
    first-start rows checked against the real two-commit start; a hosted
    store constructor over supplied maps, retaining secret exchanges;
    shared checked sizing translations and generated bound validation.
    A consuming restart transfer must preserve the retained driver's
    independent oracle; reusable store transfer still needs replacement
    storage.
  - Store execution: borrowed row translation and commit encoding,
    cheaper independent history/mirror/inflight ownership, and batched
    restore/map reads preserving encoded bounds, lookahead, first-row
    refusals and corruption diagnostics. Engine decisions, page readback
    and the independent restart and final-store oracle remain exercised.
  - Words: checked ordered vocabulary construction, fewer nested owned
    allocations and clones, and shared capacity/presence/maximum scans
    that still detect public vocabulary mutation, missing words and
    overflow. Fusing validation passes must preserve whole-load error
    precedence, every form and hole, and bounded occurrence counters.
  - Drawing: indexed semantic-tree matching that retains stable keyed
    and role identities and exact patches; incremental application and
    checking that retains an independent patch oracle. Domain setup and
    rendering consumed by assertions still run, with the complete View
    in drawing stories.
  - Small setup and command costs: paired configurations, piece copies,
    idle bookkeeping, referee/default words, fakes, components and engine
    limits; Face and connection settling, routing and manifest work;
    process configuration templates and runtime-path scripts. Required
    GitHub signing needs cryptographic work or a separately justified
    seam preserving authentication. File/exec kernel cost needs its own
    measurement. Loader/process floors require separate binary/loader
    work; cheap rule assertions remain the behaviour checked.
  Default: retain these paths and their proofs until a profile justifies
  the required interface, representation or algorithm change. Stop a
  fixture repair when its next gain is below about 1% of its binary;
  larger remaining causes need the work above, never skipped checks.

- **Reference obligations for a goal that designs** (`checks.md`, 5.1): fold
  `../draft/goal-as-designer.md`'s workflow into the reference, its charters
  and stage one's judge together. Goal-delivery stories use test charters;
  they do not establish the reference's roles, models, efforts or wakes.
  Default: keep the current reference obligations until that change.

- **Tests that release a port and bind it again** (`real-loop.md`): the fake-backend
  test `peer_reservations_transfer_once_and_close_with_preparation` binds
  the released addresses to check closure and retained reuse. Another
  process can take one first. Keeping a listener through transfer would
  remove that race, but needs a different closure assertion. Default:
  this test keeps its assertion until a failure or a test change reaches
  it; the sandbox's reservation retries are `../protocol/setup/box.md`, 5's.
- **Reusable ports on macOS** (`process-tier.md`, `macos.md`): read its configured ephemeral
  range before drawing ports. With no Linux range file, the chooser uses
  49152, macOS's default; a machine configured with a lower boundary can
  put a chosen port inside its ephemeral range.
- **A shared chooser for another tree** (`process-tier.md`): jig's browser tier has a
  similar reservation hook. Moving the mechanism into skein waits for
  that second consumer; temper's chooser covers its own participating
  processes across runs, tiers and checkouts.
- **A retained box's restart beside other port choosers** (`process-tier.md`): the
  after-landing process and browser runs prove their restarts together
  on the story's ports. Within a plan, the proof stays in focused tests
  of disjoint choosers and retained sandbox ports; slow-tier changes
  compile without running.

- **A started cut with the box's sign-in enabled** (`process-tier.md`): the cut holds
  the engine before it hears the group's `started`, while the source
  step needs that engine's discovery before the group can finish. A
  proof on that configuration needs a cut or a source step that breaks
  this dependency without weakening the cut's assertions. Default:
  `process-tier.md`'s no-sign-in fixture for this cut alone; the final setup and
  removal cut use the box's sign-in.

- **A default box's combined browser and terminal proof** (`process-tier.md`;
  `../protocol/setup/box.md`, 1 and 2, and `../protocol/setup/testing.md`, 4): one 1440-pixel walk with the
  default runner and separate configuration, data and cache directories,
  signing in by one click on a fresh start, after a restart and after a
  moved temper port, then closing its pseudo-terminal and checking that
  its processes and ports are free for another start. A runner-disabled
  browser walk and separate lifetime tests do not replace this sequence.

- **The process tier's concurrency proof.** Three tests at a time (`process-tier.md`)
  are proved by the next after-landing run on main, per the request
  `process-threads` of 2026-11-06, in place of `test-speed`'s three whole
  runs and a loaded one. Only focused and fuzzy run within the plan.
  The 540 CPU s budget (360 raised by half by the user's word of 11-08,
  for now, to be revisited) and the profile's 25-minute timeout stand; a
  red after landing is repaired under `process-tier.md`'s rule, without raising either.
- **The fake box's two walks, after landing** (`process-tier.md`): the next
  after-landing run on main runs `here_key` and the browser tab and reads
  the tier's CPU without the ten walks whose behaviour went to lower
  worlds (the request `walks-lower` of 2026-11-08), against the same
  540 CPU s. A plan compiles the walks and runs none; a red after landing
  is repaired by name, under `process-tier.md`'s rule.
- **The two drivers' offline proofs, after landing** (`process-tier.md`, *Two drivers'
  offline proofs*): the next after-landing run on main runs B0's setup
  and twin walk and the managed sitting's Ready in profile
  `process-temper` and reads their CPU against the tier's budgets; B0's
  twin serves its setup and walk for at most 700 s. The owner's next
  `scripts/proof-live-github-managed.sh fake` dry run proves the test as
  the sitting's client, its driver setting up the same fake half as the
  test. A red is repaired by name, under `process-tier.md`'s rule; none of the three
  moves to `live-temper`, which needs a real model.
- **jig's referee tests, repeated.** `tests/engine/domain/tests/suite/referee.rs`
  keeps tests of jig's referees that jig's own suites hold:
  `authority_referee_rejects_a_person_without_a_grant` observes what
  `rejects_an_observer_routed` does in `jig/tests/people/tests/suite/referee.rs`.
  Default: they stay, until a debt item names them.
- **The live pair's bound.** The owned-forge landing pair runs in one
  invocation within profile `live-temper`'s 50 minutes, each story
  bounding itself at 40 (`live-tier.md`). The profile's figure is set from the
  pair's first after-landing wall times, and is the user's.
- **A signal to the landing story with its box gone.** A signal to the
  test process runs no `Drop`. Between the sittings the record is back at
  its path and nothing is owed; moved in with no box left up, the runner's
  `give-back` fails: the record stays in the store and the notice says so
  (`live-tier.md`). Closing it is a return that starts its own box, in the runner's
  part.
- **A managed sitting by a member who isn't an owner.**
  `scripts/proof-live-github-managed.sh live` takes
  `TEMPER_TEST_MANAGED_PERSON`, the login that makes the App under its own
  account (`../protocol/setup/github.md`, 2), while the repository and the
  installation stay `fabrig-ai`'s. That login is an owner of `fabrig-ai`:
  a member's request waits on an owner's approval
  (`../draft/github.md`, section 18).
- **A cached state graph.** Below the real loop a state is a pure function
  of its seed and path, so a crash sweep could fork at a prefix it now walks
  again. The slow tiers wait: a process story's restore has its meaning to
  prove, and the request `test-speed` of 2026-11-06 takes what needs no cache
  first.
- **macOS's later work** is `macos.md`, 10's.
