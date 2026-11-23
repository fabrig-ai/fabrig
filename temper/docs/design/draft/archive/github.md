# GitHub, and forges connected from the client

Draft, 2026-10-20. temper on github.com, as easy to start as GitHub
allows; and, on the way, every forge connected from temper's client,
its secrets in jig's secrets' store, and repositories added to projects
whenever a person likes. It records a design session with the user on
2026-10-20 and the probes run on github.com that day (`../github-app.md`, section 7). It
crosses skein (the dialect, the component, the setup's plan, the fakes,
the conformance), jig (credentials made at runtime) and temper
(configuration, the connector, the setup, the client) on purpose; an
owner splits it. It builds on skein's forge kit (skein's `forge/README.md`,
`skein/docs/design/forge/connection.md`, `skein/docs/design/forge/client.md`, `skein/docs/design/forge/setup/README.md`,
`skein/docs/design/testing/fakes/forge.md`), and fills the seams those documents
keep for GitHub (skein's `forge/README.md`, section 8; `skein/docs/design/forge/setup/github.md`,
section 1; `skein/docs/design/forge/connection.md`, 5.1). Nothing here is adopted. Part A
is what both forges share, part B is GitHub, part C moves Forgejo onto
part A. Where each piece goes is `github.md`, section 15, an order `github.md`, section 16, what
is open `github.md`, section 19.

## 1. In one page

- **Connect, then add repositories, any time.** A forge is a record in
  the store, made as it is set up or connected (`playground-reset.md`,
  section 3), never a credential in a file and never a repository list.
  A person connects the forge from temper's client, then adds
  repositories to a project from a list of those temper can reach, now
  or later. Nothing is imported in bulk at configuration (section 2).
- **Secrets in the store.** A forge's credentials are made from the
  client, or by temper while it connects, and kept in jig's secrets'
  store under temper's tag (jig's `protocol/store/secrets.md`),
  never in a file temper reads at start: one way, in every forge and
  mode. An owned Forgejo's preparation, which runs before the engine,
  hands its admin's token over once, by a file the engine takes in and
  removes. This reverses ../../protocol/credentials.md's "no temper secret in jig's
  secrets' store" (`github.md`, section 3).
- **A forge connected while temper runs.** skein's component takes a
  forge's credential at runtime, and again when it is replaced or
  renewed; git identities take new generations as they change
  (section 4).
- **temper is a GitHub App,** one per deployment, never the person.
  GitHub refuses an author's approval of their own pull request, and
  temper tells its own writes from others' by its user's id, so temper
  must be someone else (`../github-app.md`, section 1).
- **Connecting GitHub is two clicks** (`../github-app.md`, section 2): "Connect GitHub" in
  the client takes the tab to a page on temper's listener that posts
  the App's manifest to GitHub as it loads; the person clicks "Create",
  GitHub sends the browser back with a code, temper exchanges it for
  the App's key and keeps it; the person picks an account and
  repositories on the install page, and GitHub's setup redirect brings
  the tab back to the page that started it. No organization is asked:
  the App is the person's, and GitHub's install page is the menu of
  accounts. No token is pasted. GitHub has no API that creates or installs an
  App, so nothing does better. Probed (`../github-app.md`, section 7).
- **No public address.** The App's webhook points at a host that cannot
  resolve; GitHub still records each delivery, and temper reads that
  record, the App's delivery log, every few seconds as its hint feed.
  Conditional reads of what temper waits on cost nothing when nothing
  changed. A public address, when a deployment has one, adds pushed
  deliveries; nothing depends on it (`../github-app.md`, section 3).
- **Credentials that expire.** An installation's token lasts an hour:
  the component renews it, and the git identity takes a new generation
  (`../github-app.md`, section 4).
- **Landing on its own, wherever the rules allow.** A merge at the head
  temper's decision saw; GitHub's merge queue, entered at that head,
  where the branch requires it. A rule temper's identity cannot satisfy
  is named at adoption, with the one setting that would let temper land;
  temper never changes a repository's rules (`github.md`, section 2.3, and `../github-app.md`, section 6).
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

- **A forge's record names the forge, not its access**
  (`playground-reset.md`, section 3). Each forge's kind and addresses:
  for Forgejo its `api`, `transport` and `git`; for GitHub, whose
  addresses default to github.com's and are given only to point at a
  fake. No token, user id, webhook secret or repository.
- **A forge not yet connected** is kept, shown and idle: the
  engine starts and serves; the client's deployment page says "Not
  connected" with its Connect action, and the adoption form offers none
  of its repositories. Nothing on it runs. This is how an LLM provider
  with no account is shown (../../protocol/credentials-llm-accounts.md, section 1).
- **Who connects:** a person whose role holds the deployment's
  connectors (jig's `client/domain/pages.md`, section 7): in standalone,
  the deployment's owner.
- **How,** per forge and mode:
  - GitHub: "Connect GitHub", the manifest flow (`../github-app.md`, section 2), or an App
    the person made, its id and key pasted (`../github-app.md`, 2.5);
  - Forgejo connected: the bot's two tokens pasted (`../github-forgejo.md`, section 1.1);
  - Forgejo managed: a site admin's token pasted, temper making its bot
    (`../github-forgejo.md`, section 1.2);
  - Forgejo owned: connected by its preparation, at start (`../github-forgejo.md`, section 2).
- **What connecting ends with** is skein's access (skein's
  `forge/setup/README.md`, section 2), kept in the store: who temper is there,
  its credentials, and the webhook secret. The component then opens the
  forge and the client offers its repositories.
- **Replaced from the client:** a new token, a new App key. The new
  credential is kept, handed to the component, and the old one erased.
  Disconnecting is later (section 18).
- **`temper ctl`** offers the same, as tools (`../../client/ctl/README.md`), so
  that QA's agents and tests connect a forge as a person would.

### 2.2 Adding repositories, any time

- **No repository is named at configuration or at connecting.** A
  project's owner adds repositories to the project from the client,
  whenever they like, through the adoption form (`../../client/forge/family.md`,
  4; `../../client/forge/repositories.md`, 1).
- **The form becomes a picker.** It lists, per connected forge, the
  repositories temper can reach and the project does not hold, and
  takes an owner and a name for one not listed. What temper can reach:
  - GitHub: the repositories of the App's installations;
  - Forgejo connected: those the bot can write;
  - Forgejo managed: every repository of the forge, since temper grants
    itself access at adoption (`../github-forgejo.md`, section 1.2);
  - any forge: any repository the credential can read, as context.
- **A repository temper cannot reach** is shown with what would make it
  reachable, in words: on GitHub, granting it to temper's App on
  GitHub's installation page, linked, or installing the App on another
  account through temper's own link (`../github-app.md`, section 2.3); on a connected
  Forgejo, making the bot, named by its login, a collaborator. Once the
  forge says so, the repository is listed, without a restart.
- **The rest of the form is as it is:** role, home, prefix, landing
  branch, CI or checks, and what temper found there before the person
  confirms (`../../client/forge/repositories.md`, 1), with GitHub's rules in words
  (`../github-app.md`, section 6.3).
- **Forgejo too.** setup's configured `orgs` go: access is per
  repository, at adoption (`../github-forgejo.md`, section 1.2).

### 2.3 Landing on its own

- **The project's landing rules decide** whether temper lands
  unattended (`../../protocol/configuration/engine.md`, `landing`;
  `../../domain/forge/change/gates.md`, 2). Landing waits for people only where those
  rules ask for their approval.
- **temper lands the forge's way:** a merge at the head its decision saw;
  on GitHub, where the branch requires its merge queue, the pull request
  enters the queue at that head (`../github-app.md`, section 6.2).
- **A rule of the forge's that temper's identity cannot satisfy** is
  found at adoption, and shown with the one setting that would let
  temper land, for the person to choose: on GitHub, a required approval
  (the App cannot approve its own pull request) is met by a person's
  approval, or by the App on the ruleset's bypass list, which adoption
  reads as the ruleset's `current_user_can_bypass` (`../github-app.md`, section 7, P8 and
  P11); required signed commits, by temper's commits made through the
  API (`github.md`, section 18); a code owner's review, by a person. Until then, its
  landings wait for a person, which the change's panel says
  (`../../client/forge/README.md`, 2.5). temper never changes a repository's
  rules.
- **An approval that is not the person's** is possible later: an App's
  approval counts toward a ruleset's required approvals (`../github-app.md`, section 7,
  P8), so a second App of temper's, the reviewer, could approve what an
  agent's review passed (`github.md`, section 18).

## 3. Secrets in the store

- **One way, in every forge and mode:** each credential the engine uses
  lives in the store, and nowhere else, and enters it through a
  connect: pasted in the client, made by the engine's own exchange or
  reconcile, or, on an owned Forgejo alone, handed over once by its
  preparation, which runs before the engine and cannot write the store
  (`../github-forgejo.md`, section 2).
- **Where:** jig's secrets' store, a private skein-kv store of the
  engine's (jig's `protocol/store/secrets.md`), which keeps "an
  application's own records, under its tag: its connectors'
  credentials, which it decodes itself". temper's records there:
  - GitHub: the App's id, slug and private key, and its webhook secret;
    the installation tokens are never kept, but minted (`../github-app.md`, section 4);
  - Forgejo: the bot's API token, its git token, its id and the webhook
    secret; managed and owned, a site admin's token too (`../github-forgejo.md`, 1.2, `../github-forgejo.md`, section 2
    );
  - each with the forge user's id it stands for, checked at each start.
- **Not there, being no credential of the engine's:** what other
  programs read from files, an owned Forgejo's own secrets (`app.ini`,
  `state`) and its runner's, and the person's first password on it
  (`../github-forgejo.md`, section 2); a worker's secret and its digest, jig's files, which
  open the link before any store is reached; LLM accounts, already in
  the store as jig's.
- **jig's rules hold** (jig's `protocol/store/secrets.md`): written
  by components alone, through the protocol layer's routing; kept, and
  durable, before used; replaced stating the digest it replaces; erased
  when replaced or removed; a failed commit stops the engine. No root,
  record, fact, log or request's answer holds a value.
- **A pasted token** reaches the engine in a request the client sends
  once and never keeps, kept in the store before the core sees the
  request, as an LLM key is (jig's `domain/accounts.md`, section 5;
  jig's `protocol/client/README.md`, section 3). This is jig's "request
  carrying a value": Forgejo's connect and an App pasted (`../github-app.md`, 2.5) take it;
  the manifest flow does not, since GitHub hands the App's key to the
  engine's own exchange (`../github-app.md`, section 2).
- **../../protocol/credentials.md changes:** its table's rows for git identities, the
  forges' API tokens and webhook secrets name the store; its rule "no
  temper secret in jig's secrets' store" goes; "least power" is
  restated for the admin's token (`../github-forgejo.md`, section 1.2).
- **A person's backups** of the store's directories now hold the forges'
  credentials, as they already hold LLM accounts'.

## 4. A forge connected while temper runs

- **skein's component takes a forge's credential at runtime.** Today
  the owner hands each forge's credential and user id at start
  (skein's `forge/connection.md`, 3.1). A forge may now start with
  none: its calls are refused as not connected, at once, not waited on;
  the owner hands the credential later, as a message, and the forge's
  start (the identity read, the page cap read; 3.3) runs then. A
  replaced credential is the same message; the calls in flight end as
  they would at a revocation. Renewal (5.1) is the component's own
  replacement, from the dialect's plan.
- **The identity it must stand for** is the one recorded at connect,
  kept beside the credential, in place of `forges[].user`. The start's
  check is as fail-stop as now (../../protocol/credentials.md, section 5): a
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
  installations' tokens (GitHub, `../github-app.md`, section 4). The engine announces
  `Event::Credential` with a new generation whenever the value changes,
  at runtime, where today it announces generation 1 at start alone
  (jig's `domain/engine/README.md`, 6.1; ../../protocol/credentials-git-identities.md, section 1). The
  worker keeps the two newest generations, so an operation in flight
  across a change finishes on the value it began with.
- **The forge writer at runtime.** skein's forge client takes its
  writers, one per forge, at construction (skein's `forge/client.md`,
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

Moved to [`../github-app.md`](../github-app.md), section 1.

## 6. Connecting GitHub: the manifest flow

Moved to [`../github-app.md`](../github-app.md), section 2.

## 7. Hearing changes without a public address

Moved to [`../github-app.md`](../github-app.md), section 3.

## 8. Credentials that expire

Moved to [`../github-app.md`](../github-app.md), section 4.

## 9. The dialect

Moved to [`../github-app.md`](../github-app.md), section 5.

## 10. Adoption and landing on GitHub

Moved to [`../github-app.md`](../github-app.md), section 6.

## 11. What the probes found

Moved to [`../github-app.md`](../github-app.md), section 7.

# Part C: Forgejo on the same footing

## 12. Connected and managed, from the client

Moved to [`../github-forgejo.md`](../github-forgejo.md), section 1.

## 13. Owned

Moved to [`../github-forgejo.md`](../github-forgejo.md), section 2.

## 14. What changes in temper's documents

- **`../../protocol/credentials.md`:** the table's rows for git identities,
  API tokens and webhooks' secrets name the store and the client;
  "no temper secret in jig's secrets' store" goes; least power restated
  (`../github-forgejo.md`, 1.2); its open question on short-lived git credentials answered
  (`../github-app.md`, section 4); a GitHub App's key, and its installations' tokens, as
  rows.
- **`../../protocol/configuration/README.md`:** `forges[]` loses `api_token`,
  `user` and `webhook_secret`, and `kind` takes `"github"`, whose `api`,
  `transport` and `git` default to github.com's; `identities[]` loses
  `token`, and its `login` is the forge's (the Forgejo bot's,
  `x-access-token` on GitHub), with `author` defaulting to the bot's on
  GitHub; `forge.writers` in the root's `Config` comes from the store
  (section 4); limits for the puller, the installations, the store's
  records and the manifest's routes.
- **`../../protocol/setup/README.md`:** the modes' table (who is given what, now
  from the client); the layers' files (`../github-forgejo.md`, section 2), `admin_token` a
  handover the engine takes and removes, the only file `temper serve`
  reads of a setup; reconcile's host in the engine; `setup.orgs` gone,
  `person.orgs` for owned; the box's steps; `../../protocol/setup/README.md`, section 5's table, every
  credential of the engine's in the store; GitHub (part B) in place of
  "Not here".
- **`../../protocol/forge/README.md`:** GitHub's dialect as temper uses it, the
  delivery log as a hint feed, credentials at runtime.
- **`../../domain/forge/README.md`:** a forge connected or not (`../../domain/forge/adoption.md`); the
  writer at runtime; adoption's org paragraph replaced by access at
  adoption on managed and owned forges (`../github-forgejo.md`, 1.2); GitHub's rules and its
  merge queue as a landing queue (`../github-app.md`, section 6).
- **`../client/`:** the deployment's page (`../../client/forge/deployment.md`)
  with each forge's connection and its Connect, Replace, "Install on
  another account" and "Forget the admin's token"; the adoption form
  (`../../client/forge/family.md`, 4) as a picker, with "not listed" and its
  words; `../../client/ctl/README.md`'s tools for each.
- **`../../help/`:** connecting GitHub; connecting Forgejo, each mode.

## 15. Where each piece goes

- **skein, `skein-forge`:** GitHub's dialect (REST at the pin, the few
  GraphQL documents, the deliveries and the delivery log, the manifest
  and its conversion, installations); the vocabulary's additions
  (validators, the rate's window, pauses); the credentials' kinds, the
  App's and its renewed installation token.
- **skein, `skein-forge-connection`:** a forge's credential handed and
  replaced at runtime, and a forge with none (section 4); renewal
  (skein's `forge/connection.md`, 5.1); the App's JWT, through `ring`;
  an installation's token chosen per repository; the delivery log's
  puller (`../github-app.md`, section 3); a job log's redirect followed to its host alone.
- **skein, `skein-forge-client`:** a writer given at runtime; the
  budget's rate window and free `304`s.
- **skein, `skein-forge-setup`:** GitHub's plan (the manifest, the
  conversion, accepting or uninstalling an installation, the hook's
  address, the App's checks), and Forgejo's access per repository; its
  reconcile without orgs' teams and hooks (`../github-forgejo.md`, 1.2); its host, as an
  in-process one too.
- **skein, the fakes:** a GitHub protocol layer over the fake forge's
  domain (skein's `testing/fakes/forgejo.md`, section 1): the REST and GraphQL
  temper uses, the manifest's form and conversion, installations and
  their tokens with expiry on the fake's clock, the delivery log, the
  merge queue, rulesets; the forge peer serving it.
- **skein, the conformance:** GitHub's, live, opt-in (section 17).
- **jig:** application credentials whose value changes while the engine
  runs, a generation each time (jig's `domain/engine/README.md`, 6.1); a
  request carrying a value (jig's `protocol/client/README.md`), already
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
(`../../testing/README.md`).

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
  (`../github-app.md`, section 5); the delivery log's pages and its 19-digit ids.
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
  (skein's `testing/fakes/throwaway.md`, section 2), so the fake's quirks are
  GitHub's.
- **Left to this work,** since part C reshapes what they test. Each
  lands in skein's kit worlds, on part C's footing:
  - *The setup's world,* through HTTP bytes on
    part C's reconcile: a token on page two replaced; a collaborator or
    repository hook on page two neither repeated nor skipped; two
    services keeping separate access on one forge; another service's
    bot refused as unmarked; a listener off loopback making no hook and
    no secret.
  - *The component's worst case:* every holder filled and
    metered against `worst_case`, the credentials handed at runtime,
    the App's key, the installations' tokens and the puller among them.
  - *The protocol world:* a scripted durable owner
    with restart, in skein's `tests/forge/protocol`. Stories: a creation
    lost on the wire found after the restart; listings following another
    party's changes; deliveries, signed or read from the log, reaching
    the owner. Invariants: one answer per call under cuts; a call's cost
    charged as skein's design answers it (a request cut inside its head
    is charged, not served); no credential in a record, a request line
    or a fact, the handed, replaced and renewed ones too. temper's
    protocol world covers these until then.
- **The installation's rate,** in the protocol world's focused suite
  (`tests/engine/protocol-forge/tests/suite/rate.rs`): one, two and three
  changes on one installation, heard by polling at the shipped
  cadences, counted as GitHub charges them (the installation's HTTP
  requests, the App's JWT calls apart) over a virtual window scaled to
  the hour. Three shapes: CI pending, heads pushed every five minutes,
  and main moving besides. Each states its total per hour and its
  requests by kind, and the base reads a shared read would save.
- **The real loop** (`../../testing/real-loop.md`): GitHub connected
  through the manifest flow, the browser's steps driven as requests;
  repositories added to a project later; a direct landing, and one
  through the merge queue with an ejection repaired; an approval rule
  waiting on a person, then lifted by the bypass; a token renewed in the
  middle of a run, git's generation changing under it; a repository
  granted on GitHub appearing in the picker; and, for Forgejo, each
  mode connected from the client. After each, no secret in a root, a
  fact, a record but the store's, a request or an answer
  (../../protocol/credentials.md, section 7).
- **The browser's tier,** only for what is the browser's: the manifest
  page's form posted to GitHub's address (`../../client/web/README.md`).
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
  (`../../domain/forge/README.md`, section 8).
- **Signed commits:** temper's commits made through the API, which
  GitHub signs (P3, P10), for repositories that require them.
- **The reviewer:** a second App of temper's whose approval counts
  (P8), for a review an agent passed.
- **Signing people in with GitHub,** through the App's OAuth client
  (../../protocol/credentials.md, section 8, the forge as sign-in provider).
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
  a crate of its own. Default: a module, as skein's `forge/README.md` (2.1)
  lays the dialects out.
- **A public App, accepted by link:** any GitHub account may install a
  public App; temper accepts only installations that come back with its
  `state`, and uninstalls the rest. Default: so, with each stray named
  on the deployment's page.
- **Git's identity on GitHub:** one identity whose value holds every
  installation's token (`../github-app.md`, section 4), or an identity per installation.
  Default: one, since a forge has one identity (../../protocol/credentials-git-identities.md, section
  1) and accounts come and go at runtime.
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
  6,000 requests an hour, over the installation's 5,000 (`../github-app.md`, section 4);
  two ask about 4,300, and three with CI pending about 4,200
  (`suite/rate.rs`). The base's reads are about 5 % of it, and sharing
  them across changes would save under 1 %. The pull's keep, six reads per
  pass, is about 70 %, and the repository's listing 720 an hour.
  The fake charged every answer here, while GitHub answers an unchanged
  read with a validator `304` at no cost (`../github-app.md`, sections 5 and 7, P6); which
  of the keep's reads would be `304`s is still to check. Which cut, if
  any, brings three iterating changes under the rate: those `304`s, the
  keep's passes, or the listing's interval?
- **Generations across starts** (../../protocol/credentials.md, section 8): generation
  1 at every start, and growing while the engine runs. Default: still
  standalone only.
