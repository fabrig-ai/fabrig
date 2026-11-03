# The forge's setup

Provisional, 2026-10-16, revised 2026-11-02. How a forge comes to be
ready for temper, in three modes: connected, where a person runs and sets
up the forge and gives temper only what it strictly needs; managed,
where a person runs it and temper sets itself up on it, with a Forgejo
site admin's token or, on GitHub, with an App made from temper's
manifest; and owned, where temper runs Forgejo and its runner, and sets
itself up on them. A fourth, fake, is the playground's: temper runs its
own fake Forgejo, seeded with the playground's repository, and sets
itself up on it as on an owned one (8.4). A forge is a record of temper's
store, made by the act that sets it up or connects it (forge.md, 2.1 and
2.2); a person chooses a forge's mode and makes it from temper's client
(`../client/forge.md`, the deployment's page), and may remove it
(`../domain/forge.md`, 4.1). The
engine is the same in every mode: each mode makes the records the one
below reads, kept in jig's secrets' store (credentials.md, the records),
and the engine's forge component reads connected's. Owned mode is
commands a box arranges, and `temper box` is wiring only. A box is what temper's first milestone is set up on and
checked by CI on (`../README.md`, section 2). The members are configuration.md's (section 3),
the credentials credentials.md's (section 2), the webhooks forge.md's
(section 5), and the commands and crates README.md's (sections 4 and 5).
The walks are the pages `box`, `forgejo` and `github` (`temper help
NAME`). What comes later is section 15; what is open,
section 16; connecting GitHub, section 17.

## 1. In one page

- **Three modes, per forge** (section 2), chosen and connected from
  temper's client. Connected: a person runs and sets up the forge, and
  pastes the bot's two tokens on Forgejo, or an App's id and key on
  GitHub. Managed: a person runs it, and pastes a Forgejo site admin's
  token, or on GitHub makes temper's App from its manifest in their
  browser. Owned: temper runs Forgejo and its runner. A fourth, fake,
  is the playground's, made when the person tries it and never chosen
  as a mode: temper runs its fake Forgejo in place of Forgejo, with no
  runner and its git a route of its own, and sets it up as an owned one
  (8.4).
- **Forges are records in the store** (forge.md, 2.1): each made by the
  act that sets it up or connects it, `SetUp` for an owned Forgejo,
  `Connect` for GitHub or a Forgejo elsewhere, `StartPlayground` for the
  playground's fake one, and numbered next when it is made, a number
  never reused (forge.md, 2.2). The document names no forge: it keeps
  the machine's, its listeners, what a box's owned Forgejo and fakes
  take from the machine, and the limits (section 8). A forge can be
  removed whole (`../domain/forge.md`, 4.1); its box group is then
  stopped and its directory removed, the cache kept (7.1).
- **Each mode is a layer that makes the inputs of the one below.**
  Owned's preparation makes a Forgejo and an admin's token, managed's
  input, and hands the token over in a file; managed's plans make the
  bot, its tokens, its id and the webhooks' secret, connected's inputs.
  The forge component always runs connected: it reads the same records,
  checks the same user, and never holds an admin's token (section 3).
- **The records are the interface.** Each forge's records, under its
  number, a stable id never reused, are kept in jig's
  secrets' store by the engine's protocol layer alone (credentials.md,
  the records). The one file that passes between the layers is an owned
  forge's handover, `admin_token`, which the engine takes once.
- **The core is step code; the hosts are thin.** The setup's plans are
  skein's `skein-forge-setup` (skein's `forge-setup.md`, sections 4 and
  5), run inside the engine by skein's setup host component (its host,
  6.3): per forge, one writing plan at once and the picker's read beside
  it (5.1). The preparation, Forgejo's settings and the processes' specs
  are machines and pure functions of `temper-forge-setup`, hosted by
  three commands, `temper forge fetch`, `temper forge prepare` and
  `temper forge run` (section 6). No command writes the records.
- **`temper box` is wiring only.** It runs a configuration's steps in
  order (the fakes, fetch, prepare, Forgejo, its runner, `temper
  serve`), restarts what exits, and stops them in reverse order. It
  knows kinds, not forges: an owned Forgejo's group and the playground's
  fakes' wait, dormant, until `temper serve` asks over a pipe for one of
  them for a forge, and run again at each later start once started
  (7.1). It names no Forgejo request and nothing of setup but its
  steps. systemd or a test arrange the commands another way, with no
  owned forge (section 7).
- **`temper init` writes a deployment that starts** into the three
  directories (configuration.md, section 12): the default document, its
  agent's configuration, the worker's secret and the scripted model's
  key; no forge, no account and no credential, which are the client's.
  `temper box` runs it first on an empty configuration directory (8.2,
  8.3).
- **A first run is empty** (8.3): the store holds no forge, and the
  person tries the playground or sets temper up. The playground is made
  when chosen (`StartPlayground`): a fake forge's record, its box group,
  the scripted account and the playground's project, into which
  `playground/widget` is adopted; its reset (`EndPlayground`) deletes
  the project and removes the forge and the account, and the person is
  on the welcome's Forge step as if they had never tried it. It is
  offered while no real forge is kept, and never once one is (8.4).
  Real forges are made from the client: an owned Forgejo set up under
  the box, its group asked when the person sets it up (6.5, 7.1), and
  GitHub or a Forgejo elsewhere connected (section 17, 5.5); none needs
  a restart.
- **A set up writes as it goes, and starts over by removal** (6.7): the
  forge's record and its first repository's intent are kept as it goes;
  an unfinished forge found later is removed and set up again, Forgejo's
  binaries kept in the cache.
- **Reconcile converges at every start** of a managed or owned Forgejo,
  in the engine: the bot user, made when missing and otherwise checked,
  never edited; its two tokens; and each
  adopted repository's access, the bot a collaborator with write and the
  repository's hook; each made when missing, repaired when changed, and
  left alone otherwise (section 5).
- **Access at adoption.** Adopting a repository of a managed or owned
  Forgejo first gives the bot its access (`Adopt`, 5.4); nothing is
  granted org-wide. The picker offers every repository the site admin
  can see, read by the `Reachable` plan, which writes nothing (5.2).
- **GitHub** (section 17): managed by the manifest flow, through the
  clients' listener's routes, or connected by an App the person pasted;
  both end in the same records. Its reconcile, after each connect,
  repairs only the App's hook's address; its changes reach the engine
  through the App's delivery log, with no webhook secret.
- **Owned is a stock Forgejo.** temper writes its `app.ini`, runs its
  migrations, makes its own admin and the person's account with
  Forgejo's command line, registers the runner offline, dumps before an
  upgrade's migration, and runs only the versions it pins, the
  conformance's, which it fetches itself when asked, checked against
  the digests it pins (section 6). Its directory, once temper stops running
  it, is a Forgejo a person can run themselves, as managed.
- **A box is reactive.** Every change on the forge that a box's work
  waits on reaches the engine as a signed webhook delivery, and becomes a
  hint, read within the keep's `hinted`: pushes, pull requests, issues,
  comments, reviews and each CI job's end. Adoption and reconcile make
  the hooks; an owned Forgejo is allowed to deliver to the engine's
  listener. Polling is unchanged, the fallback for a delivery lost or
  refused (section 10).
- **Loopback.** An owned Forgejo listens on loopback, as temper's
  clients' and webhooks' listeners do; a person on another machine
  reaches them through a tunnel. A forge on another host polls: its
  webhooks need skein's TLS server side (section 15).
- **Not here:** webhooks from another host, the isolation of CI's jobs,
  provisioning at runtime and the forge as people's sign-in provider
  (section 15).

## 2. The modes

| | connected | managed | owned | fake |
|---|---|---|---|---|
| Runs the forge and CI's runner | the person | the person | temper | temper: its fake forge, `temper fake forge`, whose CI answers each push itself; no runner |
| Makes temper's bot, its tokens, its access and the webhooks | the person | temper: on Forgejo with a site admin's token; on GitHub, an App from temper's manifest | temper, with an admin it makes | temper, with the admin the fake is seeded with |
| temper is given, on Forgejo | the bot's two tokens, pasted; optionally a webhooks' secret | a site admin's token, pasted | the Forgejo and runner binaries, or `"fetched"` for temper's (6.6); an address on loopback; the person's login and email | an address on loopback; the seed's owner and repositories |
| temper is given, on GitHub | an App's id and its private key, pasted | the person's steps in their browser, from a link temper prints | none: GitHub is never owned | none: the fake is a Forgejo |

- **Fake is owned's layers,** with temper's fake forge in place of
  Forgejo and its runner (8.4). What this document and the others say
  of an owned forge's handover, directory, reconcile, records, adoption
  and picker holds of a fake one; what they say of Forgejo's or the
  runner's binaries, `app.ini`, the preparation's command line and the
  person does not. A fake forge
  keeps nothing across its starts but its directory's handover: each
  start serves the seed afresh, and reconcile makes the bot again
  (8.4).
- **Per forge.** A deployment may mix them: an owned Forgejo beside a
  connected one, or beside GitHub. The playground's fake forge is made
  only while no real forge is kept (8.4). Each owned or fake forge has a directory and an address
  of its own, and a box runs at most one of each kind (7.1).
- **A mode is its forge's record's** (forge.md, 2.1), set by the act
  that made the forge: owned by `SetUp` (6.5), fake by
  `StartPlayground` (8.4), managed by `Connect` with a Forgejo site
  admin's token (5.5); a Forgejo connected with its bot's tokens is
  connected (section 4), and so is a managed one whose `AdminToken`
  record is forgotten (below). It is no credential record
  (credentials.md, the records). On
  GitHub, managed and connected end in the same records, and differ only
  in how the App came to be (section 17).
- **A mode changes:**
  - owned to managed is later (section 15);
  - managed to connected, from the client, "Forget the admin's token":
    the `AdminToken` record is erased, no reconcile runs for the forge,
    and its bot, tokens, id and hooks stay as reconcile last kept them;
    adopting a repository then grants nothing, and the person makes the
    bot a collaborator (section 4);
  - connected to managed, from the client, "Connect as its admin": a
    site admin's token is pasted, and reconcile runs with it, taking
    over the connected user as its bot when its full name carries
    `temper:bot` (5.5).

## 3. The layers and their files

The three modes and their layering (each mode makes the inputs of the
one below, and the service always runs connected) are held by skein's
`forge-setup.md`, section 3; fake is owned's layer with another forge in
it (section 2). temper's layers, and what passes between them:

```
owned      temper forge fetch        ──► the Forgejo and runner binaries, in the cache (6.6)
           temper forge prepare      ──► the forge's work path, the runner's, person_password,
                                         admin_token (the handover)
           temper forge run          ──► forgejo web (its child on Linux), forgejo-runner daemon (exec'd)
fake       temper forge prepare      ──► the forge's directory, admin_token (the handover)
           temper fake forge         ──► the fake, seeded with temper-admin and that token (8.4)
managed    the engine's setup host   ──► the forge's records (below); takes the handover once
connected  the forge component       ◄── its access, built from the records
```

- **One directory per owned or fake forge,** named by its kind and
  number beneath the data directory, `forge-N` for an owned forge and
  `playground-N` for the playground's fake one, and removed with the
  forge (7.1): private, `0o700`, the user's own and not reached through a link,
  as a worker's root is (configuration.md, section 4). `temper forge
  prepare` makes it so when it is absent; a directory that is not so
  refuses the command, naming it. A managed or connected forge has none.
  A fake forge's holds `admin_token` until the engine takes it; `temper
  fake forge` reads it before binding (8.4). An owned forge's files:

  | File | Made by | Read by | Holds |
  |---|---|---|---|
  | `forgejo/` | prepare | Forgejo, through `run` | Forgejo's work path: `app.ini`, its database, repositories and logs |
  | `runner/` | prepare | the runner, through `run` | the runner's configuration, its registration, its jobs' directories, `secret` |
  | `backups/` | prepare | the person | one dump before each upgrade's migration |
  | `state` | prepare | prepare, `run` | owned's record: the versions prepared, whether the first preparation ended, Forgejo's own secrets |
  | `admin_token` | prepare, when missing | the engine's setup host, once, at start or the box's `started` (6.5) | the handover: temper's admin's token, kept as the forge's `AdminToken` record by `TakeOver`, after whose `Done` the file is removed (6.5) |
  | `person_password` | prepare | the person | the person's first password on the owned Forgejo |
  | `person_read.header`, `person_write.header` | prepare | the person, or their agent (`../client/tools.md`, 11.5) | the person's two tokens as one header line each, `Authorization: token TOKEN`: one that reads (`read:repository`, `read:issue`, `read:organization`, `read:user`), one that writes (`write:repository`, `write:issue`, `read:organization`, `read:user`) |

- **What the preparation leaves for the engine** is `admin_token`
  alone: the token's text, nothing else, private and written whole;
  made at the first preparation and at each later one that finds it
  missing, which it is once the engine has taken it, and left as it is
  when present (6.3).
- **Fetched binaries are the cache's,** not the forge's directory's:
  `<cache>/forgejo/VERSION/forgejo` and
  `<cache>/forgejo-runner/RUNNER_VERSION/forgejo-runner`, shared by
  every owned forge whose binaries are `"fetched"`, written by `temper
  forge fetch` alone and fetched again at will (6.6; configuration.md,
  section 12).
- **No other credential file is written or read for owned or fake.**
  Their `api_token`, `git_token`, `user` and `webhook_secret` files,
  including any left by an earlier temper, stay unread. Each record is
  written by a plan, `User` as its 8 bytes within a version-1 record
  (credentials.md, the records).
- **Written whole.** Each file is written beside its name and renamed
  over it, private, so a reader never sees half a token.
- **The handover is read** as configuration.md, section 6 reads a
  secret's file, within `records.value_bytes`: it is the one credential
  file of an owned or fake forge that `temper serve` reads (6.5).
- **The records, by mode** (credentials.md, the records):

  | Forge | Records |
  |---|---|
  | Forgejo, connected | `ApiToken`, `GitToken`, `User`, `Origin`; `WebhookSecret` when the person gave one |
  | Forgejo, managed, owned or fake | those, and `AdminToken`; `WebhookSecret` drawn by reconcile when the webhooks' listener is configured |
  | GitHub, managed or connected | `AppId`, `AppSlug`, `AppKey`, `User` (the App's bot user), `Accepted`, `Origin`; no `WebhookSecret` |

- **Written by the setup host's plans alone,** each write kept before
  the plan goes on (5.1). A value a person pastes is kept by jig until
  its run ends (credentials.md, pasted values), and is no record.
- **Records made for another forge,** an `Origin` naming another kind
  or address, and unreadable ones, leave that forge not connected,
  named on the deployment's page; nothing is erased, and connecting
  again overwrites them (credentials.md, the records).
- **Nothing passes between the layers** in an argument or the
  environment but the directories' flags and the forge's number; the
  commands read the forge's setup from its group's file (7.1).

## 4. Connected

- **What temper strictly needs** of a connected Forgejo: the bot's API
  token and its git token, and, for webhooks, their secret; the bot's
  id is read with the tokens (credentials.md, sections 4 and 5). The
  scopes (skein's `forge-setup.md`, section 8): `read:user`,
  `write:issue` and `write:repository` for the API token;
  `write:repository` and `read:user` for the git token, the second so
  that the setup's plans can ask whose it is (skein's `forge-setup.md`,
  4.3, step 3, and 4.6).
- **What the person does:** a Forgejo user for temper, not a site admin;
  its two tokens; its write access to each repository temper will adopt,
  as a collaborator or through a team; optionally, each repository's or
  org's webhook to temper's listener, `http://<listener>/hooks/<number>`,
  with the secret and the events of skein's `forge.md`, section 7, and
  Forgejo's `[webhook] ALLOWED_HOST_LIST` letting it reach the listener;
  CI, as each repository has it.
- **Pasted from the client, together** (`Connect` with `ForgejoTokens`;
  `../client/forge.md`, the deployment's page; `../client/tools.md`,
  5.5): both tokens, and the secret when given, one value, kept by jig
  until its run ends (credentials.md, pasted values). Its connect makes
  the forge's record, connected, from the address the person gives, its
  API's host, port and transport, its git and web bases, and its git
  program when given, numbered next (forge.md, 2.2), in the connect's
  admission, before any run; or, on a connected Forgejo kept and not
  connected or stopped, connects it again. Any other forge refuses it,
  *not taken*. The setup host's `Check { forge }` (skein's
  `forge-setup.md`, 4.6) finds that both answer as one user, not a site
  admin, and writes that user; the routing keeps the tokens and the
  secret as the forge's records, then its `Origin`, and the forge
  connector keeps the user's login as the record's bot (forge.md, 2.2);
  the page then says "Connected as LOGIN", with its id on an Account
  line (6.5). Tokens of two users stop it `OtherUser`, a site admin's
  `BotIsAdmin`, a refused token `Refused`, each named, and the forge
  stays as it was. This form makes no bot, organization, repository or
  hook: the person's user is temper's, as it is.
- **Connected again,** on a kept forge whose `User` is kept, by the same
  user alone: `Check` compares the tokens' user with it, and tokens of
  another user stop it `OtherUser`, nothing kept, the forge as it was.
  A forge's writer is its user, so another user's tokens connect a new
  forge, once this one is removed (credentials.md, 2.1).
- **Replaced** from the client (`Replace`): a new API token, git token
  or both, and the secret when it changes, one value: `Check` reads each
  pasted token before the forge's record, so a token not pasted is the
  kept one, and finds that both answer as the kept user; each pasted
  item is then kept over the old, and the forge connects with them. A
  stop keeps nothing, and the forge stays as it was.
- **Nothing more is required** (`../domain/forge.md`, section 4): polling
  stands in for webhooks, and a project's checks for CI.
- **A connected GitHub** is an App the person made, pasted (17.3).

## 5. Managed: reconcile

Reconcile is skein's plan, run by the engine's setup host at each start
of a managed, owned or fake Forgejo, at a managed connect and at an
owned forge's set up from the client (5.1). Nothing but the setup host
runs it or writes what it makes.

### 5.1 The setup host in the engine

- **skein's host component** (skein's `forge-setup.md`, its host, 6.3),
  a child of the engine's protocol layer beside the forge component; the
  routing carries what passes between it, the root and the records
  (forge.md, the setup's host). It holds an admin's token and a GitHub
  App's key only while a plan that asked for them runs, and drops them
  at its end.
- **Per forge, one writing plan at once, and one `Reachable` beside
  it,** since the picker's read writes nothing. A second of either kind
  is refused "busy", never queued, and so is one past `setup.runs`,
  which counts both kinds. One exception: an `Install` waits for the
  engine's own run on its forge (the reconcile after a connect, at a
  start or on a hint), at most one per forge and for at most
  `setup.state_life_ms`, and is refused busy only by a run a person
  asked, since its link writes nothing.
- **Its needs:** a write is a `Keep` of the forge's record, answered
  `Written` only once `Kept`, so a browser is answered and a connect
  sent only on what a restart would find; a read is served from the
  run's pasted value first, then from the forge's record. It draws its
  random bytes from the engine's source.
- **States,** for GitHub's browser steps, are its own, in memory: at
  most `setup.states`, each living `setup.state_life_ms`, an hour at
  most; a restart forgets them (17.2).
- **At each start,** once the records are loaded and the root restored:
  an owned or fake forge's handover by `TakeOver` whenever its file is
  present, even with a kept `AdminToken`, its file removed, then
  `Reconcile` with the restored repositories and connect (6.5); each
  managed Forgejo's reconcile; the fake's start-time adoption (8.4); and
  GitHub's `Reconcile` after each GitHub forge connects (17.5). The
  deployment's page shows a forge "Connecting" until its `Connected`.
- **At the box's `started`,** the setup host runs the same handover,
  removal, reconcile and connect sequence for the forge (6.5). A stop or
  failed removal leaves it set up failed, named by the step, and offered
  again; every other forge runs on.
- **A run's end:** `Done`, or `Lacking` (done, with what the App lacks
  named), keeps its writes, then connects the forge after a plan that
  connects; `Stop` names its failure and its step, with no secret, and
  leaves the forge as it was. A start's reconcile that stops leaves that
  forge not connected, named, until the next start, a managed connect or
  an owned forge's next set up ask (6.5); the engine and every other
  forge run on.

- **Closing progresses without diagnostic consumption.** The service's
  setup adapter tracks closing requested separately from host `Close`
  sent. A close request enters closing once, even when active work's
  room gates are full. While closing it drains the host's outputs
  independently of notes, store, forge and events room, discarding
  non-`Closed` outputs as the shutdown path does. It sends `Close` once
  bounded host room permits, at most once, and continues io settlement
  and draining until the actual `Closed`. Neither requesting close nor
  sending it requires the application to pop notes; sending it never
  completes shutdown. Active-phase room gates still protect their
  downstream writes. Host `Close` and outside-caller terminal semantics
  remain those of skein's host (skein's `forge-setup.md`, 6.3) and
  programming-model.md, 5.2: closed follows settlement, never a request.

### 5.2 The setup host's plans

| Plan | Forge | When | skein's `forge-setup.md` |
|---|---|---|---|
| `Reconcile { forge, adopted }` | Forgejo, managed, owned or fake | each managed, owned or fake start; a managed connect; the box's `started` (5.1) | section 4 |
| `Adopt { forge, repository }` | Forgejo, managed, owned or fake | an adoption, before its reads; the fake's start (8.4) | 4.5 |
| `Create { forge, org, repository, files }` | Forgejo, owned or managed | an owned forge's, after reconcile at the box's `started` or a start, while its record keeps a first repository's intent not yet made (6.7); a managed connect's with `first` (5.5) | 4.9 |
| `Check { forge }` | Forgejo, connected | a connect; a replacement | 4.6 |
| `TakeOver { forge, admin_token }` | Forgejo, owned or fake | a start or the box's `started` with a handover, even with a kept `AdminToken` (6.5) | 4.7 |
| `Reachable { forge, installation?, page }` | every kind | the picker | 4.8 |
| `Manifest { forge, target }` | GitHub | Connect, managed | 5.1 |
| `Convert { forge, code, state }` | GitHub | the redirect back | 5.2 |
| `Install { forge }` | GitHub | Install the App, or Install on another account | 5.4 |
| `Accept { forge, installation, state? }` | GitHub | the setup route's visit; the owner's accept of a stray (17.4) | 5.4 |
| `Reconcile { forge }` | GitHub | after each connect | 5.6 |
| `Paste { forge, app }` | GitHub | Connect with a pasted App; Replace | 5.8 |

- **What each does, and when the routing runs it,** is forge.md's (the
  setup's host); each plan's steps, its bounds and what it refuses are
  skein's.
- **`Reachable`** serves every kind, pages counted from 1, at most
  `reachable` rows, which temper gives from `picker.page`
  (configuration.md, section 7), and ends `Done(Reachable { rows, more
  })`, `more` when the page is full; it writes nothing. On GitHub it
  reads one accepted installation's repositories, `installation` its
  id, on a token minted for the page and dropped; an id not in
  `Accepted` stops it `Installation`, `Missing`. On a connected Forgejo
  it reads what the API token can push to; an `installation` given
  there stops it `Installation`, `Unexpected`. On a managed, owned or
  fake Forgejo it reads every repository the admin can see, by id ascending,
  on the admin's token, which stays the setup host's, and is refused
  `Missing` or `NotAdmin` as reconcile's first step is. Forgetting that
  token makes the forge connected, and its picker with it.
- **Outcomes:** `Done`; `Manifest { state, page }`, the state and the
  manifest page's bytes, which temper holds with the state; `Link`, the
  install link; `Lacking`, done, with each permission or event the App
  lacks named.

### 5.3 Reconcile's given values, and its answers

- **The plan is skein's.** Its six steps, in this order (1 the admin; 2
  the bot, marked `<word>:bot`; 3 the bot's two tokens; 4 the bot's id;
  5 each org, when owned; 6 each adopted repository's access, the bot a
  collaborator and the repository's hook), each read first and written
  only where the forge differs; its wait for the forge; its bounds; and
  what it refuses are held by skein's `forge-setup.md`, section 4. It
  does no io and reads no configuration: it is given values, says each
  request, and the setup host runs it.
- **Its given values, from temper's configuration:**

  | Reconcile's value | temper's |
  |---|---|
  | the forge's number | its record's (forge.md, 2.1) |
  | the bot's login and email | its record's bot (forge.md, 2.1); at a connect as its admin of a forge connected before, the connected user's login (5.5) |
  | the tokens' names | `temper-api` for the API's, `temper-git` for git's |
  | the orgs, and their owner | none: reconcile makes no org; an owned forge's org is its first repository's, made by `Create` (6.7) |
  | the repositories | the forge's adopted repositories, the root's, restored before the start's reconcile |
  | the name a failure gives the admin | `temper-admin` |
  | the listener's address | `application.webhooks.listener`'s, when configured; none otherwise. Each hook's URL, `http://<address>/hooks/<number>`, is derived from it (skein's `forge-setup.md`, 4.1) |
  | the word | `temper`, so the bot's mark is `temper:bot` |
  | its limits | `orgs` from `setup.orgs`, `repositories` from `forge.client.repositories`, `reachable` from `picker.page`, and `deadline`, `backoff` and `backoff_max` from `setup.deadline_ms`, `setup.backoff_ms` and `setup.backoff_max_ms` (configuration.md, section 7) |

  The API's address is the forge's record's place (forge.md, 2.1).
  temper-forge-setup maps the setup onto these values and runs the
  kit's check on them, at each start and for every command (section
  11); a value the check refuses comes back as its `Place` (skein's
  `forge-setup.md`, 6.1), and is named by the path this table gives for
  it.
- **What the setup host answers, as needs:**

  | Reconcile asks for | The setup host answers |
  |---|---|
  | the admin's token | at a managed connect, the pasted value; otherwise the `AdminToken` record |
  | the API and git tokens, the bot's id | the `ApiToken`, `GitToken` and `User` records, or none when absent |
  | the webhooks' secret | the `WebhookSecret` record, or none, and reconcile then draws one |

  Each answer is a record or a pasted value, never a file: a forge's
  `api_token`, `git_token`, `user` or `webhook_secret` file, from a
  temper before the records, is not read, and reconcile makes what is
  missing afresh (section 3). Each item reconcile writes is kept as the
  forge's record; a kept `WebhookSecret` is never drawn again. The
  fake's start-time adoption (8.4) reads the same records, after its
  handover and reconcile.

### 5.4 Access at adoption

- **`Adopt { forge, repository }`** runs before an adoption's reads on a
  managed, owned or fake Forgejo (skein's `forge-setup.md`, 4.5): the bot made
  a collaborator with write, and the repository's hook made when the
  listener is configured, with the forge's secret. Stopped, the adoption
  is refused, named (`../domain/forge.md`, access at adoption). A
  `403` here, at reconcile's step 6 or at the picker's read, is the
  admin's token without `write:repository` (skein's 4.3, step 1): it
  is shown as that scope missing, named with its step.
- **At each start,** reconcile's step 6 checks each adopted
  repository's access, and repairs what changed.
- **A fake forge:** its repositories are
  its setup's (section 8), so the setup host runs `Adopt` for each of them at the
  engine's start, before the forge connects, in place of each
  adoption's (8.4, the fake's start-time adoption).
- **Nothing org-wide:** the bot reaches the repositories adopted, a
  person's own among them, and no other.
- **A connected Forgejo, and a managed one forgotten,** grant nothing at
  adoption: the person makes the bot a collaborator, and the repository
  polls unless the person adds its hook (section 4).

### 5.5 Connecting as its admin, and forgetting

- **"Connect as its admin"** (`Connect` with `ForgejoAdmin`;
  `../client/forge.md`, the deployment's page), for a Forgejo elsewhere:
  its connect makes the forge's record, managed, from the address the
  person gives, its API's host, port and transport, its git and web
  bases, and its git program when given, numbered next (forge.md, 2.2),
  in the connect's admission, before any run; or, on a managed Forgejo
  kept and neither connected nor connecting, connects it again. Any
  other forge refuses it, *not taken*. The site admin's token, with the
  preparation's scopes (6.3), is pasted and kept by jig until its run
  ends. One run holds the paste:
  `Reconcile` reads it, never an older `AdminToken` record; its writes
  are kept, then the forge's `Origin` under `AdminOrigin`. With `first`,
  `Create` (6.7) runs with the same paste. Only after both plans are Done
  is the token kept as the forge's `AdminToken` record, and the forge
  connects, managed.
  Without `first`, Create is skipped. This connect never runs `TakeOver`.
  Every end erases the carried value and drops the held token; a stop
  keeps no admin token. Each later start
  reconciles from that record and connects, with no Create or `TakeOver`;
  without the record it offers Connect, not connected. This form serves
  no organizations or organization owner and no listener: it polls.
- **Its bot:** on a forge with no secret records, its record's bot,
  the one its connect carries, `temper` unless it names another, made
  when missing; on a forge with them, the recorded user's login.
  Reconcile uses it only when its full name carries `temper:bot`, or stops `Unmarked`,
  named, and restores the connection's previous state.
- **A first organization named as the bot** is refused at admission
  (`FirstIsBot`), before any run or write, and its carried value named
  unused: a Forgejo user and organization can't share a name, so Create
  would stop at the organization, and a second paste couldn't help. The
  check compares `first`'s organization with the record's bot's login,
  ASCII case-folded, as Forgejo compares names. A connect refused at
  admission makes no record.
- **"Forget the admin's token"** (`Forget`): the `AdminToken` record is
  erased, and the forge is connected, its bot, tokens, id and hooks
  kept; no reconcile runs for it after.

### 5.6 What holds of every plan on Forgejo

- **Plaintext to loopback alone, for owned and fake:** an owned or fake
  forge's `transport` is plaintext, and one whose transport is TLS is
  refused at start, naming it (section 15). A managed forge's is TLS, or
  plaintext to a loopback host alone; plaintext to another host is refused
  at start, naming it.
- **The bot's email** is its record's bot's (forge.md, 2.1) when
  reconcile makes the bot, and stays as it is after: an existing bot's
  email is the person's to change, on Forgejo. temper reads no mail of
  the bot's, and its commits name its bot's git identity, the record's
  (forge.md, 2.1).
- **The scopes** of the tokens are section 4's.
- **What it cannot set:** Forgejo's settings, which its API does not
  change (skein's `forge-setup.md`, section 4): the webhooks'
  `ALLOWED_HOST_LIST`, and whether Actions are enabled. A managed forge
  whose host list refuses temper's listener delivers no webhook, and
  polling keeps up (section 10).

## 6. Owned: prepare and run

### 6.1 Its inputs and its binaries

- **Inputs:** the Forgejo binary, and the runner's, or no runner, each
  an absolute path or `"fetched"`, temper's pinned build in the cache
  (6.6); the address Forgejo listens on, a socket address on loopback;
  the person's login and email; the runner's labels, capacity and
  `PATH`; the forge's directory.
- **The binaries are checked** by every command that runs them: the
  first line of each one's version equal to temper's pin, Forgejo's and
  the runner's (`skein-forge`'s `VERSION` and `RUNNER_VERSION`, skein's
  `forge.md`, section 4.3), the versions skein's conformance checks
  (`fake-forge.md`, section 7). Another binary refuses the command,
  naming its path and what it printed. A fetched binary is checked so
  too, after its digest at its fetch; temper downloads nothing but by
  `temper forge fetch` (6.6).
- **One pin,** `skein-forge`'s, read by owned mode, the throwaway and
  the conformance alike, so the Forgejo temper runs is the Forgejo whose
  facts it checked (13.3): the versions, and beside them each fetched
  binary's SHA-256 for each architecture temper fetches for (6.6).

### 6.2 Forgejo's settings

`app.ini`, written by every preparation from the setup and `state`,
private, never edited by hand:

- `RUN_USER` the process's user, `WORK_PATH` the work path;
- `[server]`: `HTTP_ADDR` and `HTTP_PORT` from the address, `ROOT_URL`
  `https://<public>/` with `DOMAIN` its host when `public` is given, and
  `http://<address>/` otherwise, `DISABLE_SSH`, `OFFLINE_MODE`;
- `[database]`: SQLite in the work path;
- `[repository]`: its root in the work path, `DEFAULT_BRANCH` `main`;
- `[security]`: `INSTALL_LOCK`, `SECRET_KEY` and `INTERNAL_TOKEN`;
  `[oauth2]`: `JWT_SECRET`; each drawn once, at the first preparation,
  and kept in `state`;
- `[service]`: `DISABLE_REGISTRATION`; `[mailer]`: off;
- when the forge signs its people in through temper, named by
  `application.sign_in.forges` (`sign-in.md`, 9.7): `[oauth2_client]`
  `ENABLE_AUTO_REGISTRATION = true`, `USERNAME = preferred_username` and
  `ACCOUNT_LINKING = login`, and `[service]`
  `ALLOW_ONLY_EXTERNAL_REGISTRATION = true` (`sign-in.md`, 9.4). The
  source itself is not the preparation's: it is added by the command
  `temper forge sign-in` prints, as for a managed Forgejo (6.8);
- `[webhook]`: `ALLOWED_HOST_LIST` the webhooks' listener's IP address,
  when one is configured, and Forgejo's default otherwise, which refuses
  loopback;
- `[actions]`: `ENABLED` when the setup has a runner;
- `[log]`: files in the work path;
- `[api]`: Forgejo's defaults, which the forge component reads at its
  start (skein's `forge-connection.md`, section 3).

### 6.3 The preparation

`temper forge prepare [DIRECTORIES] FORGE`, before Forgejo runs. It takes
Forgejo's lock (6.4), and the runner's too when it writes the runner's
files: at a first preparation, for changed settings (item 2), and for
a runner older than its pin (item 3) or added (item 4); checks the
binaries; and decides by
`state`:

1. **The first preparation:** the layout of section 3; `app.ini`;
   `forgejo migrate`; temper's admin, `temper-admin`, made as a site
   admin with a password Forgejo draws and nobody keeps, and its token
   made by `admin user generate-access-token` into `admin_token`, the
   handover (section 3), with the four scopes `write:admin`, `read:user`,
   `write:organization` and `write:repository` (skein's `forge-setup.md`,
   4.3, step 1: the users, their tokens and the orgs; `GET /user`; the
   orgs' and their repositories' making and reading; the repositories'
   routes of `Adopt`, reconcile's step 6 and the picker's read), and no
   other, under a name that
   ends in a random suffix, since Forgejo refuses a second token of one
   name; the
   person's account, a site admin with a password Forgejo draws into
   `person_password`, private, never printed, never in an argument or a
   log, with no change forced at the first sign-in (`admin user create
   --random-password --must-change-password=false`; the user, 10-26), so
   that the person's tokens work at once; the person's
   two tokens, each `admin user generate-access-token --raw --scopes`
   for the person's login under a name ending in a random suffix,
   written whole into `person_read.header` and `person_write.header`,
   private, so that the person's agent reads the forge, or acts on it
   as the person, by a file's path and never a token on a command line
   (`../client/tools.md`, 11.5); the runner's
   secret drawn into `runner/secret` (40 hexadecimal digits, no line
   feed), registered as a global runner by `forgejo-cli actions register
   --secret-file`, and the runner's connection written from it into the
   runner's configuration (`server.connections`), its token in a private
   file the connection names (`token_url: file:`); the work path's
   repositories directory, which `forgejo dump` needs. `state` records
   the first preparation's start before anything else, and its end last.
   A first preparation that `state` says began and did not end is cut
   short: the next one removes what it left (the work path, the runner's
   directory, `admin_token`, `person_password`, the two header files)
   and begins again from an
   empty layout, since Forgejo refuses a user made twice and never shows
   a drawn password again. A forge's directory that holds Forgejo's files
   with no `state` is refused, naming it: the preparation removes only
   what a first preparation it recorded left.
2. **A later preparation at the same versions, with a runner exactly
   when `state` records one:** `app.ini` written again, so a change of
   the setup reaches Forgejo; `admin_token` made again when missing,
   which it is once the engine has taken it (6.5), under a fresh suffix,
   for temper's admin, which Forgejo's command line cannot make twice:
   an admin a person deleted fails the preparation, naming it; the
   person's two header files made again, each alone, when missing, as
   on a forge prepared before they were (a person who revokes a token
   deletes its file, and the next start mints a new one). A configured
   runner is registered again with its existing secret, keeping the same
   registration, and its configuration is rendered from the current setup.
   If the configuration is identical, its file is left untouched and no
   runner lock is needed, so a box can restart Forgejo while the runner
   runs. A changed or missing configuration is written whole under the
   runner's lock; a running runner refuses that preparation. The prepared
   versions and Forgejo's secrets are recorded unchanged. Stop the box,
   edit `CONFIG/temper.json` and restart with the same directories to
   apply changed capacity, labels or `PATH` without replacing secrets.
3. **An upgrade:** a recorded Forgejo older than the pin is dumped
   (`forgejo dump`) into `backups/forgejo-<old>-to-<new>.zip`, written as
   `backups/dump.partial.zip` (Forgejo adds `.zip` to a name without it)
   and renamed, never overwritten (a dump already there is not made
   again), then migrated, then recorded. A recorded runner older than its
   pin has its files prepared again, under the runner's lock too, with no
   dump, since the runner keeps no data. A recorded version newer than
   its pin, Forgejo's or the runner's, refuses: Forgejo's migrations only
   go forward.
4. **A runner added or removed:** the setup and `state` disagree on
   whether there is one, and `app.ini`'s `[actions]` follows the setup
   (6.2). A runner added is prepared under the runner's lock too, as a
   first preparation prepares it: its secret drawn into `runner/secret`
   when that file is missing, and kept otherwise; registered; its
   connection written. A runner removed is recorded as none, and its
   files and its registration are left, unused, so that a runner added
   again registers as the same one. A runner's recorded version is
   compared with its pin only while the setup has one. Either may come
   with an upgrade, in one preparation.

- **Forgejo's command line only,** on the work path, never its web: under
  the lock no Forgejo runs, so nothing writes its database beside the
  preparation.
- **No secret in an argument.** A password is drawn by Forgejo (the user
  creation's random password) and read from its output; the runner's
  secret goes by file (`--secret-file`). The runner's `create-runner-file`
  takes its secret only as an argument, so the preparation never runs
  it: it writes the runner's connection itself, in the runner's
  configuration, with its token in a file (13.3).
- **Its commands run on skein-io's processes,** as the box's children
  do: each spawned with its arguments and the spec's environment, its
  output read through pipes within a bound, its exit status awaited.
- **Interrupted** by a termination signal while a command runs, it
  kills the command's process group and exits 1, printing `forge <n>:
  interrupted`. `state` is not marked ended, so the next preparation
  begins again from an empty layout, for a first one, or repeats, for a
  later one or an upgrade.
- **Its commands' output,** kept for a failure's message, is masked of
  every secret it holds, as the throwaway Forgejo masks its own.

### 6.4 Running

`temper forge run [DIRECTORIES] FORGE forgejo|runner`:

- **Checks** the binary, and that `state` records a preparation at the
  pinned versions, with a runner exactly when the setup has one;
  refuses otherwise, naming what to run: `temper forge prepare` for a
  forge not prepared, a preparation that did not end, an older version,
  or a runner added or removed (6.3); no command of this temper for a
  newer version, or one it cannot read.
- **Takes the lock,** an exclusive lock, taken without waiting, on a
  file of the forge's directory, Forgejo's or the runner's. Two Forgejos
  never share a work path, and no preparation runs under one: the
  preparation takes Forgejo's lock, and the runner's too when it writes
  the runner's files (6.3), and refuses, naming the forge, while one it
  needs is held.
- **Checks Forgejo's listener,** when the box gave it one (7.1):
  descriptor 3, with `LISTEN_FDS=1`. It checks that the descriptor
  listens at the setup's `listen` (skein's `listener_at`, `shell.md`,
  6.6) before taking the lock, refusing otherwise, naming the descriptor
  and the address it found. Forgejo then takes the socket whose address
  its `app.ini` names, in place of binding it (its `modules/graceful`,
  which reads no `LISTEN_PID`). Given none, as under every other
  arrangement (7.4), Forgejo binds its address itself.
- **Stays Forgejo's parent when given a listener.** Forgejo 17 sends
  its parent `SIGTERM` once when it starts with `LISTEN_FDS` set (its
  `KillParent`, which no setting turns off), so forge run doesn't exec,
  which would make the box that parent. It holds the lock for Forgejo's
  whole life, adopts the listener into skein-io, and spawns `temper
  internal forge-child --parent PID --signal kill -- forgejo web …`, PID
  its own, with the listener at the child's descriptor 3, the spec's
  environment ending in `LISTEN_FDS=1`, and the spec's directory.
  Forgejo's standard input is empty; its output and errors are relayed
  to forge run's own.
  - **The hidden `internal forge-child`** reads no document and takes no
    directories. It execs its program with what it was given, through
    skein's `exec_dying_with_parent` (skein's `shell.md`, 3) with the
    parent-death signal its caller names, Kill for forge run, so that a
    forge run killed takes Forgejo with it at once. Kill, not Terminate:
    a Forgejo still stopping gracefully when nothing waits for it would
    meet the restarted forge run's on the box's socket and its database
    (`crates/temper/tests/suite/startup.rs`,
    `killing_forge_run_alone_kills_its_forgejo_child`).
    Forgejo's graceful stop is forge run's forwarded Terminate. A
    failure, the parent changed or the program missing, prints `temper
    internal forge-child: WHY` and exits 1.
  - **Its shutdowns:** one that io names as Forgejo's (skein's `io.md`,
    section 7) is ignored, every time; every other is forwarded to
    Forgejo as the same Terminate or Interrupt. Forgejo's exit ends
    forge run with Forgejo's code, or 128 and the signal's number, so
    the box supervises forge run as it would Forgejo, and restarts it
    when Forgejo is killed. The decisions are a pure machine in
    `temper-forge-setup`; the shell only carries them out.
  - **On Linux only.** On macOS a shutdown names no sender (kqueue's
    `EVFILT_SIGNAL`) and no parent-death signal exists, so the box gives
    Forgejo's step no listener and Forgejo binds its own port, as above.
- **Replaces itself** (`execve`) otherwise: for the runner, and for
  Forgejo given no listener. It becomes `forgejo web` or
  `forgejo-runner daemon`, with the spec's arguments and the spec's
  environment alone: cleared, then `HOME` in the forge's directory,
  `PATH`, and git's configuration isolated, as the throwaway Forgejo's
  is. The lock crosses the exec as the process's standard input, so the
  process holds it for its life, and the process a host supervises is
  then Forgejo's or the runner's own, with its signals and its exit
  status.
- **Outside skein's shell, in one module.** skein's shell spawns but
  never execs, and its locks never pass to a child (skein's
  `programming-model.md`, 2.1; skein's `io.md`, section 5). So the lock and
  the exec are std's, `File::try_lock` and `CommandExt::exec`, with no
  `unsafe`, in one module of `temper-forge-setup-shell`, which the
  preparation's lock uses too, and which builds `internal
  forge-child`'s command for skein's parent-death exec; the
  preparation's commands and Forgejo's parent loop are not in it, and
  run on skein-io (6.3). It is the one place temper locks a file or
  uses `std::process` outside
  skein: a boundary test scans the production crates' sources (`crates/*/src`, test files aside) and
  fails the gate when either appears outside that module, but
  `std::process::ExitCode` in the binary's `main`. Test crates
  (`testing/`, `tools/`) are not scanned. It moves onto skein when
  skein's shell offers both (section 15).
- **The runner's spec:** its configuration file (its connection,
  capacity, labels, timeouts, its jobs' directory, no cache), its `PATH`
  from the setup. Its jobs run as the user that runs it (section 15).

### 6.5 Then the engine's start, and the person

- **The handover is the engine's,** for an owned or fake forge, at
  `temper serve`'s start after the records' load and the root's restore,
  or at the box's `started FORGE` (7.1). Whenever the handover file is
  present, even with a kept `AdminToken`, the setup host reads
  `admin_token` once by the service's io, as a private file beneath the
  forge's directory, within `records.value_bytes` (configuration.md,
  section 6). It runs `TakeOver { forge, admin_token }` (skein's
  `forge-setup.md`, 4.7): the token checked as a site admin's, kept as
  the forge's `AdminToken` record, and temper's admin's other tokens
  deleted. Only after `TakeOver`'s `Done` and the record kept does the
  host remove the file. It then runs `Reconcile { forge, adopted }` with
  the root's restored repositories (section 5). A fake then runs its
  start-time adoption (8.4), before `Request::Connect` with access built
  from the records (forge.md, 2.1, 2.2). The forge is
  connecting from the start or `started` until its `Connected`; its git
  value comes from its `GitToken` record (forge.md, 2.5).
- **A present handover is always taken over.** With an `AdminToken`
  record and no handover file, the host reconciles from the record. With
  the file present, `TakeOver` replaces the kept token and deletes the
  older tokens; taking the same token again is harmless. No owned or
  fake forge's `api_token`, `git_token`, `user` or `webhook_secret` file
  is written or read.
- **A step that stops blocks that forge's connect.** A `TakeOver`,
  `Reconcile`, `Create` or `Adopt` stop, a handover read that fails, or a removal
  that fails leaves the forge not connected, named by the plan's step or file
  operation, never a secret; at `started`, it is set up failed. A
  `TakeOver` stop leaves the file; a failed removal blocks reconcile.
  The row says not connected after a start; at `started`, set up
  failed with its `why`, the step and a short reason within the wire's
  256 bytes, a file operation naming the file as "forge N's
  admin_token". The operator's note names the plan's step and its
  reason, and for a file operation the file's full path; never the
  file's bytes. The next start or ask takes that handover over again. The
  engine and every other forge run on. "Set up again"
  asks the box, which answers a started group `started` at once (7.1),
  and the engine runs the same sequence again.
- **Then reconcile runs as managed's** (section 5), and makes no org:
  the set up's org is its first repository's, made by `Create` (6.7).
- **Then the first repository,** at the box's `started`, when the set
  up asked for one (6.7): `Create`, before the forge's connect.
- **A forge not prepared,** an owned forge's record with neither a
  handover nor an `AdminToken` record, is not set up: no access and no
  connect; every other forge starts as it would. The deployment's page
  (`../client/forge.md`, 5.4):
  - **Under the box,** offers "Set up a Forgejo here" while no owned
    forge is kept and the machine's part names the box's owned Forgejo
    (section 8). Its card says CI jobs run here unisolated, Forgejo
    stops with `temper box`, and its files are kept in temper's data
    directory. `SetUp` makes the forge's record, owned, numbered next
    (forge.md, 2.2), with its first repository's intent (6.7), in its
    decision, and the routing asks the box for the forge's group (7.1);
    at `started FORGE` the engine takes the handover, reconciles, makes
    the first repository (6.7) and connects; at `failed FORGE WHY` the
    page names the step and offers it again, "Set up again", on the
    same record. A box runs one owned Forgejo: a `SetUp` of a new one
    while one is kept is refused *not offered*, or *busy* while that one
    sets up.
  - **Without the box** (7.4), offers none: an owned forge's group is
    the box's, and `SetUp` is refused *not under the box*.
- **The page's actions** are the engine domain's offers, from the same
  function that gives the set up's offer (`../domain/forge.md`, 4.1).
  "Connect as its admin" makes a Forgejo elsewhere from the address the
  person gives (5.5).
- **The connected card** says "Connected as LOGIN", with its id on an
  Account line. Access is "Site admin, with temper's own token" for
  owned, "Site admin, with a token you gave" for managed, and "Its bot's
  tokens, which you gave" for connected. Owned's Runs line names Forgejo
  and its runner on this machine, their versions, and CI jobs running
  here as the person. Its First password line names the path to
  `person_password`, never its password, from set up on: the person's
  first sign-in, through temper too, links their account by it
  (`sign-in.md`, 9.4), and temper keeps no record of that sign-in by
  which to drop the line after it. A stopped owned
  forge says "Restart temper box: its preparation mends it". Owned and
  managed offer "New repository", with the set up's organization and
  repository fields (section 15).
- **The person** signs in on Forgejo's web as a site admin with
  `person_password`, and may change it when they like; makes
  repositories in their orgs, with Actions,
  and adopts them through temper's client. temper's admin is temper's; a
  person who deletes it makes the next start's reconcile and the next
  preparation stop, naming it.
- **The forge's place,** in its record (forge.md, 2.1), is the
  machine's owned Forgejo's (section 8): `api` its address, `transport`
  plaintext, `git` `http://<address>`.

### 6.6 Fetching

`temper forge fetch [DIRECTORIES] FORGE`, before an owned forge's
preparation, for a forge whose `forgejo` or `runner.binary` is
`"fetched"` (section 8): a once step of the box (7.1), and a command of
its own for any other host.

- **What it fetches,** each binary its setup gives as `"fetched"`, at
  temper's pins (6.1), for the machine's architecture, `ARCH`:

  | Binary | From | Into |
  |---|---|---|
  | Forgejo | `https://codeberg.org/forgejo/forgejo/releases/download/vVERSION/forgejo-VERSION-linux-ARCH` | `<cache>/forgejo/VERSION/forgejo` |
  | the runner | `https://code.forgejo.org/forgejo/runner/releases/download/vRUNNER_VERSION/forgejo-runner-RUNNER_VERSION-linux-ARCH` | `<cache>/forgejo-runner/RUNNER_VERSION/forgejo-runner` |

  `ARCH` is `amd64` or `arm64`; another machine refuses, naming its
  architecture, and its binaries are given by path.
- **One request, one answer.** Each URL answers `200` with the binary,
  never a redirect: skein-http follows none, and any other status
  refuses, naming the URL and the status.
- **A mirror,** the owned entry's `mirror`, absent by default (section
  8): an `https` origin for each binary, in place of codeberg.org's and
  code.forgejo.org's, its paths and digests the pins' still. A host that
  keeps its own copies names it; the process tier needs none, its
  binaries placed where the fetch looks (13.4).
- **TLS with the machine's roots,** read at the command's start as
  skein's shell reads them (skein's `shell.md`, 6.2): the bundle
  `SSL_CERT_FILE` names, or the first of the distributions' bundles that
  exists. skein-tls embeds none, so a machine with neither refuses the
  command.
- **The digests are pinned,** one SHA-256 per binary and architecture,
  in `skein-forge` beside `VERSION` and `RUNNER_VERSION` (skein's
  `forge.md`, 4.3), and required: a fetched binary is the build the
  conformance checked, byte for byte.
- **Streamed, never held:** the body's chunks are asked for one at a
  time, as the file takes them; each is hashed into the digest, which
  is incremental, and written at its offset into `<file>.part`, private,
  within `setup.fetch_bytes` (configuration.md, section 7). At the
  body's end, a digest equal to the pin makes the part file `0o755` and
  renames it over its name.
- **A wrong digest** removes the part file and refuses, naming the URL,
  the pinned digest and the digest it got. Nothing is left under the
  binary's name.
- **A file already there is hashed,** never fetched again: its pin's
  digest is nothing to do; another is refused, naming its path, and
  never overwritten, so a binary a person put there stays theirs to
  remove.
- **A part file an interrupted fetch left** is written again from its
  start.
- **Then the preparation checks its version** as every command that
  runs it does (6.1); a digest is no version.
- **Its code:** the fetch's machine in `temper-forge-setup`, its
  requests, chunks, writes and digests its inputs and outputs, and its
  host in `temper-forge-setup-shell`, on skein-io, skein-http and
  skein-tls (section 11). It holds no credential and writes no record
  (section 9).

### 6.7 The first repository

- **Asked with the set up, with no project.** `SetUp` carries an
  organization (default `work`) and a repository (default `hello`),
  and names no project: the repository is made unadopted, and its
  adoption is a later act's, into a project the person has
  (`../draft/first-run-projects.md`, 4). The forge domain keeps
  them in the forge's record (forge.md, 2.1), the set up's intent: one
  per forge, written in the decision that admits `SetUp`, and replaced
  by the next `SetUp` it admits.
- **Asked at a managed connect.** `ForgejoAdmin.first` carries the same
  organization and repository, with no project, and the person's runner label
  (default `docker` in the client and ctl), kept in the forge's record
  as `SetUp`'s are. After Reconcile and its
  `AdminOrigin` write are kept, Create makes the repository
  before the admin token is kept and the forge connects (5.5). A missing
  organization is owned by the admin Create has verified. No listener is
  supplied, so Create adds no hook.
  The forge's row carries `admin`, the string form's admin-connect offer;
  `first.label`, the runner label; and `ended_stop`, an optional
  `AdminStop`: `NotAdmin`, `Refused { status, stage: SetupStage }`,
  `Unmarked` or `Other`. `SetupStage` names admin, bot, token, org,
  repository, file, app, installation or state. `ended` is the separate
  diagnostic; the client chooses its stop's words from `ended_stop`.
- **Made after reconcile.** At the box's `started FORGE`, or at a start
  while the record keeps an intent whose repository is not made, once
  `TakeOver` and `Reconcile` are done (6.5), the setup host runs
  `Create { forge, org, repository, files }` (skein's `forge-setup.md`,
  4.9) by the admin's token: the org, private and owned by the person;
  the repository, private, with Forgejo's README on `main`; the bot a
  collaborator with `write`, and its hook on the webhooks' listener;
  then the starter workflow, below. Then the forge connects, as before
  (6.5). Each step finds what is already there and leaves it, so a set
  up asked again finishes what the last one left. The engine's limits
  for it are 2 files of at most 4 KiB each, the starter workflow and,
  for the playground's copy, its `src/main.rs` (skein's
  `forge-setup.md`, 4.1).
- **The starter workflow,** `.forgejo/workflows/ci.yml`, its commit
  the bot's, run on a push to `main` and on each pull, on the owned
  runner's label, `temper-host`. At a managed connect it uses `first.label`
  instead: 1–64 ASCII letters, digits, `.`, `_` or `-`, checked at the wire
  and again before the YAML is made. The workflow is otherwise the same:

  ```
  on:
    push:
      branches: [main]
    pull_request:
  jobs:
    check:
      runs-on: temper-host
      steps:
        - run: echo "temper's starter check passed; replace this step with your own checks"
  ```

  Its push is the repository's first CI run, on the owned runner: the
  owned forge enables Actions (6.2), and Forgejo gives a repository
  made through its API the Actions unit (skein's `forge-setup.md`,
  section 8). The step needs no network and no action from elsewhere.
- **The playground's copy,** asked with `SetUp`'s `playground`
  (`../client/tools.md`, 5.5, `forge setup --playground`): the first
  repository is `playground/widget`, its organization and name fixed,
  made with the playground's `src/main.rs` (8.4, `temper-fake`'s
  constant, one source for both) beside the starter workflow, in one
  Create, so that the playground's three suggested requests (8.4) are
  asked of a real forge and a real model. Forgejo's README stays, since
  Create keeps a file already there (skein's `forge-setup.md`, 4.9);
  the fake's `check.yml` is not copied, since the box's runner runs the
  starter. ctl's call adopts it, as below, into a new project named
  `widget` unless the call names one. A set up asked again takes its own request's fields, with
  the flag or without: a first repository already made stays, and the
  new one is made beside it.
- **The row says it,** the forge's first repository beside its
  connection (`../client/forge.md`, 5.4; `../client/README.md`, 4.6):
  none; making `ORG/NAME` from the `SetUp`'s admission; made
  `ORG/NAME`, unadopted, once `Create` is done; or failed
  `ORG/NAME`, at the step that stopped (the admin, the org, the
  repository with its access, or the file). A `Create` that stops
  blocks the connect as any step does (6.5): the forge is set up
  failed, its `why` naming the step, and "Set up again" asks with the
  same fields. A restart reads the row's first repository from the
  record, making `ORG/NAME`, until `Create` answers again.
- **Adopted by ctl's call alone.** The welcome and `ConnectForge`
  adopt nothing: the made repository waits, unadopted, for the person
  to adopt it into a project of theirs from the adoption form
  (`../client/README.md`, 3.5 and 4.4), one choice away.
  `temper ctl forge setup` and `forge connect … --first` hold their
  `--project` in their own run (`../client/tools.md`, 5.5): once the
  row says made, the call sends, under the same party, jig's
  `MakeProject` for a new project, its name and policy 1, the default
  document's (8.3), when `--project` names no project, then `Adopt`:
  the project, the forge, `ORG`, `NAME`, landing on `main`, owned, the
  project's home, with CI. Nothing in the engine reads a project for
  it.
- **A restart before the repository is made** keeps the intent, since
  the record holds it: the start's sequence, or the box's `started`,
  makes the repository at the next connect, after reconcile (6.5).
- **A set up writes as it goes:** each act and plan keeps what it makes
  when it makes it, the forge's record and its intent at `SetUp`'s
  admission, each credential record as its plan writes it (5.1), and the
  box's group file and record as the group starts (7.1). Nothing is
  staged in memory to be written at the end, and no write spans two
  stores.
- **Found later, it starts over.** A set up cut short, by a restart or a
  failure, leaves its forge kept but not connected, or its first
  repository not made: the welcome's Forge step shows it unfinished,
  "FORGE · not finished", with Start over (`../client/README.md`, 3.5),
  which removes the forge (`../domain/forge.md`, 4.1) and opens the
  step's choices again. An owned Forgejo then sets up afresh, as a new
  forge numbered next: its binaries from the cache (6.6), which its
  removal keeps, and its directory prepared anew (6.3), the old one
  removed with its forge (7.1).
- **GitHub goes on by default:** its App's records are kept as they come
  (17.2), so the step offers Go on, to the App's installation, before
  Start over, which removes the forge and names the App on GitHub as the
  person's to delete (`../domain/forge.md`, 4.1).
- **An account added before** stays: it is the person's, not the set
  up's, and the Model step shows it ready (`../client/README.md`, 3.5).
- **Its code:** the intent, in the forge's record, in
  `temper-engine-domain-forge`; `Create`'s start and its end in
  `temper-service`'s setup; the starter workflow's bytes and the plan's
  limits in `temper-forge-setup`.

### 6.8 Signing in through temper

`temper forge sign-in [DIRECTORIES] FORGE`, for a Forgejo of
`application.sign_in.forges` (`sign-in.md`, section 9), managed or
owned, on temper's host, before `temper serve`:

1. **The provider's key,** when its file, `engine.clients.provider.key`,
   is missing: `openssl genpkey -algorithm RSA -pkeyopt
   rsa_keygen_bits:3072`, run with umask 077 into a file beside the
   key's and renamed over it, so the key is PKCS#8 PEM, mode 0600 and
   the service's own user's. It is never printed, copied or logged; a
   key there is left as it is. No `openssl` on the `PATH` refuses the
   command, naming it.
2. **The forge's client secret,** when its file, the forge's client's
   `secret`, is missing: 32 bytes of the system's randomness in
   base64url, written whole and private (section 3). A secret there is
   left as it is.
3. **It prints,** never running them: the `add-oauth` command
   (`sign-in.md`, 9.5) with the client's id, the secret file's absolute
   path in `--secret "$(cat PATH)"`, never the secret's bytes, and the
   discovery document's URL at temper's public origin, or its listener's
   with none. The source's callback is the forge's browser origin's
   `/user/oauth2/temper/callback`: its owned setup's `public` when present,
   else its `git` URL (`sign-in.md`, 9.1 and 9.7), matching Forgejo's
   `ROOT_URL` (6.2). For an owned Forgejo, the command has the binary and
   its `--work-path` and `--config`; and Forgejo's settings of
   `sign-in.md`, 9.4, which an owned Forgejo's `app.ini` already holds
   (6.2), and a managed one's owner
   writes.

Then `temper serve`, and the printed command, once, on Forgejo's host.

- **Printed for an owned Forgejo too, not run** (with o/sign-in,
  2026-10-23), for two reasons: `add-oauth` fetches temper's discovery
  document at once and adds no source when temper does not answer, and
  the preparation runs before `temper serve` (6.3, 7.2); and `add-oauth`
  takes its client secret only as an argument, where the preparation
  puts no secret (6.3). A source added by Forgejo's command line while
  Forgejo runs works at once, so the order costs a person nothing.
- **Its code:** the command, and the key's and the secret's making, in
  `temper`'s `forge_sign_in.rs`, dispatched from `forge.rs`; the printed
  bytes in `temper-forge-setup`'s `sign_in.rs`.

## 7. The box: wiring only

### 7.1 What it runs

`temper box [DIRECTORIES] [--echo none|steps]` runs its steps, which
`temper-forge-setup`'s `kinds` and `temper-fake`'s fake slot give from
the document (section 8), each a command of the `temper`
binary, as a child process given the box's three directories as flags,
`--config C --data D --cache K`, each absolute (configuration.md,
section 12), written `DIRS` below, `N` a forge's number:

```
temper box
│                                     the fake kind's group, the playground's fakes: asked (below)
├── temper fake llm DIRS              long (8.4)
├── temper forge prepare DIRS N       once: the fake forge's handover
├── temper fake forge DIRS N          long (8.4)
│                                     the owned kind's group, an owned Forgejo's: asked (below)
├── temper forge fetch DIRS N         once, when its binaries are fetched (6.6)
├── temper forge prepare DIRS N       once
├── temper forge run DIRS N forgejo   long: Forgejo's parent on Linux with fd 3 (6.4)
│   └── forgejo web                   via temper internal forge-child; execs without fd 3
├── forgejo-runner daemon             long: temper forge run DIRS N runner
└── temper internal serve DIRS --box  long: its own tree (README.md, section 4); its setup
                                      host takes each handover and reconciles (section 5);
                                      its listeners from fd 3, then its pipe to the box (below)
```

- **An empty configuration directory,** absent or with no entry, is
  written first by `temper init` (8.2), in the box's own process, before
  the document is read; the box says so on its terminal, `First run:
  wrote the configuration, <path>` (7.2), and goes on to its steps.
  Otherwise the box
  never writes the document: a configuration directory with entries but
  no document is refused as every command refuses it.
- **The box knows kinds, not forges.** It has a group for each kind
  the document's machine part gives it (section 8): the owned kind's,
  an owned Forgejo's fetch, preparation, Forgejo and runner
  (`temper-forge-setup`'s steps), when the machine names the box's owned
  Forgejo; and the fake kind's, the playground's fake LLM, a fake
  forge's preparation and the fake forge (`temper-fake`'s), when it
  names the fakes and the document has `application.demo`. Each is
  dormant until serve asks for it for a forge, and then runs for that
  forge alone, its steps' forge number that forge's: a box runs at most
  one group of each kind. It reads no store and names no forge of its
  own: it stays wiring only.
- **A forge's group, asked.** Under the box, serve writes the group's
  settings, the forge's kind, its setup entry (section 8) and its port,
  to `<data>/box/groups/N`, private, then asks `start N` on the pipe:
  at an owned forge's set up (6.5) and at the playground's start (8.4).
  The box builds the group's steps by the kind from that file, and runs
  them in order; `temper forge fetch`, `prepare`, `run` and `remove`
  read forge N's setup from the same file, and refuse a forge with none.
  Once the group's long steps have started and, for an owned Forgejo,
  Forgejo answers, the box writes its record, `<data>/box/started/N`, one empty file, and
  answers `started N`. `box`, `box/groups` and `box/started` are made as
  needed, 0o700, their files 0o600. A record that cannot be written is
  named on the box's stderr, and the group answers `failed N record not
  written` and is dormant again.
- **At a later start,** the box lists `box/groups`, at most `box.groups`
  files (configuration.md, section 7), and runs each group whose
  `started` record exists from its file, in its place, as an unasked
  group, before `temper serve`; a group with a file and no record stays
  dormant until asked. A file it cannot read, or a record it cannot look
  at, is named on its stderr, and its group stays dormant; of two
  started groups of one kind, the one of the higher number runs, and the
  other is named and stays dormant.
- **A forge's group, removed.** At a forge's removal
  (`../domain/forge.md`, 4.1) serve asks `remove N`, and again at each of
  its starts until it is answered `removed N`. The box stops the group's
  steps as it stops a group (7.2), then runs its remover, `temper forge
  remove DIRS N`, a once step, which removes the forge's directory,
  `<data>/forge-N` or `<data>/playground-N`, and refuses any path
  outside the data directory or within the cache, so the cache, and
  Forgejo's fetched binaries in it, stay (6.6). The box then removes
  `box/groups/N` and `box/started/N`, and answers `removed N`. For a
  forge with no group's file it knows, it removes those files alone. A
  removal that fails answers `failed N WHY`, `WHY` one of `remove:
  exited <code>`, `remove: did not start` and `remove: files not
  removed`, and the group is dormant again, to be asked again.
- **The listeners are the box's,** every step's, from the box's start
  to its end, a dormant group's among them: so that the port a step
  reports is its own from before it starts, and a second first run's
  init finds the first one's ports taken (8.2). Each step names its
  listeners, the socket addresses the document gives it (7.3): `temper
  serve` the clients' and the webhooks' listeners, the fake LLM and the
  fake forge their ports, and, on Linux, `temper forge run N forgejo` the
  owned Forgejo's port, the machine's (section 8; 6.4). On a
  first run the box keeps the sockets `temper init` chose (8.2); on a later
  start it opens each at the document's address before any step, with
  skein's `open_listener`, and a port taken refuses its start, naming
  the port and the document, as any start is refused. A box started with
  listeners of its own (`LISTEN_FDS`, systemd's convention, as `temper
  internal serve`, `temper fake` and `temper forge run` read it: skein's
  `shell.md`, 6.6) takes each one given for a step's address, matched by
  its address, and opens only the rest, so a person's socket activation
  and a test's held port reach the box's steps as they reach serve; one
  given for no step's address refuses its start, naming it. It holds each
  one, never accepting, and gives it to its step at every spawn, a
  restart's among them, as the descriptors 3, 4, … in the step's order,
  with `LISTEN_FDS` set to their count in the step's environment and
  dropped from its own: systemd's convention, which skein's shell reads
  (skein's `shell.md`, 6.6) and Forgejo reads as well (6.4). The step's
  own `Listen` for that address then takes the socket it was given
  (skein's `io.md`, 3.2), so nothing above io knows it was given one. A
  step with no listener is given none, and no `LISTEN_FDS`. On Linux,
  `temper forge run N forgejo` is given the owned Forgejo's port and
  stays Forgejo's parent (6.4); on macOS it is given none, and a fresh
  box closes the reservation `temper init` made for its port, so
  Forgejo binds it itself. A listener at port 0 is not held: its step binds it, as without the box, since a
  port the kernel chooses can't collide. A held one could not be claimed
  either, since the step's `Listen` names port 0, not the port chosen.
  It changes at each restart, as it does without the box. `LISTEN_FDS`
  counts only the sockets given.
- **The pipe,** two of them, made again at each spawn of `temper
  internal serve`, at the two descriptors after its listeners: 3 + *n*,
  serve to box, and 4 + *n*, box to serve, *n* its listeners' count, so
  fds 5 and 6 under the default document's two. `--box` tells `temper
  internal serve` to adopt them, and a serve given `--box` whose
  descriptor 3 + *n* or 4 + *n* is not an open pipe refuses its start,
  naming the fd, *n* being the `LISTEN_FDS` it was given, 0 without it;
  without `--box`, as under every other arrangement (7.4), serve has no
  pipe and leaves those descriptors alone. One line each, ending in a line
  feed, `FORGE` the forge's number:

  | Line | From | Says |
  |---|---|---|
  | `start FORGE` | serve, once it has written the group's file: at an owned forge's set up (6.5) and at the playground's start (8.4) | start the forge's group, by its file |
  | `step FORGE STEP` | the box | the group's step `fetch`, `prepare` or `run` is starting, for the client's row (`../client/forge.md`, 5.4) |
  | `started FORGE` | the box | the group's long steps are started, an owned Forgejo answers, and its record is written; the engine then takes the handover and reconciles (6.5) |
  | `failed FORGE WHY` | the box | the group stopped: `WHY` names the step by its name, as its log file and the box's terminal name it (7.2), such as `prepare-3 exited 1`: `<step> exited <code>`, `<step> killed by signal <n>`, `<step> did not start`, `record not written`, or a removal's `remove: exited <code>`, `remove: did not start` or `remove: files not removed`, cut to fit the line's bound, and holds no secret; a step's own lines are in its log, the last of them under the terminal's `failed` line (7.2) |
  | `stop FORGE` | serve | stop the forge's group, its file and record kept |
  | `stopped FORGE` | the box | the group's steps are closed, as the box's own stop closes them (7.2), and the group is dormant again; its file and record stay |
  | `remove FORGE` | serve, at the forge's removal (`../domain/forge.md`, 4.1), and at each of its starts until answered | stop the forge's group and remove its directory, its file and its record (above) |
  | `removed FORGE` | the box | the group's steps are closed, and its directory, its file and its record removed; the cache is kept |

  One ask at a time: a `start` for another forge while a group starts,
  or while a failed group's steps close, is answered `failed FORGE busy`;
  one for the forge whose group is starting, or whose failed steps
  close, by that group's own lines, its `failed` once they have closed;
  one for a forge with no group, `failed FORGE not asked`; one for a
  started group, `started FORGE`; one while the box stops,
  `failed FORGE stopping`. A `stop` is an ask as a `start` is, one at a
  time: for a started group, the box closes its steps, then answers
  `stopped FORGE`; for a dormant one, `stopped FORGE` at once; for a
  group starting, or whose failed steps close, `failed FORGE busy`; for
  a forge with no group, `failed FORGE not asked`; and while the box
  stops, `failed FORGE stopping`. A `remove` is an ask as a `stop` is,
  answered `removed FORGE` where a `stop` is answered `stopped FORGE`,
  once its remover has run and its files are removed; for a forge with
  no group's file the box knows, it removes the files and answers at
  once. While a group's steps close at its `stop` or `remove`, or its
  remover runs, a `start`, a `stop` or a `remove` for any forge is
  answered `failed FORGE busy`. An
  answer to a `temper serve` that has exited is dropped: its restart
  finds the handover at its start (6.5). Each line is at most
  `box.line_bytes` (configuration.md, section 7), which both sides
  read within; another line, or a longer one, is named on its reader's
  stderr and ignored. serve's side, its asks, its reads and the states
  they tell the engine's root, is the routing's box link (forge.md,
  2.8).
- **A group's failure** before `started` answers `failed`, and the group
  is dormant again, to be asked again; the box runs on. Once started,
  its long steps are restarted, and given up on, as 7.2 says.
- **A first run's steps** (8.3): `temper serve` alone runs at the
  start; the fake kind's group is dormant until the person tries the
  playground (8.4), and the owned kind's until the person sets up an
  owned Forgejo (6.5). No step is a link: serve's first line is the
  person's.

### 7.2 What it does

- **In order:** each step starts once the one before it has started, when
  that one is long, or has exited 0, when it is once or link. A link
  step starts once the long step before it has written its first line
  to its stdout.
- **A link step** is given that line whole on its stdin, which is then
  closed; it is the link step's alone, and the box's stdout shows the
  long step's lines after it. A long step that exits before writing a
  line hands its restart's first line to the link step. A link step
  runs once per box: the long step's restarts never run it again.
- **Restarts** a long step that exits, after a backoff doubling from
  `box.backoff_ms` to `box.backoff_max_ms` (configuration.md, section 7),
  first running again the steps that are once just before it: a Forgejo is
  restarted after its preparation, so a change of the setup reaches its
  `app.ini`. `temper serve` restarts on its own, with a pipe made afresh,
  and its start takes a handover the preparation left and reconciles each
  managed and owned forge. A token revoked under a running engine stops
  that forge alone, named, and not the engine (forge.md, a forge stopped);
  the engine's next start makes it again. A fake forge's restart is such a
  revoke: it serves its seed afresh, without the bot, so that forge stops
  alone until the engine's next start takes the handover its preparation
  drew again (8.4).
- **Gives up,** exit 1, naming the step, when a once or link step fails,
  but a dormant group's before its `started`, which answers `failed`
  (7.1), or a long one exits more than `box.restarts` times within
  `box.window_ms`. The backoff is counted from a step's exits within the
  window, so it starts again from `box.backoff_ms` once a step has run a
  window without exiting. Its stop treats a step still starting as the box's
  stop does.
- **Stops** at a first termination signal: the long steps in reverse
  order, each by a signal, `box.grace_ms`, a second signal, which turns
  `temper serve`'s drain into an abort (README.md, section 4),
  `box.grace_ms` again, then a close. The machine knows a step only as
  once, long or link, so every long step, and a link step still
  running, is sent both; a dormant group's steps run nothing, and are
  sent nothing. A second termination
  signal closes them all. A step the box has not yet asked to start when the
  stop begins is not started. One it has asked for is waited on in its place
  in that order, and a child that starts then is stopped as above, or closed
  at once after a second termination signal; it is never readied, continued,
  recorded or restarted. The box cannot interrupt the system's creation of a
  process: that call blocks the box's one loop for as long as it lasts,
  without bound on a filesystem that stalls (skein's `kernel.md`, section 6),
  and a signal or a deadline is acted on only once it returns. It exits 0
  after a stop it was asked for.
- **Stops a group** at `stop FORGE` or `remove FORGE` (7.1): the group's running steps
  alone, as the box's stop does, in reverse order, each by a signal,
  `box.grace_ms`, a second signal, `box.grace_ms` again, then a close;
  a step that exits during it is not restarted, and a once step still
  running is closed with them; a step still starting is treated as at the
  box's stop. Once they are all closed the group is dormant, the box
  says `<label>: stopped` on its terminal (below), and answers
  `stopped FORGE`, or at a `remove` runs the group's remover (7.1), and
  runs on; a termination signal during it
  stops the box as above, which closes them too.
- **Its stdout** carries `temper serve`'s stdout alone, the sign-in
  links, but for its first line when a link step follows it, which is
  that step's (8.4). The box reads every step's output a line at a
  time, and writes a line only once it holds it whole, to its line feed
  or its step's end, so that nothing written to its stderr lands inside
  a link when both streams are one terminal. A terminal-bound serve URL has
  at most 4096 payload bytes, excluding its line feed; an EOF fragment within
  that bound receives a line feed. An oversized URL is discarded through its
  line feed or EOF and named once on stderr, without printing any of its
  bytes; the next line can be read normally. This bound covers the configured
  public origin and sign-in path. The legacy first-line handoff to a link
  step keeps its existing bounded-piece semantics and receives no sign-in
  block. Other child streams retain bounded pieces, each completed with a
  line feed at the piece bound or EOF.
- **Its terminal,** its stderr, is quiet: the box's own lines alone.
  Before the first sign-in block (below), `First run: wrote the
  configuration, <path>` when `temper init` ran (7.1), `Logs:
  <data>/box/logs/`, and `Starting <step>` as each step of `always`
  starts. After it, each line begins with the time of day in UTC,
  `HH:MM:SSZ `, since the box reads no time zone:
  - for an asked group, named by its label (7.3): `<label>: starting`
    at its `start`, then `<label>: started`, `<label>: failed, <WHY>` and
    `<label>: stopped` as the pipe says them; its `step` lines are the
    client's row's alone; each start of an owned forge, an asked
    group's `started` or the box's own start of one that starts with it,
    is followed by its hands (`../client/tools.md`, 11.5): `<label>: api
    http://ADDRESS/api/v1` and `<label>: person LOGIN, read header PATH,
    write header PATH`, the files' paths and never their contents;
  - a long step's exit, `<WHY>; restarting in <n> s`, and the box's give
    up, `<WHY>; giving up`, `<WHY>` in `failed`'s words (7.1);
  - `Stopping` at a first termination signal, and `Stopped` once every
    step is closed;
  - and the box's own refusals and failures, as this section and 7.1
    name them: a record, a pipe's line, a log.

  `--echo steps` (`TEMPER_ECHO`; the default, `none`; skein's
  `shell.md`, 6.5) also writes every line each step writes, but serve's
  stdout, prefixed `<step>: `, as it reaches the step's log.
- **The sign-in block:** at `temper serve`'s first link, the box
  writes the lines below, each once the one before it is written, so
  that a terminal shows them in order: the link alone on stdout, the
  rest on stderr.

  ```

  Open this link in a browser on this machine. It signs you in as this
  temper's owner, once and for a short while, so keep it private.
  Opened again once spent, it prints a fresh one here.

  http://127.0.0.1:8367/jig/v1/sign-in/link?secret=...

  Ctrl-C stops temper.

  ```

  Each later link, a fresh one after a spent or expired one was opened
  (jig's `protocol/client.md`, section 8) or a restarted serve's first,
  comes after `<time> A fresh sign-in link:` on stderr. A line given to
  a link step has no block.

  One bounded FIFO owns whole terminal items tagged stdout or stderr. It
  admits the first block as three adjacent items (words, URL, Ctrl-C words),
  or a fresh block as two (announcement, URL), before consuming that URL.
  With N steps, at most max(3, 2N + 1) items are outstanding, including the
  active item. Only one item is in flight across both descriptors: after
  sending it, the host demands the target's entire io output budget, and
  its Room acknowledgement proves the write finished before the next item
  starts. A small write grant alone is not that acknowledgement.

  Full output pauses new child read demands; already admitted reads retain
  their bounded per-pipe chunk. Policy orders and their source wall times
  grow in paired internal FIFOs. Ordinary events keep their relative order
  while terminal acknowledgements may bypass a front blocked on request
  credit. A control command waits until earlier orders and control replies
  have dispatched; a record diagnostic enters the notice FIFO before its
  policy answer, and the record itself settles before started is announced.
  Captured ordinary events precede policy timers. An unsent spawn respects
  a queued withdrawal even when a terminal notice holds it back.
  Runnability and deadlines wait on the real destination's credit, so
  blocked output does not spin the loop. Its internal queues grow as they
  fill; what bounds the box is at its edges: terminal entries, pipe bytes
  and kernel operations, as above.

  The machine emits typed terminal facts, without words or wall time:
  Starting for an always step; a requested group's starting, started, failed
  and stopped; a step's restart or the box's give up; Stopping and Stopped.
  Group facts identify the group, and failure facts retain the exact step
  identity and structured reason (exit code, signal, not started, or record
  not written). A restart also carries the chosen backoff duration. Labels
  and step names are looked up by identity, never copied into policy facts.
  Numeric pipe replies keep their existing WHY and settlement semantics.

  A requested group's starting is emitted once when an accepted start
  begins its work; repeated asks that only repeat a pipe reply emit no new
  lifecycle notice. Its failed and stopped notices follow actual group
  settlement, as their pipe replies do. A restart notice is emitted when
  the exit selects its backoff, before its timer fires. Giving up is emitted
  once when decided, before failure teardown, and is not repeated when the
  final outcome is collected. An exit during an intentional stop emits
  neither restart nor give up. Retry preparation failures name the actual
  once step that failed. A delay is written in seconds, preserving a
  fractional part when the configured millisecond delay is not whole seconds.

  A pure terminal formatter receives the host's supplied wall time, step
  names and group labels, and owns every lifecycle word. Starting names
  always steps; Stopping is emitted once on the first requested stop, and
  Stopped after actual settlement.
  Timestamping starts once the first visible block is admitted, and survives
  serve restarts; a refused URL or legacy handoff does not start it. A line's
  completion wall time is retained across output backpressure, rather than
  replaced with its later write time. Policy facts retain their decision's
  supplied wall time across order and terminal queues. Their admission and
  item-size bounds include labels and the longest failure/restart line.
- **Its logs,** `<data>/box/logs/`, mode 0700, made as needed: one file
  per step, its name with each space a `-` and `.log` after it
  (`serve.log`, `prepare-3.log`), mode 0600, holding every line the
  step writes to its stdout or its stderr, each after the time the box
  read it, `2026-10-23T14:03:20Z `, but serve's stdout: no file holds a
  sign-in link. At its start the box moves the logs a box before it
  left to `<data>/box/logs.previous/`, replacing that one, so the last
  two runs are kept. A start cut short after the move keeps
  `logs.previous/`; the box removes nothing else and follows no link.
  A file that would pass `box.log_bytes` (configuration.md,
  section 7) moves to `<name>.log.1`, replacing that one, and starts
  afresh, so a step's logs stay within twice the bound.
  A record is never split: a file moves to `.log.1` before the record
  that would take it past the bound, so the bound holds at least one.
  A log the box cannot write is named once on its terminal, `box: cannot
  write <path>: <error>; the step's lines follow here`, and that step's
  lines then go to the terminal as `--echo steps` writes them. Forgejo also
  logs to its own files.
- **A failure's tail:** under each terminal line of a step's failure,
  a long step's exit, a group's `failed` or a give up, the box writes
  the last `box.tail_lines` lines it logged for that step since its last
  start, held in memory, each as `--echo steps` writes it, indented four
  spaces, without its time, then `Log: <path>`, indented the same.
  With `--echo steps` it writes no tail, the lines already shown.
  A failure that is neither a step's exit nor its readiness, a step
  that logged nothing since its start, or a log not written has no tail.
- **Its environment:** each step gets the box's; `temper forge run`
  clears it for Forgejo (6.4).
- **Its children** are skein-io's (skein's `io.md`, section 6), each in a
  process group of its own; closing one kills its group, so nothing the
  box started outlives its stop.

### 7.3 What it names

- **Steps only:** each step's name, its arguments and whether it is
  once, long or link; and which of them are a kind's dormant group,
  bound to a forge by its number alone at its ask. Its policy is a small
  machine of its own (steps, exits, timers and the pipe's lines in;
  spawns, signals, closes, the pipe's lines and the groups' files and
  records out), hosted on skein-io.
- **The steps in groups:** `Steps { always, asked }`, `always` the steps
  run in order, and `asked` a list of `Group { kind, forge, before,
  steps, remover, recorded, label }`, each a kind's group, dormant until
  asked for a forge (7.1). Its steps name the forge by a placeholder,
  `FORGE_ARGUMENT`, which the box replaces with the forge's number when
  it binds the group to it, its steps' names taking the number too;
  `remover` is the once step its removal runs (7.1); and its `started`
  writes the box's record. `label: Box<[u8]>` is the words the terminal
  names it by (7.2), owned opaque bytes passed from the setup/fake
  constructor into the box's layout and written verbatim: `Playground`
  for the fake kind's, `Forgejo` for the owned kind's. The box never
  infers a label from arguments; it adds no label field to a wire reply
  or stored record. `before` is its place in the order:
  the index in `always` it runs just before (its length puts it last).
  Each step carries its `Stage`, whose word its `step` lines carry, and
  its listeners, the socket addresses the box holds and gives it (7.1). The
  machine's step tests drive a group from dormant to `started`, and to
  `failed`, a started one to `stopped`, and one to `removed`, by lines
  and exits alone (13.1).
- **The binary hands it the steps.** `temper box` resolves its three
  directories, runs `temper init` when the configuration directory is
  empty (7.1), checks the document as section 8 says, asks
  `temper-forge-setup` for its kinds' steps, `temper serve` and the
  owned kind's group, and `temper-fake` for the fake kind's group (8.4),
  and gives them to `temper-box`, each asked group apart.
- **Welcome peers first:** the fake-backend configuration supplies an
  always-running prefix before the ordinary steps. Every asked group's
  insertion point, the playground's included, follows that prefix.
  An empty prefix leaves the ordinary order unchanged. Stopping or
  removing an asked group never stops these peers.
- **Checked:** a boundary test fails the gate when `temper-box` depends
  on a crate named `temper-*` but `temper-forge-setup`, or names an item
  of it but `steps`, `Steps`, `Group`, `Stage`, `Step` and its `Kind`,
  `FORGE_ARGUMENT`, and of its kinds and group's files `BoxKind`,
  `group_path`, `group_file` and `parse_group_file`.

### 7.4 Other arrangements of the same commands

None has the box's pipe: `temper serve` runs without `--box`, and runs
no owned forge. An owned forge's group is the box's: `SetUp` of an
owned forge is refused *not under the box* (6.5), and `temper forge
fetch`, `prepare`, `run` and `remove` find no group's file for a forge
the box did not run (7.1). GitHub and a Forgejo elsewhere connect as
under the box.

- **systemd,** a unit for `temper box`, whose own stop stops its tree
  (7.2), or one for `temper serve` alone; `temper serve`'s
  unit keeps systemd's default kill of its control group, which ends a
  crashed service's whole tree before its restart (skein's `shell.md`,
  13);
- **the process tier** (section 13.4), which runs the commands itself,
  writing a group's file as serve does under the box;
- **a container image** whose entry point is `temper box`, every process
  of its tree ending with the container.

### 7.5 The welcome's fake backends

temper box --fake-backends [--bundle DIR] [DIRECTORIES] prepares the retained isolated
sandbox of configuration.md, 12.1. Ordinary children read the effective
temper.json and agent.json through their effective directory flags; peer
commands read absolute effective document paths. Changing only the parent's
parsed bytes is insufficient.

Build the loopback bundle once, then start even with empty configuration:

    scripts/bundle.sh --loopback DIR && temper box --fake-backends --bundle DIR

The flag overrides an init-shaped source's application.clients.bundle;
without it, that configured bundle is used. Missing both refuses with the
bundle command and --bundle. Empty C stays untouched: init writes directly
into the effective sandbox. Box startup performs no build or fetch.

Before starting its supervised peers, preparation prints the effective
directories, the fakes' origins and the files a walk pastes
(configuration.md, 12.1).
GitHub has one fixed person and repository. Its App 1 is seeded and
installed, with the setup URL a person gives an App of their own (17.3),
`http://<listener>/forges/1/github/setup`, the number GitHub's forge
takes when it is the first forge a walk makes, so that the own-App path's
Install comes back to temper; App 1's private key is D'/github-app.pem,
mode 0600. The dummy model key is the key a walk pastes. The elsewhere
Forgejo's admin token is kept in its private file: instructions name only
the path, and no log, report or record contains its contents. The fixed signed-in model bearer uses the first added
credential stream with the same seed as the issuer, while the primary
model key stays the API-key choice. Neither value is a real credential.

The standalone issuer command is:

    temper fake oauth --document FILE [--ready PREFIX] [--seed N]

Its bounded document has a loopback listen socket with a nonzero port,
redirect_port (the shipped 1455 by default) and redirect_path
(auth/callback by default). The listen socket determines its HTTP origin
and shipped authorization, token, revocation and keys paths. Unknown,
duplicate, ill-typed and oversized members are refused before any output
or listener: at most 16 KiB for the document and 1024 bytes per URL/path.

temper-fake decodes the document and configures skein's OAuth peer;
temper owns the files, signal handling, readiness and the real loop.
Authorization grants immediately with the shipped client/scopes/resource,
the fixed fake person, OpenID tokens/keys and the fixed bearer accepted
by the model. No login or consent page is added.

The wrapper refills its bounded token-plan queue each pass. Each issuer
launch admits at most 64 sign-ins, since the primitive retains used codes.
Exhaustion, or refresh after revoking the fixed refresh token, requires an
issuer restart. Generated configuration keeps callback port 1455; concurrent
human sign-ins from several local boxes are not promised. Tests name
explicit callback ports. Long-running peers retain no observation stream
unless one is explicitly requested.

The GitHub and model peers are long steps named `fake github` and
`fake model`, before ordinary steps and outside the playground's asked
group. Their arguments are `fake forge --document C'/github.json --seed 7`
and `fake llm --document C'/llm.json --seed 7`, respectively, with no
directory flags. The issuer uses the same seed, so its bearer matches the
model's first added key. A supervised issuer is `fake issuer`, using
`fake oauth --document C'/oauth.json --seed 7`; it requires the primitive's
inherited-listener support. It never substitutes a self-bound startup race
for that contract.

The box owns the peers' held listeners before spawning them. First
preparation transfers its reservations exactly once; on reuse the ordinary
box listener mechanism binds the retained documents' addresses. Each is
loopback with a nonzero port, distinct from the other peer ports and the
ordinary box listeners. Output paths stay within D'. The retained documents
are revalidated on both paths; edits are not silently replaced.

Held listeners provide the ordinary fake peers' readiness and accept
backlog. Restart, backoff, give-up and reverse-order shutdown use the
ordinary supervisor. The playground's reset ends no peer of these, so a
person's newly connected model or issuer runs on. Peer memory lasts one launch: a restarted peer
uses the same seed and output files, but does not recover in-memory Apps or
other changes. Effective configuration and temper's data remain retained.

Forgejo here remains owned and real. The machine's part points GitHub
at its fake (section 8), and names no forge: each is made by the walk's
own `SetUp` or `Connect`. Elsewhere is a Forgejo the walk connects as its
admin (5.5), at the address preparation prints.
Its separate `fake forgejo` long step uses
`fake forge --document C'/forgejo.json --seed 7`, with its held loopback
listener before ordinary steps and outside the playground's asked group.
The fixed peer prefix order is fake github, fake issuer, fake model,
then fake forgejo. Its retained token output is D'/forgejo-admin.token,
subject to the same private-file validation and seed-output reuse as the
other peers. Admission and reconciliation use the
ordinary configuration checks, without raised budgets or policy limits.
The program way uses a separately provisioned fake adapter
(configuration.md, 12.1) and the ordinary program-login mechanism. Only
after validating the adapter does the box print the exact program-login
command for its effective directories; a walker starts the box before using
it. No product path branches on the flag. A conflicting issuer listener is a
refusal, not a reason to bind another port or regenerate the retained
document.

## 8. The configuration

- **One document,** `temper serve`'s, `temper.json` in the
  configuration directory (configuration.md, sections 2 and 12). Every
  `temper forge` command and `temper box` read it whole, check it
  as configuration.md, section 8 says, `application.machine` among it,
  and act on their part: they read no file but their own part's.
- **It names no forge.** A forge is a record of the forge connector,
  made by the act that sets it up or connects it, and numbered next when
  it is made (forge.md, 2.1 and 2.2): its kind, mode, place, git
  program, bot and first repository's intent are its record's, carried
  by the act that makes it, `SetUp` for an owned Forgejo (6.5, 6.7),
  `Connect` for a Forgejo elsewhere (5.5) or GitHub (section 17), and
  `StartPlayground` for the playground's fake one (8.4). A first run's
  document names none (8.3).
- **The document keeps the machine's:** the clients' and the webhooks'
  listeners; in `application.machine` (configuration.md, section 3),
  what the forges a box can run take from this machine, the box's owned
  Forgejo's port and the fakes' ports among it, which `temper init`
  chooses and the box holds from its start (7.1, 8.2); and the limits,
  whose forge members count the deployment's forges and repositories,
  never a sum over its forges (forge.md, section 7).
- **`application.machine`,** each of its parts absent, or null, for
  none:

  ```
  "machine": {
    "owned": {
      "forgejo": "/opt/forgejo/forgejo",
      "runner": {"binary": "/opt/forgejo/forgejo-runner",
                 "labels": ["temper-host:host"], "capacity": 1,
                 "path": "/usr/local/bin:/usr/bin:/bin"},
      "listen": {"socket": "127.0.0.1:3000"},
      "remote": "http://127.0.0.1:3000",
      "person": {"login": "ada", "email": "ada@example.invalid"}},
    "fake": {
      "listen": {"socket": "127.0.0.1:8369"},
      "remote": "http://127.0.0.1:8369",
      "git": "demo/bin/git"},
    "github": null}
  ```

  | Member | Type | Bounds |
  |---|---|---|
  | `owned` | the box's owned Forgejo | its members below; the owned kind's group (7.1) and the offer to set one up (6.5) need it |
  | `owned.forgejo`, `owned.runner.binary` | cache paths (configuration.md, section 12), or `"fetched"` | checked against the pins of 6.1; `"fetched"` names temper's pinned build in the cache, which `temper forge fetch` writes (6.6) |
  | `owned.mirror` | absent, or `{"forgejo", "runner"}`, each an origin | each `https://HOST` or `https://HOST:PORT`, no path: where `temper forge fetch` asks for the pinned paths (6.6) |
  | `owned.public` | absent, or an origin | `https://HOST` or `https://HOST:PORT`, no path: the address people reach this Forgejo at, behind a proxy; written as `ROOT_URL` and `DOMAIN` (6.2) |
  | `owned.runner` | the runner's members, or null for none | `capacity` at least 1; `labels` at least one, at most 8 |
  | `owned.listen` | an address, jig's format | a socket address on loopback; unique among the deployment's listeners and the machine's ports |
  | `owned.remote` | an HTTP base | `http://<listen>`: the forge's git base |
  | `owned.person` | `{"login", "email"}` | a Forgejo login other than the bot's and `temper-admin` |
  | `fake` | the playground's fake forge | its members below; the fake kind's group (7.1) and the playground's start (8.4) need it, and `application.demo` beside it |
  | `fake.listen` | an address, jig's format | as `owned.listen` |
  | `fake.remote` | an HTTP base | `http://<listen>` |
  | `fake.git` | a data path | the playground's git, a link to `temper fake git` (8.2, 8.4): the program the engine names in each of the fake forge's repositories' records, for the worker to spawn in place of `workers[].git` (channel.md, section 4) |
  | `github` | GitHub's place, `{"api", "transport", "remote", "web", "git"}`, or null | null for github.com's (17.1); a place only to point at a fake (7.5) |

- **Runner slots:** change CI and tiers whose workflows use `temper-host`
  share the owned runner's capacity, including other repositories' jobs.
  `application.machine.owned.runner.capacity` has minimum and default 1
  (8.3). To run CI beside one single-job tier takes 2 slots; beside two
  such tiers takes 3. Multi-job workflows need a slot for each concurrent
  job. With only one slot a slow tier can keep CI waiting and prevent
  landings during its run. A capacity edit reaches an existing runner
  through re-preparation (6.3); stop the box, edit `CONFIG/temper.json`,
  then restart on the same configuration, data and cache directories.
- **A forge's setup entry,** what the setup host and the box's commands
  set an owned, fake or managed forge up with, is made from its record
  and, for an owned or fake forge, the machine's part of its kind: its
  bot, the record's, `temper` for an owned or fake one; an owned or fake
  forge's directory, `forge-N` or `playground-N` (section 3); an owned forge's binaries,
  runner, port, person and origins, `owned`'s; a fake forge's port and
  git, `fake`'s, its seed's owner `playground` and its one repository
  `widget`, landing on `main` (8.4). A managed forge's is its record's
  alone: its records are made when the person connects as its admin
  (5.5), and its admin token's name is `admin`, not the owned forge's
  `temper-admin`. No entry names an org: an owned forge's org is its
  first repository's, made by `Create` (6.7).
- **A Forgejo connected** with its bot's tokens has no setup entry
  (section 4); GitHub takes none (section 17).
- **The page's actions are the engine domain's offers**
  (`../domain/forge.md`, 4.1): "Set up a Forgejo here" while the
  machine names the box's owned Forgejo and none is kept (6.5), and
  "Connect as its admin", which makes a Forgejo elsewhere from the
  address the person gives (5.5). The client cannot infer these offers
  from the configuration.
- **The owned card's configuration** gives its data directory, Forgejo's
  and its runner's versions and the person CI jobs run as; the card says
  CI is unisolated and Forgejo stops with `temper box` (6.5;
  `../client/forge.md`, 5.4). After connect it offers "New repository",
  with the set up's organization and repository fields (section
  15), including after a restart. The card
  names the `person_password` path, never its value.
- **GitHub's connected card** shows the setup URL to set on GitHub
  (17.3); before any accepted installation, the action is "Install the
  App", and after one, "Install on another account" (17.4).
- **A forge's record names no credential:** its secret records hold
  them (section 3). An owned or fake forge's place is the machine's
  part's, written out, never derived: its `api` the port's address,
  plaintext, and its `git` the `remote`, `http://<listen>`, which a fake
  forge does not serve yet (8.4).
- **A forge's number is its stable id** (forge.md, 2.1), given when it
  is made and never reused, even once the forge is removed: its secret
  records are kept under it; and records made for another kind or
  address leave a forge not connected, named, with nothing erased
  (section 3).
- **`webhooks`,** `application.webhooks`, `{"listener": address}`, on
  loopback; each Forgejo forge's secret is its `WebhookSecret` record
  (configuration.md, section 3; skein's `forge.md`, section 7).
- **The clients' listener at a fixed address,** for GitHub: GitHub's
  redirect and setup URLs name it, so its port is never 0 (17.1).
- **`temper serve` checks `application.machine`,** and reads one file
  of a forge's setup, an owned or fake forge's handover (section 3).
  Under the box, as `temper internal serve --box`, it adopts the box's
  pipe, over which it asks for a forge's group, and for its removal
  (7.1).
- **`application.demo`,** the scripted model's fake LLM, `llm` alone,
  or null for none (8.4).
- **An owned example** ships for a person, a test or QA's host to start
  from (8.1).
- **`temper init`** writes a deployment that starts (8.2), with the
  default document (8.3).

### 8.1 The owned example

The box document written by `temper init` and the owned example give
`documents.lines` eight entries for each creating executor list, alongside
`tasks.charters` eight.
This holds the reference's eligible charters 1, 2, 4, 5, 6 and 7 and its
configured procedures without shortening either offer. The deployment's
effective limits must still fit its codecs.

`docs/examples/owned/` is a configuration directory (configuration.md,
section 12). Its document, `temper.json`, is a whole one, `engine`,
`workers` and `application`, for the box's owned Forgejo with a runner,
webhooks, one worker and two LLM providers, with no
LLM account: a person adds one from the client once the deployment has
started (credentials.md, section 3). Beside it are the files it names
that hold no secret: the agent's configuration, `agent.json`, and the
charters, `charters/`, short texts a person edits. The files that hold
a secret, the worker's secret and its digest, are data paths and never
shipped: the walk `box` (`temper help box`) says how each is made in the data
directory. A person, a test or QA's host starts from a copy, edits the
agent's absolute paths to the copy's, and `temper box --config COPY`
runs it (section 7).

- **Its owned Forgejo,** the machine's `owned` (above), with a runner
  whose label is `temper-host:host`, at `127.0.0.1:3000`, and the person
  `ada`, whose org is `acme`; the document names no forge. The person
  sets it up from the client under the box (6.5): its `SetUp` makes
  forge 1, the first forge made, in
  `forge-1`, its bot `temper`, its place the machine's, `api` and `git`
  from `listen`, plaintext; no credential, since its records are made
  at `started`, from the handover (6.5).
- **Its landing rule** covers `acme/widget`'s `main`, and asks for CI,
  Actions on the owned runner, the head up to date with its base, and an
  agent's review, gate 1, at a clean head (configuration.md, section 3).
  `acme/widget` is a repository the person makes in the org on
  Forgejo's web, with Actions, then adopts (6.5).
- **Its owner may do the forge's work in the org,** and no more: the
  rules' ceiling, the project's ceiling, its role's authority and its
  projections each hold one grant on the org, `acme`, for each kind a
  change and its goal's issue use (`../domain/forge.md`, section 6):
  read, push, open a pull request, merge, comment, create an issue, edit
  a pull request, update it from its base, close or reopen it, and edit,
  close or reopen an issue; 10, as the process tier's owner holds them.
- **Its LLM providers** (configuration.md, section 2), written out
  whole: 1, OpenAI, its public Responses endpoint and gpt-6-luna with
  its window, output and prices, offering keys and ChatGPT's sign-in.
  The sign-in is a plan's (`signed_in_plan`, its manage-usage address
  `https://chatgpt.com/settings/usage`), on ChatGPT's public
  registration as jig's live sign-in uses it (jig's
  `domain/providers.md`, 2.2): the issuer `https://auth.openai.com`,
  the entry client `dynamic_agent_client`, the name hint `temper`,
  authorization at `/api/accounts/authorize`, the token at
  `/api/accounts/oauth/token`, revocation at
  `/api/accounts/oauth/revoke` and the published keys at
  `/.well-known/jwks.json`, all on `auth.openai.com` over TLS; the
  scopes `openid profile email offline_access resource.invoke
  chatgpt.tokens.use.direct`, the last of which must be granted; the
  resource `https://api.openai.com/v1`; the redirect
  `127.0.0.1:1455/auth/callback`, its port fixed, as the registration
  requires. A plan's runs use the same endpoint and model as a key's,
  with the grant as a bearer (jig's `protocol/llm.md`, 5.3). The charters
  allow the provider's models in order, the default the first whose LLM
  provider has a usable account (llm.md, section 5; jig's
  `domain/tasks.md`, section 3).
- **Its agent's configuration** (configuration.md, section 5),
  `owned/agent.json`, declares OpenAI's endpoint and model, and
  Claude Code's program, its path and its programs directory, a
  directory the person names; and takes the process tier's profile, the
  sizes its live tier runs a real model with: smith's `standard`, with
  `declared` `memory` 2,147,483,648, `tool_payload` 4,098, `read_window`
  256, `conversations` 3 (room for a run and two sub-agents, smith's
  `protocol/limits.md`, 2.2: a run's main conversation and its
  sub-agents', each a program session of its own on Claude Code, smith's
  `protocol/programs.md`, 5.10) and `calls_per_response` 1 (sub-agent
  calls run one after another, smith's `domain/session.md`, 5, and
  `protocol/limits.md`, 2.3), and `policy`
  `max_turns` 8, `check_timeout` 60,000 ms, `group_stop` 10, `inbox` 8
  and `unacknowledged` 8. Both charters require eight turns; each permits
  modifying work with a 60,000 ms reserve, so the agent's check timeout
  must fit that reserve. The engine requires 4,096-byte messages;
  smith's two-byte rendered separator makes the minimum payload 4,098
  bytes, advertising 4,096 message bytes. Its largest call is then
  4,098 bytes, the larger of that payload and its 4,096-byte outcome
  (configuration.md, section 7, `small` against the agent). smith's
  standard sizes with nothing declared refuse the example's model
  declaration at start.
  `temper init`'s stock profile instead takes `calls_per_response` 2:
  two adjacent inspect-only sub-agent calls can run side by side. Its
  demo endpoint gives a smaller turn and resume carrier than the owned
  example (configuration.md, 13.8).
- **Its `engine.limits`** are jig's small profile with the overrides
  configuration.md, section 7 asks so that the engine's documents, its
  watch room and the forge's paths fit temper's client, and those its
  owner's grants need, and no other:
  - `authority.segments`, `tasks.hold_segments` and
    `tasks.authority_segments`, 8: a name's segments, the forge's
    deepest path;
  - `documents.rows` 2, `documents.lines` 8 and `documents.text_bytes`
    128: temper's client's rows and text;
  - `views.watchers` and `views.watchers_per_party`, 16: the watch room,
    as the root holds it whatever `engine.limits` says
    (configuration.md, section 7);
  - `authority.grants` and `tasks.authority_grants`, 12: its owner
    holds a grant for each of the forge's kinds its work uses, 10
    (above), and small's 4 hold none of that work; with `notes.scopes`
    15, since jig's core admits an authority's grants only with 3 notes'
    scopes more, and refuses every override of the core's otherwise
    (`jig-domain-core::valid`);
  - `tasks.authority_bytes` 512 and `tasks.batch` 1, the process
    tier's: the bounds above widen a task's stored row past the small
    profile's store, whose bounds no override changes, so the start
    would be refused (jig's `protocol/store.md`, 5.4); these two bring
    the row back within.
- **Its worker's `limits`** are jig's small profile with the two of the
  hub's that jig's small, sized for any agent, sets past smith's agent
  (configuration.md, section 7, the worker's check against the agent),
  and no other: `hub.accounts` 4, the agent's accounts, where jig's small
  has 8; and `hub.event_bytes` 4,096, the agent's advertised message
  bytes, where jig's small has 65,536. Its `application.limits` is
  `small` with `checkout.message_bytes` 4,098 so the checkout accepts
  the agent's largest call.
- **Its turn's bounds derive from smith's largest turn,** T, the
  agent's `turn_body`, which its configuration derives (configuration.md,
  section 7, the turn's check). The model's reasoning allowance,
  `reasoning_item`, is 65,536 bytes: a plan's model's reasoning items
  stay within it in ordinary use, so a chat on a plan is not held for
  its reasoning, and a person may lower it, with these bounds
  (configuration.md, section 5). Each bound is the least the turn's
  check takes, past jig's small profile's:
  - both hubs, `engine.limits` and the worker's `limits`:
    `hub.turn_bytes` T; `hub.transcript_bytes`
    W + (B − 4) × ⌊W / 4⌋, where W is `core.resume_bytes` and B is
    `size_of::<Box<[u8]>>()`: the window and a box for each turn it can hold;
    `hub.turn_queue_bytes` 8T, the agent's eight unacknowledged turns
    (the worker's check, above);
  - `engine.limits`: `core.resume_bytes` T + 4, one largest turn with
    its length, so that a chat's window resumes whole up to that size;
  - `application.limits`: `root.journal.transcript_bytes` T + 4, the
    resume setting's bound; `root.loads.transcript_bytes` T;
    `root.loads.bytes` and `root.loads.reply_bytes` T and a row's slot;
    `root.loads.read_bytes` 2T, the turn and room as large again for
    the evidence a read carries with it (configuration.md, section 11).

  A test holds the example's and `temper init`'s figures to this
  derivation from their agent, and its writer rewrites them:

  ```sh
  SKEIN_CODEGEN_WRITE=1 cargo nextest run --locked -p temper --test suite -E 'test(startup::turn_bounds)'
  ```

  jig's small store page ceiling admits the largest shipped ordinary
  transcript's full frame (jig's `protocol/configuration.md`, 6).
  The other KV bounds and the worker wire's ceilings stay as they are:
  a turn's body is a transcript file's, never a KV value. Startup refuses
  a figure past its wire field's ceiling.
- **Its `memory_bytes`** are deployment budgets, never declarations of
  a service's worst case. `engine.memory_bytes` meters the engine's
  held bytes (`../domain/root.md`, 4.1); no start compares memory with
  a sum or names a deficit (programming-model.md, 6.3).
  - The forge's API token is within `small`'s
    `forge_component.token_bytes`, 512, past which the start is
    refused; an owned forge's longest `git` base is `http://` with a
    loopback address and a port, `http://127.255.255.254:65535`,
    28 bytes.
  - The worker's agent's `program` and the configuration's path among
    its `arguments` fit Linux's `PATH_MAX`, 4,096 bytes; Claude Code's
    path and programs directory, which the host's half holds to check
    a program's login (smith's `protocol/programs.md`, section 4),
    fit the same bound.
  - Its other addresses and secrets, logins, emails, binaries and
    charters, and LLM providers keep jig's limits.

  Channel frames and queues use max(the checked credential value bound,
  the 32-byte source name, the generated Identity bound), independently
  on each side; configured keys include jig's 4,096-byte ceiling, and
  workers reserve it without provider declarations. The program paths
  are retained once, separately from the per-slot spawn command paths.
  Startup checks the example's limits with its strings at their bounds,
  without starting a service. Artificial paths are never opened. A
  separate shell test prepares the example with ordinary paths.
  Program declarations survive startup; runnable program grants still
  wait for smith's program backend and jig's program sources.
- **Three loopback addresses,** Forgejo's (`listen`, `127.0.0.1:3000`),
  the clients' listener's (`127.0.0.1:7100`) and the webhooks'
  listener's (`127.0.0.1:7101`), are what a host changes to run several
  boxes on one machine, with each box's directories.

### 8.2 `temper init`

`temper init [DIRECTORIES]` writes a deployment that starts, from its
three directories (configuration.md, section 12) and what it finds on
the machine. `temper box` runs it first when the configuration
directory is empty (7.1).

- **Refused,** exit 1, before it writes anything, naming the file: when
  the configuration directory holds `temper.json`, or the data
  directory holds `worker.secret`, `worker.digest`, `demo/llm.key` or
  `demo/bin/git`. It never replaces a file; a person who starts again
  removes them.
- **Its directories:** the data and cache directories made and checked
  as every command makes and checks them (configuration.md, section
  12), and the configuration directory made the same way when absent;
  an existing configuration directory is taken as it is; and the data
  directory's `demo/` and `demo/bin/`, made private (0o700).
- **Its files,** each created new, in this order, the document last, so
  that a configuration directory holding one holds them all:

  | File | Directory | Mode | What it is |
  |---|---|---|---|
  | `worker.secret` | data | 0o600 | 32 bytes from the kernel's random source, as 64 lowercase hexadecimal digits and a newline |
  | `worker.digest` | data | 0o600 | the SHA-256 of the secret's digits without its newline, as 64 lowercase hexadecimal digits and a newline, as the walk `box` (`temper help box`) makes it |
  | `demo/llm.key` | data | 0o600 | the scripted model's key (8.4): 32 bytes from the kernel's random source, as 64 lowercase hexadecimal digits and a newline |
  | `demo/bin/git` | data | a link | the playground's git, `temper fake git` (8.4): a symbolic link to the binary |
  | `agent.json` | configuration | 0o644 | smith's agent configuration (configuration.md, section 5): the owned example's endpoints, models and profile (8.1), and the scripted provider's endpoint and model (8.3); Claude Code's program at the `claude` found on `PATH`, and its programs directory, `programs` in the data directory; with no `claude` on `PATH`, no program, and Anthropic's models by key alone |
  | `temper.json` | configuration | 0o644 | the default document (8.3) |

- **What it finds,** each written absolute:
  - *the binary,* the running `temper`'s own path: the worker's agent's
    `program`, with `arguments` `["internal", "agent",
    <configuration>/agent.json]` and `configuration` that same path
    (configuration.md, section 5), and the playground's git link's target;
  - *git,* the `git` found on `PATH`: the worker's `git`; with none,
    `null`, written without refusing, and init says so (git.md, section
    3). The playground's link, `<data>/demo/bin/git`, is the machine's
    `fake.git` (section 8), so that the playground's git is `temper fake
    git` (8.4) whatever git the machine has, or none;
  - *the person,* the machine's owned Forgejo's (8.3): its `login` the
    name of the effective uid's entry in `/etc/passwd` (`geteuid`, then
    that file), the login the person signs in to their Forgejo with; its
    `email` `LOGIN@localhost`. With no entry, or a name that is not a
    Forgejo login, is one of Forgejo's reserved names (such as `admin`),
    or is the bot's, `temper-admin` or `work`, the first repository's
    default org (6.7), the login is `person`, and init says so: no owned
    forge is set up until the person asks, so the person's login is
    never a reason the first run does not start;
  - *the bundle:* none to find. `clients.bundle` is always `null`, the
    binary's own (`../client/browser.md`, 2.6): a temper built with its
    browser starts its first run with it. From a binary built without
    one (`TEMPER_BUNDLE=none`), init says the browser is not served;
    the link still signs in a terminal (`temper open`), and a browser
    that opens it is answered the clients' listener's page for no
    bundle (`../client/browser.md`, 2.2), never a bare `404`;
  - *the clients' listener's port:* 8367, when a socket binds
    `127.0.0.1:8367`, else the port the kernel assigns to a bind of port
    0; the port written fixed, as GitHub requires (17.1). A later start on a port taken is
    refused, naming it and the document, as any start is;
  - *the machine's ports:* 8368, the owned Forgejo's (8.3); 8369, the
    playground's fake forge's, and 8370, the scripted model's fake LLM's
    (8.4), each by the same rule;
  - *the webhooks' listener's port:* 8371, by the same rule (8.3). The
    five sockets, the clients', these three and the webhooks', are held
    until all five ports are chosen, so that no two are the same; then
    `temper init` closes them, and `temper box`, whose first run ran
    init in its own process, keeps them, and holds each for its step
    from then on (7.1), so that no port it wrote is free between init and
    the step's bind. The port is written fixed, since the setup host gives the
    hook's address from the document at its start (5.3); a listener
    bound to port 0, its hooks given the address it reports, is later.
- **Choosing ports for another box:** keep the first running and give
  init three separate empty directories. Its occupied preferred ports
  fall back by the rule above, and both documents name the chosen ports
  consistently, including the machine's remotes and the scripted LLM
  endpoint.
  The saved addresses stay fixed; a separate init releases its
  reservations when it exits, so it does not reserve them for a later
  box. A first-run `temper box` on empty directories keeps the
  reservations through startup. The walk `box`, step 1, and the
  `commands` page's init entry explain these choices (`temper help`).
- **What it says,** on stderr: `init: wrote <path>` for each file;
  `init: this temper has no browser (built with TEMPER_BUNDLE=none):
  the browser is not served` from a binary built without its bundle;
  `init: no git on PATH: a real forge's repositories need git
  2.41 or later, set as workers[].git` when there is none; `init:
  LOGIN cannot be a Forgejo login: the owned Forgejo's person is
  person` when the login falls back (above); `init: uid UID has no login
  in /etc/passwd: the owned Forgejo's person is person` with no login;
  and last,
  `init: start it with temper box`, with the flags init was given,
  then `init: after starting, continue with ChatGPT or add an OpenAI API
  key from the accounts page`. Its
  stdout is empty; exit 0.
- **Nothing else:** no forge, no account, no credential, no repository
  and no store. The engine's first start makes its store; accounts,
  forges and repositories are the client's (credentials.md, section 3;
  `../client/forge.md`), and the playground's forge, account, project
  and repository its start's (8.4).
- **Its code:** `temper-init`, step code (section 11): given the facts
  above, git's path or none and the person's login among them, the three
  directories, the secret's 32 bytes and the scripted key's 32 bytes, it
  answers the files, each with its directory, its name, its bytes and
  whether it is private, and the person's login, fallen back to `person`
  or not; it reads, writes and binds nothing. The binary's `init` finds
  the facts, checks the refusals and writes the files, on skein-io.
- **Checked:** temper-init's step tests check that its `engine.llm` is the
  owned example's with the scripted provider (8.3), that the digest is
  the secret's, the files' names, directories and order, and the
  machine's part with the person's login found and fallen back. The binary's
  crate tests, on scratch directories, check that the document init writes
  passes configuration.md, section 8, as `temper serve` checks it, with
  the agent's configuration it writes, at the fixed ports and at kernel's
  ports, the five always distinct, with and without `claude` and git;
  a second init while the first holds its five listeners, on separate
  directories, writes disjoint ports and matching listener, remote
  and scripted LLM addresses in both documents; `clients.bundle` always
  `null`; each refusal writing nothing, the person's fallback login
  among them; the files' modes; and the link's target. A box on a fresh init,
  among those tests, reaches `temper serve`'s sign-in link with no forge
  and every group dormant: no `forge-N` directory, no group's file and
  nothing in the cache. The process
  tier starts a box on empty directories (13.4).

### 8.3 The default document

A working minimum, for the mechanics and a person's first run; good
defaults for a person's real work are open (section 16).

- **No forge** (section 8): a first run's store holds none, and the
  person makes each from the client, none needing a restart: the
  playground's fake one by trying it (8.4), an owned Forgejo by setting
  it up under the box (6.5), GitHub or a Forgejo elsewhere by connecting
  it (section 17, 5.5). A start resolves each kept forge's `api` name
  once (README.md, section 14); a name that does not resolve, GitHub's
  on a machine without DNS, refuses no start: that forge starts not
  connected, and each connect of it, restored or asked, is answered
  unreachable, naming the host, until a start that resolves it. The
  other forges start as they would. A forge so held keeps its connection
  and its setup entry, built with the address `127.0.0.1:0`, which no
  listener can hold, and temper sends neither a connect nor a setup
  request for it: the placeholder never reaches a socket.
- **Its machine part,** `application.machine` (section 8):
  - `owned`, the box's owned Forgejo: its `forgejo` and `runner.binary`
    `"fetched"` (6.6); its runner the owned example's,
    `temper-host:host`, capacity 1, and its `path`; its `listen`
    `127.0.0.1:<its port>`, 8368 (8.2), and its `remote`
    `http://127.0.0.1:<its port>`; and its person the one init finds,
    `LOGIN@localhost` (8.2). Set up, it is forge N in `forge-N`, its
    handover `forge-N/admin_token`; its access and git value are
    records, with no other credential file written or read;
  - `fake`, the playground's fake forge: its `listen`
    `127.0.0.1:<the fake forge's port>`, its `remote`, and its `git` the
    playground's link (8.2);
  - `github` null, GitHub at github.com's addresses (section 17).

- **The webhooks' listener,** `application.webhooks.listener`,
  `127.0.0.1:<its port>` (8.2). Each Forgejo's `WebhookSecret` is a
  record, drawn by the engine's reconcile. Adoption makes the
  repository's hook; the fake's start-time adoption hooks its seeded
  repositories (8.4). GitHub's hints come from its App's delivery log.
- **Its LLM providers:** the owned example's `engine.llm`, whole (8.1),
  with no account; and 3, `demo`, the scripted provider, the
  playground's (8.4): one endpoint,
  `demo`, at `127.0.0.1:<the fake LLM's port>`, plaintext, OpenAI's
  dialect on its public route; one model, `demo`, priced as the owned
  example's model is, 1 per 1,000,000 for input, cached and output (jig
  and smith refuse a price of 0); its window 177,523 and its output
  8,192, giving smith's 51,717-byte host part at the owned threshold,
  4/5, for the largest stock charter's tools and authorised brief
  (`../domain/agent.md`, 4.3 and 4.6). The owned message bound stays
  4,098 bytes, and a request at the threshold
  stays within the fake's request bound; its source a key. The agent's
  configuration declares it too (8.2).
- **Its projects:** none. Its `engine.projects` is `[]`: jig makes
  each first owner a deployment owner with no project configured
  (jig's `domain/people.md`, 3), and the engine, the playground and the
  client run a deployment with no project. Projects are made at
  runtime: the playground's start makes the playground's, "Demo"
  (8.4), and the welcome's Ready the person's first, "Default"
  (`../client/README.md`, 3.5). A deployment whose
  first start seeded a project, project 1 as the document seeded it
  before, keeps it: no start removes or renames it. One policy, `engine.policies` 1 (jig's
  `protocol/configuration.md`, section 3), which each project made
  starts from: its owner's grants, the rules' ceiling and its
  projections the owned example's, on the pattern below in place of
  the org's. The deployment's landing rule, `landing.deployment`,
  covers every repository of every forge, the pattern `{"segments":
  ["forge"], "last": "open", "value": ""}`, `enforce`d, with `ci` and
  `up_to_date`, and no gate or approval: a change lands on green CI
  with no person in between, in every project, seeded or made
  (configuration.md, section 8). Each project is made on that policy
  by jig's `MakeProject` (8.4, 6.7). The policy and the landing rule are
  the binary's stock settings, policy 1 and the deployment's rule 1, and
  the document names neither (configuration.md, 13.4); `temper init`
  writes no `engine.policies`, `engine.rules` or `application.landing`.
- **Its acts:** `engine.acts` (jig's `protocol/configuration.md`,
  section 3) grants the forge connector, 1, the playground's reset's
  acts (8.4), `{"connector": 1, "may": ["remove_project",
  "remove_account"]}`, and no other connector any. The
  document's first owner, whom the grant needs, is its `owners`' first.
  A document without the grant still serves: `temper serve` starts on
  it, and only the reset's acts are refused, *not offered*: each is
  kept, and named in serve's log with the grant to add to
  `engine.acts`, the one `temper init` writes, until a start's document
  grants it.
- **Its charters and templates,** the owned example's, but both
  charters' models end with the scripted model, `demo`: chat, 1, and coding, 2, the one the
  forge's procedures start agents' work under (kind 3's number 1,
  configuration.md, 13.2);
  the templates `chat` and `goal`. A task with no model chosen takes the
  first of its charter's models whose provider has an account serving
  its payer (jig's `jig-domain-core` `llm.rs`), so in the playground,
  whose project's one account is the scripted one (8.4), a chat and the
  coding agent of the change it delegates both run on the scripted
  model, with no pick. jig resolves that default once, at the task's
  Create, and records it: a chat started before an account of a real
  provider stays on the model it took, and a chat started after takes
  the charter's first,
  `gpt-6-luna` once an OpenAI key serves it. The composer's line says
  which, before the start, from jig's `resolved_default` preview in
  `me`, never by guessing from catalogue order (`../client/README.md`,
  5.5). The request keeps its omitted choice; the preview does not turn
  it into an explicit one. The charters, the
  templates and the forge's charter are the binary's stock settings,
  their models every model `engine.llm` configures in the document's
  order, the scripted model last as `temper init` writes it; the document names
  none of them (configuration.md, 13.4), and a person changes them from
  "How work runs" (`../client/README.md`, 5.13).
- **One worker,** in the process, its `root` `workspaces` in the data
  directory, its `git` the machine's, or null (8.2); the deployment's directory
  `deployment` in the data directory; the clients' listener
  `127.0.0.1:<port>` (8.2).
- **Its limits:** the owned example's (8.1), with `forge.repositories`
  and `forge.client.repositories` 3, counts of the deployment's
  (forge.md, section 7), and `forge.output` 82. Its `memory_bytes` are
  deployment budgets, as the owned example's are (8.1). Startup checks
  its limits with its strings at their bounds, the machine's
  `fake.git`, which the engine names to the worker, among them; the two
  literals of `temper-init`'s forgejo-elsewhere/demo document are
  checked the same way.
- **`application.demo`,** the scripted model's fake LLM (8.4).
- **Its box's steps** (7.1): `temper serve`; the fake kind's group, the
  fake LLM, a fake forge's preparation and the fake forge, dormant until
  the person tries the playground (8.4); and the owned kind's group,
  dormant until the person sets up an owned Forgejo: its fetch, its
  preparation, Forgejo's web and the runner's daemon; the engine takes
  the handover and reconciles (6.5).

### 8.4 The playground

A person's first run offers the playground (`../draft/archive/first-run.md`,
3.5, 3.6 and 5, step 3): once the person tries it, the engine makes its
records, a fake forge's, the scripted account and the playground's
project; the box runs a fake LLM and a fake Forgejo from temper's own
binary for that forge, the engine sets itself up on the fake as on an
owned one, and adopts `playground/widget` into the playground's project
itself (its start, below). Once the person sets temper up, its reset
removes them all (below). Its pages and its banner are the client's
(`../client/README.md`, 4.8).
Its bar: one suggested request landed
end to end on the scripted model, by the engine's own loop with nothing
staged. The chat delegates a change on `playground/widget`; its coding agent
edits a file; the worker commits and pushes the branch with `temper
fake git`, on the fake forge's git route; the change opens its pull
request, passes the fake's CI and lands into `main`; and the chat says
so. The other suggestions answer in text. The engine hears the fake by
its deliveries to the box's webhooks' listener (8.3), and polls beside
them as it does any Forgejo.

- **Its records:** a `Playground` row of temper's store (store.md,
  section 2), none when there is no playground, naming what its start
  makes: its fake forge's number, and, once made, its project's and its
  account's; its state *starting*, from its start's decision until its
  acts are made, *running*, or *resetting* at its reset's step,
  `project`, `forge` or `account` (below). Its fake forge is a forge's
  record (forge.md, 2.1), fake, its place the machine's `fake` (section
  8), its bot `temper`, its setup entry the playground's: its directory
  `playground-N`, its seed's owner `playground` and its one repository
  `widget` (section 8). Its account is the deployment's account of the
  scripted provider, `application.demo.llm.provider`.
- **Offered while no real forge is kept:** `StartPlayground` is
  refused *not offered* while any forge but the playground's own is
  kept, whatever its state but ended: an unfinished one is started over
  first (6.7), and a whole one is not; one ended, being removed
  (`../domain/forge.md`, 4.1), is in no list and counts for nothing, as
  the welcome counts it (`../client/README.md`, 3.5); also while the
  row reads resetting, and when the document has no `application.demo`
  or the machine no `fake`. A real account alone does not end it. It is offered again
  once its reset is done, while no real forge is kept: it has no ended
  state and no end but its reset, and no other forge's connect changes
  it.
- **Its start,** `StartPlayground` ("Try the playground",
  `../client/README.md`, 3.5 and 4.8), of the deployment's scope,
  admitted as `SetUp` is. With no row, in one decision, the forge
  connector makes the fake forge's record, numbered next (forge.md,
  2.2), and the row, *starting*, naming it; and under the box, unless
  the fake forge is connected or connecting, asks the box for its group
  (7.1), which the box link holds while another group starts and sends
  once that one answers. Without a pipe it asks nothing: the fakes are
  the runner's (7.4). Once the fake forge connects, its acts follow, in
  order, each under the sign-in of the person who sent the start, with a
  key derived from the start's, since the account's uses are its
  holder's request, a first owner's for the deployment's account, which
  no act in the deployment's name makes (jig's `domain/people.md`, 5.1
  and 5.3):
  1. the scripted account: `llm.key`'s bytes less a final newline, as
     the deployment's account of `llm.provider`, by `AddKey { provider,
     join: None, digest }`, unless the deployment holds one of that
     provider. The service reads the file at each start and carries its
     bytes as a pasted value's (`../client/README.md`, section 8); a
     start that adds no account leaves the value unused, erased before
     the answer;
  2. the playground's project: jig's `MakeProject`, named `Demo`, on
     the stock policy 1 (configuration.md, 13.4), the person its owner:
     an ordinary project, which the client shows by the name its record
     holds (`../client/README.md`, 5.6);
  3. the scripted account's uses, naming the playground's project
     alone (jig's `domain/accounts.md`, 4.2);
  4. `playground/widget` adopted into the project by `Adopt`, from its
     fake forge, owned, landing on `main`, the project's home, with CI.

  The row keeps each of the project and the account as it is made. The
  start is answered once all are done, *running*, the row then running;
  or at the first that fails, *start failed*, naming why: the group's
  `failed` with its `WHY`, the account's refusal, the project's or the
  adoption's, or the forge not connected within `setup.deadline_ms` of
  the start. A start while the playground runs is answered *running* at
  once; one while another is under way waits for it and takes its
  answer. The answer is kept in memory alone, as `SetUp`'s is
  (`../client/README.md`, section 8): the same key while it is held
  answers the same outcome, and after a restart, unknown.
- **The scripted model, the playground's alone:** the playground's
  project allows only the scripted model, and no other project allows
  it, so a person's own account added before the start never runs the
  playground's scripts, nor the scripted account a real project's work.
  The scripted provider stays in `engine.llm` (8.3), its endpoint the
  fake LLM's port, the machine's (section 15).
- **Its reset,** `EndPlayground`, the banner's "set up temper"
  (`../client/README.md`, 4.8), of the deployment's scope: in one
  decision the row reads *resetting*, a start under way is answered *not
  offered*, and the forge connector keeps the reset's acts, in order,
  each the deployment's, the row's step naming the one under way, and
  each sent under a key derived from the fake forge's number and the
  step; the project's and the account's are acts in the deployment's
  name, the forge connector's own (jig's `domain/people.md`, 5.3),
  granted by the document's `acts` (8.3), and the forge's is the
  connector's own removal, which needs no grant:
  1. `project`: the playground's project deleted, by `RemoveProject`
     (jig's `domain/people.md`, 5.2), with the grace of temper's chat
     template (template 1, `configuration.md`, section 2) as the
     settings hold it when the act is sent, or none once it is removed:
     jig's core reads no template, so temper's root reads it and fills
     the act, as it reads the goal template for a goal
     (`../domain/agent.md`, 4.3.2). It is answered removed once the
     project is closed, so the step passes then; its chats are asked to
     finish within that grace, as their person's close asks one, then
     cancelled and erased with it in the deletion's own kept steps,
     which go on beside the reset's next steps until whole. The act
     sent again at a start carries the grace the settings hold then: if
     the act was made before the restart with another, it is another
     act under the key, refused as a key conflict, which moves the row
     on as a refusal does (below), and the deletion made then goes on
     with the grace it keeps;
  2. `forge`: the fake forge removed (`../domain/forge.md`, 4.1), its
     group stopped and its directory, file and record removed (7.1);
  3. `account`: the scripted account removed, by `RemoveAccount`;

  then the row erased, none again, and the reset answered *reset*. A
  step with nothing to remove, the project or the account not made,
  passes on. The page shows "Leaving the playground…" until the reset
  answers, then opens the welcome's Forge step, `/welcome?at=forge`
  (`../client/README.md`, 3.5): the store as a person who pressed "Set
  up temper" from the welcome finds it. With no row the reset is
  refused *not offered*; one while another is under way waits for it
  and takes its answer.
- **Kept until whole:** a step's act answered with a saved answer, made
  or refused, moves the row to its next step. One answered with none
  keeps the row at its step: *not offered*, the act not granted, named
  in serve's log with the grant to add to `engine.acts` (8.3); lacking
  the role, no first owner configured, named too; or busy. At each start
  of the engine, once the core is ready (jig's restart's `Open`), a row
  reading resetting sends its step's act again, with its key: an act
  made before the restart is answered from its saved answer, and one the
  restart lost is made then, so a reset a restart cut ends whole, and
  the page shows the reset under way until the row is none.
- **A start a restart cut short,** the row *starting*, is reset at the
  next start, as `EndPlayground` resets it, with no client: its fake
  forge removed, and its project and account deleted and removed when
  made. The person then finds the welcome, the store empty.
- **A box restarted while the playground runs:** its fake forge keeps
  no state across its starts (below), so the engine's records of the
  branches it lost would outlive it. The start resets the playground,
  as the banner does, and the person meets the welcome.
- **`application.demo`,** or null for none:

  ```
  "demo": {
    "llm": {"listen": {"socket": "127.0.0.1:8370"}, "provider": 3,
            "key": "demo/llm.key",
            "asked": [{"cue": "Add CSV export to widget",
                       "turns": ["..."]}],
            "cued": [{"cue": "Export widget's table as CSV",
                      "turns": ["..."]}],
            "otherwise": ["..."]}}
  ```

  | Member | Type | Bounds |
  |---|---|---|
  | `llm.listen` | an address, jig's format | a socket address on loopback; unique among the deployment's listeners and the machine's ports |
  | `llm.provider` | u32 | the number of one of `engine.llm`'s providers (configuration.md, section 2), whose endpoints are each at `llm.listen`, plaintext, OpenAI's dialect on its public route, and whose source is a key alone |
  | `llm.key` | a data path (configuration.md, section 12) | a secret's file, 0o600 (section 9): the key is its bytes less a final newline |
  | `llm.asked` | a list of `{"cue", "turns"}` | at most 8 with `cued`'s; each `cue` non-empty text of at most 256 bytes, unique across `asked` and `cued`, the words of a request the client suggests (`../client/README.md`, 4.8 and 5.6); `turns` 1 to 8 turns (below) |
  | `llm.cued` | a list of `{"cue", "turns"}`, optional, none when absent | as `asked`'s; the scripts the fake plays and the client never suggests, an agent's, cued by its brief |
  | `llm.otherwise` | a list of turns | 1 to 8 |

  The scripts, `asked`, `cued` and `otherwise` together, fit skein's fake LLM's
  `script_bytes` (skein's `fake-llm.md`), which `temper-fake` sets.
  Any other member is refused, naming it (configuration.md, section 1).
- **A turn** is skein's `Turn` (skein's `fake-llm.md`, section 2), in
  either of two spellings:
  - *a text:* one text line, finishing `stop`, its tokens its bytes
    divided by 4, rounded up;
  - *an object,* `{"lines", "finish", "tokens"}`, the document form's and the
    playground's: `lines`
    1 to 8, each `{"text"}`, non-empty text, or `{"call", "arguments"}`,
    a tool's name of 1 to 64 ASCII letters, digits, `_` and `-` and its
    arguments a JSON object, which `temper-fake` writes compact as the
    call's bytes; `finish` one of `stop`, `tool_calls`, `length` and
    `content_filter`; `tokens` a u64, at least 1.

  Each is checked as skein checks a script's turns (its section 2.3).
  No turn holds a hold, a failing line or opaque bytes.
- **Its words,** as `temper init` writes them (8.2), the playground's,
  as the person meets it (`../draft/archive/first-run.md`, 2.3; the
  client's phrasebook, `../client/README.md`, 5.6): one `asked` script
  for each request the client suggests in the playground's project
  (`../client/mockups/first-run.html`), "Add CSV export to widget", "Add
  a dark theme to widget" and "Fix widget's empty search state"; one
  `cued` script, for the coding agent of the change the first
  delegates; and an
  `otherwise` that says it is the scripted model, the playground's,
  which answers only the
  suggested requests, and that setting temper up, from the banner,
  gives other work a model of the person's. Each turn is an object, its
  tokens its lines' bytes divided by 4, rounded up. Their content is the
  first run's follow-up
  (`../draft/archive/first-run.md`, section 8).
  - *"Add CSV export to widget",* the chat's, three turns, each
    finishing `tool_calls`:
    1. the text "I’ll delegate this change in playground/widget. It
       will push a branch, open a pull request, check CI and land into
       main on Playground forge.", then `delegate`, one item: the change procedure,
       `{"kind": "procedure", "connector": 1, "code": 2}`; its spec's words
       `Export widget's table as CSV`, and its parameters the repository,
       `{"kind": "resource", "connector": 1, "resource": 1}`, the fake's
       `playground/widget` (below), and the bytes `main`; its contract a
       change on connector 1, kind 1, in at most 128 words; its
       authority the one the real loop's landing story gives its change
       (`tests/real-loop`'s `landing.rs`), its grants on the project's
       pattern (8.3) and its delegation to the coding charter's agents,
       `{"kind": "agent", "number": 2}`;
    2. `{"call": "wait", "arguments": {}}`;
    3. once the change's result wakes the chat, the text "CSV export is
       available from widget’s table. The change landed on Playground
       forge. This was a scripted change.", then the same `wait`.

    The two texts are the mockups' first run's
    (`../client/mockups/world-first-run.js`), with their typographic
    apostrophes. The first
    goes with the `delegate` it announces, since the chat's page shows a
    message's text beside its calls (`temper-client-view`'s chat page).

    It never calls `finish`: the chat stays open, its composer with it,
    and the chat's 32-byte report bound (`engine.templates`' `chat`)
    never applies.
  - *`Export widget's table as CSV`,* the coding agent's, cued by the
    delegate's words, which are the change's title and so its agent's
    brief; the chat's request never reaches that agent's system text,
    nor do these words reach the chat's, so the two cues match apart.
    Three turns, each finishing `tool_calls`: `read` of `src/main.rs`;
    `edit` of it, `{"path", "edits": [{"old", "new"}]}`, its `main`
    printing the table as CSV when run with `--csv`, a header line
    `name,count`, then a line for each row; and `{"call": "finish",
    "arguments": {"form": "change", "fields": {"title": "Add CSV export
    to widget", "body": …}}}`. The worker commits and pushes what it
    left, and the procedure opens, checks and lands the change
    (`../domain/forge.md`); the coding charter's agent writes no git.
  - *The dark theme, the empty search, and `otherwise`,* one turn each:
    its text line, answering what it would change and why, finishing
    `stop`. The chat is ended by its requester (`../domain/agent.md`,
    4.6), so its run is smith's Open mode, which offers no `finish`: a
    turn that ends with text is its reply (smith's `domain/run.md`,
    section 6), committed with the turn, and the chat waits for the
    person's next words, its composer with it, as the CSV's does with
    `wait`.

  What the scripts cannot know, they do not say: the task's number,
  which the client's card for the change shows; and how the change
  ended, so the CSV's third turn says it landed even when the task's own
  row says otherwise. A message the person sends while the CSV's chat
  waits answers its wait and plays its third turn early, and one more
  is past its turns, answered "done", nudged once, and fails the run. A
  second CSV request, once the first landed, finds `main` past the
  edit's `old` and its change fails, until a restart seeds the fake
  again. `temper-init`'s tests check the scripts as data: each cue
  matched apart where it is looked for, a chat's in its person's
  request and an agent's in its system text, and absent from the
  others',
  within `temper-fake`'s bounds; the real loop's playground stories
  play them (13.4). The real loop's landing story already plays a cued
  chat's `delegate` to its landing on skein's peers, so the playground
  adds no story there.
- **`temper fake llm DIRS`,** a long step: skein's fake LLM peer, run
  `without_records` as a peer serving a person is (skein's
  `fake-llm.md`), its domain `Domain::played` with `asked` as its asked
  scripts, `cued` as its system-cued scripts, and `otherwise` as its
  fallback, no menu. A fresh Open chat's opening user message holds
  its person's brief, whole; its system text holds its instructions
  (smith's `domain/run.md`, 3.4). The chat's suggested requests are
  therefore `asked`; the coding agent's and reviewer's briefs stay in
  system text and are `cued`. Skein chooses a system cue first, then
  a cue in the last user message holding text, then the fallback
  (skein's `fake-llm.md`, 2.4). An asked script's turns count the
  assistant messages after that user message. Tool results, the
  waking words returned by a pending `wait` among them, keep that
  count: the CSV chat delegates, waits, then reports landing and
  waits again. A new user message holding text selects from its own
  words and starts the count again. A one-turn text reply waits for
  the person's next words; the chat stays open (above). Past a
  script's turns the reply is "done". It reads the key file before it
  binds, and refuses to start without it, naming the file; it accepts
  that one credential, and refuses every other request as the provider
  would. It serves `llm.provider`'s models, at their windows and
  outputs; no failure and no latency is configured, and its limits
  are `temper-fake`'s constants.
- **`temper forge prepare DIRS N` on a fake forge,** a once step: its
  directory made private; `admin_token` written when absent, 20 bytes
  from the kernel's random source as 40 lowercase hexadecimal digits,
  as Forgejo's tokens are. The fake reads it before it binds, so the
  handover is the preparation's, not the fake's own at its start, as
  the draft had it: no step races `temper serve` for the file.
- **`temper fake forge DIRS N`,** a long step: skein's fake forge peer,
  `without_records`, at the machine's `fake.listen`. It reads `admin_token` before it
  binds, refusing to start without it, naming the file, and leaves the
  file for `temper serve`. Its seed:
  - *temper's admin,* `temper-admin`, a site admin of the setup's
    routes, its one token the handover's, with every scope (skein's
    `fake-forge.md`, section 4);
  - *the owner,* its setup's, `playground` (section 8), a user of the
    domain, numbered apart from the setup's accounts (skein's
    `fake-forge.md`, section 5);
  - *its repositories,* its setup's, `widget` on `main`, each the owner's, in the
    protocol's `config.repositories` and the domain alike, numbered
    from 1 in the list's order at every start, so the box's
    `playground/widget` and the document form's `fake/widget` are each 1; on its
    `landing` branch, one commit of `temper-fake`'s constants, its
    files, author and time alike, so its head is the same commit at
    every start: `README.md`; `src/main.rs`, whose `main` prints
    widget's table, three rows of a name and a count, the file the
    playground's change edits; and `.forgejo/workflows/check.yml`, which
    describes the check the fake's CI stands for and is not run;
  - *its CI,* `Checks` with one context, `check`, with no latency,
    silence or rerun: on a repository with a cue, the domain's `Cue { path, green
    }` (skein's `fake-forge.md`, 2.3), a head whose file at `path` holds
    `green` passing and any other failing; on one without, every head
    passing. The box gives none; the process fixture's form gives each repository's
    (below).

  The bot is reconcile's, and its access to a repository the adoption's
  collaborator grant, which reaches the fake's domain (skein's
  `fake-forge.md`, section 4): the seed grants nothing, and the
  engine's setup host grants it, by `Adopt` (below). It serves its
  git route among its routes (skein's `fake-forge.md`, section 4), and
  no git smart HTTP; it keeps no state across its starts.
- **The fake's start-time adoption,** in `temper serve`'s setup host:
  after the records' load, the root's restore, the handover and
  reconcile (6.5), the playground's forge's at its group's `started` (7.1),
  one `Adopt { forge, repository }` for each of
  its setup's repositories not already adopted, numbered from 1 in the list's
  order as the fake numbers them. A repository restored by the root is
  already adopted: reconcile checks its access, and start-time adoption
  skips it. The plans run one after another (5.1), with the same given
  values as owned (5.3), the admin's token and webhooks' secret
  from records, and the deployment's listener when configured. Each
  makes the bot a collaborator with write and the repository's hook; the
  forge connects from records once they all end done. A stop leaves the
  forge not connected, named by its step; the engine and every other
  forge run on. The playground's start adopts the repository into its
  project once the forge is connected (above). `temper forge fetch` and `run` refuse a fake, "it is fake,
  not owned".
- **Its deliveries follow the setup's hooks** (skein's `fake-forge.md`,
  section 4): each event goes to each hook a repository's setup holds
  that takes it, signed by that hook's secret. A repository's hook is
  made at adoption when the deployment has a webhooks' listener
  (section 10), at `http://<listener>/hooks/<number>`, so the fake
  reaches serve's listener as a Forgejo does: a push, a pull request's
  change, CI's end and the merge each send one. The box's listener is
  on loopback (8.3), and its fake draws no loss and no latency; it
  records no delivery, being `without_records`. A runner's serve
  has one too, on loopback.
- **`temper fake git`,** no step: the playground's forge's git, which the
  worker spawns for the fake's repositories by the program their
  records name, the machine's `fake.git`, the playground's link (8.2; channel.md,
  section 4), in place of `workers[].git`, as it spawns real git
  (git.md, section 5), and which it checks at its first use as it checks
  real git, by `git version` (git.md, section 3). A real forge's
  repositories, on the same worker, take the machine's git. It is the
  fake git program (`../domain/testing.md`,
  4.3): the fake checkout's `.git` in the worker's working trees; its
  remote the fake forge's git route, at the host of the clone URL the
  engine gives; its credential the bot's, from git's
  `http.extraHeader`; its store the fake's domain, so a push is the
  head the engine reads.
- **Its programs' flags,** for the box and a runner alike, after the
  operands:
  - `--ready PREFIX`: once its listener is bound, the program writes
    PREFIX, then the bound address as `127.0.0.1:<port>`, then a
    newline on stdout, flushed; nothing else is on its stdout;
  - `--seed N`: its domain's seed, a u64 in decimal (skein's
    `fake-llm.md`'s `Domain::played`; skein's `fake-forge.md`'s
    `Service::new`), 0 when absent, as the box runs it; in a document's
    form, its credentials' too (below);
  - `--observations FILE`, the fake forge's, and in a document's form
    the fake LLM's, whose lines are its queries
    (`../domain/testing.md`, 4.1): FILE created new at
    0o600, its first line a checkpoint, `{"observations_lost":0}`,
    written before the ready line, and each observation its domain
    makes (skein's `fake-forge.md`, 2.4) drained after each pass, as a
    referee drains it, and written to FILE in skein's encoding, one
    JSON line each in drain order. A pass that observed or lost any
    ends its lines with a checkpoint, `{"observations_lost":N}`, the
    count lost so far, and the program writes one more at its stop, so
    FILE is whole up to its last checkpoint from its ready line on, a
    copy taken while it runs as much as the file, and a stopped
    program's last line is one. Nothing
    of a pass is sent, no reply and no delivery, before its lines'
    write completes, so a reader that has seen a reply, or what a reply
    caused, finds in FILE what the pass observed. Without the flag, the
    program drains and drops them, and sends as it serves.

  A write failure stops the program with exit 1. After the first failed
  write it writes nothing more to FILE, including at stop: it drains and
  drops observations and refuses every later pass's sends. A drain past
  its byte bound has the same rule. FILE retains its earlier writes and
  whatever bytes the failed write left, with no later checkpoint to
  certify lines it lost.

  Both peers run `without_records`, with the flags or without them,
  but the fake LLM given `--observations`, which records its queries.
- **From a document of its own,** `temper fake llm --document FILE` and
  `temper fake forge --document FILE`, the document form, which reads
  no deployment: FILE is a fake's configuration, a JSON object, as
  `temper-fake` derives it from the deployment's document in the box's
  form, with what a run needs beside it. The LLM's:

  | Member | Type | Bounds |
  |---|---|---|
  | `listen` | an address, jig's format | a socket address on loopback, port 0 for the kernel's |
  | `key` | an absolute path | the key's file, which the fake writes (below) |
| `added_keys` | a list of absolute paths, optional, none when absent | at most 15, each distinct and other than `key`'s: further keys' files, which the fake writes (below), the provider's accepted credentials 1 onward in the list's order, `key`'s being 0 |
  | `tls` | `{"root"}`, or null for plaintext | `root` an absolute path: the test root's file, which the fake writes (below) |
  | `models` | a list of `{"name", "window", "output"}` | 1 to 8, each as a provider's model's members are (configuration.md, section 2) |
  | `cued` | a list of `{"cue", "turns"}`, each with `inputs` optional | at most 8, as `llm.asked` is; each cue looked for in the system text (skein's `fake-llm.md`, 2.4) |
  | `asked`, `otherwise` | as `llm.asked` and `llm.otherwise`, each of `asked` with `inputs` optional | the same |
  | `control` | an address, jig's format, optional | a socket address on loopback, port 0 for the kernel's; required when a turn holds a piece |

  A script's `inputs` are its turn rules (skein's `fake-llm.md`, 2.4):
  a list of `{"cue", "turn"}`, 1 to 16, in the order they are tried,
  `cue` text of at most 256 bytes, empty in the last rule and only
  there, and `turn` the index of one of the script's turns; without
  them, the script counts its replies as before. A turn's lines may
  also be `{"pieces"}`, here alone and never in a deployment's
  document: 1 to 8 pieces, each `{"text", "hold"}`, `text` non-empty
  and `hold` a name of 1 to 64 bytes or null (skein's `fake-llm.md`,
  2.2). The fake releases a hold by its `control`, a listener of its
  own beside the provider's: `POST /release/NAME` answers 204 once the
  release is the domain's event, whether or not a reply is held at
  that name, and any other request 404. A document whose turns hold a
  piece and that names no `control` is refused at the start, naming
  it, since nothing could release the hold (skein's `fake-llm.md`,
  2.2).

  A turn's lines may also hold a failing line, `{"fail"}`, here alone
  and never in a deployment's document: `{"error", "credentials"}`,
  skein's `Line::Fail` (skein's `fake-llm.md`, 2.3), with its report
  the domain's default. `error` is one of `"overloaded"`,
  `"unavailable"`, `"context_too_long"`, `"unauthorized"`,
  `"refused"`, `"unsupported"` and `"invalid_request"`, or
  `{"rate_limited": {"retry_after_ms"}}` or `{"exhausted":
  {"retry_after_ms"}}`, `retry_after_ms` a u64 of milliseconds, at
  least 1, null in an `exhausted` for a limit without a reset; skein's
  error classes are translated one for one, so that a class skein adds
  is a build error in `temper-fake` the day it is added.
  `credentials` is a list of 0 to 16 distinct indices of the
  document's accepted credentials (`key`'s 0, then `added_keys`'),
  each below their count, an empty list failing every call. Where a failing
  line falls in its turn and whose calls it fails are skein's (its
  2.3): a refused document is refused at the start, naming the turn.

  The forge's:

  | Member | Type | Bounds |
  |---|---|---|
  | `listen` | an address, jig's format | a socket address on loopback, port 0 for the kernel's |
  | `admin_token` | an absolute path | the handover's file, which the fake writes (below) |
  | `owner` | a Forgejo login | other than the bot's and `temper-admin` |
  | `repositories` | a list of `{"name", "landing", "ci"}`, each with `id` and `tree` optional | as `fake.repositories` are; `ci` `{"path", "green"}`, or null for every head passing: `path` a file's path in the repository, relative, none of the seed's, and `green` non-empty text, both within the domain's content limits; `id` and `tree` as `../domain/testing.md`, 4.2, gives them |

  The box's derivation gives `cued` none, `tls` and each `ci` null.

  Both documents may also hold `pace`, here alone and never in a
  deployment's document, so that a person can follow a story at its
  speed (`../playground.md`, 5.1): the LLM's `{"reply_ms"}`, a u32 of
  milliseconds before each reply's first piece, skein's injected latency
  with its least and its most alike (skein's `fake-llm.md`); the forge's
  `{"ci_ms"}`, a u32 of milliseconds from a head's push to its CI's
  verdict, the `Checks` latency with its least and its most alike
  (skein's `fake-forge.md`, 2.3). Each at most 60000; absent, 0, as the
  box's fakes and a runner's run.
- **Its outputs,** in a document's form, where the box's fakes read the
  files `temper init` and the preparation drew: the fake writes each
  file the document names before it binds, so that each exists before
  its ready line, created new at 0o600 when absent. A retained admin token,
  model key, added key or TLS root is kept only when it is this user's
  private single-link regular file and contains exactly the expected
  seeded bytes. The check rejects symlinks, compares the opened and named
  device/inode, and bounds the read by the expected length plus one;
  any refusal names the file. A retained file is never rewritten or
  included in this invocation's rollback. Observations remain create-new
  and refuse an existing file. Each file's directory must exist, but the admin
  token's, the forge's directory (section 3), which the fake makes
  private when absent and refuses, naming it, when it is not private, as
  `temper forge prepare` does. Each key, `key`'s and `added_keys`', and the admin token are each 20
  bytes as 40 lowercase hexadecimal digits, from skein-lib's `Rng`
  seeded by `--seed` and a constant of each credential's (an added key's its index's), a stream apart
  from the domain's draws, so the runner draws none. A seeded credential is no secret
  from whoever holds the run's record: it opens only that run's loopback
  processes, which end with it, which is why the box draws its own from
  the kernel instead. `tls.root` gets
  `skein_fake_peers::transport::ROOT`'s bytes, and the fake serves the
  fixed chain that root signs, whose leaf names `skein.test` and
  `127.0.0.1` (skein's `fake-llm.md`, section 3); an endpoint reaching
  it names `skein.test` as its server and trusts that file alone
  (configuration.md, section 12), as a runner's serve and agent do.
  The fake removes none of them at its stop; the engine takes and
  removes the handover (6.5), and the run's directories go with the run.
- **A runner** starts the fakes first, learns their addresses from their
  ready lines, then writes temper's document naming them and the
  handover the fake wrote, and starts `temper serve`.
- **`temper serve`** takes the fake's handover and reconciles from
  records (6.5), runs its start-time adoption from those records, then
  connects (above): under the box, the playground's forge once its
  group has started (7.1).
- **`temper fake forge check DIRS N OBSERVATIONS STORE`,** no step: the
  check of the fake forge's promises. It is
  skein's judge (skein's `fake-forge.md`, 2.5) over forge N's
  observations, with temper's decisions from temper's own record, a
  copy of its store, never from the merges judged. It
  reads forge N's setup from its group's file (7.1), a fake forge's,
  the document for the
  store's limits; it reaches no peer and writes nothing but the copy,
  which opening may repair.
  - *OBSERVATIONS,* the file `--observations` wrote, or a copy of it:
    its lines through its last checkpoint, the flush boundary, whose
    count is the judge's loss. What follows is a flush still being
    written, and is not read. A file without a checkpoint, or a line
    before it that skein's decoder refuses at the fake's limits, is
    undecided: `observations missing`.
  - *STORE,* a copy of the deployment's store, `store/` in
    `engine.deployment.directory`, never the store itself, taken once
    serve has exited, stopped or killed: `/bin/cp -R -- <store>/.
    <empty directory>/`. A copy taken while serve runs is not one:
    skein's KV store has one process (skein's `kv.md`, section 1), and a
    copy made beside its writer can open to part of a commit with
    nothing refused. A caller takes it after the run's processes stop; a caller without
    that collection step has no copy. It is opened as serve's store
    opens it, at the limits `temper-service` derives from the document,
    and its format checked as jig checks it, temper's interval read in
    key order; one that is absent, does not open, is of another
    `STORE_VERSION`, or has a row that does not decode is undecided:
    `store unreadable`.
  - *The profile,* temper's as a client of forge N: `client` forge N's
    `Writer`, the bot's number on the fake, and with none, undecided:
    `client unknown`; `marker` temper's key's delimiters,
    `<!-- temper:key ` and ` -->` with its line feed (forge.md,
    section 4); `owned_branches` the `prefix` of each of forge N's
    `Repository` rows; `allowed` the writes temper makes on items it
    did not open: `Comment`, `Merge` and `Close`, and `Review` and
    `Request` when one of those rows' `kinds` grants `review`, since a
    task reviews an outside contribution and sets its reviewers
    (`../domain/forge.md`, section 13), while an update of one stays
    foreign, since temper updates only its own change's branch
    (section 10 there); `bounds` the decisions' count, and
    `temper-fake`'s constant for the objects, past which the judge is
    undecided: `judge full`. The constant is sized for the runs the check
    judges, not for the fake's limits: the judge reserves each repository
    slot's tables at it, so its memory grows with its square, and a run's
    history, each branch and key it saw, is bounded by no limit.
  - *The decisions,* each merge temper decided on forge N, as the
    judge's `Decision`: its repository `owner/name`, its pull and its
    head. They are read from two families (store.md, section 2):
    - each `Entry` whose write is a merge, settled or not, sent or not:
      its `Write::Merge`'s number and head, and its repository;
    - each `Landed` row, a merge temper made, joined by its task to the
      task's `Change` row: that row's repository, `pull` and
      `last_head`, since a change's head stays put from its decision to
      its landing.

    A change in `Landing`, or held over it, adds none of its own: its
    `Entry` stands for it until the merge settles, and its `Landed`
    row after, if temper made the merge. A head is the fake's commit,
    in skein's `fake-forge.md`, section 3's form: `mix(count)`
    big-endian (the SplitMix64 finalizer), the count big-endian, then
    four zero bytes, padded to 32 bytes with zeros in the store. Any
    other is undecided: `store unreadable`. So is a `Landed` row whose
    task has no `Change` row, or a `Change` row with no `pull` or
    `last_head`.
    Rows of other forges' repositories are skipped. The decisions may
    be incomplete when the `Landed` rows number the forge domain's
    `landings` limit (store.md, section 7), or when a decision's
    `Repository` row is gone. An `Order` breach is then undecided,
    `decisions incomplete`, not a fail, since a decision missing can
    only make a merge look undecided; the other breaches stand.
  - *Its order:* the decisions, then `decided`, then the observations,
    then the verdict, so no merge is judged before every decision is
    in. The collection copies OBSERVATIONS no later than STORE: a
    decision is saved before its merge is sent (`../domain/forge.md`,
    section 6), so a store copied after the observations holds the
    decision behind every merge they hold.
  - *Its verdict,* by the check's contract: a pass is status 0 and no line; breaches are status
    1, a `{"fail": LINE, "summary": TEXT}` line each, its `line()` the
    signature and its summary the promise's words, then an
    `{"undecided": REASON}` line when the evidence is also incomplete;
    undecided alone is status 2 and its line. The judge's reasons read
    `observations lost`, `judge full` and `observations unforwarded`,
    and the check's own `observations missing`, `store unreadable`,
    `client unknown` and `decisions incomplete`; with the decisions
    given first and the stream cut at a checkpoint, no merge precedes
    `decided` and no stream is truncated.
  - *Its stderr* is what it read, kept as its evidence, in JSON lines:
    `{"boundary":{"lines":N,"lost":L}}`, the checkpoint's line count
    and loss; `{"profile":PROFILE,"decisions_complete":BOOL}`, the
    profile's members by their names in skein's `Profile`; and
    `{"decision":{"repository":OWNER/NAME,"number":N,"head":H}}` for
    each decision, its head the fake's commit number. A reader's refusal
    follows as a text line.
  - *Its own errors:* a refused argument, a document that does not read
    or check, or a forge N that is no fake forge is the check's error,
    not the run's: status 1, its refusal on stderr and no line, which
    a caller reads as the check's error, never a pass. The check creates no directory.
- **Its code:** the playground's row, its start's and its reset's
  decisions, the forge connector's, in `temper-engine-domain-forge`, the
  acts they make routed by `temper-engine-domain`, and the row kept by
  `temper-protocol-store`; the fakes' in `temper-fake`, step code
  (section 11): the member's
  types and checks; the fake LLM's and the fake forge's configurations
  and seeds, from the deployment's document or from one of their own;
  the fake kind's group, its slot among the box's steps. It names no host. Its dependencies:
  `temper-forge-setup`, `skein-fake-forge-domain`,
  `skein-fake-forge-protocol`, `skein-fake-llm-domain`,
  `skein-fake-llm-protocol`, `skein-llm`, `skein-http`, `skein-forge`,
  `skein-fake-peers`, `jig-protocol-config`, `skein-lib` and
  `skein-json`; and for the check, `temper-protocol-store` and
  `temper-engine-domain-forge`, since the check's derivation is
  `temper-fake`'s too: from the store's rows to the judge's profile
  and decisions, and from its verdict to the check's lines. Its table
  fixtures also name `skein-forge-client`,
  `temper-engine-domain-forge-change` and
  `temper-engine-domain-forge-issues`. The binary's
  `crates/temper/src/fake.rs` hosts the peers on skein-shell, reads the
  check's files and opens the store's copy at the limits
  `temper-service`'s `startup::store_limits` derives from the document,
  and `temper fake git` is `temper-fake-git`'s core on the real file
  system (`../domain/testing.md`, 4.3), which `main` runs when its
  `argv[0]`'s last component is `git`.
  `Mode::Fake` and the link step's kind are `temper-forge-setup`'s.

## 9. Credentials

| Credential | Mode | Made by | Kept | Used by |
|---|---|---|---|---|
| a site admin's token | Forgejo, managed; owned; fake | the person, pasted; the preparation, by Forgejo's command line, or drawn for a fake, or the fake from its seed in a document's form (8.4) | the `AdminToken` record; an owned or fake forge's handover, until the engine takes it, which a fake forge reads at its start | the setup host alone, while a plan runs: reconcile, `TakeOver`, access at adoption (`Adopt`), and the picker's read (`Reachable`, skein's `forge-setup.md`, 4.8) |
| the bot's API and git tokens | Forgejo, every mode | the person, pasted; reconcile | the `ApiToken` and `GitToken` records | the forge component; the identities' table |
| the bot's password | Forgejo, managed, owned, fake | reconcile, random, when it makes the bot | nowhere | nothing |
| temper's admin's password | owned | Forgejo, random | nowhere | nothing |
| the person's first password | owned | Forgejo, random, at the first preparation | `person_password` | the person |
| the runner's secret | owned | the preparation | `runner/secret` | Forgejo's command line; the runner's connection, through the token's file it names |
| Forgejo's own secrets | owned | the preparation | `state`, and `app.ini` | Forgejo |
| the webhooks' secret | Forgejo, every mode | the person, pasted; reconcile, drawn | the `WebhookSecret` record | the webhooks' listener; Forgejo's hooks |
| a GitHub App's key | GitHub | GitHub, at the manifest's conversion; the person, pasted as PEM | the `AppKey` record, DER; a PEM never | the forge component, for its tokens; the setup host, while a plan runs |
| an installation's token; a GitHub git token | GitHub | the forge component, from the App's key | memory alone | the forge component; git |
| the scripted model's key | the playground | `temper init`, random (8.2), or the fake LLM from its seed in a document's form (8.4) | `demo/llm.key`, or the document's `key` | the fake LLM, its one credential; the service, which carries it at the playground's start for the deployment's scripted account (8.4) |

- **Every credential the engine holds is a record** in jig's secrets'
  store (credentials.md, the records), written by the setup host's
  plans; a pasted value is kept by jig until its run ends
  (credentials.md, pasted values).
- **Least power, kept.** The forge component holds the bot's tokens and
  the webhooks' secret, or the App's key, as in connected, and never an
  admin's token; the bot is never a site admin; the setup host holds the
  admin's token only while a plan that asked for it runs, and drops it
  at its end.
- **No secret in an argument, a URL or a log,** for Forgejo's command
  line as for git (credentials.md, section 4): 6.3.
- **No new credential to set up from the client:** `temper forge
  fetch` sends none and keeps none, its downloads trusted by their
  pinned digests alone (6.6); the box's pipe carries no secret, a
  failure's reason among its lines (7.1); and the handover is taken as
  at a start (6.5).
- **Never across a domain boundary:** the setup host is in the engine's
  protocol layer, below the core, which sees a pasted value's number and
  digest alone; the engine's rules (credentials.md, section 1) hold for
  the records and for the handover.

## 10. Webhooks on a box

The listener is skein's (`forge-connection.md`, section 6), and so are
the signature, the events and the hints they become (`forge.md`, section
7). This section says what makes them reach the engine on a box, and
that they cover what its work waits on.

- **Made at adoption, checked at each start** (section 5; skein's
  `forge-setup.md`, 4.5, and 4.3, step 6): one hook per adopted
  repository, to `http://<listener>/hooks/<number>`, with the events of
  skein's `forge.md`, section 7, and the forge's secret. An owned
  Forgejo's `app.ini` allows the listener's address (6.2). On a box,
  nothing is left to the person.
- **What a box's work waits on, and what ends each wait,** with the keep
  reading the forge within `hinted` of the hint, which gathers one
  change's hints into one read (`../domain/forge.md`, 5.2):

  | The domain waits on | Done on the forge by | Its delivery | Its hint |
  |---|---|---|---|
  | a landing branch's tip | a merge, a person's push | `push` | the branch moved |
  | a pull request's head, base or state | a push to its head or base; closing, reopening, retargeting or editing it | `push`; `pull_request` | the branch moved; the item |
  | CI on a head | each Actions job ending | the job's end | the commit |
  | a participating object's comments and reviews | a comment, a review, an approval or a rejection | the item's comment or review events | the item |
  | an issue of a live goal | editing, closing, labelling or assigning it | `issues` | the item |
  | drift of a branch temper holds | a push, a deletion, a creation | `push`; `delete`; `create` | the branch moved; the branch |

- **Echoes go up too.** Forgejo delivers temper's own writes. The client
  reads on every hint it holds a resource for, so each costs one read;
  temper, the owner, drops its own echoes by key and by pusher (skein's
  `forge-client.md`, section 4; `../domain/forge.md`, 5.2).
- **What Forgejo settles after it delivers,** a pull request's
  mergeability, which it answers not mergeable while it checks (skein's
  `forge.md`, section 9), is read again at `hinted`, doubling up to
  `poll` apart, within `settle` of the move, as is a hinted job's CI
  that reads pending.
  These are the owner's fresh reads: the client runs no window (skein's
  `forge-client.md`, sections 4 and 5). A pull request that reads not
  mergeable within `settle` of the move is never taken as conflicted
  (`../domain/forge.md`, 5.2).
- **What sends no delivery,** and so is found by polling alone: a
  commit status that a system other than Actions posts, which a
  box has none of.
- **Polling is unchanged** (skein's `forge-client.md`, section 4): the
  keep's alarms, its paces and the slow pass are what they are with no
  webhook. A delivery lost while `temper serve` is down, refused for its
  size, or never sent is found by them; Forgejo does not send a failed
  delivery again (skein's `forge.md`, section 9).
- **GitHub sends no delivery to temper:** its App's changes come from
  the App's delivery log, read by the forge component, with no address
  GitHub must reach and no webhook secret (forge.md, a GitHub App's
  delivery log; 17.6).
- **Checked:** the process tier's hinted story (13.4) checks the
  table's third row, CI on a head: the one wait of its landing that the
  forge ends. Its other waits end on temper's own writes, its push, its
  pull request and its merge, whose deliveries are echoes. Within its
  budget, only the job's end delivery brings a read of its CI:
  - the keep's `poll`, `poll_max` and `slow` are raised past the budget
    (configuration.md, section 7);
  - so is the change's `stall`, whose end reads its pull request afresh
    before it holds the change (`../domain/forge.md`, 5.3 and 8.4);
  - its job outlasts `settle` from its change's push, so the rereads
    within `settle` end before CI does (`../domain/forge.md`, 5.2).

  The first two rows, moved by someone else's push or edit, and the
  others, comments, reviews, issues and drift, are checked from hints
  in the worlds (`../domain/testing.md`, 2.2 and 6.3).

## 11. The crates

```
skein-forge                  + the setup's documents: admin users and their tokens, orgs, repositories'
│                              collaborators and hooks, GitHub's App documents; both sides, as its
│                              others; the pins, the versions and the fetched binaries' digests
│                              (skein's forge.md, sections 4 and 5)
skein-forge-setup            the setup's plans, their types and checks, the access they end with, and the
│                              host component a service runs them on (skein's forge-setup.md, section 6)
├── skein-forge
└── skein-lib, skein-json
temper-forge-setup           step code: the setup's types and checks; the machine's part, read from the
│                              configuration, and a forge's setup entry by its kind; the setup mapped
│                              onto the plans' given values, and the kit's check of them; the fetch's
│                              machine and the preparation's; app.ini and the runner's files; the
│                              processes' specs; the kinds' steps and the group's file. Names no host
├── skein-forge-setup, skein-forge
└── jig-protocol-config, skein-lib, skein-json
temper-forge-setup-shell     ordinary Rust: the hosts of fetch, prepare and run: commands, files, the
│                              forge's locks, the exec, the fetch's requests over TLS, on skein-io
├── temper-forge-setup
└── skein-io, skein-http, skein-tls, skein-lib, skein-json, skein-shell
temper-box                   the box's machine (step code), its kinds' groups, their files and records
│                              and its pipe to temper serve among it, and its host (ordinary Rust) on
│                              skein-io's processes; nothing of setup but its steps and its kinds
│                              (7.3); one crate, under the subset's
│                              lints, its host written within the subset
├── temper-forge-setup
└── skein-lib, skein-io, skein-shell
temper-init                  step code: the default document and the files temper init writes, from
│                              the facts the binary finds (8.2, 8.3); reads, writes and binds nothing
└── skein-lib, skein-json
temper-fake                  step code: the playground's member, application.demo, its types and
│                              checks; the fakes' configurations and seeds; the fake kind's group (8.4).
│                              Names no host
├── temper-forge-setup
├── skein-fake-forge-domain, skein-fake-forge-protocol, skein-fake-llm-domain, skein-fake-peers
└── jig-protocol-config, skein-lib, skein-json
temper-fake-git              ordinary Rust: the fake git program's core, temper fake git, its replay and
│                              its two remotes (../domain/testing.md, 4.3)
├── skein-fake-checkout, skein-fake-forge-domain, skein-fake-forge-protocol, skein-fake-peers
└── skein-lib, skein-io, skein-http, skein-json
temper                       the binary: temper forge fetch|prepare|run|remove, temper box, temper init,
                               temper fake llm|forge|git
```

- **The engine runs the setup host** in its protocol layer (5.1):
  temper-service maps each forge's setup onto the plans' given values
  through `temper-forge-setup` (section 5), and depends on neither
  `temper-forge-setup-shell` nor `temper-box`.
- **The binary checks the machine's part and its limits** for every
  command, `temper serve`'s included: `temper-forge-setup` checks
  `application.machine` and the `setup.` limits, and `temper-box` the `box.` limits, beside
  temper-service's startup.
- **Machines, not scripts.** The plans (skein's) and the preparation
  decide from what they read, and say each request or command they need;
  a host runs it and hands back its outcome. Step tests drive them with
  answers; the plans' world, skein's, drives them against the fake forge
  (13.2).
- **Not the throwaway's code.** `testing/skein-throwaway-forgejo` is a
  test crate (skein's `fake-forge.md`, section 6). The production
  machines are written on skein, under the subset, and the process tier
  provisions its Forgejo with them (13.4).
- **The fake forge serves the setup's documents** too
  (`skein-fake-forge-protocol`, skein's `fake-forge.md`, section 4),
  from the server's side of `skein-forge`, as it serves the engine's.
- **The fakes are product,** `crates/skein-fake-*` (skein's
  `fake-forge.md`, section 1), since `temper fake`
  ships them; the worlds and the real loop use them as before.

## 12. Where it serves

- **The first milestone** (`../README.md`, section 2), on a box run from
  a copy of the owned example (8.1), its CI Actions on the owned runner.
  the page `box` is its walk, from the binaries to a change asked
  for.
- **The process tier** (`../domain/testing.md`, 2.6), on owned mode in place of
  its own provisioning, with CI and webhooks (13.4).

## 13. Testing

The tiers, their budgets, the fakes and the layout are `../domain/testing.md`'s;
this section names what checks this document's contract, and what it
relies on.

### 13.1 Step tests

- **Forgejo's and the runner's files:** `app.ini` and the runner's
  configuration for each setup, as golden text.
- **The preparation's decisions:** a first preparation; one at the same
  versions; an upgrade, dumped before migrated; a newer recorded version
  refused; a runner added, under its lock, and one removed; another
  binary refused; a missing `admin_token` made again, under a fresh name;
  a preparation cut short made again from an empty layout; Forgejo's
  files with no `state` refused; a later preparation taking Forgejo's
  lock alone when the runner configuration is unchanged; a changed
  capacity at the same versions rewrites the runner's YAML while keeping
  its registration and secrets. The shell checks that a live runner's
  lock permits identical YAML to remain and refuses changed YAML until
  the runner stops.
- **The specs:** each command's arguments and environment, and no secret
  of the preparation's in any argument.
- **The plans' decisions** from answers are skein's (skein's
  `forge-setup.md`, section 7). What is temper's is the mapping of
  section 5, in temper-forge-setup: each member of a managed setup and
  of an owned one becomes its value, the word `temper` and the names
  `temper-api` and `temper-git` among them, and each `Place` the kit's
  check refuses is named by its path.
- **The configuration:** each bound of section 8's machine part
  refused, naming its path; a forge's setup entry made from the machine's
  part by its kind; the kinds' steps with and without each part.
- **The playground's fakes,** in temper-fake: each bound of
  `application.demo` refused, naming its path; the fakes' configurations
  and seeds from the default document; the fake kind's group.
- **The box's machine:** the order; a restart after its once steps; the
  backoff and the cap; the stop's order, and `temper serve`'s second
  signal; a link step started on its long step's first line, given it
  alone, once per box, and given a restart's first line when the long
  step exited before writing one; a stop, a give up, a group's stop and its
  failure withdrawing a spawn not yet asked for, once, a second termination
  ordering no second withdraw; a spawn asked for waited on in its place,
  its late child stopped, or closed at once after a second termination,
  never readied, continued, recorded or restarted, and its late failure
  ending it as a stop's.
- **The fetch's machine** (6.6), in temper-forge-setup, given its URLs
  and digests: each pin's URL and digest for each architecture, and
  another architecture refused; another status or a redirect refused;
  a body past `setup.fetch_bytes` refused. With its host, in
  temper-forge-setup-shell's tests against a local file server over
  TLS, its root skein's test root named by `SSL_CERT_FILE`: a wrong
  digest, its part file removed and its refusal naming the URL and both
  digests; the right one, made 0o755 and renamed; a file already there
  hashed, nothing asked when it matches, and a mismatch refused, naming
  its path, the file left; a part file left by a fetch cut short
  written again.
- **The box's asked groups and its pipe** (7.1), in temper-box: a
  dormant group running nothing at the box's start; `start FORGE`
  running the group's steps in order, a `step` line for each, then the
  record written and `started FORGE`; a step's failure answering `failed
  FORGE WHY`, the group dormant again and the others running on; a
  second `start` while one runs answered busy, a re-ask of the starting
  forge answered by its own lines, an ask while stopping, a record not
  written, a forge with no group `not asked`, and a started one
  `started`; a kind's group bound to the forge it is asked for, its
  file and its record written; at a later start, a group with its
  record run in place as an always group, one without it dormant, and a
  second started group of one kind not started, named; `remove FORGE`
  stopping the group, running its remover and removing its file and
  record, answered `removed FORGE`, on the real ring too, and a removal
  that fails answered `failed`; a line past `box.line_bytes`
  refused; `temper serve` restarted with a pipe afresh, an answer meant
  for the exited one dropped; a restart held unsent and then a stop
  launching nothing; a held spawn's late start and late failure during a
  box's stop, and a late start during a group's stop, the box ending with
  nothing open.
- **The box's terminal and logs** (7.2), in temper-box, its clock the
  test's: given one pipe for its stdout and its stderr, serve's link
  written in pieces between other steps' lines reads whole there; the
  sign-in block's parts in order around it, and a later link's shorter
  block; a step's lines in its log and not on the terminal, and on both
  with `--echo steps`; no file holding a link; the terminal's lines
  for a group's start, `started`, `failed` and `stopped`, by its label,
  a restart, a give up and a stop; a failure's tail, its lines and its
  log's path; a log moved to `.1` at `box.log_bytes`, and the last run's
  moved to `logs.previous/` at a start; a log not written named once,
  its step's lines then on the terminal.
  The link/ordering cases include separate held write acknowledgements,
  full output and policy queues with acknowledgement progress, first/fresh
  and restarted-serve blocks, UTC midnight rollover, bounded EOF fragments,
  exact/over-bound URLs with recovery, and the legacy first-line handoff.
  One default real-loop story runs the real box Service and synthetic
  children on one real ring, making split writes to a shared stdout/stderr
  pipe with interposed diagnostics. It observes complete ordered blocks,
  then asks for stop and drains all owned resources; it uses no backend,
  download or sleep. Exhaustive saturation cases stay in simulated tests.
  Pure policy tests pin group identities and structured failure reasons,
  exact retry delays, window reset and give up at decision time. Formatter
  tests use fixed wall values and supplied labels, including labels whose
  bytes cannot be inferred from arguments. Host tests pin labelled group
  start, started, failed/retry and stopped against unchanged numeric pipe
  replies, intentional-stop exits, repeated asks, failure teardown ordering,
  and no duplicate final give up. A failed requested group leaves serve
  usable. Held output acknowledgements preserve facts' source wall values
  and the existing complete sign-in blocks.
- **The setup host's handover at runtime** (5.1), in temper-service's
  step tests: at `started`, `TakeOver` when the handover is present, its
  record kept before removal, then `Reconcile` and the connect, the row
  "Connecting" between; a stop or failed removal leaving it "Set up
  failed", named by its step; `failed` naming the box's step and
  pointing to its output for the rest; with no pipe, `SetUp` of an
  owned forge refused *not under the box* (6.5).
- **The setup host's close** (5.1), in the existing temper-service
  setup step fixture: notes full and never popped, host outputs full
  before close is requested, and `Closed` behind the active notes,
  store, forge and events gates. The close request is retained, host
  `Close` is sent once when its bounded room permits, and only the
  actual host/socket terminal events and io settlement complete it.
  Events and counts establish the order, with no sleeps, new harness,
  unbounded queue or timeout change.
- **The playground's start and reset** (8.4), in the engine's domain
  world (`tests/engine/domain`) and the forge connector's tests: no
  row, none; a start making the fake forge's record and the row,
  starting, and asking for its group in one decision, the box link
  holding it while another group starts; once the forge connects, the
  account added unless the deployment holds one of the provider, the
  value left unused otherwise, the project made, the account's uses
  naming it alone, and `playground/widget` adopted; the answer,
  running, once all are done, and start failed at the first that fails,
  the forge not connected by `setup.deadline_ms` among them; a second
  start while running answered the same; the same key answered from
  memory, and unknown after a restart; a start refused *not offered*
  while a real forge is kept, or while the row resets; a start a
  restart cut short reset at the next start; the reset deleting the
  project, then removing the forge, then the account, then erasing the
  row; an act not granted kept, named with its grant, and sent again at
  the next start; a reset cut after each act ending whole at the next
  start; and in temper-service's step tests, a start carrying
  `llm.key`'s bytes.
- **The fake's start-time adoption** (8.4), in temper-service's step
  tests: after handover and reconcile, one `Adopt` for each seeded
  repository not already restored, in order, with the deployment's
  listener when configured;
  the admin token and webhooks' secret answered from records, and the
  connect's access built from records; a stop named by its step.
  `temper-init`'s step tests check the document's listener, and the
  binary's crate tests the five ports distinct.

### 13.2 Reconcile's world

The world is skein's (`forge-setup.md`, section 7): the plans on the
fake forge's protocol layer, serving the setup's routes, and the host
component's own waits and bounds. What is temper's is the engine's
routing of the setup host, checked in `temper-service`'s own tests
(`forge.md`, section 10) and by the process tier (13.4):

- a start's handover taken, its file removed only after `TakeOver`'s
  `Done`, and left when it stops; a handover readable by others
  refused, naming it;
- each write answered `Written` only once kept, so a restart between
  finds what a browser was told;
- a read served from the run's pasted value before the forge's record;
  a pasted value whose digest differs refused, "the pasted value
  changed";
- one writing plan per forge and one `Reachable` beside it, a second of
  either refused busy, and a request past `setup.runs` refused busy;
- a start's reconcile that stops leaving that forge not connected,
  named with its step and no secret, and the others served;
- records made for another kind or address leaving the forge not
  connected, named, nothing erased; an `api_token`, `git_token`, `user`
  or `webhook_secret` file in a forge's directory left unread.

### 13.3 What the setup relies on

Forgejo's and the runner's behaviours at the pinned versions (6.1), each
recorded by skein's conformance (`fake-forge.md`, section 7) on the
throwaway Forgejo and runner, and named here by its fact in the
conformance's `facts.txt`. What reconcile relies on, and the deliveries,
are skein's: `forge-setup.md`, section 8 and `forge.md`, section 9. What
the preparation relies on stays here:

- **Forgejo's API takes the person's tokens** once the person need not
  change their password (`api-token-of-a-user-whose-password-must-change-is-refused`,
  403 "You must change your password", recorded at 17.0.0; hence
  `--must-change-password=false`), and after a later change of it: a
  token is the person's until it is revoked
  (`api-token-survives-a-password-change`);
- **Forgejo's command line** (6.3):
  - `admin user create --random-password` prints `generated random
    password is '<password>'` on its first line
    (`cli-user-create-prints-the-random-password-first`), and refuses a
    user made twice (`cli-user-create-again-is-refused-naming-the-user`);
  - `admin user generate-access-token --raw --scopes` prints the token
    alone (`cli-token-raw-prints-the-token-alone`), limited to the scopes
    it names, comma-joined (`cli-token-scopes-limit-what-the-token-reads`,
    `cli-token-admin-scopes-comma-joined-are-taken-under-two-suffixes`),
    and refuses a second token of one name
    (`cli-token-name-used-twice-is-refused`);
  - `migrate` runs on a fresh work path, and again on a migrated one
    (`cli-migrate-runs-on-a-fresh-work-path`,
    `cli-migrate-runs-again-on-a-migrated-work-path`);
  - `dump` needs the work path's repositories directory and its
    `--tempdir` (`cli-dump-needs-the-repositories-directory`,
    `cli-dump-needs-its-tempdir-to-exist`), adds `.zip` to a name without
    it (`cli-dump-appends-zip-to-a-file-name-without-it`), and overwrites
    a file of its name (`cli-dump-over-an-existing-file-overwrites-it`);
  - a user made to change its password is refused the API until it has
    (`cli-user-made-to-change-its-password-is-refused-the-api-until-cleared`).
- **The runner** (6.3):
  - `forgejo-cli actions register --secret-file` takes exactly 40
    hexadecimal digits with no line feed
    (`runner-register-refuses-a-secret-of-another-length`,
    `runner-register-refuses-a-secret-that-is-not-hexadecimal`), and run
    again with the same secret keeps one global runner
    (`runner-register-again-keeps-one-global-runner`);
  - the runner's `create-runner-file` takes its secret only as an
    argument
    (`runner-create-runner-file-takes-its-secret-only-as-an-argument`);
  - a connection of the runner's `server.connections` has, as its `uuid`,
    the secret's first sixteen characters in hexadecimal, and as its
    token the secret
    (`runner-uuid-is-the-secrets-first-sixteen-characters-in-hex`,
    `runner-token-is-the-secret`), which a file may hold (`token_url:
    file:`; `runner-connection-with-a-token-file-comes-online`); a runner
    started on that connection alone takes the instance's jobs
    (`runner-connection-alone-takes-the-jobs`).

### 13.4 The process tier

The tier, its budget and its referee are `../domain/testing.md`'s (2.6). What
it checks of this document:

- **Its directories:** every command a test runs is given the three
  directories' flags, each a scratch directory of the test's own; the
  owned example is copied as a configuration directory (8.1). No test
  resolves a default under `$HOME`.
- **Owned in place of the test's support:** the test writes the
  forge's group's file as serve does under the box (7.1), runs `temper
  forge prepare`, `run forgejo` and `run runner` itself, then `temper serve`,
  whose setup host takes the handover and reconciles (6.5), with the
  binaries `TEMPER_FORGEJO` and `TEMPER_FORGEJO_RUNNER` name, and
  webhooks on. Its repository runs Actions, so its landing rule asks for
  CI.
- **The hinted story:** the same story with the keep's `poll`,
  `poll_max` and `slow` and the change's `stall` raised past the story's
  budget (configuration.md, section 7; `../domain/forge.md`, 8.4), and its
  CI's job outlasting `settle` from the change's push, so that it lands
  within its budget only if the job's end is delivered and the reads
  that follow it (section 10, "Checked"); and with no conflict resolved,
  since its changes never conflict. With no webhooks, the same story
  misses its budget.
- **`temper box`,** a test of its own on a scratch directory, with no
  `--echo`, its terminal the box's lines alone and each step's lines in its
  log; a story that reads a step's lines runs it with `--echo steps`: the sign-in
  link on its stdout, and whole on one pipe given as both its stdout and
  its stderr, as a terminal is (7.2); Forgejo killed and restarted, with the engine still
  serving; a stop leaving no process it started.
- **A first run, into the playground and out,** a test of its own:
  `temper box` on three empty scratch directories writes the
  configuration (8.2), says so (init's line for no browser among its
  lines only from a binary built without one: the test checks the build
  it runs in, `../client/browser.md`, 2.6), and prints one sign-in link,
  which signs the owner in, with no forge kept and every group dormant.
  The owner starts the playground with `temper ctl playground start`
  (`../client/tools.md`, 5.5), answered running once its fake forge is
  connected and `playground/widget` adopted into Demo, the project its
  start made (8.4); then resets it with `temper ctl playground end`,
  answered reset. The store then reads empty, as a first run's: no
  forge listed, no account of the deployment's, no project, and no
  `playground-N`, `box/groups/N` or `box/started/N`
  left; and a native client reloaded opens the welcome at its Forge
  step (`../client/README.md`, 3.5).
- **A reset the box's death cut,** a test of its own: a first run's box,
  the playground started, then reset while the box is held before its
  `removed`, after the project's deletion is committed, and the box
  killed with the reset unanswered. The next box on the same
  directories, with no client asking, ends the reset whole at its start
  (8.4): the store reads empty as above.
- **Two first runs, one after the other,** a test of its own: a first
  run's box, signed in, then a second box on three other empty scratch
  directories while the first runs. The second's init finds the first's
  ports held and writes the ports the kernel assigns (8.2), and the
  second signs in, starts its playground and answers a suggested request
  on its own fakes while the first, which starts none, holds the
  defaults. No first-run story takes a lock
  on the default ports: each box holds its own (7.1), so the stories
  run beside each other as any others do.
- **A set up cut short, started over,** a test of its own: under the
  box, with Forgejo's binaries placed in the cache as below, the
  owner's set up of an owned Forgejo is killed with the box at each of
  its steps, its fetch, its preparation, its Forgejo's run, and after
  its `started` record with the engine held before it hears it. Each
  next box finds the forge kept and its first repository not made, and
  its removal, `temper ctl forge remove N`, answers removed and leaves
  no `forge-N`, `box/groups/N` or `box/started/N`. A set up after the
  four makes forge 5, since numbers are never reused, connects it and
  makes its first repository; the cache's binaries are the ones placed,
  unchanged (6.7). Then a removal the box's death cut, the box held
  after the removal was answered and killed: the next box ends it whole
  at its start, no forge listed and nothing of it left.
- **GitHub gone on after a restart,** a test of its own, on `temper box
  --fake-backends`' fake GitHub (7.5): GitHub's set up stopped after
  its App's records are kept, the box restarted, and the forge found
  unfinished, then gone on, to the App's installation, with no Start
  over, and connected (6.7).
- **A first run set up for real,** a test of its own: `temper box` on
  three empty scratch directories, with the binaries `TEMPER_FORGEJO`
  and `TEMPER_FORGEJO_RUNNER` name placed where the fetch looks,
  `<cache>/forgejo/VERSION/forgejo` and
  `<cache>/forgejo-runner/RUNNER_VERSION/forgejo-runner`, so that the
  fetch hashes them against the pins and fetches nothing (6.6): no
  mirror, no TLS and no call off the machine. The owner asks "Set up a
  Forgejo here" through the client, `temper ctl DIRECTORY forge
  setup`, answered setting up; the new forge's row, forge 1, the first
  made, followed by `forge list --until settled`, goes through "Setting
  up" and "Connecting" to "Connected as temper", with no restart, and
  its first repository made, `work/hello`; the call adopts it into the
  new project `hello`, project 1, the first made (6.7). The
  workflow's push has run on the owned runner: before the owner's first
  task, the runner's output, which the box, run with `--echo steps`,
  gives on its stderr as `runner-1: ` lines, says it took a task on `work/hello`, main's
  push's, since no pull exists yet. It shows the run taken, not its
  verdict: no read of temper's says CI at a repository's `main`, and
  the story reads no credential. The owner's first task
  in the project, on the fake LLM, changes `hello`: its worker clones
  it, its pull's run of the starter workflow passes, and it lands; the
  test reads it all through temper's own reads, with no credential:
  the set up's answer names `work/hello` adopted, owned, on forge 1;
  the task, given in project 1, runs on and lands into `work/hello`;
  the change's checks at its pull's head are the starter workflow's
  run, passed; and the task ends landed. The
  handover file is gone, and
  `forge-1` has no `api_token`, `git_token`, `user` or
  `webhook_secret` file. The service's step world shows the git value
  and its generation (forge.md, section 10, the box's link). A second
  box on the same directories starts the group by its record, before
  `temper serve`. Preparation makes a new handover; serve takes it over
  and connects forge 1 from its records (6.5), and the task is found
  landed, its CI at its head unknown until a next step reads it
  (`../domain/forge.md`, 8.8).
- **The first-run behaviours** past these use their lower owners: the
  stock scripts' data tests, the real-loop playground stories, and the
  client and engine setup/reset worlds. The process tier has one short
  fake wiring test; it does not replay the playground before the owned
  repository story (`../domain/testing.md`, 2.6).

## 15. Later

Each is intended; what it needs is named with it.

- **After the first repository** (6.7): "New repository" on a connected owned or managed card is the general
  form of the same flow, with the same fields and `Create`, a managed
  forge's org needing an owner, and like it adopting nothing. The
  first repository is adopted by the person from the adoption form, or
  by ctl's call in its own run, while the playground's start makes its
  acts itself (8.4). Projects are
  made at runtime by jig's `MakeProject` (jig's `domain/core.md`,
  6.1), none seeded (8.3): the welcome's Ready makes the first run's,
  "Default" (`../client/README.md`, 3.5), and ctl's `forge setup`
  makes its `--project`, named after the repository unless the call
  names one, so that on a box set up by ctl alone `work/hello` is
  project 1's, the first made (`../client/tools.md`, 5.5).
- **An owned forge made managed** (section 2): an act that makes an
  owned forge's record managed, after which temper stops running
  Forgejo, its directory's `forgejo/` is a stock work path a person
  runs, and its `AdminToken` record, taken over (6.5), stays valid.
- **The LLM providers out of the document,** each a design of its own;
  the scripted provider, the playground's (8.4), may become the binary's
  stock first, its endpoint the fake LLM's port, the machine's.
- **A lock and an exec in skein's shell,** onto which 6.4's module
  moves once skein's shell offers an exec and a lock held across it,
  which skein's `Lock` is not (skein's `io.md`, section 5). A second
  service that execs is what would ask skein for them.
- **A story of Reconcile over TLS.** The managed string form takes a
  forge on another host over TLS and the service hands that transport to
  skein (5.6), but every test of the admin connect, the process story
  among them, reaches its Forgejo over loopback plaintext: a TLS story
  waits for a fake forge that serves TLS (skein's `forge-setup.md`,
  section 9).
- **Webhooks from another host,** over TLS (skein's
  `forge-connection.md`, section 6), once skein has TLS's server side.
- **CI's isolation.** An owned runner's jobs in host mode run as the
  box's user, who can read every file of temper's as credentials.md,
  section 8 accepts of agents in standalone, the handover among them
  until the engine takes it. Until they are isolated, an owned forge
  runs the person's and temper's agents' code only, never a stranger's
  pull request.
- **Provisioning at adoption** (jig's `domain/connectors.md`, 12): a
  repository in a person's own namespace, made without the person.
- **The forge as people's sign-in provider** (credentials.md, section
  8), which would link the person's account on an owned Forgejo to their
  owner in jig.

## 16. Open questions

- **Good defaults for a person's real work:** charters, landing rules,
  limits, the memory bound. Default: 8.3's working minimum, refined once
  a first run works.
- **Overrides of `app.ini`.** Default: none; the settings are temper's.
- **A token for the person's scripts and agents.** Settled: the
  preparation makes two, the person's read and write header files
  (6.3, 6.5), which the box names (7.2); the person may make others on
  Forgejo's web, with the scopes they choose; never the preparation's
  `admin_token`, which the engine's setup host alone takes (section
  9).
- **Backups beyond an upgrade's.** Default: the person's, of the forge's
  directory and the store, with the box stopped.
- **Two deployments on one managed forge.** Default: each its own bot
  login, carried by its connect (5.5), and so its own collaborators and
  hooks; the marker does not tell deployments apart.
- **A hook left at an old listener's address,** after the listener
  moves, is skein's question (skein's `forge-setup.md`, section 9).
  Default: left alone; the person deletes it.
- **A managed forge whose host list refuses the listener.** Default:
  nothing tells the person but the hints that never arrive
  (`../domain/forge.md`, section 4).
- **The managed picker's departure.** On a managed or owned Forgejo the
  picker lists every repository the site admin can see, not those the
  bot can write, as a connected forge's does, since adoption grants the
  bot its access (5.4). Default: the admin's view, through `Reachable`,
  plus "not listed" by owner and name; the admin's token stays the
  setup host's, and the picker gets the rows alone.
- **A start's reconcile that stops.** Default: that forge is not
  connected, named with its step, until the next start, a managed
  connect or an owned forge's next set up ask (6.5).
- **A managed or owned forge stopped at runtime,** its bot's token
  revoked. Default: stopped, named, until the next start's reconcile or
  a managed connect; Replace is a connected forge's alone.
- **The owned example's agent, sized for a person's work.** Default: the
  process tier's profile (8.1), sized for the live tier's stories.
- **The check's decisions** (8.4) are read from rows kept for other
  ends, and two decisions leave none: a change whose pull is replaced
  while `Landing` joins its `Landed` row to the later pull, and an
  enqueue the forge's merge queue drops leaves no `Entry` once settled,
  a GitHub path the Forgejo fake never takes. Default: as 8.4 reads
  them; a family of temper's decisions, `(repository, pull, head)`
  written in the merge's `Entry`'s commit, is the route when a run
  needs either, a store schema change in place until the first release
  (store.md, section 4; skein's `codec.md`, section 5).
- **Demo after a reset** (8.4; `../draft/first-run-projects.md`, 9):
  a second playground run makes "Demo" again, the first having been
  deleted by the reset, and reuses nothing but the name. The user's to
  confirm. Default: as 8.4's start makes it, the reading the draft's
  status records while the user is away.

## 17. Connecting GitHub

GitHub is managed or connected, never owned. Its managed mode means an App
made through the manifest, not Forgejo's string-managed admin connect (5.5).
Both GitHub modes end in the same
records (section 3), and the engine runs both alike after; they differ
in who made the App. The plans are skein's (`forge-setup.md`, section
5), run by the setup host (5.1); the browser's routes are the clients'
listener's (`../client/forge.md`, the browser's steps).

### 17.1 GitHub's App and its permissions

- **One App per deployment,** public, so that it installs on any
  account (skein's `forge-setup.md`, section 5). Its permissions:
  contents, pull requests, issues and merge queues, write; checks,
  statuses, actions, administration and metadata, read; never
  workflows, never administration's write. Its events are skein's
  (5.6). Beside them (`../domain/landing.md`, 8): statuses and actions,
  write,
  so that a project landing by turn has its gates dispatched and its
  receipts and verdicts posted; installations made before keep reading
  only until their owner accepts the new permissions, and adoption says
  so. The `Always` bypass the turn needs is a repository's ruleset's,
  granted by its admin, never asked by the App. `workflows` stays
  unasked: a change under `.github/workflows/` (a gate's or a tier's
  definition, or a member rebased over one) is pushed by a person, and
  whether GitHub refuses a ref update or a merge that brings one
  without it is to check.
- **Its hook** points at `https://<slug>.invalid/`, an address that
  never resolves, so GitHub delivers nowhere and holds no secret of
  temper's; the App's changes come from its delivery log (17.6).
- **Its records:** `AppId`, `AppSlug`, `AppKey` (DER; a PEM is never
  kept), `User` (the App's bot user), `Accepted` and `Origin`
  (credentials.md, the records).
- **Its addresses,** its record's place, are the machine's GitHub
  place's, github.com's when the machine's `github` is null (section 8);
  the machine names a place only to point at a fake (7.5).
- **The clients' listener at a fixed address,** a loopback port the
  person's browser reaches, since GitHub's redirect and setup URLs name
  it; a GitHub `Connect` on its port 0 is refused, naming
  `application.clients.listener` (forge.md, 2.1; configuration.md,
  section 3).
- **A repository's own rules** are read and said at adoption, and never
  changed (`../domain/forge.md`, what adoption says of the branch's
  rules): a merge queue is taken as it is; required approvals need the
  App on the rule's bypass list or a person's approval; required
  signatures, linear history and code owners are each named with the
  one setting that lets temper land.

### 17.2 Managed: the manifest flow

1. **Connect,** "Connect GitHub" on the deployment's page or its tool
   of `temper ctl` (`Connect` with `GitHubManifest { back? }`). The
   setup host runs `Manifest { forge, target }` (skein's
   `forge-setup.md`, 5.1), `target` always `Personal`, for a fresh
   state, ending `Done(Manifest { state, target, manifest })`; temper
   renders the page from them (client/forge.md, 9.1) and holds it and
   `back` with the state. With `back`, the link is a path on the
   page's own origin, `/forges/<n>/github/manifest?state=<s>`: the
   browser goes to it in the page's own tab, as a sign-in's does,
   and comes back to `back` at the end. Without `back`, the link is
   the whole address on the clients' listener,
   `http://<listener>/forges/<n>/github/manifest?state=<s>`; the
   terminal prints it and never opens it.
2. **The manifest's page** posts the manifest to
   `<web>/settings/apps/new?state=<s>` as it loads, by one inline
   script its policy allows by hash, and shows its button where the
   script doesn't run or the page was reached through the browser's
   history; its headers allow a form to `<web>` alone, keep no copy
   and send no referrer.
3. **GitHub's form:** the person creates the App, under their own
   account. Nothing asks for an organization: the App is public, so
   GitHub's install page offers every account the person may install
   it on (with the user, 2026-10-23; an App an organization owns is
   for later, `../draft/github.md`, section 18).
4. **The redirect back,** `http://<listener>/forges/<n>/github/redirect`:
   `Convert { forge, code, state }` (5.2) exchanges the code for the
   App's id, slug and key, finds its bot user, and writes the records;
   the conversion's PEM, webhook secret and OAuth pair are never kept.
   Once they are kept, `Install` (5.4) answers the App's install link,
   the browser goes on to it, and the forge connects (forge.md, the
   setup's host).
5. **Installing:** the person picks an account, and all repositories,
   GitHub's default, or those they select. GitHub sends the browser to
   `http://<listener>/forges/<n>/github/setup`, where `Accept` (5.4)
   checks the installation and adds it to `Accepted`, and the route
   sends the browser back to `back`, or, with none, says "temper has
   the installation; you may close this tab." A member of an
   organization who isn't its owner can only request the App there:
   GitHub comes back with `setup_action=request` and no installation,
   and the route says why temper can't go on, and to install it on
   their own account or an organization they own (client/forge.md,
   9.1).

- **States** are one use, drawn by the setup host, at most
  `setup.states`, each living `setup.state_life_ms`, at most the
  conversion code's hour; a used, unknown or expired one is refused,
  "This link has ended. Start again from temper.", and a restart
  forgets them all.
- **The redirect and setup URLs** temper gives the plans are whole,
  built from the clients' listener's address (forge.md, the setup's
  host). The manifest's `url`, the App's homepage, is
  `http://<listener>/`.

### 17.3 Connected: an App pasted

- **The person makes an App** on GitHub with 17.1's permissions and
  events, and pastes its id and its private key as GitHub gives it,
  PEM: "Connect an App of your own" (`Connect` with `GitHubApp { id, read_only?, key
  }`). No webhook secret is asked: the paste check sets the App's hook
  unless `--read-only` leaves it as read (17.7).
- **Its setup URL is the person's to give** on GitHub's form, since no
  plan can set it: `http://<listener>/forges/<n>/github/setup`, with
  redirect on update, no callback URL and no user's authorization at
  install (the walk `github`). An installation of an App without
  it never comes back through the setup route, and is a stray (17.4).
- **The paste check,** `Paste { forge, app }` (skein's
  `forge-setup.md`, 5.8): the key, kept by jig until its run ends,
  decoded to DER; the App read with it, its id the one pasted, else
  `OtherApp`; its bot user found; its hook set to
  `https://<slug>.invalid/` unless read-only (17.7); its permissions and events checked as its
  reconcile checks them. It writes `AppId`, `AppSlug`, `AppKey` and
  `User`, the records the manifest flow writes, and ends done, or
  `Lacking` with each permission or event named, never refused for it.
  A forge whose `app_read_only` is true has its hook read and left
  (17.7).
- **A restart during the paste check** drops its run: the carried key
  is erased at start, and the forge reads as its records say
  (`credentials.md`, 2.2; a stored connect, which would say
  "interrupted; paste again", is an open question there).
- **Installing** is 17.4's.
- **Replace,** a new key: the paste check with the kept id, so a key of
  another App stops it `OtherApp` and the forge stays as it was.

### 17.4 Another account, and repositories later

- **"Install on another account"** (`Install { forge, back? }`,
  skein's `forge-setup.md`, 5.4): a fresh state and GitHub's install
  link (`Link`), where the browser goes as at 17.2; the person installs
  on another account or org, and the setup route accepts it and sends
  the browser back.
- **Repositories added later** to an installation of selected
  repositories are the person's, on the installation's page on GitHub;
  GitHub's visit to the setup route with no state refreshes an accepted
  installation's repositories and sends the browser to Forges, and is
  refused otherwise. The forge
  component reads an installation's repositories again on its events.
- **An installation temper did not accept,** a stray made with no
  state of temper's, is shown as "not accepted · kept", regardless of
  age. A person's App's installations are theirs: temper uninstalls
  nothing it did not make. One deleted on GitHub leaves `Accepted`;
  one suspended is shown.
- **Accepting a stray,** `Accept { forge, installation }` (the tools'
  `forge accept`, `../client/tools.md`, 5.5): an installation that
  exists already, made on GitHub before temper knew it, is taken by the
  deployment's owner's own act, with no browser. The setup host runs
  `Accept` with no state (skein's `forge-setup.md`, 5.4): the person's
  sign-in stands for the state, which binds only a browser's return to
  the install its person started. The installation is read on GitHub
  as the App's, a `404` refusing it, and added to `Accepted` as the
  setup route's accept adds it. Its refusals, before any run, are
  `Install`'s; the setup route's own accept still requires a state.

### 17.5 Its reconcile

- **`Reconcile { forge }`** (skein's `forge-setup.md`, 5.6) runs after
  each `Connected`: the App's key, permissions, events and installations
  checked. It repairs only its hook's address, and with `app_read_only`
  (17.7) leaves the hook as read. A stray is always kept, never uninstalled.
- **A key GitHub refuses** stops the forge: the forge component, which
  signs with the same key, stops it (forge.md, 2.3), named, with
  Replace (17.3); the reconcile's own `Stop` is named to the root and
  changes nothing.
- **A permission or event lacking** is named, `Lacking`, never refused:
  the person grants it in the App's settings and approves it on each
  installation, as GitHub asks.

### 17.6 Job logs, and the delivery log

- **A job's log** lies behind a redirect to a storage host, which the
  forge component follows only to an origin its forge's log origins
  list (forge.md, GitHub's API as temper uses it): on GitHub one origin
  by default, its kind's. An origin not listed is named on the
  deployment's page.
- **The delivery log** is read by the forge component every
  `log_poll`, and its deliveries become hints as Forgejo's webhooks do
  (forge.md, a GitHub App's delivery log).

### 17.7 A pasted App another owns: read-only

- **`forge connect [FORGE] github-app APP FILE --read-only`** sets the
  kept forge's record's `read_only` (forge.md, 2.1), called
  `app_read_only` below. The wire's `GitHubApp.read_only` is optional, absent meaning
  false; it travels through the engine's `How::GitHubApp` to that record.
  It says the
  pasted App (17.3), its hook and its installations are another's to
  change: a probe App shared with skein's conformance, whose hook stays
  where its owner put it, as the live tier's (`../domain/testing.md`,
  2.7). The setup host gives skein `repair_hook: !read_only` and
  `uninstall_strays: false` (skein's `forge-setup.md`, 5.10). Without the
  flag, ctl says in one line that temper points the App's webhook at
  `https://SLUG.invalid/` and to connect with `--read-only` to leave it.
- **What changes:** the paste check (17.3) and reconcile (17.5) read
  the App's hook and leave it; reconcile lists the installations and
  keeps `Accepted` as it says, and a stray (17.4) is shown as kept,
  never uninstalled, until the person accepts it.
- **What doesn't:** the App's key, id, permissions and events are read
  and checked, a lacking one named; `Install`'s link, the setup route's
  `Accept` and the accept of a stray (17.4), which read; and the forge
  component's connection, its mints and its delivery log (17.6).
