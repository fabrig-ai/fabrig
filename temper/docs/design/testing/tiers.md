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
`protocol/channel/testing.md`, section 1); the same reply in temper's client is
`checks.md`, 3's and the client world's (below).

Two workers have their story there (../protocol/cluster.md, 6): two
one-slot workers, each a lane of its own routed by its channel and
position, never a second slot of one worker, take two keyed chats of
one party, one each, their replies held until both run, and run each
to its end: each chat's reply and close, and the runs' identities
exactly the chats', none twice. Their figures, links and drains are
the one-process world's (1.5).

A person's latch has its story there (`qa/setups.md`, 6): a goal's run
delegates one batch of a person task, the latch, a report contract,
and two report tasks that wait on it. Before the answer, each waits on
the latch alone, pending, with no run started and no attempt; the
person's one answer at the latch's revision releases both, with no
other act: each starts at or after the latch's end and ends with its
own result, and the goal ends with its report. It opens no change.
The landing of the setup's red top is the forge connector's world's
(`tests/engine/forge`): a failing member turns the dispatched top of
two red, its job failed, its tree holding both members' files, the
history's forming decision of two and its gate ended failed, and main
holding neither its commit nor its content.

A chat's note rewritten whole has its story there
(`../domain/agent/agents.md`, 3): the chat writes its note in parts,
then its person's message wakes the parked chat, whose next run
replaces the note whole; the new attempt, both committed replies and
the whole second revision are read back. So does a chat hearing its
change land: its reply commits only after the landed result, naming
the pull and its merge commit, reached its run.

A chat made after its home's adoption, delivering its edit to its own
run branch before its reply, is one story there, ignored until a
chat's `deliver` before its end is built (`../domain/agent/tools.md`).

The ladder's twins, B0 and B1, have their stories there, fuzzy
(`../draft/v0.1.0.md`, section 3): each rung's passing walk replayed on
the first run's cast.

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
- **B0, the green light:** the chat opens one agreed goal, which asks
  for approval before any change. The question waits in the person's
  inbox, on the chat, its plan in the chat's activity, while the
  chat's tree holds the goal alone and no branch or pull exists; the
  person answers it there. The chat hears the numbered answer notice
  after the question and relays nothing; the goal hears the exact
  approval as the chat's answer before its one change call. The
  reviewer reads the pull and approves; one pull merges to main with
  CI green at its head, main holding the walk's edit whole; the goal's
  report and the chat's last reply name the pull and the merge; no
  question is left.
- **B1, the repair:** the same approval; the change's first head fails
  CI for its content and its second passes, the branch moving once on
  one pull; the change's one repair has a spec naming the failed head
  and check, and a brief holding the failed step and its log's end
  whole; the reviewer approves the second head, which merges, main
  holding the producer's edit and the repair's note; the landed
  result names the pull, the merge and its one repair, its evidence
  cut as its bound says; the goal and the chat cite them and claim no
  test passed.

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
why it is joined. The line terminal's pages are its too
(../client/line.md): a chat started from Chats by the printed controls,
its reply's first piece read before its `▍` while the rest is held,
each page's text a prefix of the next and of the committed turn, never
shorter, and the committed turn printed under `Conversation:` and
`Temper` with its clock and no marker on any later page. GitHub's three routes on the clients' listener (the
manifest page, the redirect and the setup address; ../client/forge/served.md,
1) are not tested here: they are `temper-service`'s own tests, as
requests through `fixtures::http::Peer` to the service's routes: each
answer, its headers, and a state used, unknown or past its life
refused.

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
- **Stories:**
  - a person signs in by the link `temper-box serve` prints, and a chat is
    answered;
  - a provider's list read by the worker's hosted list program (jig's
    `domain/engine/settings-catalogue.md`, 2): asked once, no run, its new name
    drawn in `me`, nothing spent;
  - a person still signed in signs in again by a fresh link, with the
    same directory, and keeps their sign-in: its number, token and
    expiry, and the deployment's sign-ins unchanged
    (../client/sign-in.md, section 1);
  - a repository is adopted through temper's client, and a goal's change
    is prepared, committed and pushed on the fake forge through the fake
    git program;
  - a merge in progress is resolved and pushed with both parents;
  - on GitHub's layer (skein's `testing/fakes/github.md`, section 1), its App
    seeded as GitHub's stories' are (below): a goal's change cloned by its
    installation's token, the token renewed `renew_before` ahead of
    GitHub's hour on the simulator's clock while the run holds its
    workspace, git's generation changing under it, and the push
    authenticated by the renewed token, as the layer's record of its git
    calls shows (../protocol/credentials-git-identities.md, section 1);
  - a signal mid-run drains the engine, the worker ends, and the next
    start settles the worker's claims at once, on a chat's run and on a
    coding run;
  - a stop while a chat's run holds a turn of smith's fake `claude` on a
    program's source: the turn interrupted, the engine drained, and no
    program left running (`../protocol/setup/box-supervision.md`, 1); a program's
    own cancel is smith's agent-process world's;
  - the service is killed, or the machine's power lost, at a drawn
    moment, then started again on the same disk, an adoption pending
    among what it finds;
  - those `../protocol/channel.md`, 9, `../client/testing.md` and
    `../protocol/sign-in.md`, 2.5 place here;
  - on the line terminal, a person signing in by the fresh link the
    service prints once `ctl` spent the first: the first open refused,
    the second signed in, and after the close, its exit 0, its
    DIRECTORY `0700` and its token `0600`, the user's, the token kept;
  - `temper ctl sign-in` leaving DIRECTORY's files `0700` and `0600`,
    the user's, on the fake machine; two end waits on one chat still
    open while `me` answers, both ending once the chat closes;
  - a hooked repository's one signed push answered by one `204` from
    the engine's webhooks' listener, through the service's dispatch
    (../protocol/forge/README.md, section 4); `temper ctl repository
    adopt` adopting the forge's repository as owned, printed in the
    forge's words;
  - a private file's key, its one final newline stripped, carried by
    the chat's model request, as the scripted peer's credential
    observation shows;
  - a call whose answer is lost after its commit, its caller ended
    before it prints, asked again under its name from a second
    DIRECTORY signed in by a fresh link as the same party: answered
    the first answer, the party's one chat, its reply read;
  - a welcome's model check on a program's source: one stock `claude`
    print on its chosen model, the trial ended before the stop, its
    seconds on the simulator's clock, the login's email on Ready, and
    no direct peer's traffic (`../domain/trial.md`);
  - a service started again on its disk keeping a replied chat, its
    reply read cold with no model asked, and a `ctl` setting kept
    across the provider's rebinding at that start;
  - a model's request held by its provider after a shell call (smith's
    `protocol/llm.md`, 6): held once, cut at its endpoint's `idle_ms`,
    asked again with the same input and answered; held on every
    attempt, the run held as its run's failure, one run try and no
    agent try, well before the worker's no-progress bound, and no agent
    stopped; on the simulator's clock, the cut and retries the protocol
    world's (smith's `protocol/llm.md`, 10);
  - a chat on a program's source whose reviewed change lands: four
    `claude` prints, the fourth resuming the first's session, the
    coder's read and edit of the README, no direct peer's traffic
    (smith's `protocol/programs-claude-code.md`);
  - two workers alone, each on its own directories, taking their
    figures once, linking once at their positions and draining with
    the engine at a stop (../protocol/cluster.md, 6);
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
    1. a direct landing: the App connected by the paste and installed
       on `acme`, `acme/app` adopted, and a chat's change whose coder
       edits it and whose reviewer approves, checked by its head's check
       run and merged there; every git request by a token minted for
       `acme`'s installation, one of them a write; the protocol world
       holds the one pull opened and merged by the App's bot at the
       checked head, the engine's domain world the landed result read
       twice the same, and the system world the chat's reply after it
       (1.1, 1.2);
    2. GitHub connected through the manifest flow from `temper ctl`,
       story 6's start: one visit's statuses as GitHub answers them,
       the close-tab page, the forge connected as the App's bot, and no
       App key downloaded;
    3. a repository added to a project later, from the picker;
    4. a landing through GitHub's merge queue, with an ejection
       repaired;
    5. an approval rule waiting on a person, then the App on the
       bypass list;
    6. git on GitHub's tokens: two accounts, `acme` installed on all
       its repositories and accepted, `umbra` holding an installation
       of its own the deployment never accepted; the picker lists only
       `acme`'s repositories; `acme/app` adopted, and a chat's change
       whose coder edits it, its clone and its one push each by a token
       minted for `acme`'s installation, the branch's head the coder's
       one new commit; `umbra/tool`'s adoption refused (`NoAccess` on
       the client's wire until it names `NotInstalled`:
       ../domain/forge/adoption.md, 3), with no git call and no token
       for `umbra`;
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
  Each story, each run above among them, is a test of its own;
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

- **A started cut with the box's sign-in enabled** (`process-tier.md`): the cut holds
  the engine before it hears the group's `started`, while the source
  step needs that engine's discovery before the group can finish. A
  proof on that configuration needs a cut or a source step that breaks
  this dependency without weakening the cut's assertions. Default:
  `process-tier.md`'s no-sign-in fixture for this cut alone; the final setup and
  removal cut use the box's sign-in.

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
  offline proofs*): the next after-landing run on main runs B0's setup
  and twin walk and the managed sitting's Ready in profile
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
