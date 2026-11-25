# LLM accounts

Provisional, 2026-10-19, revised 2026-10-27. How an application on jig pays for its runs'
LLMs: who holds an account, the sources by which a run may use it, the
states the accounts child keeps of each, whose work it pays for, which
account each run gets, what people do with their accounts, and how an
account moves. It applies providers.md, the LLM providers' terms. The
model a task runs on is the task's (tasks/README.md, section 3), within the
models its authority names (authority.md, section 3); the accounts
child's place in the engine is engine/README.md, section 6; where records and
keys live is `../protocol/store/secrets.md`, and how the protocol
layer signs in, refreshes and lends is `../protocol/llm.md`, sections 5
and 6. Running a program, such as Claude Code, as the agent's backend is
smith's (smith's `protocol/programs.md`). What is still open is listed
in `accounts.md`, section 7.

## 1. In one page

- **Each application is its own client, with its own state.** It signs
  in under its own name, keeps its records in its own store, and
  refreshes them itself. No application reads another program's login
  or shares a file with one: not Codex's, not a person's `~/.claude`,
  not another application's (section 2).
- **An account is a holder, an LLM provider, its sources and its
  uses.** The holder is a person or the deployment; the sources, in the
  holder's order, say how a run may use the account; the uses say whose
  work it pays for (section 3).
- **A source is how a run is authorized:** *signed in*, with the
  application's own registration; a *key*; a *program*'s own login on a
  host, such as Claude Code's (`claude_code`); or *none*, for a local
  model. A new LLM provider is configuration, and a dialect or a
  program's adapter where it needs one, never a change to the domain
  (`accounts-sources.md`, 1).
- **A person may hold several accounts with one LLM provider,** each its
  own account, told apart by the provider's subject, never by an email;
  a subject has one holder in a deployment (3.3).
- **Each source has a state:** usable, limited, refused, needs sign-in,
  misconfigured or unloaded. The accounts child keeps them, from what
  the accounts' component and the runs' agents tell it, and starts no
  run on a source that is not usable (`accounts-sources.md`, 2).
- **One person, no friction.** A plan pays, by default, for its holder's
  own work and for the automatic work of each project in which its
  holder is the only party, so a person alone on a deployment signs in
  once and every run uses their subscription (`accounts-uses.md`, section 1).
- **Who pays is decided with the run's check** and recorded in its
  claim: the account that serves the work's payer, and its first source
  usable on a host with room. A task keeps its account from run to run,
  chooses again when its payer changes, and waits for it at a limit,
  never moving to another (`accounts-uses.md`, 1.3).
- **A host that keeps a person's program logins takes only runs that
  person's accounts pay for,** until a run's process tree is contained
  from them, since a run there can read a login and a subscription
  serves one person. A deployment of one person is unaffected (`accounts-sources.md`, 1).
- **Values are never committed.** Records and keys are in the secrets'
  store, kept before they are lent; a request carrying a key names it by
  number once the protocol layer has kept it. The engine alone
  refreshes, so no two processes race a rotating token (section 5;
  `../protocol/store/secrets.md`).
- **Moving an account:** a signed-in source, between deployments of one
  application, never copied: out into a private file, then erased; in
  from it, refreshed once before it is lent, so that a copy left behind
  is dead (section 6).

## 2. Each application is its own client

- **Its own registration.** An application signs a person in under its
  own name: Sign in with ChatGPT's `agent_name_hint` is the
  application's name, used consistently (providers.md, 2.2). The
  installation's identifier that OpenAI asks for, `ext_agent_host_id`,
  is drawn once per deployment, at its first start, as `urn:uuid:…`,
  and kept in its store; it is never a person's identifier.
- **Its own records, refreshed by itself.** A sign-in's record and a
  key are kept in the application's store (`../protocol/store/secrets.md`
  ), and its engine's accounts' component alone refreshes a
  record (skein's `oauth/accounts.md`), so refreshes of one record are
  serialized by construction (providers.md, 2.3).
- **No borrowed login.** No application reads another program's login
  file, a Codex login or another application's records. A program the
  agent runs (`accounts-sources.md`, 1, *program*) keeps its own login in a configuration
  directory of the application's own, one per source and per host
  (smith's `protocol/programs.md`, section 4), never the person's: a
  person's own Claude Code settings, hooks and plugins never load into a
  run.
- **smith's own product does likewise:** it signs in under its own name,
  keeps its records in its own state, and borrows nothing (smith's
  `shell/credentials.md`).

## 3. Accounts

### 3.1 What an account is

```
Account
  number      the domain's name for it, never reused
  provider    an LLM provider of the configuration's, by number
  holder      a person, or the deployment
  subject     the LLM provider's stable identifier for the account, where
              a signed-in source gives one; beside it, a display name
              its holder gives
  sources     in the holder's order: each a kind and a state, and for a
              program's, its name and its login on each host (3.4)
  uses        the payers it pays for (4.2)
```

What its LLM provider says of it, its email among them, is kept beside it,
in a record of its own (`accounts-lists.md`, 1).

- **An LLM provider** is configuration: its number, its name for
  showing, its endpoints and models, the sources it offers and what each
  needs (a sign-in's registration, a program), and the facts its terms
  ask an application to show, such as a manage-usage address
  (providers.md, 2.6; `../protocol/configuration-engine.md`, section 1). jig's
  code names no LLM provider. It is not an identity provider
  (people/README.md, section 3), and the two are numbered apart.
- **Accounts are made at runtime,** by a person's sign-in, a key added
  or a program's source declared (section 5), and kept in the store
  (engine/decisions.md, 1.4). An LLM provider whose source is *none* has one
  account, the deployment's, made at the first start that configures
  it. An application's configuration names LLM providers, never
  accounts, and an endpoint names no account. On restart the accounts
  child restores those durable rows and names every signed-in or key
  source for the component, including deployment-held sources. The source
  sweep retains exactly those named secrets, leaving application-carried
  values to their own owner.
- **The domain sees names and states,** never a value: an account by
  number, a source by its account and its place, a program's also by
  its name, and a credential by its source and a generation, with how
  long it is valid (`accounts-sources.md`, 2).
- **The subject** is the LLM provider's validated identifier (an ID
  token's `sub`), never an email. It is the account's own, and no
  person's subject in people/README.md, section 3: an LLM provider is no
  identity provider, and a subject here opens no session. A key or a
  program's login gives none:
  an account with no signed-in source has no subject, and is told apart
  by its number and the display name its holder gives it. jig checks no
  subject for a program's source, since the program names no account it
  is signed in to (smith's `protocol/programs.md`, section 4); an email
  its status names is a fact to show (`accounts-lists.md`, 1), never a subject.

### 3.2 Holders

- **Only a person holds a plan:** a subscription a person pays for, such
  as ChatGPT's or Claude's, used through a signed-in source or a
  program's login. A subscription serves one person (providers.md,
  section 1). An account with a plan among its sources is a *plan's
  account*.
- **A person or the deployment holds a key:** a person's own, or the
  deployment's, billed to the organization that owns it (providers.md,
  2.7 and 3.1). The deployment holds a *none* provider's account.
- **A service holds no account.** A plan's uses may name one (`accounts-uses.md`, 1.2), and
  a key's may.
- **A holder decides about their account:** the order of its sources
  and its place among their accounts, its uses, signing it out,
  removing it, and moving it (section 6). For the deployment's
  accounts, its owners decide: the first owners its configuration names (people/README.md,
  section 3; `../protocol/configuration-engine.md`, section 1), and, in their
  name, a connector the configuration grants the deployment's `AddKey`
  or `RemoveAccount` (people/acts.md, 3), where no client is there to ask.

### 3.3 Several accounts with one LLM provider

- **Each its own account.** A person may hold several subscriptions
  with one LLM provider, under several emails, and each sign-in to a new
  subject makes a new account. A sign-in to a subject the person already
  holds renews that account's record; where the LLM provider issues the
  renewal another client id, the replaced refresh token is revoked.
- **Told apart by the subject,** never the email: two sign-ins stay
  apart even when their emails match (providers.md, 2.2).
- **A subject has one holder.** A sign-in whose subject another holder
  holds is refused, naming the LLM provider's rule, and its new record is
  revoked: two holders would share one subscription, and race one
  rotating token.
- **A program's logins** are one configuration directory per source on
  each host, named by the source's name (`accounts-sources.md`, 1), each signed in on its
  own, with its own email, so one host may carry several Claude Code
  logins of one person.

### 3.4 Sources

Moved to [`accounts-sources.md`](accounts-sources.md), section 1.

### 3.5 States

Moved to [`accounts-sources.md`](accounts-sources.md), section 2.

### 3.6 Bounds

Accounts in the deployment, accounts per holder, sources per account,
sign-ins at once, lifetime program source names (retired names included),
and program sources per worker are limits
(`../protocol/limits.md`), within which the accounts'
records, skein's driver, the hub's grants, a worker's hello and the
secrets' store are sized. A request that would pass one is refused,
naming it. A move out or in holds one of the sign-ins at once from its
ask to its end (`accounts.md`, section 6), as a sign-in and a key being kept do, so
that the child's tables for them share that bound. Client receiving
counts for accounts, sources and sign-ins
are at least the engine's document lines, so every list cut there fits
(`../client/domain/client/README.md`, section 11).

A restored set past its limit, lowered since it was written (the
accounts, one account's sources, the source table as a whole, or the
lifetime program source names), or a stored name past its byte bound,
refuses the start, naming the record's key, the set and the limit
(`engine/restart.md`); the refusal stays until the next cold start,
whatever rows follow it. An account a source names that is not held, or
a name reserved by two sources, is not a limit but a link the start
cannot make, and refuses the start as `Failure::Link`, naming the
source's key and what it links to (`engine/restart.md`).

`valid(&Limits)` checks that `accounts * sources` is representable,
alongside the individual bounds. This sizes the retained source table;
it does not reserve an output queue's room. Internal outputs are
uncapped and grow with the admitted work.

### 3.7 An account's list and what a provider says

Moved to [`accounts-lists.md`](accounts-lists.md), section 1.

## 4. Uses: whose work an account pays for

Moved to [`accounts-uses.md`](accounts-uses.md), section 1.

## 5. What people do

Each is a party's request through the client (people/acts.md, 1), checked
against the account's holder (3.2) and decided by the accounts child,
or, for the deployment's key added and account removed, the
deployment's act a connector makes in its owners' name (people/acts.md, 3);
each answer and each state is a fact the application's pages show
(`../client/domain/contract/documents.md`), which a party watches
among their own (`../client/domain/contract/watches.md`, 1.1).

| Request | What it does |
|---|---|
| sign in | starts an LLM provider's sign-in, cancelling first the party's own still waiting with that provider for the same holder (one whose address a finish has not yet had taken), which ends *cancelled* and frees its place in the bound at once, so that starting again gets a port the provider fixes (`../protocol/llm.md`, 5.4); the visit goes to the client that asked, and the redirect to a loopback listener the engine binds for this sign-in; makes an account, renews one of the same subject, or is refused for a subject another holder holds (3.3); refused before it starts for the deployment where the provider's sign-in brings a plan, since only a person holds one (3.2) |
| finish a sign-in | the address the sign-in's browser was sent to, which its person pasted, for a browser on another machine than the engine's: carried as a key is, and handed to the sign-in as its redirect (`../protocol/llm.md`, 5.4); only from the party who started that sign-in, while it waits; answered *finishing* once the sign-in takes it, the sign-in then ending as any does; refused, the sign-in still waiting, as another address, another sign-in's state, or *too many* once it has refused a few for their state; refused *not waiting* when no sign-in of the party's under that number waits, whoever's it is, once it has taken a redirect, ended, or the engine restarted, and while another finish of it is out; a sign-in that ends with a finish out answers it *not waiting* before its end |
| sign out | revokes the record at the LLM provider, or has each worker sign a program's login out, and removes the source; when revocation is not confirmed, the answer says to disconnect the application in the provider's settings (providers.md, 2.3) |
| add a key, replace it, remove it | adding names the account it joins, a new key's account by default; a person's, or the deployment's by its owners, or added by the deployment's `AddKey` (people/acts.md, 3) |
| declare a program's login | makes the account, or adds the source to one, under the name its holder gives (`accounts-sources.md`, 1); each host then reports whether its login is usable, and the answer names the application's command to run there, which hands the terminal to the program's own sign-in (smith's `protocol/programs.md`, section 4); the name of the holder's own removed source is declared again (`accounts-sources.md`, 1); refused *name taken* for a name a live source holds or another holder's reservation, and *not offered* where the LLM provider offers the holder no program's login |
| check a program's login | has each host's half check the source's login now (`accounts-sources.md`, 1) |
| log a program in through a lent terminal | asks a terminal the person's client lends to run the application's command for a name, and at its exit 0 declares that name, or checks the person's source of it, as the two rows above; cancelled by the person; ended by the client's report or its terminal's going (`terminal-login.md`) |
| order, uses, resume | the holder's: an account's place among theirs and its sources' order, its uses, a limited source resumed; resuming a source that is not limited is refused *not offered* |
| remove an account | signs each of its sources out, as sign out does, and erases its keys; its number is never reused, and a task that kept it chooses again (`accounts-uses.md`, 1.3); the deployment's also by the deployment's `RemoveAccount` (people/acts.md, 3) |
| move out | withdraws the account's signed-in source and hands its record to the client that asked, once, never in a committed answer, naming the move; sent once, never kept by the client (`../client/domain/contract/requests.md`, 1); refused "not signed in" without a signed-in source holding a record (`accounts.md`, section 6) |
| written | the client wrote the moved record: erases the source, and the account with it when no source is left; refused "cancelled" for a move not under way |
| move in | takes a moved record carried as a key is: refused "unreadable" for a file that does not decode, "not ours" for another application's, "held" for a subject another holder holds; makes the account or renews the holder's of its subject, refreshing the record once before it is usable (section 6) |

- **A value is never committed.** A key, or a moved record, is
  taken out by the clients' protocol layer as it decodes the request,
  kept in the secrets' store first, and handed to the core by a drawn
  number and its digest. An LLM key, or a moved record, is then adopted
  into its source's secret row before it is granted (`../protocol/store/secrets.md`).
  Requests and their answers are committed and kept (engine/decisions.md, 1.4);
  a value is not. A client sends a request carrying a value once, and
  never keeps nor resends it
  (`../protocol/client/README.md`, section 3, and `../protocol/client/small-store.md`, section 1). A moved-out record goes
  the other way once, with the answer to the client that asked, and is
  never in the answer kept (`accounts.md`, section 6): the request is sent once, and a
  second answer under its key, without the record, is *lost* to the
  client.
- **The sign-in's redirect is on the engine's machine.** The engine
  binds a loopback listener on an ephemeral port for each sign-in, or
  on the port its configuration fixes (`../protocol/llm.md`, section 5),
  as an LLM provider that requires a loopback redirect allows
  (providers.md, 2.2). A person whose browser is on another computer
  finishes the sign-in with the address that browser was sent to
  (above), or signs in on a computer of their own whose record a move
  then brings in (section 6).
- **A pasted address is a value too,** held, never kept: in memory,
  for as long as its request takes, and erased whatever its answer
  (`../protocol/store/secrets.md`). It holds the sign-in's state and
  its code, but no token or verifier: the code is spent once, and only
  with the verifier the engine holds. Only its starter may hand it
  in, since a state or a sign-in's number alone grants nothing.
- **Plan usage declined** keeps the sign-in and marks the source
  refused, and the answer offers its two ways on: sign in again with
  the scope, or another source (providers.md, 2.2).
- **A sign-in settles with its source's first answer.** One that makes
  or renews an account ends there, as its party is told (made,
  renewed), and leaves the sign-ins under way and their bound; but the
  party's accounts stay unsettled (`../client/domain/contract/documents.md`,
  1) until the component's first answer for the source it brought: its
  first generation granted, or a failure (`accounts-sources.md`, 2), or
  that source's going first (sign out, a renewal replacing it, its
  account removed, the close). So a party that waits for *settled*
  reads the source's state, never a source still waiting for its
  grant: the grant is the component's, answered after the sign-in ends,
  and a read between the two would show a made account no run can yet
  use. A refused sign-in brings no source and settles as it ends. What
  is awaited is kept with the source's number and the party who asked,
  in memory, and forgotten at restart, as the sign-in is.
- **The terms' disclosures are facts:** whether an account is a plan's,
  its LLM provider's name and manage-usage address, the first use of a
  plan, so that the application can show them where its pages need them
  (providers.md, 2.6). The words and screens are the application's.

## 6. Moving an account

A signed-in source moves between deployments of one application, each
side by its holder's request: out of one deployment into a private
file, and from the file into another. A rotating refresh token is valid
in one place (providers.md, 2.3), so the move keeps one holder by
construction, never by a person deleting a copy.

- **Signed-in sources only.** A key is added again; a program's login is
  per host. Moving out an account with no signed-in source, or whose
  signed-in source holds no record that reads (it needs sign-in, or is
  unloaded), is refused "not signed in".
- **Out, in two steps.** Its holder asks to move the account out. The
  child takes the move's number from its counter, and withdraws the
  signed-in source: it releases its grant, and the component takes it
  out of the driver, so that no run takes it and no refresh follows; an
  exchange already sent finishes and keeps its record first, and live
  runs keep their access token until it lapses. The source is held
  *moving*, stored. Once the protocol layer has read the source's
  latest record and made the file's bytes (`../protocol/llm.md`, 5.6),
  the request is answered *moving out*, naming the move's number; the
  bytes go to the client that asked with that answer, once, never in a
  committed one: an answer lost on its way, or the request's key sent
  again, leaves the move with no file, and its deadline cancels it.
  The client writes the file new and private, and says
  *written*, naming the move: the child then erases the source and its
  record, and the account with them when it has no other source; its
  number is never reused. The file is then the only holder.
- **Cancelled.** A move with no *written* within the move's deadline (a
  limit, `../protocol/limits.md`), or under way at a
  restart, is cancelled: the source comes back to the driver with its
  kept record, which is **refreshed once before it is lent again**, so
  that a file written meanwhile holds a dead refresh token. The source
  stays held moving until its first new generation is granted, so a
  restart before then refreshes it again; a refresh refused as sign in
  again leaves it needing sign-in. A *written* of a move that is not
  under way is refused "cancelled", and the client removes the file it
  wrote. A move whose record cannot be read is refused "not signed in",
  and its source comes back the same way.
- **In.** Its holder sends the file's bytes as a carried value, taken
  into the secrets' store as a key is (section 5). The protocol layer
  reads them and checks that they are this application's: its name, and
  the issuer and client id of a configured LLM provider's registration;
  otherwise the request is refused "not ours"; bytes that do not decode
  as a moved file are refused "unreadable" before any of these checks.
  The child then decides
  as for a sign-in (3.2, 3.3): it refuses the deployment where the LLM
  provider's sign-in brings a plan, and a subject another holder holds
  ("held"), else makes the account, or renews the holder's account of
  that subject. A move in counts as a sign-in at once (3.6). The record
  is adopted into the new source's row, the carried value erased, and
  the source added to the driver, held moving: **refreshed once before
  it is usable**, so that a file left behind is dead at once. The
  request is answered once the record is durable, naming the account;
  the client then removes the file.
- **While a source moves,** its holder may order it and set its
  account's uses, but its sign-out, its account's removal, another move
  out, and a renewal of its account by a sign-in or a move in are
  refused "moving", until the move is written or cancelled.
- **One holder at a time.** A rotating refresh token is valid in one
  place: a copy that refreshes signs the other out (providers.md, 2.3),
  so a record is moved, never copied.
- **It serves three needs:** a host with no browser, signed in on a
  computer with one (providers.md, 2.2); a deployment moved to another
  machine; and unattended tests of the live layer on a signed-in
  subscription, which move one account in and out of each run's fresh
  deployment. A test on a program's login finds it signed in on its
  host (`accounts-sources.md`, 1).

## 7. Open questions

- **Reasons for key and sign-in refusals** (section 5): a key's add or
  replace and a sign-in's start or finish can be refused not offered
  for several causes in the core. Distinguishing each cause for a
  client's reason and next step needs finer account refusals and their
  client mapping (`../client/domain/contract/README.md`, section 9). Default:
  the existing refusal; a declaration's not offered and a resume's have
  the single meanings in section 5.

- **An account's other facts:** kinds 3 to 6 (`accounts-lists.md`, 1) wait for each provider's
  adapter to learn them (`../draft/account-facts.md`, section 6); kind 7 is
  said (temper's `draft/usage-windows.md`). The user is still deciding on
  them; the format holds them. Default: nothing says them.
- **A stored default that reads as named:** a store from when Create
  recorded a task's default holds that default as a choice, so its task
  waits for it as named until it is amended through its chooser or closed.
  Default: no migration before v0.1.0,
  as the request `first-usable-model` of 2026-11-06 allows; a fresh box.
- **Pinning the model a task took:** a person who picks the model a task
  given no choice already shows sends no change, since the client's draft
  equals what it shows, so the task stays free to take another when it
  next chooses (`accounts-uses.md`, 1.3). Default: as it stands; picking another names it.
- **Replay across accounts:** whether a provider's opaque replay
  (encrypted reasoning, thinking signatures) is accepted from another
  account of the same LLM provider. Default: dropped (`accounts-uses.md`, 1.3), until a
  capture shows it is accepted.
- **A redirect relayed by a client,** for an engine on another machine:
  the native client binds the loopback listener and hands the code to
  the engine. A remotely hosted application asks OpenAI first
  (providers.md, 2.1). Default: the engine's machine (section 5).
- **Who decides for the deployment's accounts** beyond its first owners:
  whether every owner in the deployment's roles (people/README.md, section 3)
  does, not only those its configuration names. Default: the first
  owners (3.2).
- **An occurrence whose account no longer serves:** whether a recurring
  template's occurrence is refused, rather than made to wait for its
  account. Default: made, its run waiting (`accounts-uses.md`, 1.3).
- **`Domain::select`'s arguments** (`accounts-uses.md`, 1.3): a public seam kept whole, past
  clippy's count, since other crates read it and three of its arguments are
  borrowed slices, which no owned struct may hold (programming-model.md, 10.3).
  Default: as it stands, until its signature changes for another reason.
- **Program logins on a shared host,** once a run's process tree is
  contained from the programs directory: whether the rule of `accounts-sources.md`, 1 then
  goes, or stays for a program's own tools. Default: the rule holds.
