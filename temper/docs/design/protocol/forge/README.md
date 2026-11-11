# The forge's protocol layer

Provisional, 2026-10-03, revised 2026-11-02. How the engine reaches a
forge, on skein's forge kit: what temper's forge component, its setup's
host and its forge connector do with it, and what the kit leaves to
temper. The kit is skein's (`forge/README.md`, `skein/docs/design/forge/connection.md`,
`skein/docs/design/forge/client.md`, `skein/docs/design/forge/setup/README.md`, `skein/docs/design/forge/github.md`,
`skein/docs/design/testing/fakes/forge.md`): Forgejo's API and GitHub's over HTTP and JSON, the
keys' markers written at the head of what the engine creates, the
webhooks Forgejo sends back to a listener on loopback and the deliveries
a GitHub App's log keeps, the client, the component, the setup and the
fake forge. A forge's credentials are records in jig's secrets' store,
loaded at start and handed over at runtime (`engine.md`; ../credentials.md,
the forges' records). The overview is ../components.md and ../README.md, section 4); the
vocabulary carried is skein's (`forge/README.md`, section 3), and the forge
client's use of it is `../../domain/forge/README.md`'s (`../../domain/forge/keeping-up.md` and `../../domain/forge/README.md`, section 4); the
machines are skein's (skein's `http.md`, `skein/docs/design/data/json.md`, `skein/docs/design/net/tls.md`). What
others owe this layer is in `README.md`, section 10, what temper relies on of each
forge in `README.md`, section 11, and what is open in `README.md`, section 12.

## 1. In one page

- **The component, the setup's host and the client are skein's, owned
  by temper.** `skein-forge-connection`, the forge component, and
  `skein-forge-setup`'s host run as children of the engine's protocol
  layer (`engine.md`, 6). `skein-forge-client`, the forge client, is owned by
  temper's forge connector (`temper-engine-domain-forge`). Each
  dialect's documents and call plans, Forgejo's and GitHub's, the keys'
  markers, the deliveries, the setup's plans and the fakes are skein's
  too (skein's `forge/README.md`, section 2; `skein/docs/design/forge/github.md`). This document
  holds what the kit leaves to its owner.
- **Translation only, and no retries.** Each call of the forge client (a
  `Read` or a `Write`) is answered by exactly one `Answer` or `Error`,
  made of one or a few HTTP exchanges on one connection. A call goes out
  once. Its answer says how many requests it cost, so the client's
  budget charges what the forge actually served (skein's
  `forge/connection.md`, section 1; `skein/docs/design/forge/client.md`, section 6).
- **temper's routing hands the component every value** (`engine.md`): the
  component reads no file. The values are each forge's records', in
  temper's store and in jig's secrets' store, and the configuration's
  machine and limits, with two of temper's own: the word `temper` and
  the user agent `temper/0.0.0`.
- **A forge is a record, made at runtime and removable** (`engine.md`, 1 to 3).
  The forge connector keeps each forge in temper's store: its kind,
  mode, place, git program, bot and a first set up's intent. `SetUp` and `Connect` make it, numbered next and never
  reused; no forge is configured. Its removal ends it at once: no call
  served, its access dropped, its secrets erased, and the box asked to
  remove its group (`engine.md`, 8).
- **Repositories by the forge's own id; paged, bounded, and decoded as
  it streams.** Forgejo's conventions are skein's (`forge/forgejo.md`, section
  1); the paths and the users the component keeps are
  `skein/docs/design/forge/connection.md`'s (`skein/docs/design/forge/connection.md`, section 4). Adoption first locates a
  repository by its owner and name (skein's `forge/forgejo.md`, 1.2).
- **The engine's marks are keys' markers only** (section 3). A creation
  carries its key at the head of its body. The word is `temper`, and the
  key holds the deployment's id as its namespace. The marker's form is
  skein's (`forge/README.md`, section 5). Nothing else of the engine's is
  written into the forge.
- **A forge starts connected or not, and stops alone.** Its access (an
  API token, or a GitHub App's id and key, and the user it must stand
  for) is built from the forge's records at start (`engine.md`, 1), or handed over
  by `Request::Connect` once a setup's run has kept them (`engine.md`, 2). A forge
  with none answers each call `NotConnected`. A forge whose credential
  is refused is stopped and named to the root, and the engine and every
  other forge run on; a forge removed is ended (`engine.md`, 3). The credential is the component's alone:
  never granted to a worker, never in the main store, and the first
  exchange with the forge checks that it stands for the forge's user
  (skein's `forge/connection.md`, section 3; ../credentials.md, the forges'
  records).
- **GitHub's installations and git's tokens come up as events** (`engine.md`, 4, 5
  ): the routing keeps the accepted installations in the forge's
  records, and makes each token the component mints a new generation of
  the forge's git identity, its value kept before jig's core hears of
  it.
- **Webhooks for latency, polling for liveness** (section 4). Forgejo's
  deliveries reach a listener of the component's own, on loopback, at
  `application.webhooks`'s address: verified as they stream, answered
  first, each hint naming what caused it (skein's `forge/README.md`, section 6;
  `skein/docs/design/forge/connection.md`, section 6). Polling stands as it is and finds
  what a delivery lost, refused or never sent told of; a forge on
  another host polls alone, since deliveries over TLS need skein's TLS
  server side (../setup/README.md, section 9). A GitHub forge's deliveries are
  read from its App's log every `log_poll` instead, with no secret and
  no guid kept across process starts (4.1).
- **The vocabulary carries what a forge connected at runtime answers**
  (`engine.md`, 7): `NotConnected` and `NotInstalled` at once, at cost 0;
  `RateLimited { after }`; the rate each answer reports; validators,
  whose `304`s are free.
- **The load rules' values are temper's** (section 5): the rules are
  kept in skein's crates, and `../../domain/forge/README.md` holds the values.
- **The fake forge, the throwaway and the conformance are skein's**
  (`skein/docs/design/testing/fakes/forge.md`). temper's worlds run temper's stories over them and
  count its costs (section 9).
- **What temper relies on of each forge is recorded.** Forgejo's by
  skein's conformance against a real Forgejo v17.0, each behaviour a
  named fact (skein's `forge/forgejo.md`, section 2). temper requires that
  release: one patch release of it is pinned (skein's `forge/README.md`, 4.3),
  and no other version is served. GitHub's are skein's `forge/github.md`
  facts, which temper's tests run against skein's fake alone (`README.md`, section 11
  ).

## 2. Forgejo's API as temper uses it

Held by skein's `forge/forgejo.md`, section 1, and `skein/docs/design/forge/connection.md`, sections 3 to 5
.

### 2.1 Conventions

Held by skein's `forge/forgejo.md`, section 1.1.

### 2.2 Each operation

Held by skein's `forge/forgejo.md`, section 1.2.

### 2.3 Decoding answers

Held by skein's `forge/forgejo.md`, section 1.3.

### 2.4 Failures

Held by skein's `forge/forgejo.md`, section 1.4.

### 2.5 What the protocol layer keeps

Held by skein's `forge/connection.md`: each forge's start and its access
in skein's `forge/connection.md`, section 3, the repositories, the users and a GitHub App's
installations in skein's `forge/connection.md`, section 4, the credentials and the installation tokens
in skein's `forge/connection.md`, section 5, and a GitHub App's delivery log in skein's `forge/connection.md`, 6.5. What temper keeps
of a forge is its records (`engine.md`, 1), never the component's state.

### 2.6 GitHub's API as temper uses it

Held by skein's `forge/github.md`: the dialect's paths, headers, paging,
validators, pauses and failures, each a named fact (skein's `forge/github.md`, section 13). Of it,
temper sets one thing, where a job log may be read from:

- **A job log's redirect** is followed only to an origin its forge's
  kind names, resolved at start: at most `forges.log_origins` entries,
  each an origin, `https`, or `http` to a loopback host alone, matched
  as skein's component matches a redirect's (scheme, host in ASCII
  case, port; skein's `forge/connection.md`, 4.5). GitHub's names the
  twenty results hosts `productionresultssa0.blob.core.windows.net`
  through `productionresultssa19.blob.core.windows.net`, over TLS on
  port 443, when the machine names no GitHub of its own; a GitHub the
  machine names (../configuration/engine.md) and a Forgejo name none.
  The shell resolves them before serving, with no network probe. An
  unresolved host is skipped and named once in the log, and never
  refuses startup. If none resolves, or the kind names none, no job log
  is fetched; a check's failure is then shown without its log.
- **A redirect to any other origin** is answered `Refused`, and the
  component's `Fact::Origin` names that origin once for each connect
  (`engine.md`, 4). The routing keeps it per forge in memory, at most
  `forge_component.log_origins` of them, and forgets them at the forge's
  next connect. The latest distinct origin appears in that forge's row
  diagnostic (../../client/forge/README.md, the deployment's page), when no connect
  failure occupies it. The routing logs each of the first
  `forge_component.log_origins` distinct refused origins once per start,
  retaining that bounded log memory across reconnects. The log contains
  only scheme, host and port, never a signed path, query or
  credentials; no member adds an origin (section 12).
- **No document, example or test of temper's calls github.com:** GitHub's
  addresses are the machine's GitHub place, github.com's when it names
  none (../configuration/engine.md), and every test names skein's fake
  there (`README.md`, section 9).

## 3. The engine's marks

What the engine writes inside what it creates: keys' markers, and
nothing else. No record, outcome or page of the engine's is written into
the forge, and no call of skein's `forge/forgejo.md`, 1.2 writes a label: labels
are for display only, and no decision reads one (`../../domain/forge/adoption.md`
).

- **A creation's key,** at the head of an issue's body, a comment or a
  review, is a marker: its form, its decoding and its bounds are
  skein's (`forge/README.md`, section 5). temper's word is `temper`, so a
  marker reads `<!-- temper:key <the key in lowercase hexadecimal> -->`
  and a line feed, then the text. An HTML comment, which Forgejo's
  rendering hides.
- **The key is the client's** (`effect_key`, skein's `forge/client.md`,
  section 7): version 1, the namespace's length as two bytes
  big-endian, the namespace, then the effect's identity, opaque bytes
  to the client. temper's namespace is the one the root derives from the
  deployment's id; its identity is its effect's purpose and its numbers,
  which the connector's top encodes (`../../domain/forge/README.md`, section 4).
  So the marker carries the deployment's id, and deployments that share
  a repository tell their creations apart (jig's `domain/connectors.md`,
  4.4). The key's first byte is its version; the marker has none of its
  own.
- **Decoded** as skein's `forge/README.md`, section 5 says; a body whose marker
  carries another word has no key. Whether to believe a key is the
  client's: its version, its namespace, and whether its author is the
  forge's writer (skein's `forge/client.md`, section 7). A person who
  copies a marker into their own comment gives it no provenance.
- **Bounded.** The key is within the client's `op_bytes`, the marker
  twice its length and 21 bytes more (the word's 6 and 15), and the body
  sent the marker and the text, which `body_bytes` holds (skein's
  `forge/connection.md`, section 7).
- **The worlds** write the same marker (the forge connector world's
  translation, `tests/engine/forge/client/src/translate.rs`); the
  protocol world (section 9) runs the component's own encoder and
  decoder.

## 4. Webhooks

Forgejo's deliveries, and the deliveries a GitHub App's log keeps (4.1),
made hints: they cut polling's latency, and decide nothing. Polling
stands as it is with no webhook (section 5; `../../domain/forge/keeping-up.md`, 2):
its `poll`, `poll_max`, `hinted` and `slow` are unchanged, and it finds
whatever a delivery lost, refused or never sent told of. A Forgejo
delivery reaches the engine on loopback; a forge on another host polls
alone (../setup/README.md, section 9).

- **A delivery's signature, its events and its hints** are skein's:
  how Forgejo signs a body, the events a hook subscribes to, the hint
  each makes, who caused it, and the bounds on a delivery (skein's
  `forge/README.md`, section 6).
- **The listener,** its connections, the verification as a body streams,
  the answer first, each refusal and the bounds are skein's (skein's
  `forge/connection.md`, section 6).
- **The webhooks' listener's address,** `application.webhooks`,
  `{"listener": address}` (../configuration/engine.md): a socket
  address on loopback (in `127.0.0.0/8`, or `::1`), apart from the
  clients' listener (`../../client/engine.md`). The routing
  hands it to the component, which takes `POST` on one path per forge,
  `/hooks/<number>`, the forge's `number`.
- **Each Forgejo forge's secret** is its `WebhookSecret` record
  (../credentials.md, the forges' records): drawn and kept by a setup plan
  on a managed or owned forge served the listener (../setup/reconcile.md, 3),
  pasted with the API and git tokens on a connected one. The routing
  hands it in the forge's access (`engine.md`, 1) only with `application.webhooks`,
  and again when a plan draws it on a connected forge (../setup/reconcile.md, 4); a
  forge with a secret and no listener is
  handed none, polls alone, and is named once in the log. A forge with
  no secret has no path (skein's `forge/connection.md`, 6.1).
  `WebhookSecret` is Forgejo's alone: a GitHub forge has none (4.1).
- **What the domain does with a hint.** It drops its own writes'
  echoes: a creation's key for an item, the pusher for a push
  (`../../domain/forge/keeping-up.md`, 2; skein's `forge/client.md`, section 4). A
  hint for a repository the deployment has not adopted goes up all the
  same; the domain drops it.
- **What has no delivery** is found by polling alone
  (`../../domain/forge/keeping-up.md`, 2): a commit status, which Forgejo never
  delivers, so CI that a system other than Actions reports and the
  verdicts temper posts.
- **What Forgejo settles after it delivers** (a pull request's
  mergeability, a job's commit status: section 11) is the domain's to
  read again, within `settle` of the move (`../../domain/forge/keeping-up.md`, 2);
  the component hints at once.
- **A listener that fails** while the engine runs stops the engine, as
  the clients' listener's does: the service drains as at a shutdown, and
  whatever starts `temper-box serve` again (the box, a service manager)
  starts it with a listener again.
- **Not sent again.** Forgejo does not resend a failed delivery (skein's
  `forge/README.md`, section 6): one lost while `temper-box serve` is down, refused
  for its size or its signature, or never sent is found by polling. A
  hint that arrives before the engine's domain is ready is dropped, and
  the keep's first listing finds what it told of.
- **Hooks are set up by reconcile, or by the person,** at
  `http://<listener>/hooks/<number>`, with the forge's secret, the
  events of skein's `forge/README.md`, section 6, JSON and active (../setup/README.md, section 4 and ../setup/reconcile.md and ../setup/README.md, section 6
  ; skein's `forge/setup/forgejo.md`, section 1.3, step 6).
  A repository with no hook polls (`../../domain/forge/adoption.md`).

### 4.1 A GitHub App's delivery log

GitHub's deliveries are read, not received: the component's puller
reads the App's log (skein's `forge/connection.md`, 6.5;
`skein/docs/design/forge/github.md`). The engine takes no delivery from GitHub, and needs
no address GitHub can reach.

- **The App's hook** points at no listener: its address is
  `https://<slug>.invalid/`, a name that never resolves (RFC 2606), and
  it is active, so GitHub records each delivery it fails to make in the
  App's log. The manifest sets it, and `Paste` (../setup/README.md, connecting
  GitHub); GitHub's `Reconcile { forge }` repairs that address alone
  when it differs (`engine.md`, 6).
- **No secret.** A GitHub forge has no `WebhookSecret` record: the
  conversion's webhook secret is never kept, `Paste` draws none, and
  the routing hands the access `webhook_secret: None`. The forge has no
  path on the webhooks' listener, and the log's reads are vouched for by
  TLS and the App's JWT (skein's `forge/connection.md`, 3.5 and 6.1;
  `skein/docs/design/forge/setup/README.md`, the conversion).
- **Each `log_poll`** (5 s in the profile `small`), while the forge is
  connected, the puller reads the log's first page conditionally, with
  the validator the last first page answered: a `304` is free (`engine.md`, 7) and
  ends the poll. Otherwise it reads back, newest first, to the newest
  delivery it read, within `log_pages` (skein's `forge/connection.md`,
  6.5). The reads are counted in no call's cost.
- **No guid is kept across process starts.** The newest delivery's
  guid is the component's, in memory: keeping it would write the store
  at each delivery, and hints are only hints. At a process's first
  start of a forge, the component's first poll keeps the newest guid
  and reads nothing back. What changed while temper was down is read by
  the forge client at its restart, which reads each adopted repository
  afresh (`ReadAfresh`; skein's `forge/connection.md`, 6.5, and
  `skein/docs/design/forge/client.md`, section 8; `../../domain/forge/README.md`, section 9).
- **A later start of the forge within one process,** a connect after a
  stop, reads back to the guid the component kept, as far as
  `log_catch_up` and `log_pages` allow (skein's `forge/connection.md`,
  6.5).
- **A gap,** a poll ending on `log_catch_up` or `log_pages` before it
  meets the guid kept, is said once as `Fact::Gap`, which the routing
  logs and nothing else (`engine.md`, 4), and the installations are read again; the
  client's polling finds the rest.
- **An installation's rows** read the installations again, and come up
  as `Event::Installation` (`engine.md`, 4); a row that hints is read whole within
  `delivery_bytes` and goes up as a Forgejo delivery's hint does, the
  domain dropping its own echoes and what it has not adopted, as above.
  Any other row is passed over, unread.
- **A `401`** on the log stops the forge, its key refused (`engine.md`, 3); any
  other refusal ends that poll, and the next asks again.

## 5. The load rules, and where each is kept

A rule kept in skein's crates points to the section that keeps it. The
values are temper's: `../../domain/forge/keeping-up.md`, 2 and 4.

| Rule | Kept by |
|---|---|
| No listing without a filter on a periodic path; no history | the client's one keep-up listing per repository, always `since` (skein's `forge/client.md`, section 4; the listing, skein's `forge/forgejo.md`, 1.2); the slow pass reads live resources one at a time (`skein/docs/design/forge/client.md`, section 4) |
| Calls per pass do not grow with history | one listing per pass, nothing listed by label (skein's `forge/client.md`, section 4); checked by temper's cost checks (10) |
| Cache by change token | comments and reviews by revision (skein's `forge/forgejo.md`, 1.3); repositories' paths and users cached (`skein/docs/design/forge/connection.md`, section 4); a pull request read on a backoff (`skein/docs/design/forge/client.md`, section 4); a read given a validator is conditional, and its `304` is free (`engine.md`, 7), as each poll of a GitHub App's log (4.1) |
| One path to CI; never a closed pull request's | statuses of the head only (skein's `forge/forgejo.md`, 1.2); a closed item leaves the working set when its owner stops naming it (`skein/docs/design/forge/client.md`, section 3; temper's top, `../../domain/forge/keeping-up.md`, 1) |
| API only; `page` and `limit` always; the forge's cap | skein's `forge/forgejo.md`, 1.1 |
| Contents only when a decision reads them | a pull request's files and a comparison, read for a brief or an overlap, each file's reads charged (skein's `forge/forgejo.md`, 1.2; `skein/docs/design/forge/client.md`, section 6) |
| Webhooks: hints, answered first, merged, targeted, echoes dropped | skein's `forge/README.md`, section 6 and `skein/docs/design/forge/connection.md`, section 6; the client merges hints into one earlier pass (`skein/docs/design/forge/client.md`, section 4); temper's domain drops its own echoes, by key and by pusher (`../../domain/forge/keeping-up.md`, 2); a GitHub App's log read every `log_poll`, conditionally (4.1) |
| Closed items by deltas, not sweeps | the keep-up listing's `since`: a closed item is listed only when it changes, and its owner stops naming it (skein's `forge/client.md`, sections 3 and 4) |
| Backoff and jitter; honour the rate; no blind retries of writes | the client's backoff and budget (skein's `forge/client.md`, sections 4 and 6); `RateLimited { after }`, Forgejo's `429` and GitHub's pauses alike, and no retry in the component (skein's `forge/forgejo.md`, 1.4, and `skein/docs/design/forge/github.md`); each answer's `rate`, the least of which the budget keeps (`engine.md`, 7); keyed creations found again after a `Timeout` (`skein/docs/design/forge/client.md`, section 7) |
| Slow cadences; webhooks for latency, polling for liveness | the client's `poll`, `hinted` and `slow`, unchanged by webhooks (skein's `forge/client.md`, section 4); a forge with no hook, or on another host, polls alone (section 4); a GitHub forge polls as a Forgejo one does, its log's hints only cutting the latency (4.1) |

Nothing the engine writes changes often: it writes the forge only to
make an effect (`../../domain/forge/README.md`, section 4), never as a heartbeat.

## 6. Entities and entrance limits

Held by skein's `forge/connection.md`, section 7. temper's values are
its `limits` member (../configuration/limits.md), set in
`temper-service`'s profile `small`. By name, those this layer adds:

- **The forges kept:** `forge.forges`, the forges kept at most, of
  every kind and mode, an ended one counted until its removal is whole
  (`engine.md`, 1, 3); a forge made past it is refused busy (`engine.md`, 2). It sizes the
  component's forges, the setup host's, the identities' table and the
  routing's per-forge state, and is at most the accounts child's
  `credentials` in `engine.limits` and in the worker's `limits`, since
  each kept forge has a git identity. Counts, not a list: no forge is
  configured, and `forge.repositories` and its kin are the
  deployment's, shared by every kept forge, never a sum over them
  (../configuration/limits.md).
- **temper's own:** `records.value_bytes`, the largest record or carried
  value, checked against jig's secrets' store at start (`engine.md`, 1);
  `forges.installations`, the installations a GitHub forge accepts, at
  most the component's `installations` (`engine.md`, 4); `forges.log_origins`, at
  most 32 hosts per forge, default 20 (2.6).
- **The component's, for GitHub** (skein's `forge/connection.md`, 7.2):
  `installations`, `log_origins`, `key_bytes`, `renew_before` (10 min;
  `engine.md`, 5), `log_poll` (5 s), `log_catch_up` (24 h), `log_pages` (10) and
  `delivery_bytes` (4.1). The component has no defaults: each is
  temper's, named in its limits module and given once at
  `Component::new` for every forge, none per forge. `log_catch_up`
  bounds only a later start of a forge
  within one process.
- **The setup's host:** `setup.states`, `setup.state_life_ms` (1 h) and
  `setup.runs` (`engine.md`, 6); its `Reachable` plan's `reachable` is the
  picker's `picker.page`.
- **The box's link** (`engine.md`, 8): `box.line_bytes`, its one line read and
  its one line written, one ask, and one start and one removal held.
- **Beside this layer,** the clients' listener's `routes.query_bytes`
  and the picker's `picker.page` (100, GitHub's page), which bound the
  GitHub routes and the picker's reads (../../client/README.md, the routes and
  the picker; `../../domain/forge/adoption.md`, 3).

The domain's `temper_engine_domain_forge::valid` checks the entrance
constraints of `../../domain/forge/adoption.md` and `../../domain/forge/change/README.md` and `../../domain/forge/README.md`, section 9, and count
representability: the forges kept, reconciliation rows and landing
capacities.
Names and gates fit the panel, and the client's admissions cover the
top's rows. The connection owner's forge-by-installation row count must
be representable. Internal output envelopes and heap sums do not
constrain admission.

## 7. State machines

Held by skein's `forge/connection.md`, section 8: a forge connection
(7.1), an installation's tokens (8.6) and the delivery log's puller
(8.7) among them; the setup's runs are skein's `forge/setup/README.md`'s.

### 7.1 A forge's connection, as the routing tells the root

| State | Entered on | Its calls |
|---|---|---|
| not connected | start with no access, an incomplete, unreadable or mismatched set (`engine.md`, 1); a setup run's `Stop` on a forge with none | `NotConnected` |
| not set up | start, an owned forge with neither its handover file nor an `AdminToken` record (../setup/owned.md, 5) | `NotConnected` |
| setting up | the box's link's `start` written, for an owned forge's `SetUp` or the playground's fake forge, then each `step` line, with its step (`engine.md`, 8) | `NotConnected` |
| set up failed | the box's `failed`, or its pipe closed during the ask (`engine.md`, 8); a managed forge's start whose plan stopped (../setup/reconcile.md, 1) | `NotConnected` |
| connecting | a run that connects begins (`engine.md`, 6) | `NotConnected` |
| connected | `Event::Connected` (`engine.md`, 2) | served |
| stopped | `Event::Stopped` (`engine.md`, 3) | `NotConnected` |
| ended | its removal's decision, its owners' `RemoveForge` or the playground's reset, or a start whose record reads ended (`engine.md`, 3) | `NotConnected`, by the routing, which passes none on to the connection component |

A connecting forge goes back to what it was when its run stops; a
stopped one is connected again by its next `Connected`, after a Replace
or, on a connected Forgejo, a `Connect` with its tokens.
A setting up forge, at the box's `started`, connects as `engine.md`, 2 says; set up
failed, it is asked again from the client, as not set up is, and a
managed one is connected again as its admin (../setup/reconcile.md, 5). Only a
forge the box runs, owned or fake, is ever setting up, only one of those
or a managed one set up failed, and only an owned one not set up. An
ended forge stays ended until its
removal is whole and its record erased (`engine.md`, 3): the routing's events for
it change nothing, the setup host runs nothing for it, a webhook's hint
for it is dropped, and no list shows it (`../../domain/forge/adoption.md`, 1).
The state is the routing's and the root's, in memory, and is rebuilt at
each start from the records and the events (`../../domain/forge/adoption.md`,
1).

## 8. The fake forge's protocol layer

Held by skein's `testing/fakes/forge.md`, section 4, for Forgejo's dialect, and
skein's `testing/fakes/github.md`, section 1, its GitHub layer, for GitHub's (with skein's
`forge/github.md`): its App, installations, installation tokens, merge
queue, delivery log, job log redirect, rate windows and pauses, its
fixed test keys, and its two browser pages answered as redirects, so
the setup's routes run end to end with no browser.

temper-fake-git's counted ids use skein's `testing/fakes/forge.md`, section 3's
shared 20-byte form: `mix(count)` big-endian (the SplitMix64 finalizer),
the count big-endian, then four zero bytes, written as forty lowercase
hex digits. Its encoder is `skein-fake-forge-domain::object_bytes`; its
count reader uses `skein-fake-forge-forgejo::routes::commit`, which
validates the whole form. An absent previous head in a fetch keeps git's
all-zero marker; count zero has its own mixed id. Stored little-endian
counts are not ids.

## 9. Testing

The tiers, the fakes and the layout are `../../testing/README.md`'s (`../../testing/README.md`, section 2, and `../../testing/fakes.md`, section 2, and `../../testing/README.md`, section 5
). The kit's own tests are skein's. The forge's, in temper:

- **Step tests** of the documents, the encoders and decoders, the call
  plans and the marker: skein's `forge/README.md`, section 9.
- **The machine world** and the protocol world of the component and the
  fake forge: skein's `forge/connection.md`, section 10.
- **temper's protocol world,** `tests/engine/protocol-forge`: temper's
  forge component, which is `skein-forge-connection` handed temper's
  values, and the fake forge's protocol layer, skein's, joined by
  in-memory streams cut at random, running the forge connector world's
  stories (`tests/engine/forge`, `../../domain/forge/README.md`, section 10)
  through bytes: a change produced, opened, checked, queued and landed;
  a creation and a merge whose answers were lost, found after a restart
  and never made twice; a failed head repaired and checked again; a held
  branch moved by another hand, reported as drift; on GitHub, a change
  waiting out 90 seconds of CI without delivery hints, landing within
  the pull keep's maximum poll after its verdict, with request costs
  bounded by the keep's paces and App log reads counted apart; on
  Forgejo, a change whose Actions CI stays pending for ten minutes, with
  a job's delivery every 20 seconds and with none, its requests, its
  head's status reads and its pull reads counted in every minute past
  its first two against `../../domain/forge/keeping-up.md`, 2's pace, and
  landing on its verdict. Its
  invariants: what the engine's domain asked is what the fake's domain
  did; each call is answered once, however its streams are cut; no credential reaches any
  record but its own in jig's secrets' store, nor any request, fact,
  answer or log line. The fake's deliveries reach the engine's listener
  signed, late, out of order or never, so its stories run with hints and
  without them; on GitHub's dialect they reach the App's log, read by
  the puller, with no listener.
- **temper's routing,** in `temper-service`'s own tests, not in the
  protocol world, which runs the component without the service: the
  store on skein-kv's in-memory mode, the component's and the setup
  host's answers and events played to the routing, each forge's
  connection as `engine.md` has it:
  - **at start:** a forge with no rows, an incomplete set, an unreadable
    row and a set made for another forge each start not connected,
    named, nothing erased, while every other forge serves; carried rows
    found at start are erased on the core's decision;
  - **connecting at runtime:** a run's writes kept before
    `Request::Connect`, the access built from the records, `Connected`
    told to the root with the identity, and calls `NotConnected` at cost
    0 until then; a connected Forgejo's `Check` with its pasted tokens,
    its `User` write, the tokens and the secret then the `Origin` kept,
    its login the record's bot's before `Request::Connect`, and
    `Connected` naming it; tokens of two users, of a site admin, or of
    another user than the kept one, each stopping it with nothing kept;
    a `Replace` of the git token alone, checked with the kept API token
    and kept over the old one; and on GitHub a `Replace` running `Paste`
    with the kept `AppId`;
  - **one forge stopped:** its key refused, it answers `NotConnected`,
    is named with Replace, and the engine and the other forges run on;
  - **a forge made and removed:** a record kept, numbered next, joining
    the component, the identities' table and the setup host; a removal
    dropping it from each, its calls `NotConnected`, its rows erased
    one after another, each durable before the next, and the root told
    once the last is; a removal restored at start erasing only once the
    start's store mark is durable; and the other forges serving
    throughout;
  - **installations:** an `Accept` kept before `Request::Installations`,
    a deletion rewriting `Accepted`, a suspension's calls `Forbidden`,
    and `NotInstalled` at cost 0;
  - **generations:** strictly increasing per identity, the value kept in
    the value table before `Event::Credential`, a push across a renewal
    ending on its value, and generation 1 at each start;
  - **pasted values:** a digest that differs refuses the run, "the
    pasted value changed"; the carried row erased when its run ends,
    done or failed, its items kept first on `Done`;
  - **the box's link** (`engine.md`, 8): its machine's step tests, lines in and
    out: an ask written, another refused "busy" until `started` or
    `failed`, and every ask refused "not under the box" with no pipe;
    each line for the forge asked; a line for another forge, an unknown
    word, step or number, each named and dropped; a line at
    `box.line_bytes` read and one byte past it named and skipped; the
    pipe closed during an ask; `remove 3` held behind an ask and sent
    once it is answered, a start held for forge 3 dropped, `removed 3`
    telling the root, and `failed 3 WHY` named with the step left
    undone. Then the routing with a scripted box on a
    pipe pair: an ask, `start 3`, `step 3 prepare`, `failed 3 prepare 3
    exited 1`, the ask again and `start 3` again, a third refused
    "busy", a stray `started 9` dropped, and the root told each state;
    and the service's step world (../setup/owned.md, 5): at `started 3`, the
    forge connecting, the handover read and `TakeOver` run, its record
    kept before the file's removal, `Reconcile` with the root's restored
    list, then `Request::Connect` from records and the git value put at
    its generation before `Event::Credential` and `Request::Connect`; a
    `TakeOver` stop leaving the file and the forge set up failed; a
    failed removal blocking `Reconcile`; and the next ask, answered
    `started 3`, taking the handover over again, removing it and
    connecting;
  - **GitHub's three routes** on the clients' listener (the manifest
    page, the redirect and the setup address; `../../client/forge/served.md`,
    1): requests through `fixtures::http::Peer` to the service's
    routes, each answer and its headers, and a state used, unknown or
    past its life refused (`../../testing/README.md`, 2.1).

  The stories through temper's real loops, connecting, adopting and
  landing on each dialect, are `../../testing/README.md`'s (`engine.md`, 4).
- **No test calls github.com.** GitHub's dialect runs against skein's
  fake alone. An App's key is one of the fake GitHub layer's fixed test
  keys (skein's `testing/fakes/github.md`, section 1.1: 2048-bit, made for the
  fake and public), the same the real loop's first story seeds, since
  generating one per test is slow; every other credential a test draws
  is a token of random bytes. No test uses a real credential.
- **The cost checks,** in the forge connector's world counting
  `Request::Forge`, and in the protocol world counting HTTP requests:
  - **idle:** over a span with no change on the forge, the calls made are
    at most the keep-up listings, one per repository every `poll`, the
    held pull requests due on their backoff, and the slow pass's reads
    every `slow`; the bound is computed from the limits;
  - **history-free:** the same scenario on a forge preloaded with ten
    times the closed items and comments makes the same calls while idle;
  - **busy:** listing pages per pass are at most the changed items over
    the page size plus one, and item reads at most the changed items
    held times one plus the pages of their new comments;
  - **honest costs:** in the protocol world, the requests the fake served
    equal the sum of the costs the answers reported, the forges' starts'
    reads aside.
- **The real Forgejo check,** its transcripts, its facts and the
  throwaway Forgejo it runs against (`../../testing/process-tier.md`): skein's
  `testing/fakes/forge.md`, sections 6 and 7.
- **The pin** is skein's (skein's `forge/README.md`, 4.3): one patch release of Forgejo
  v17.0 and one of forgejo-runner v13.2, constants the throwaway
  Forgejo, the conformance and owned mode share (../setup/owned.md, 1); a
  binary of another release is refused, naming it. A later patch changes
  skein's constants and records, and not this document.
- **The process tier** runs the forge component's start, polling and
  webhooks against an owned Forgejo (`../../testing/process-tier.md`; ../setup/testing.md,
  4).

## 10. What others owe

The boundary, the configuration and the setup's hooks are `engine.md` and `README.md`, section 4
's. What an owner owes the component is skein's
(`forge/connection.md`, section 9). The forge client
(`skein-forge-client`, owned by temper's forge connector,
`../../domain/forge/adoption.md`) owes the component:

- **`Read::Locate { owner, name }`,** answered `Answer::Located {
  repository }`, addressed with the forge's number and repository 0
  (skein's `forge/forgejo.md`, 1.2); its answer's shape checked as every other's,
  a located id never 0, and its owner and name counted in the call's
  bytes.
- **Costs,** as the boundary carries them: `Event::Answered { call,
  cost, free, rate, result }`, `cost` the requests the call wrote, cache
  fills included, 0 for none and for an answer given at once
  (`NotConnected`, `NotInstalled`); `free` its `304`s; `rate` the
  window the forge last reported, if any. The budget charges `cost` less
  `free`, and keeps the least `rate` (`../../domain/forge/keeping-up.md`, 4; skein's
  `forge/client.md`, section 6).
- **The writer,** given by the connector at each `Connected`, since a
  forge's writer is its connect's user (`engine.md`, 2).
  The forge client keeps it as its own record, `writer(d, env, Writer {
  forge, author }, out)`: `Stored::Writer` under `Key::Writer(forge)`,
  in temper's small store under its tag 16, restored
  at start, so a restored effect never meets `Forbidden` (skein's
  `forge/client.md`, 7.1).
- **Hints,** as the vocabulary has them: `Hint { repository, change, key
  }`, with the pusher on `Change::BranchMoved`, so the domain drops its
  own echoes (section 4; skein's `forge/README.md`, section 6).
- **Shared limits:** the client's `calls`, `rows`, `answer_bytes`,
  `op_bytes` and `lifetime`, and the forge's `repositories` and
  `name_bytes`, are given to the component too, which sets and checks
  its own by them (skein's `forge/connection.md`, section 7).

jig owes the routing its secrets' store under temper's tag (`Keep`,
`Erase`, `Get` and `Load`, each `Keep` answered once durable), and the
request carrying a value (../credentials.md, pasted values; jig's
`protocol/client/README.md`): the value, at most 4 KiB, kept at decode under a
number jig gives it from 2^63 up and keeps the next of in a counter
row, only `{ number, digest }` reaching the core; and the core's one
output naming each carried value it leaves unused, which temper's root
forwards to jig's clients' component to erase.

## 11. What temper relies on

Held by skein's `forge/forgejo.md`, section 2: what the component and the forge
connector rely on of Forgejo, on the pinned release (`README.md`, section 9), each
named by its fact in the conformance's `facts.txt` (skein's
`testing/fakes/forge.md`, section 7). What no fact records yet is open: the kit's
in skein's `forge/forgejo.md`, section 2, and skein's `forge/README.md`, section 10, temper's own in `README.md`, section 12.
temper's part:

- **What it settles after it delivers.** temper relies on neither order
  of a delivery and the state it settles, only on each settling within
  `settle` of the move, 30 s in the profile `small`
  (`../../domain/forge/keeping-up.md`, 2).
- **The setup's, and git's.** What the setup relies on is skein's
  (`forge/setup/forgejo.md`, section 2; ../setup/testing.md, 3); what git relies on of
  Forgejo (a renamed repository's redirect, a protected branch's refusal
  of a push) is ../git.md's (`../git.md`, section 3 and 4.8), named in
  `crates/temper-git-transcripts-tool/facts.txt`.
- **GitHub's,** each a named fact of skein's `forge/github.md`, which
  temper's tests run against skein's fake alone (section 9): the App's
  installations and their tokens' hour, the delivery log and its
  validators, a job log's redirect, the rate windows and their pauses,
  `404` for a repository an installation did not select, and the merge
  queue's answer (`../../domain/forge/queues.md`, 1). No test of temper's
  checks them against github.com: a fact skein's fake gets wrong is
  skein's to correct, and temper's stories follow.

## 12. Open questions

- **Links beyond a box's sole owner** (`engine.md`, 1): several members require
  separate account evidence and a family of links; a pasted App's key
  alone identifies no person. Default: adoption's seeds there
  (`../setup/README.md`, section 9; `../../draft/forge-identity.md`, section 12).
- **The kit's own** are skein's: `README.md`, sections 8 and 10;
  `skein/docs/design/forge/connection.md`, section 11; `skein/docs/design/forge/client.md`, section 11. They
  hold every question and every unrecorded fact about the vocabulary, a
  dialect or the component, among them how long threads are read, a
  dismissed review, contents on demand, a file too large, bodies past
  `body_bytes`, an edit within a creation's second, the merge style, an
  expired or revoked token, what comments page, and a job no runner took.
- **What no fact records yet and temper's domain relies on,** read from
  Forgejo's API description and sources alone (section 11):
  - that the push workflow an update starts runs on exactly the head
    the update made, so CI is read again there (`../../domain/forge/README.md`,
    section 6);
  - that a merge after its pull request's base was changed lands in the
    new base: the pull request is read afresh before a merge, and one
    retargeted since the decision is not merged, but nothing makes the
    merge itself conditional on its base (`../../domain/forge/README.md`, section
    4).
- **GitHub's dialect and setup's open questions** are skein's
  (`forge/github.md`; `skein/docs/design/forge/setup/README.md`, GitHub's plans). temper's:
  - **A job log's origin learned at runtime.** Default: the kind's
    list; an origin outside it is named by `Fact::Origin`, and its log
    is not fetched (2.6). A member naming more origins, later. DNS
    resolution while serving remains later: skein's loop has no
    resolver.
  - **A Forgejo secret with no listener.** Default: handed none, the
    forge polls alone, named once in the log (section 4).
  - **Rows of a forge with no record.** Default: left as they are,
    named once in the log at start; a removal erases a kept forge's
    (`engine.md`, 1, 3).
  - **Generations across starts.** Default: generation 1 at each start,
    while no worker outlives its engine (`engine.md`, 5; ../credentials.md, its open
    questions).
