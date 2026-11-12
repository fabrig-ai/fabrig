# Managed: reconcile

Reconcile is skein's plan, run by the engine's setup host at each start
of a managed, owned or fake Forgejo, at a managed connect and at an
owned forge's set up from the client (1). Nothing but the setup host
runs it or writes what it makes.

## 1. The setup host in the engine

- **skein's host component** (skein's `forge/setup/README.md`, its host, `owned.md`, 3),
  a child of the engine's protocol layer beside the forge component; the
  routing carries what passes between it, the root and the records
  (../forge/README.md, the setup's host). It holds an admin's token and a GitHub
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
  most; a restart forgets them (`github.md`, 2).
- **At each start,** once the records are loaded and the root restored:
  an owned or fake forge's handover by `TakeOver` whenever its file is
  present, even with a kept `AdminToken`, its file removed, then
  `Reconcile` with the restored repositories and connect (`owned.md`, 5); each
  managed Forgejo's reconcile; the fake's start-time adoption (`playground.md`); and
  GitHub's `Reconcile` after each GitHub forge connects (`github.md`, 5). The
  deployment's page shows a forge "Connecting" until its `Connected`.
- **At the box's `started`,** the setup host runs the same handover,
  removal, reconcile and connect sequence for the forge (`owned.md`, 5). A stop or
  failed removal leaves it set up failed, named by the step, and offered
  again; every other forge runs on.
- **A run's end:** `Done`, or `Lacking` (done, with what the App lacks
  named), keeps its writes, then connects the forge after a plan that
  connects; `Stop` names its failure and its step, with no secret, and
  leaves the forge as it was. A start's reconcile that stops leaves that
  forge not connected, named, until the next start, a managed connect or
  an owned forge's next set up ask (`owned.md`, 5); the engine and every other
  forge run on. A managed forge's row names it too: its plan's stop
  shows as `SetUpFailed`, with the plan, the step and the why, and Connect
  stays offered, so a repository whose access a start could not repair
  is said where the person looks.

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
  remain those of skein's host (skein's `forge/setup/README.md`, 4.3) and
  skein/docs/foundation/programming-model.md, 5.2: closed follows settlement, never a request.

## 2. The setup host's plans

| Plan | Forge | When | skein's `forge/setup/README.md` |
|---|---|---|---|
| `Reconcile { forge, adopted }` | Forgejo, managed, owned or fake | each managed, owned or fake start; a managed connect; the box's `started` (1) | `README.md`, section 4 |
| `Adopt { forge, repository }` | Forgejo, managed, owned or fake | an adoption, before its reads; the fake's start (`playground.md`) | 4.5 |
| `Create { forge, org, repository, files }` | Forgejo, owned or managed | an owned forge's, after reconcile at the box's `started` or a start, while its record keeps a first repository's intent not yet made (`owned.md`, 7); a managed connect's with `first` (5) | 4.9 |
| `Check { forge }` | Forgejo, connected | a connect; a replacement | 4.6 |
| `TakeOver { forge, admin_token }` | Forgejo, owned or fake | a start or the box's `started` with a handover, even with a kept `AdminToken` (`owned.md`, 5) | 4.7 |
| `Reachable { forge, installation?, page }` | every kind | the picker | 4.8 |
| `Manifest { forge, target }` | GitHub | Connect, managed | 1 |
| `Convert { forge, code, state }` | GitHub | the redirect back | 2 |
| `Install { forge }` | GitHub | Install the App, or Install on another account | 4 |
| `Accept { forge, installation, state? }` | GitHub | the setup route's visit; the owner's accept of a stray (`github.md`, 4) | 4 |
| `Reconcile { forge }` | GitHub | after each connect | 6 |
| `Paste { forge, app }` | GitHub | Connect with a pasted App; Replace | 5.8 |

- **What each does, and when the routing runs it,** is ../forge/README.md's (the
  setup's host); each plan's steps, its bounds and what it refuses are
  skein's.
- **`Reachable`** serves every kind, pages counted from 1, at most
  `reachable` rows, which temper gives from `picker.page`
  (../configuration/limits.md), and ends `Done(Reachable { rows, more
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
  lacks named, and the App's account; `Made`, a `Convert`'s, with the
  App's account; `Person`, a `Reconcile`'s given a person's login, with
  that user's id or none (3).

## 3. Reconcile's given values, and its answers

- **The plan is skein's.** Its seven steps, in this order (1 the admin; 2
  the bot, marked `<word>:bot`; 3 the bot's two tokens; 4 the bot's id;
  5 each org, when owned; 6 each adopted repository's access, the bot a
  collaborator and the repository's hook; 7 the person's id, when a login
  is given), each read first and written
  only where the forge differs; its wait for the forge; its bounds; and
  what it refuses are held by skein's `forge/setup/forgejo.md`, section 1. It
  does no io and reads no configuration: it is given values, says each
  request, and the setup host runs it.
- **Its given values, from temper's configuration:**

  | Reconcile's value | temper's |
  |---|---|
  | the forge's number | its record's (../forge/engine.md, 1) |
  | the bot's login and email | its record's bot (../forge/engine.md, 1); at a connect as its admin of a forge connected before, the connected user's login (5) |
  | the tokens' names | `temper-api` for the API's, `temper-git` for git's |
  | the orgs, and their owner | none: reconcile makes no org; an owned forge's org is its first repository's, made by `Create` (`owned.md`, 7) |
  | the person's login | an owned forge's person's, on a box (`../../draft/forge-identity.md`, section 9), whose answer the connector keeps as the forge's link to its person (the same, 4.1); none otherwise, a fake forge's among them |
  | the repositories | the forge's adopted repositories, the root's, restored before the start's reconcile |
  | the name a failure gives the admin | `temper-admin` |
  | the listener's address | `application.webhooks.listener`'s, when configured and in the forge's reach: an owned or fake forge's, and a managed forge's whose API is on loopback (5); none otherwise. Each hook's URL, `http://<address>/hooks/<number>`, is derived from it (skein's `forge/setup/forgejo.md`, 1.1) |
  | the word | `temper`, so the bot's mark is `temper:bot` |
  | its limits | `repositories` from `forge.client.repositories`, `reachable` from `picker.page`, and `deadline`, `backoff` and `backoff_max` from `setup.deadline_ms`, `setup.backoff_ms` and `setup.backoff_max_ms` (../configuration/limits.md); `orgs` 1, the least the kit takes, since temper gives no org |

  The API's address is the forge's record's place (../forge/engine.md, 1).
  temper-forge-setup maps the setup onto these values and runs the
  kit's check on them, at each start and for every command (`README.md`, section 7
  ); a value the check refuses comes back as its `Place` (skein's
  `forge/setup/README.md`, 4.1), and is named by the path this table gives for
  it.
- **What the setup host answers, as needs:**

  | Reconcile asks for | The setup host answers |
  |---|---|
  | the admin's token | at a managed connect, the pasted value; otherwise the `AdminToken` record |
  | the API and git tokens, the bot's id | the `ApiToken`, `GitToken` and `User` records, or none when absent |
  | the webhooks' secret | the `WebhookSecret` record, or none, and the plan that makes a first hook then draws one (4) |

  Each answer is a record or a pasted value, never a file: a forge's
  `api_token`, `git_token`, `user` or `webhook_secret` file, from a
  temper before the records, is not read, and reconcile makes what is
  missing afresh (`README.md`, section 3). Each item reconcile writes is kept as the
  forge's record; a kept `WebhookSecret` is never drawn again. The
  fake's start-time adoption (`playground.md`) reads the same records, after its
  handover and reconcile.

## 4. Access at adoption

- **`Adopt { forge, repository }`** runs before an adoption's reads on a
  managed, owned or fake Forgejo (skein's `forge/setup/forgejo.md`, 1.5): the bot made
  a collaborator with write, and the repository's hook made when the
  forge is served the listener (3), with the forge's secret; a managed
  forge on another host is served none (5). A person's adoption gives
  the plan the path its request carries, since the bot may not see the
  repository yet; the
  fake's start-time plans give ids. Stopped, the adoption
  is refused, named (`../../domain/forge/README.md`, access at adoption). A
  `403` here, at reconcile's step 6 or at the picker's read, is the
  admin's token without `write:repository` (skein's 4.3, step 1): it
  is shown as that scope missing, named with its step.
- **A stop reads as one of the adoption's refusals**
  (`../../client/forge/family.md`, 6), and the log names the plan, its step and
  its why:

  | The plan's stop | The adoption's refusal |
  |---|---|
  | `Repository`, `Missing` | not found |
  | `Admin`, whatever its why; `Refused(_)` at `Repository` | no access |
  | `Down` or `TimedOut`, or the run ended by the forge's end | unreachable |
  | the host busy (1) | in flight |
  | any other | invalid answer |
- **At each start,** reconcile's step 6 checks each adopted
  repository's access, and repairs what changed.
- **A secret drawn on a connected forge:** a forge served the listener
  with no repository at its reconcile connects with no secret, so with
  no path on the listener (../forge/README.md, section 4). The plan that makes its
  first hook draws the secret. Once that `WebhookSecret` is kept, before
  the plan goes on to the hook, the engine hands the forge's access again
  with it (skein's `forge/connection.md`, 3.5), so no delivery of that
  hook meets a `404`. It is a replacement, ending the calls then open on
  that forge as one does; a forge with a repository already has its
  secret from its reconcile.
- **A fake forge:** its repositories are
  its setup's (`init.md`), so the setup host runs `Adopt` for each of them at the
  engine's start, before the forge connects, in place of each
  adoption's (`playground.md`, the fake's start-time adoption).
- **Nothing org-wide:** the bot reaches the repositories adopted, a
  person's own among them, and no other.
- **A connected Forgejo, and a managed one forgotten,** grant nothing at
  adoption: the person makes the bot a collaborator, and the repository
  polls unless the person adds its hook (`README.md`, section 4).

## 5. Connecting as its admin, and forgetting

- **"Connect as its admin"** (`Connect` with `ForgejoAdmin`;
  `../../client/forge/README.md`, the deployment's page), for a Forgejo temper does
  not run, on this host or another: its connect makes the forge's record,
  managed, from the address the person gives, its API's host, port and transport, its git and web
  bases, and its git program when given, numbered next (../forge/engine.md, 2),
  in the connect's admission, before any run; or, on a managed Forgejo
  kept and neither connected nor connecting, connects it again. Any
  other forge refuses it, *not taken*. The site admin's token, with the
  preparation's scopes (`owned.md`, 3), is pasted and kept by jig until its run
  ends. One run holds the paste:
  `Reconcile` reads it, never an older `AdminToken` record; its writes
  are kept, then the forge's `Origin` under `AdminOrigin`. With `first`,
  `Create` (`owned.md`, 7) runs with the same paste. Only after both plans are Done
  is the token kept as the forge's `AdminToken` record, and the forge
  connects, managed.
  Without `first`, Create is skipped. This connect never runs `TakeOver`.
  Every end erases the carried value and drops the held token; a stop
  keeps no admin token. Each later start
  reconciles from that record and connects, with no Create or `TakeOver`;
  without the record it offers Connect, not connected. This form serves
  no organizations or organization owner.
- **Its listener:** a forge whose API is on loopback, by its record's
  address as resolved, so on this host, is served the listener, and its
  hooks are made and checked as an owned forge's are (4, `README.md`, section 6).
  One on another host is served none and polls: a loopback listener is
  out of its reach, and given one it would post, signed, to its own
  loopback.
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

## 6. What holds of every plan on Forgejo

- **Plaintext to loopback alone, for owned and fake:** an owned or fake
  forge's `transport` is plaintext, and one whose transport is TLS is
  refused at start, naming it (`README.md`, section 9). A managed forge's is TLS, or
  plaintext to a loopback host alone; plaintext to another host is refused
  at start, naming it.
- **The bot's email** is its record's bot's (../forge/engine.md, 1) when
  reconcile makes the bot, and stays as it is after: an existing bot's
  email is the person's to change, on Forgejo. temper reads no mail of
  the bot's, and its commits name its bot's git identity, the record's
  (../forge/engine.md, 1).
- **The scopes** of the tokens are `README.md`, section 4's.
- **What it cannot set:** Forgejo's settings, which its API does not
  change (skein's `forge/setup/forgejo.md`, section 1): the webhooks'
  `ALLOWED_HOST_LIST`, and whether Actions are enabled. A managed forge
  on this host whose host list refuses temper's listener delivers no
  webhook, and polling keeps up (`README.md`, section 6); `temper help forgejo` says
  what to set. A delivery's failure stays the forge's: temper reads no
  delivery's outcome from a Forgejo, so none reaches a task, the forge's
  row or its connection.

## 7. Open questions

- **A story of Reconcile over TLS.** The managed string form takes a
  forge on another host over TLS and the service hands that transport to
  skein (`reconcile.md`, 6), but every test of the admin connect, the process story
  among them, reaches its Forgejo over loopback plaintext: a TLS story
  waits for a fake forge that serves TLS (skein's `forge/setup/README.md`,
  section 6).

- **A secret drawn at a reconcile with no repository,** so that a forge's
  first hook needs no access handed again, which ends the calls then open
  (`reconcile.md`, 4). It is a change to skein's plan (skein's `forge/setup/forgejo.md`, 1.3),
  which the request `managed-listener` left to keep to one task.

- **An adopted row's kinds after the forge changes** (`reconcile.md`, 4;
  `../../domain/forge/adoption.md`, 2): a row keeps the kinds its adoption read,
  and only a capability refusal reads them afresh, which a row that may
  not push never meets. So a repository adopted while temper's user could
  only read stays so in its row until it is adopted again, whatever the
  forge says after: a person's write on a connected Forgejo, or reconcile's
  repair on a managed one, for a row whose adoption ran no `Adopt`. "Work
  there fails until it has write" (`../../client/forge/family.md`, 6) does not
  say so. Reading each adopted repository's permission at a connect is
  what would close it.

- **A managed forge whose host list refuses the listener.** Default:
  `temper help forgejo` says what to set, and at run time nothing tells
  the person but the hints that never arrive (`../../domain/forge/adoption.md`
  ).

- **Two deployments on one managed forge.** Default: each its own bot
  login, carried by its connect (`reconcile.md`, 5), and so its own collaborators and
  hooks; the marker does not tell deployments apart.
