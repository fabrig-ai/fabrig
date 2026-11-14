# The engine's and the worker's services

Provisional, 2026-10-10, revised 2026-10-19. What an application writes
around its roots to make running processes: the protocol layer that
composes jig's components with its own, `iterate`, which carries every
event between io, the components and the root, and `main`, which reads
configuration, opens the store, starts the loop and ends it; and its
server, which owns the clients' listener and its URL space, and mounts
jig's clients' component there. It is to the protocol layer what
`../domain/root.md` is to the domain: one shape every application copies,
from the reference binary and its service (section 7), around the parts
jig ships for it (section 11). The mechanics are
skein's programming-model.md, sections 2 and 4. What is still open is
listed in section 9.

## 1. In one page

- **Every process is the application's** (`../README.md`, section 3): its
  root, its protocol layer, its `iterate` and its `main`. jig supplies
  components of the protocol layer, as it supplies children of the root.
- **The protocol layer is a parent of components.** Each of jig's (the
  store's, the channel's, the accounts', the clients'), each of smith's
  and skein's a process uses (smith's LLM component and skein's
  connections for runs in the process, smith's host half for agent
  processes), and each of the application's (its connectors' protocols,
  its credentials' sources) is a child with its own entrance limits and
  deadlines. Internal hand-offs grow as they fill and are uncapped;
  the kernel's hand-offs keep their edge sizes. The protocol layer routes
  between them, io and the root, and decides nothing.
- **The clients' listener is the application's,** with its routes and
  its URL space: its server serves its pages and its bundle, its own
  API and its sign-ins, and mounts jig's clients' component
  (`jig-protocol-engine-clients`) wherever it chooses, carrying jig's
  messages to it and back. The component knows no HTTP; jig's default
  HTTP mapping, `jig-protocol-engine-clients-http`, is one way to carry
  them, which the application uses, wraps or does without (section 2;
  `client/default-http.md`; `browser/README.md`, section 12).
- **The journal is the root's only way out,** and `iterate` its only
  reader: commits go to the store, held outputs leave once the store has
  answered, outputs addressed to a child come back as events
  (`../domain/root.md`, section 3).
- **Start in order:** configuration, then the store opened, then the
  components, then the listeners, then the core's restart script, then
  decisions (section 3).
- **End in order:** a failed commit stops at once; a shutdown drains until
  nothing waits for a commit, then closes everything, as skein's owner
  close asks (programming-model.md, 5.2).
- **The reference is jig's test application's:** its binary, `jig-test`,
  and its service, `jig-test-service`, step code that composes jig's
  service parts (section 11) and whose routing, `iterate` and startup are
  written to be copied, like the reference root, which mount the clients'
  component at `/jig` with the default HTTP mapping and serve jig's test
  application's bundle themselves (section 7).
- **What every application writes alike, jig ships:** the accounts' host,
  the store's host, the program sources, the worker route, the workers'
  side, the server and the clients' side, as product code (section 11).
- **In one process,** a client the process holds reaches the clients'
  component by an in-process transport, with no listener (section 10).

## 2. The engine's protocol layer

```
<application>-service                 iterate: io ↔ components ↔ root
├── <application>-domain              the root, with the journal (../domain/root.md)
└── the protocol layer
    ├── jig-protocol-store            commits, loads, transcripts, secrets (store.md)
    ├── jig-protocol-engine-workers   workers, if any (channel.md)
    ├── jig-protocol-accounts         LLM accounts' sign-ins, records and table of values, if it runs agents (llm.md)
    │   └── skein-oauth-accounts
    ├── smith-protocol-llm            smith-inline-agent's completions, for the engine's own slots (shape 2) (llm.md)
    │   └── skein-llm-connection
    ├── <server>                      the clients' listener, its routes, its pages and bundle: the application's, on jig's server part (section 11; browser/README.md, section 12)
    ├── jig-protocol-engine-clients-http   jig's default HTTP mapping, its engine end, mounted under the server's prefix (client/default-http.md; browser/README.md, 12.2)
    ├── jig-protocol-engine-clients   the clients' component: jig's messages, credentials, the stream's pacing; no HTTP (client.md)
    ├── <connector>…                  each connector's protocol, and its webhooks' listener: the application's
    ├── <credentials>                 the application's credentials for runs' workspaces: its table, its sources
    └── timers                        the root's and every component's deadlines
```

- **Routing from io:** each io event goes to the component whose token it
  carries (programming-model.md, 4.2).
- **Routing from components to the root:** what a component hears from its
  peer, translated into the domain's vocabulary (the core's or the hub's,
  wrapped, or a connector's) and with the application's own parts decoded
  by the application's codecs, becomes one root event. A component never
  steps the root itself.
- **Routing from the root to components:** what the journal releases, and
  what leaves through its door, goes to the component it names: a commit
  or a load to the store; an assignment, a relay's answer, an
  acknowledgement, a credential grant or a cancel to the channel's
  component; a party's answer, a read's answer or a watch's delivery to
  the clients' component; a completion `smith-inline-agent` asks for, or
  its cancel, to smith's LLM component, with its slot's result contract
  (`llm.md`, section 3); a source signed in or out, wanted or released,
  a key added or removed, a sign-in cancelled, to the accounts'
  (`llm.md`, 5.2); a program source's set, check or sign-out to the
  channel's (`channel/records.md`, 2); an outbox entry to make, to its
  connector's protocol.
- **Calls decoded, answers rendered, in the routing:** a host tool's call
  the core hands out is decoded, and an answer rendered, by functions the
  routing calls, `jig-protocol-tools`' for the core's tools and the
  application's for its connectors', in the step that routes them
  (`channel/README.md`, 5.3); they keep no state.
- **smith's payloads, decoded and encoded in the routing:** the root
  hands out each of smith's payloads between its own hub and
  `smith-inline-agent`, and the routing decodes or encodes it with
  smith's codecs, the charter against the engine's
  endpoints, keeping no state, and hands it back (`../domain/hosts/hub.md`,
  8). As it decodes a charter, it keeps the slot's result contract
  beside smith's LLM component (`llm.md`, section 3).
- **Routing between components:** secrets' values go from the accounts'
  table, and from the application's table of its credentials, to the
  components that fill them in (`llm.md`, section 6; `channel/README.md`,
  section 7), never through the root. A record skein's driver asks to
  keep goes to the secrets' store, and `kept` back once it is durable; a
  key a request carried goes from the clients' component to the secrets'
  store before the request goes to the root, and from the secrets' store
  to the accounts' table once the accounts child names it a source's
  (`store/secrets.md`; `client/README.md`, section 3).
- **The clients' traffic, through the application's server.** The
  server is the application's: skein's HTTP server over the clients'
  listener, or any other, and its routes. It answers its pages, its
  bundle and its own routes itself (`browser/README.md`, section 12), and
  routes every request under the prefix it mounts jig's mapping at to
  `jig-protocol-engine-clients-http`, which turns each into one of the
  clients' component's messages (an exchange with its credential as
  opaque bytes, a stream opened, taken or gone) and the component's
  answers, streamed messages and ends back into HTTP answers and
  server-sent events (`client/wire.md`, 2.2 and `client/default-http.md`). The prefix is its
  choice; jig reserves no path.
  - *Its own transport,* in place of the mapping (a local socket for a
    command line, an in-process one: section 10), carries the same
    messages to the component and back.
  - *Its own sign-in:* its routes verify an identity by its own means,
    and its server's code asks the component to mint a credential for
    it, with the component's mint message, never a client's; the
    answer, a credential or a refusal, comes back to that code, which
    hands it to its client as its transport carries credentials
    (`client/sign-in.md`; `browser/README.md`, section 11).
  - *The component's side toward the root* is as every component's, and
    knows nothing of the transport: a party's request, read or watch to
    the root as a root event, and the root's answers back to it (above).
- **Timers.** The protocol layer reports the earliest deadline of the root
  and every component, and fires each where it fires its own
  (programming-model.md, section 9); it reports the same apart from io's
  own, as the policy deadline (section 4).
- **Admission.** Internal hand-offs are uncapped. The root takes
  completions of admitted work at any backlog, and hands new external
  work back while its backlog is at the watermark. The component that
  sent it keeps it until admission resumes (`../domain/root.md`,
  section 3). Completions may pass a held call, in arrival order, until
  the next new event.

## 3. Starting

1. **Configuration,** read and checked whole before the loop
   (`configuration.md`).
2. **The store opened,** both of its stores, and its own records read
   (`README.md`, section 9). A store that refuses (another format,
   records that disagree with its own header, a recovery that fails) ends
   the start.
3. **The components made,** with their limits and their parts of the
   configuration; skein's accounts driver is made with each signed-in
   source's record from the secrets' store, the accounts' table is
   filled with each key there (`llm.md`, 5.1 and 5.3; `store/opening.md`
   ), and the application's credentials' sources are read or
   started.
4. **The listeners opened:** the workers', the clients' (the
   application's server's), and each connector's own for its webhooks.
   Workers may dial in and say hello during the restart, which needs
   them (`../domain/engine/restart.md`); the clients' component
   answers clients not ready until decisions open
   (`../client/domain/contract/README.md`, section 3). Where configuration
   enables the printed link, or a browser's approval, the startup first
   checks that the clients' listener, a member of the application's own,
   is on a loopback address, and refuses the start, naming both, if it
   is not (`configuration.md`, section 3, clients). With the printed
   link enabled, only after that check does the component give the link's
   secret, and `main` print its URL, on the application's origin and
   route (`client/sign-in.md`; `browser/README.md`, section 11). Approval
   alone prints nothing; its secret is drawn when a client asks.
5. **The root's restart:** `iterate` gives the root its start, and the
   core's script asks for one step at a time; the protocol layer does the
   store's and the connectors' parts of each step as the root asks; the
   core wants every signed-in source from its first step
   (`../domain/root.md`, section 8; `../domain/engine/README.md`, section 6).
6. **Decisions open** once the core says so; from then on the root takes
   every event.

## 4. iterate

The shape of skein's `iterate` (programming-model.md, section 2), with
these passes:

```
up:    io completions → components → root events → root.step (one decision each)
       journal: store answers → committed/failed → released outputs
down:  released and door outputs → components → io submissions
       outputs addressed to a child → root events, next pass
```

- **One decision per root event,** and at most one commit; a decision's
  commit goes to the store in the down pass of the iteration that made
  it.
- **Released outputs leave in order,** a bounded number per call
  (`../domain/root.md`, section 4).
- **A store's answer is always taken,** however full the journal, so the
  bound on commits in flight never stops the answer that frees it
  (`../domain/engine/README.md`, 5.1).
- **Nothing waits in `iterate`** between iterations but what its
  components and the root hold as state.
- **The policy deadline.** skein's `Host` asks each service for its
  earliest deadline other than io's own, the one the teardown invariant
  watches (skein's `runtime/shell.md`, section 11, `next_policy_deadline`): a
  service's `iterate` reports there its root's and its components'
  deadlines, as it reports them for its loop; in one process (section
  10), over every root and protocol layer it owns.

## 5. Ending

- **A failed commit stops the engine.** The journal releases nothing
  tagged with it or after it (`../domain/root.md`, section 4); `iterate`
  aborts every component, which closes without draining; `main` ends the
  process with a failure status, and a supervisor starts it again, which
  recovers from the last durable commit.
- **A shutdown drains,** from a signal or an operator:
  1. **The listeners stop admitting:** workers' openings refused as
     stopping, clients' requests answered not ready.
  2. **The core hears that the engine is stopping,** settles the claims
     of its agent runs in this process as interrupted, spending no try,
     in one decision, and from then on takes nothing that would make a
     commit, and ends every watch saying so (`../domain/engine/restart.md`, 1).
  3. **What is in flight finishes:** the commits made before are answered,
     and their held outputs released, until nothing waits for a commit,
     which the core says.
  4. **Each component closes,** its peers told, and reports closed; the
     store closes last: `other_store_drivers_closed` holds its root open
     until the other drivers' handles are closed, their io ready and
     their terminal queues empty. The reference service holds this with
     delayed driver completions in `jig-test-service`,
     `tests::engine::the_store_root_closes_after_delayed_driver_completions`.
     A stop before the start's mark (the switch
     step's wires commit, `../domain/engine/restart.md`) finds the
     components that wait for it never made, the accounts' driver and
     the secrets' gate among them: each closes at once as closed, with
     nothing outward, and none waits for a mark the stopped start will
     never commit.
  5. **`main` ends the process** once everything it owns has reported
     closed (programming-model.md, 5.2).

  A second signal turns the drain into an abort: what is in flight is
  dropped, which a restart recovers from, as from a crash.
- **Runs in the engine** die with it: a drain has settled its agent
  claims as interrupted (step 2), leaving job claims for the next start
  to settle, since their effects may have happened. After a crash the
  next start settles both (`../domain/engine/restart.md`). Runs on
  workers in other processes go on for the engine's grace and are
  adopted when their workers dial in again.

## 6. A worker's service

```
<application>-worker-service      iterate
├── <application>-worker          the worker's root: jig-domain-host, smith's host domain, its workspace
└── the protocol layer
    ├── jig-protocol-worker-link  the link to the engine; smith's payloads carried as bytes (channel.md)
    ├── smith-host-protocol       one channel per agent process, over its pipes; grants filled into its start
    ├── agent processes           each in a process group of its own, or a contained tree where skein gives one (README.md, section 10)
    ├── <credentials>             the application's table, filled from grants: the application's
    └── <workspace>               the workspace's io: the application's
```

- **No store.** A worker keeps nothing durable (`../domain/hosts/README.md`,
  section 4).
- **Agents as child processes.** The worker's root composes smith's host
  domain, which asks for an agent process per run; the protocol layer
  spawns the agent program its configuration names (`configuration.md`,
  section 4), with an environment holding no credential, in a process
  group of its own (skein's `runtime/io/processes.md`), and gives smith's host
  half its pipes (smith's `protocol/host.md`, section 3). A run on a
  program source names its configuration directory under the programs
  directory (`configuration.md`, section 4), where the program keeps
  its own login; that login is on the host, beyond the environment, and
  is why a host keeping a person's logins takes only runs that person's
  accounts pay for, until a run's process tree is contained from them
  (`llm.md`, 5.5). The program may be the application's own binary run
  as its `agent` command, smith's agent shell (`smith-agent-shell`).
- **smith's payloads, decoded and encoded in the routing,** as in the
  engine (section 2), the charter against the endpoints of the worker's
  agent configuration (`configuration.md`, section 4), and smith's facts
  decoded into jig's activity (`channel/README.md`, 5.1 and 5.4).
- **Credentials:** `jig-protocol-worker-link` takes grants' values out
  into the worker's tables, and the protocol layer hands LLM accounts'
  values, each with its scheme, or a program source's name, to smith's
  host half, which fills them into each agent's start and grants
  (`channel/README.md`, section 7; smith's `protocol/host.md`, section 2); an
  application's credentials go to its workspace's io.
- **Program logins:** for the program sources the engine names, the
  worker's protocol layer has smith's host half check each login, runs
  each sign-out in the source's environment, and reports both over the
  link (`channel/records.md`, 1 and 2; smith's `protocol/host.md`, section
  2; smith's `protocol/programs.md`, section 4).
- **Or agents in the worker's process,** as the worker's root's choice,
  for one person's machine: a root that composes `smith-inline-agent`
  has smith's LLM and machine components in its protocol layer instead
  (smith's `protocol/README.md`, section 4; `llm.md`, section 2). A run
  is then no process of its own: no containment, and no watchdog on
  progress (smith's `domain/host.md`, 9.3).
- **Starting:** configuration; the components; the link dialled at once;
  the hub reports what it hosts once the link is up (`channel/README.md`,
  9.1).
- **Ending:** the root drains the hub for its drain bound, then cancels
  what is left, and the hub admits no more; the worker is done once the
  engine has every turn and answer, or past its grace with no engine
  (`../domain/hosts/hub.md`, 7), and at the latest at its stop bound,
  which the root keeps as a deadline (`../domain/hosts/hub.md`, 6). jig's
  reference worker (section 7) does not drain: it cancels at once, and
  keeps no deadline beyond its components' own.

## 7. The reference binary and service

- **The reference service, `jig-test-service`,** among jig's test kits
  but written as step code: jig's test application's engine protocol
  layer, composed from jig's service parts (section 11), with the host
  tools' translation and its test connector's system answered in memory.
  Its routing, its `iterate` and its startup are what an application
  copies; the parts it composes, an application composes too. Its
  engine runs no agents of its own and a test worker's (the link,
  smith's host half and its agent processes, the test workspace), each
  with its `iterate`, composed in one process (section 10), with a
  startup function that takes configuration's bytes and returns the
  service. jig's protocol world runs it (section 8).
- **Its kernel hand-offs** are sized at the edge: the store allows 12
  entries, four for each of its three single-flight file drivers; the
  service's staging queues allow `skein_io::operations` plus those 12;
  a worker's hand-off uses its own kernel budget. The queues between
  the components and the root are uncapped. Store routes and client
  ending and secret-owner correlations grow with admitted work, rather
  than reserving a largest output envelope.
- **Starting lasts until decisions open.** The reference service keeps
  answering clients not ready while the core's restart is under way,
  including after `RestartBegin` has been submitted. The end of starting
  is the core's readiness, as section 3 requires.
- **Its server,** the example of an application's (`browser/README.md`,
  section 12): on the clients' listener, it mounts the clients'
  component at `/jig` with jig's default HTTP mapping and its example
  guards whole, its `Host` allow-list the address its printed link
  names; and it serves jig's test application's bundle itself, as
  skein's `ui/web/bundle.md`, 4 says, under `/bundle/`, its page at every
  other GET and at the printed link's route, and one root file
  (`browser/README.md`, 3.5, step 5), with the reference page policy
  (`browser/README.md`, 12.1). The bundle is data its startup gives it, or
  none, which serves no browser.
- **The reference binary, `jig-test`,** beside it: its `main` reads its
  configuration with skein's startup kit (`configuration.md`), opens the
  store on a directory, and runs the engine and one test worker in one
  process (section 10), the worker's agents smith's real agent
  processes, run as its own `agent` command (section 6). Being its
  agents' own executable, it serves smith's two agent modes, given as
  its first argument before it reads a command, as `smith-agent` does:
  `jig-test --mcp-relay 127.0.0.1:PORT`, a program's MCP relay, and
  `jig-test --program-child PARENT PROGRAM ARGUMENTS...`, each program
  process a program-backed agent starts (smith's
  `protocol/programs.md`, 5.1 and 5.3). It prints the
  printed link's URL once the clients' component gives its secret,
  `http://<its listener>/jig/v1/sign-in/link?secret=…`, on a line of its
  own, by which a native client on loopback signs in (`client/sign-in.md`
  ), and which the browser tier's harness opens
  (`browser/README.md`, section 11 and `browser/testing.md`). It keeps no local link and drops
  the secret `LocalLink` gives (`client/sign-in.md`, *The local
  link*). Its server's settings are members
  of jig's test application's own configuration, as every application's
  server's are its own, beside jig's part, which names none of them
  (`configuration.md`, section 3): the clients' listener, a loopback
  address or one with its certificate and key files, the key read as a
  private file (`configuration.md`, section 6); and the bundle's
  directory, or none. Its startup reads the bundle there
  and gives it, its page included, to its server as data: jig's test
  application's browser client, built from the reference shell,
  `jig-client-test-shell-web` (`browser/README.md`, 3.1); none serves no
  browser. Its LLM endpoint is its configuration's. Its periods are the
  smallest source's: one open
  period per configured project, at the deployment's first start
  (`configuration.md`, section 5). An application copies it for its own
  `main`, and jig's end-to-end and live tests run it (`service.md`, section 8).
- **`ops`'s service follows them,** as the reference for an application
  with agents in the engine and no workers: its crates, its server and
  its periods are `../examples.md`'s (sections 4 and 9.1).
- **A worker's reference** is the reference service's test worker, which
  jig's channel world runs too (`channel/README.md`, section 11), since `ops`
  has none.

## 8. Testing

Each within jig's suites' budgets.

- **The domain worlds** run without any of this (`../domain/testing.md`).
- **The components' worlds** test each component alone (`channel/README.md`,
  section 11; `llm.md`, section 8; `store/README.md`, section 9; `client/README.md`,
  section 11), and the default HTTP mapping's engine end and its guards
  in their step tests (`browser/testing.md`).
- **The protocol world:** the reference service (section 7), every
  component in it, over skein's simulated io (testing-strategy.md, 2.7):
  a store on the simulator's crashing disk, scripted parties'
  connections to its server, through jig's default HTTP mapping, skein's
  fake LLM peer, smith's agent processes over skein's
  fake LLM
  spawned on skein's fake machine beneath the test worker (smith's
  `protocol/host.md`, section 7), and the application's systems' fakes,
  the process killed and started again at drawn moments, and drained by
  a shutdown. Its referee is the conformance world's, seen through bytes:
  nothing a party, a host or a system saw depended on a commit that was
  lost; and a drained shutdown ends with nothing waiting for a commit.
- **End to end** (testing-strategy.md, 2.9): the reference binary,
  `jig-test`, started on a scratch directory with skein's fake LLM peer
  as a process of its own on loopback (skein's `testing/fakes/llm.md`) as its
  endpoint, a person's requests answered through a native client on
  loopback, over jig's default HTTP mapping,
  runs taken by smith's real agent processes, then the binary killed and
  restarted, and the work found where it was. The focused suite has four
  paths through `main`: that restart story, a binary told to stop with a
  sign-in under way, and its fixture bundle served under the reference
  policy, each through the real loop, whose shared `Deadline`
  classifies unanswered waits under measured starvation; and a table of
  its agent modes (section 7), a row each, the program child running a
  scripted Claude Code's `auth status` whose output smith reads as its
  status check does, with no loop. Account sign-in,
  refresh, sign-out and moves are covered in their lower worlds.
- **The live layer:** `jig-test` with a real LLM account, a key added
  by a request, a sign-in made as the test starts, or a program's login
  on its test worker's host (`llm.md`, 5.4 and 5.5), a native client
  signing in by the printed link, and a request answered by a real
  model, within a bound on calls. No record is read from a file.
  Run `scripts/live-sign-in.sh` for a fresh ChatGPT sign-in, a model answer
  and confirmed sign-out; it prints the browser link and the SSH tunnel
  needed for the fixed loopback callback on port 1455, and however the
  run ends, the sign-in's record does not outlive it.
  The live story declares low effort on the engine's and agent's models;
  focused checks read the real, fake and refusing configurations through
  startup's readers, without network or a person.
  After the sign-in, the story prints a line at each change of the
  chat's run, and a run that can no longer answer (a failed or refused
  attempt, a hold, another ending) fails it at once in those words. A
  failed run prints its agent trace's failures (class, status, code)
  before the trace is erased. The test catches nothing
  (`temper/docs/design/draft/panics.md`, 5): the story's run and its
  final check return their findings, and on one the test prints those
  failures and its wall time, then fails. Its scratch is kept whatever
  ends it, and the script, once the test process has ended (on a
  signal, having stopped the runner and reaped it), erases the records
  and traces, keeps the rest on a failure and removes it on a pass. The refused-plan run-failure words are
  checked by the focused step test
  `live_sign_in::tests::a_refused_plan_is_reported_with_the_run_failure`.
- **The browser tier:** `jig-test` serving jig's test application's
  bundle and the mapping at `/jig`, and Chromium, children of the
  harness's loop, a person in a page, and jig's attack stories against
  its server (`browser/testing.md`); a suite of its own, outside the
  default and fuzzy suites.

## 9. Open questions

- **Approval on a listener beyond loopback** (3): the public-name flow
  and its transport are `client/README.md`, section 12's later work. Enabling
  approval continues to require a loopback clients' listener.

- **Generating the routing:** `iterate`'s routing has one shape, as the
  root's does; whether it is copied and filled, or generated.
- **Supervision:** what restarts an engine that stopped, and how often,
  is the deployment's; whether jig's guidance names one.
- **A worker stopping in another process** (section 6): its Shutdown
  cancel reaches the running engine, where an agent's cancelled answer
  counts as `Failed(Agent)` and spends an agent try. Whether an orderly
  worker restart should instead interrupt that attempt is left for
  workers apart. Default: the no-try interruption applies only when
  the engine stops its own in-process agent claims (section 5).
- **Programs after a forced agent stop** (section 10): a second signal
  kills an agent's group, leaving a program in its own group outside the
  kill. Contained trees (README.md, section 10) must cover both, including
  in one process. Default: the orderly stop cancels runs so agents end
  their programs; forced stops have no such guarantee.
- **The server's home.** The server and its bundle helpers (11.4) name
  no type of jig's, and a generic mechanism belongs in skein. `kind`
  spells skein's closed list a second time, since skein's own is in
  `skein_ui_bundle::types`, a build tool step code can't link. Default:
  they stay in `jig-protocol-engine-clients-http` until skein has a
  server to offer.
- **smith's agent modes in one call** (section 7): a binary that is its
  agents' executable gives `--mcp-relay` and `--program-child` to
  smith's agent shell itself, one line each copied from the reference
  binary, so a copy that misses one fails each program-backed run at its
  start. One function of smith's agent shell taking the first argument
  and running the mode it names would make the copy a line, and give
  every such binary a mode smith adds; it changes smith's interface.
  Default: a line each, pinned by each binary's table of its agent modes
  (the reference binary's in section 8).
- **`sources::Component::start`'s arguments** (11.2): a public seam kept whole,
  past clippy's count, since an application's service reads it. Default: as it
  stands, until its signature changes for another reason.

## 10. One process

For one person's machine, development and tests (`README.md`, section
4): an engine and its workers in one process, from the same parts as
between hosts.

- **One service, one loop.** The application's `iterate` owns the
  engine's protocol layer and each worker's as children, each with its
  own root and journal, and routes io's events by the tokens each gave.
  They share io and the loop, and nothing else: no root reads another's
  state, and nothing crosses between them but bytes.
- **Joined by in-process addresses.** The engine's workers' listener and
  each worker's link use an in-process address (skein's `runtime/io/sockets.md`, 5),
  so the channel's opening, authentication, liveness, relays and grants
  run as they do between hosts, in plaintext, with no TLS.
- **A client on loopback.** The clients' listener, the application's
  server's, is a TCP listener on a loopback address; a browser or a
  native client reaches it as it reaches any engine's, through jig's
  default HTTP mapping in the reference, and a person signs in by the
  link `main` prints at start (`client/sign-in.md`; `browser/README.md`,
  section 11).
- **A client in the process.** A client the process holds itself (a
  service acting as a party, a bot, a command the same binary runs)
  reaches the clients' component by an in-process transport, the
  application's routing, with no listener and no HTTP:
  - the client's exchanges, and its stream opened and closed, go to the
    component as its exchanges and its stream opened and gone; the
    component's replies, streamed messages and the stream's end come
    back as the client's events (`client/wire.md`, 2.2 and 2.3);
  - they cross as jig's messages' bytes, as on any transport, so each
    end runs its own code below the transport and keeps its own limits,
    and nothing crosses but bytes, as between roots;
  - a delivery is taken once the client has stepped with it, which
    paces the stream as a connection's draining does (`client/README.md`,
    section 5);
  - the client signs in as any service does, with its credential in
    hand (`client/sign-in.md`), since the component checks every
    exchange's credential, whatever carried it.

  jig ships no crate for it: it is a few arms of the routing, each
  message onto its peer's.
- **Agents as child processes,** as between hosts (section 6): one
  binary may serve as the service and each agent it spawns, as the
  reference binary does (section 7). Or, as a worker's root's choice, in
  the worker's process with `smith-inline-agent`, with no containment: a
  person's machine accepts that, a shared host does not.
- **Credentials as between hosts:** the engine grants them with each
  assignment over the in-process link (`channel/README.md`, section 7).
- **Starting and ending together:** configuration for all, then the
  engine's start (section 3), then each worker's (section 6). A signal
  drains the engine (section 5); once it has drained, the service ends
  the workers with no grace for their turns and answers, since no engine
  will come back while they live. Each worker's hub cancels every run,
  as at a worker's ending (section 6), so that each agent process ends
  what it started, a program among them (smith's `protocol/programs.md`,
  section 2), within its host's cancel, then kill (smith's
  `domain/host.md`, section 4). Once no agent process is left, the
  service closes the worker, its other children's groups killed as io
  closes them. A second signal closes the workers at once: their agent
  processes' groups are killed, and whatever an agent started in a group
  of its own outlives it until agents run in contained trees (README.md,
  section 10). Their runs end with the service, as the engine's own
  slots' do, so the engine settled their agent claims as interrupted when
  it heard it was stopping, spending no try of their tasks
  (`../domain/engine/restart.md`, 1). After a crash the next start settles
  them at once (`../domain/engine/restart.md`).
- **Its world** is the protocol world of section 8, with workers: the
  engine's and a test worker's services in one loop, joined by in-process
  addresses, the test worker's agents smith's agent processes over skein's
  fake LLM.
  Joined sockets are io's own and never reach skein's simulator (skein's
  `runtime/io/sockets.md`, 5), so the link's cuts and stalls are the channel's protocol
  world's, over streams cut at random (`channel/README.md`, section 11).

## 11. The service parts

What every application's protocol layer writes alike, jig ships as
product code, which the reference service and an application's service
both compose. A fix in a part reaches every application, and one ordering
of token refreshes holds everywhere (`jig/AGENTS.md`). An application keeps
its routing, `iterate` and `main`, its connectors and its own settings.

- **A part is a host,** step code a service owns as a field and drives
  from `iterate`. It holds no root type, and no trait, generic item or
  callback (programming-model.md, 10.3). `jig_protocol_store::host::Drivers`
  is the form.
- **Values in, typed outputs out.** A part takes values at `new`. What it
  can't decide it hands its owner as a typed output, which the owner pulls
  in its own pass and routes: tagging tokens from its own table, pushing
  onto its own root queue, calling its own store. A pass hands out in the
  order it holds, so the owner's root sees events in the part's order.
- **A difference is the owner's.** Where two applications act differently
  on one fact, the part reports the fact and each owner acts; it takes no
  setting that names an application. A difference of amount is a value.
- **What two services have in common is each one's composition.** An
  application's pass, adapters, tokens and sizes are what this section
  leaves it, so the lines two services share there are no copy for a part
  to take.
- **A part that holds credentials shows none in `Debug`:** it implements
  no `Debug`, or a hand-written one that counts them.

| Part | Crate, module | It holds | The application keeps |
|---|---|---|---|
| the accounts' host | `jig-protocol-accounts`, `host` | the accounts' component and its provider templates, the pass over the store's answers and the component's effects, the table's grants | its tokens, its store's calls, its root's event, what an invalid restored secret means |
| the store's host | `jig-protocol-store`, `host` | the component and the file drivers behind one owner's calls: requests, the secrets' answers in the same pass, closing, submissions by lane | its routes, its route's table and answers, its sizes and wires, its tokens |
| the program sources | `jig-protocol-worker-agents`, `sources` | the source's command, one at a time, and the program's environment, hidden from `Debug` | how settings name a program |
| the worker route | `jig-protocol-worker-agents`, `route` | smith's host half's records to and from the channel's values | its application's decoding of parts, what a delivery's evidence must say (`channel/README.md`) |
| the workers' side | `jig-protocol-engine-workers`, `host` | the channel component, its listeners and sockets, closing them | its assignment's encoding, its parts' decoding, what a lost listener means |
| the server | `jig-protocol-engine-clients-http`, `server`, `bundle` | one listener's connections, their deadlines and page policy, each call handed out as routed; what serving a bundle has in common | its routes, its bundle, its pages' extra fields, its answers' policy |
| the clients' side | `jig-protocol-engine-clients`, `host` | the bridge between the component's secrets and the store's and sign-in endings; beside them, the printed link in `jig-protocol-engine-clients-http` and the settings every application fills alike in `jig_protocol_config::clients` | its pass, its calls, its families |

`jig-protocol-worker-agents` links smith, so an application whose workers
run no agents leaves it out, as `README.md`, 5 says. The kernel's hand-off
between a process's owners of records is skein's, `kernel::Routes`
(skein's `runtime/kernel.md`, 5).

### 11.1 The accounts' host

`jig_protocol_accounts::host::Accounts` is the accounts' component with
its effects and its io, and one pass over them. The engine's accounts'
component alone refreshes a record (`../domain/accounts.md`, section 2),
so this order is jig's: the owner's `iterate` calls the host, and holds
no step of the order but its own store's and root's. The host takes no
store: it hands out its store requests, and the owner calls its own store
and feeds the answer back.

- **At `new`:** the application's name, jig's part of the configuration,
  the crypto seed, the drawn installation identifier, the secrets' store's
  limits and a `Runtime`, which holds each provider's endpoint and
  origins, prepared by the shell, and `trust`, the bounds of the trust
  roots it prepared. With bounds, a provider on TLS is refused if either
  is zero; with `None` the shell answers for its roots and the host checks
  nothing. `jig-protocol-accounts` links `jig-protocol-config` for this,
  which links nothing back. The files a shell reads for a configuration
  are `jig_protocol_config::Files`; this host reads none.
- **A pass.** `begin` takes the iteration's time, whether the store's
  start mark is durable, the installation, and the secrets' answers the
  owner holds for the host. In this order, the component hears `Opened`,
  once; it steps with each answer; it fires. `begin` returns the pass's
  count: the effects waiting then, and the held event if there is one.
  The owner calls `output` that many times, each taking one effect in the
  component's order, and ends the pass with the table's `expire`.
- **What stays in.** `output` takes the owner's table, and does itself
  what needs no owner: an io request joins the host's `io`, a grant or a
  removal changes the table, a source's release ends there. It returns
  `None` for those, so a grant is in the table before the event that
  announces it leaves. Three things come out as `Output`:
  - `Account`, an event for the root: the owner wraps it and pushes it;
  - `Store`, a request on the host's exchange token: the owner tags the
    token and calls its store;
  - `InvalidSecret`, a restored secret of the wrong kind: what that means
    is the owner's.
- **What the owner keeps.** It routes each output before it asks for the
  next, so its root, its store and the table see the component's order.
  A store that answers within the pass (a read does) goes to `answer` at
  once. An effect that any step makes after `begin` has counted waits for
  the next pass.
- **Tokens.** The host's exchange tokens and io owners sit below the top
  four bits (skein's `runtime/lib.md`, 4). The owner tags `io`'s requests and
  `Store`'s exchange in its own namespace, and hands io's events back
  untagged. `answer` and `begin` take a store answer's namespace off.
- **Room.** The queue to a root is uncapped (section 2). An owner whose
  queue may be full gives an `Account` back with `hold` and ends the
  loop. The host keeps it in one slot: it leaves first in the next pass,
  and the host counts as work until it has. The rest of the count waits
  with it, and the next `begin` counts them again.
- **Aborting.** After `abort` the store is closing: the host opens
  nothing and drops the store requests it holds, and an open it will
  never make is no work.
- **Work.** `work_pending` takes `marked`, whether the store's start mark
  is durable, since the open waits for it. An owner whose own gate
  holds the host back until the mark passes `true`.
- **Reads for an owner's diagnostics and tests:** whether it has opened,
  the effects waiting, whether it holds an event, and the component, for
  what the component reports of itself. None of them formats a value.

### 11.2 The worker's agents

`jig-protocol-worker-agents` holds two parts of a worker's protocol
layer: `sources`, a component, and `route`, functions that keep no
state. Both speak the hub's values and smith's, and neither names a
root.

- **One answer per call.** The program sources run one command at a
  time, so each of `start`, `from_file`, `from_io` and `fire` returns the owner and
  the answer it settled, if any, and the service pushes it as its root's
  event in the same pass.
- **A name smith refuses is an answer.** `start` checks a source's name
  against smith's grammar; a name it refuses answers `Unconfigured`, and
  no command runs.
- **How settings name a program is the service's.** `Resources::new`
  takes smith's `Programs`, the environment, the bound on a path,
  the effective user and an optional shell-opened programs parent.
  A missing parent makes a check unconfigured. Each service keeps one
  small function from its settings to `Resources`.
- **Inspection has file I/O below it.** Both worker services keep a small
  file stage with fixed limits for one descriptor and one request, with
  no configuration member. It routes completions in the worker's token
  range and file events by owner to smith's inspection, and participates
  in deadlines, settled and emptiness checks. The programs parent closes
  through the same stage when the worker closes.
- **The route takes smith's values and the hub's,** and gives them back,
  or the hub's `Undecoded`, what of a start did not decode, or `route`'s
  own `Processed`, an agent process's event as a live piece or as smith's
  process event. A worker's root hands its payloads out in its own
  enums, in a domain crate, and `jig-domain-host` links no smith, so
  each service keeps a few lines matching those enums onto the route.
- **A delivery's evidence is a value.** The application keeps it and
  says what it must say (`channel/README.md`, 5.2), and `decode` takes that as
  `Evidence`: `Text`, where the bytes are a receipt's or a refusal's
  words, or `Reply`, where they are smith's `DeliveryReply`, which must
  decode and agree with the outcome the core kept. `evidence` makes
  that check alone. `decode` reports the first thing that fails in the
  start's order: the charter, the transcript, then each answered call,
  so a delivery's evidence is checked in its place.

### 11.3 The workers' side

`jig_protocol_engine_workers::host::Workers` holds the channel
component, the workers' listeners and their accepted sockets, behind
one owner's calls. The owner makes it from the engine's configuration,
its private files, a payload version and `Tokens`, the namespaces of
the listeners' and the connections' tokens, and drains `io` as it
drains every component's. This `host` is the one module of the channel's
crate that reads configuration.

- **The root's events are pulled.** After every call that moves the
  component (`down`, `up`, `iterate`, `close`, `abort`) the owner reads
  `pending` and calls `output` that many times, before anything else in
  its pass. Each call takes one of the component's requests, in order.
  `output` does itself what needs no owner: an io request joins `io`, a
  socket is bound or forgotten, the component's close is noted, and it
  returns `None` for those. `Hello`, `Up` and `Lost` come out as
  `Output`, and the owner routes each before it asks for the next, so
  its root sees the component's order. An `Up` keeps its parts' bytes:
  the owner decodes them or drops them.
- **A grant's value comes in.** The table of values is the application's.
  `grants` names the grants a `LinkDown` carries; the owner looks up
  each, in order, and hands `down` the finished `Event::Down`, one slot
  per grant. An assignment is the owner's whole: it encodes one the same
  way.
- **What a lost listener means is the owner's.** `up` returns `Seen`:
  `Taken`, `Unknown` for an event the host has no use for, or
  `ListenerClosed`, which says whether the host was already ending. An
  owner may stop its engine on a listener it did not close, or carry on.
- **Closing.** `close` closes the listeners and drains the component;
  `abort` aborts them. A listener that opens after either is closed or
  aborted at once. `closed` holds once the component has closed, no
  socket or listener is left and `io` is empty.
- **The bound address.** The host keeps each listener's configured
  address, for the owner's report of a lost one, and the address last
  bound, in `listening`, until the owner takes it.
- **Its `Debug` counts.** A queued write holds an encoded grant, and a
  read what a worker sent, a credential among them. So `Workers`,
  `Component`, `Request` and `Event` show counts and variants, and no
  bytes.

### 11.4 The server

`jig_protocol_engine_clients_http::server::Server` is one listener and
its connections, each with skein's HTTP server over it, their deadlines
and the page policy. It knows no route: what a call means is its owner's.

- **Every call goes to the owner.** A head comes out as `Routed::Call`,
  and each later event of that call as `Routed::Http`. The owner hands
  the call to the mapping, or answers it from its own routes through
  `answer`, and ends it with `done`. Its pages and its bundle are its
  own (`browser/README.md`, section 12).
- **The page policy is on every response:** `policy::FIELDS`, the
  reference's (`browser/README.md`, 12.1), but for a field the response already
  holds.
- **Amounts are `Limits`:** the connections, skein's HTTP limits and the
  stall. `valid` says whether limits can run at all. A full listener closes an
  idle connection, with no call or request begun, and admits the next once that
  close settles; it rejects only when none is idle.
- **A peer's end is a value.** `Limits`' `peer_end` says what a peer's
  end does while a call is unfinished. `PeerEnd::Connection` ends the
  connection and tells the owner the call's connection closed, as
  `client/default-http.md`, 4 asks. `PeerEnd::Call` leaves the connection to
  skein's server, which lets a response go on once the request is all
  read and says nothing of that end above (`skein/docs/design/net/http/server.md`,
  5): the owner hears the call end with its response.
- **A call the owner holds.** `hold` keeps a call's connection to a
  deadline of the owner's, past the stall, and `release` gives the call
  back to the stall.
- **Closing.** `stop` closes the listener. `close` also ends each
  connection as its call ends; once the owner says `finish`, that no
  response is left to come, it ends those with a call still open, after
  what they have queued. `abort` ends everything at once, and `abandon`
  one call's connection.
- **Its `Debug` counts.** A head holds its cookies and a response its
  `Set-Cookie`, so `Server` shows its listener, its phase and how many
  connections it holds, and `Routed` its variant and its call's token.
- **What serving a bundle has in common** is `bundle`, beside the server:
  `File` and `kind`, a file's type by skein's closed list (`skein/docs/design/ui/web/bundle.md`,
  4), a `File` holding its compressed form where the bundle has one;
  `compressed_of`, the file a `NAME.gz` is the form of; `target`, a
  head's path; `file`, the file a path names under its digest; `field`,
  one header; `demand` and `chunk`, a body's way out in chunks of the
  server's `send`. They hold no state and answer nothing.
  The owner keeps its `Bundle`, its route, its answers, what it caches
  and how a call ends (`browser/README.md`, section 12).

### 11.5 The store's host

`jig_protocol_store::host::Hosted` is the store's component and
`Drivers`' three file drivers (`store/README.md`, section 8) behind one owner's
calls, with the queues its answers wait in. The owner makes it from the
store's limits, the application's sizes, the models and a seed. `Drivers`
stands alone for a world that drives the drivers itself.

- **Answers are pulled.** `above` holds the store's terminals in order,
  and the owner's dispatch takes them. `secrets` and `sources` hold the
  terminals that dispatch hands its clients' side and its accounts'
  host, until their passes take them (`store/secrets.md`). These three
  and `component` are public fields: the owner's dispatch reads them and
  a test's fake terminal goes in through them, and a method for each
  would hide nothing. `mark_durable` says whether the start mark is
  durable, as every owner's pass asks.
- **Requests.** `ask` takes any request. `secrets` takes a secrets'
  request, and returns the answer to a `Get` or a `Load`, which stays in
  the pass; a write's terminal comes through `above`.
- **A get the store can't answer yet is a value.** With
  `NotReady::InPass`, `secrets` returns `SecretsNotReady` as it does any
  other answer to a get. With `NotReady::Above`, it stays in `above` for
  the owner's dispatch.
- **Submissions by lane.** `pop_submission` hands out each kernel record
  with its lane, and the owner tags it from its own table; it takes a
  completion's tag off and gives `complete` the lane (`Lane`). The
  namespaces are the owner's.
- **Closing.** `close` closes the drivers and drops what the three queues
  hold. `idle` and `closed` answer for the drivers and the queues; the
  owner adds what it holds.
- **Its `Debug` counts** its queues: a terminal can hold a kept digest.
- **What the owner keeps:** its routes and what they hand up, its sizes,
  the wires it gives the component after `new`, the staging queue its
  tagged submissions wait in, and whether the store has opened and with
  which installation, which its dispatch learns.

### 11.6 The clients' side

`jig_protocol_engine_clients::host` holds what stands between the
clients' component and the rest of a service. The component, the mount
and the server stay fields of each service's own `Clients`, whose pass is
the application's: its calls and families have their places in that
order, and no part cuts across it.

- **`Bridge` carries secrets.** `ask` takes one of the component's
  secrets' requests, names it with a token in the owner's namespace,
  keeps what it was for, and returns the token and the store's request;
  the owner calls its own store with them. `answer` takes the store's
  terminal for such a token and returns the component's `SecretsAnswer`.
  When a request goes is the owner's: it may hold requests back until its
  store's start mark is durable.
- **`Endings` holds a sign-in's ending until its own commit is durable**
  (`../domain/root.md`, section 5). `ended` records a sign-in a commit
  ends, `committed` the durable number, and `next` hands out the next
  ending whose commit is durable, in commit order; the owner calls the
  component with each. The most commits it holds is a value at `new`,
  and an owner with a bound asks `room` before it commits. `ending` reads
  the sign-in from a core key: which of an application's writes end one
  is its own to read.
- **The printed link.** The mapping's `address_text` is a bound address
  as text, and its `printed_link` joins that to the path its `link_path`
  gave. An owner with a public origin writes its own.
- **The component's settings are a value,** `Settings`. This crate can't
  read configuration, which names it, so what every application fills
  alike is `jig_protocol_config::clients::settings`, beside `models`.
  `clients::Application` carries what differs: the printed link's
  identity, the families served, the application's own public kinds and
  the moved file's bound. The two seeds come in as they are. An owner
  with its own issuer sets `provider` and `addresses` on the result.
- **What the owner keeps:** its pass, its calls and their replies, how a
  root's event becomes a component's, which starting requests it parks,
  and when it builds the component.
