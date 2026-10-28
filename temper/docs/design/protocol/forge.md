# The forge's protocol layer

Provisional, 2026-10-03, revised 2026-10-22. How the engine reaches a
forge, on skein's forge kit: what temper's forge component, its setup's
host and its forge connector do with it, and what the kit leaves to
temper. The kit is skein's (`forge.md`, `forge-connection.md`,
`forge-client.md`, `forge-setup.md`, `forge-github.md`,
`fake-forge.md`): Forgejo's API and GitHub's over HTTP and JSON, the
keys' markers written at the head of what the engine creates, the
webhooks Forgejo sends back to a listener on loopback and the deliveries
a GitHub App's log keeps, the client, the component, the setup and the
fake forge. A forge's credentials are records in jig's secrets' store,
loaded at start and handed over at runtime (section 2; credentials.md,
the forges' records). The overview is README.md (sections 5 and 6); the
vocabulary carried is skein's (`forge.md`, section 3), and the forge
client's use of it is `../domain/forge.md`'s (sections 5 and 6); the
machines are skein's (skein's `http.md`, `json.md`, `tls.md`). What
others owe this layer is in section 12, what temper relies on of each
forge in section 13, and what is open in section 14.

## 1. In one page

- **The component, the setup's host and the client are skein's, owned
  by temper.** `skein-forge-connection`, the forge component, and
  `skein-forge-setup`'s host run as children of the engine's protocol
  layer (2.6). `skein-forge-client`, the forge client, is owned by
  temper's forge connector (`temper-engine-domain-forge`). Each
  dialect's documents and call plans, Forgejo's and GitHub's, the keys'
  markers, the deliveries, the setup's plans and the fakes are skein's
  too (skein's `forge.md`, section 2; `forge-github.md`). This document
  holds what the kit leaves to its owner.
- **Translation only, and no retries.** Each call of the forge client (a
  `Read` or a `Write`) is answered by exactly one `Answer` or `Error`,
  made of one or a few HTTP exchanges on one connection. A call goes out
  once. Its answer says how many requests it cost, so the client's
  budget charges what the forge actually served (skein's
  `forge-connection.md`, section 1; `forge-client.md`, section 6).
- **temper's routing hands the component every value** (section 2): the
  component reads no file. The values are temper's configuration's and
  each forge's records', with two of temper's own: the word `temper` and
  the user agent `temper/0.0.0`.
- **Repositories by the forge's own id; paged, bounded, and decoded as
  it streams.** Forgejo's conventions are skein's (`forge.md`, section
  5); the paths and the users the component keeps are
  `forge-connection.md`'s (section 4). Adoption first locates a
  repository by its owner and name (skein's `forge.md`, 5.2).
- **The engine's marks are keys' markers only** (section 4). A creation
  carries its key at the head of its body. The word is `temper`, and the
  key holds the deployment's id as its namespace. The marker's form is
  skein's (`forge.md`, section 6). Nothing else of the engine's is
  written into the forge.
- **A forge starts connected or not, and stops alone.** Its access (an
  API token, or a GitHub App's id and key, and the user it must stand
  for) is built from the forge's records at start (2.1), or handed over
  by `Request::Connect` once a setup's run has kept them (2.2). A forge
  with none answers each call `NotConnected`. A forge whose credential
  is refused is stopped and named to the root, and the engine and every
  other forge run on (2.3). The credential is the component's alone:
  never granted to a worker, never in the main store, and the first
  exchange with the forge checks that it stands for the forge's user
  (skein's `forge-connection.md`, section 3; credentials.md, the forges'
  records).
- **GitHub's installations and git's tokens come up as events** (2.4,
  2.5): the routing keeps the accepted installations in the forge's
  records, and makes each token the component mints a new generation of
  the forge's git identity, its value kept before jig's core hears of
  it.
- **Webhooks for latency, polling for liveness** (section 5). Forgejo's
  deliveries reach a listener of the component's own, on loopback, at
  `application.webhooks`'s address: verified as they stream, answered
  first, each hint naming what caused it (skein's `forge.md`, section 7;
  `forge-connection.md`, section 6). Polling stands as it is and finds
  what a delivery lost, refused or never sent told of; a forge on
  another host polls alone, since deliveries over TLS need skein's TLS
  server side (setup.md, section 15). A GitHub forge's deliveries are
  read from its App's log every `log_poll` instead, with no secret and
  no guid kept across process starts (5.1).
- **The vocabulary carries what a forge connected at runtime answers**
  (2.7): `NotConnected` and `NotInstalled` at once, at cost 0;
  `RateLimited { after }`; the rate each answer reports; validators,
  whose `304`s are free.
- **The load rules' values are temper's** (section 6): the rules are
  kept in skein's crates, and `../domain/forge.md` holds the values.
- **The fake forge, the throwaway and the conformance are skein's**
  (`fake-forge.md`). temper's worlds run temper's stories over them and
  count its costs (section 10).
- **What temper relies on of each forge is recorded.** Forgejo's by
  skein's conformance against a real Forgejo v17.0, each behaviour a
  named fact (skein's `forge.md`, section 9). temper requires that
  release: one patch release of it is pinned (skein's `forge.md`, 4.3),
  and no other version is served. GitHub's are skein's `forge-github.md`
  facts, which temper's tests run against skein's fake alone (section
  13).

## 2. In the engine

```
skein-forge-client                  the forge client: Read, Write, Answer, Error, Hint (skein-forge's vocabulary)
  ▲  the root: Request::Forge; Event::Forge (the client's Answered, the forge's Hint); each forge's connection (2.2 to 2.4)
the routing (temper-service)        io ↔ components ↔ root; builds each forge's access from its records (2.1); runs the setup's plans (2.6)
  ▲                    ▲
  │                    skein-forge-setup's host     the setup's plans, one run per forge; its reads and writes are records
skein-forge-connection              calls, connections, repositories' paths, users, each forge's start; GitHub's installations and tokens; the webhooks' listener and a GitHub App's log: verified deliveries as hints
  ▲  documents, call plans
skein-forge                         Forgejo's and GitHub's requests and answers, both sides; deliveries' payloads; the markers; the setup's documents
  ▲  tokens
skein-json
  ▲  body stream
skein-http (client; a server for webhooks, plaintext on loopback)
  ▲  plaintext
skein-tls (client, for a forge whose transport is TLS; later, a server for webhooks from another host)
  ▲  ciphertext
skein-io socket
```

- **The kit's crates** are skein's (`forge.md`, section 2). `skein-forge`
  depends on skein-lib and skein-json and holds nothing of temper's
  domains; `skein-forge-client` depends on skein-lib and `skein-forge`'s
  vocabulary; `skein-forge-connection` depends on the crates below it in
  the diagram. `skein-forge-setup` is the setup's: its host component
  runs in the engine beside the forge component (2.6). temper runs the
  plans there alone: no command of temper's drives one (setup.md,
  sections 5 and 11).
- **The forge component,** `skein-forge-connection`, is a child of the
  engine's protocol layer, beside jig's components, with its own limits,
  worst case, deadlines and most outputs per entry point; the routing
  carries what passes between it, io and the root, and decides nothing
  (README.md, sections 5 and 6; jig's `protocol/service.md`, section 2).
  One component serves every configured forge. What it holds is skein's
  (`forge-connection.md`, sections 3 to 7): the calls in flight, the
  forges' connections, each forge's start, the repositories' paths and
  the users, a GitHub App's installations and tokens, the translation
  between the forge client's vocabulary and the documents, the webhooks'
  listener and a GitHub App's delivery log.
- **What the routing hands the component.** The component reads no file
  (skein's `forge-connection.md`, section 3), and no forge's credential
  is a file: each is a record in jig's secrets' store, which the
  routing, `temper-service`, loads (2.1). For each entry of
  `application.forges` (configuration.md, section 3), a `Forge { number,
  kind, address, transport, access, log_origins }`:
  - the forge's number, and its kind, Forgejo or GitHub, which names the
    dialect;
  - its API's address and transport: plaintext, or TLS with its server
    name and its trust (the machine's roots when `trust` is null);
    GitHub's default to github.com's, and are given only to point at a
    fake;
  - its access, `Some` or `None` (2.1): the credential, the user it must
    stand for, and a Forgejo forge's webhook secret when it has one;
  - its `log_origins`, the origins a job log's redirect may reach (3.6).

  And `application.webhooks`, the listener's address, when configured.
  And temper's two values of its own: the word `temper`, which marks its
  keys and its bot (section 4), and the user agent, `temper/0.0.0`. And
  the limits: temper's `limits` member (configuration.md, section 7),
  and the values the component shares with the forge client (section
  12).
  The setup's host gives its plans the same word (setup.md, section 5),
  and their requests carry the same user agent. The component reads no
  other member: a forge's `git` base is the routing's, for runs' remotes
  (channel.md, section 4), its `web` is the setup's and the client's
  (GitHub's browser pages), and its `identity` names a git identity,
  which the component never holds (2.5).
- **What crosses to the root** is the root's boundary: a
  `Request::Forge { call, repository, op }`, which the routing hands to
  the component, ended by exactly one `client::Event::Answered { call,
  cost, free, rate, result }`, which the routing hands the root as
  `Event::Forge` (the forge's `Event::Client`); the forge's `Event::Hint
  { hint }`, which no request causes: a verified delivery's (section 5;
  `../domain/forge.md`, 5.2); and each forge's connection, which the
  routing tells the root as it changes: connected and as whom, stopped
  and why, and a GitHub installation's changes (2.2 to 2.4).
- **Room, and the drain,** as the component keeps them (skein's
  `forge-connection.md`, section 2) and as jig's components keep them
  (jig's `protocol/service.md`, section 2; jig's `domain/engine.md`,
  6.1): an answer or a hint the root cannot take yet is held and offered
  again, at most one per call and one per webhook connection, and
  nothing more is read off its connection meanwhile. One the root
  returns during its drain (its `Request::Busy`) is never offered
  again: it goes when the component closes, and the next process reads
  afresh and settles its outbox (`../domain/forge.md`, section 16). The
  routing's own news of a connection or an installation that the root
  returns during its drain is dropped. The root's view of them is not
  kept, so the next process's root starts from what a fresh start
  publishes: each connection afresh, and the installations by the
  baseline listing (skein's `forge-connection.md`, section 4.4). The
  routing closes the component after the root's drain (jig's
  `protocol/service.md`, section 5); calls in flight end with their
  connections, and no answer goes up once the root has stopped.
- **A start that fails for good stops that forge only** (2.3): a user
  other than the forge's own, a credential refused, or a `4xx` that
  asking again would not mend (skein's `forge-connection.md`, sections 3
  and 8). The engine and every other forge run on.

### 2.1 At start: each forge's records, and its access

- **One load, before the component starts.** The routing reads temper's
  rows of jig's secrets' store (`Request::Secrets`, `Load` under
  `SecretOwner::Application`, paged by number; jig's
  `protocol/store.md`, section 9) and groups them by forge. A row's
  number is `family << 48 | forge << 16 | item`: family 1 is a forge's
  records, `forge` the configured forge's `number`, and `item` what the
  row holds: 1 `AdminToken`, 2 `ApiToken`, 3 `GitToken`, 4 `User`, 5
  `WebhookSecret` (Forgejo's alone), 6 `AppId`, 7 `AppSlug`, 8 `AppKey`,
  9 `Accepted` and 10 `Origin`, codes that never change meaning
  (credentials.md, the forges' records). `User` is the forge's user
  temper acts as: on Forgejo temper's user, on GitHub the App's bot
  user. temper's numbers keep the top
  bit clear, below 2^63, where jig numbers carried values under
  temper's tag itself (2.6).
- **Each item's bytes** are as skein's setup host writes them, pinned by
  skein's `forge-setup.md`, 6.4: tokens and secrets as text; `User` and
  `AppId` a `u64` little-endian, never 0; `AppSlug` text; `AppKey`
  PKCS#1 DER; `Accepted` the installations' ids, each a `u64`
  little-endian, ascending and distinct. `Origin` is temper's, the
  forge's kind and `api` address the records were made for, written at
  each connect. They are decoded with skein's item decoders only where
  they build an access.
- **The access** (skein's `forge-connection.md`, 3.5):
  - Forgejo: `Access { credential: Token(ApiToken), user: User,
    webhook_secret: WebhookSecret }`, the secret only when its row is
    there and `application.webhooks` is configured (section 5);
  - GitHub: `Access { credential: App { id: AppId, key: AppKey }, user:
    User, webhook_secret: None }`, since a GitHub forge has no secret
    (5.1), then `Request::Installations { forge, accepted }` from its
    `Accepted` row (2.4).
- **Not connected, and named.** A forge starts with `access: None` when
  it has no rows; when its set is incomplete (a conversion a crash cut:
  an `AppId` kept and no `AppKey`, say); when a row is unreadable (a
  version temper does not know, bytes skein's decoder refuses, a value
  past `records.value_bytes`), named on the deployment's page "its
  records are unreadable"; or when its rows were made for another forge
  (an `Origin` naming another kind or `api` address, an `AppKey` under a
  Forgejo forge, an `ApiToken` under a GitHub one), named "its records
  were made for <kind> at <address>". Nothing is erased: connecting
  again overwrites. Every other forge starts as it would.
- **Rows of a forge the configuration does not name** are left as they
  are, and named once in the log; erasing them is disconnecting, later
  (section 14). Carried rows, pasted values whose run a restart cut, are
  loaded from 2^63, where jig numbers them; the core decides their
  erase, which the routing carries out (2.6).
- **A managed, owned or fake Forgejo starts connecting:** after the
  records' load and the root's restore, its access stays `None` until
  the setup host's `Reconcile { forge, adopted }` is done and its writes
  kept, then `Request::Connect` (2.2). `adopted` is the root's restored
  repositories for the forge; a fake then runs its start-time adoption
  before connecting (setup.md, 8.4). For owned and fake, the engine first takes
  the handover whenever its file is present, even with a kept
  `AdminToken`, removes the file after `TakeOver`'s `Done` and the
  record kept, and reconciles only after removal succeeds (setup.md,
  6.5). With no file it reconciles from the kept record. A stop or
  failed removal leaves that forge not connected, named by the step,
  never a secret; every other forge runs on. A connected Forgejo and a
  GitHub forge start with their access.
- **An owned forge set up from the client** (setup.md, 6.5) starts not
  set up: one whose setup says `"start": "asked"`, with neither the
  handover file nor an `AdminToken` record, has its access `None` and no
  run until the box answers `started` on its pipe (2.2). Then the engine
  runs the same handover, removal, reconcile and connect sequence as at
  a start, with access and git value from records.
- **The mode is no record** (setup.md, the modes): owned is the
  configuration's (a `setup` entry owning the forge), managed is an
  `AdminToken` row on a forge not owned, and connected is neither.
- **Checked at start,** each refusal named (configuration.md, its
  starts): `records.value_bytes` against the largest value jig's
  secrets' store takes; and, when a GitHub forge is configured, a fixed
  address for the clients' listener, since the App's `redirect_url` and
  `setup_url` name it and the API cannot change them.

### 2.2 Connecting at runtime

- **`Request::Connect { forge, access }`** hands a running forge its
  access: a first connect, a replacement and a reconnect after a stop
  alike (skein's `forge-connection.md`, 3.5). The routing sends it after
  a setup's run is done and its writes are kept (2.6): a managed or
  owned Forgejo's `Reconcile`, at start or after the box's `started`, a
  connected Forgejo's `Check`, GitHub's `Convert` and `Paste`, and each
  `Replace`. The access is built from
  the records, as at start, never from the run's outcome: a forge
  connected at runtime is the forge a restart would load.
- **`Event::Connected { forge }`** reaches the root through the routing
  with the identity the records hold, its id and login (the `User` row;
  on GitHub, the App's bot user): the forge is connected, and the
  deployment's page says as whom (client/forge.md, the deployment's
  page). The connector then gives the forge client its writer, which the
  forge client keeps as its own record and restores at start (skein's
  `forge-client.md`, 7.1; `../domain/forge.md`, 4.1). A GitHub forge's
  `Reconcile { forge }` runs next, once the forge's writing run is free
  (2.6); it ends in no connect.
- **The box's pipe** (setup.md, 7.1), under the box alone, through the
  routing's box link (2.8): at `SetUp { forge }` (client/README.md,
  section 8), admitted by the root, the link writes `start FORGE`, one
  ask at a time, and refuses it "not under the box" with no pipe; it
  tells the root each `step` line, the forge setting up and at which
  step; at `started FORGE`, the forge's `TakeOver` when the handover
  file is present, then its `Reconcile`, then `Request::Connect`, with
  the handover removed before reconcile and access built from records
  (setup.md, 6.5); at `failed FORGE WHY`, the forge set up failed, with
  `WHY`. An answer for a forge it did not ask for is named in the log
  and dropped.
- **Until then** each call on the forge is answered `NotConnected` at
  once, at cost 0 (2.7). The root shows the forge connecting while a run
  holds it, setting up while the box runs its group, and not connected
  otherwise.
- **A replacement** ends the calls the old access began `Forbidden`, as
  a revocation would. A credential that answers as another user is
  refused before anything is kept (`Check`, `Paste`); one the component
  refuses stops the forge (2.3).

### 2.3 A forge stopped

- **`Event::Stopped { forge, why }`** stops that forge only: its key or
  token refused, a user other than its own, an access the component
  finds invalid, or a `4xx` that asking again would not mend (skein's
  `forge-connection.md`, 3.3 and 3.5). Its calls are answered
  `NotConnected` until its next connect.
- **Named to the root,** which shows the forge and why on its row, with
  Replace (client/forge.md, the deployment's page). The engine and every
  other forge run on; nothing is erased, and the next start tries the
  same records. Pending in the code: it is named on `temper serve`'s
  standard error, not yet to the root.
- **The webhooks' listener** stays the engine's: one that fails stops
  the engine (section 5).

### 2.4 GitHub's installations

- **The accepted installations** are the forge's `Accepted` row: the
  whole list, at most `forges.installations` ids, within the
  component's `installations`. One is accepted when the person comes
  back through the setup URL with a state of temper's (`Accept`, 2.6),
  or when the deployment's owner accepts one that exists already,
  through the tools, with no state (`setup.md`, 17.4).
  The routing hands the list over with `Request::Installations { forge,
  accepted }` at start, after an `Accept`'s `Done` and after a deletion;
  the component reads the installations and mints a token for each one
  accepted (skein's `forge-connection.md`, 4.4 and 5.1).
- **`Event::Installation { forge, installation, account, change }`,**
  an `account` being its id and login:
  - `Created`: nothing, unless accepted, at the setup URL or by the
    owner's accept. One never accepted is a stray: shown on the
    deployment's page, and uninstalled by reconcile after
    `setup.state_life_ms` (skein's `forge-setup.md`), unless the App is
    read-only (`setup.md`, 17.7).
  - `Deleted`: the `Accepted` row is rewritten without it and kept, then
    `Request::Installations`. The git identity's value loses that
    account's token (2.5). Every adoption on the account is marked
    (`../domain/forge.md`, 4.3).
  - `Repositories` and `Renamed`: to the root. No names come with them:
    the connector lists that installation again, through the setup's
    host's `Reachable` plan (2.6), and marks each adoption whose
    repository left the installation (`../domain/forge.md`, 4.3). Each
    `Connected` lists again every accepted installation that holds an
    adoption, so a change made while temper was away is found. The marks
    are held in memory, never stored.
  - `Suspended` and `Unsuspended`: to the root, shown on the deployment's
    page. A suspended installation's calls are answered `Forbidden`, and
    its git token leaves the identity's value until it is unsuspended;
    its adoptions are marked until then.
- **`Event::Fact { forge, fact }`** decides nothing: it is the
  component's diagnostic for the owner's log (skein's
  `forge-connection.md`, 2.1). `Fact::Unkept`, an installation's change
  the component had no room for, and `Fact::Gap` (5.1) are logged only.
  `Fact::Origin` (3.6) is kept as well, per forge in memory.
- **A call on an account no accepted installation covers** is answered
  `NotInstalled` at once, at cost 0; one on a repository its account's
  installation did not select answers `Missing`, as GitHub's `404`
  says (2.7).

### 2.5 Git's tokens, and generations

One git identity per forge (credentials.md, git identities): its number
and author are the configuration's, its value the forge's.

- **Forgejo's value** is the `GitToken` record's, its login the bot's
  (`User`): generation 1 at start, or at the first connect after the
  box's `started` when the start had none. Each later value takes the
  next generation of its number. The value is put before its
  `Event::Credential` and before `Request::Connect` (2.2, 2.8).
- **GitHub's value** is a list of `(account id, account login, token,
  expires)`, one per accepted installation, built from `Event::GitToken
  { forge, installation, account, token, expires }` and held in memory
  only (skein's `forge-connection.md`, 5.3). Its login is
  `x-access-token`; its author defaults to the bot's, `<slug>[bot]` and
  `<id>+<slug>[bot]@users.noreply.github.com`, `<id>` the `User`
  record's.
- **Each change is a new generation,** strictly increasing per number:
  a token minted or renewed, an installation deleted, suspended or
  unsuspended. Each accepted installation's token is renewed about
  hourly, `renew_before` ahead of its expiry, so with N installations a
  GitHub forge's identity takes about N generations an hour.
- **In order:** the value goes into temper's value table first, which
  keeps the two newest values per number and whose value server answers
  either; then `Event::Credential { number, generation, valid }` to
  jig's core, which pushes it to the live attempts holding the identity
  (jig's `domain/engine.md`, 12.1). A worker keeps the two newest, so a
  push in flight across a change ends on the value it began with.
  `valid` is Forgejo's for as long as the engine runs, and GitHub's the
  earliest `expires` among the value's tokens, less now.
- **Generation 1 at each start:** safe only because a worker never
  outlives the engine that started it (standalone). A fleet whose
  workers outlive it needs generations kept across starts
  (credentials.md, its open questions).
- **The worker picks** the entry whose account id is the repository's
  owner's, which the workspace item carries; with none, the push is
  refused, named "temper's App is not installed on the account owning
  <name>" (channel.md, section 6). The agent
  never sees a token: jig's hub forwards no application's credential to
  it.
- **The value's codec,** engine to worker, is temper's wire format,
  versioned, holding Forgejo's single token or GitHub's list
  (credentials.md, git identities). It is never stored.

### 2.6 The setup's host

skein's in-process host component (`forge-setup.md`, its host), a child
of the engine's protocol layer beside the forge component. The routing
carries what passes between it, the root and the records.

The engine builds the setup host once at start from the configured
forges, admitting `host::worst_case(&limits, &settings)` against its
own bound before it builds it; a full host answers `Why::Busy` at a
Run's start, and the page shows it as busy, try again (skein's
`forge-setup.md`, 6.5).

- **Down,** `Run { run, plan }` and `Answer { run, answer }`; **up,**
  `Need { run, need }`, `Done { run, outcome }` and `Stop { run,
  failure }`. It holds an admin's token and a GitHub App's key only
  while a plan runs. temper has no use for skein's one-shot host
  (`forge-setup.md`, 6.2): `temper forge prepare` and `run` drive no
  plan (setup.md, section 5).
- **The plans the routing runs** (skein's `forge-setup.md`; setup.md):

| Plan | When | What it does |
|---|---|---|
| `Reconcile { forge, adopted }` | each start of a managed or owned Forgejo; a managed connect; the box's `started`, after its `TakeOver` (2.2) | converges the forge on what the deployment needs (setup.md, section 5), then the forge connects |
| `TakeOver { forge, admin_token }` | an owned or fake forge's start, or the box's `started`, whenever the handover file is present, even with a kept `AdminToken` (setup.md, 6.5) | keeps the handover's admin token; only after its `Done` and the record kept is the file removed, before reconcile |
| `Adopt { forge, repository }` | a managed or owned forge's adoption, before its reads | gives the bot access to the repository (`../domain/forge.md`, 4.2) |
| `Check { forge }` | a connected Forgejo's pasted tokens, and their replacement | both tokens answer as one user, and as the kept user on a replacement |
| `Manifest { forge, target }` | GitHub's Connect, always for `Personal`: the App is the person's, public, and installs on any account GitHub offers them (setup.md, 17.2) | a fresh state, the form's target and the manifest; it ends `Done(Manifest { state, target, manifest })`, from which temper renders the manifest's page (client/forge.md, 9.1) |
| `Convert { forge, code, state }` | GitHub's redirect back | the code exchanged for the App's id, slug and key, the conversion's webhook secret and OAuth pair never kept; the bot user found; it writes `AppId`, `AppSlug`, `AppKey` and `User` |
| `Install { forge }` | Install the App, or Install on another account | a fresh state, and the install link (`Link`) |
| `Accept { forge, installation, state? }` | the setup URL's visit, with its state; the owner's accept of a stray, with none (`setup.md`, 17.4) | the installation checked, and added to `Accepted` |
| `Paste { forge, app }` | a pasted App's Connect; Replace on GitHub, with the kept id | the pasted key (skein's `Pasted`, read once, never written) decoded to DER, `GET /app` matching the id, the bot user found, the App's hook set to `https://<slug>.invalid/`, the permissions and events checked as reconcile checks them; it writes `AppId`, `AppSlug`, `AppKey` and `User` |
| `Reconcile { forge }` | after each GitHub `Connected` | the App's permissions, events, installations and key checked; it repairs only its hook's address (5.1) |
| `Reachable { forge, installation?, page }` | the picker, on every kind | the repositories the picker lists, paged by number from 1, at most `reachable` rows (temper's `picker.page`), `more` set when the page is full: on GitHub one accepted installation's, `installation` its id; on a connected Forgejo what the API token can push to; on a managed or owned Forgejo every repository the admin's token sees, by id ascending, refused `Missing` or `NotAdmin` as reconcile's first step; an `installation` not in `Accepted` stops `Installation` (`Missing`), and one given for a Forgejo stops `Installation` (`Unexpected`); it ends `Done(Reachable { rows, more })`, each row the repository's id, its owner's `{ id, login }`, its name and whether it is private; it writes nothing (`../domain/forge.md`, 4.3) |

- **`Reachable`** is skein's (`forge-setup.md`, 4.8). On a managed or
  owned Forgejo it runs on the admin's token, which stays the setup
  host's; on a connected Forgejo, on the API token (skein's
  `forge-setup.md`, 4.6); on GitHub, on a token minted for the page and
  dropped. Since it writes nothing, it runs beside the forge's one
  writing plan.
- **Outcomes.** A run ends `Stop` with its failure, or `Done`, which
  carries what temper serves or shows (skein's `forge-setup.md`, 4.2):
  `Reachable`'s page for that plan; `Manifest { state, target,
  manifest }`, the state, the form's target and the manifest's JSON,
  from which temper renders the manifest's page (client/forge.md, 9.1),
  at most `page_bytes`, and holds it in memory with the state while it
  lives and serves it on
  the clients' listener (client/README.md, the routes); `Link`, the
  install link; each state held with the request's `back`, a
  connect's passing to the install its `Convert` starts, where the
  setup route sends the browser (client/forge.md, 9.1); or `Lacking`, each permission or event the App lacks,
  shown on the deployment's page and never refused (client/forge.md,
  the deployment's page).
- **The redirect and setup addresses** temper gives the plans are whole
  URLs, built from the clients' listener's address:
  `http://<listener>/forges/<n>/github/redirect` and
  `http://<listener>/forges/<n>/github/setup` (2.1, the fixed address).
  The manifest's `url`, the App's homepage, is `http://<listener>/`.

- **A run's needs.** `Need::Write { item, bytes }` is a `Keep` under the
  forge's family-1 number for that item, answered only on `Kept`.
  `Need::Read(item)` is served from the run's carried row first, by the
  number jig gave it, else from the forge's row.
- **Pasted values** (credentials.md, pasted values): a `Connect` or a
  `Replace` may carry one value, temper's bundle of items. jig's
  clients' component strips it at decode and keeps it in the secrets'
  store under temper's tag before jig's core sees the request, at a
  number jig gives it from 2^63 up, at most 4 KiB; the core gets
  `{ number, digest }` (jig's `protocol/client.md`). A carried value the
  core leaves unused, its request refused, replayed, joined to another
  or cut by a stop, is named in one output of the core's, which temper's
  root forwards to jig's clients' component to erase. The root checks
  the request and
  asks the routing to run its plan with that number. The routing `Get`s
  the row and checks that its digest is the request's, or refuses the
  run, "the pasted value changed"; and serves each pasted item's
  `Need::Read` from it. When the run ends, done or failed, the core
  decides the carried row's erase, which the routing carries out, a
  `Done`'s writes, and each pasted item but `Pasted`, being kept under
  the forge's family-1 numbers first (credentials.md, pasted values).
- **Carried rows found at start,** loaded from 2^63, are rows whose
  run a restart cut: the core decides
  each one's erase, which the routing carries out, and a connect the
  root still holds as running ends "interrupted; paste again".
- **A run's `Done`:** its writes kept and its carried row's erase
  carried out, then `Request::Connect` (2.2) after a plan that connects
  (a Forgejo's `Reconcile` or `Check`, GitHub's `Convert` or `Paste`),
  and never after GitHub's `Reconcile`; after `Convert`'s, `Install`
  first, whose link answers the browser (setup.md, 17.2);
  `Request::Installations` after an `Accept` (2.4); the adoption's
  reads after an `Adopt`; `Reconcile` after a `TakeOver`. **A run's
  `Stop`:** its failure named to the
  root, and the forge as it was.
- **States** are the host's, in memory: at most `setup.states`, each
  living `setup.state_life_ms` (1 h, the conversion code's hour). A
  restart forgets them, and a state used, unknown or expired is
  refused. Nothing about a state is stored.
- **One writing run per forge at once,** and one `Reachable` beside it;
  a second of either is refused "busy", never queued, and so is a
  request past `setup.runs`, which counts both (skein's
  `forge-setup.md`, 6.3).

### 2.7 What the vocabulary adds

skein's (`forge.md`, section 3; `forge-connection.md`, 4.1), as the
routing carries it to the root, and the domain reads it
(`../domain/forge.md`, sections 4 and 5):

- **`Error::NotConnected`:** a call on a forge with no access (not
  connected, connecting, stopped, or its replacement refused), answered
  at once, at cost 0.
- **`Error::NotInstalled`:** on GitHub, a call on an account no accepted
  installation covers, answered at once, at cost 0.
- **`Error::Missing`** on a repository its account's installation did
  not select (GitHub answers `404`), and **`Error::Forbidden`** for a
  suspended installation's calls and the calls a replacement ended.
- **`Error::RateLimited { after }`:** Forgejo's `429` and GitHub's
  pauses alike, `after` as skein's `forge-github.md` gives it (GitHub's
  `Retry-After`; else the window's reset when nothing remains; else
  60 s). Nothing retries in the component; the client's budget holds
  every class until then (skein's `forge-client.md`, section 6).
- **`api::Rate { limit, remaining, reset_at }`,** beside an answer's
  cost: the window the forge reported on the call's last answer, per
  installation on GitHub. The client's budget keeps the least it hears.
- **Validators** (`api::Validator`, at most 128 printable bytes): a read
  given one is conditional, and each `304` is counted in the answer's
  `free`, which no rate counts and the budget charges nothing for. The
  delivery log's polls take one (5.1).
- **A forge no one configured:** `Unavailable`, at once.

### 2.8 The box's link

`temper serve`'s side of the box's pipe (setup.md, 7.1): a step machine
of the routing in `temper-service`, its lines and the root's asks in,
its lines, the root's states and its answers out, hosted on the
service's io.

- **Under the box alone.** `temper internal serve --box` adopts the
  box's pipe, the two fds after its listeners (setup.md, 7.1), and the
  link writes its lines on the first and holds one read on the second,
  a line at most `box.line_bytes` (configuration.md, section 7), its
  line feed included, as the box counts it. Without `--box` there is no
  pipe and no link; the engine's forge domain, told so at its construction,
  refuses every ask "not under the box" (`../domain/forge.md`, 4.1).
  The link decides nothing: it carries the domain's asks out and the
  box's lines in.
- **An ask** is the forge domain's, once it has decided the `SetUp {
  forge }` by the forge's offer (`../domain/forge.md`, 4.1), which
  refuses "not offered", "not under the box" and "busy" itself. The
  link writes `start FORGE`, tells the root the forge setting up, with
  no step yet, and answers the ask by the request's token the root gave
  it: setting up. The domain asks once at a time, so a second ask while
  one runs is a fault of the domain's: the link answers it "busy",
  named in the log. The answer goes back to the root, whose forge
  connector keeps it as the `SetUp` request's outcome, in memory alone,
  and the asking client reads it (`../client/README.md`, section 8).
- **The box's lines,** for the forge asked:
  - `step FORGE STEP`, `STEP` one of `fetch`, `prepare` and `run`: the
    forge setting up, at that step;
  - **`started FORGE`:** the ask ends and the forge goes connecting. The
    setup host takes the handover with `TakeOver` whenever the file is
    present, even with a kept `AdminToken`, removes it after `Done` and
    the record kept, then runs `Reconcile` with the root's restored
    repositories and `Request::Connect` from records (2.1, 2.2;
    setup.md, 6.5). With no file it reconciles from the kept record. Its
    git value is the `GitToken` record's, put at its next generation
    before `Event::Credential` and `Request::Connect` (2.5). A stop or
    failed removal sets the forge up failed with the step named, never a
    secret; a `TakeOver` stop leaves the file. The other forges run on.
    The next ask, answered `started` at once, runs the sequence again;
  - `failed FORGE WHY`: the ask ends, and the forge's set up failed,
    with `WHY` as the box wrote it, which holds no secret (setup.md,
    7.1).
- **The failed step** is the forge domain's, from the connection it
  held when told set up failed (`../domain/forge.md`, 4.1), including
  `BoxClosed` during setup. The producers keep their existing events;
  the FORGE row's `SetUpFailed` carries `why` and `at: option
  FailedStep` (fetch, prepare, run or connect). The schema, generated
  codec and goldens update in place at CLIENT_VERSION 11 before the
  first release; no store or record version changes.
- **A stop** is the forge domain's too, at the demo's end (setup.md,
  8.4), which no request waits on: the link writes `stop FORGE`,
  holding it while another ask is out and sending it once that one is
  answered, as it holds a start. `stopped FORGE` ends it, telling the
  root nothing, since the forge is already ended (8.1); `failed FORGE
  WHY` ends it too, named in the log. The pipe closed during it ends it
  with the link.
- **Named and dropped,** each in one line of the log and changing no
  state: a line for a forge other than the one asked, or while none is;
  a line whose word, forge number or step the link does not know; and a
  line past `box.line_bytes`, named by its bound, skipped to its line
  feed.
- **The pipe closed:** the box's end of it closed, or a read or a write on the
  pipe failing, ends the link, named in the log once, and the link
  tells the root `BoxClosed`: the forge domain fails an ask in flight,
  "the box closed the pipe", and offers no set up after it
  (`../domain/forge.md`, 4.1). serve's own end is
  the box's to make (setup.md, 7.2).
- **Nothing kept.** The link's ask and the states it tells are in
  memory alone (8.1): a restart makes a pipe and a link afresh, with no
  ask, and a group the box still runs answers the next ask by its own
  lines (setup.md, 7.1).

## 3. Forgejo's API as temper uses it

Held by skein's `forge.md`, section 5, and `forge-connection.md`,
sections 3 to 5.

### 3.1 Conventions

Held by skein's `forge.md`, section 5.1.

### 3.2 Each operation

Held by skein's `forge.md`, section 5.2.

### 3.3 Decoding answers

Held by skein's `forge.md`, section 5.3.

### 3.4 Failures

Held by skein's `forge.md`, section 5.4.

### 3.5 What the protocol layer keeps

Held by skein's `forge-connection.md`: each forge's start and its access
in section 3, the repositories, the users and a GitHub App's
installations in section 4, the credentials and the installation tokens
in section 5, and a GitHub App's delivery log in 6.5. What temper keeps
of a forge is its records (2.1), never the component's state.

### 3.6 GitHub's API as temper uses it

Held by skein's `forge-github.md`: the dialect's paths, headers, paging,
validators, pauses and failures, each a named fact (section 13). Of it,
temper sets one thing, where a job log may be read from:

- **A job log's redirect** is followed only to an origin named in the
  forge's `log_origins` (configuration.md, the forges), resolved at
  start: at most `forges.log_origins` entries, each an origin, `https`,
  or `http` to a loopback host alone, matched as skein's component
  matches a redirect's (scheme, host in ASCII case, port; skein's
  `forge-connection.md`, 4.5). GitHub's default names the one blob
  store GitHub's logs are served from; empty, no job log is fetched,
  and a check's failure is shown without its log.
- **A redirect to any other origin** is answered `Refused`, and the
  component's `Fact::Origin` names that origin once for each connect
  (2.4). The routing keeps it per forge in memory, at most
  `forge_component.log_origins` of them, and forgets them at the forge's
  next connect; it shows it on that forge's row (client/forge.md, the
  deployment's page), and logs it once until a start names it; the
  operator adds the origin to `forges[].log_origins` and restarts
  (section 14).
- **No document, example or test of temper's calls github.com:** GitHub's
  default addresses are named in the configuration, and every test points
  the forge at skein's fake (section 10).

## 4. The engine's marks

What the engine writes inside what it creates: keys' markers, and
nothing else. No record, outcome or page of the engine's is written into
the forge, and no call of skein's `forge.md`, 5.2 writes a label: labels
are for display only, and no decision reads one (`../domain/forge.md`,
section 4).

- **A creation's key,** at the head of an issue's body, a comment or a
  review, is a marker: its form, its decoding and its bounds are
  skein's (`forge.md`, section 6). temper's word is `temper`, so a
  marker reads `<!-- temper:key <the key in lowercase hexadecimal> -->`
  and a line feed, then the text. An HTML comment, which Forgejo's
  rendering hides.
- **The key is the client's** (`effect_key`, skein's `forge-client.md`,
  section 7): version 1, the namespace's length as two bytes
  big-endian, the namespace, then the effect's identity, opaque bytes
  to the client. temper's namespace is the one the root derives from the
  deployment's id; its identity is its effect's purpose and its numbers,
  which the connector's top encodes (`../domain/forge.md`, section 6).
  So the marker carries the deployment's id, and deployments that share
  a repository tell their creations apart (jig's `domain/connectors.md`,
  4.4). The key's first byte is its version; the marker has none of its
  own.
- **Decoded** as skein's `forge.md`, section 6 says; a body whose marker
  carries another word has no key. Whether to believe a key is the
  client's: its version, its namespace, and whether its author is the
  forge's writer (skein's `forge-client.md`, section 7). A person who
  copies a marker into their own comment gives it no provenance.
- **Bounded.** The key is within the client's `op_bytes`, the marker
  twice its length and 21 bytes more (the word's 6 and 15), and the body
  sent the marker and the text, which `body_bytes` holds (skein's
  `forge-connection.md`, section 7).
- **The worlds** write the same marker (the forge connector world's
  translation, `tests/engine/forge/client/src/translate.rs`); the
  protocol world (section 10) runs the component's own encoder and
  decoder.

## 5. Webhooks

Forgejo's deliveries, and the deliveries a GitHub App's log keeps (5.1),
made hints: they cut polling's latency, and decide nothing. Polling
stands as it is with no webhook (section 6; `../domain/forge.md`, 5.2):
its `poll`, `poll_max`, `hinted` and `slow` are unchanged, and it finds
whatever a delivery lost, refused or never sent told of. A Forgejo
delivery reaches the engine on loopback; a forge on another host polls
alone (setup.md, section 15).

- **A delivery's signature, its events and its hints** are skein's:
  how Forgejo signs a body, the events a hook subscribes to, the hint
  each makes, who caused it, and the bounds on a delivery (skein's
  `forge.md`, section 7).
- **The listener,** its connections, the verification as a body streams,
  the answer first, each refusal and the bounds are skein's (skein's
  `forge-connection.md`, section 6).
- **The webhooks' listener's address,** `application.webhooks`,
  `{"listener": address}` (configuration.md, section 3): a socket
  address on loopback (in `127.0.0.0/8`, or `::1`), apart from the
  clients' listener (`../client/README.md`, section 8). The routing
  hands it to the component, which takes `POST` on one path per forge,
  `/hooks/<number>`, the forge's `number`.
- **Each Forgejo forge's secret** is its `WebhookSecret` record
  (credentials.md, the forges' records): drawn and kept by reconcile on
  a managed or owned forge, pasted with the API and git tokens on a
  connected one. The routing hands it in the forge's access (2.1) only
  with `application.webhooks`; a forge with a secret and no listener is
  handed none, polls alone, and is named once in the log. A forge with
  no secret has no path (skein's `forge-connection.md`, 6.1).
  `WebhookSecret` is Forgejo's alone: a GitHub forge has none (5.1).
- **What the domain does with a hint.** It drops its own writes'
  echoes: a creation's key for an item, the pusher for a push
  (`../domain/forge.md`, 5.2; skein's `forge-client.md`, section 4). A
  hint for a repository the deployment has not adopted goes up all the
  same; the domain drops it.
- **What has no delivery** is found by polling alone
  (`../domain/forge.md`, 5.2): a commit status, which Forgejo never
  delivers, so CI that a system other than Actions reports and the
  verdicts temper posts.
- **What Forgejo settles after it delivers** (a pull request's
  mergeability, a job's commit status: section 13) is the domain's to
  read again, within `settle` of the move (`../domain/forge.md`, 5.2);
  the component hints at once.
- **A listener that fails** while the engine runs stops the engine, as
  the clients' listener's does: the service drains as at a shutdown, and
  whatever starts `temper serve` again (the box, a service manager)
  starts it with a listener again.
- **Not sent again.** Forgejo does not resend a failed delivery (skein's
  `forge.md`, section 7): one lost while `temper serve` is down, refused
  for its size or its signature, or never sent is found by polling. A
  hint that arrives before the engine's domain is ready is dropped, and
  the keep's first listing finds what it told of.
- **Hooks are set up by reconcile, or by the person,** at
  `http://<listener>/hooks/<number>`, with the forge's secret, the
  events of skein's `forge.md`, section 7, JSON and active (setup.md,
  sections 4, 5 and 10; skein's `forge-setup.md`, section 4.3, step 6).
  A repository with no hook polls (`../domain/forge.md`, section 4).

### 5.1 A GitHub App's delivery log

GitHub's deliveries are read, not received: the component's puller
reads the App's log (skein's `forge-connection.md`, 6.5;
`forge-github.md`). The engine takes no delivery from GitHub, and needs
no address GitHub can reach.

- **The App's hook** points at no listener: its address is
  `https://<slug>.invalid/`, a name that never resolves (RFC 2606), and
  it is active, so GitHub records each delivery it fails to make in the
  App's log. The manifest sets it, and `Paste` (setup.md, connecting
  GitHub); GitHub's `Reconcile { forge }` repairs that address alone
  when it differs (2.6).
- **No secret.** A GitHub forge has no `WebhookSecret` record: the
  conversion's webhook secret is never kept, `Paste` draws none, and
  the routing hands the access `webhook_secret: None`. The forge has no
  path on the webhooks' listener, and the log's reads are vouched for by
  TLS and the App's JWT (skein's `forge-connection.md`, 3.5 and 6.1;
  `forge-setup.md`, the conversion).
- **Each `log_poll`** (5 s in the profile `small`), while the forge is
  connected, the puller reads the log's first page conditionally, with
  the validator the last first page answered: a `304` is free (2.7) and
  ends the poll. Otherwise it reads back, newest first, to the newest
  delivery it read, within `log_pages` (skein's `forge-connection.md`,
  6.5). The reads are counted in no call's cost.
- **No guid is kept across process starts.** The newest delivery's
  guid is the component's, in memory: keeping it would write the store
  at each delivery, and hints are only hints. At a process's first
  start of a forge, the component's first poll keeps the newest guid
  and reads nothing back. What changed while temper was down is read by
  the forge client at its restart, which reads each adopted repository
  afresh (`ReadAfresh`; skein's `forge-connection.md`, 6.5, and
  `forge-client.md`, section 8; `../domain/forge.md`, section 16).
- **A later start of the forge within one process,** a connect after a
  stop, reads back to the guid the component kept, as far as
  `log_catch_up` and `log_pages` allow (skein's `forge-connection.md`,
  6.5).
- **A gap,** a poll ending on `log_catch_up` or `log_pages` before it
  meets the guid kept, is said once as `Fact::Gap`, which the routing
  logs and nothing else (2.4), and the installations are read again; the
  client's polling finds the rest.
- **An installation's rows** read the installations again, and come up
  as `Event::Installation` (2.4); a row that hints is read whole within
  `delivery_bytes` and goes up as a Forgejo delivery's hint does, the
  domain dropping its own echoes and what it has not adopted, as above.
  Any other row is passed over, unread.
- **A `401`** on the log stops the forge, its key refused (2.3); any
  other refusal ends that poll, and the next asks again.

## 6. The load rules, and where each is kept

A rule kept in skein's crates points to the section that keeps it. The
values are temper's: `../domain/forge.md`, 5.2 and 5.4.

| Rule | Kept by |
|---|---|
| No listing without a filter on a periodic path; no history | the client's one keep-up listing per repository, always `since` (skein's `forge-client.md`, section 4; the listing, `forge.md`, 5.2); the slow pass reads live resources one at a time (`forge-client.md`, section 4) |
| Calls per pass do not grow with history | one listing per pass, nothing listed by label (skein's `forge-client.md`, section 4); checked by temper's cost checks (10) |
| Cache by change token | comments and reviews by revision (skein's `forge.md`, 5.3); repositories' paths and users cached (`forge-connection.md`, section 4); a pull request read on a backoff (`forge-client.md`, section 4); a read given a validator is conditional, and its `304` is free (2.7), as each poll of a GitHub App's log (5.1) |
| One path to CI; never a closed pull request's | statuses of the head only (skein's `forge.md`, 5.2); a closed item leaves the working set when its owner stops naming it (`forge-client.md`, section 3; temper's top, `../domain/forge.md`, 5.1) |
| API only; `page` and `limit` always; the forge's cap | skein's `forge.md`, 5.1 |
| Contents only when a decision reads them | a pull request's files and a comparison, read for a brief or an overlap, each file's reads charged (skein's `forge.md`, 5.2; `forge-client.md`, section 6) |
| Webhooks: hints, answered first, merged, targeted, echoes dropped | skein's `forge.md`, section 7 and `forge-connection.md`, section 6; the client merges hints into one earlier pass (`forge-client.md`, section 4); temper's domain drops its own echoes, by key and by pusher (`../domain/forge.md`, 5.2); a GitHub App's log read every `log_poll`, conditionally (5.1) |
| Closed items by deltas, not sweeps | the keep-up listing's `since`: a closed item is listed only when it changes, and its owner stops naming it (skein's `forge-client.md`, sections 3 and 4) |
| Backoff and jitter; honour the rate; no blind retries of writes | the client's backoff and budget (skein's `forge-client.md`, sections 4 and 6); `RateLimited { after }`, Forgejo's `429` and GitHub's pauses alike, and no retry in the component (`forge.md`, 5.4; `forge-github.md`); each answer's `rate`, the least of which the budget keeps (2.7); keyed creations found again after a `Timeout` (`forge-client.md`, section 7) |
| Slow cadences; webhooks for latency, polling for liveness | the client's `poll`, `hinted` and `slow`, unchanged by webhooks (skein's `forge-client.md`, section 4); a forge with no hook, or on another host, polls alone (section 5); a GitHub forge polls as a Forgejo one does, its log's hints only cutting the latency (5.1) |

Nothing the engine writes changes often: it writes the forge only to
make an effect (`../domain/forge.md`, section 6), never as a heartbeat.

## 7. Entities, limits and the worst case

Held by skein's `forge-connection.md`, section 7. temper's values are
its `limits` member (configuration.md, section 7), set in
`temper-service`'s profile `small`. By name, those this layer adds:

- **temper's own:** `records.value_bytes`, the largest record or carried
  value, checked against jig's secrets' store at start (2.1);
  `forges.installations`, the installations a GitHub forge accepts, at
  most the component's `installations` (2.4); `forges.log_origins`, at
  most 4 hosts per forge (3.6).
- **The component's, for GitHub** (skein's `forge-connection.md`, 7.2):
  `installations`, `log_origins`, `key_bytes`, `renew_before` (10 min;
  2.5), `log_poll` (5 s), `log_catch_up` (24 h), `log_pages` (10) and
  `delivery_bytes` (5.1). The component has no defaults: each is
  temper's, named in its limits module and given once at
  `Component::new` for every forge, none per forge and none in
  `forges[]`. `log_catch_up` bounds only a later start of a forge
  within one process.
- **The setup's host:** `setup.states`, `setup.state_life_ms` (1 h) and
  `setup.runs` (2.6); its `Reachable` plan's `reachable` is the
  picker's `picker.page`.
- **The box's link** (2.8): `box.line_bytes`, its one line read and
  its one line written, and one ask.
- **Beside this layer,** the clients' listener's `routes.query_bytes`
  and the picker's `picker.page` (100, GitHub's page), which bound the
  GitHub routes and the picker's reads (client/README.md, the routes and
  the picker; `../domain/forge.md`, 4.3).

The worst case grows by a GitHub forge's: its accepted installations'
tokens and their renewals, one log poll in flight, and one setup run.

## 8. State machines

Held by skein's `forge-connection.md`, section 8: a forge connection
(8.1), an installation's tokens (8.6) and the delivery log's puller
(8.7) among them; the setup's runs are skein's `forge-setup.md`'s.

### 8.1 A forge's connection, as the routing tells the root

| State | Entered on | Its calls |
|---|---|---|
| not connected | start with no access, an incomplete, unreadable or mismatched set (2.1); a setup run's `Stop` on a forge with none | `NotConnected` |
| not set up | start, an owned forge whose setup says `"start": "asked"` with any of its files absent (setup.md, 6.5) | `NotConnected` |
| setting up | the box's link's `start` written, then each `step` line, with its step (2.8) | `NotConnected` |
| set up failed | the box's `failed`, or its pipe closed during the ask (2.8) | `NotConnected` |
| connecting | a run that connects begins (2.6) | `NotConnected` |
| connected | `Event::Connected` (2.2) | served |
| stopped | `Event::Stopped` (2.3) | `NotConnected` |
| ended | the demo's end, or a start whose `Demo` record reads ended: the demo's forge alone (setup.md, 8.4) | `NotConnected`, by the routing, which passes none on to the connection component |

A connecting forge goes back to what it was when its run stops; a
stopped one is connected again by its next `Connected`, after a Replace.
A setting up forge, at the box's `started`, connects as 2.2 says; set up
failed, it is asked again from the client, as not set up is. Only an
owned forge whose setup says `"start": "asked"` is ever not set up,
setting up or set up failed. An ended forge stays ended: the routing's
events for it change nothing, the setup host runs nothing for it, a
webhook's hint for it is dropped, and no list shows it
(`../domain/forge.md`, 4.1).
The state is the routing's and the root's, in memory, and is rebuilt at
each start from the records and the events (`../domain/forge.md`,
4.1).

## 9. The fake forge's protocol layer

Held by skein's `fake-forge.md`, section 4, for Forgejo's dialect, and
section 10, its GitHub layer, for GitHub's (with skein's
`forge-github.md`): its App, installations, installation tokens, merge
queue, delivery log, job log redirect, rate windows and pauses, its
fixed test keys, and its two browser pages answered as redirects, so
the setup's routes run end to end with no browser.

temper-fake-git's counted ids use skein's `fake-forge.md`, section 3's
shared 20-byte form: `mix(count)` big-endian (the SplitMix64 finalizer),
the count big-endian, then four zero bytes, written as forty lowercase
hex digits. Its encoder is `skein-fake-forge-domain::object_bytes`; its
count reader uses `skein-fake-forge-protocol::routes::commit`, which
validates the whole form. An absent previous head in a fetch keeps git's
all-zero marker; count zero has its own mixed id. Stored little-endian
counts are not ids.

## 10. Testing

The tiers, the fakes and the layout are `../domain/testing.md`'s (sections 2,
4.2 and 7). The kit's own tests are skein's. The forge's, in temper:

- **Step tests** of the documents, the encoders and decoders, the call
  plans and the marker: skein's `forge.md`, section 11.
- **The machine world** and the protocol world of the component and the
  fake forge: skein's `forge-connection.md`, section 10.
- **temper's protocol world,** `tests/engine/protocol-forge`: temper's
  forge component, which is `skein-forge-connection` handed temper's
  values, and the fake forge's protocol layer, skein's, joined by
  in-memory streams cut at random, running the forge connector world's
  stories (`tests/engine/forge`, `../domain/forge.md`, section 18)
  through bytes: a change produced, opened, checked, queued and landed;
  a creation and a merge whose answers were lost, found after a restart
  and never made twice; a failed head repaired and checked again; a held
  branch moved by another hand, reported as drift; on GitHub, a change
  waiting out 90 seconds of CI without delivery hints, landing within
  the pull keep's maximum poll after its verdict, with request costs
  bounded by the keep's paces and App log reads counted apart. Its
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
  connection as section 2 has it:
  - **at start:** a forge with no rows, an incomplete set, an unreadable
    row and a set made for another forge each start not connected,
    named, nothing erased, while every other forge serves; carried rows
    found at start are erased on the core's decision;
  - **connecting at runtime:** a run's writes kept before
    `Request::Connect`, the access built from the records, `Connected`
    told to the root with the identity, and calls `NotConnected` at cost
    0 until then;
  - **one forge stopped:** its key refused, it answers `NotConnected`,
    is named with Replace, and the engine and the other forges run on;
  - **installations:** an `Accept` kept before `Request::Installations`,
    a deletion rewriting `Accepted`, a suspension's calls `Forbidden`,
    and `NotInstalled` at cost 0;
  - **generations:** strictly increasing per identity, the value kept in
    the value table before `Event::Credential`, a push across a renewal
    ending on its value, and generation 1 at each start;
  - **pasted values:** a digest that differs refuses the run, "the
    pasted value changed"; the carried row erased when its run ends,
    done or failed, its items kept first on `Done`;
  - **the box's link** (2.8): its machine's step tests, lines in and
    out: an ask written, another refused "busy" until `started` or
    `failed`, and every ask refused "not under the box" with no pipe;
    each line for the forge asked; a line for another forge, an unknown
    word, step or number, each named and dropped; a line at
    `box.line_bytes` read and one byte past it named and skipped; the
    pipe closed during an ask. Then the routing with a scripted box on a
    pipe pair: an ask, `start 3`, `step 3 prepare`, `failed 3 prepare 3
    exited 1`, the ask again and `start 3` again, a third refused
    "busy", a stray `started 9` dropped, and the root told each state;
    and the service's step world (setup.md, 6.5): at `started 3`, the
    forge connecting, the handover read and `TakeOver` run, its record
    kept before the file's removal, `Reconcile` with the root's restored
    list, then `Request::Connect` from records and the git value put at
    its generation before `Event::Credential` and `Request::Connect`; a
    `TakeOver` stop leaving the file and the forge set up failed; a
    failed removal blocking `Reconcile`; and the next ask, answered
    `started 3`, taking the handover over again, removing it and
    connecting;
  - **GitHub's three routes** on the clients' listener (the manifest
    page, the redirect and the setup address; `../client/forge.md`,
    9.1): requests through `fixtures::http::Peer` to the service's
    routes, each answer and its headers, and a state used, unknown or
    past its life refused (`../domain/testing.md`, 2.1).

  The stories through temper's real loops, connecting, adopting and
  landing on each dialect, are `../domain/testing.md`'s (2.4).
- **No test calls github.com.** GitHub's dialect runs against skein's
  fake alone. An App's key is one of the fake GitHub layer's fixed test
  keys (skein's `fake-forge.md`, section 10.1: 2048-bit, made for the
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
  throwaway Forgejo it runs against (`../domain/testing.md`, 2.6): skein's
  `fake-forge.md`, sections 6 and 7.
- **The pin** is skein's (`forge.md`, 4.3): one patch release of Forgejo
  v17.0 and one of forgejo-runner v13.2, constants the throwaway
  Forgejo, the conformance and owned mode share (setup.md, 6.1); a
  binary of another release is refused, naming it. A later patch changes
  skein's constants and records, and not this document.
- **The process tier** runs the forge component's start, polling and
  webhooks against an owned Forgejo (`../domain/testing.md`, 2.6; setup.md,
  13.4).

## 12. What others owe

The boundary, the configuration and the setup's hooks are sections 2
and 5's. What an owner owes the component is skein's
(`forge-connection.md`, section 9). The forge client
(`skein-forge-client`, owned by temper's forge connector,
`../domain/forge.md`, section 4) owes the component:

- **`Read::Locate { owner, name }`,** answered `Answer::Located {
  repository }`, addressed with the forge's number and repository 0
  (skein's `forge.md`, 5.2); its answer's shape checked as every other's,
  a located id never 0, and its owner and name counted in the call's
  bytes.
- **Costs,** as the boundary carries them: `Event::Answered { call,
  cost, free, rate, result }`, `cost` the requests the call wrote, cache
  fills included, 0 for none and for an answer given at once
  (`NotConnected`, `NotInstalled`); `free` its `304`s; `rate` the
  window the forge last reported, if any. The budget charges `cost` less
  `free`, and keeps the least `rate` (`../domain/forge.md`, 5.4; skein's
  `forge-client.md`, section 6).
- **The writer,** given by the connector at each `Connected`, since a
  forge connected at runtime has no writer in the configuration (2.2).
  The forge client keeps it as its own record, `writer(d, env, Writer {
  forge, author }, out)`: `Stored::Writer` under `Key::Writer(forge)`,
  in temper's small store under its tag 16 (`STORE_VERSION` 3), restored
  at start, so a restored effect never meets `Forbidden` (skein's
  `forge-client.md`, 7.1).
- **Hints,** as the vocabulary has them: `Hint { repository, change, key
  }`, with the pusher on `Change::BranchMoved`, so the domain drops its
  own echoes (section 5; skein's `forge.md`, section 7).
- **Shared limits:** the client's `calls`, `rows`, `answer_bytes`,
  `op_bytes` and `lifetime`, and the forge's `repositories` and
  `name_bytes`, are given to the component too, which sets and checks
  its own by them (skein's `forge-connection.md`, section 7).

jig owes the routing its secrets' store under temper's tag (`Keep`,
`Erase`, `Get` and `Load`, each `Keep` answered once durable), and the
request carrying a value (credentials.md, pasted values; jig's
`protocol/client.md`): the value, at most 4 KiB, kept at decode under a
number jig gives it from 2^63 up and keeps the next of in a counter
row, only `{ number, digest }` reaching the core; and the core's one
output naming each carried value it leaves unused, which temper's root
forwards to jig's clients' component to erase.

## 13. What temper relies on

Held by skein's `forge.md`, section 9: what the component and the forge
connector rely on of Forgejo, on the pinned release (section 10), each
named by its fact in the conformance's `facts.txt` (skein's
`fake-forge.md`, section 7). What no fact records yet is open: the kit's
in skein's `forge.md`, sections 9 and 12, temper's own in section 14.
temper's part:

- **What it settles after it delivers.** temper relies on neither order
  of a delivery and the state it settles, only on each settling within
  `settle` of the move, 30 s in the profile `small`
  (`../domain/forge.md`, 5.2).
- **The setup's, and git's.** What the setup relies on is skein's
  (`forge-setup.md`, section 8; setup.md, 13.3); what git relies on of
  Forgejo (a renamed repository's redirect, a protected branch's refusal
  of a push) is git.md's (section 3 and 4.8), named in
  `tools/git-transcripts/facts.txt`.
- **GitHub's,** each a named fact of skein's `forge-github.md`, which
  temper's tests run against skein's fake alone (section 10): the App's
  installations and their tokens' hour, the delivery log and its
  validators, a job log's redirect, the rate windows and their pauses,
  `404` for a repository an installation did not select, and the merge
  queue's answer (`../domain/forge.md`, 9.1). No test of temper's
  checks them against github.com: a fact skein's fake gets wrong is
  skein's to correct, and temper's stories follow.

## 14. Open questions

- **The kit's own** are skein's: `forge.md`, sections 9 and 12;
  `forge-connection.md`, section 11; `forge-client.md`, section 11. They
  hold every question and every unrecorded fact about the vocabulary, a
  dialect or the component, among them how long threads are read, a
  dismissed review, contents on demand, a file too large, bodies past
  `body_bytes`, an edit within a creation's second, the merge style, an
  expired or revoked token, what comments page, and a job no runner took.
- **What no fact records yet and temper's domain relies on,** read from
  Forgejo's API description and sources alone (section 13):
  - that the push workflow an update starts runs on exactly the head
    the update made, so CI is read again there (`../domain/forge.md`,
    section 10);
  - that a merge after its pull request's base was changed lands in the
    new base: the pull request is read afresh before a merge, and one
    retargeted since the decision is not merged, but nothing makes the
    merge itself conditional on its base (`../domain/forge.md`, section
    6).
- **GitHub's dialect and setup's open questions** are skein's
  (`forge-github.md`; `forge-setup.md`, GitHub's plans). temper's:
  - **A job log's origin learned at runtime.** Default: `log_origins` is
    resolved at start; another origin is named (`Fact::Origin`, 3.6),
    and the operator adds it and restarts.
  - **A Forgejo secret with no listener.** Default: handed none, the
    forge polls alone, named once in the log (section 5).
  - **Rows of a forge the configuration does not name.** Default: left
    as they are, named once in the log at start; erasing them is
    disconnecting, later (2.1).
  - **Generations across starts.** Default: generation 1 at each start,
    while no worker outlives its engine (2.5; credentials.md, its open
    questions).
