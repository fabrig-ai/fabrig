# Setting up a forge

Provisional, 2026-10-20, revised 2026-10-23. `skein-forge-setup` makes a code forge ready
for a service. It ends with an access: the identity the service acts as,
its credentials, the repositories it may write and the hooks that
deliver to its listener. Each making is a plan, a step machine that says
each request and fails naming its step: Forgejo's reconcile and its
access per repository, and GitHub's App made by manifest, installed and
checked. Its host component runs the plans in the owner's loop.

## 1. In one page

- **A crate of step code.** Its plans depend on skein-lib, skein-json
  and `skein-forge` (../README.md, section 2), name no host, read no file and
  open no socket. Its host component asks its owner's io (4.3).
- **It ends with an access** (section 2): an identity, an API and a git
  credential, a reach, and for Forgejo a webhooks' secret and hooks. The
  component (../connection.md) needs these and nothing more.
- **Three modes** (section 3): connected, where a person makes the
  access; managed, where the service makes it with an authority the
  person gives; owned, where the service runs the forge and then sets up
  as managed. Only Forgejo can be owned.
- **The admin's token is kept,** by the owner, and read by the setup
  host alone, for reconcile and for access at adoption (section 3). The
  forge component never holds it.
- **Reconcile is Forgejo's plan** (`forgejo.md`, section 1): a pure machine, events in
  and at most one need out, with six steps in a fixed order. Each reads
  first and writes only where the forge differs. Beside it: access to
  one repository at its adoption (`forgejo.md`, 1.5), connected's tokens checked
  (`forgejo.md`, 1.6), an owned forge's admin's token taken over (`forgejo.md`, 1.7), and a page of
  the repositories it reaches, for a picker (`forgejo.md`, 1.8).
- **GitHub's plans** (`github.md`, section 1): the App's manifest as a pure function,
  its conversion kept before the browser is answered, the bot's id, an
  installation accepted by its state, the App's hook and checks, an App
  a person made, pasted, and a page of an installation's repositories
  (`github.md`, 1.9). Each state is one-time (`github.md`, 1.7).
- **Given values, not configuration.** The owner gives the bot's login
  and email, the tokens' names, an owned forge's orgs and their owner,
  the person's login, the admin's name for a failure, the listeners'
  addresses, the word and
  its limits (`forgejo.md`, 1.1, `README.md`, 4.3). A plan asks its host for each item.
- **Items are named, never placed** (4.4). Each has its bytes pinned, and
  the crate decodes them for the owner. The owner keeps each written
  item durably before it acts on the plan's end.
- **Two hosts** (section 4): the host component, which an owner runs in
  its protocol layer beside the forge component, and the one-shot host
  that skein-shell's `drive` runs for a command.
- **Verified** against answers and in worlds against the fake forge
  (section 5), on the facts of Forgejo in `forgejo.md`, section 2 and of GitHub in
  `github.md`, section 2.

## 2. The result: an access

The setup ends with what the component needs (../connection.md,
section 3.5), and nothing more:

- **The identity** the service acts as, by the forge's id: Forgejo's
  bot, or GitHub's App's bot user, `<slug>[bot]`.
- **Its API credential and its git credential,** of the forge's kind
  (../README.md, section 7). Forgejo's are tokens that do not expire, with
  the scopes of `forgejo.md`, 1.3, step 3, both the owner's. GitHub's is the App's id
  and private key; the component mints each installation's tokens from
  it, git's among them (../connection.md, section 5).
- **Its reach:** the repositories it may write. Forgejo's is the bot a
  collaborator with `write` on each repository the owner adopted (`forgejo.md`, 1.5).
  GitHub's is the App's installations the owner accepted (`github.md`, 1.4).
- **Forgejo's webhooks' secret, and hooks** that deliver to the
  service's listener (../connection.md, section 6), one on each
  adopted repository, at `/hooks/<number>`, the number being the
  owner's for the forge. GitHub's App has a hook at no listener and no
  secret kept: the component reads its delivery log (`github.md`, 1.5).

An access is of one forge. One without hooks is whole: a forge given no
listener, or one not on loopback, gets neither hooks nor secret, and the
service polls (../client.md, section 4). The crate holds no access:
it gives its host each value to keep, as items (4.4), and the owner
builds the access from them.

## 3. Three modes

- **Connected:** the person runs the forge and makes the access.
  - On Forgejo, the service strictly needs the identity's API token, git
    token and id, and for webhooks their secret. The person makes a
    forge user for the service, not a site admin, with its two tokens
    (`forgejo.md`, 1.3, step 3) and its write access to each repository the service
    works on, as a collaborator or through a team; and optionally each
    repository's or org's webhook to the listener, with the secret and
    the events of ../README.md, section 6, and Forgejo's `[webhook]
    ALLOWED_HOST_LIST` letting it reach the listener. A repository with
    no hook is polled. The two tokens are checked as one user (`forgejo.md`, 1.6).
  - On GitHub, the person makes an App and pastes its id and private
    key; the service checks it, names what it lacks, sets its hook and
    installs it through its own link (`github.md`, 1.8).
- **Managed:** the person runs the forge and gives an authority: a
  Forgejo site admin's token, or the making of a GitHub App by its
  manifest (`github.md`, section 1).
  - **The admin's token is kept** by the owner, durably and privately,
    and read by the setup host alone, for reconcile at each start and
    for the access at each adoption (`forgejo.md`, 1.5). The forge component never
    holds it, and no request of the running service carries it. It is
    asked for once, never at each adoption, and stays after setup.
  - **Forgetting it** makes the forge connected: the bot, its tokens and
    its id stay, and no reconcile runs.
- **Owned:** the service runs Forgejo on loopback and CI's runner beside
  it, makes an admin on it, and sets up as managed with that admin's
  token, which it takes over from the preparation (`forgejo.md`, 1.7). The preparation
  and the running are the service's (section 6). Reconcile differs in
  one step: it is given orgs to make when absent, private and owned by
  the person whose login it is given (`forgejo.md`, 1.1; `forgejo.md`, 1.3, step 5).
- **Layered.** Each mode makes the values the one below reads:

  ```
  owned      prepare, run   ──► a forge on loopback, an admin's token
  managed    reconcile      ──► the access's values
  connected  the service    ◄── reads them
  ```

  The service is the same in every mode. Nothing passes between the
  layers but the values and the forge's number.
- **A mode is per forge.** Owned to managed: the service stops running
  the forge, and the admin's token it took over stays valid. Managed to
  connected: the admin's token is forgotten, and the values stay as last
  written. Connected to managed: an admin's token is given, and the
  connected user is kept: reconcile is given its login as the bot's and
  takes it over when its full name carries the mark (`forgejo.md`, 1.3, step 2), and
  refuses it `Unmarked` otherwise. The identity never changes under a
  forge's writer (../client.md, section 7.1).

## 4. The crate, and its host

### 4.1 The crate

- **Step code,** under the Rust subset (../../../foundation/programming-model.md, section
  10): the access's types and checks, Forgejo's plans, GitHub's plans,
  the items' decoders and the host component. Its plans depend on
  skein-lib, skein-json and `skein-forge`, which holds the setup's
  documents (../forgejo.md, section 1; ../github.md, section 11). The host
  component adds io's vocabulary, `skein-link`, skein-http's client and
  skein-tls's client and signing, as the forge component does
  (../connection.md, section 2).
- **The owner checks the setup's values and limits at its start,** for
  every command it runs. The crate gives the checks. A refusal names a
  `Place`, the crate's name for each given value and each limit (an org
  or a repository by its index), which the owner maps to its own path
  for the person; never a value's content. Two limits out of order name
  both places, in the order `forgejo.md`, 1.4 writes them (`backoff <= backoff_max <
  deadline`).

### 4.2 The one-shot host

The one-shot host is the service's, for a command: ordinary Rust on
skein-http over skein-io, running one plan for one forge, driven by
skein-shell's `drive` (../../runtime/shell.md, section 11). The plan says; the host
runs and hands back the outcome. Its duties:

- **Run a `Send`:** encode the request with `skein-forge`'s encoder, add
  the `Host` (`skein-forge`'s `host`, ../forgejo.md, section 1.1), the
  header of its `auth` and the owner's user agent, make one exchange on
  a connection closed after it, and decode the answer into a `Reply`
  (`forgejo.md`, 1.2).
- **Keep and give the items.** It decides where each lives, and keeps it
  private and whole: written beside its place and renamed over it. A
  `Read` is answered with the value, or none.
- **Draw** `n` bytes from its own source of randomness.
- **Wait for the forge** as `forgejo.md`, 1.4 says: the waiting and its backoff are
  the host's, never the machine's. When `deadline` passes, end the run:
  before the first answer, naming the forge and the deadline; after it,
  answering the request in flight `Down`.
- **Hold the forge's lock** across a run, and say the outcome in the
  service's way: its exit status, its words.

### 4.3 The host component

The host component runs the plans inside a running service: a component
of the owner's protocol layer, beside the forge component and apart from
it, in skein's component shape. It holds the admin's token and the App's
key only while a plan that asked for them runs, and drops them at its
end.

- **Construction:** `Host::new(limits, settings, crypto)` checks both
  (4.1) and keeps `crypto`, the owner's `CryptoRng`, for the plans'
  draws. The settings name each forge, its number, kind, API address and
  transport as ../connection.md's do, and its given values: Forgejo's
  of `forgejo.md`, 1.1 but the repositories, and GitHub's of `github.md`, 1.1 but the state; and
  the owner's user agent. Each forge's `reachable` is at most the host's
  `inner`, the most elements a listing keeps, or the host is unusable: a
  picker page must fit its decoder.
- **Entry points:** `down` takes one request of the owner, `up` one of
  io's events, `fire` one expired deadline, and `next_deadline` says the
  earliest; each emits at most 4 events up and 4 requests to io. The
  owner gives io its own tokens for the host's connections, keeping the
  host's token in each, as for the forge component.

| Down | Up |
|---|---|
| `Request::Run { run, plan }` | `Event::Need { run, need }`: a `Read` or a `Write` |
| `Request::Answer { run, answer }`: `Read(value)` or `Written` | `Event::Done { run, outcome }` |
| `Request::Add(forge)` | `Event::Stop { run, failure }` |
| `Request::Remove { forge }` | `Event::Said { run, words }` |
| `Request::Close` | `Event::Closed` |

- **A run** is the owner's number for one plan:
  `Reconcile { forge, adopted }`, `Adopt { forge, repository }`,
  `Create { forge, org, repository, files }` (`forgejo.md`, 1.9),
  `Copy { forge, org, repository, from, files }` (`forgejo.md`, 1.10), `Check { forge }`, `TakeOver { forge, admin_token }`,
  `Manifest { forge, target }`, `Convert { forge, code, state }`,
  `Install { forge }`, `Accept { forge, installation, state? }`,
  `Reconcile { forge }` (GitHub's) or `Paste { forge, app }`, each of
  the forge's kind, or `Reachable { forge, installation, page }` (`forgejo.md`, 1.8, `github.md`, 1.9
  ). A forge has at most one writing run at once and one
  `Reachable` beside it: another of
  either stops at once, `Busy`, and so does one past `runs`, which
  counts both. A `Reconcile`'s `adopted` is checked by the owner first, by
  `given::check_adopted`, a `Create`'s by `given::check_create`, and a
  `Copy`'s by `given::check_copy`: a plan that fails its check is the
  owner's bug, which the host asserts.
- **It does the needs** of `forgejo.md`, 1.2 but the items: a `Send` on a connection of
  its own to the forge, opened for it and closed after its answer,
  waiting for the forge as `forgejo.md`, 1.4 says on its own deadlines; a `Draw` from
  that `CryptoRng`; the states of 5.7. A `Read` or a `Write` goes up, and
  the run waits for its answer. One answer per need: an answer for a run
  with no need waiting is ignored.
- **Each run ends once,** `Done` or `Stop`, within `deadline`, or a
  `Copy` within `copy_deadline`. `Close`
  ends every run unheard and the host says `Closed` when io has settled
  every socket.
- **A forge's words:** a `Migrate` answered with no 2xx has its body's
  JSON `message` read by the dialect's error decoder (`error_decoder`),
  cut to `string_bytes` at a UTF-8 boundary, and said at most once,
  `Event::Said`, before the `Stop` it leads to. A body that holds none
  says nothing, and the status goes to the plan either way. The owner
  holds the words for the run's stop. No other request's body is read
  on a refusal, and a `401`'s is not, since Forgejo's may echo the token
  (../forgejo.md, section 1).
- **A forge added or removed while it runs,** so that a service sets
  up the forges it makes and removes without a restart. `Add` hands one
  forge's settings, as construction names them, checked as
  construction checks each forge: a number the host holds, a forge past
  its `forges`, or one a check refuses is ignored, since the owner
  checks its settings first. `Remove` drops one forge and forgets its
  runs and callbacks: each of its live runs ends at once, with the end
  it held or else the `Stop`, `Down`, of its halt; its connections are
  aborted, a connection still settling
  after its run's end among them, an exchange in flight unheard; and
  its states (`github.md`, 1.7) are forgotten, so that no plan takes a callback to
  it. A run for a forge the host does not hold stops at once,
  `Unexpected`. Another forge's runs go on.

### 4.4 The items

An item is a value a plan reads or writes, named by what it is. The
owner keeps each where it chooses, durably and privately, and hands back
the bytes it was given, unchanged. It keeps each written item before it
answers `Written`, and acts on a plan's end only after: so a browser is
answered, and a connect sent, only on what a restart would find.

| Item | Bytes |
|---|---|
| `AdminToken`, `ApiToken`, `GitToken`, `WebhookSecret` | the token's or the secret's text, at most `token_bytes` |
| `User` | the identity's forge user id: 8 bytes, little-endian, never 0 |
| `AppId` | the App's id: 8 bytes, little-endian, never 0 |
| `AppMade` | the id of the App a manifest made (`github.md`, 1.2): 8 bytes, little-endian, never 0. `Convert` alone writes it, and no plan reads it |
| `AppSlug` | the App's slug's text, at most `name_bytes` |
| `AppKey` | the App's key, PKCS#1 `RSAPrivateKey` DER, at most `key_bytes` |
| `Accepted` | the accepted installations' ids, 8 bytes each, little-endian, ascending and distinct, at most `installations`; empty is none |
| `Pasted` | a pasted App key's PEM, read once, never written (`github.md`, 1.8) |
| `Copying` | the mark that a `Copy`'s `Migrate` was sent (`forgejo.md`, 1.10): the one byte `1`. One per `Copy` the owner runs, kept with its record of that copy, never by the forge alone |

- **The crate decodes each** for the owner, which builds the access
  from them (../connection.md, section 3.5): one total function per
  item, from its bytes to its value, none for bytes that break the
  table.
- **`AppMade` is a mark for the owner to compare:** equal to `AppId`, the
  App kept is the one a manifest made; the kit compares nothing and decides
  nothing from it. The owner keeps it with `AppId`.
- **An item's bytes are a stored format:** a change to them is a new
  item, never new bytes for an old one. The owner's own records, its
  forge's kind and address among them, are its own.

### 4.5 The host's limits

The host checks limits and settings before it starts, for Forgejo and
GitHub alike: the given plans must fit their decoder, page and transport
bounds, and every derived capacity must be representable. The settings
change only by `Add` and `Remove` (4.3): an owner builds the `Host`
once at start, from the forges it holds then, and adds and removes
forges as it makes and drops them. Its configuration compatibility
check is `supports`: a forge must support the requested plan.

- **One slot holds a run and its connection.** A run has at most one
  connection open, and a run that has ended keeps its slot until its
  connection has settled. A `Run` with no free slot stops at once,
  `Busy`, before its plan starts, as one beside a live run of the same
  kind on its forge does (4.3). An admitted run always finds room for
  its connection: the host never waits for one or refuses one mid-run.
- **The settling is bounded.** A connection closes once its reply is
  taken, over a link (../../net/link.md): TLS's `close_notify` goes first, and the
  link aborts what hasn't reached the socket within io's `close_timeout`.
  io then aborts a graceful close the peer has not finished within the
  same time; a run past its deadline (`Times.deadline`) aborts
  its connection outright. An ended run's slot comes back within
  twice `close_timeout` of its end, so a `Busy` for want of a slot is always
  transient.

## 5. Verification

The tiers and budgets are ../../../foundation/testing-strategy.md's and skein's `../../testing/README.md`.

- **Step tests,** each plan driven by answers: each step read, then
  written only where the forge differs; the marker's refusal, and the
  mark found only whole; a site admin bot refused, and a user both
  unmarked and a site admin refused `Unmarked`; a revoked token made
  again, and one answered `403`; an empty item taken as none; the hooks
  patched; an event out of order; the hook's URL for loopback addresses
  alone; the limits' relation at its bound and one past; the admin's
  search, a full page with `more`, a short last page, and a non-admin
  refused; connected's listing on the API token, its rows lacking `push`
  dropped and `more` read before the drop; the person's read answering an
  id, none for a `404` and nothing with no login given; `AppMade` before a
  conversion's other writes, and none from a paste; the App's account in
  each `Done` that read `GET /app`; an installation given to a
  Forgejo, or none to GitHub, stopped sending nothing. The hook's secret
  asked once, drawn only when missing, and neither asked nor drawn for a
  listener off loopback, since a plan alone shows a draw, a host's `Need`
  being `Read` or `Write` (4.3); a plan's sends and replies with no secret
  in their `Debug`. For GitHub: the
  manifest's contents, field by field, for each target; the conversion's
  writes before `Done`; a key of another form refused; an id other than
  the kept one refused; each permission or event lacking named; a stray
  installation uninstalled and a young one left; a picker's installation
  not accepted, and a mint's `404`, each `Missing`, and the minted token
  in no item, outcome or failure; each item's decoder at its bounds. The
  host's forges added and removed (4.3): an added forge serving a run;
  an add past `forges`, of a number held, or of settings a check
  refuses, ignored; a removal ending its runs with their `Stop` and
  aborting their connections, one still settling and one io has not
  yet named among them; a run for a forge not held stopped at once. The
  documents are tested with `skein-forge` (../README.md, section 9;
  ../github.md, section 14).
- **The setup world** (../../../foundation/testing-strategy.md, 2.4), in skein's
  `tests/forge/setup`, plays the host component over a store of items
  and the fake forge's setup routes and GitHub layer (../../testing/fakes/forge.md,
  section 4), through HTTP bytes. It checks:
  - an empty forge converged: the bot, both tokens with their scopes,
    answering as the bot, and each adopted repository's collaborator and
    hook;
  - a token kept untouched when it answers as the bot, made again when it
    is another user's or was never made, and one found on a listing's
    second page replaced;
  - a second run writing to the forge only each hook, deleted and made
    again;
  - a deleted bot, a revoked token, a removed collaborator and a deleted
    hook, each made again;
  - a collaborator or a repository's hook on a listing's second page
    neither repeated nor passed over; two hooks with the listener's URL,
    both deleted and one made, and a hook of another URL left alone;
  - a hook left with another secret made again with the kept one, on a
    fake that, as Forgejo, keeps a hook's secret through a patch; a
    refused deletion or making stopping the run, and the next run, no
    longer refused, leaving one hook with the kept secret;
  - an owned forge's missing org made for its owner; the secret drawn
    when missing and kept when present; a person's account under the
    bot's login refused; another service's bot refused as unmarked; two
    services keeping separate access on one forge; a listener off
    loopback making no hook and no secret;
  - connected's tokens as one user, and as two, refused naming which;
    an admin's token taken over and its older tokens deleted;
  - the repositories the admin can see, listed page by page to the end,
    another owner's private one among them, while a writing run goes on
    for the same forge; a connected user's, those it can write to alone;
    and an installation's, page by page, one mint each;
  - on GitHub: the conversion's answer kept before `Done`; a state used
    twice, or unknown, refused; an installation accepted, and a stray
    one uninstalled; the hook's address set; a pasted App lacking a
    permission named;
  - no secret in a failure's `Debug` or in the trace.
- **The scenario's ceiling,** against the counting allocator (skein-heap):
  the setup world meters the host alone, the fake and the store outside it
  (skein-heap's `Measured`, as the machine world does), and checks at
  every iteration that its measured peak is within the ceiling its scenario
  names, with headroom (../../runtime/shell.md, 11): over plaintext and TLS, Forgejo,
  GitHub and both, `runs` runs live with their connections and `states`
  states held. The pure plans keep their edge and derived-capacity checks.
- **Fuzzy:** random initial forges, their hooks of the listener's URL
  holding the kept secret or another, converge, a run that a refused
  hook deletion or making stops converging at the next; and a second run
  writes only each hook, deleted and made again
  (../../../foundation/testing-strategy.md, section 8).
- **Against a real forge,** the conformance records `forgejo.md`, section 2's and
  10's facts, and a service tests its host on the throwaway Forgejo
  (../../testing/fakes/forge.md, sections 7 and 6). No suite calls GitHub itself.
- **The App never changes live** (../../testing/fakes/github.md, section 2, which holds
  the rule). The probe App's fixture installations are accepted by no
  state; GitHub's reconcile (`github.md`, 1.6) with `uninstall_strays` and
  `repair_hook` true would uninstall them and set the App's hook. No plan of `github.md`, section 1 runs against github.com: a live run
  never sends `DELETE /app/installations/<id>` or
  `PATCH /app/hook/config` and changes nothing of the App; strays, the
  uninstall and the hook's set are shown on the fake alone; and a
  repository a run adds to an installation or removes from it is put
  back at the run's end.

## 6. Open questions

- **Interpreting the identity answers** (`forgejo.md`, 1.3, and `github.md`, 1.2, 1.6 and 1.8, and `README.md`, 4.4).
  Default: the caller decides what a person's id or an App's account
  means, and compares `AppMade` with `AppId` if it needs that evidence.
  The kit keeps no person link and makes no authority decision from
  these answers.
- **Owned Forgejo's preparation:** the settings file, migrations, the
  runner's registration and upgrades' dumps. Default: it stays the
  service's, since one service runs a forge. It moves into the Forgejo
  plan when a second does.
- **Reconcile over TLS,** for a managed forge on another host. Default:
  plaintext to loopback alone, and the owner refuses a forge on TLS.
  The host component speaks TLS to either kind, so the default is the
  owner's, not the host's.
- **A hook left at an old listener's address,** after the listener
  moves. Default: left alone; the person deletes it.
- **An App's hook made inactive by hand.** `PATCH /app/hook/config` sets
  the URL, the content type and the secret, never whether it is active,
  and an inactive hook logs no delivery. Whether the App's read shows it
  stays `unchecked` on github.com (`inactive-hook-visible`,
  ../github.md, section 13), since a live run changes nothing of the
  App (section 5); an App made to be changed would find it. Default: the
  client's polling keeps up, and a stopped log is the person's to see.
- **An `Add` a check refuses** is ignored, and the owner hears nothing
  (4.3). Default: so, the owner having checked first; whether the
  crate's check of one forge is the owner's to call, or the host answers
  the `Add`.
- **A path `Adopt` is given** (`forgejo.md`, 1.5) has no check of `given`'s, as a
  `Create`'s and a `Copy`'s names have (4.3), and no `Place`: the
  request's encoding bounds it. Default: so, the owner holding what a
  person typed to its own bounds first.
- **A fine-grained token as a connected GitHub.** Default: later; a
  connected GitHub is an App the person made (`github.md`, 1.8).
- **What a refused copy leaves.** Forgejo keeps an empty repository at
  the name of a source it could not clone (`forgejo.md`, section 2), and only this
  copy, run again, takes it away (`forgejo.md`, 1.10, step 3). Default: so; the plan
  could delete it at the refusal, for an owner that runs no such copy.
- **Forgejo's own bound on a migration,** `[git.timeout] MIGRATE`. temper's
  owned `app.ini` sets none of `[git.timeout]`, so Forgejo's default
  holds, which may end a large copy before `copy_deadline` does.
  Default: Forgejo's, and such a copy stops with Forgejo's words; a
  deployment that needs more sets its Forgejo.
- **The deployment's public address** as the App's hook, for a
  deployment GitHub can reach. Default: later; every App's hook is at no
  listener (`github.md`, 1.5).
