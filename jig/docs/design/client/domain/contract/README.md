# The client and the engine

Provisional, 2026-10-09, revised 2026-10-28. jig's API: what crosses
between a party's client and the engine, at the level of the two
domains. It says who the party is; the plumbing operations a party asks,
alone or several in one compound request, each keyed and answered once
durable; the templates a client fills them from; the answers; the
documents the engine gives, `me` and the task line among them, with the
acts they offer and what a decision will do, as data; the reads, task
queries and watches that bring them; an application's traffic on the
same terms; and versions. It is to clients what `../../../domain/hosts/README.md`,
section 2 is to hosts: the contract both sides keep, whatever carries
it. Its messages and their bytes are `../../../protocol/client/README.md`'s.
`README.md`'s section 7 lists what the engine's domain owes the client, and where
`../../domain/` designs it. What is still open is listed in section 9.

## 1. In one page

- **Plumbing, in general form.** The API is jig's primitives as
  operations: create a batch of tasks, propose, withdraw, message,
  amend, cancel (with a grace), release, stop, move, decide, answer,
  take and hand back, mark seen, and the project's own (`requests.md`, 2). No
  operation, document or subject names a shape of work: a chat and a
  goal are tasks with fields (`../../../domain/model.md`, section 9).
- **Keyed requests, answered once durable.** Everything a party does that
  changes something is a request, keyed by its client, checked against
  the party's role, committed, and answered only once that commit is
  durable. Ordinary deciding acts sent twice under a retained key are
  made once and answered alike; `Seen` advances a read position only
  forward and retains no answer (`requests.md`, 1; `../../../domain/people/acts.md`, 1).
- **Several operations, one decision.** A compound request holds a short
  list of operations, admitted in order and committed together or not
  at all, under one key with one answer (`requests.md`, 4).
- **Proposing is explicit.** A party's create or amendment beyond their
  authority is refused, naming what it lacked and whether a proposal of
  it could be accepted; `Propose` asks. Nobody is asked by accident
  (`requests.md`, 5).
- **Templates are sugar the client fills in.** The application's
  configuration defines them, a project's policy may override their
  defaults, and `me` serves them merged. The engine never expands or
  checks one: it checks the request a template becomes as it checks any
  other (`requests.md`, 6).
- **Decisions name what they saw.** A request deciding about something
  that changes names the revision of that thing as the client was given
  it: a hold's, a task's spec's, a person task's, a note's (`documents.md`, 2). The
  engine refuses one whose revision has moved, saying what it is now.
- **Refusals carry facts:** what was lacking and by how much, which limit,
  what the object is now, how it ended, whether a proposal could be
  accepted (`answers.md`, section 1).
- **Documents per owner.** The engine gives the client typed documents
  about one subject each: jig's carry jig's values, and name an
  application's values as parts, which the application's own documents
  carry (`documents.md`).
- **`me` says who the party is:** their projects and roles, what waits
  for them, the templates each project offers, and the bounds a draft is
  compared with (`documents.md`, 1).
- **Documents say what the party may do, and what a decision will do.**
  Each object's document carries the acts the engine would admit from
  this party now: made, proposable (and to whom), or refused, with
  bounds where the answer depends on what the party writes. What a
  decision will do (what accepting grants and costs, what cancelling
  ends) is data, which the application shows wherever it confirms (`documents.md`, 3, 4
  ). A client never computes authority; the engine checks again when
  the request comes.
- **Lists are task queries.** One record, the task line, for every list
  of tasks; a query by a fixed set of keys and orders, within a window;
  live tasks scanned, ended ones found through the store's indexes
  (`reads.md`, 1.1).
- **Reads decide nothing, and show only what is durable.** A read is
  answered from the engine's live state or a page of its store, needs no
  key, and its answer waits until every commit it reflects is durable, so
  a party never sees state a crash would lose.
- **Watches are level-triggered.** A watch delivers its subject's current
  document whenever it has changed since the last delivery, one delivery
  in flight. Nothing is queued, so nothing is missed: a slow client sees
  fewer deliveries. A run's live text is the one stream, read from the
  offset the client has (`watches.md`, section 1).
- **An application's requests and documents** cross on the same terms and
  the same messages, kept by the people child by key like jig's, each
  naming its owner by its number (../root.md, section 15), routed by the
  engine's root to that owner, and each read and subject naming its
  scope (a project, the deployment, or the party's own) and its window
  (`watches.md`, 1.5).
- **Versions per owner,** a version not served refused with a typed
  answer (section 6).
- **jig's promises stay below the API,** whatever the presentation: a
  decision names the revision it saw (`documents.md`, 2); an ordinary deciding request
  is made once, and every request is answered once durable (`requests.md`, 1); a
  document reflects only committed state (`reads.md`, section 1, `watches.md`, section 1.2); admission and
  authority are the engine's (`requests.md`, 2, 5);
  a credential is its party's (`README.md`, section 2). Confirming a deciding act is
  the application's, with the facts jig gives as data (`documents.md`, 4); keeping a
  credential from being used across sites is the application's server's
  (`README.md`, section 8).

The contract's documents include:

- [`answers.md`](answers.md): outcomes and refusals.
- [`reads.md`](reads.md): reads, task queries and windows.
- [`watches.md`](watches.md): subjects, watches and streams.

## 2. Parties and signing in

- **Who:** a party is known by an identity: a provider's number and a
  subject there, with a login and a name for display
  (`../../../domain/people/README.md`, section 3). The domain sees an
  authenticated identity, and the kind of party it is.
- **jig owns credentials, not sign-in flows.** It mints a credential for
  an identity, verifies a presented one to its party, expires it, and
  revokes it, a party's sign-out included. A credential is its party's:
  a request presented with a valid one is that party's, and everything
  after is checked by their role and authority.
- **Its built-in ways of signing in:** the printed link's one-time
  secret, on a loopback listener, for one person's machine; the asked
  link, a fresh one a person's signed-in client asks for that person's
  browser, signing in the asker's own party no longer than the asker; a
  service's credential, issued when an owner makes the service; and a
  sign-in approved by a signed-in browser, for that browser's own
  party: a client asks, the browser, by its cookie alone, approves the
  ask's code, and the client takes the sign-in with the ask's secret,
  which alone makes it, on a loopback listener too
  (`../../../protocol/client/sign-in.md`).
- **Every other way is the application's sign-in:** its own flow, on its
  own routes, redirects included (an OAuth server, a company's
  directory, a chat platform's identity), which verifies the identity
  and then asks the engine, from the server's side, to mint a credential
  for it: a `Mint`, refused unless the identity's provider is of kind
  application (`../../../protocol/configuration-engine.md`, section 1). The
  application decides how the credential travels to its client.
- **jig adds no "acting for."** A program relaying a person's decision
  (a bot on a chat platform) holds that person's own credential, from
  the application's sign-in, and its request is the person's, naming the
  revision it showed them.
- **A sign-in is numbered,** and every request, read and watch carries
  its credential below the domain, which hands the domain the sign-in's
  number; the engine refuses one whose sign-in has ended, and the client
  reports signed out to the application (../client/sign-in.md, section 1).
- **`me`** (`documents.md`, 1) says who the party is, so a client never decides it
  from what it kept.

## 3. Answers

Moved to [`answers.md`](answers.md), section 1.

## 4. Reads

Moved to [`reads.md`](reads.md), section 1.

## 5. Watches

Moved to [`watches.md`](watches.md), section 1.

## 6. Versions

- **Each owner's documents and requests are versioned,** jig's by jig and
  each application's by the application, and every message names its
  owner and its version (`../../../protocol/client/versions.md`, section 1).
- **The engine serves a range of each owner's versions,** which the hello
  of its stream states first, then `me`, at the version of jig's the
  stream's opening names (`../../../protocol/client/wire.md`, section 2);
  `me` carries no range.
- **A version not served is refused with a typed answer,** *version not
  served* (`answers.md`, section 1), naming the owner and the ranges the engine
  serves, with nothing read, made or kept; a read and a watch in it
  likewise. What the client then does with that owner's requests, reads
  and watches, and when it sends what it kept, is ../client/watches.md's, 2;
  what the application says of it is its own (the reference UI's:
  ../pages.md, section 8).
- **One jig pin.** In an application, `jig-client` and the engine come
  from one jig pin, so they agree on jig's records unless the engine was
  updated under a client still running. A change to the plumbing moves
  jig's wire version, and before a release each range is one version
  (`../../../protocol/README.md`, section 7).
- **How jig is released and pinned,** and how a change of jig's reaches
  every application, is `../../../README.md`'s, section 14.

## 7. What the engine's domain owes the client

What `../../domain/` owes this contract, each designed where named:

1. **The plumbing operations of `requests.md`, 2 for parties:** a `Create` of a
   batch, person tasks among its executors; `Propose` of every kind of
   action, and `Withdraw`; `Cancel` with a grace, bounded by a limit of
   the tasks child's; `Move`; `Seen` over a conversation's turns; a
   release, an amendment and a person task's answer naming the revision
   they decide, a priority-only amendment naming none; an approval
   naming the state it approves; an uncertain effect's release saying
   whether to make it again (`../../../domain/people/acts.md`, 1;
   `../../../domain/tasks/README.md`, section 4 and `../../../domain/tasks/changes.md` and `../../../domain/tasks/proposals.md`).
2. **The compound request** (`requests.md`, 4): the people child's limit on
   operations per request; operations admitted in order, one pool
   counted across them, committed in one decision or refused naming the
   first that failed; done, answered with the first of the message
   numbers its operations took, one each in order
   (`../../../domain/people/acts.md`, 1); and the tasks child's limit of
   goals per project, the live tracked tasks of one project, refused at
   the entrance as live tasks per project are (`../../../domain/tasks/README.md`,
   section 6).
3. **Proposing explicit** (`requests.md`, 5): a party's create or amendment beyond
   their authority refused, saying what it lacked and whether a proposal
   of it could be accepted (answers.md, section 1); a party's proposal naming the
   party as its proposer (`../../../domain/people/acts.md`, 1;
   `../../../domain/tasks/proposals.md`).
4. **A party's root has someone to escalate to** (`requests.md`, 2;
   `../../../domain/people/acts.md`, 1).
5. **A task's fields** (`requests.md`, 2): its label, its conversation and listed
   flags, its retention class, its last activity, and what it waits on,
   computed; and `AsAllowed`, resolved at admission and answered
   (`../../../domain/tasks/README.md`, section 3).
6. **Read positions over conversations,** kept by the people child only
   for tasks with a conversation, and those of ended ones in the store
   (`../../../domain/people/README.md`, section 5; `../../../protocol/store/keys.md`, 1.4).
7. **Templates served, never expanded** (`requests.md`, 6): the configuration's
   templates and each policy's overrides, kept only to serve each
   project's merged templates in `me`; an override naming no template,
   or one another override of the same policy names, refused when the
   policy is changed; a retained LLM choice incompatible with the
   effective executor refused at startup, policy change and restore, the
   store against the configuration (`../../../domain/engine/restart.md` and `../../../domain/engine/views.md`, section 1
   ;
   `../../../protocol/configuration-templates.md`, 1).
8. **Credentials minted on the server's word:** a sign-in committed for
   an identity the application's server verified, its provider of kind
   application (section 2; `../../../domain/people/README.md`, section 3).
9. **An application's keyed requests,** admitted and kept by the people
   child as jig's are, told apart by a digest, judged as the party's when
   they ask the core for something, their outcomes kept by their
   connector for the retention (`requests.md`, 3; `../../../domain/people/acts.md`, 1;
   `../../../domain/connectors-facts.md`, 1.5).
10. **Refusals with facts,** as `answers.md`, section 1 lists them
    (`../../../domain/people/acts.md`, 1).
11. **Revisions** of holds, specs and person tasks, in their documents
    and in the decisions that name them (`documents.md`, 2 lists them;
    `../../../domain/people/acts.md`, 1).
12. **Acts offered** in each object's document, made, proposable or
    refused, creating and amending with bounds, reordering in a
    project's document by the party's role alone, and a decision's
    effect as data (`documents.md`, 3, 4; `../../../domain/engine/views.md`, 5).
13. **`me`, and the inbox across projects:** the party's entries in
    every project they hold a role in, needs counted exactly and updates
    up to a limit (`../../../domain/people/README.md`, section 5).
14. **A task's activity kept in commit order,** including each party's
    words as written, so that a conversation reads whole after its words
    are taken (`../../../domain/engine/executors-transcripts.md`, 1).
15. **The reads of `reads.md`, section 1,** task queries among them (`reads.md`, 1.1), their
    answers held until the last commit made is durable
    (`../../../domain/engine/decisions.md`, 1.2 and 1.3).
16. **Level-triggered views** with the subjects of `watches.md`, 1.1, what marks them
    changed, task queries by their keys among them (`watches.md`, 1.2), a run's text
    by turn, retry and offset, and an application's subjects through
    the same watches, each naming its scope, the party's own among them,
    and its window (`watches.md`, 1.5; `../../../domain/engine/views.md`). The
    views child keeps, per watch, its subject, its window, whether a
    delivery is in flight and whether its subject changed since; the
    subject's owner (the core, or a connector through the root) computes
    each document when a delivery is made, and the delivery is held as a
    read's answer is.
17. **A watch is not a keyed request** (`requests.md`, 2; `../../../domain/people/acts.md`,
    1).
18. **The party's LLM accounts** (`requests.md`, 2, `documents.md`, 1): the accounts' requests
    admitted from parties, a sign-in answered with its address and
    ended on the `accounts` watch, a key named by the request that
    carried it, the accounts' bounds and the providers' rules refused
    with their facts, and each account's sources, states, uses and the
    terms' facts in the party's accounts and the deployment's documents,
    with the *usable* and *settled* facts of the first, marked as `watches.md`, 1.2
    says (`../../../domain/accounts.md`, section 3, and `../../../domain/accounts-uses.md`, section 1, and `../../../domain/accounts.md`, section 5;
    `../../../domain/people/acts.md`, 1).
19. **A task's LLM choice** (`requests.md`, 2): given at `Create`, changed by
    `Amend` for the next run, or, where it gives none, taken at each
    run's check, the first of the charter's models it can use
    (`../../../domain/accounts-uses.md`, 1.3), within the authority's models; the
    models a party may choose
    in `me` (`../../../domain/tasks/README.md`, sections 3 and 4 and `../../../domain/tasks/changes.md`;
    `../../../domain/authority.md`, section 3).

## 8. Below the domain

What the protocol layers owe this contract, which the client's protocol
designs (`../../../protocol/client/README.md`):

- **jig's messages, transport-agnostic.** The API crosses as jig's
  messages: a keyed request, a read, opening and closing a watch, a
  stream gone, signing in and signing out, each answered once, each
  naming its owner and version, each with the credential it came with
  as opaque bytes; and one stream per client, carrying every delivery
  tagged with its watch (`../../../protocol/client/wire.md`, section 2). The
  clients' component, `jig-protocol-engine-clients`, takes them and
  answers them, and knows nothing of HTTP, paths, cookies, hosts or
  server-sent events.
- **The transport is the application's:** HTTP, a local socket for a
  command line, or an in-process channel for one process or a world.
  jig ships a default HTTP mapping, both ends, with example guards,
  which an application uses if it suits it, wraps, or replaces
  (`../../../protocol/client/default-http.md`).
- **The credential is checked below the domain:** the component verifies
  it against its digest and hands the domain the sign-in's number.
  Signing in is jig's built-ins and `Mint`, the application's server's
  word (section 2; `../../../protocol/client/sign-in.md`).
- **A value is taken out below the domain:** a key a request carries is
  kept in the secrets' store by the clients' component before the
  domain sees the request, which names it by the request alone
  (`../../../protocol/client/README.md`, section 3).
- **`jig-client-protocol-wire`** encodes jig's messages and documents
  for both ends, from one schema, with their versions; each
  application's own wire crate does the same for its own.
- **Turns** cross as their bytes, inside activity lines, and are decoded
  by `jig-protocol-transcript` in the client's link.
- **What the application's server keeps:** a credential from being used
  across sites, with jig's example guards and attack stories, as it
  keeps it out of its logs (`../../../protocol/browser/README.md`); its own pages
  and bundle, which jig never serves (`../../../protocol/service.md`).

## 9. Open questions

- **Reasons within account acts' not-offered answers** (`answers.md`, section 1):
  key add or replace and sign-in start or finish can reach this answer
  through several core causes. Carrying each cause needs finer account
  refusals and their translation (`../../../domain/accounts.md`, section 7).
  Default: the existing refusal. Declarations and resumes keep the
  single meanings in `answers.md`, section 1; clients can word those from their ask.

- **Browser-approved sign-ins beyond loopback** (2), and a person's
  sign-ins listed and each ended: `../../../protocol/client/README.md`, section 12.
  The built-in approval here signs in the approving browser's own party.

- **Refusal paths within charter parts** (`requests.md`, 2): a malformed part in a
  settings change is refused at its charter's request item as out of shape
  or past its count. Naming the part and its `on` member needs a refusal
  vocabulary beyond the charter-document paths. Default: the item's
  position and the existing shape or count refusal; configuration keeps
  its own part paths (`../../../protocol/configuration-engine.md`, section 1).

- **Decisions on replaced task proposals** (requests.md, 2, and answers.md, 1): automatically
  replacing a task's pending settings proposal keeps no decision record.
  Default: `Unknown`, with no new decider or archive. Returning who decided
  first and how for that proposal needs retention and a rule for naming
  the automatic withdrawal's decider (`../../../domain/tasks/proposals.md`).

- **Proposal objects on committed calls** (`documents.md`, 1): naming the proposal in a
  call's `objects` needs its identity carried through the core's turn record
  and the archived turn, and a mapping from call position to proposal number.
  A verdict or procedure proposal has no call. Default: the activity line
  names the proposal and its making independently of any call; filling call
  objects is separate work.

- **Historical public boards** (section 10): ended goals across a restart
  or from before publication need the source index in
  `../../../domain/people/README.md`, section 11. Default: the current public records,
  projected from that document's memory-only index (13.9); no history is
  reconstructed by the client contract.
- **Windows of a large tree:** how a tree watch's window follows what a
  person opened, so that a tree of hundreds of tasks costs what is shown.
- **Finer cuts among a project's parties**
  (`../../../domain/people/README.md`, section 11).
- **Search** over tasks, results, notes and transcripts, which the engine
  has only over notes' descriptions.
- **A document past its bounds:** a read whose answer cannot be encoded or
  decoded ends malformed (`reads.md`, section 1). Whether the engine instead derives bounds
  so it cannot happen, as a path's segments are, or takes a client's bounds to
  page within them. A watch's opening is refused as unknown and an oversized
  delivery ends its watch (`watches.md`, 1.2); a watch's answer of its own remains open.
- **A compound's project on the wire** (`requests.md`, 4): each of its operations names a
  project on the client wire, and the clients' protocol layer refuses one other
  than the compound's before the ask goes further. Whether people refuses it,
  which takes a project in each of its `Operation`s and a refusal its stored
  enum has no variant for, or the check stays as the wire's own shape
  (`jig/docs/development/debt/D026-engine-clients-project-mismatch.md`). Default:
  it stays.

## 10. Public documents

What the public party reads (`../../../domain/people/public-reading.md`),
each a record of its own in jig's wire, never a cut of another
document: a field is in it only once added there, and the wire's golden
shows every addition in review. Read by `PublicRead`
(`../../../protocol/client/public-reads.md`); no watch, no stream. Text is
limited to a task title and a project name; no record holds an internal
number: a project and a task are named by their public identifiers
(`../../../domain/people/public-reading.md`, 8),
`PublicId`, 64 bits.

| Kind | Names | Answered with |
|---|---|---|
| *projects* | nothing | `PublicProjects`: each public project's `PublicProject { project: PublicId, name }`, within the deployment's projects, and the `max-age` public answers carry |
| *board* | a project | `PublicBoard`: the project's `PublicId` and name; its live public goals by priority within the board's bound, and the count of live public goals past it; then its ended public goals the index holds, most recent first; each a `PublicGoal`: its `PublicLine`, its tree's public lines below it in tree order within the bound per goal, and the count of public lines past it |
| *card* | a project, a public task of it | `PublicCard`: the task's `PublicLine`, its lineage up to its public goal as `PublicId`s, nearest first, within the tree's depth; its public delegates' lines within the bound, and the count past it |

- **`PublicLine`:** `task`; `title: text 256`, the spec's first line cut
  at 256 bytes on a character boundary, and `title_cut: bool`, true
  when cut; `parent: Option<PublicId>`, none for a
  public goal; `goal: bool` and `priority: Option<u32>`, a goal's;
  `executor: PublicExecutor`, its kind alone: agent, procedure,
  person or job (`documents.md`, 7); `phase: PublicPhase`, jig's phases (`documents.md`, 1) as unit
  variants, an ended task's saying how it ended, `done`, `failed` or
  `cancelled`, and
  `hold: Option<PublicHold>`, a held task's hold's kind as a unit
  variant; `made`, `changed`, `active: Option`, `ended: Option`, each a
  time to the second; `spent: Option<u64>` in the deployment's unit, a
  goal's, none for a task; no budget;
  `delegates: PublicCounts`, its public delegates counted by phase
  class.
- **Schema ceilings:** projects 4,096; live and ended goals 256 each;
  tree lines and delegates 256 each; lineage 64; application tasks
  1,024 and record bytes 16,777,216; project and board names 64 bytes.
  `PublicIndexBounds` supplies the projects, goals, ended, lines and
  depth limits. `public_limits` refuses a bound past its ceiling;
  `public_bounds` returns the generated largest encodings for projects,
  board and card, or `u32::MAX` for each when bounds are invalid.
- **`PublicApplication { tasks, record }`:** an application's public
  document in jig's envelope (`../../../protocol/client/public-reads.md`):
  the `PublicId`s of the tasks it names, then its record in its own
  wire, which names a task only by its place in `tasks`.
- **Every field's source,** and nothing else:

  | Field | Source | Made by |
  |---|---|---|
  | a project's `PublicId` | the project's number | the component's permutation (13.8) |
  | `PublicProject.name`, `PublicBoard.name` | the project's record | as is, within 64 bytes |
  | `title`, `title_cut` | the task's spec's first line | cut at 256 bytes on a character boundary, saying whether cut |
  | a task's `PublicId`, `parent`, `lineage` | the task's and its ancestors' numbers, up to its public goal | the same; `parent` none at the public goal |
  | `goal`, `priority` | whether the task is tracked, and its priority | as is |
  | `executor` | the executor's kind | the kind alone, never who or which |
  | `phase`, `hold` | the task's phase and, ended, its ending; a held task's hold's kind | the kind alone, never its reason |
  | `made`, `changed`, `active`, `ended` | the task's times (`../../../domain/tasks/README.md`, section 3) | to the second |
  | `spent` | a goal's spend, with what its delegates spent | as is; none for a task |
  | `delegates`, a folded count | the public index's public tasks | counted after filtering (`../../../domain/people/public-reading.md`, 7) |
  | `max-age` | `clients.public.max_age` | as is |
  | `PublicApplication`'s `tasks` | the numbers the connector listed | the component's permutation |

- **Nothing else crosses:** no spec beyond its first line, instructions,
  inputs, result, words, question or answer, turn, note, reason, label's
  meaning, proposal, escalation, name of a charter, procedure, role or
  party, account or resource, a task's spend or any budget.
- **A schema of their own:** jig's public records are a schema apart
  in `jig-client-protocol-wire`, `public-v1.schema`, versioned apart
  from its other records, naming only one another and scalars, never a
  record of another schema, so a field added to a private document
  never reaches a visitor. A test reads the schema and refuses a name
  from elsewhere.
- **Its golden is the whole reachable wire shape:** a text file beside
  the schema lists every record and enum reachable from the kinds'
  answers, each field with its type, tag and default, and each enum's
  variants with their tags and payloads, which a test compares with
  the schema. Any change of what a visitor can receive changes it, and
  shows in review.
- **Its tests check meaning, not shape alone:** each projection is
  compared with an expected record built independently from the
  source table above, from a state whose forbidden sources (the spec
  beyond its first line, another name, a reason, words, an internal
  number past the identifiers' range) hold distinct markers that no answer's bytes may
  contain; and mutation tests show the golden failing on a nested field
  added to a copy of the schema, the closure test failing on a name
  from elsewhere, and the projection test failing on a field filled
  from the wrong source.
- **An application's public documents** are its own records in its own
  wire, beside these, under the same rule, carried in
  `PublicApplication`.
