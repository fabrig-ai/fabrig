# The forge's setup

Provisional, 2026-10-16, revised 2026-11-02. How a forge comes to be
ready for temper, in three modes: connected, where a person runs and sets
up the forge and gives temper only what it strictly needs; managed,
where a person runs it and temper sets itself up on it, with a Forgejo
site admin's token or, on GitHub, with an App made from temper's
manifest; and owned, where temper runs Forgejo and its runner, and sets
itself up on them. A fourth, fake, is the playground's: temper runs its
own fake Forgejo, seeded with the playground's repository, and sets
itself up on it as on an owned one (`playground.md`). A forge is a record of temper's
store, made by the act that sets it up or connects it (../forge/engine.md, 1 and
2); a person chooses a forge's mode and makes it from temper's client
(`../../client/forge/README.md`, the deployment's page), and may remove it
(`../../domain/forge/adoption-connection.md`, 1). The
engine is the same in every mode: each mode makes the records the one
below reads, kept in jig's secrets' store (../credentials.md, the records),
and the engine's forge component reads connected's. Owned mode is
commands a box arranges, and `temper box` is wiring only. A box is what temper's first milestone is set up on and
checked by CI on (`../../README.md`, section 2). The members are ../configuration/README.md's (`../configuration/engine.md`),
the credentials ../credentials.md's (`../credentials.md`, section 2), the webhooks ../forge/README.md's
(`../forge/README.md`, section 4), and the commands and crates ../README.md's (`../binaries.md` and `../components.md`).
The walks are the pages `box`, `forgejo` and `github` (`temper help
NAME`). What comes later is `README.md`, section 9; what is open,
`README.md`, section 10; connecting GitHub, `github.md`.

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
  (`playground.md`).
- **Forges are records in the store** (../forge/engine.md, 1): each made by the
  act that sets it up or connects it, `SetUp` for an owned Forgejo,
  `Connect` for GitHub or a Forgejo elsewhere, `StartPlayground` for the
  playground's fake one, and numbered next when it is made, a number
  never reused (../forge/engine.md, 2). The document names no forge: it keeps
  the machine's, its listeners, what a box's owned Forgejo and fakes
  take from the machine, and the limits (`init.md`). A forge can be
  removed whole (`../../domain/forge/adoption-connection.md`, 1); its box group is then
  stopped and its directory removed, the cache kept (`box.md`, 1).
- **Each mode is a layer that makes the inputs of the one below.**
  Owned's preparation makes a Forgejo and an admin's token, managed's
  input, and hands the token over in a file; managed's plans make the
  bot, its tokens, its id and the webhooks' secret, connected's inputs.
  The forge component always runs connected: it reads the same records,
  checks the same user, and never holds an admin's token (section 3).
- **The records are the interface.** Each forge's records, under its
  number, a stable id never reused, are kept in jig's
  secrets' store by the engine's protocol layer alone (../credentials.md,
  the records). The one file that passes between the layers is an owned
  forge's handover, `admin_token`, which the engine takes once.
- **The core is step code; the hosts are thin.** The setup's plans are
  skein's `skein-forge-setup` (skein's `forge/setup/forgejo.md`, section 1, and `skein/docs/design/forge/setup/github.md`, section 1
  ), run inside the engine by skein's setup host component (its host,
  `owned.md`, 3): per forge, one writing plan at once and the picker's read beside
  it (`reconcile.md`, 1). The preparation, Forgejo's settings and the processes' specs
  are machines and pure functions of `temper-forge-setup`, hosted by
  three commands, `temper-box forge fetch`, `temper-box forge prepare` and
  `temper-box forge run` (`owned.md`). No command writes the records.
- **`temper box` is wiring only.** It runs a configuration's steps in
  order (the fakes, fetch, prepare, Forgejo, its runner, `temper-box
  serve`), restarts what exits, and stops them in reverse order. It
  knows kinds, not forges: an owned Forgejo's group and the playground's
  fakes' wait, dormant, until `temper-box serve` asks over a pipe for one of
  them for a forge, and run again at each later start once started
  (`box.md`, 1). It names no Forgejo request and nothing of setup but its
  steps. systemd or a test arrange the commands another way, with no
  owned forge (`box.md`).
- **`temper-box init` writes a deployment that starts** into the three
  directories (../configuration/directories.md): the default document, its
  agent's configuration and the worker's secret; no forge, no account
  and no credential, which are the client's. With `--box` it adds the
  box's parts: the welcome's member, the owned Forgejo and the
  playground. `temper box` runs `temper-box init --box` first on an empty
  configuration directory (`init.md`, 2, 3).
- **A first run is empty** (`init.md`, 3): the store holds no forge, and, where
  the document shows the welcome (../configuration/engine.md), the
  person tries the playground or sets temper up. The playground is made
  when chosen (`StartPlayground`): a fake forge's record, its box group,
  the scripted account and the playground's project, into which
  `playground/widget` is adopted; its reset (`EndPlayground`) deletes
  the project and removes the forge and the account, and the person is
  on the welcome's Forge step as if they had never tried it. It is
  offered while no real forge is kept, and never once one is (`playground.md`).
  Real forges are made from the client: an owned Forgejo set up under
  the box, its group asked when the person sets it up (`owned.md`, 5, `box.md`, 1), and
  GitHub or a Forgejo elsewhere connected (`github.md`, `reconcile.md`, section 5); none needs
  a restart.
- **A set up writes as it goes, and starts over by removal** (`owned-repositories.md`, 1): the
  forge's record and its first repository's intent are kept as it goes;
  an unfinished forge found later is removed and set up again, Forgejo's
  binaries kept in the cache.
- **Reconcile converges at every start** of a managed or owned Forgejo,
  in the engine: the bot user, made when missing and otherwise checked,
  never edited; its two tokens; and each
  adopted repository's access, the bot a collaborator with write and, on
  a forge served the listener (`reconcile.md`, 3), the repository's hook; each made
  when missing, repaired when changed, and left alone otherwise (`reconcile.md`
  ).
- **Access at adoption.** Adopting a repository of a managed or owned
  Forgejo first gives the bot its access (`Adopt`, `reconcile.md`, 4); nothing is
  granted org-wide. The picker offers every repository the site admin
  can see, read by the `Reachable` plan, which writes nothing (`reconcile.md`, 2).
- **GitHub** (`github.md`): managed by the manifest flow, through the
  clients' listener's routes, or connected by an App the person pasted;
  both end in the same records. Its reconcile, after each connect,
  repairs only the App's hook's address; its changes reach the engine
  through the App's delivery log, with no webhook secret.
- **Owned is a stock Forgejo.** temper writes its `app.ini`, runs its
  migrations, makes its own admin and the person's account with
  Forgejo's command line, registers the runner offline, dumps before an
  upgrade's migration, and runs only the versions it pins, the
  conformance's, which it fetches itself when asked, checked against
  the digests it pins (`owned.md`). Its directory, once temper stops running
  it, is a Forgejo a person can run themselves, as managed.
- **A box is reactive.** Every change on the forge that a box's work
  waits on reaches the engine as a signed webhook delivery, and becomes a
  hint, read within the keep's `hinted`: pushes, pull requests, issues,
  comments, reviews and each CI job's end. Adoption and reconcile make
  the hooks; an owned Forgejo is allowed to deliver to the engine's
  listener, and a managed one's settings are the person's (`reconcile.md`, 6). Polling
  is unchanged, the fallback for a delivery lost or refused (`webhooks.md`, section 1
  ).
- **Loopback.** An owned Forgejo listens on loopback, as temper's
  clients' and webhooks' listeners do; a person on another machine
  reaches them through a tunnel. A forge on another host polls: its
  webhooks need skein's TLS server side (section 9).
- **Not here:** webhooks from another host, the isolation of CI's jobs,
  provisioning at runtime and the forge as people's sign-in provider
  (section 9).

Documents:

- `box.md`: the box's steps and arrangements.
- `box-supervision.md`: step supervision, restarts, stops and output.
- `box-backends.md`: the retained sandbox and fake backends.
- `box-front.md`: the box beneath its terminal front.
- `owned.md`: preparing, running and fetching an owned forge.
- `owned-repositories.md`: the first repository and later makes or copies.
- `owned-sign-in.md`: signing in through temper to an owned forge.
- `webhooks.md`: webhook delivery on a box.
- `crate-layout.md`: the setup crates and their dependencies.

- `github.md`: connecting GitHub.
- `init.md`: the configuration and init.
- `place.md`: a forge's place.
- `playground.md`: the playground's setup.
- `reconcile.md`: managed reconcile.
- `testing.md`: testing the setup.

## 2. The modes

| | connected | managed | owned | fake |
|---|---|---|---|---|
| Runs the forge and CI's runner | the person | the person | temper | temper: its fake forge, `temper-box fake forge`, whose CI answers each push itself; no runner |
| Makes temper's bot, its tokens, its access and the webhooks | the person | temper: on Forgejo with a site admin's token, the webhooks on this host alone (`reconcile.md`, 5); on GitHub, an App from temper's manifest | temper, with an admin it makes | temper, with the admin the fake is seeded with |
| temper is given, on Forgejo | the bot's two tokens, pasted; optionally a webhooks' secret | a site admin's token, pasted | the Forgejo and runner binaries, or `"fetched"` for temper's (`owned.md`, 6); an address on loopback; the person's login and email | an address on loopback; the seed's owner and repositories |
| temper is given, on GitHub | an App's id and its private key, pasted | the person's steps in their browser, from a link temper prints | none: GitHub is never owned | none: the fake is a Forgejo |

- **Fake is owned's layers,** with temper's fake forge in place of
  Forgejo and its runner (`playground.md`). What this document and the others say
  of an owned forge's handover, directory, reconcile, records, adoption
  and picker holds of a fake one; what they say of Forgejo's or the
  runner's binaries, `app.ini`, the preparation's command line and the
  person does not. A fake forge
  keeps nothing across its starts but its directory's handover: each
  start serves the seed afresh, and reconcile makes the bot again
  (`playground.md`).
- **Per forge.** A deployment may mix them: an owned Forgejo beside a
  connected one, or beside GitHub. The playground's fake forge is made
  only while no real forge is kept (`playground.md`). Each owned or fake forge has a directory and an address
  of its own, and a box runs at most one of each kind (`box.md`, 1).
- **A mode is its forge's record's** (../forge/engine.md, 1), set by the act
  that made the forge: owned by `SetUp` (`owned.md`, 5), fake by
  `StartPlayground` (`playground.md`), managed by `Connect` with a Forgejo site
  admin's token (`reconcile.md`, 5); a Forgejo connected with its bot's tokens is
  connected (`README.md`, section 4), and so is a managed one whose `AdminToken`
  record is forgotten (below). It is no credential record
  (../credentials.md, the records). On
  GitHub, managed and connected end in the same records, and differ only
  in how the App came to be (`github.md`).
- **A mode changes:**
  - owned to managed is later (section 9);
  - managed to connected, from the client, "Forget the admin's token":
    the `AdminToken` record is erased, no reconcile runs for the forge,
    and its bot, tokens, id and hooks stay as reconcile last kept them;
    adopting a repository then grants nothing, and the person makes the
    bot a collaborator (section 4);
  - connected to managed, from the client, "Connect as its admin": a
    site admin's token is pasted, and reconcile runs with it, taking
    over the connected user as its bot when its full name carries
    `temper:bot` (`reconcile.md`, 5).

## 3. The layers and their files

The three modes and their layering (each mode makes the inputs of the
one below, and the service always runs connected) are held by skein's
`forge/setup/README.md`, section 3; fake is owned's layer with another forge in
it (section 2). temper's layers, and what passes between them:

```
owned      temper-box forge fetch        ──► the Forgejo and runner binaries, in the cache (6.6)
           temper-box forge prepare      ──► the forge's work path, the runner's, person_password,
                                         admin_token (the handover)
           temper-box forge run          ──► forgejo web (its child on Linux), forgejo-runner daemon (exec'd)
fake       temper-box forge prepare      ──► the forge's directory, admin_token (the handover)
           temper-box fake forge         ──► the fake, seeded with temper-admin and that token (8.4)
managed    the engine's setup host   ──► the forge's records (below); takes the handover once
connected  the forge component       ◄── its access, built from the records
```

- **One directory per owned or fake forge,** named by its kind and
  number beneath the data directory, `forge-N` for an owned forge and
  `playground-N` for the playground's fake one, and removed with the
  forge (`box.md`, 1): private, `0o700`, the user's own and not reached through a link,
  as a worker's root is (../configuration/worker.md, section 1). `temper-box forge
  prepare` makes it so when it is absent; a directory that is not so
  refuses the command, naming it. A managed or connected forge has none.
  A fake forge's holds `admin_token` until the engine takes it; `temper-box
  fake forge` reads it before binding (`playground.md`). An owned forge's files:

  | File | Made by | Read by | Holds |
  |---|---|---|---|
  | `forgejo/` | prepare | Forgejo, through `run` | Forgejo's work path: `app.ini`, its database, repositories and logs |
  | `runner/` | prepare | the runner, through `run` | the runner's configuration, its registration, its jobs' directories, `secret` |
  | `backups/` | prepare | the person | one dump before each upgrade's migration |
  | `state` | prepare | prepare, `run` | owned's record: the versions prepared, whether the first preparation ended, Forgejo's own secrets |
  | `admin_token` | prepare, when missing | the engine's setup host, once, at start or the box's `started`, and once more on a restart of a box that signs its person in (`owned.md`, 5) | the handover: temper's admin's token, kept as the forge's `AdminToken` record by `TakeOver`, after whose `Done` the file is removed (`owned.md`, 5) |
  | `person_password` | prepare | the person | the person's first password on the owned Forgejo |
  | `person_read.header`, `person_write.header` | prepare | the person, or their agent (`../../client/ctl/testing.md`, 5) | the person's two tokens as one header line each, `Authorization: token TOKEN`: one that reads (`read:repository`, `read:issue`, `read:organization`, `read:user`), one that writes (`write:repository`, `write:issue`, `read:organization`, `read:user`) |

- **What the preparation leaves for the engine** is `admin_token`
  alone: the token's text, nothing else, private and written whole;
  made at the first preparation and at each later one that finds it
  missing, which it is once the engine has taken it, and left as it is
  when present (`owned.md`, 3).
- **Fetched binaries are the cache's,** not the forge's directory's:
  `<cache>/forgejo/VERSION/forgejo` and
  `<cache>/forgejo-runner/RUNNER_VERSION/forgejo-runner`, shared by
  every owned forge whose binaries are `"fetched"`, written by `temper-box
  forge fetch` alone and fetched again at will (`owned.md`, 6; ../configuration/directories.md
  ).
- **No other credential file is written or read for owned or fake.**
  Their `api_token`, `git_token`, `user` and `webhook_secret` files,
  including any left by an earlier temper, stay unread. Each record is
  written by a plan, `User` as its 8 bytes within a version-1 record
  (../credentials.md, the records).
- **Written whole.** Each file is written beside its name and renamed
  over it, private, so a reader never sees half a token.
- **The handover is read** as ../configuration/README.md, section 5 reads a
  secret's file, within `records.value_bytes`: it is the one credential
  file of an owned or fake forge that `temper-box serve` reads (`owned.md`, 5).
- **The records, by mode** (../credentials.md, the records):

  | Forge | Records |
  |---|---|
  | Forgejo, connected | `ApiToken`, `GitToken`, `User`, `Origin`; `WebhookSecret` when the person gave one |
  | Forgejo, managed, owned or fake | those, and `AdminToken`; `WebhookSecret` drawn by the plan that makes its first hook, on a forge served the listener (`reconcile.md`, 3, 4) |
  | GitHub, managed or connected | `AppId`, `AppSlug`, `AppKey`, `User` (the App's bot user), `Accepted`, `Origin`; `AppMade` when a manifest made the App (`github.md`, 2); no `WebhookSecret` |

- **Written by the setup host's plans alone,** each write kept before
  the plan goes on (`reconcile.md`, 1). A value a person pastes is kept by jig until
  its run ends (../credentials.md, pasted values), and is no record.
- **Records made for another forge,** an `Origin` naming another kind
  or address, and unreadable ones, leave that forge not connected,
  named on the deployment's page; nothing is erased, and connecting
  again overwrites them (../credentials.md, the records).
- **Nothing passes between the layers** in an argument or the
  environment but the directories' flags and the forge's number; the
  commands read the forge's setup from its group's file (`box.md`, 1), and with
  none as `box.md`, 4 says.

## 4. Connected

- **What temper strictly needs** of a connected Forgejo: the bot's API
  token and its git token, and, for webhooks, their secret; the bot's
  id is read with the tokens (../credentials-git-identities.md, section 1, and ../credentials.md, section 5). The
  scopes (skein's `forge/setup/forgejo.md`, section 2): `read:user`,
  `write:issue` and `write:repository` for the API token;
  `write:repository` and `read:user` for the git token, the second so
  that the setup's plans can ask whose it is (skein's `forge/setup/forgejo.md`,
  1.3, step 3, and 4.6).
- **What the person does:** a Forgejo user for temper, not a site admin;
  its two tokens; its write access to each repository temper will adopt,
  as a collaborator or through a team; optionally, each repository's or
  org's webhook to temper's listener, `http://<listener>/hooks/<number>`,
  with the secret and the events of skein's `forge/README.md`, section 6, and
  Forgejo's `[webhook] ALLOWED_HOST_LIST` letting it reach the listener;
  CI, as each repository has it.
- **Pasted from the client, together** (`Connect` with `ForgejoTokens`;
  `../../client/forge/README.md`, the deployment's page; `../../client/ctl/areas.md`
  ): both tokens, and the secret when given, one value, kept by jig
  until its run ends (../credentials.md, pasted values). Its connect makes
  the forge's record, connected, from the address the person gives, its
  API's host, port and transport, its git and web bases, and its git
  program when given, numbered next (../forge/engine.md, 2), in the connect's
  admission, before any run; or, on a connected Forgejo kept and not
  connected or stopped, connects it again. Any other forge refuses it,
  *not taken*. The setup host's `Check { forge }` (skein's
  `forge/setup/forgejo.md`, 1.6) finds that both answer as one user, not a site
  admin, and writes that user; the routing keeps the tokens and the
  secret as the forge's records, then its `Origin`, and the forge
  connector keeps the user's login as the record's bot (../forge/engine.md, 2);
  the page then says "Connected as LOGIN", with its id on an Account
  line (`owned.md`, 5). Tokens of two users stop it `OtherUser`, a site admin's
  `BotIsAdmin`, a refused token `Refused`, each named, and the forge
  stays as it was. This form makes no bot, organization, repository or
  hook: the person's user is temper's, as it is.
- **Connected again,** on a kept forge whose `User` is kept, by the same
  user alone: `Check` compares the tokens' user with it, and tokens of
  another user stop it `OtherUser`, nothing kept, the forge as it was.
  A forge's writer is its user, so another user's tokens connect a new
  forge, once this one is removed (../credentials.md, 2.1).
- **Replaced** from the client (`Replace`): a new API token, git token
  or both, and the secret when it changes, one value: `Check` reads each
  pasted token before the forge's record, so a token not pasted is the
  kept one, and finds that both answer as the kept user; each pasted
  item is then kept over the old, and the forge connects with them. A
  stop keeps nothing, and the forge stays as it was.
- **Nothing more is required** (`../../domain/forge/adoption.md`): polling
  stands in for webhooks, and a project's checks for CI.
- **A connected GitHub** is an App the person made, pasted (`github.md`, 3).

## 5. Credentials

| Credential | Mode | Made by | Kept | Used by |
|---|---|---|---|---|
| a site admin's token | Forgejo, managed; owned; fake | the person, pasted; the preparation, by Forgejo's command line, or drawn for a fake, or the fake from its seed in a document's form (`playground.md`) | the `AdminToken` record; an owned or fake forge's handover, until the engine takes it, which a fake forge reads at its start | the setup host alone, while a plan runs: reconcile, `TakeOver`, access at adoption (`Adopt`), and the picker's read (`Reachable`, skein's `forge/setup/forgejo.md`, 1.8) |
| the bot's API and git tokens | Forgejo, every mode | the person, pasted; reconcile | the `ApiToken` and `GitToken` records | the forge component; the identities' table |
| the bot's password | Forgejo, managed, owned, fake | reconcile, random, when it makes the bot | nowhere | nothing |
| temper's admin's password | owned | Forgejo, random | nowhere | nothing |
| the person's first password | owned | Forgejo, random, at the first preparation | `person_password` | the person |
| the runner's secret | owned | the preparation | `runner/secret` | Forgejo's command line; the runner's connection, through the token's file it names |
| Forgejo's own secrets | owned | the preparation | `state`, and `app.ini` | Forgejo |
| the webhooks' secret | Forgejo, every mode | the person, pasted; a setup plan, drawn (`reconcile.md`, 4) | the `WebhookSecret` record | the webhooks' listener; Forgejo's hooks |
| a GitHub App's key | GitHub | GitHub, at the manifest's conversion; the person, pasted as PEM | the `AppKey` record, DER; a PEM never | the forge component, for its tokens; the setup host, while a plan runs |
| an installation's token; a GitHub git token | GitHub | the forge component, from the App's key | memory alone | the forge component; git |
| the scripted model's key | the playground | `temper-box init`, random (`init.md`, 2), or the fake LLM from its seed in a document's form (`playground.md`) | `demo/llm.key`, or the document's `key` | the fake LLM, its one credential; the service, which carries it at the playground's start for the deployment's scripted account (`playground.md`) |

- **Every credential the engine holds is a record** in jig's secrets'
  store (../credentials.md, the records), written by the setup host's
  plans; a pasted value is kept by jig until its run ends
  (../credentials.md, pasted values).
- **Least power, kept.** The forge component holds the bot's tokens and
  the webhooks' secret, or the App's key, as in connected, and never an
  admin's token; the bot is never a site admin; the setup host holds the
  admin's token only while a plan that asked for it runs, and drops it
  at its end.
- **No secret in an argument, a URL or a log,** for Forgejo's command
  line as for git (../credentials-git-identities.md, section 1): 6.3.
- **No new credential to set up from the client:** `temper-box forge
  fetch` sends none and keeps none, its downloads trusted by their
  pinned digests alone (`owned.md`, 6); the box's pipe carries no secret, a
  failure's reason among its lines (`box.md`, 1); and the handover is taken as
  at a start (`owned.md`, 5).
- **Never across a domain boundary:** the setup host is in the engine's
  protocol layer, below the core, which sees a pasted value's number and
  digest alone; the engine's rules (../credentials.md, section 1) hold for
  the records and for the handover.

## 6. Webhooks on a box

Moved to [`webhooks.md`](webhooks.md), section 1.

## 7. The crates

Moved to [`crate-layout.md`](crate-layout.md), section 1.

## 8. Where it serves

- **The first milestone** (`../../README.md`, section 2), on a box run from
  a copy of the owned example (`init.md`, 1), its CI Actions on the owned runner.
  the page `box` is its walk, from the binaries to a change asked
  for.
- **The process tier** (`../../testing/process-tier.md`), on owned mode in place of
  its own provisioning, with CI and webhooks (`testing.md`, 4).

## 9. Later

Each is intended; what it needs is named with it.

- **The reference's combined project settings** (`init.md`, 3;
  `../configuration/reference.md`): its after-landing walk applies
  `docs/examples/reference/project.json` after making the project,
  setting both its ways and its main-only rule 1. The stage-one
  description must ask for that rule as well as the deployment's two;
  without it the project's stock still asks CI and review on goal
  branches (`../configuration/reference.status.md`).

- **The TUI's fresh-box terminal check** (`box-supervision.md`, 1, `init.md`, 3): after landing,
  check local sign-in with nothing pasted, then the pasted-link and
  browser-approval fallbacks, refusal and asking again
  (`../../client/README.md`, section 5). The recorded approval journey
  (`../../client/tui/journeys.md`, section 9) does not establish a real
  terminal's result. A public-name deployment and its Google sign-in
  remain `../../draft/tui.md`, section 7's later pass.

- **Links for several members or a pasted App** (`reconcile.md`, 3, `github.md`, 2): several
  members need a family of links and a connecting owner's evidence; a
  pasted App needs user authorization or an owner's verified claim.
  Default: a box's sole-owner link (`../../domain/forge/adoption-person.md`, 1), and
  adoption's seeds in those other deployments
  (`../../draft/forge-identity.md`, section 12).

- Repeated admission during fake-box preparation: `box.md`, section 7.

- **The running steps' lifetime proof after landing** (`testing.md`, 4): check
  hangup, retained rebinding and Linux's killed-box descendants through
  the process tier, within its standing budget. The separate killed-test
  story checks another ownership link; it does not replace killing a
  box whose playground steps run (`../../testing/process-tier.md` and `../../testing/README.md`, 7).

- **The owned-forge Claude Code lifetime proof** (`box-supervision.md`, 1): after landing,
  run the live landing story with its no-survivor assertion and the box's
  stop bounds unchanged. For any survivor, use its rows and stop snapshot
  to identify the spawning owner, then repair that lifetime at its lowest
  place under a failing test. A coding run's own child belongs to the
  run's sandbox lifetime.

- Four agents on a laptop: `init.md`, section 4.
- The fake box's retained ports under concurrent stories: `box.md`, section 7.

- Discovering models per account: `init.md`, section 4.

- A handover made by a Forgejo's own restart: `owned.md`, section 10.
- A box killed outright on macOS: `box.md`, section 7.

- A box started in the background of a terminal: `box.md`, section 7.

- The terminal after a front killed outright: `box.md`, section 7.

- **A `temper box` started in the background of a terminal** (`box-front.md`,
  1): `temper box &` from a shell with job control has a terminal on its
  standard input and output, so it runs its TUI, which job control stops
  at its first read. `temper-box` runs the box alone. Telling a
  background start from a foreground one needs `temper` to read its
  terminal's foreground group, which skein's `runtime/shell.md`, 6.3 does
  not give yet.

- **The terminal after `temper box` killed outright** (`box-front.md`, 1): its
  TUI's modes and screen stay on the terminal, as any program's killed
  outright do (skein's `runtime/shell.md`, 6.3); the box stops at its
  life pipe's end.

- A start holding the engine's documents to temper's clients: `init.md`, section 4.
- The deployment's own UX: `init.md`, section 4.
- Flags of their own: `init.md`, section 4.
- **After the first repository** (`owned-repositories.md`, 1): "New repository" on a connected owned or managed card is the general
  form of the same flow, with the same fields and `Create`, a managed
  forge's org needing an owner, and like it adopting nothing. The
  first repository is adopted by the welcome's Ready, by the person
  from the adoption form, or by ctl's call in its own run, while the
  playground's start makes its acts itself (`playground.md`). Projects are
  made at runtime by jig's `MakeProject` (jig's `domain/model.md`,
  6.1), none seeded (`init.md`, 3): the welcome's Ready makes the first run's,
  "Hello" for `work/hello` (`../../client/pages/README.md`), and ctl's `forge setup`
  makes its `--project`, named after the repository unless the call
  names one, so that on a box set up by ctl alone `work/hello` is
  project 1's, the first made (`../../client/ctl/areas.md`).
- An owned forge made managed: `owned.md`, section 10.
- The LLM providers out of the document,: `init.md`, section 4.
- A lock and an exec in skein's shell,: `owned.md`, section 10.
- A story of Reconcile over TLS: `reconcile.md`, section 7.
- A secret drawn at a reconcile with no repository,: `reconcile.md`, section 7.
- **CI's isolation.** An owned runner's jobs in host mode run as the
  box's user, who can read every file of temper's as ../credentials.md,
  section 8 accepts of agents in standalone, the handover among them
  until the engine takes it. Until they are isolated, an owned forge
  runs the person's and temper's agents' code only, never a stranger's
  pull request. Such a job, or an agent's run, can read
  `person_write.header` and post a review as the person, so a rule that
  nothing lands without the person's approval stops an agent's mistakes
  there, not one trying to get past it (`../../draft/forge-identity.md`,
  section 6, accepted by the user for now on 2026-11-07).
- **Provisioning at adoption** (jig's `domain/connectors.md`, 12): a
  repository in a person's own namespace, made without the person.
- **The forge as people's sign-in provider** for several members
  (../credentials.md, section 8). A box's sole-owner link names its person
  on an owned Forgejo (`../../domain/forge/adoption-person.md`, 1); several members need
  separate sign-in and account evidence.
- A launchd agent for the box: `box.md`, section 7.
- **The box's link on a terminal, as a story** (`box-supervision.md`, 1; `testing.md`, 4): the box's process
  stories start it on pipes, and the request `box-small-2` of 2026-11-06 leaves
  a real terminal to the user's try. skein's end-to-end kit starts a binary
  under a pseudo-terminal (skein's `examples.md`, section 6), so a story started
  so would read the link's OSC 8 from a real box, at the process tier's budget.
- A copy of a bottle of temper's own: `owned.md`, section 10.
- **One percentage for both binaries** (`owned.md`, 6): from sizes pinned beside
  their digests (skein's `forge/dialects.md`, 1.3), known before the first byte,
  so that the bar does not start again for the runner
  (`../../client/forge/deployment.md`).
- **The other steps' progress** (`box.md`, 1): preparing and starting take
  seconds, and keep the moving bar.
- A fake fetch for the fake box,: `box.md`, section 7.
- **Claude Code's download walked** (`owned.md`, 6): every test's download is a
  fake one, so the pinned URL, digests and sizes first meet
  `downloads.claude.ai`, and the build first runs as temper's launcher
  runs it, when someone chooses Claude on a machine with no `claude`.
  Whoever walks this next does that once.
- The person's `claude` installed after init: `owned.md`, section 10.
- The other fake peers' refused connections: `box.md`, section 7.
- A Forgejo's own links at its `web`: `../../client/forge/served.md`, section 3.
- An adopted row's kinds after the forge changes: `reconcile.md`, section 7.
- **Agents in contained trees** (`box-supervision.md`, 1; ../service-process.md, section 1, and ../README.md, section 11): a
  serve the box closes, or one a second signal aborts, kills its agents'
  groups, and a program an agent started, in a group of its own,
  outlives it. Ending both needs skein's contained trees (skein's
  `runtime/io/processes.md`, 3 and 4), or a parent-death signal skein-io's spawn can
  ask for. Default: serve's first signal stops its agents, which end
  their programs (`box-supervision.md`, 1).
- Reading a runner's row: `owned.md`, section 10.

## 10. Open questions

- Good defaults for a person's real work: `init.md`, section 4.
- Overrides of `app.ini`: `owned.md`, section 10.
- **A token for the person's scripts and agents.** Settled: the
  preparation makes two, the person's read and write header files
  (`owned.md`, 3, 5), which the box names (`box-supervision.md`, 1); the person may make others on
  Forgejo's web, with the scopes they choose; never the preparation's
  `admin_token`, which the engine's setup host alone takes (`README.md`, section 5
  ).
- Backups beyond an upgrade's: `owned.md`, section 10.
- Two deployments on one managed forge: `reconcile.md`, section 7.
- A task's text past what the clients receive: `init.md`, section 4.
- **The managed picker's departure.** On a managed or owned Forgejo the
  picker lists every repository the site admin can see, not those the
  bot can write, as a connected forge's does, since adoption grants the
  bot its access (`reconcile.md`, 4). Default: the admin's view, through `Reachable`,
  plus "not listed" by owner and name; the admin's token stays the
  setup host's, and the picker gets the rows alone.
- **A start's reconcile that stops.** Default: that forge is not
  connected, named with its step, until the next start, a managed
  connect or an owned forge's next set up ask (`owned.md`, 5).
- **A managed or owned forge stopped at runtime,** its bot's token
  revoked. Default: stopped, named, until the next start's reconcile or
  a managed connect; Replace is a connected forge's alone.
- **The check's decisions** (`playground.md`) are read from rows kept for other
  ends, and two decisions leave none: a change whose pull is replaced
  while `Landing` joins its `Landed` row to the later pull, and an
  enqueue the forge's merge queue drops leaves no `Entry` once settled,
  a GitHub path the Forgejo fake never takes. Default: as `playground.md` reads
  them; a family of temper's decisions, `(repository, pull, head)`
  written in the merge's `Entry`'s commit, is the route when a run
  needs either, a store schema change in place until the first release
  (../store-versions.md, section 1; skein's `data/codec/README.md`, section 5).
- **Demo after a reset** (`playground.md`; `../../draft/first-run-projects.md`, 9):
  a second playground run makes "Demo" again, the first having been
  deleted by the reset, and reuses nothing but the name. The user's to
  confirm. Default: as `playground.md`'s start makes it, the reading the draft's
  status records while the user is away.
- **A progress page while the box starts** (`box-supervision.md`, 1;
  `../../draft/box-link-first.md`, 6): a page the box would serve until it
  hands the port to serve. A start takes a few seconds, and a browser's
  own loading covers them. The user's, who chose the block over it for
  now. Default: none.
- The time of day on the terminal's error lines: `box.md`, section 7.
- **Forgejo serving a source it listed when its discovery failed**
  (`box-supervision.md`, 1; `../openid-provider.md`, 9): the source step relies on Forgejo loading its
  sources again at a sign-in naming one it holds no provider for, which
  Forgejo's sign-in does and which no walk has measured on Forgejo 17;
  the process tier's story pins it after landing. Default: the step's
  request; if Forgejo does not serve the source after it, the step keeps
  failing, saying so, and Forgejo's next start serves it.
- **A sign-in source for `temper-box serve`'s owned Forgejo** (`owned-sign-in.md`, 1): its
  command is printed, not run, so nothing retries it; a source whose
  discovery failed at Forgejo's start serves at a person's first sign-in,
  as above. Default: as `owned-sign-in.md`, 1 says.
