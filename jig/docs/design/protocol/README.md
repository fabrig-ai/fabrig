# jig's protocol layer

Provisional, 2026-10-10, revised 2026-10-19. The design of what lies
between jig's domains and bytes: the components an application's protocol
layers own to carry jig's vocabularies to the store, to workers, to LLM
providers and to clients; what an application writes around them; their
versions; the secrets they hold; and what they need from skein and smith.
jig as a whole is `../README.md`, whose section 3 says every process is
the application's and jig supplies the pieces that speak jig's
vocabularies. The mechanics are those of skein's `programming-model.md`,
section 4. What is still open is listed in section 14.

## 1. In one page

- **Translation, nothing else.** jig's protocol components turn bytes
  into jig's domain vocabularies and back. Every decision stays the
  domain's: the core's, the hub's, the client's. A component runs the
  timers and the exchanges the domain asks for, and its own deadlines,
  and says what happened (programming-model.md, section 4).
- **Components, owned by the application.** Each component is a child of
  a process's protocol layer, with its own limits, worst case, deadlines
  and most outputs per entry point, as a child domain is of a root. The
  application's protocol layer owns jig's components beside smith's,
  skein's and its own connectors', and routes between them, io and its
  root (`service.md`).
- **Five boundaries in the engine** (section 4): the store, workers,
  LLM providers and accounts, clients, and connectors' systems. The
  first four are jig's, each with components of jig's; connectors'
  protocols are the application's. At the clients' boundary jig's
  vocabulary is its messages, and the transport that carries them is
  the application's.
- **Clients, by jig's messages.** The clients' component,
  `jig-protocol-engine-clients`, takes jig's messages with an opaque
  credential and answers with jig's messages, and knows no HTTP, path,
  cookie or listener. The application's service owns the listener, its
  routes, its URL space and every HTTP and browser guard, and mounts
  the component under any prefix. jig ships a default HTTP mapping, both
  ends, with example guards, to use, wrap or leave (`client/wire.md`, section 2 and `client/default-http.md`
  ; `browser/serving.md`, 1.2).
- **jig owns credentials, not sign-in flows.** The clients' component
  mints, verifies, expires and revokes credentials. Its built-ins are
  the printed link, on loopback, and services' credentials; any other
  way of signing in is the application's, on its own routes, and ends
  in a `Mint` its server sends (`client/sign-in.md`).
- **Generic mechanisms are skein's,** as for smith: framed channels,
  codecs generated from schemas, the key-value store, LLM connections,
  OAuth, HTTP and server-sent events, TLS, contained processes. jig keeps
  what is about its own vocabularies.
- **Agents and their records are smith's.** A hub's agent capability is
  `smith-host-domain` or `smith-inline-agent`; jig ships none (section
  4). The hub, the core and jig's channel carry smith's payloads (the
  charter, the transcript, turns' bodies, calls' names, results) as
  smith's own versioned bytes, and never read them; the root that
  composes an agent capability hands each out to its protocol layer to
  be decoded or encoded (`decisions.md`, section 1).
- **The application's parts stay the application's.** Where jig's records
  carry an application's value (a workspace's items, what a run left, an
  application's documents), they carry it as bytes the application
  encodes, versions and decodes in its own protocol layer.
- **Nothing is parsed in a domain.** Host tools' calls and runs' results
  are decoded, and calls' answers rendered, in the engine's protocol
  layer, for runs on workers and in the engine alike (`channel/owners.md`, 1.1
  and 1.3).
- **jig ships the store,** `jig-protocol-store`: skein-kv in the
  engine's process, jig's records and the application's in one commit,
  transcripts as files, secrets in a second store (`store/README.md`). The
  other components rely only on the boundary section 9 states.
- **Secrets never reach a domain.** A domain names a secret by number and
  generation; components hold values in tables and in the store's
  secrets' records, LLM accounts' records and keys among them, and
  they are filled in where they leave (section 8).
- **Configuration is read once, before the loop,** strict, composed per
  child as the root is (`configuration.md`).

## 2. Reading order

1. **README.md:** this page;
   **components.md**, the components the application composes in each process;
   **needs.md**, what the protocol layer needs from skein and smith;
   **decisions.md**, the decisions this design takes.
2. **service.md:** the engine's and the worker's services: the protocol
   layer as a parent of components, `iterate`, starting and ending, one
   process, the application's server, which owns the clients' listener
   and mounts the clients' component, and the reference binary,
   `jig-test`, with its service;
   **service-parts.md**, the components of the services;
   **service-testing.md**, the services' tests and worlds.
3. **channel/README.md:** jig's channel between the engine and its workers, and
   where host tools' calls are decoded; **channel/wires.md**, the wires a
   worker's opening states by layout and text, the engine's check, the
   kinds' table and terms; **channel/layouts.md**, how the engine
   converts between its layouts and a worker's, and what the worker's
   cannot keep;
   **channel/owners.md**, bytes of other owners and where they are decoded;
   **channel/liveness.md**, liveness and flow control;
   **channel/worker-link.md**, the worker's link on each side;
   **channel/testing.md**, the channel's tests and worlds.
4. **llm.md:** LLM calls in a host's own process, for
   `smith-inline-agent`'s runs, and LLM accounts: skein's accounts
   driver for sign-ins, keys from requests, program sources on their
   hosts, the translation, the table of values, and grants.
5. **configuration.md:** what an engine and a worker are told at start;
   **configuration-engine.md**, jig's part of an engine's configuration;
   **configuration-templates.md**, the templates and their members.
6. **client/README.md:** the client's protocol: jig's messages and their wire,
   the clients' component and its messages toward the application's
   transport and server, the default HTTP mapping's routes and stream,
   keyed requests, reads, watches, signing in and credentials, versions,
   the small store, shells;
   **client/versions.md**, the versions of the wire and its records;
   **client/small-store.md**, the small store's keys and encoding;
   **client/shells.md**, the native and browser shells;
   **client/testing.md**, the client's protocol tests and worlds.
7. **store/README.md:** the store: skein-kv in the engine, keys and records,
   commits and loads, transcripts, secrets, growth;
   **store/keys.md**, key order, owner tags and ranges;
   **store/commits.md**, commit ordering, durability and failure;
   **store/growth.md**, retention, admission and the map census;
   **store/testing.md**, the store's tests and worlds;
   **store/opening-owners.md**, owner sets and frames' sites at open;
   **store/opening-rehearsal.md**, rehearsing an upgrade and going back.
8. **browser/README.md:** a browser's client: the default HTTP mapping's
   browser end, `jig-client-shell-web`, over skein's browser parts
   (skein's `ui/web/README.md`), and the cookie that carries a browser's
   credential; the example guards and jig's attack stories, for the
   application's server; jig's test application's browser shell, server
   and page policy, as the example of an application's; and where jig's
   promise stops;
   **browser/application.md**, what an application writes for its browser client;
   **browser/loop.md**, the application's loop, elements, patches and scenes;
   **browser/sign-in.md**, signing in and out in a browser;
   **browser/serving.md**, the application's server, page policy and example guards.

## 3. Names

| Name | What it is |
|---|---|
| protocol layer | a process's layer between its root and io: its components, and the application's routing between them (programming-model.md, section 4) |
| component | a crate of a protocol layer that the application's protocol layer owns as a child and routes to |
| component, a shell's | the client's name (`../client/domain/README.md`, section 4): a shell's native implementation of a node (skein's `ui/README.md`, section 10), never a protocol layer's component |
| codec crate | a crate of records generated from a schema, used by components at both ends (skein's `data/codec/README.md`) |
| record | a value of a protocol's vocabulary, encoded to its schema |
| service | a process's `iterate` with its root and protocol layer; its `main` is its shell |
| worker's link | a worker's one channel with its engine, dialled and redialled (`channel/README.md`); the hub's link to the core, over a channel |
| worker's credential | what a worker's opening carries: its name, its secret, the wires it speaks (`channel/README.md`, 4.1) |
| wire | an encoding two builds exchange, its version frozen and its layouts named by fingerprint: jig's channel, smith's charter, turns and channel, and the application's own parts carried inside jig's records, each stated at the opening by its layout and its text (`channel/wires.md`) |
| credential grant | an LLM account's grant, or an application's credential, named by account and source (or the application's number for it) and generation, with how long it is valid; its value is beside it (section 8). An LLM account's is a credential with its scheme, `bearer` or `key`, or a program source's name, which carries no value (`llm.md`, section 6). Not the authority's grant (`../domain/README.md`, section 4) |
| table | a component's holding of secrets' values, by account, source and generation; jig's accounts' table, or the application's of its credentials |
| secrets' records | what the store keeps of secrets that change while the engine runs: minted credentials' and services' digests, LLM accounts' records and keys, and an application's own |
| jig's messages | what a client and the clients' component exchange, whatever carries them: a keyed request, a read, opening and closing a watch, signing in and out, and the stream's events; each names its owner and its version (`client/wire.md`, 2.1) |
| the clients' component | `jig-protocol-engine-clients`, the engine's side of jig's messages: it takes them with an opaque credential from the application's transport, and a `Mint` from its server; it mints, verifies, expires and revokes credentials; it knows no HTTP (`client/wire.md`, 2.2) |
| the transport | the application's carrier of jig's messages between `jig-client` and the clients' component: HTTP, a local socket, or an in-process channel |
| the default HTTP mapping | jig's optional transport over HTTP, both ends, `jig-client-protocol-http` and `jig-protocol-engine-clients-http`, with example guards (`client/default-http.md`; `browser/serving.md`, 1.2) |
| a credential | what signing in leaves: minted for an identity, its digest in the secrets' records, opaque bytes to the transport (`client/sign-in.md`). Neither a credential grant nor an application's credential |
| profile | a named set of every limit jig takes, consistent with one another (`limits.md`) |
| agent capability | the hub's agent: `smith-host-domain`, agent processes, one per run; or `smith-inline-agent`, smith's domains in the host's process (`../domain/hosts/README.md`, section 5) |
| smith's parent face | the vocabulary both agent capabilities speak toward their parent (`smith_host_domain::parent`; smith's `domain/host.md`, 9.4) |
| smith's values | smith's charter, transcript, turns' bodies and results as typed values, which smith's parent face carries and its agent capabilities take and give; the hub carries them as smith's bytes |
| an application's credential | a credential of the application's that a run's workspace needs (a token to fetch and push), named by the application's number for it and a generation; its value in the application's own table, from a source of its own (`channel/README.md`, section 7) |
| the reference binary | `jig-test`, jig's test application's `main`, which an application copies for its own (`service.md`, section 7) |

The domain's names (`../domain/README.md`, section 4) and the client's
(`../client/domain/README.md`, section 4) keep their meaning here; the
client's link is a client's reach of the engine, not a worker's link.

## 4. The processes

```
engine     the root (../domain/root.md) and its protocol layer:
           the store, workers' channels, LLM calls and accounts, host tools,
           clients (the application's server and transport, jig's clients' component), connectors' systems
worker     if any: the worker's root (jig-domain-host, an agent capability, a workspace) and its protocol layer:
           the link to the engine, smith's host half and its agent processes, the workspace's io
client     the application's presentation over jig-client (../client/domain/), and its transport:
           jig's messages to the engine and back, the small store's host, the shell
```

By `../domain/hosts/README.md`, section 3's shapes:

| Shape | The engine's protocol layer has | Workers |
|---|---|---|
| no agents | the store, clients, connectors | none |
| agents in the engine | and smith's LLM component for `smith-inline-agent`'s runs, accounts, host tools | none |
| agents on workers | and workers' channels, accounts, host tools | the link, smith's host half and agent processes, workspaces |

An application that mixes the last two has both.

- **The agent capability is smith's:** in the engine,
  `smith-inline-agent` with no workspace; on a worker, agent processes,
  or `smith-inline-agent` as its root's choice, for one person's machine,
  with no containment (smith's `domain/host.md`, 9.3).
- **One process,** for one person's machine, development and tests: the
  engine and its workers on one loop, each with its own root and
  protocol layer, the workers' links joined to the engine's listener by
  skein's in-process addresses (`needs.md`, section 1), over the same components,
  codecs and channels, with no TLS, and the workers' agents child
  processes, as between hosts (`service.md`, section 10). A client
  reaches the engine through the application's server, over TCP on
  loopback.

## 5. Components

Moved to [`components.md`](components.md), section 1.

## 6. What an application writes

| Part | In | Written |
|---|---|---|
| the protocol layer's routing: io's tokens, events to the root, outputs to components, values between components, timers, smith's payloads decoded and encoded with smith's codec functions | each process | copied from the reference service (`service.md`, section 7), around jig's service parts (`service-parts.md`, section 1) |
| `iterate` and `main` | each process | copied from the reference service and the reference binary, `jig-test` |
| an `agent` command, smith's agent shell, and smith's agent modes beside it (`--mcp-relay`, `--program-child`), if its workers run agent processes from its own binary | the binary | copied from the reference binary (`service.md`, section 7) |
| its configuration's own members, and their parsing | each process | its own |
| its connectors' protocols: their systems' clients, credentials, and webhooks, on a listener of each connector's own at an address its configuration names, or on its server's, routed by path (`decisions.md`, section 1) | the engine | its own |
| its server: the clients' listener, on jig's server part (`service-parts.md`, 1.4), its TLS, its routes and its URL space; its own pages, bundle and API; jig's clients' component, mounted under any prefix it chooses, since jig reserves no path; every HTTP and browser guard, jig's example guards as they are, wrapped, or its own | the engine | its own, after the reference service's (`service.md`, section 7; `browser/serving.md`, section 1) |
| its transport: jig's messages carried between `jig-client` and the clients' component, each with its credential: jig's default HTTP mapping, as it is or wrapped, or its own (a local socket, an in-process channel) | engine and client | jig's default mapping, or its own (`client/default-http.md`) |
| its sign-in: every way of signing in but the printed link and services' credentials (OAuth, a directory, a chat platform's identity), on its own routes, redirects included, with its settings and secrets; the identity it verified, handed to the clients' component in a `Mint`, from its server code alone; how the minted credential travels | the engine, the client | its own (`client/sign-in.md`) |
| its presentation: its routes and screens, words and look, confirmations and drafts, home and navigation, on `jig-client` and jig's helpers if they suit, with skein's UI kit, anything else, or nothing; or jig's reference UI, `jig-client-view`, as `ops` and jig's test application use it | the client | its own (`../client/domain/`) |
| its credentials for runs' workspaces: their tables, and their sources (a file, or an exchange it refreshes) on the engine | engine and worker | its own (`channel/README.md`, section 7) |
| its connectors' host tools: schemas, decoding, rendering | the engine | beside each tool, as `jig-protocol-tools` is for the core's (`../domain/hosts/charters.md`) |
| its parts' encodings: workspace items, left and saved work, delivery evidence, and the wires they are (`channel/wires.md`, 2) | engine and worker | its own codecs (`channel/owners.md`, 1.2) |
| its periods' source | the engine | a child of its root (`configuration.md`, section 5) |
| the worker's root, with its choice of agent capability, its workspace and its io | the worker | its own (`../domain/hosts/README.md`, section 6) |
| its store's families: keys' encoders, codecs, registered value types, current schemas, indexes and jobs | the engine | its own, by `store/README.md`, section 7 |
| its documents' and requests' wire, versioned per owner | engine and client | its own codec crate, generated as jig's is (`client/README.md`, section 8) |
| its templates: its own shapes of work, and jig's chat and goal kept, changed or dropped | the engine's configuration | its own data (`configuration-templates.md`, 1) |

It writes no codec of jig's records, none of the channel's rules, none of
the core's tools' decoding, no OAuth exchange or key's keeping of an LLM
account, no agent, none of `smith-inline-agent`'s runs' LLM calls, and
no credential's keeping: jig mints, verifies, expires and revokes them.

## 7. Versions

- **Every family keeps its version,** stored or sent alike, and a
  layout is named by its fingerprint (skein's `data/codec/README.md`,
  section 5). The store converts a
  value at read from the texts in its registry (`store/layouts.md`, 4).
  An added field takes a default, a renamed part its former name
  (`was`), and a change no plan derives its owner's written job
  (`store/jobs.md`, 4). An upgrade registers the build's layouts and runs
  its declared jobs in a new generation (`store/opening.md`, 2); it
  carries no map patch. Formats outside the registry are fixed at their
  current versions (`store/records.md`, 3): a change to one designs and
  brings, in the same landing, how a store at the floor reaches it.
- **Wires convert in the engine.** A wire, an encoding two builds
  exchange, changes its layout in its current text alone, with no old
  text and no patch. A worker states each wire's layout and sends its
  text at its opening; the engine converts what it receives by Read
  plans and what it sends by Keep plans, the worker speaking its own
  layouts alone (`channel/layouts.md`). A value the worker's layout
  can't keep is refused before it is sent (`channel/layouts.md`, 5).
- **The engine and its workers may be of two builds.** The store opens
  only at or above its floor and at the build's owner levels: one
  without the floor, below it or behind the levels is refused, naming
  `FLOOR_BRIDGE` (`store/opening.md`, 1.8).
  A non-empty store without a registry is refused too, never seeded.
  A worker whose layouts give the engine no plan, or whose
  `smith-pieces` layout is not the engine's, is refused at its channel's
  opening, by name (`channel/wires.md`, 3).
- **The channel** is a wire, its version frozen, its kinds' table in
  its schema; the application's payloads inside it and smith's charter,
  turns and pieces are wires beside it, which the worker's credential
  states (`channel/wires.md`, 2). smith's channel joins a worker to its
  own agent, one build, and is stated nowhere.
- **smith's charter, turns and results** carry smith's frozen versions
  (smith's `protocol/README.md`, section 8). The engine writes a charter
  at its own layout and converts it to the worker's, and reads a result
  or a turn the worker's agent wrote by a Read plan to its own
  (`channel/layouts.md`, 2.1).
- **What outlives a process** carries its version: the store's records,
  per family (`store/records.md`), and the client's small store
  (`../client/domain/client/small-store.md`, section 1).
- **jig's messages are the exception:** they are versioned per owner,
  jig's by jig and the application's by the application, each message
  naming its owner and its version, and the engine serves one layout
  of each owner, at its one frozen version, so a client that does not
  come from the engine may lag it, its layout judged at the hello
  (`../client/domain/contract/README.md`, section 6).
  - *A version not served* is answered with a typed refusal, *version
    not served*, naming the owner, never a bare failure of the
    transport (`client/versions.md`, section 1).
  - *One jig pin:* an application takes `jig-client` and the engine's
    components from one revision of jig. A change to jig's operations
    moves its wire's layout, which the stream's hello names and the
    client judges (`client/versions.md`, section 1).
  - *How jig itself is released and pinned* is open (`../README.md`,
    section 14).
- **Configuration** has no version: unknown fields refuse the start
  (`configuration.md`, section 8).

## 8. Secrets

| Secret | Held by | Kept | Reaches |
|---|---|---|---|
| an LLM account's signed-in source's record: its tokens and their expiry, the client id its provider issued, the scopes granted, the ID token | skein's accounts driver; the access tokens lent, `jig-protocol-accounts`' table | the store's secrets' records, by the source's number, before the access token is lent (`llm.md`, section 5; `store/secrets.md`) | the refresh token, its provider alone; the access token, with its scheme, `smith-inline-agent`'s runs in the engine through smith's LLM component, and workers through credential grants, then their agent processes through smith's host half, or smith's LLM component on a worker that composes `smith-inline-agent` (`llm.md`, section 6) |
| an LLM account's key | `jig-protocol-accounts`' table | the store's secrets' records, at a drawn carried number before the core sees the request, then adopted into its source's row before it is granted; that request is never kept by the client (`client/README.md`, section 3; `store/secrets.md`) | as an access token, with its scheme |
| a program source's login | the program, in its configuration directory on each host; never jig | the host's, never the engine's (`llm.md`, 5.5) | the program alone; a grant carries the source's name |
| a worker's secret | the worker; its digest on the engine | configuration's files | the opening of the worker's link, over TLS, loopback, or an in-process address (`channel/README.md`, 4.1) |
| a credential, a sign-in's | `jig-client`, where its transport needs it in hand (a bearer); or a browser, as a cookie the application's server sets and the client never sees; its digest on the engine | the store's secrets' records | the application's server once, in `Minted`, where it asked for it; then each of the party's messages, verified by the clients' component (`../domain/people/README.md`, section 3) |
| a service's credential | the service; its digest on the engine | the store's secrets' records | its sign-in only, which answers a sign-in's credential |
| the printed link's secret | the clients' component, which gives it; the application's startup, which prints it in a URL of its own | the component's memory only, until spent or lapsed: a restart gives a fresh one | the client that signs in with it, once, on loopback: a browser that opens the link, or a native client |
| the workers' listener's TLS key | the engine | configuration's files | the workers' TLS |
| the clients' listener's TLS key; the secrets of the application's sign-in | the application's server | its own configuration's files | its exchanges |
| a connector's credentials | the application's protocol layer | its files, or its own records in the secrets' store | its system |
| an application's credential, for a run's workspace | the application's protocol layer: its table on the engine, and on each worker that holds a grant of it | its own source's: a file, or an exchange it refreshes | the workspace's systems, through credential grants with the assignment (`channel/README.md`, section 7) |

- **Names in the domain, values beside it.** A domain names a secret by
  number and generation (`../domain/engine/decisions.md`, section 1.4, and `../domain/engine/README.md`, section 6); no
  domain state, fact, record of the main store or log holds a value.
  Values move between components through the protocol layer's routing,
  never through a root.
- **Kept before used.** An account's record is durable in the secrets'
  records before the access token it bought is lent, by skein's driver
  and its owner's-store keeper; a key is durable there before the core
  sees the request that carried it. A sign-in is committed
  first, which gives it its number; its credential's digest is then
  kept under that number; and only then is the credential answered, to
  the client that signed in or to the application's server that asked
  for it. A crash between them leaves at worst a record nobody uses, or
  a sign-in no client holds a credential for: never a name with two
  values, or a credential the engine does not know.
- **Across hosts, only over TLS,** in plaintext over loopback, or over a
  joined socket in one process, and validity as durations, never times.

## 9. The store

- **jig's store is skein's key-value store in the engine** (skein's
  `data/kv.md`): embedded in the engine's process, with transcripts beside it
  and secrets in a second store. There is no store process and no wire:
  the store is a component of the engine's protocol layer, and its bytes
  are its records on disk. jig's part is the engine's own records (its
  families, their keys and encoding, versions, commits and loads,
  transcripts, secrets, opening, growth and retention); the application
  adds its families beside jig's, in the same commits.
- **Its design is `store/README.md`.** What the rest of the protocol layer
  relies on is this:
  - **commits** numbered as the journal numbers them, applied in order,
    each answered once durable, which makes every earlier one durable; a
    failed commit, or one the store refuses, answered failed, after which
    nothing is answered and the engine stops (`../domain/engine/decisions.md`, 1.1
    and 1.5);
  - **loads** by the domain's ranges, in bounded pages, seeing every
    commit answered before they were asked;
  - **transcripts** appended with the commit that accepts their turns,
    each turn kept with whether it opens a window, and read from either
    end, or from the turn that opens a task's current window
    (`../domain/engine/executors-transcripts.md`, 1);
  - **secrets' records** in a store of their own, in a directory only the
    engine's user may read, written by components alone, durable on
    request before what they protect is used, read at start;
  - **opening** before the root starts, refusing a store of another
    format, or one whose records disagree with its own header.

## 10. What jig needs from skein

Moved to [`needs.md`](needs.md), section 1.

## 11. What jig needs from smith

Moved to [`needs.md`](needs.md), section 2.

## 12. What the domain owes

What the protocol layer relies on from jig's domain is designed there:

- **the engine stopping:** `../domain/engine/restart.md`, 5;
- **credential grants pushed and resent, and LLM accounts:**
  `../domain/engine/fleet.md` and `../domain/engine/README.md`, section 6; `../domain/accounts.md`;
  `llm.md`, section 5;
- **the hub's link vocabulary, whole, and smith's payloads and facts by
  hand-out:** `../domain/hosts/hub.md`, 1 and 8;
- **what smith gives** (`needs.md`, section 2): windows, and resuming from the
  current one (`../domain/engine/executors-transcripts.md`, 1); the start's messages
  (`../domain/engine/executors.md`, 1; `../domain/hosts/README.md`, section 7), and
  messages refused or unread kept in the inbox (`../domain/tasks/messages.md`,
  2 and 3); refusals carried with turns (`../domain/hosts/hub.md`, 4);
  the hello's agent bounds checked (`../domain/engine/fleet.md`);
  no run placed that smith would refuse for its budget
  (`../domain/authority-budgets.md`, section 1);
- **an application's credentials:** `../domain/engine/README.md`, 6.1;
- **the client's API:** `../domain/people/README.md`, section 3, and `../domain/people/acts.md`, section 1, and `../domain/people/README.md`, section 5;
  `../domain/tasks/README.md`, sections 3 and 4 and `../domain/tasks/changes.md` and `../domain/tasks/README.md`, section 6; `../domain/engine/views.md`
  ; what the domain owes the client, item by item,
  `../client/domain/contract/README.md`, section 7.

What the store asks of the domain is `store/README.md`, section 10.

## 13. Decisions this design takes

Moved to [`decisions.md`](decisions.md), section 1.

## 14. Open questions

- **Browser-approved sign-ins between hosts** (13), and a person's
  sign-ins listed and each ended: `client/README.md`, section 12. Approval's
  transport and startup remain confined to loopback.

- **A sign-in's redirect relayed by a client,** for a browser on
  another machine than the engine's (`llm.md`, section 10).
- **Upgrades:** in place, across a store's format: records translated
  between versions, and rolling upgrades of the engine and its workers
  (section 7).
