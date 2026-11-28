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
jig ships for it (`service-parts.md`, section 1). The mechanics are
skein's programming-model.md, sections 2 and 4. What is still open is
listed in `service.md`, section 9.

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
  `client/default-http.md`; `browser/serving.md`, section 1).
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
  service parts (`service-parts.md`, section 1) and whose routing, `iterate` and startup are
  written to be copied, like the reference root, which mount the clients'
  component at `/jig` with the default HTTP mapping and serve jig's test
  application's bundle themselves (`service.md`, section 7).
- **What every application writes alike, jig ships:** the accounts' host,
  the store's host, the program sources, the worker route, the workers'
  side, the server and the clients' side, as product code (`service-parts.md`, section 1).
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
    ├── <server>                      the clients' listener, its routes, its pages and bundle: the application's, on jig's server part (section 11; browser/serving.md, section 1)
    ├── jig-protocol-engine-clients-http   jig's default HTTP mapping, its engine end, mounted under the server's prefix (client/default-http.md; browser/serving.md, 1.2)
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
  (`channel/owners.md`, 1.3); they keep no state.
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
  bundle and its own routes itself (`browser/serving.md`, section 1), and
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
    (`client/sign-in.md`; `browser/sign-in.md`, section 1).
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
   (`../client/domain/contract/answers.md`, section 1), but for a
   stream's `me`: the watch a stream opens with, whose hello waits for
   it (`client/default-http.md`, section 2), waits in the service until
   decisions open and then reaches the root in its order, and its
   unwatch meanwhile takes it unsent. Refused, it would end the stream
   before its hello with a `409`, the want of room for its `me`, which
   the start is not, and a failed load a browser logs. Where configuration
   enables the printed link, or a browser's approval, the startup first
   checks that the clients' listener, a member of the application's own,
   is on a loopback address, and refuses the start, naming both, if it
   is not (`configuration-engine.md`, section 1, clients). With the printed
   link enabled, only after that check does the component give the link's
   secret, and `main` print its URL, on the application's origin and
   route (`client/sign-in.md`; `browser/sign-in.md`, section 1). Approval
   alone prints nothing; its secret is drawn when a client asks.
5. **The root's restart:** `iterate` gives the root its start, and the
   core's script asks for one step at a time; the protocol layer does the
   store's and the connectors' parts of each step as the root asks; the
   core wants every signed-in source from its first step
   (`../domain/root.md`, section 8; `../domain/engine/README.md`, section 6).
6. **Decisions open** once the core says so, which is once the engine's
   own records are in and its runs adopted, before any connector has
   caught up (`../domain/engine/restart.md`, steps 4 and 5); from then on
   the root takes every event, and clients are answered whatever a
   connector's system does.

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
  (`../domain/engine/decisions.md`, 1.1).
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
     commit, and ends every watch saying so (`../domain/engine/restart.md`, 5).
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
    ├── agent processes           each in a process group of its own, or a contained tree where skein gives one (needs.md, section 1)
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
  decoded into jig's activity (`channel/owners.md`, 1.1 and 1.4).
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
  the hub reports what it hosts once the link is up (`channel/worker-link.md`,
  1.1).
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
  layer, composed from jig's service parts (`service-parts.md`, section 1), with the host
  tools' translation and its test connector's system answered in memory.
  Its routing, its `iterate` and its startup are what an application
  copies; the parts it composes, an application composes too. Its
  engine runs no agents of its own and a test worker's (the link,
  smith's host half and its agent processes, the test workspace), each
  with its `iterate`, composed in one process (`service.md`, section 10), with a
  startup function that takes configuration's bytes and returns the
  service. jig's protocol world runs it (`service-testing.md`, section 1).
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
- **Its server,** the example of an application's (`browser/serving.md`,
  section 1): on the clients' listener, it mounts the clients'
  component at `/jig` with jig's default HTTP mapping and its example
  guards whole, its `Host` allow-list the address its printed link
  names; and it serves jig's test application's bundle itself, as
  skein's `ui/web/bundle.md`, 4 says, under `/bundle/`, its page at every
  other GET and at the printed link's route, and one root file
  (`browser/application.md`, 1.5, step 5), with the reference page policy
  (`browser/serving.md`, 1.1). The bundle is data its startup gives it, or
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
  `protocol/programs-claude-code.md`, 1.1 and 1.3). It prints the
  printed link's URL once the clients' component gives its secret,
  `http://<its listener>/jig/v1/sign-in/link?secret=…`, on a line of its
  own, by which a native client on loopback signs in (`client/sign-in.md`
  ), and which the browser tier's harness opens
  (`browser/sign-in.md`, section 1 and `browser/testing.md`). It keeps no local link and drops
  the secret `LocalLink` gives (`client/sign-in.md`, *The local
  link*). Its server's settings are members
  of jig's test application's own configuration, as every application's
  server's are its own, beside jig's part, which names none of them
  (`configuration-engine.md`, section 1): the clients' listener, a loopback
  address or one with its certificate and key files, the key read as a
  private file (`configuration.md`, section 6); and the bundle's
  directory, or none. Its startup reads the bundle there
  and gives it, its page included, to its server as data: jig's test
  application's browser client, built from the reference shell,
  `jig-client-test-shell-web` (`browser/application.md`, 1.1); none serves no
  browser. Its LLM endpoint is its configuration's. Its periods are the
  smallest source's: one open
  period per configured project, at the deployment's first start
  (`configuration.md`, section 5). An application copies it for its own
  `main`, and jig's end-to-end and live tests run it (`service-testing.md`, section 1).
  When it ends on its own, at no signal (a failed commit, a component's
  or a listener's failure), its `main` writes why to stderr in one line,
  `jig-test: stopped: <reason>`, before its failure status: the reason's
  words, never a secret or a sign-in's address.
- **`ops`'s service follows them,** as the reference for an application
  with agents in the engine and no workers: its crates, its server and
  its periods are `../examples.md`'s (sections 4 and 9.1).
- **A worker's reference** is the reference service's test worker, which
  jig's channel world runs too (`channel/testing.md`, section 1), since `ops`
  has none.

## 9. Open questions

- **Approval on a listener beyond loopback** (3): the public-name flow
  and its transport are `client/README.md`, section 12's later work. Enabling
  approval continues to require a loopback clients' listener.

- **Generating the routing:** `iterate`'s routing has one shape, as the
  root's does; whether it is copied and filled, or generated.
- **Supervision:** what restarts an engine that stopped, and how often,
  is the deployment's; whether jig's guidance names one.
- **A worker stopping in another process** (section 6): the attempts
  its stop cuts are suspended, spending no try
  (`../domain/engine/executors.md`, 6), but an agent's cancelled answer
  that reaches its hub before the hub hears the stop is still the run's
  own failure. Default: so until an agent with a live host leaves its
  stop to its host (`../domain/engine/executors.md`, 6.7).
- **Programs after a forced agent stop** (section 10): a second signal
  kills an agent's group, leaving a program in its own group outside the
  kill. Contained trees (needs.md, section 1) must cover both, including
  in one process. Default: the orderly stop cancels runs so agents end
  their programs; forced stops have no such guarantee.
- **The server's home.** The server and its bundle helpers (`service-parts.md`, 1.4) name
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
  (the reference binary's in `service-testing.md`, section 1).
- **`sources::Component::start`'s arguments** (`service-parts.md`, 1.2): a public seam kept whole,
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
  link `main` prints at start (`client/sign-in.md`; `browser/sign-in.md`,
  section 1).
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
  of its own outlives it until agents run in contained trees (needs.md,
  section 1). Their runs end with the service, as the engine's own
  slots' do, so the engine settled their agent claims as interrupted when
  it heard it was stopping, spending no try of their tasks
  (`../domain/engine/restart.md`, 5). After a crash the next start settles
  them at once (`../domain/engine/restart.md`).
- **Its world** is the protocol world of `service-testing.md`, section 1, with workers: the
  engine's and a test worker's services in one loop, joined by in-process
  addresses, the test worker's agents smith's agent processes over skein's
  fake LLM.
  Joined sockets are io's own and never reach skein's simulator (skein's
  `runtime/io/sockets.md`, 5), so the link's cuts and stalls are the channel's protocol
  world's, over streams cut at random (`channel/testing.md`, section 1).
