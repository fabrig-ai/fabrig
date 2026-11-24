# Testing tiers

This holds temper's testing tiers, split from the [testing design](README.md).

## 1. temper's tiers

| Tier (skein/docs/foundation/testing-strategy.md, section 2) | In temper |
|---|---|
| step tests | every domain crate and every fake; every codec (records drift, built-value round trips, bounds, fuzz targets) and each translation both ways; git's argument and environment builders, and git's output parsers against transcripts of real git (../protocol/git.md, section 9); Forgejo's decoders, skein's, against the conformance's transcripts (skein's `forge/README.md`, section 9), and GitHub's, skein's (skein's `forge/github.md`); the forges' records and a git identity's value, built-value round trips and fuzz targets (../protocol/forge/engine.md, 1; ../protocol/credentials-git-identities.md, section 1); `temper-service`'s routing of the forges: start, connect, stop, installations, generations and pasted values, and GitHub's three routes on the clients' listener (../protocol/forge/README.md, section 9; 1.1); the setup's machines, files and specs, and the box's machine (../protocol/setup/testing.md, 1); lib's are skein's |
| domain worlds | each child domain with the world as its parent, and each component's root domain with its child domains; most of temper's tests |
| system worlds | the engine, the worker and smith's agents together; temper's client joined to the engine's root (1.1) |
| machine worlds | the git component against scripted process ends and outputs; the terminal shell's loop against a scripted stdin and stdout; skein's, for the machines it ships, the forge component among them (skein's `forge/connection.md`, section 10) |
| protocol worlds | the forge's: the forge component against the fake forge's protocol layer, deliveries included; reconcile's is skein's (1.2) |
| io worlds | skein's: temper does not retest io |
| simulated worlds | the one-process world: `temper-service` on skein's simulator (1.3, 1.5) |
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
../protocol/setup/box-backends.md, 1): CLI and filesystem preparation in temper's unit
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

### 1.1 System worlds

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

- **What it holds:** the stories whose behaviour crosses the engine,
  the worker and an agent, each placed here by the design that owns it:
  the composed workload, several goals of two parties on one fake forge,
  and its sweep of every family of faults (`workloads.md`, 5 and 8.2);
  a change's merge job, from the job made to the engine going on from
  its branch, its command the world's (`../domain/forge/change/repairs.md`;
  `../domain/worker/jobs.md`), its rows of the worker alone the worker's
  world's (`tests/worker/domain`); a run's live text reaching the party's
  watch before its turn commits, smith's producer and the bytes between
  the roots jig's (jig's `protocol/channel/testing.md`, section 1); two
  one-slot workers each running one chat to its end
  (../protocol/cluster.md, 6), their figures, links and drains the
  one-process world's (1.5); a person's latch releasing the batch
  waiting on it (`qa/setups.md`, 6), its red top the forge connector's
  world's (`tests/engine/forge`); a chat's note rewritten whole after
  its person's message (`../domain/agent/agents.md`, 3), and a chat's
  reply committed only once its change's landed result reached its run;
  a chat made after its home's adoption, delivering to its own run
  branch, ignored until a chat's `deliver` before its end is built
  (`../domain/agent/tools.md`); and a reply of text alone, Open and
  Finite, holding temper's choice of mode for each
  (`temper-engine-smith`'s start) to the parking and the nudge that are
  smith's and jig's own worlds' (`../domain/agent/charter.md`, 5).
- **Which test checks what** is the code's; a story moved here from the
  real loop is beside its holders in
  `../../development/slow-tests-moves-2026-11-24-temper-real-loop.md`.

The ladder's twins, B0 and B1, have their stories there, fuzzy
(`../draft/v0.1.0.md`, section 3): each rung's passing walk replayed on
the first run's cast, B0's green light and B1's repair.

- **Its scripts are the walk's,** a fake LLM document made from the
  passing run's transcripts by `bin/temper-twin` (`fakes.md`, 1), never
  invented by hand: each agent says what the real model said, turn by
  turn. A structured adaptation preserves every other turn, is
  validated by the twin, and has its provenance recorded in the
  fixtures' README. The world hands the composed provider that
  document whole with its configuration, its cues, rules and takes
  choosing each turn and its peer's state kept across activations,
  never translated into hand-written calls.
- **Its forge is the fake's,** seeded as the walk's box seeded
  `playground/widget`, with any commit the walk's person made on it,
  and a CI cue that judges as the walk's CI did: for B1, a head passes
  only when it holds the second change note, which the walk's repair
  wrote. The person's acts are the walk's,
  in the same words, as the world's reads and answers at the same
  points.
- **Its takes,** for QA's twin setups (`qa/setups.md`, 4): each
  fixture names the tasks, pull requests and heads of its walk's box
  as takes (skein's `testing/fakes/llm.md`, 2.6), from a `takes.json`
  beside it, applied by `temper-twin --takes` (`fakes.md`, 1), and its
  `regenerate.sh` passes it. Each take finds the value the walk
  recorded, so the story reaches the same end, and that proves each
  take's pattern is in the conversation smith writes.
- **Its data is the playground's story** of the rung
  (`../playground.md`, 5.3), one source, which QA's setups and the
  box's opening read too; until that crate holds it, the fixtures the
  walk's recording made.
- **What a twin cannot hold** is what only a real model shows: the
  words it reads. A change to them is rechecked by the rung's live
  story (`live-tier.md`), and its twin made again from that run's
  transcripts, `temper-twin --replace` covering assistant text as well
  as call arguments.

temper's client joined to the engine's root, `tests/client`
(`temper-client-world`), is a system world too: temper's client root and
view, skein's fake person reading the view's tree, jig's client worlds'
faults and jig's check, and temper's engine root on the engine world's
store, hosts and forge (../client/testing.md). `checks.md`, section 3 says
why it is joined. The line terminal's pages are its too
(../client/line.md). GitHub's three routes on the clients' listener (the
manifest page, the redirect and the setup address; ../client/forge/served.md,
1) are not tested here: they are `temper-service`'s own tests, through
`fixtures::http::Peer` to the service's routes, against no fake.

### 1.2 Protocol worlds

Each component's protocol layer meets the protocol layers of its peers'
fakes, joined by in-memory streams cut and joined at random, and the
machine sits there at its io face (`README.md`, 3.2). They test what translation adds
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
  expires on the fake's clock, at GitHub's hour (../protocol/credentials-git-identities.md,
  section 1).
- **Reconcile's,** in skein (skein's `forge/setup/README.md`, section 5):
  reconcile's machine against the fake forge's protocol layer serving the
  setup's routes, through bytes: a forge converged from empty and again,
  each thing deleted made again, the refusals, no secret in a message or a
  log (../protocol/setup/testing.md, 2).
- **The worker's link and the client's connection** have their protocol
  worlds in jig (jig's `protocol/channel/testing.md`, section 1; jig's
  `protocol/client/testing.md`, section 1), and an agent's channel and its LLM
  calls theirs in smith and skein. temper's parts and records cross them
  as bytes those worlds never read; the one-process world carries
  temper's through bytes (1.5).
- **git has no protocol world:** it is a program, not a peer on a stream.
  Its tiers are the git component's machine world, the fake git program
  and real git in the process tier (../protocol/git.md, section 9).

### 1.3 Simulated worlds

temper's simulated world is the one-process world (1.5). Every process
in a scenario runs its `iterate` on skein's simulator: `temper-box serve`'s
service, the agents it spawns, the fake forge as a service, the fake LLM
peer and the person's client. A spawn the worker asks for starts an agent
process or the fake git program, and a kill ends it. The machine sits
behind the simulator (`README.md`, 3.2).

### 1.5 The one-process world

`tests/one-process`, after jig's (jig's `protocol/service-testing.md`, section 1, and `protocol/service.md`, section 10
). Its composition is `temper-playground`'s, the world the
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
  and no terminal (`../client/journey-walks.md`, 3.2 and 3.3, and `../client/journey-walks-world.md`, 1). The
  walker feeds their lines and prints their marks as pages.
- **What it holds:** what needs the service's components together in
  one process, with its disk, its clock and its kills, and no real
  kernel, which the real loop keeps (`real-loop.md`): a sign-in by the
  link `temper-box serve` prints, and again by a fresh link keeping its
  sign-in (../client/sign-in.md, section 1); a provider's list read by
  the worker's hosted list program (jig's
  `domain/engine/settings-catalogue.md`, 2); a repository adopted and a
  goal's change pushed through the fake git program, a merge resolved
  with both parents, and a GitHub installation's token renewed while a
  run holds its workspace (../protocol/credentials-git-identities.md,
  section 1); a signal mid-run, a stop while a program's turn runs
  (`../protocol/setup/box-supervision.md`, 1), and a kill or power loss
  at a drawn moment, each followed by the next start on the same disk;
  `ctl`'s files on the fake machine, a lost answer asked again from a
  second DIRECTORY, and the line terminal signed in by a fresh link;
  the engine's webhooks' listener (../protocol/forge/README.md,
  section 4); a private file's key reaching a chat's model; a model's
  request held by its provider after a shell call (smith's
  `protocol/llm.md`, 6 and 10); a welcome's model check and a reviewed
  change on a program's source (`../domain/trial.md`; smith's
  `protocol/programs-claude-code.md`); a service started again on its
  disk keeping its chats and settings; workers alone taking their
  figures, linking and draining (../protocol/cluster.md, 6); a drained
  service leaving nothing for the next one in its process, each
  numbering its first tasks from one, the first's agent roots read
  released before the next fresh disk resets them, through
  `temper-playground`'s `machine::roots_are_released`, a seam behind its
  `test-inspection` feature that production never builds; and those `../protocol/channel.md`,
  9, `../client/testing.md` and `../protocol/sign-in.md`, 2.5 place here.
- **Which test checks what** is the code's; a story moved here from the
  real loop is beside its holders in
  `../../development/slow-tests-moves-2026-11-24-temper-real-loop.md`.
- **Stories:**
  - **GitHub's stories,** on the fake's GitHub layer (skein's
    `testing/fakes/github.md`, section 1), which the forge peer hosts
    beside its Forgejo face over the same forge as the fake git
    program. The configuration points `application.test.github`'s
    `api`, `transport`, `remote` and `web` at the peer
    (../protocol/configuration/engine.md). A story that starts
    connected has its App seeded before the service starts by
    `testing/temper-github-app`: the App on the layer's first fixed
    test key, its six records (../protocol/credentials.md, 2.1) in
    jig's secrets' store; the process tier seeds the same
    (`process-tier.md`). A browser's steps are plain HTTP requests,
    following each `302` and the one link (skein's
    `testing/fakes/github.md`, 1.1), and every git call goes through the
    layer's git route, which checks its installation's token. One story
    per feature:
    1. a direct landing on the App connected by the paste, checked by
       its head's check run and merged there, every git request by its
       installation's token; its halves in the protocol world, the
       engine's domain world and the system world (1.1, 1.2);
    2. GitHub connected through the manifest flow from `temper ctl`,
       with no App key downloaded;
    3. a repository added to a project later, from the picker;
    4. a landing through GitHub's merge queue, with an ejection
       repaired;
    5. an approval rule waiting on a person, then the App on the
       bypass list;
    6. git on GitHub's tokens: only the accepted installation's
       repositories listed, cloned and pushed by its token, and an
       installation the deployment never accepted refused at adoption
       (`NoAccess` on the client's wire until it names `NotInstalled`:
       ../domain/forge/adoption.md, 7), with no git call or token for it;
    7. a repository granted on GitHub appearing in the picker;
    8. Forgejo connected, managed and owned from the client.

    After each, no secret outside jig's secrets' store
    (../protocol/credentials.md, section 7), by the observer of saved
    artifacts (below).
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
  - the same start rehearsed first (jig's `protocol/store/opening-rehearsal.md`, 1),
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
  world's (jig's `protocol/store/testing.md`, section 1). An effect in
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
  - the engine's referee over the fake forge (`scenarios.md`, 1.2): nothing lands on a
    default branch without green CI, and a run's change is the tree its
    agent left;
  - no git identity's value or API token in a domain's state, a store
    record or an agent process's environment;
  - the observer of saved artifacts (`scenarios.md`, 1.2): each value
    the fakes issue (the App key, its DER, its body and each of its
    lines of 32 bytes or more; each installation token and its
    `x-access-token:` credential), in its bytes, its Debug forms and
    base64, is absent from every window the observer keeps. A window
    is an artifact and its name; an append, with as many bytes before
    it as the longest value it is checked for, less one; a stream's new
    bytes, with as many before them as the longest value, less one; a
    stream's earlier bytes, for the values issued since; and, at the
    close, every public file, its name and its turns' bodies. Each
    window is checked at a checkpoint for the values issued since it
    was last checked, a token issued after a checkpoint against every
    byte saved before it. A finding names the kind of the
    earliest-issued value found and the surface, never the value.
    Each window is matched against all its values in one pass, by one
    automaton over each value's first bytes, and the whole value is
    compared where they are found: an automaton over whole values is
    built again at each issued token and walked through every value's
    bytes, and a matcher per group of values or a finder per value
    cost more (`temper/docs/development/limits-worlds.md`, *Causes
    found and left*).
- **Budget:** a story aims at 2 s (`checks.md`, 1); sweeps of drawn kills are fuzzy
  tests. The upgrade module's tests are placed as `../protocol/floor.md`, 7
  lists.
  Each story is a test of its own;
  Anthropic's keys through `ctl` are `tests/client`'s
  (`../client/ctl/testing.md`, 2), as is the template's paging story; a
  worker's opening with chosen wires is `temper-worker-service`'s
  (`../protocol/wires.md`, 11).

### 1.8 macOS

What macOS tests is `macos.md`'s: one workflow, outside the gate, on
GitHub's `macos-26`, dispatched after landing at a commit of main for a
set of parts, each a job: the process tier's stories, the one story among
them, its Forgejo and runner fetched first through `temper-box forge
fetch`; the browser tiers; each tree's focused and fuzzy suites; and the
live stories on the user's key. One JUnit report names the failing tests
by tier main's ids, a build that fails among them, and fabrig's forge
hands it its macOS tiers' runs. Linux's own tests are listed by name.

## 2. Open questions

- **The process tier's privileges:** the containment it shares with
  production needs user namespaces and a delegated cgroup on the test
  host. Default: process groups, but the cluster's tests, which observe
  several processes and need skein-world's cgroup trees, delegated by
  the user's service manager, and fail at their start without them
  (`process-tier.md`).
- **The machine's git in the process tier.** Default: whatever git the
  test host has, 2.41 or later, which ../protocol/git.md's start refuses below
  (../protocol/git.md, section 3); transcripts are of 2.47.

## 3. Later

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
  `goal`'s declaration (`../domain/agent/tools-goal.md`, 1). Keep the requirement for the
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
- **The budgets raised for now** (`README.md`, 1, `process-tier.md`): the process
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

- **The running box's lifetime stories after landing** (`process-tier.md`): account
  for the hangup/rebind and killed-box stories within the process tier's
  540 CPU s suite budget (360 raised by half by the user's word of 11-08,
  for now, to be revisited), 60 CPU s per test and wall hard limit.
  The killed-box story starts playground steps and kills their box; the
  existing killed-test story kills their owning test without asking for
  playground steps. Their different ownership links and descendants
  remain separate checks. Default: retain both; any tier overage follows
  skein/docs/foundation/testing-strategy.md, 8.1, without raising a figure for the move.

- **Tests that release a port and bind it again** (`real-loop.md`): the fake-backend
  test `peer_reservations_transfer_once_and_close_with_preparation` binds
  the released addresses to check closure and retained reuse. Another
  process can take one first. Keeping a listener through transfer would
  remove that race, but needs a different closure assertion. Default:
  this test keeps its assertion until a failure or a test change reaches
  it; the sandbox's reservation retries are `../protocol/setup/box-backends.md`, 1's.
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

- **A default box's combined browser and terminal proof** (`process-tier.md`;
  `../protocol/setup/box.md`, 1, and `../protocol/setup/box-supervision.md`, 1, and `../protocol/setup/testing.md`, 4): one 1440-pixel walk with the
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
  offline proofs*): the next after-landing run on main runs B0's twin
  walk and the managed sitting's Ready in profile
  `process-temper` and reads their CPU against the tier's budgets; B0's
  twin serves its setup and walk for at most 700 s. The owner's next
  `scripts/proof-live-github-managed.sh fake` dry run proves the test as
  the sitting's client, its driver setting up the same fake half as the
  test. A red is repaired by name, under `process-tier.md`'s rule; none of the three
  moves to `live-temper`, which needs a real model.
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
  (`../draft/archive/github.md`, section 18).
- **macOS's later work** is `macos.md`, 10's.
