# GitHub, and forges connected from the client

Draft, 2026-10-20. temper on github.com, as easy to start as GitHub
allows; and, on the way, every forge connected from temper's client,
its secrets in jig's secrets' store, and repositories added to projects
whenever a person likes. It records a design session with the user on
2026-10-20 and the probes run on github.com that day (section 11). It
crosses skein (the dialect, the component, the setup's plan, the fakes,
the conformance), jig (credentials made at runtime) and temper
(configuration, the connector, the setup, the client) on purpose; an
owner splits it. It builds on skein's forge kit as the forge-kit plan
leaves it (skein's `forge.md`, `forge-connection.md`, `forge-client.md`,
`forge-setup.md`, `fake-forge.md`), and fills the seams those documents
keep for GitHub (skein's `forge.md`, section 10; `forge-setup.md`,
section 5; `forge-connection.md`, 5.1). Nothing here is adopted. Part A
is what both forges share, part B is GitHub, part C moves Forgejo onto
part A. Where each piece goes is section 15, an order section 16, what
is open section 19.

## 1. In one page

- **Connect, then add repositories, any time.** Configuration names a
  forge and its addresses, never a credential and never a repository. A
  person connects the forge from temper's client, then adds
  repositories to a project from a list of those temper can reach, now
  or later. Nothing is imported in bulk at configuration (section 2).
- **Secrets in the store.** A forge's credentials are made from the
  client, or by temper while it connects, and kept in jig's secrets'
  store under temper's tag (jig's `protocol/store.md`, section 9),
  never in a file temper reads at start: one way, in every forge and
  mode. An owned Forgejo's preparation, which runs before the engine,
  hands its admin's token over once, by a file the engine takes in and
  removes. This reverses credentials.md's "no temper secret in jig's
  secrets' store" (section 3).
- **A forge connected while temper runs.** skein's component takes a
  forge's credential at runtime, and again when it is replaced or
  renewed; git identities take new generations as they change
  (section 4).
- **temper is a GitHub App,** one per deployment, never the person.
  GitHub refuses an author's approval of their own pull request, and
  temper tells its own writes from others' by its user's id, so temper
  must be someone else (section 5).
- **Connecting GitHub is two clicks** (section 6): "Connect GitHub" in
  the client takes the tab to a page on temper's listener that posts
  the App's manifest to GitHub as it loads; the person clicks "Create",
  GitHub sends the browser back with a code, temper exchanges it for
  the App's key and keeps it; the person picks an account and
  repositories on the install page, and GitHub's setup redirect brings
  the tab back to the page that started it. No organization is asked:
  the App is the person's, and GitHub's install page is the menu of
  accounts. No token is pasted. GitHub has no API that creates or installs an
  App, so nothing does better. Probed (section 11).
- **No public address.** The App's webhook points at a host that cannot
  resolve; GitHub still records each delivery, and temper reads that
  record, the App's delivery log, every few seconds as its hint feed.
  Conditional reads of what temper waits on cost nothing when nothing
  changed. A public address, when a deployment has one, adds pushed
  deliveries; nothing depends on it (section 7).
- **Credentials that expire.** An installation's token lasts an hour:
  the component renews it, and the git identity takes a new generation
  (section 8).
- **Landing on its own, wherever the rules allow.** A merge at the head
  temper's decision saw; GitHub's merge queue, entered at that head,
  where the branch requires it. A rule temper's identity cannot satisfy
  is named at adoption, with the one setting that would let temper land;
  temper never changes a repository's rules (sections 2.3 and 10).
- **Forgejo on the same footing** (part C): connected and managed
  connected from the client; reconcile's host moves into the engine;
  access granted per repository at adoption in place of setup's
  configured orgs; owned keeps its preparation, its files being
  Forgejo's own.
- **Tested on a GitHub-shaped fake,** skein's fake forge with a GitHub
  protocol layer, and checked live by a conformance on github.com that
  records the facts the dialect relies on, outside every suite
  (section 17).

# Part A: connecting forges

## 2. The person's way

### 2.1 Connecting a forge

- **Configuration names the forge, not its access.** Each forge's kind
  and addresses: for Forgejo its `api`, `transport` and `git`; for
  GitHub, `kind: "github"`, whose addresses default to github.com's and
  are given only to point at a fake. No token, user id, webhook secret
  or repository.
- **A forge not yet connected** is configured, shown and idle: the
  engine starts and serves; the client's deployment page says "Not
  connected" with its Connect action, and the adoption form offers none
  of its repositories. Nothing on it runs. This is how an LLM provider
  with no account is shown (credentials.md, section 3).
- **Who connects:** a person whose role holds the deployment's
  connectors (jig's `client/domain/pages.md`, section 7): in standalone,
  the deployment's owner.
- **How,** per forge and mode:
  - GitHub: "Connect GitHub", the manifest flow (section 6), or an App
    the person made, its id and key pasted (6.5);
  - Forgejo connected: the bot's two tokens pasted (section 12.1);
  - Forgejo managed: a site admin's token pasted, temper making its bot
    (section 12.2);
  - Forgejo owned: connected by its preparation, at start (section 13).
- **What connecting ends with** is skein's access (skein's
  `forge-setup.md`, section 2), kept in the store: who temper is there,
  its credentials, and the webhook secret. The component then opens the
  forge and the client offers its repositories.
- **Replaced from the client:** a new token, a new App key. The new
  credential is kept, handed to the component, and the old one erased.
  Disconnecting is later (section 18).
- **`temper ctl`** offers the same, as tools (`../client/tools.md`), so
  that QA's agents and tests connect a forge as a person would.

### 2.2 Adding repositories, any time

- **No repository is named at configuration or at connecting.** A
  project's owner adds repositories to the project from the client,
  whenever they like, through the adoption form (`../client/README.md`,
  4.4; `../client/forge.md`, 5.1).
- **The form becomes a picker.** It lists, per connected forge, the
  repositories temper can reach and the project does not hold, and
  takes an owner and a name for one not listed. What temper can reach:
  - GitHub: the repositories of the App's installations;
  - Forgejo connected: those the bot can write;
  - Forgejo managed: every repository of the forge, since temper grants
    itself access at adoption (section 12.2);
  - any forge: any repository the credential can read, as context.
- **A repository temper cannot reach** is shown with what would make it
  reachable, in words: on GitHub, granting it to temper's App on
  GitHub's installation page, linked, or installing the App on another
  account through temper's own link (section 6.3); on a connected
  Forgejo, making the bot, named by its login, a collaborator. Once the
  forge says so, the repository is listed, without a restart.
- **The rest of the form is as it is:** role, home, prefix, landing
  branch, CI or checks, and what temper found there before the person
  confirms (`../client/forge.md`, 5.1), with GitHub's rules in words
  (section 10.3).
- **Forgejo too.** setup's configured `orgs` go: access is per
  repository, at adoption (section 12.2).

### 2.3 Landing on its own

- **The project's landing rules decide** whether temper lands
  unattended (`../protocol/configuration.md`, section 3, `landing`;
  `../domain/forge.md`, 8.7). Landing waits for people only where those
  rules ask for their approval.
- **temper lands the forge's way:** a merge at the head its decision saw;
  on GitHub, where the branch requires its merge queue, the pull request
  enters the queue at that head (section 10.2).
- **A rule of the forge's that temper's identity cannot satisfy** is
  found at adoption, and shown with the one setting that would let
  temper land, for the person to choose: on GitHub, a required approval
  (the App cannot approve its own pull request) is met by a person's
  approval, or by the App on the ruleset's bypass list, which adoption
  reads as the ruleset's `current_user_can_bypass` (section 11, P8 and
  P11); required signed commits, by temper's commits made through the
  API (section 18); a code owner's review, by a person. Until then, its
  landings wait for a person, which the change's panel says
  (`../client/forge.md`, 2.5). temper never changes a repository's
  rules.
- **An approval that is not the person's** is possible later: an App's
  approval counts toward a ruleset's required approvals (section 11,
  P8), so a second App of temper's, the reviewer, could approve what an
  agent's review passed (section 18).

## 3. Secrets in the store

- **One way, in every forge and mode:** each credential the engine uses
  lives in the store, and nowhere else, and enters it through a
  connect: pasted in the client, made by the engine's own exchange or
  reconcile, or, on an owned Forgejo alone, handed over once by its
  preparation, which runs before the engine and cannot write the store
  (section 13).
- **Where:** jig's secrets' store, a private skein-kv store of the
  engine's (jig's `protocol/store.md`, section 9), which keeps "an
  application's own records, under its tag: its connectors'
  credentials, which it decodes itself". temper's records there:
  - GitHub: the App's id, slug and private key, and its webhook secret;
    the installation tokens are never kept, but minted (section 8);
  - Forgejo: the bot's API token, its git token, its id and the webhook
    secret; managed and owned, a site admin's token too (12.2, section
    13);
  - each with the forge user's id it stands for, checked at each start.
- **Not there, being no credential of the engine's:** what other
  programs read from files, an owned Forgejo's own secrets (`app.ini`,
  `state`) and its runner's, and the person's first password on it
  (section 13); a worker's secret and its digest, jig's files, which
  open the link before any store is reached; LLM accounts, already in
  the store as jig's.
- **jig's rules hold** (jig's `protocol/store.md`, section 9): written
  by components alone, through the protocol layer's routing; kept, and
  durable, before used; replaced stating the digest it replaces; erased
  when replaced or removed; a failed commit stops the engine. No root,
  record, fact, log or request's answer holds a value.
- **A pasted token** reaches the engine in a request the client sends
  once and never keeps, kept in the store before the core sees the
  request, as an LLM key is (jig's `domain/accounts.md`, section 5;
  jig's `protocol/client.md`, section 5). This is jig's "request
  carrying a value", not yet built (jig's `STATUS.md`): Forgejo's
  connect and an App pasted (6.5) wait on it; the manifest flow does
  not, since GitHub hands the App's key to the engine's own exchange
  (section 6).
- **credentials.md changes:** its table's rows for git identities, the
  forges' API tokens and webhook secrets name the store; its rule "no
  temper secret in jig's secrets' store" goes; "least power" is
  restated for the admin's token (section 12.2).
- **A person's backups** of the store's directories now hold the forges'
  credentials, as they already hold LLM accounts'.

## 4. A forge connected while temper runs

- **skein's component takes a forge's credential at runtime.** Today
  the owner hands each forge's credential and user id at start
  (skein's `forge-connection.md`, 3.1). A forge may now start with
  none: its calls are refused as not connected, at once, not waited on;
  the owner hands the credential later, as a message, and the forge's
  start (the identity read, the page cap read; 3.3) runs then. A
  replaced credential is the same message; the calls in flight end as
  they would at a revocation. Renewal (5.1) is the component's own
  replacement, from the dialect's plan.
- **The identity it must stand for** is the one recorded at connect,
  kept beside the credential, in place of `forges[].user`. The start's
  check is as fail-stop as now (credentials.md, section 5): a
  credential that answers as someone else stops that forge, named, and
  the client says so; it no longer stops the engine, since other forges
  run on.
- **temper's routing** reads the forge's records from the store at each
  start and hands them to the component, and hands it each new one a
  connect or a replacement keeps, as jig's routing hands an LLM
  account's record to skein's driver (jig's `protocol/llm.md`, section
  5).
- **Git identities.** Configuration keeps each identity's number and
  author; its token comes from the store (Forgejo) or from the
  installations' tokens (GitHub, section 8). The engine announces
  `Event::Credential` with a new generation whenever the value changes,
  at runtime, where today it announces generation 1 at start alone
  (jig's `domain/engine.md`, 12.1; credentials.md, section 4). The
  worker keeps the two newest generations, so an operation in flight
  across a change finishes on the value it began with.
- **The forge writer at runtime.** skein's forge client takes its
  writers, one per forge, at construction (skein's `forge-client.md`,
  7.1), and refuses every effect on a forge with none, `Forbidden`. It
  now takes a forge's writer later too, once, when the forge connects:
  temper's routing tells the root, which hands it on. At a start the
  writers are the store's, as configuration's are today. A forge's
  writer never changes: a replacement that answers as another user is
  refused, since the keys and echoes already on the forge are the old
  user's. Connecting a forge as someone else is a disconnect and a new
  connection, later (section 18).
- **The connector** knows a forge as connected or not: an adoption on
  one not connected is refused as such, and the client offers Connect.

# Part B: GitHub

## 5. temper as a GitHub App

- **An App, one per deployment, made by the manifest flow** (section 6),
  or by the person (6.5), owned by the account that connects it. A
  shared public "temper" App cannot be: its private key would have to
  live with every deployment.
- **Never the person.** GitHub refuses an author's approval of their own
  pull request, and temper tells its writes and their echoes from
  others' by its user's id (`../domain/forge.md`, 5.2): acting as the
  person would turn the person's own comments into echoes. The person's
  token is never asked for.
- **Public, installed only through temper's link.** A private App can
  be installed only on the account that owns it; to work in an org the
  person does not own the App under, or in their own account and an
  org, the App must be public. Each installation is accepted only when
  it comes back through temper's link with the `state` temper gave it
  (6.3); any other is uninstalled (`DELETE /app/installations/{id}`).
- **One forge, many accounts.** The App has an installation per
  account, user or org, each with its own token. The forge is
  github.com; installations are its runtime state, found and followed
  through the delivery log (section 7), never configured. skein's
  component holds one credential per forge today (skein's
  `forge-connection.md`, section 5): for GitHub that credential is the
  App, and the component mints, renews and chooses an installation's
  token by the repository a call names. A forge per account would have
  broken forks across accounts, and made adding an account a
  configuration change.
- **Its identity** is the App's bot user, `<slug>[bot]`, with an id of
  its own: the forge writer (`Writer { forge, author }`) and the user
  echoes are told by. At connect, before any installation, its id is
  read with no credential, `GET /users/<slug>%5Bbot%5D`; at each start,
  GraphQL's `viewer { login databaseId }` on an installation's token
  checks that the credential answers as it, since REST's `GET /user`
  refuses an installation's token (section 11, P2).
- **Its permissions,** asked in the manifest and nothing more: contents,
  pull requests and issues to write; checks, statuses, actions,
  administration and metadata to read (administration reads the rules
  adoption shows); merge queues to write. Never workflows (section 18),
  never administration to write. The engine holds the App's key, so the
  App's permissions are temper's least power on GitHub.
- **Its events:** push, pull request and its reviews and review
  comments, issue comments, issues, check runs and suites, statuses,
  workflow jobs and runs, branch creation and deletion, merge groups;
  installations and their repositories come to every App.

## 6. Connecting GitHub: the manifest flow

### 6.1 The two clicks

1. **Connect.** The person chooses "Connect GitHub" in the client; the
   engine answers an address on the clients' listener, with a one-time
   `state` tied to that request and the page's address to come back
   to, and the browser's tab goes there, as for a sign-in (the terminal
   prints it and opens no browser; `../client/README.md`, 6.5).
2. **The manifest.** That address serves a page whose form posts the
   manifest to `https://github.com/settings/apps/new?state=<state>` as
   it loads: the App is always the person's own, and public, so it
   installs on any account they may install it on (with the user,
   2026-10-23). The person clicks "Create GitHub App" on GitHub.
3. **The exchange.** GitHub sends the browser to the manifest's
   `redirect_url` on the clients' listener, with a `code` and the
   `state`. The engine checks the state, exchanges the code with `POST
   /app-manifests/{code}/conversions` within the hour GitHub allows, and
   keeps the App's id, slug, key and webhook secret in the store before
   it answers the browser. The client and secret of the App's OAuth are
   not kept (section 18).
4. **The install.** The engine sends the browser to
   `https://github.com/apps/<slug>/installations/new?state=<state>`.
   The person picks an account and its repositories, all or selected,
   and clicks Install. GitHub sends the browser to the manifest's
   `setup_url`, on the clients' listener, with the installation's id and
   the state; temper accepts the installation and sends the browser
   back to the page that started the connect, which lists its
   repositories to start in. A member of an organization who isn't its
   owner gets a request, not an installation: GitHub comes back with
   `setup_action=request` and none, and temper's page says why it
   can't go on, and to install it on their own account or an
   organization they own.

### 6.2 The manifest

- `name` `temper-<random>`, under GitHub's bound on names; `url` the
  deployment's (or temper's project page); `public` true (section 5).
- `redirect_url` and `setup_url` on the clients' listener, with
  `setup_on_update` true, so a person changing the repositories on
  GitHub comes back to temper. GitHub takes `http://127.0.0.1:<port>`
  for both (section 11).
- `hook_attributes`: `url` `https://<slug>.invalid/` (RFC 2606's
  `.invalid` never resolves), `active` true, so deliveries are recorded
  (section 7); or the deployment's public address, when it has one.
- `default_permissions` and `default_events`, section 5.
- **Built by skein's setup plan** for GitHub (skein's `forge-setup.md`,
  section 5), a pure function of the given values, and served by
  temper's server on the clients' listener: the page, the redirect and
  the setup route are temper's, its owner's.

### 6.3 Another account later

- **"Install on another account"** in the client gives a fresh state
  and the install link, so the person adds their org, or another, at
  any time; the same `setup_url` brings the tab back.
- **Repositories granted later** on GitHub's installation page reach
  temper as an `installation_repositories` delivery (section 7), and
  appear in the picker.

### 6.4 Reconcile, on GitHub

At each start and after a connect, temper checks what it can, and
repairs nothing on GitHub but its own hook's address:

- the App's permissions and events against what this temper needs; a
  newer temper asking more names the setting, and GitHub asks each
  installation's owner to accept it;
- the hook's address against the deployment's (`PATCH /app/hook/config`,
  with the App's JWT);
- the installations against those accepted (section 5);
- the App's key still taken: a key revoked on GitHub stops the forge,
  and the client offers to replace it.

### 6.5 Managed and connected, on GitHub

skein's modes (skein's `forge-setup.md`, section 3) on GitHub, where no
token makes an App:

- **Managed is the manifest flow:** temper drafts its App, and the
  person only clicks (6.1). It is the way the client offers first.
- **Connected is an App the person made** on GitHub's settings, by hand
  or by their own tooling: its id and private key pasted in the client,
  a request carrying values (section 3). Reconcile checks it as 6.4
  does, and names each permission or event missing; it sets the hook's
  address itself, the one repair the App's key allows. The person
  installs it through temper's link (6.3), as for managed.
- **A fine-grained token** as connected is later (section 18): the
  person's token would make temper the person (section 5), so it would
  have to be a second account's, a machine user.
- **Owned:** none; github.com is GitHub's.

## 7. Hearing changes without a public address

- **GitHub pushes, never streams.** Its API has no long poll, stream or
  subscription. The Events API is polled, with a latency GitHub
  documents as 30 seconds to 6 hours. `gh webhook forward` streams, but
  for testing alone ("not supported for use in production"), one
  forwarder per repository or org, and not for an App's hook. A webhook
  needs an address GitHub can reach, with TLS.
- **The delivery log is the hint feed.** The App's hook points nowhere
  (6.2), so every delivery fails, and GitHub records each one: `GET
  /app/hook/deliveries`, with the App's JWT, lists them newest first,
  each with its `guid`, `event`, `action`, `installation_id`,
  `repository_id`, `status` and `delivered_at`; `GET
  /app/hook/deliveries/{id}` gives one whole, with its payload. One
  conditional read every `log_poll` covers every repository of every
  installation, and answers `304` when nothing was delivered. A change
  is in the log about two seconds after it is made (section 11, P7).
- **A hint from each delivery,** as the listener makes one (skein's
  `forge.md`, section 7): the event, the installation and the repository
  from the list; where the hint names more (a branch, a pull request, a
  commit), the delivery's payload, read whole within its bound and
  decoded by the same decoders as a pushed delivery's. Events whose
  hint the list alone gives, or which no wait needs, are not read.
- **An installation's events are always read whole:** `installation`
  and `installation_repositories` come with no installation or
  repository in the list (section 11, P7), and say which account was
  installed, and which repositories were granted or withdrawn. They
  change what the component may reach, and what the picker lists.
- **A delivery's id** is a 19-digit number, decoded exactly as a `u64`
  (a JSON decoder going through a float would lose it).
- **The component's puller** sits beside its listener (skein's
  `forge-connection.md`, section 6): it keeps the newest `guid` seen,
  reads back to it at each poll (paging by cursor when more arrived),
  and at start reads back within `log_catch_up`, so deliveries made
  while temper was down still hint. Its reads run on the App's JWT,
  apart from the installations' rates.
- **What temper waits on, read conditionally.** A fresh read with the
  validator of the last answer is answered `304` when nothing changed,
  which GitHub does not count against the rate (section 11, P6), so the
  client's keep can read a waited-on pull request's CI every few
  seconds at no cost. This is skein's validators addition (skein's
  `forge.md`, section 10).
- **Polling is unchanged:** the keep's paces and slow pass are the
  backstop, as on Forgejo (skein's `forge-client.md`, section 4).
- **A public address,** later and optional: the person's proxy, or
  skein's TLS server side, named in configuration; reconcile sets the
  hook to it (6.4); pushed deliveries, checked by `X-Hub-Signature-256`,
  and the log's are told apart by `guid`, and each hints once.
- **QA does not need it.** QA's boxes run an owned Forgejo, whose
  webhooks are on loopback (`../protocol/setup.md`, section 10).

## 8. Credentials that expire

- **The App's key** signs a JWT (RS256, through `ring`, as skein-tls has
  it), valid for minutes, for the App's own routes: its installations,
  its hook's configuration and delivery log, and minting tokens.
- **An installation's token** is minted from it (`POST
  /app/installations/{id}/access_tokens`) and expires within the hour
  (section 11, P1). skein's renewal (skein's `forge-connection.md`, 5.1)
  is built for it: the dialect's plan mints the next token `renew_before`
  its expiry; a call needing it meanwhile waits, within its deadline. A
  token may be minted narrower than the App, in permissions and
  repositories, and a write beyond it is refused (section 11, P1): the
  component mints git's with contents alone. Neither names repositories:
  a token minted for the whole installation reaches a repository granted
  after it at once, where one minted for named repositories does not
  (P12).
- **Each installation's token has its own rate,** 5,000 requests an hour
  on REST's core (P1); the App's JWT calls carry no rate headers, and
  are few (section 7).
- **Git's identity on GitHub** is one identity for the forge, whose value
  holds each accepted installation's current git token, `x-access-token`
  as the login, and the bot's author; the git component takes the token
  of the repository's account. Each renewal, and each installation
  accepted or removed, is a new generation (section 4). The author is
  the bot's noreply address, `<id>+<slug>[bot]@users.noreply.github.com`,
  so GitHub shows its commits as the App's.
- **credentials.md's open question** on short-lived git credentials
  (section 8) is answered by this.

## 9. The dialect

- **In skein,** GitHub's documents and call plans beside Forgejo's
  (skein's `forge.md`, sections 4 and 10): REST, pinned to API version
  `2026-03-10` (the `X-GitHub-Api-Version` header; requests without it
  get 2022-11-28, and a version stays supported 24 months after the next
  one), with GraphQL for what REST lacks: entering the merge queue
  (10.2), the commit a queue landed, and the bot's identity (section 5).
  temper does not use auto-merge: it decides each landing itself.
- **The pin** is the API version and the conformance's date. GitHub is a
  service, not a release a deployment runs: the conformance runs on a
  schedule, and a fact that changes is a finding (section 17).
- **The vocabulary's additions** (skein's `forge.md`, section 10):
  validators, whose `304` the budget counts at no cost; the rate's
  window from `x-ratelimit-*` on every answer, which the client's budget
  takes in place of its configured one; the secondary limits' refusals,
  a `403` or `429` with `Retry-After`, as a pause. Batched GraphQL reads
  are later.
- **Each call's GitHub shape,** the facts in section 11 and the old
  crate's quirks (`~/src/rust/old/temper/crates/temper-forge-github`,
  a catalogue only, never code to take): pull requests listed among
  issues and told apart by `pull_request`; `merged` absent from listings
  (`merged_at`); a fork's head repository null; reviews `APPROVED`,
  `CHANGES_REQUESTED`, `COMMENTED`, `DISMISSED`, `PENDING`; `mergeable`
  computed after a read, null meanwhile; pages by `Link`; pull requests
  and issues numbered together, and a new issue missing from a listing
  for a moment (section 11, P13).
- **Refusals the dialect tells apart,** by status and GitHub's message,
  since several share a status: a stale head (`409`), not mergeable yet
  or a rule unmet (`405`, P5, P8, P9), an expected head that moved
  (`422` on update, `UNPROCESSABLE` in GraphQL), beyond the token's
  reach (`403` "Resource not accessible by integration"), a push refused
  by a rule (`GH013`) or for touching workflows (P3, P10).
- **CI** is check runs and commit statuses, read apart: the combined
  status stays pending with no status at all while Actions' check runs
  end, and other Apps' suites (a Codecov, a Travis) may sit queued
  forever (section 11, P4). So CI on a head is the checks the adoption
  names: the branch's required status checks when its rules have them,
  and otherwise the repository's Actions workflows' check runs. Actions'
  jobs, with their steps, are read through the Actions API.
- **A job's log** is a redirect to Azure's blob storage, signed, needing
  no credential, valid about ten minutes, and blind to `Range`: the
  component follows it to that host alone, on TLS, sending nothing of
  the forge's, and keeps the log's tail as it streams, within the call's
  bound (section 11, P4).
- **Credentials' kinds:** the App (key, id), whose renewed kind is the
  installation's token (8). A fine-grained token is a static kind,
  later (section 18).
- **Deliveries:** `X-GitHub-Event`, `X-GitHub-Delivery`,
  `X-Hub-Signature-256`; payloads decoded keeping the hint's fields; the
  delivery log's two documents.
- **The setup's documents:** the manifest, its conversion, the
  installations and their repositories, the hook's configuration.

## 10. Adoption and landing on GitHub

### 10.1 Merging

- **At the head temper saw:** `PUT /repos/{o}/{r}/pulls/{n}/merge` with
  `sha`; another head is refused, `409`, and the change goes on as on
  Forgejo. Just after a head moves, GitHub may answer `405` "Pull
  Request is not mergeable" in its place (section 11, P5): within
  `settle` of a move, a `405` is not known yet, and read again, as
  Forgejo's not-mergeable-while-checking is (`../domain/forge.md`, 5.2).
- **CI does not hold a merge** where no rule asks it to (P5). temper's
  landing rules are its own enforcement (`../domain/forge.md`, section
  4), so nothing changes.
- **The merge method** is the first the repository allows of merge,
  squash and rebase, unless the adoption names one (section 19).
- **Updating from the base:** `PUT .../pulls/{n}/update-branch` with
  `expected_head_sha`; a head that moved is refused, `422`; the right
  one is accepted, `202`, and done later: the head moves once GitHub has
  merged, and temper hears it as any move (P5).
- **Mergeability settles after a read:** `mergeable` null is read again
  within `settle` (P4).

### 10.2 The merge queue

- **Where a branch requires GitHub's merge queue,** a direct merge is
  refused, `405` "Changes must be made through the merge queue" (section
  11, P9). temper enters the queue with GraphQL's `enqueuePullRequest`,
  giving `expectedHeadOid`, the head its decision saw: a head that moved
  is refused, and the change goes on as for a stale head. Its decision
  holds at the queue's door.
- **GitHub's queue is then that branch's landing queue**
  (`../domain/forge.md`, section 9): temper's own queue for the branch
  decides only what it enters and when. GitHub makes a merge group, a
  commit of its own on the base, runs CI on it (`merge_group`), and
  lands it.
- **The landed commit is the group's,** not the head temper saw, and
  REST reads the pull request merged with no `merge_commit_sha` (P9):
  temper reads it from GraphQL's `mergeCommit`, or from the base's tip
  once the pull request reads closed and merged.
- **What temper hears,** through the delivery log: the pull request
  `enqueued`, then `dequeued` with its reason (`MERGE`, `CI_FAILURE`,
  and GitHub's others); `merge_group` `checks_requested` and `destroyed`
  (`merged`, `dequeued`). The change's panel says "in GitHub's merge
  queue" and its position, read from the entry
  (`../client/forge.md`, 2.1).
- **Ejected,** a pull request dequeued for any reason but `MERGE` is a
  repair, as a failed CI is (`../domain/forge.md`, 8.4). GitHub ejected
  a failed group about half a minute after its CI failed (P9).
- **Merge queues exist** on org-owned public repositories, and private
  ones on GitHub Enterprise Cloud; elsewhere the branch merges directly.

### 10.3 What adoption reads, and says

- **Reach and permission:** the installation's, and the repository's
  role in it.
- **The rules on the landing branch,** as GitHub applies them: `GET
  /repos/{o}/{r}/rules/branches/{branch}`, each rule with the ruleset it
  comes from, and each ruleset's `current_user_can_bypass`, the App's
  own; never the bypass list, which the App cannot read (section 11,
  P11). A branch under rulesets reads "Branch not protected" at
  `/branches/{branch}/protection` (`404`, P11), so adoption reads both,
  rulesets and classic protection, and takes neither's absence for the
  other's. Each rule is said in words with what it means for landing
  (2.3):
  - a merge queue: temper enters it (10.2);
  - required approvals: a person approves, or the App bypasses;
  - required status checks: CI as the landing rule's;
  - code owners: a person reviews;
  - signed commits: temper's git pushes are refused (P10), so its
    landings wait (section 18);
  - linear history: a merge method that keeps it;
  - temper's prefix reserved to the App: welcome.
- **Workflows:** the App never changes `.github/workflows/`, and GitHub
  refuses a push that does (P3). Adoption says so where the repository
  has workflows; a change whose push touches them is refused, and its
  panel says why (section 18).
- **Archived** repositories adopt as context only. **Actions off:** the
  adoption names the checks that stand for CI.
- **Actions on, no workflow:** when adoption finds no workflow and no
  required checks, it says "No CI found: Actions is on, but this repository
  has no workflow and no required checks." It asks: **Land without CI**,
  or **Name the checks that stand for CI**. "Land without CI" adopts with
  CI off; naming checks makes those checks the gates in place of CI. This
  choice does not waive required checks or any other landing rule.
- **Outside contributions** stay participating objects
  (`../domain/forge.md`, section 13). A first-time contributor's CI
  waits for a maintainer's approval on GitHub, which temper shows as a
  wait on a person. CI on GitHub's runners is GitHub's isolation:
  setup.md's concern for CI's isolation is a self-hosted runner's.
- **Context** reads any public repository on github.com, with no
  installation.

## 11. What the probes found

Run on 2026-10-20 with a throwaway public App, `temper-probe-d0fc06`,
installed on the account `freeekanayaka` (repository
`freeekanayaka/temper-probe`) and the org `fabrig-ai`, then named
`fabrig-dev` (repository `fabrig-ai/temper-probe`; the API answers the
old names with a 301), with API version 2026-03-10; each repository
had one Actions workflow, a job `ci` that sleeps 45 seconds and fails
when the head's message says `FAIL`. Rulesets, pull requests, issues,
branches and tokens made by the probes were removed after. The scripts
and their evidence are not kept; the live conformance records the
facts for good (section 17). The App and the repositories are kept as
its fixtures, with a spare repository, `freeekanayaka/temper-probe-2`,
not granted to the App. The App's key, its hook secret, the person's
tokens (section 17) and a README are in the person's private
`~/src/rust/tmp/github-conformance/`, never in a repository. Each fact
is cited by its probe's number across this document.

- **Setup.** The manifest flow takes `http://127.0.0.1:<port>` as its
  `redirect_url` and `setup_url`; the conversion answers `id`, `slug`,
  `pem`, `webhook_secret`, `client_id`, `client_secret` and the App's
  `permissions` and `events`; every permission and event of section 5
  was taken, `merge_queues` and `merge_group` among them. A hook at
  `https://hooks.temper-probe.invalid/github` is taken.
- **P1, tokens.** An installation's token expires an hour after it is
  minted. One minted narrower, in permissions or repositories, is
  refused beyond them: `403` "Resource not accessible by integration".
  REST on an installation's token answers `x-ratelimit-limit` 5000 on
  the `core` resource; the App's JWT calls answer no `x-ratelimit-*`.
- **P2, identity.** `GET /user` on an installation's token: `403`.
  GraphQL's `viewer { login databaseId }` on it names the bot, its id
  the bot user's (GraphQL calls it a `User`, REST a `Bot`); `GET
  /users/<slug>%5Bbot%5D` gives the same id. `/app` and its routes need
  the JWT: `401` on an installation's token.
- **P3, git.** Clone and push over HTTPS with `http.extraHeader` basic
  authentication, `x-access-token` and the token, work. The App's git
  commits are unsigned (`verified` false, `unsigned`); its commits made
  through the contents API are GitHub's signed ones (`verified` true). A
  push changing `.github/workflows/ci.yml` is refused: "refusing to
  allow a GitHub App to create or update workflow
  `.github/workflows/ci.yml` without `workflows` permission".
- **P4, pull requests and CI.** A new pull request reads `mergeable`
  null, `mergeable_state` `unknown`, then settles within two seconds.
  Actions make a check run per event (push and pull request, each `ci`),
  with jobs and steps; suites of other Apps installed on the account (a
  Codecov, a Travis) sit `queued` with no run. The combined commit
  status stays `pending` with no status while Actions' runs end. A job's
  log answers `302` to `productionresultssa1.blob.core.windows.net`,
  fetched there with no credential, `Range` ignored (`200`, the whole
  log), its signed address valid 600 seconds (GitHub's documents say a
  minute).
- **P5, merging and updating.** A merge with a stale `sha`: `409` "Head
  branch was modified. Review and try the merge again."; made at once
  after the head moved: `405` "Pull Request is not mergeable"; with the
  head: `200`. With no rule, a merge succeeds while the head's CI runs.
  `update-branch` with a stale `expected_head_sha`: `422` "expected head
  sha didn’t match current head ref."; with the head: `202`, done
  asynchronously.
- **P6, conditional reads.** `If-None-Match` with the last `ETag` is
  answered `304` on a pull request, a commit's check runs and the
  delivery log, and a `304` leaves the rate's counters as they were. A
  pull request's `ETag` also changes when its repositories'
  `updated_at` does.
- **P7, the delivery log.** Each failed delivery is listed, newest first,
  with `status` "failed to connect to host", `status_code` 502, and its
  `guid`, `event`, `action`, `installation_id`, `repository_id`,
  `delivered_at`, `redelivery` and `duration`. A push was listed about
  1.5 seconds after it, a pull request opened about 2.2 seconds after.
  Pages go by `per_page` and an opaque `cursor`, with `Link` `next`. One
  delivery read whole holds the request's headers and payload (a push's
  about 8.6 KB) and the response's. `ping`, `installation` and
  `installation_repositories` are listed with no installation and no
  repository; their payloads name them. No delivery was retried. Ids
  have 19 digits.
- **P8, reviews.** The App approving its own pull request: `422`
  "Review Can not approve your own pull request". Its approval of a
  person's pull request counts toward a ruleset's required approval.
  Under a ruleset asking one approval, the App's merge of its own pull
  request: `405` "Repository rule violations found ... At least 1
  approving review is required by reviewers with write access."; with
  the App on the ruleset's bypass list (`Integration`, its id,
  `always`): `200`.
- **P9, the merge queue.** Under a ruleset requiring the queue, a REST
  merge: `405` "Repository rule violations found ... Changes must be
  made through the merge queue". `enqueuePullRequest` with a stale
  `expectedHeadOid`: `UNPROCESSABLE` "Failed to add PR #4: expected head
  oid does not match the current head oid"; with the head: `QUEUED`,
  then `AWAITING_CHECKS` on a group commit, which is the commit that
  lands, read by GraphQL's `mergeCommit` and on the base; REST reads
  `merged` true and `merge_commit_sha` null. Deliveries: `merge_group`
  `checks_requested` and `destroyed` (`merged` or `dequeued`), the pull
  request `enqueued`, `closed` and `dequeued` (`MERGE`, `CI_FAILURE`). A
  failed group's pull request was dequeued 28 seconds after its CI
  failed, unmerged. `enablePullRequestAutoMerge` on a repository not
  allowing it: "Auto merge is not allowed for this repository".
- **P10, signed commits.** Under a ruleset requiring them, the App's git
  push: refused, `GH013` "Commits must have verified signatures."; its
  commit through the contents API: taken.
- **P11, rules.** With administration to read, the App reads a branch's
  rules (`/rules/branches/{branch}`) with their rulesets, and each
  ruleset, with no bypass list but `current_user_can_bypass`, which
  turned from `never` to `always` once the App was on it. A branch under
  rulesets alone reads `404` "Branch not protected" at
  `/branches/{branch}/protection`.
- **P12, an installation's repositories.** A repository added to or
  removed from a selected installation by its owner is reached, or not,
  at once by a token minted for the whole installation before the
  change; `installation_repositories` `added` and `removed` are
  delivered. A fine-grained token with "GitHub App installation
  repository access" adds and removes one too (checked after the
  probes), but is refused the installation's list of repositories,
  which the App's token reads.
- **P13, issues.** The App opens issues and comments; issues and pull
  requests share numbers; `/issues` lists pull requests, each with a
  `pull_request` member; a new issue was missing from the first listing
  after it.

# Part C: Forgejo on the same footing

## 12. Connected and managed, from the client

### 12.1 Connected

- **What the person does on Forgejo** is setup.md's (section 4): a user
  for temper, not a site admin; its API token and its git token, with
  their scopes; its write access to each repository temper will adopt;
  optionally, hooks to temper's listener.
- **"Connect Forgejo"** in the client takes the two tokens, and the
  hooks' secret when the person adds hooks, in one request carrying
  values (section 3).
- **The engine checks before it keeps:** the API token's user (`GET
  /api/v1/user`), which becomes the forge's identity, shown to the
  person ("connected as temper, id 7"); the git token answering as the
  same user, which its `read:user` scope allows (skein's
  `forge-setup.md`, 4.3, step 3). A mismatch is refused, naming which.
  `forges[].user` goes from configuration: the identity is the one the
  token answered when the person connected it, kept and checked at each
  start (section 4).
- **Replaced** from the client: a token that must answer as the same
  user (section 4).

### 12.2 Managed

- **"Connect Forgejo as its admin":** the person pastes a site admin's
  token. The engine runs skein's reconcile (skein's `forge-setup.md`,
  section 4) with it, as its host, and keeps what reconcile makes, the
  bot's two tokens, its id and the hooks' secret, in the store. The
  forge then connects as 12.1's would.
- **Reconcile's host is in the engine:** its protocol layer runs it at
  each start, and when the person connects, before the component opens
  the forge; its items (skein's `forge-setup.md`, 4.1) are the store's
  records. The forge waits, "connecting", meanwhile; the engine serves.
- **Access per repository, at adoption,** in place of setup's `orgs`:
  adopting a repository on a managed forge first grants the bot write on
  it, as a collaborator (`PUT /api/v1/repos/{o}/{r}/collaborators/{bot}`),
  and makes its hook when the deployment has a webhooks' listener; then
  the adoption reads as any other (`../domain/forge.md`, section 4).
  This is a plan of skein's setup beside reconcile, run by the same
  host with the admin's token. Reconcile's org steps narrow: step 5
  makes an owned forge's orgs when missing (section 13), with no team,
  and step 6, each org's hook, goes. In their place, at each start, it
  checks and repairs each adopted repository's collaborator and hook.
- **The picker** lists every repository the admin can see: all of the
  forge's, since temper grants itself access when one is chosen
  (2.2).
- **The admin's token is kept** in the store, since adoption needs it
  at any time, as on an owned forge (section 13). It is read by the
  setup's host alone and never handed to the forge component, which
  makes every request of temper's work. Least power is restated so: the
  component never holds an admin's token; the engine's setup host holds
  one for a managed or owned forge, and uses it for reconcile and for
  access at adoption alone. On a managed forge, "Forget the admin's
  token" in the client makes the forge connected: the bot and its
  tokens stay, and access is the person's to give again.
- **Connected, then managed:** an admin's token given for a forge
  already connected keeps its user, since a forge's writer never changes
  (section 4). Reconcile is given the connected user's login, and takes
  it over when it carries temper's mark, `temper:bot`, in its full name;
  an unmarked user stops it (`Unmarked`, skein's `forge-setup.md`, 4.3),
  and the client says to add the mark on Forgejo, so that no login
  written wrong takes over a person's account.

## 13. Owned

- **Preparing and running stay commands,** with their files:
  `temper forge prepare` and `temper forge run` (setup.md, section 6).
  Forgejo's own secrets (`app.ini`, `state`) and the runner's are
  Forgejo's and the runner's to read, from files, as now; and so is the
  person's first password, `person_password`.
- **The admin's token lives in the store,** as a managed forge's does
  (12.2); only its way in differs, since temper's preparation makes it,
  not a person. The preparation makes it by Forgejo's command line,
  under Forgejo's lock and before the engine runs, so it cannot write
  the store: it leaves the token in `admin_token`, a handover, as today.
  The engine's setup host takes it at its start, through the same
  connect as a pasted admin's token: kept in the store, the file
  removed, and temper's admin's other tokens deleted on Forgejo, by the
  routes reconcile uses for the bot's (skein's `forge-setup.md`, 4.3,
  step 3). The file lives from the preparation to the engine's next
  start, and no credential is ever read from a file after.
- **A new token at each preparation:** with the file gone once taken,
  each preparation makes the next, as it makes one now when the file is
  missing (setup.md, 6.3, item 2). A box's every start so renews
  temper's admin's token, and one revoked on Forgejo is replaced at the
  next. `temper serve` started again alone, with no preparation before
  it, takes no file and runs on the store's token.
- **What reconcile makes goes to the store:** the bot's tokens, its id
  and the hooks' secret. The forge's directory loses `api_token`,
  `git_token`, `user` and `webhook_secret`, and configuration's forge
  stops naming them (section 14).
- **The box drops its reconcile steps** (setup.md, section 7): prepare,
  Forgejo, its runner, `temper serve`. A token revoked under a running
  engine is made again by the engine's next reconcile, at its restart.
- **The person's orgs:** `setup.orgs` goes. An owned setup keeps
  `mode.owned.person.orgs`, the orgs reconcile makes for the person when
  missing, owned by them, so that a fresh box has somewhere for
  repositories; temper's access to those repositories is granted at
  adoption, as on a managed forge (12.2).
- **Connected from the start:** an owned forge is connected by its
  preparation and the engine's first reconcile; the client has nothing
  to paste. An owned forge not yet prepared is not connected, and the
  client names `temper forge prepare`.

## 14. What changes in temper's documents

- **`../protocol/credentials.md`:** the table's rows for git identities,
  API tokens and webhooks' secrets name the store and the client;
  "no temper secret in jig's secrets' store" goes; least power restated
  (12.2); its open question on short-lived git credentials answered
  (section 8); a GitHub App's key, and its installations' tokens, as
  rows.
- **`../protocol/configuration.md`:** `forges[]` loses `api_token`,
  `user` and `webhook_secret`, and `kind` takes `"github"`, whose `api`,
  `transport` and `git` default to github.com's; `identities[]` loses
  `token`, and its `login` is the forge's (the Forgejo bot's,
  `x-access-token` on GitHub), with `author` defaulting to the bot's on
  GitHub; `forge.writers` in the root's `Config` comes from the store
  (section 4); limits for the puller, the installations, the store's
  records and the manifest's routes.
- **`../protocol/setup.md`:** the modes' table (who is given what, now
  from the client); the layers' files (section 13), `admin_token` a
  handover the engine takes and removes, the only file `temper serve`
  reads of a setup; reconcile's host in the engine; `setup.orgs` gone,
  `person.orgs` for owned; the box's steps; section 9's table, every
  credential of the engine's in the store; GitHub (part B) in place of
  "Not here".
- **`../protocol/forge.md`:** GitHub's dialect as temper uses it, the
  delivery log as a hint feed, credentials at runtime.
- **`../domain/forge.md`:** a forge connected or not (section 4); the
  writer at runtime; adoption's org paragraph replaced by access at
  adoption on managed and owned forges (12.2); GitHub's rules and its
  merge queue as a landing queue (section 10).
- **`../client/`:** the deployment's page (`forge.md`, 5.4) with each
  forge's connection and its Connect, Replace, "Install on another
  account" and "Forget the admin's token"; the adoption form
  (`README.md`, 4.4) as a picker, with "not listed" and its words;
  `tools.md`'s tools for each.
- **`../../help/`:** connecting GitHub; connecting Forgejo, each mode.

## 15. Where each piece goes

- **skein, `skein-forge`:** GitHub's dialect (REST at the pin, the few
  GraphQL documents, the deliveries and the delivery log, the manifest
  and its conversion, installations); the vocabulary's additions
  (validators, the rate's window, pauses); the credentials' kinds, the
  App's and its renewed installation token.
- **skein, `skein-forge-connection`:** a forge's credential handed and
  replaced at runtime, and a forge with none (section 4); renewal
  (skein's `forge-connection.md`, 5.1); the App's JWT, through `ring`;
  an installation's token chosen per repository; the delivery log's
  puller (section 7); a job log's redirect followed to its host alone.
- **skein, `skein-forge-client`:** a writer given at runtime; the
  budget's rate window and free `304`s.
- **skein, `skein-forge-setup`:** GitHub's plan (the manifest, the
  conversion, accepting or uninstalling an installation, the hook's
  address, the App's checks), and Forgejo's access per repository; its
  reconcile without orgs' teams and hooks (12.2); its host, as an
  in-process one too.
- **skein, the fakes:** a GitHub protocol layer over the fake forge's
  domain (skein's `fake-forge.md`, section 4): the REST and GraphQL
  temper uses, the manifest's form and conversion, installations and
  their tokens with expiry on the fake's clock, the delivery log, the
  merge queue, rulesets; the forge peer serving it.
- **skein, the conformance:** GitHub's, live, opt-in (section 17).
- **jig:** application credentials whose value changes while the engine
  runs, a generation each time (jig's `domain/engine.md`, 12.1); a
  request carrying a value (jig's `protocol/client.md`), already
  planned as LLM accounts' second part.
- **temper:** configuration (section 14); the engine's store records
  and their codec under temper's tag; the routing that hands them to the
  component and to the root; the clients' listener's routes for the
  manifest page, its redirect and its setup URL; the setup's host in the
  engine; the connector's connected state, writer, picker reads,
  GitHub's rules at adoption and its merge queue; the client's pages and
  tools; the box's steps; the help.

## 16. Order

Each step lands working, and shows what it proved in the real loop
(`../domain/testing.md`).

1. **The meeting point: temper lands on a GitHub-shaped fake.** GitHub's
   dialect for the calls temper makes, the fake's GitHub layer, the
   App's credential with its renewal, and the forge records read from
   the store at start (sections 3 and 4, at start alone). A real-loop
   story seeds the store with an App's record and lands a change on the
   fake, directly. Everything after builds on a path that works end to
   end.
2. **Connecting GitHub from the client:** the manifest flow on the
   clients' listener against the fake, the setup's GitHub plan, a
   credential and a writer handed at runtime, the deployment page's
   Connect, "Install on another account". The story connects as a
   person would, through `temper ctl`.
3. **The hint feed:** the delivery log's puller, conditional reads and
   the budget's additions; the story's latency with no listener.
4. **Adoption and landing:** the picker, for both forges; GitHub's rules
   in words; the merge queue as the branch's landing queue, ejection as
   repair.
5. **The live conformance** on github.com, once step 1's dialect exists,
   and on a schedule after.
6. **Forgejo's move** (part C), once jig's request carrying a value
   lands: connected pasted, managed's host in the engine with access at
   adoption, owned's files and the box's steps; the configuration's
   members removed last, when nothing reads them.

## 17. Testing

- **The dialect,** in step tests: each document encoded and decoded
  against the conformance's recorded answers; each refusal told apart
  (section 9); the delivery log's pages and its 19-digit ids.
- **The component's world:** renewal before expiry and a call waiting
  on it, on the fake's clock; a credential handed at runtime, replaced,
  and one answering as someone else; an installation's token chosen by
  repository; the puller's catch-up, cursor and `304`s; a job log's
  redirect followed to one host alone.
- **The setup's world:** the manifest's contents; the conversion's
  answer kept before the browser is answered; a state used twice, or
  unknown, refused; a stray installation uninstalled; Forgejo's access
  per repository and its repair.
- **The fake against GitHub:** the conformance runs the same checks on
  the fake and, opt-in, on github.com, as skein's does for Forgejo
  (skein's `fake-forge.md`, section 7), so the fake's quirks are
  GitHub's.
- **From the forge-kit plan, left to this work** (its ruling 189: part C
  reshapes what they would test). Each lands in skein's kit worlds, on
  part C's footing:
  - *The setup's world* (forge-kit's skein 07.4), through HTTP bytes on
    part C's reconcile: a token on page two replaced; a collaborator or
    repository hook on page two neither repeated nor skipped; two
    services keeping separate access on one forge; another service's
    bot refused as unmarked; a listener off loopback making no hook and
    no secret.
  - *The component's worst case* (skein 08.5): every holder filled and
    metered against `worst_case`, the credentials handed at runtime,
    the App's key, the installations' tokens and the puller among them.
  - *The protocol world* (skein 09.2–09.4): a scripted durable owner
    with restart, in skein's `tests/forge/protocol`. Stories: a creation
    lost on the wire found after the restart; listings following another
    party's changes; deliveries, signed or read from the log, reaching
    the owner. Invariants: one answer per call under cuts; a call's cost
    charged as skein's design answers it (a request cut inside its head
    is charged, not served); no credential in a record, a request line
    or a fact, the handed, replaced and renewed ones too. temper's
    protocol world covers these until then.
- **The installation's rate,** in the protocol world's qa profile
  (`tests/engine/protocol-forge/tests/qa_rate.rs`): one, two and three
  changes on one installation, heard by polling at the shipped
  cadences, counted as GitHub charges them (the installation's HTTP
  requests, the App's JWT calls apart) over a virtual window scaled to
  the hour. Three shapes: CI pending, heads pushed every five minutes,
  and main moving besides. Each states its total per hour and its
  requests by kind, and the base reads a shared read would save.
- **The real loop** (`../domain/testing.md`, 2.4): GitHub connected
  through the manifest flow, the browser's steps driven as requests;
  repositories added to a project later; a direct landing, and one
  through the merge queue with an ejection repaired; an approval rule
  waiting on a person, then lifted by the bypass; a token renewed in the
  middle of a run, git's generation changing under it; a repository
  granted on GitHub appearing in the picker; and, for Forgejo, each
  mode connected from the client. After each, no secret in a root, a
  fact, a record but the store's, a request or an answer
  (credentials.md, section 7).
- **The browser's tier,** only for what is the browser's: the manifest
  page's form posted to GitHub's address (`../client/browser.md`).
- **Live, outside every suite:** the conformance on github.com with the
  fixtures' App and repositories, credentials from the person's private
  directory, never the suites'. The person's part (rulesets, approvals,
  an installation's repositories) is played by fine-grained tokens,
  one per owner since such a token has one, each limited to that
  owner's probe repositories, with administration, contents, pull
  requests and the App installation's repository access to write, and
  metadata to read. The user made them on 2026-10-20, beside the App's
  key; each was checked to write its own repositories and be refused the
  other's. The user's own classic token is never used.
- **Its credentials expire,** the person's tokens on their dates and
  the App's key never: each run reads a token's expiry from GitHub's
  `github-authentication-token-expiration` header, and names a token
  that expires within the next month, so the user makes it again before
  a run fails on it.

- **temper's live landing story** (`scripts/proof-live-github.sh`, R2-156)
  uses only `app.pem` as a credential, with public App and installation
  ids from `app.json`. It adopts the probe with `--no-ci --check 1`, using
  the scripted gate because the probe repositories run no CI. Its prefix
  is `temper/live-<run>-`: the run and root number share one branch
  segment, keeping the change's authority name within eight segments.
  Before starting the product, the harness signs an
  RS256 App JWT with Skein's JWT builder and signer, then mints a token
  for the owner's probe installation through the guarded relay, scoped
  to `temper-probe`. This makes the first verifier read independent of
  product startup. The relay keeps that token with the product's minted
  tokens in memory for the secrets scan and revokes them at teardown;
  the verifier JWT is scanned too. Reads need metadata, contents, pull
  requests and checks read permissions; cleanup also needs contents and
  pull requests write, already needed by the product's landing story.
  The script's backstop signs and mints its own temporary cleanup token
  from the same key and revokes it afterwards. It requires Python's
  `cryptography` package. This run needs no person token; the separate
  conformance's person actions above do not belong to it.
  After the backstop, the script removes its private state directory on
  every exit, pass or fail; set `TEMPER_LIVE_GITHUB_KEEP=1` to keep it for inspection.

## 18. Later

- **Workflows:** the `workflows` permission, so temper's changes may
  touch `.github/workflows/`, with what that lets an agent run.
- **A public address:** pushed deliveries through a person's proxy, or
  skein's TLS server side; the log stays the backstop.
- **A fine-grained token** as connected, a machine user's.
- **GitHub Enterprise Server:** the same dialect at another address, its
  own API versions.
- **Forks:** temper's fork of an upstream on GitHub
  (`../domain/forge.md`, section 14).
- **Signed commits:** temper's commits made through the API, which
  GitHub signs (P3, P10), for repositories that require them.
- **The reviewer:** a second App of temper's whose approval counts
  (P8), for a review an agent passed.
- **Signing people in with GitHub,** through the App's OAuth client
  (credentials.md, section 8, the forge as sign-in provider).
- **Batched reads** in one GraphQL query.
- **Disconnecting,** and connecting a forge as someone else: what
  becomes of the adopted repositories, their keys and their branches.
- **An App an organization owns,** made through the organization's
  manifest address, `/organizations/<org>/settings/apps/new` (skein's
  `Target::Org`), for a team whose App should outlive its maker's
  account. Until then the App is the person's own, and installs on
  their organizations.
- **An owner's approval:** a member who isn't an owner requests the
  App, and an owner approves it later, with no state of temper's;
  temper would wait for that installation and accept it, where it now
  says why it can't go on and treats it as a stray.

## 19. Open questions

Each with the default the draft takes.

- **The dialect's home:** a module of `skein-forge` beside Forgejo's, or
  a crate of its own. Default: a module, as skein's `forge.md` (2.1)
  lays the dialects out.
- **A public App, accepted by link:** any GitHub account may install a
  public App; temper accepts only installations that come back with its
  `state`, and uninstalls the rest. Default: so, with each stray named
  on the deployment's page.
- **Git's identity on GitHub:** one identity whose value holds every
  installation's token (section 8), or an identity per installation.
  Default: one, since a forge has one identity (credentials.md, section
  4) and accounts come and go at runtime.
- **`log_poll` and the hook's fate:** five seconds by default; GitHub
  publishes no rate for the App's JWT calls, and may one day disable a
  hook that never answers. The probe App's hook is to be checked again
  some days after the probes; a disabled hook would make the log empty,
  and the client would then say so and fall back on polling.
- **The merge method:** the first allowed of merge, squash and rebase;
  or the adoption asks. Default: the first allowed, shown at adoption.
- **The installation's scope:** all repositories, or selected. Default:
  the install page's own choice, all repositories, as GitHub offers it
  (with the user, 2026-10-23).
- **`log_catch_up`:** how far back the puller reads at start. GitHub's
  documents keep deliveries three days. Default: back to the newest
  `guid` temper kept, and within a day at most; the slow pass covers the
  rest.
- **Three iterating changes on one installation:** they ask about
  6,000 requests an hour, over the installation's 5,000 (section 8);
  two ask about 4,300, and three with CI pending about 4,200
  (`qa_rate.rs`). The base's reads are about 5 % of it, and sharing
  them across changes would save under 1 %. The pull's keep, six reads per
  pass, is about 70 %, and the repository's listing 720 an hour.
  The fake charged every answer here, while GitHub answers an unchanged
  read with a validator `304` at no cost (sections 9 and 11, P6); which
  of the keep's reads would be `304`s is still to check. Which cut, if
  any, brings three iterating changes under the rate: those `304`s, the
  keep's passes, or the listing's interval?
- **Generations across starts** (credentials.md, section 8): generation
  1 at every start, and growing while the engine runs. Default: still
  standalone only.
