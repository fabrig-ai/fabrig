# On the engine's side

The engine's service is `../protocol/README.md`'s (`../protocol/README.md`, section 4). What it
does for the client:

- **The clients' listener** is `temper-service`'s: TCP on the loopback
  address temper's configuration names
  (`../protocol/configuration/engine.md`), at a fixed port, never 0,
  which `temper-box init` chooses, since a GitHub App's redirect and setup
  addresses name it (`../protocol/configuration/README.md`, the clients'
  listener; `../protocol/setup/box.md`, 1). On
  it, it mounts jig's clients' component (`jig-protocol-engine-clients`)
  with jig's default HTTP mapping's engine end
  (`jig-protocol-engine-clients-http`) under the prefix `/jig` (jig's
  `protocol/service.md`, section 2). A request's path is offered first
  to jig's component, which says whether it claims it (jig's clients'
  listener claims `/jig/...`); then to temper's forge routes,
  `/forges/...` (the routes, below); then to the browser's page and
  bundle, the binary's own or the one the configuration names, or else,
  from a binary built without its own and a configuration naming none,
  to the page for no bundle (`web/README.md`, 2.2 and 2.6). Every other
  path is answered not found.
- **The routes,** temper's beside jig's: a GitHub connect's browser
  steps, `GET /forges/N/github/manifest`, `redirect` and `setup`, N the
  forge's number, each the setup's host's run for the state its query
  names (`../protocol/forge/README.md`, the setup's host). Their answers, their
  headers, their `302`s and their refused pages are `forge/served.md`, 1's.
  The state is their only authenticator: 128 bits or more, good once,
  for its life (`setup.state_life_ms`), for that forge alone; they take no
  cookie and no credential of jig's. A query malformed or past
  `routes.query_bytes` is answered `400`, a method other than `GET`
  `405`, and a forge the store does not keep or not GitHub `404`. No client of
  temper's calls them: GitHub sends the person's browser there.
- **A client's leaving, on the listener.** `temper-service` reads each
  connection's events below skein's server. When the peer ends a
  connection before its call is finished, it aborts the connection, as
  it does one that fails, and the call's owner hears the connection
  closed: for a call under `/jig`, jig's engine end, which takes it as
  the call's end (jig's `protocol/client/default-http.md`, 4: "A peer's end is
  the call's end"). So a stream ends when the client closes it or its
  process ends, and its room among the profile's streams is free for
  the next call at once (`ctl/README.md`, 6.2).
- **The printed link opened in a browser:** a `GET` of the link's route
  spends nothing, and the mapping leaves it to the application's page
  (jig's `protocol/browser/README.md`, section 11). With a bundle, temper's
  server answers it with the page, which signs the browser in by it
  (`web/README.md`, 2.4); with none, the page for no bundle, which says
  so and names `temper-box open` (`web/README.md`, 2.2), with
  `Referrer-Policy: no-referrer`, and the link is `temper-box open`'s alone
  (`line.md`, 1).
- **jig's example guards, as they are** (jig's `protocol/browser/README.md`,
  12.2), the Host allow-list holding the listener's own address. The
  terminal sends its credential as a bearer token; the cookie is the
  browser's (`web/README.md`).
- **jig's clients' component** (jig's `protocol/client/README.md`) serves jig's
  wire's versions and, for the family `FORGE`, `CLIENT_VERSION` alone
  (`forge/records.md`). temper gives it no `Mint` in standalone: the printed link and
  services' credentials are jig's own ways of signing in (jig's
  `protocol/client/sign-in.md`).
- **A stream refused by jig's component** emits a journal line through
  the service's operator notes and the shell's stderr. It names the
  client's sign-in number and the queue bound or translation path and
  reason, with no document content (jig's `protocol/client/README.md`, 5).
- **The forge's request, decoded before the root** (jig's
  `protocol/client/README.md`, section 8). An application request of the
  family `FORGE` at `CLIENT_VERSION` is decoded by
  `temper-client-protocol-wire`; one that does not decode, or of
  another family or version, is answered undecodable, as jig's
  reference service answers it. The routing fills the adoption's host
  from that forge's `git` base, as its record keeps it: its host, with
  `:port` when it names one, as the forge's resource paths have it
  (`../protocol/forge/engine.md`, 1). It leaves the host empty for a forge
  number the store does not keep, and hands the root the party's
  keyed request: its reply, its sign-in, its key, the digest of the
  record's bytes, and the adoption, or another of the FORGE requests
  (below) with the number and digest of the value it carried.
- **The FORGE requests** beside `Adopt` (`forge/repositories.md`, 1 and `forge/deployment.md`), each
  keyed, each naming the deployment as its scope, `Dismiss` and `Looked`
  alone naming a project (jig's
  `protocol/client/README.md`, section 8). `forge` in a `Connect` or a
  `SetUp` is `Making` (`forge/records.md`): a kept forge's number, or a forge to make,
  which the request's decision makes as a record of the forge
  connector, numbered next, before its run starts
  (`../protocol/forge/engine.md`, 1 and 2):

  | Request | The value it carries | What the engine does | Its outcome |
  |---|---|---|---|
  | `Connect { forge, how: GitHubManifest { back? } }` | none | the setup's host's `Manifest` run, for the person's own account, always (`forge/served.md`, 1), ending `Done(Manifest { state, target, manifest })`, from which temper renders the manifest's page (`forge/served.md`, 1); the page and `back` held with the state | a link to the manifest's page for a fresh state (the routes, above): with `back`, a path on the page's own origin, `/forges/N/github/manifest?state=S`; without it, the terminal's whole address on the clients' listener |
  | `Connect { forge, how: GitHubApp { id, read_only?, key } }` | `key`, the App's private key, PEM as GitHub gives it; `id` rides in the request | `Paste`: the key decoded and matched to the App `id` names, the App's user found, its hook pointed at `https://SLUG.invalid/` unless `read_only` leaves it as read (no webhook secret on GitHub: changes are heard from the App's delivery log), its permissions and events checked | connecting |
  | `Connect { forge, how: ForgejoAdmin { first? } }` | the site admin's token, a bundle of one `AdminToken` item, at most 4 KiB in all | `Reconcile` with the paste, its `AdminOrigin` write kept, then `Create` of `first` when given; only after Done is `AdminToken` kept, then the forge connects; every end erases the carried value; never `TakeOver` (`../protocol/setup/reconcile.md`, 5); a managed forge (`ForgeRow.admin`) offers the client's admin Details, a new one's `ManagedForge` naming its addresses, with `first`'s org, repository and runner label | connecting; then the watched row, its typed `ended_stop` on a stop, or `first` made, unadopted |
  | `Connect { forge, how: ForgejoTokens }` | the bot's API token and git token, and the hooks' secret when given: `ApiToken`, `GitToken` and `WebhookSecret` in one bundle | `Check { forge }`: both tokens answer as one user, not a site admin, and on a kept forge as its kept user; on Done the run's `User` write kept, then the tokens and the secret under the forge's numbers, then `Origin`, its login kept as the record's bot, and the forge connects; every end erases the carried value (`../protocol/setup/README.md`, section 4); `forge` a kept connected Forgejo, not connected or stopped, or a new one, `connected` naming its addresses | connecting; then the watched row, connected as its login, or its last connect's line on a stop |
  | `Install { forge, back? }` | none | `Install`, GitHub alone; `back` held with the state | a link, the install page for a fresh state |
  | `Accept { forge, installation }` | none | `Accept` with no state, GitHub alone (`../protocol/setup/github.md`, 4) | accepted; forge refused, not found |
  | `Replace { forge }` | a new App key, one `Pasted` item; or a new API token, git token or both, and the hooks' secret when it changes | `Paste` with the kept App id, or `Check` against the kept user: it must answer as the forge's user; each new record is kept over the old, then the forge connects with them, the calls its old access began ending `Forbidden` (`../protocol/forge/engine.md`, 2) | connecting; then the watched row, connected, or its last connect's line on a stop, the forge as it was |
  | `Forget { forge }` | none | the admin's token erased; a managed forge becomes connected (`../protocol/credentials.md`) | forgotten |
  | `SetUp { forge }` | none | an owned forge, new, or kept and not set up, or whose set up failed, whose setup says `"start": "asked"`: the routing asks the box for the forge's group, `start FORGE` on its pipe (`../protocol/setup/box.md`, 1), refused "not under the box" without a pipe and "busy" while a group starts; at the box's `started`, its handover taken, then reconciled and connected (`../protocol/forge/engine.md`, 2) | setting up |
  | `RemoveForge { forge }` | none | the forge ended in the request's decision, then removed whole by the kept acts that follow it: its live work cancelled, its box group stopped and its directory removed, its records erased (`../domain/forge/adoption.md`, 1); refused for the playground's forge, whose reset removes it | removed; remove refused, `ForgeRefusal` |
  | `MakeRepository { forge, org, name, from, into }` | none; its scope the project `into` names, or the deployment | the row kept in the request's decision; the setup then runs `Create` or `Copy` and the connector adopts into `into` (`../domain/forge/adoption.md`, 5; `../protocol/setup/owned.md`, 9) | kept; make refused, `MakeRefusal`; then the watched row |
  | `SetAway { project, on, until }` | none; its scope the project | the party's `Away` row moved in the request's decision, and the subject `away` marked, a row that comes out the same writing nothing (`../domain/forge/attention.md`, 4) | away; forge refused, too large |
  | `Overturn { project, number }` | none; its scope the project | the party's `Decided` row marked overturned in the request's decision, and the subject `decided` marked (`../domain/forge/attention.md`, 5) | overturned; overturn refused, already or unknown |
  | `SetDraft { project, repository, path, change }` | none; its scope the project | for `ready`, the repository's landing branch read, then the path at its head, the request held in memory while they are out; then the project's `LandedDraft` row made or moved in the request's decision, and the subject `drafts` marked; a row that comes out the same writing nothing (`../domain/forge/drafts.md`, 2) | drafted, with the row; draft refused, no repository, path, missing, not a file, unreachable, unknown or full |
  | `Dismiss { what }` | none | the party's `Dismissed` row saved in the request's decision, and the subject `dismissed` marked, a row held already writing nothing (`../domain/forge/attention.md`, 1) | dismissed; forge refused, too large |
  | `Looked { project }` | none; its scope the project | the party's `Looked` row moved in the request's decision, the subject `status` marked, and, at a new sitting, the status agent's run asked (`../domain/forge/attention.md`, 2 and 3) | looked; forge refused, too large |
  | `StartPlayground` | the scripted model's key, which the routing reads from its file at each start and carries as a pasted value | the playground's start (`../protocol/setup/playground.md`): the `Playground` row written starting when none, its fake forge's record made and the box asked for its group, then the scripted model's account added and its repository adopted into its project, each unless done; refused "not offered" while a real forge is kept | running, once all are done; start failed, naming why |
  | `EndPlayground` | none | the playground's reset (`../protocol/setup/playground.md`): the `Playground` row resetting in one decision, then its kept acts in order, the project deleted, the fake forge removed, the scripted account removed, the row erased | reset, once the row is erased; end refused, not offered, with no playground |
  | `Reachable { forge, installation?, page }` | none | on every kind, the setup's host's `Reachable { forge, installation?, page }` plan (skein's `forge/setup/forgejo.md`, 1.8; `../domain/forge/README.md`, what the picker offers): on GitHub one accepted installation's repositories, `GET /installation/repositories` paged by number on a token minted for the page; on a connected Forgejo what the API token can push to; on a managed or owned Forgejo every repository the site admin can see by id ascending, read on the admin's token, which stays the setup's host's (`../protocol/forge/README.md`, the setup's host) | a page, `{ rows, more }`: at most `picker.page` rows on every forge, clipped to the forge's own page cap, each the repository's id, its owner's id and login, its name and whether it is private; `more` set when the page is full |

  `first` is a `FirstAsk { org, repository, label }`: the
  first repository's fields, as `SetUp`'s, with no project, and `label`, the runner label
  its starter workflow runs on (default `docker` in the client and ctl;
  1 to 64 of letters, digits, `.`, `_` and `-`).

  Names in a carried value's braces are its bundle's items
  (`../protocol/credentials.md`, pasted values), never fields of the
  record, which holds the forge, the kind of connect, and a GitHub App's
  id. `installation` is an accepted installation's id, GitHub alone: one
  the forge has not accepted, or one given for a Forgejo, stops the plan
  (`Installation`, `Missing` or `Unexpected`); `page` a page's
  number, from 1, and the picker reads the next page when its list nears
  the end or a search still needs rows; a full last page may be followed
  by an empty answer. No Next or First page control is drawn. The plan’s
  `reachable` limit is given from `picker.page`. The request names no
  project (`README.md`, section 5).
  The forge's kind, mode, user and installations are the deployment
  part's, never the page's; an installation's page there is built by the
  engine from the forge's `web` address, as its record keeps it (`forge/repositories.md`, 4).
- **A carried value, as jig carries it** (jig's `protocol/client/README.md`,
  section 3). jig's clients' component strips the value at decode and
  keeps it in the secrets' store under temper's tag, at a number it
  gives it from 2^63 up (`../protocol/credentials.md`, pasted values),
  before the core sees the request; the core gets the number and the
  value's digest, and the value never reaches the core, a document or a
  log. The request names no number and the client draws none. A value
  is at most 4 KiB, jig's bound on a carried value. A carried value the
  core leaves unused, its request refused, replayed, joined to another
  or cut by a stop, is named in one output of the core's, which temper's
  root forwards to jig's clients' component to erase.
- **`back`,** on a GitHub connect and an install, is the page's address
  the setup route sends the browser back to once GitHub is done
  (`forge/served.md`, 1). The service checks it at decode, before the root:
  one leading `/`, not `//`, no `\`, no scheme, of visible ASCII, at most
  512 bytes; one that is not is answered undecodable. None, from the
  terminal, leaves the route's own page.
- **Who may:** a person whose role on project 0 is `Owner`; in
  standalone, the deployment's owner, for every request above. jig admits each by key and
  refuses at admission as for any request, in jig's words; the root
  refuses any other party's, before a forge is called or a run starts,
  as the outcome refused (role).
- **The root runs it** (`../domain/root.md`; `../protocol/forge/README.md`,
  the setup's host): it keeps, in memory, a connect's or a replacement's number and
  digest in the connect's own record, refuses it or asks the routing to
  run its plan, and answers. A connect's end, connected or why not, is
  the forges' page's to read (`forge/deployment.md`), never the answer's: a
  run's failure, "the pasted value changed", a GitHub App lacking
  permissions or events, each named (the setup's host's `Lacking`;
  `forge/repositories.md`, 4), are said there; a connect a restart cut is said
  nowhere until a connect is kept (`../protocol/credentials.md`, 2.2 and
  section 8). The outcomes:
  - *connecting:* the run started; a `Manifest` or an `Install` that
    stops, other than busy, answers it too, and its failure is said on
    the forge's row as the last connect's end, never in the answer;
  - *a link:* the address to open, printed and never opened (`line.md`, 5);
  - *forgotten:* the admin's token erased;
  - *accepted:* an installation kept by `Accept`, with its id; an
    `Accept` that stops answers *forge refused* with its reason (`forge/records.md`);
  - *a page:* the picker's (`forge/family.md`, 4);
  - *refused:* `Connect`'s, `Install`'s and `Accept`'s as `ForgeRefusal` (`forge/records.md`) has
    them: the role; no such forge; the forge's kind or mode does not
    take it (a Forgejo's install, a forget on a forge not managed, a
    replacement on a managed or owned forge); busy, since a forge runs
    one writing plan, a connect, an install or a reconcile, and one
    `Reachable` beside it, and a second of either kind is refused, as is
    one past `setup.runs`, which counts both; or, for `Reachable`, the
    forge's reason below, an installation the forge has not accepted or
    one given for a Forgejo (the plan's `Installation` stop), and on a
    managed or owned Forgejo the admin's token's, missing (`Missing`) or
    not a site admin's (`NotAdmin`), as the plan's first step has them
    (skein's `forge/setup/forgejo.md`, 1.3 and 1.8).
- **The root routes an adoption** (`../domain/forge/adoption.md`;
  `../domain/routing.md`, 4): jig's people child admits it by key, and
  refuses at admission as jig's answer; admitted, it goes to the forge
  connector, which refuses a forge it does not keep (no such forge) or not
  connected (not connected), locates the repository by owner and name
  (not found when the forge has none), and makes jig's adoption request
  on the party's behalf
  (`adopt_repository_ask`), which the core judges as the party's own.
  The forge connector's adoption then reads what temper may do there.
  The outcome is kept under the request's key for the retention and
  answered once durable (jig's `client/domain/contract/requests.md`, 3). The
  forge keeps at most `forge.applications` (32) such outcomes and the
  requests in flight beside them; one admitted while they are full is
  refused at once, jig's `Limit` of kind `ApplicationAnswers` with that
  bound, which jig keeps under the key (jig's `domain/people/README.md`, its
  application requests), never answered busy until the retention ends.
  The key keeps that refusal for the retention, so ctl says it as
  "temper keeps at most N answers to requests like this within a day:
  ask again later, under a new key", N the bound.
- **The forge's errors as reasons,** by the forge connector
  (`temper-engine-domain-forge`'s `reason`, from `skein-forge`'s
  `Error`, skein's `forge/README.md`, section 3.4): `NotConnected` is not
  connected; `NotInstalled` is not installed; `Missing` is not granted
  on a repository whose account has an installation of selected
  repositories, with that installation's page, and not found
  otherwise; `Forbidden` is no access; `Refused` is refused; `Busy` is
  in flight; `TooLarge` and `Full` are too large; `Unavailable`,
  `Timeout` and `RateLimited` are unreachable; any other is an invalid
  answer. A `Reachable` whose plan stops on the forge's error carries the
  same reasons.
- **The outcome, encoded** by `temper-client-protocol-wire`, goes back
  as the family's answer to jig's clients' component, an adoption's and
  every other FORGE request's alike, kept under the request's key for
  the retention, as an adoption's is, at the store's tag 15
  (`../protocol/store.md`), a refusal among them. An outcome that
  carries a link or a page is kept in memory alone, until its state's life ends
  (`setup.state_life_ms` after `Done`): the same key sent again while it
  lives answers the same link and starts no run; after its end, or a
  restart, the key is unknown, as one past the retention is. A
  `SetUp`'s outcome is kept in memory alone too, as the ask it answers
  is (`../protocol/forge/engine.md`, 8): the forge connector holds the
  latest per kept forge, and one more for a number it does not keep,
  under its request's token, and the same key sent
  again while it is held answers the same outcome and asks the box
  nothing; once a later `SetUp` of that forge replaces it, or after a
  restart, the key answers unknown.
- **A forge's set up, answered** (`../protocol/forge/engine.md`, 8): the
  root admits `SetUp` by key, of the deployment's scope, and the forge
  connector by the forge's connection; the routing's box link then
  answers it, the connector's own refusal among what it answers: the
  box asked, *setting up*, or refused, *not under the box* or *busy*,
  or the connector's *not offered*. The answer is the request's
  outcome, and the set up's course, its steps, its `started` and its
  `failed`, is the forge's row's (`forge/deployment.md`), never the answer's.
- **The deployment's forges, watched** (`forge/records.md`; `forge/deployment.md`): the
  routing hands the root the watch of `forges`, with the deployment's
  scope; the core admits it on the party's deployment role and asks the
  forge for the document, and again each time the forge marks it: at
  every connection the routing tells it and at the box's close
  (`../domain/forge/adoption.md`, 1). Every field of a row is the forge
  domain's: the routing encodes the document as it is, adding nothing.
- **The forge's reads and watches** are decoded by
  `temper-client-protocol-wire` the same way and go to the root
  (`../domain/routing.md`, 3; `forge/served.md`). One the wire has
  no record for does not decode: the routing answers the read with no
  record (unknown) and refuses the watch as unknown, without the root,
  as jig's reference service answers an undecodable one.
- **A change's panel** (`forge/records.md`; `forge/served.md`, 2): the routing hands the
  root the watch, with its scope and the change's number; the core
  admits it on the party's role in the project and asks the forge for
  the document (`../domain/routing.md`, 3), and again each time the
  forge marks its subject. Each document leaves held in the decision
  that made it, released once that decision's commit is durable, and
  the routing encodes it, building the pull request's address from
  that forge's `web` and its kind (`forge/served.md`, 2).
- **The playground's read** (`forge/records.md`, `forge/family.md`, 7) is answered by the root's forge
  connector, to any signed-in party, from the `Playground` row
  (`../protocol/setup/playground.md`): none with no row, and otherwise its
  forge, its project once made, its model's LLM provider, its
  repository's owner and name, its state, starting, running or
  resetting, and its `llm.asked` cues in order.
- **The playground's start and reset, answered**
  (`../protocol/setup/playground.md`): the root admits `StartPlayground` and
  `EndPlayground` by key, of the deployment's scope, as `SetUp`; the
  forge connector answers a start once its acts are done, or at the
  first that fails, and a reset once its last act has erased the row,
  and holds each answer in memory alone, as `SetUp`'s: the latest
  one's, under its request's token, the same key sent again while it is
  held answering the same outcome; after a restart, the key answers
  unknown. A start or a reset while another of its kind is under way
  takes that one's answer. The course is the read's, the forge's row's
  and the `accounts` document's; the answer says only how it ended.
- **A forge's removal, answered** (`../domain/forge/adoption.md`, 1): the
  root admits `RemoveForge` by key, of the deployment's scope, and the
  forge connector answers it in the decision that ends the forge,
  removed, or refused; the removal's kept acts go on after the answer,
  resumed at each start until the forge is gone. The forge leaves the
  `forges` watch's rows in that decision.
