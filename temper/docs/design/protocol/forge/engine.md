# The forge in the engine

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

- **The kit's crates** are skein's (`forge/README.md`, section 2). `skein-forge`
  depends on skein-lib and skein-json and holds nothing of temper's
  domains; `skein-forge-client` depends on skein-lib and `skein-forge`'s
  vocabulary; `skein-forge-connection` depends on the crates below it in
  the diagram. `skein-forge-setup` is the setup's: its host component
  runs in the engine beside the forge component (6). temper runs the
  plans there alone: no command of temper's drives one (../setup/reconcile.md and ../setup/README.md, section 7
  ).
- **The forge component,** `skein-forge-connection`, is a child of the
  engine's protocol layer, beside jig's components, with its own limits,
  deadlines and kernel submission edges; the routing
  carries what passes between it, io and the root, and decides nothing
  (../components.md and ../README.md, section 4; jig's `protocol/service.md`, section 2).
  One component serves every kept forge. What it holds is skein's
  (`forge/connection.md`, sections 3 to 7): the calls in flight, the
  forges' connections, each forge's start, the repositories' paths and
  the users, a GitHub App's installations and tokens, the translation
  between the forge client's vocabulary and the documents, the webhooks'
  listener and a GitHub App's delivery log.
- **What the routing hands the component.** The component reads no file
  (skein's `forge/connection.md`, section 3), and no forge's credential
  is a file: each is a record in jig's secrets' store, which the
  routing, `temper-service`, loads (1). For each forge kept in the
  store (1), a `Forge { number, kind, address, transport, access,
  log_origins }`, added as its record is kept (2) and dropped as it
  ends (3):
  - the forge's number, and its kind, Forgejo or GitHub, which names the
    dialect;
  - its API's address and transport, its record's: plaintext to a
    socket address on loopback, or TLS with its server name and the
    machine's roots; GitHub's are github.com's unless the configuration's
    machine names a fake (../configuration/engine.md);
  - its access, `Some` or `None` (1): the credential, the user it must
    stand for, and a Forgejo forge's webhook secret when it has one;
  - its `log_origins`, the origins a job log's redirect may reach, its
    kind's (`README.md`, 2.6).

  And `application.webhooks`, the listener's address, when configured.
  And temper's two values of its own: the word `temper`, which marks its
  keys and its bot (`README.md`, section 3), and the user agent, `temper/0.0.0`. And
  the limits: temper's `limits` member (../configuration/limits.md),
  and the values the component shares with the forge client (`README.md`, section 10
  ).
  The setup's host gives its plans the same word (../setup/reconcile.md),
  and their requests carry the same user agent. The component reads
  nothing else of a forge's record: its `git` base is the routing's, for
  runs' remotes (../channel.md, section 4), its `web` is the setup's, the
  client's and the sign-in page's (its browser pages, 1), and its bot
  is its git identity's, which the component never holds (5).
- **What crosses to the root** is the root's boundary: a
  `Request::Forge { call, repository, op }`, which the routing hands to
  the component, ended by exactly one `client::Event::Answered { call,
  cost, free, rate, result }`, which the routing hands the root as
  `Event::Forge` (the forge's `Event::Client`); the forge's `Event::Hint
  { hint }`, which no request causes: a verified delivery's (`README.md`, section 4;
  `../../domain/forge/keeping-up.md`, 2); and each forge's connection, which the
  routing tells the root as it changes: connected and as whom, stopped
  and why, and a GitHub installation's changes (2 to 4).
- **Room, and the drain,** as the component keeps them (skein's
  `forge/connection.md`, section 2) and as jig's components keep them
  (jig's `protocol/service.md`, section 2; jig's `domain/engine/restart.md`,
  1): an answer or a hint the root cannot take yet is held and offered
  again, at most one per call and one per webhook connection, and
  nothing more is read off its connection meanwhile. One the root
  returns during its drain (its `Request::Busy`) is never offered
  again: it goes when the component closes, and the next process reads
  afresh and settles its outbox (`../../domain/forge/README.md`, section 9). The
  routing's own news of a connection or an installation that the root
  returns during its drain is dropped. The root's view of them is not
  kept, so the next process's root starts from what a fresh start
  publishes: each connection afresh, and the installations by the
  baseline listing (skein's `forge/connection.md`, section 4.4). The
  routing closes the component after the root's drain (jig's
  `protocol/service.md`, section 5); calls in flight end with their
  connections, and no answer goes up once the root has stopped.
- **A start that fails for good stops that forge only** (3): a user
  other than the forge's own, a credential refused, or a `4xx` that
  asking again would not mend (skein's `forge/connection.md`, sections 3
  and 8). The engine and every other forge run on.

## 1. A forge's records, and its access at start

- **A forge is a record of the forge connector,** as its repositories
  are: a `Forge` row of temper's store under its number (../store.md,
  section 2), holding
  - its kind, Forgejo or GitHub, and its mode, connected, managed,
    owned or fake (../setup/README.md, the modes);
  - its place: its API's host, port and TLS name, none for plaintext to
    loopback; the base of its remotes, `git`; and its browser pages,
    `web`: GitHub's, and a Forgejo's where a connect gives them apart
    from `git` (`../openid-provider.md`, 7);
  - its git program, the executable workers run for its repositories,
    or none for their own (../configuration/README.md, section 3): a Forgejo's as
    its admin's `Connect` gives it, GitHub's the machine's
    (../configuration/engine.md); a fake forge runs the machine's fake
    `git`;
  - whether a pasted GitHub App is another's, read-only (../setup/github.md,
    7);
  - its bot's git identity: a login, and the name and email it commits
    as, read on Forgejo alone: a connected Forgejo's login is the user
    its tokens answer as, written over the one the record was made with
    at each `Check` done (2); a GitHub forge commits as its App's bot,
    from its `AppSlug` and `User` records (5);
  - for a first set up, its intent, and no project (../setup/owned.md, 7):
    `Empty`'s organization, name and runner label, kept so that a restart
    before the repository is made makes it at the next connect; `Copies`'
    addresses, kept for the client to ask them at Ready and after a restart;
    or `Nothing`;
  - its link to a box's person, or none: the forge's user who is the box's
    one person, and that person's party, `{ user, person }`, each never 0
    (`../../domain/forge/adoption.md`, 6);
  - its state: kept, or ended, with whether its secrets are erased and
    its box group removed (3).

  Nothing in it is secret: a forge's credentials are its rows in jig's
  secrets' store (below). A managed, owned or fake forge's setup entry
  is made from the record and, for an owned or fake one, the machine's
  part of its kind (../setup/init.md).
- **Numbered next, never reused.** The forge connector gives a new
  forge the number after the last it gave, kept in its `ForgeNumber`
  row (../store.md, section 2), and never gives a number twice, since the
  number keys every record of the forge: its rows in jig's secrets'
  store, its repositories, its writer and its git identity
  (../credentials.md, 2.1 and section 4). A removed forge's number stays
  spent; a forge set up again is a new forge.
- **Its users among jig's parties.** A party adoption knows from a forge
  is keyed by that forge's number and the user's (../store.md, section 2;
  `../../domain/forge/adoption.md`). As a number is never given twice, such
  a key stays that forge's user's, even after the forge is removed.
- **Restored, then handed over.** The forge's restart step restores
  each `Forge` row (../store.md, section 5), and the connector hands each
  kept one to the routing, as it hands a new one (2), which adds its
  forge to the component, its identity to the identities' table and an
  owned or fake one to the setup host. An ended one's removal resumes
  instead (3).
- **One load, before the component starts.** The routing reads temper's
  rows of jig's secrets' store (`Request::Secrets`, `Load` under
  `SecretOwner::Application`, paged by number; jig's
  `protocol/store/secrets.md`) and groups them by forge. A row's
  number is `family << 48 | forge << 16 | item`: family 1 is a forge's
  records, `forge` the forge's number, and `item` what the
  row holds: 1 `AdminToken`, 2 `ApiToken`, 3 `GitToken`, 4 `User`, 5
  `WebhookSecret` (Forgejo's alone), 6 `AppId`, 7 `AppSlug`, 8 `AppKey`,
  9 `Accepted` and 10 `Origin`, codes that never change meaning
  (../credentials.md, the forges' records). `User` is the forge's user
  temper acts as: on Forgejo temper's user, on GitHub the App's bot
  user. temper's numbers keep the top
  bit clear, below 2^63, where jig numbers carried values under
  temper's tag itself (6).
- **Each item's bytes** are as skein's setup host writes them, pinned by
  skein's `forge/setup/README.md`, 4.4: tokens and secrets as text; `User` and
  `AppId` a `u64` little-endian, never 0; `AppSlug` text; `AppKey`
  PKCS#1 DER; `Accepted` the installations' ids, each a `u64`
  little-endian, ascending and distinct. `Origin` is temper's, the
  forge's kind and `api` address the records were made for, written at
  each connect. They are decoded with skein's item decoders only where
  they build an access.
- **The access** (skein's `forge/connection.md`, 3.5):
  - Forgejo: `Access { credential: Token(ApiToken), user: User,
    webhook_secret: WebhookSecret }`, the secret only when its row is
    there and `application.webhooks` is configured (`README.md`, section 4);
  - GitHub: `Access { credential: App { id: AppId, key: AppKey }, user:
    User, webhook_secret: None }`, since a GitHub forge has no secret
    (`README.md`, 4.1), then `Request::Installations { forge, accepted }` from its
    `Accepted` row (4).
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
- **Rows of a forge with no record** are left as they are, and named
  once in the log (`README.md`, section 12); a kept forge's are erased by its
  removal (3). Carried rows, pasted values whose run a restart cut, are
  loaded from 2^63, where jig numbers them; the core decides their
  erase, which the routing carries out (6).
- **A managed, owned or fake Forgejo starts connecting:** after the
  records' load and the root's restore, its access stays `None` until
  the setup host's `Reconcile { forge, adopted }` is done and its writes
  kept, then `Request::Connect` (2). `adopted` is the root's restored
  repositories for the forge; a fake then runs its start-time adoption
  before connecting (../setup/playground.md). For owned and fake, the engine first takes
  the handover whenever its file is present, even with a kept
  `AdminToken`, removes the file after `TakeOver`'s `Done` and the
  record kept, and reconciles only after removal succeeds (../setup/owned.md,
  5). With no file it reconciles from the kept record. A stop or
  failed removal leaves that forge not connected, named by the step,
  never a secret; every other forge runs on. A connected Forgejo and a
  GitHub forge start with their access.
- **An owned forge is set up from the client** (../setup/owned.md, 5), and
  starts not set up: with neither the handover file nor an `AdminToken`
  record, its access is `None` and nothing runs for it until the box
  answers `started` on its pipe (2). Then the engine runs the same
  handover, removal, reconcile and connect sequence as at a start, with
  access and git value from records.
- **The mode is the record's** (../setup/README.md, the modes), as the act that
  made the forge gave it (2): owned by `SetUp`, fake by the
  playground's start, managed by a `Connect` as a Forgejo's admin, and
  connected by a `Connect` with a Forgejo's tokens or a GitHub App.
- **Checked at start,** each refusal named (../configuration/README.md, its
  starts): `records.value_bytes` against the largest value jig's
  secrets' store takes. **Checked at a GitHub `Connect`:** a fixed
  address for the clients' listener, since the App's `redirect_url` and
  `setup_url` name it and the API cannot change them; with port 0 the
  connect is refused, naming `application.clients.listener`.

## 2. Connecting at runtime

- **A forge is made by the act that sets it up or connects it**
  (`../../client/engine.md`): `SetUp` and `Connect` name a kept
  forge by its number, or ask for a new one, and the forge connector
  makes its record (1) in the act's decision, numbered next, once the
  act is admitted, so a refused act makes nothing:
  - `SetUp` of a new owned Forgejo, under the box alone and while no
    owned forge is kept, takes the machine's owned place
    (../configuration/engine.md), and keeps its intent, its row saying
    it (../setup/owned.md, 7);
  - `Connect` of a new GitHub forge takes the machine's GitHub place,
    github.com's when it names none, and its git program;
  - `Connect` as a Forgejo's admin (../setup/reconcile.md, 5) takes the address,
    the remotes' base, the pages and the git program the person gives,
    its mode managed, and the first repository's intent when it names
    one;
  - `Connect` with a Forgejo's bot's tokens (../setup/README.md, section 4) takes
    them as an admin's does, its mode connected, with no first
    repository;
  - the playground's start makes its fake forge on the machine's fake
    place (../setup/playground.md).

  A forge past `forge.forges` (`README.md`, section 6) is refused busy. The record is
  then handed to the routing, as a restored one is (1): its forge
  added to the component with no access, its git identity to the
  identities' table, and an owned or fake one to the setup host, until
  the act's run or the box's group connects it.
- **`Request::Connect { forge, access }`** hands a running forge its
  access: a first connect, a replacement and a reconnect after a stop
  alike (skein's `forge/connection.md`, 3.5). The routing sends it after
  a setup's run is done and its writes are kept (6): a managed or
  owned Forgejo's `Reconcile`, at start or after the box's `started`, a
  connected Forgejo's `Check`, GitHub's `Convert` and `Paste`, and each
  `Replace`. The access is built from
  the records, as at start, never from the run's outcome: a forge
  connected at runtime is the forge a restart would load.
- **`Event::Connected { forge }`** reaches the root through the routing
  with the identity the records hold, its id and login (the `User` row
  and, on Forgejo, the record's bot; on GitHub, the App's bot user): the
  forge is connected, and the
  deployment's page says as whom (../../client/forge/README.md, the deployment's
  page). The connector then gives the forge client its writer, which the
  forge client keeps as its own record and restores at start (skein's
  `forge/client.md`, 7.1; `../../domain/forge/adoption.md`, 1). A GitHub forge's
  `Reconcile { forge }` runs next, once the forge's writing run is free
  (6); it ends in no connect.
- **The box's pipe** (../setup/box.md, 1), under the box alone, through the
  routing's box link (8): at `SetUp { forge }` (../../client/engine.md
  ), admitted by the root, and at the playground's start, the
  routing writes the group's file and the link writes `start FORGE`, one
  ask at a time, and refuses it "not under the box" with no pipe; it
  tells the root each `step` line, the forge setting up and at which
  step; at `started FORGE`, the forge's `TakeOver` when the handover
  file is present, then its `Reconcile`, then `Request::Connect`, with
  the handover removed before reconcile and access built from records
  (../setup/owned.md, 5); at `failed FORGE WHY`, the forge set up failed, with
  `WHY`. An answer for a forge it did not ask for is named in the log
  and dropped.
- **Until then** each call on the forge is answered `NotConnected` at
  once, at cost 0 (7). The root shows the forge connecting while a run
  holds it, setting up while the box runs its group, and not connected
  otherwise.
- **A connected Forgejo's `Check`** (6), at its `Connect` and at each
  `Replace`, ends `Done` with the user its tokens answer as, its id and
  login. The routing keeps the run's `User` write, then each pasted
  item, the tokens and the secret, then the `Origin`, and tells the root
  the login with the run's end. The forge connector writes it as the
  record's bot login in that decision and hands the record to the
  routing again (1), before the carried row's erase and
  `Request::Connect`, so the `Connected` that follows names it and a
  restart reads it from the record. A login renamed on Forgejo is the
  same user, by its id: the next `Check` keeps the new one.
- **A forge's user stays.** A connected Forgejo's `Check` compares the
  user its tokens answer as with the kept `User`, on a forge connected
  again as on a `Replace`: tokens of another user stop the run
  `OtherUser`, nothing kept and the forge as it was. A forge's writer is
  its user (skein's `forge/client.md`, 7.1; ../credentials.md, 2.1), so
  another user's tokens connect a new forge, once this one is removed.
- **`Replace`** runs `Check` on a connected Forgejo, its pasted tokens
  read before the forge's rows, so that a token not pasted is the kept
  one, and `Paste` on GitHub with the kept `AppId` (../setup/github.md, 3); on
  `Done` each pasted item is kept over the old, then `Request::Connect`.
- **A replacement** ends the calls the old access began `Forbidden`, as
  a revocation would. A credential that answers as another user is
  refused before anything is kept (`Check`, `Paste`); one the component
  refuses stops the forge (3).

## 3. A forge stopped, or removed

- **`Event::Stopped { forge, why }`** stops that forge only: its key or
  token refused, a user other than its own, an access the component
  finds invalid, or a `4xx` that asking again would not mend (skein's
  `forge/connection.md`, 3.3 and 3.5). Its calls are answered
  `NotConnected` until its next connect.
- **Named to the root,** which shows the forge and why on its row, with
  Replace (../../client/forge/README.md, the deployment's page). The engine and every
  other forge run on; nothing is erased, and the next start tries the
  same records.
- **The webhooks' listener** stays the engine's: one that fails stops
  the engine (`README.md`, section 4).
- **A forge removed** is ended in one decision of the forge
  connector's, by its owners' `RemoveForge { forge }` or the
  playground's reset (`../../domain/forge/adoption.md`, 1; ../setup/playground.md): its
  record reads ended, and the root asks the routing for the service's
  side at once:
  - **no call served:** the component drops the forge (`Remove {
    forge }`), its calls in flight ending with it; the root sends it
    none, and one the routing meets is answered `NotConnected` (`README.md`, 7.1);
  - **its access dropped:** its endpoint and any access built for it,
    its git identity's values in the identities' table, its grant
    withdrawn (../credentials.md, section 4), and its place in the setup
    host, whose run for it ends;
  - **its secrets erased at once:** each of its family-1 rows,
    `AdminToken` through `Origin` (../credentials.md, 2.1), one `Erase`
    after another under `SecretOwner::Application`, each durable before
    the next, a row that is absent among them, since a run may have had
    a write in flight; then the root is told (`SecretsErased`) and the
    record keeps that step done. A removal restored at start erases
    once the start's store mark is durable;
  - **its writer and its flights** go in the decision itself: the
    writer's row erased, and its writes in flight withdrawn
    (`../../domain/forge/adoption.md`, 1);
  - **its box group removed:** an owned or fake forge's, by `remove
    FORGE` on the box's link (8), the step done at `removed FORGE`; a
    connected or managed forge has none, and the step reads done in the
    decision.

  On the forge itself, nothing is undone but what temper ran. **Kept
  until whole:** at each start the connector asks again for each step
  its ended record has not heard done; once both are, and no kept task
  names its repositories, the record is erased, its number spent (1;
  `../../domain/forge/adoption.md`, 1).

## 4. GitHub's installations

- **The accepted installations** are the forge's `Accepted` row: the
  whole list, at most `forges.installations` ids, within the
  component's `installations`. One is accepted when the person comes
  back through the setup URL with a state of temper's (`Accept`, 6),
  or when the deployment's owner accepts one that exists already,
  through the tools, with no state (`../setup/github.md`, 4).
  The routing hands the list over with `Request::Installations { forge,
  accepted }` at start, after an `Accept`'s `Done` and after a deletion;
  the component reads the installations and mints a token for each one
  accepted (skein's `forge/connection.md`, 4.4 and 5.1).
- **`Event::Installation { forge, installation, account, change }`,**
  an `account` being its id and login:
  - `Created`: nothing, unless accepted, at the setup URL or by the
    owner's accept. One never accepted is a stray: shown on the
    deployment's page, and uninstalled by reconcile after
    `setup.state_life_ms` (skein's `forge/setup/README.md`), unless the App is
    read-only (`../setup/github.md`, 7).
  - `Deleted`: the `Accepted` row is rewritten without it and kept, then
    `Request::Installations`. The git identity's value loses that
    account's token (5). Every adoption on the account is marked
    (`../../domain/forge/adoption.md`, 3).
  - `Repositories` and `Renamed`: to the root. No names come with them:
    the connector lists that installation again, through the setup's
    host's `Reachable` plan (6), and marks each adoption whose
    repository left the installation (`../../domain/forge/adoption.md`, 3). Each
    `Connected` lists again every accepted installation that holds an
    adoption, so a change made while temper was away is found. The marks
    are held in memory, never stored.
  - `Suspended` and `Unsuspended`: to the root, shown on the deployment's
    page. A suspended installation's calls are answered `Forbidden`, and
    its git token leaves the identity's value until it is unsuspended;
    its adoptions are marked until then.
- **`Event::Fact { forge, fact }`** decides nothing: it is the
  component's diagnostic for the owner's log (skein's
  `forge/connection.md`, 2.1). `Fact::Unkept`, an installation's change
  the component had no room for, and `Fact::Gap` (`README.md`, 4.1) are logged only.
  `Fact::Origin` (`README.md`, 2.6) is kept as well, per forge in memory.
- **A call on an account no accepted installation covers** is answered
  `NotInstalled` at once, at cost 0; one on a repository its account's
  installation did not select answers `Missing`, as GitHub's `404`
  says (7).

## 5. Git's tokens, and generations

One git identity per forge (../credentials.md, git identities): its number
is the forge's, its login and author on Forgejo the record's bot (1)
and on GitHub the App's (below), and its value the forge's.

- **Forgejo's value** is the `GitToken` record's, its login the bot's
  (`User`): generation 1 at start, or at the first connect after the
  box's `started` when the start had none. Each later value takes the
  next generation of its number. The value is put before its
  `Event::Credential` and before `Request::Connect` (2, 8).
- **GitHub's value** is a list of `(account id, account login, token,
  expires)`, one per accepted installation, built from `Event::GitToken
  { forge, installation, account, token, expires }` and held in memory
  only (skein's `forge/connection.md`, 5.3). Its login is
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
  (jig's `domain/engine/README.md`, 6.1). A worker keeps the two newest, so a
  push in flight across a change ends on the value it began with.
  `valid` is Forgejo's for as long as the engine runs, and GitHub's the
  earliest `expires` among the value's tokens, less now.
- **Generation 1 at each start:** safe only because a worker never
  outlives the engine that started it (standalone). A fleet whose
  workers outlive it needs generations kept across starts
  (../credentials.md, its open questions).
- **The worker picks** the entry whose account id is the repository's
  owner's, which the workspace item carries; with none, the push is
  refused, named "temper's App is not installed on the account owning
  <name>" (../channel.md, section 6). The agent
  never sees a token: jig's hub forwards no application's credential to
  it.
- **The value's codec,** engine to worker, is temper's wire format,
  versioned, holding Forgejo's single token or GitHub's list
  (../credentials.md, git identities). It is never stored.

## 6. The setup's host

skein's in-process host component (`skein/docs/design/forge/setup/README.md`, its host), a child
of the engine's protocol layer beside the forge component. The routing
carries what passes between it, the root and the records.

The engine builds the setup host once at start, checking the host's
entrance limits and settings before it builds it, with room for
`forge.forges`; each kept forge joins it as its record is handed over
(1, 2) and leaves it at its removal (3). A full host answers `Why::Busy` at a Run's start, and the page shows it as busy, try again (skein's
`forge/setup/README.md`, 4.5).

- **Down,** `Run { run, plan }` and `Answer { run, answer }`; **up,**
  `Need { run, need }`, `Done { run, outcome }` and `Stop { run,
  failure }`. It holds an admin's token and a GitHub App's key only
  while a plan runs. temper has no use for skein's one-shot host
  (`skein/docs/design/forge/setup/README.md`, 4.2): `temper-box forge prepare` and `run` drive no
  plan (../setup/reconcile.md).
- **The plans the routing runs** (skein's `forge/setup/README.md`; ../setup/README.md):

| Plan | When | What it does |
|---|---|---|
| `Reconcile { forge, adopted }` | each start of a managed or owned Forgejo; a managed connect; the box's `started`, after its `TakeOver` (2) | converges the forge on what the deployment needs (../setup/reconcile.md), then the forge connects |
| `TakeOver { forge, admin_token }` | an owned or fake forge's start, or the box's `started`, whenever the handover file is present, even with a kept `AdminToken` (../setup/owned.md, 5) | keeps the handover's admin token; only after its `Done` and the record kept is the file removed, before reconcile |
| `Adopt { forge, repository }` | a managed or owned forge's adoption, before its reads | gives the bot access to the repository (`../../domain/forge/adoption.md`, 2) |
| `Create { forge, org, repository, files }` | an owned or managed forge's first repository, at its set up (../setup/owned.md, 7) | the repository made in its organization, the bot given access, the owner's files put on its main (skein's `forge/setup/forgejo.md`, 1.9) |
| `Check { forge }` | a connected Forgejo's `Connect` with its pasted tokens, and each `Replace` of them | both tokens answer as one user, not a site admin, and as the kept `User` when there is one, else it writes `User`; it ends `Done(Identity { id, login })`, the login kept as the record's bot's (2) |
| `Manifest { forge, target }` | GitHub's Connect, always for `Personal`: the App is the person's, public, and installs on any account GitHub offers them (../setup/github.md, 2) | a fresh state, the form's target and the manifest; it ends `Done(Manifest { state, target, manifest })`, from which temper renders the manifest's page (../../client/forge/served.md, 1) |
| `Convert { forge, code, state }` | GitHub's redirect back | the code exchanged for the App's id, slug and key, the conversion's webhook secret and OAuth pair never kept; the bot user found; it writes `AppId`, `AppSlug`, `AppKey` and `User` |
| `Install { forge }` | Install the App, or Install on another account | a fresh state, and the install link (`Link`) |
| `Accept { forge, installation, state? }` | the setup URL's visit, with its state; the owner's accept of a stray, with none (`../setup/github.md`, 4) | the installation checked, and added to `Accepted` |
| `Paste { forge, app }` | a pasted App's Connect; Replace on GitHub, with the kept id | the pasted key (skein's `Pasted`, read once, never written) decoded to DER, `GET /app` matching the id, the bot user found, the App's hook set to `https://<slug>.invalid/`, the permissions and events checked as reconcile checks them; it writes `AppId`, `AppSlug`, `AppKey` and `User` |
| `Reconcile { forge }` | after each GitHub `Connected` | the App's permissions, events, installations and key checked; it repairs only its hook's address (`README.md`, 4.1) |
| `Reachable { forge, installation?, page }` | the picker, on every kind | the repositories the picker lists, paged by number from 1, at most `reachable` rows (temper's `picker.page`), `more` set when the page is full: on GitHub one accepted installation's, `installation` its id; on a connected Forgejo what the API token can push to; on a managed or owned Forgejo every repository the admin's token sees, by id ascending, refused `Missing` or `NotAdmin` as reconcile's first step; an `installation` not in `Accepted` stops `Installation` (`Missing`), and one given for a Forgejo stops `Installation` (`Unexpected`); it ends `Done(Reachable { rows, more })`, each row the repository's id, its owner's `{ id, login }`, its name and whether it is private; it writes nothing (`../../domain/forge/adoption.md`, 3) |

- **`Reachable`** is skein's (`forge/setup/forgejo.md`, 1.8). On a managed or
  owned Forgejo it runs on the admin's token, which stays the setup
  host's; on a connected Forgejo, on the API token (skein's
  `forge/setup/forgejo.md`, 1.6); on GitHub, on a token minted for the page and
  dropped. Since it writes nothing, it runs beside the forge's one
  writing plan.
- **Outcomes.** A run ends `Stop` with its failure, or `Done`, which
  carries what temper serves or shows (skein's `forge/setup/forgejo.md`, 1.2):
  `Reachable`'s page for that plan; `Manifest { state, target,
  manifest }`, the state, the form's target and the manifest's JSON,
  from which temper renders the manifest's page (../../client/forge/served.md, 1),
  at most `page_bytes`, and holds it in memory with the state while it
  lives and serves it on
  the clients' listener (../../client/README.md, the routes); `Link`, the
  install link; each state held with the request's `back`, a
  connect's passing to the install its `Convert` starts, where the
  setup route sends the browser (../../client/forge/served.md, 1); or `Lacking`, each permission or event the App lacks,
  shown on the deployment's page and never refused (../../client/forge/README.md,
  the deployment's page).
- **The redirect and setup addresses** temper gives the plans are whole
  URLs, built from the clients' listener's address:
  `http://<listener>/forges/<n>/github/redirect` and
  `http://<listener>/forges/<n>/github/setup` (1, the fixed address).
  The manifest's `url`, the App's homepage, is `http://<listener>/`.

- **A run's needs.** `Need::Write { item, bytes }` is a `Keep` under the
  forge's family-1 number for that item, answered only on `Kept`.
  `Need::Read(item)` is served from the run's carried row first, by the
  number jig gave it, else from the forge's row.
- **Pasted values** (../credentials.md, pasted values): a `Connect` or a
  `Replace` may carry one value, temper's bundle of items. jig's
  clients' component strips it at decode and keeps it in the secrets'
  store under temper's tag before jig's core sees the request, at a
  number jig gives it from 2^63 up, at most 4 KiB; the core gets
  `{ number, digest }` (jig's `protocol/client/README.md`). A carried value the
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
  the forge's family-1 numbers first (../credentials.md, pasted values).
- **Carried rows found at start,** loaded from 2^63, are rows whose
  run a restart cut: the core decides
  each one's erase, which the routing carries out, and a connect the
  root still holds as running ends "interrupted; paste again".
- **A run's `Done`:** its writes kept and its carried row's erase
  carried out, then `Request::Connect` (2) after a plan that connects
  (a Forgejo's `Reconcile` or `Check`, GitHub's `Convert` or `Paste`),
  after a `Check` once its login is kept (2), and never after GitHub's
  `Reconcile`; after `Convert`'s, `Install`
  first, whose link answers the browser (../setup/github.md, 2);
  `Request::Installations` after an `Accept` (4); the adoption's
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
  `forge/setup/README.md`, 4.3).

## 7. What the vocabulary adds

skein's (`forge/README.md`, section 3; `skein/docs/design/forge/connection.md`, 4.1), as the
routing carries it to the root, and the domain reads it
(`../../domain/forge/adoption.md` and `../../domain/forge/keeping-up.md`):

- **`Error::NotConnected`:** a call on a forge with no access (not
  connected, connecting, stopped, or its replacement refused), answered
  at once, at cost 0.
- **`Error::NotInstalled`:** on GitHub, a call on an account no accepted
  installation covers, answered at once, at cost 0.
- **`Error::Missing`** on a repository its account's installation did
  not select (GitHub answers `404`), and **`Error::Forbidden`** for a
  suspended installation's calls and the calls a replacement ended.
- **`Error::RateLimited { after }`:** Forgejo's `429` and GitHub's
  pauses alike, `after` as skein's `forge/github.md` gives it (GitHub's
  `Retry-After`; else the window's reset when nothing remains; else
  60 s). Nothing retries in the component; the client's budget holds
  every class until then (skein's `forge/client.md`, section 6).
- **`api::Rate { limit, remaining, reset_at }`,** beside an answer's
  cost: the window the forge reported on the call's last answer, per
  installation on GitHub. The client's budget keeps the least it hears.
- **Validators** (`api::Validator`, at most 128 printable bytes): a read
  given one is conditional, and each `304` is counted in the answer's
  `free`, which no rate counts and the budget charges nothing for. The
  delivery log's polls take one (`README.md`, 4.1).
- **A forge with no record,** never made or erased: `Unavailable`, at
  once.

## 8. The box's link

`temper-box serve`'s side of the box's pipe (../setup/box.md, 1): a step machine
of the routing in `temper-service`, its lines and the root's asks in,
its lines, the root's states and its answers out, hosted on the
service's io.

- **Under the box alone.** `temper-box internal serve --box` adopts the
  box's pipe, the two fds after its listeners (../setup/box.md, 1), and the
  link writes its lines on the first and holds one read on the second,
  a line at most `box.line_bytes` (../configuration/limits.md), its
  line feed included, as the box counts it. Without `--box` there is no
  pipe and no link; the engine's forge domain, told so at its construction,
  refuses every ask "not under the box" (`../../domain/forge/adoption.md`, 1).
  The link decides nothing: it carries the domain's asks out and the
  box's lines in.
- **An ask** is the forge domain's, once it has decided the `SetUp {
  forge }` by the forge's offer (`../../domain/forge/adoption.md`, 1), which
  refuses "not offered", "not under the box" and "busy" itself, or the
  playground's start for its fake forge (../setup/playground.md). The routing
  writes the group's file, `<data>/box/groups/FORGE`, with the forge's
  kind, its setup entry and its port (../setup/box.md, 1); the box knows
  kinds, not forges. The link writes `start FORGE`, tells the
  root the forge setting up, with no step yet, and answers the ask by
  the request's token the root gave it: setting up. The domain asks once at a time, so a second ask while
  one runs is a fault of the domain's: the link answers it "busy",
  named in the log. The answer goes back to the root, whose forge
  connector keeps it as the `SetUp` request's outcome, in memory alone,
  and the asking client reads it (`../../client/engine.md`).
- **The box's lines,** for the forge asked:
  - `step FORGE STEP`, `STEP` one of `fetch`, `prepare` and `run`: the
    forge setting up, at that step;
  - `fetched FORGE BINARY DONE TOTAL`: how far the fetch has gone, told
    to the root as it comes. The forge domain keeps the latest beside
    the forge's connection, in memory alone, while that connection is
    setting up at `fetch`, and any other line's new state drops it, as
    the pipe closing does: gone at the next step, at `started` and at
    `failed`. One heard at another step is the domain's to ignore, the
    link deciding nothing. Each is a change of the forge's row, so at
    most a hundred or so per binary reach a client
    (`../../client/forge/deployment.md`), a restart of serve forgets it, and the
    next line brings it back;
  - **`started FORGE`:** the ask ends and the forge goes connecting. The
    setup host takes the handover with `TakeOver` whenever the file is
    present, even with a kept `AdminToken`, removes it after `Done` and
    the record kept, then runs `Reconcile` with the root's restored
    repositories and `Request::Connect` from records (1, 2;
    ../setup/owned.md, 5). With no file it reconciles from the kept record. Its
    git value is the `GitToken` record's, put at its next generation
    before `Event::Credential` and `Request::Connect` (5). A stop or
    failed removal sets the forge up failed with the step named, never a
    secret; a `TakeOver` stop leaves the file. The other forges run on.
    The next ask, answered `started` at once, runs the sequence again;
  - `failed FORGE WHY`: the ask ends, and the forge's set up failed,
    with `WHY` as the box wrote it, which holds no secret (../setup/box.md,
    1).
- **The failed step** is the forge domain's, from the connection it
  held when told set up failed (`../../domain/forge/adoption.md`, 1), including
  `BoxClosed` during setup. The producers keep their existing events;
  the FORGE row's `SetUpFailed` carries `why` and `at: option
  FailedStep` (fetch, prepare, run or connect). The schema, generated
  codec and goldens update in place at CLIENT_VERSION 11 before the
  first release; no store or record version changes.
- **A removal** is the forge domain's too, at a forge's removal's
  decision and at each start while its record has not heard it done
  (3), and no request waits on it: the link writes `remove FORGE`,
  holding it while another ask is out and sending it once that one is
  answered, as it holds a start; a start held for that forge is
  dropped. The box stops the group and removes its file, its `started`
  record and the forge's directory, keeping the cache (../setup/box.md, 1).
  `removed FORGE` ends it, telling the root the box's step done, which
  the forge's record keeps; `failed FORGE WHY` ends it too, named in the
  log, the step left for the next start. The pipe closed during it ends
  it with the link, and the next start asks again.
- **Named and dropped,** each in one line of the log and changing no
  state: a line for a forge other than the one asked, or while none is
  (except a restored source fact for a kept box-owned Forgejo,
  `../openid-provider.md`, 9, restoring its page facts without another setup ask);
  a line whose word, forge number or step the link does not know; a
  `fetched` line whose `TOTAL` is zero or whose `DONE` passes it; and a
  line past `box.line_bytes`, named by its bound, skipped to its line
  feed.
- **The pipe closed:** the box's end of it closed, or a read or a write on the
  pipe failing, ends the link, named in the log once, and the link
  tells the root `BoxClosed`: the forge domain fails an ask in flight,
  "the box closed the pipe", and offers no set up after it
  (`../../domain/forge/adoption.md`, 1). serve's own end is
  the box's to make (../setup/box.md, 2).
- **Nothing kept.** The link's ask and the states it tells are in
  memory alone (`README.md`, 7.1): a restart makes a pipe and a link afresh, with no
  ask, and a group the box still runs answers the next ask by its own
  lines (../setup/box.md, 1). What a removal needs again is the forge's
  record's (3).

## 9. Open questions

- **A kept GitHub forge whose base ends in `/`.** Its record keeps
  that place (section 6), so its remotes have two `/` after the host.
  Default: so, until it is removed and connected again (../forge/engine.md,
  3); a patch of the record or a trimmed base would repair it in
  place.
