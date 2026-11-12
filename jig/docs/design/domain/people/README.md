# Parties

Provisional, 2026-10-07, revised 2026-11-03. How people, and the
services that act like them, take part: who they are and the credentials
they sign in with, what their role in a project lets them do, the
operations they ask of the engine and what waits for them, the
conversations they hold with agents, the tasks they carry out
themselves, and how they hear of what waits. Parties are requesters and
executors, as tasks are (../model.md, section 9); this document is that
party in depth. The client they use is jig's, `jig-client`, which an
application holds under its own presentation (`../client/domain/`). The
LLM accounts a person holds, which pay for runs, are ../accounts.md's. What
is still open is listed in section 11.

## 1. In one page

- **A party is a person or a service.** People create tasks and talk to
  them, accept proposals, answer questions, steer and watch through a
  client. A service does what its role allows through the same
  operations, with no presentation.
- **Known by a credential jig mints.** jig mints, verifies, expires and
  revokes credentials. The printed link and services' credentials are its
  own ways of signing in; any other is the application's sign-in, which
  ends in a `Mint` on the application's server's word. The core keeps who
  a party is, as a provider and a subject; it does not act on any system
  as them.
- **Roles are per project,** the core's own, possibly seeded from what a
  connector knows of people on an adopted resource. A role is an
  authority and the proposals it may decide (../authority.md, section 6).
- **Requests are jig's operations.** Create a batch, propose, withdraw,
  message a task, amend, cancel with or without a grace, release, stop,
  move a task to oneself, decide, answer, take and hand back, mark seen:
  each is checked against the party's role as an agent's call is against
  its task's authority, committed, and answered once durable. Ordinary
  deciding acts retain their original answers under their keys for the
  advertised retention; `Seen` retains no new answer and advances a read
  position only forward (acts.md, 1, and README.md, 5). A compound request holds
  several, committed all or none.
- **Proposing is explicit.** A request beyond the party's authority is
  refused, naming what it lacked and whether a proposal could be
  accepted; `Propose` asks.
- **The deployment's acts** are what a connector asks of the core in
  the deployment's name, where no client is there to ask: a person's
  conversation closed, the deployment's key added or account removed,
  a resource adopted, a project made or deleted. No party makes them: a
  connector the configuration grants each kind, while it names a first
  owner, keyed as a party's request and answered to it once durable
  (`acts.md`, 3).
- **A project can be deleted,** by its owners or by the deployment's
  act: closed at once, its tasks ended (its conversations first asked
  to finish within the grace the request gives, as a close asks one)
  and then erased whole, then its own rows, its adoptions released,
  each step kept until the deletion is whole (`acts.md`, 2).
- **A party's inbox** holds what waits for them: questions, proposals,
  escalations, the tasks they carry out, results and the turns of their
  conversations. What is addressed to a role waits for every party in it
  until one acts.
- **A chat is a template,** jig's default for talking to an agent: an
  agent's task flagged conversation and kept, filled in by the client and
  closed by cancelling it with a grace. The task carries its flags; the
  engine never expands or checks a template.
- **The client knows the plumbing:** these operations, tasks as a tree,
  what waits for a party, and live runs; not the shapes of work built
  from them, which are templates and helpers above it. An application
  adds its own kinds of object.
- **Anyone may read a public project,** without signing in, as one
  party fixed in code whose role grants public reads alone: a few typed
  documents of structure and mechanics, with no text but tasks' titles
  and the project's name, named by opaque identifiers, within a budget
  behind every signed-in request. Projects
  are private until their owners make them public (`public-reading.md`).

## 2. Structure

The people child domain is a child of the core (../engine/README.md, section 3), a
capability. It keeps:

- **parties:** the identities of those signed in or holding a role, and
  their sign-ins, each named by its number;
- **roles:** each live project's roles and who holds them, loaded at
  start and changed by commits;
- **deletions:** each project being deleted, the step its deletion
  has reached and the grace it gives its conversations (`acts.md`, 2), until its
  last;
- **requests in flight:** each party's pending requests, and each
  connector's pending acts in the deployment's name (`acts.md`, 3), by their
  keys, until answered, a compound request as one;
- **inboxes:** for each party with something waiting, what waits, by
  reference to the task or proposal it is, and their read positions over
  results and over live conversations (section 5).

`outputs_bound(&Limits)` counts the most outputs one step may make.
The core's settings route reserves this allowance at the store's commit
edge, alongside the settings writes. It is an edge count: the people
child's output queue is uncapped, and the count reserves no internal
hand-off or drain.

It knows nothing of tasks beyond their numbers and what the core tells
it waits for whom; every request goes to the core, which routes it to
authority and to the tasks. Parties' watches of their `me` and their
inbox are the views child's (../engine/views.md).

## 3. Parties and identity

- **An identity is an identity provider and a subject.** An identity
  provider, a *provider* in this document and in the configuration's
  `providers`, is one of the application's ways of signing in, numbered
  in its configuration (`../../protocol/configuration.md`, section 3): an
  OAuth server, a company's directory, a chat platform's identity, or
  the deployment itself, for services' credentials. The subject is the
  provider's stable id for the party; a login and a name come with it,
  for display only.
- **An LLM provider is no identity provider.** OpenAI, Anthropic or a
  local model's server is configured and numbered apart
  (`../../protocol/configuration.md`, section 3), and signs nobody in to
  jig. An LLM account's subject is the LLM provider's for the account,
  never a party's identity: signing an LLM account in changes no one's
  identity, and one person may hold several (../accounts.md, 3.1 and 3.3).
- **jig owns credentials.** It mints a credential for an identity,
  verifies each one presented to its party's sign-in, expires it after a
  configured time, and revokes it when its party signs out, or when an
  owner reissues or removes a service's (below). A sign-in is committed
  by the core, which gives it its number; the credential's digest is
  kept by the clients' component in the store's secrets
  (`../../protocol/store/secrets.md`), so a restart signs nobody out.
  The domain names a sign-in only by its number, and says each that ends
  or expires, so the component erases its digest.
  A sign-in expires when either its original monotonic lifetime or its
  saved wall expiry is reached. Both clocks make its people timer due;
  a backward wall correction never extends the original monotonic
  lifetime. Creation and restoration arm both expiry conditions; timer
  scheduling translates the saved wall expiry at the current supplied
  clocks (../root.md, section 3; ../engine/views.md).
- **Three ways of signing in are jig's own:**
  - *the printed link,* on one person's machine: the clients' component
    draws a one-time secret at start, which the application's startup
    prints in a URL of its own; presenting it signs in the configured
    first owner it names, and spends it (`../../protocol/client/sign-in.md`
    ). An application enables it by configuration, on loopback
    only;
  - *a service's credential* (below), presented to sign in as that
    service;
  - *a sign-in approved by a browser:* the clients' component holds an
    approval a client asked for until a browser signed in by its cookie
    approves it, and at its take asks people `Whose` the approving
    sign-in is (below) and signs that identity in afresh, as the printed
    link does, held none (`../../protocol/client/sign-in.md`). People
    sees an ordinary sign-in of the approver's person, counted, full and
    expiring as any; an approver whose sign-in has ended is no signer,
    and the take signs no one in. Nothing of the approval reaches
    people before the take.
- **A sign-in held is kept.** A person signing in by the printed link
  while presenting the credential of a sign-in of their own keeps that
  sign-in, rather than making another beside it:
  - *the clients' component authenticates it* as it does any exchange's
    credential, its digest under its number and its saved expiry
    (`../../protocol/client/sign-in.md`), and names that sign-in to the
    core as *held*; one unknown, ended or expired is held by no one,
    and signs nobody out;
  - *people decides whether it is the same person's:* a held sign-in
    whose person is the one the identity signs in, live by both its
    clocks, is the sign-in's answer, with its own number and its saved
    expiry, and nothing is written. Anything else (no sign-in held,
    another person's, one past either clock) signs in afresh, as
    without one;
  - *it extends nothing:* the kept sign-in's expiry stays as it was
    saved, and its credential is the one the person holds, so none is
    drawn, kept or handed out;
  - *before capacity:* the held sign-in is checked before the
    deployment's sign-ins are counted, so signing in again by a fresh
    link never fills them;
  - *the numbers drawn go unused:* the candidate person and the sign-in
    number the core drew for the exchange are kept by no one, and the
    core's next commit saves its counters past them, as after a sign-in
    refused as busy. A gap in the numbers is harmless: they name, and
    are never reused or counted; capacity counts the sign-ins held;
  - *checked once, by people:* a held sign-in that ends or expires
    after the clients' component checked it, before people answers, is
    no longer live there, and the person signs in afresh.

  Only the printed link offers a held sign-in: a `Mint`, a service's
  sign-in and an approval's take sign in afresh.
- **Full sign-ins say so.** The deployment holds at most
  `limits.people.sign_ins` sign-ins (`../../protocol/limits.md`
  ). A sign-in that would make one more is answered *full*,
  apart from busy:
  - *what it carries:* that limit and the earliest saved expiry among
    the sign-ins held, the wall moment by which one will have ended
    unless its person signs out sooner. No person, number or count: the
    printed link's holder, the deployment's first owner on its own
    machine, and an approval's taker learn when the next room comes,
    and nothing of whose it is; a `Mint` and a service learn only busy
    (`../../protocol/client/sign-in.md`);
  - *after the held sign-in:* a person keeping their own sign-in (above)
    is never answered full;
  - *only while every sign-in is live:* the deployment's sign-ins are
    counted as held, and one past either clock is held until its timer
    ends it. A sign-in that finds them full while one of them is past
    either clock ends that one there, the one its timer would end next,
    as the timer would (its digest erased, its end said), and takes its
    room: ending what has expired is no eviction. So full is answered
    only while every sign-in is live by both clocks, and its earliest
    expiry is always to come;
  - *it waits for an end:* no wait of seconds frees it, only a sign-in
    that ends: its person's sign-out, its expiry, or an owner's removal.
    Nothing ends, shortens or grows to make room;
  - *nothing is written,* and the numbers drawn go unused, as for busy.
    Full is an answer, never saved, as busy and not ready are not.

  *Busy* stays for what passes or is the deployment's own: a new identity
  while the people table is full, and the bootstrap's holdings while it
  seeds. Neither meets the printed link past bootstrap: its first owner
  is not new.
- **Every other way is the application's sign-in.** It runs on the
  application's own routes, redirects included, and verifies who the
  party is. Its server then gives the clients' component a `Mint` naming
  that identity: the provider's number, the subject, a login and a name.
  The core commits a sign-in for it, and the component answers with its
  credential, which the application carries as it chooses (a cookie, a
  bearer). jig takes the server's word for the identity, so a `Mint`
  comes from the application's server code, never from a client. It
  names a provider the configuration gives as the application's; one
  naming any other, the printed link's or the deployment's, is refused
  (`../../protocol/configuration.md`, section 3; `../../protocol/client/sign-in.md`
  ).
- **The domain sees an authenticated identity,** and the kind of party it
  is, never how it was verified. jig is given no identity provider's
  token: what the application's sign-in received is the application's,
  and jig acts on no system as a party (section 11). An LLM provider's
  record is an LLM account's, kept by the protocol layer and lent only to
  pay for runs (../accounts.md, 3.4; `../../protocol/store/secrets.md`).
- **Whose sign-in, for the OpenID provider.** The clients' component,
  as the deployment's OpenID provider (`../../protocol/client/openid.md`
  ), names a signed-in person to the deployment's other applications
  by their identity. It asks people whose a sign-in is: `Whose`, with
  the sign-in's number, answered with the signer, its party's identity
  and kind, or with none for a sign-in unknown, ended or past either
  clock. It is a read: nothing is written, and no limit is spent.
- **No acting for.** A credential is one party's own. A bot relaying a
  person's decision holds that person's credential, obtained through the
  application's sign-in, and the decision is the person's request,
  naming the revision the bot showed them.
- **A party record** is made the first time an identity signs in, and
  kept.
- **Services** are parties whose provider is the deployment: an owner
  makes one in a project, with a role, and jig mints its credential. A
  service makes the requests a person makes (`acts.md`), and its inbox
  is read by its program, not shown.
  - *Its identity* is the deployment and the service's number, recorded
    when the service is made, so its sign-ins find it; a sign-in of the
    deployment's naming no service makes no party.
  - *An owner reissues its credential:* a new one replaces it, and the
    sign-ins made with the old end. This is how an owner who lost the
    credential, or fears it leaked, gets one again; the party, its role
    and its history stay.
  - *An owner removes it:* its credential and its sign-ins end, and its
    party is kept, as every party is, for the history that names it.
- **The first owners** of a deployment are named in its configuration;
  everyone else gets a role from a project's owner, or by adoption's seed
  (section 4). The first owners decide the deployment's LLM accounts
  (../accounts.md, 3.2).
- **The deployment's roles** are a roster of their own, kept as
  project 0's: a party's role in the deployment, on which the
  deployment's scope is admitted (`../../client/domain/contract/README.md`, 5.5).
  - *The first start* makes each first owner its owner, whether or not
    configuration seeds a project. It is seeded then only: a later start
    reads it from the store, and seeds it into no store that lacks it
    (`../../protocol/configuration.md`, section 7).
  - *Project 0 is no project.* Configuration names none
    (`../../protocol/configuration.md`, section 3), and a party's projects,
    in their frame, in `me` and in an account's readiness, leave it out:
    it is read only as the party's role in the deployment, `me`'s
    `deployment_role`.
  - *Its roster counts* against people's `projects`, as a project's
    does, and never against authority's: it has no policy.
  - *Its owners make projects* (`acts.md`, 1 and 2), and hold no role in a
    project by it: a project's roles are its own roster's, which a first
    start fills with the first owners because configuration names them.

## 4. Roles

Each project has its roles, and its policy says what each may do
(../authority.md, section 6). The default roles:

| Role | May |
|---|---|
| owner | everything a maintainer may; rename the project; change the project's policy and roles; adopt resources; make services |
| maintainer | create and steer any task in the project; order its goals; decide any proposal within the project's ceiling; release, cancel, stop |
| member | create tasks, chats and goals among them, within the authority and allotment their role gives; steer and cancel what they started; answer what is addressed to them |
| observer | read and watch |

- **A role is an authority** (../authority.md, section 3): what a party in
  it may give a task they create, how much of the project's spend they
  may allot, and which kinds of proposal they may decide. The policy may
  narrow or widen each, and add roles of its own (the person on call),
  within the deployment's rules.
- **Seeded at adoption.** When a resource is adopted, its connector may
  say which parties it knows there and with what permission (a
  repository's collaborators, a team's members). The project's policy
  maps permissions to roles, and those parties are given them, unless
  they already have a role in the project. The party who made the project
  is its owner. After that, roles are the core's own: a permission that
  changes on the system does not change a role.
- **Proposals past the root** of a tree go to the role the policy names
  for that kind of proposal (../tasks/proposals.md), maintainers by
  default.
- **Who holds escalations.** A held task's escalation that passes every
  task above it reaches the party who requested its tree, when their
  role decides escalations, and past them the project's escalation role,
  if its policy names one (../authority.md, section 6; ../tasks/proposals.md
  ). A party may be the requester of a root only when one of
  the two holds it (`acts.md`, 1).
- **`public` is no project's role.** It is the public party's, fixed in
  code, with no number, so no policy can name it (`public-reading.md`, 1).
- **The deployment's sole owner** is one read: the party holding `owner`
  on the deployment's roster when no other party holds it, else none:
  with no owner, with several, or when the roster can't be read, as after
  a failed start. It answers whenever a project's members can be read,
  during a start's restore too, and allocates and changes nothing. An
  application that rests a fact on a deployment of one person reads it
  again at each start.

## 5. A party's inbox

What waits for a party, across every project they hold a role in, each
entry naming the task or proposal it is and its project:

- **questions** from tasks they asked for, conversations included,
  but a question a delegate asks a conversation, which stays with the
  conversation, which answers it or asks its person itself in its own
  words (../tasks/messages.md, 5);
- **proposals** routed to them, or to a role they hold;
- **escalations** that reached them: a held task nobody below could
  decide;
- **person tasks** addressed to them, or to a role they hold (section 7);
- **results** of the tasks they asked for, and the turns of their
  conversations they have not read.

- **Derived from the tasks,** not kept apart: an entry is there while its
  proposal is pending, its person task is active, its question is
  unanswered, or its result unread. The only state of the inbox's own is
  each party's read positions, one over results and one per conversation
  over its turns, moved by their `Seen` (`acts.md`, 1), never by a read. Each mark
  durably advances its position to `max(old, through)`. Processing checks
  the current subject and authority each time: a retry after completion
  may answer `Unknown` after history was forgotten, or `Role` after
  authority was lost, even after earlier success. It never recreates
  forgotten history or promises the earlier processing's answer. Each
  admitted caller and joined waiter gets one terminal reply, held behind
  the same commit barrier. Restart restores committed positions or their
  committed absence; new completed marks leave no retained answers.
  A question waits for the person until it is answered, not until its
  task has read it. The asked task keeps each question it heard and its
  first answer (`../../protocol/store/README.md`, 10.7). The person's inbox, a question's
  live document and an answer's admission read those lines, so a task's run
  reading a question leaves it open for its person. No task answers it
  for them: the asked task's own answer to a question that waits in its
  person's inbox is refused, naming that its person answers it there,
  so a conversation never answers for its person; the person's answer
  reaches the asking task as its requester's answer, and the asked task
  hears a notice naming the question answered, with the answer words cut
  at 256 bytes on a UTF-8 boundary and marked when cut (../tasks/lifecycle.md, 8). A
  delegate's question to a conversation waits with the conversation, not
  here (../tasks/messages.md, 5).
- **A conversation** is a task its creator flagged so (../tasks/README.md,
  section 3), whatever its charter, tracked or not (../tasks/changes.md
  ): its turns are entries for its requester, while that
  requester is a party, with their read position and an unread count.
  One position per conversation, its requester's: a move to another
  party erases the old requester's in the same decision, and starts the
  new one's with nothing read. No other task has a read position over its
  turns, so their number stays within the tasks flagged conversation,
  and those in memory within the live ones, one each.
  - *While the task is live,* its position is this child's, in a family
    keyed by party and task, loaded at start.
  - *When it ends,* the decision that ends it moves the position to this
    child's family of ended read positions, keyed by task and party and
    loaded on demand; the child holds none of them in memory, a read of
    the conversation or a task query's row joins it by a point read, and
    the inbox's census reads the family through at start
    (`../../protocol/store/README.md`, 4.4, and `../../protocol/store/loads.md`, 2 and 4).
  - *A mark on an ended conversation,* a `Seen` over its turns, loads
    its record and its position, and is decided on them: the position
    moves to the larger of the two, never back. One such mark loads at a
    time: while it does, the party's next `Seen` over that
    conversation's turns, or over results that would forget it, is
    answered busy (../engine/README.md, 5.3). Its loads are held behind the
    commits before them, so a mark sent after another was decided sees
    that one's position, and no mark saves a position over a newer one,
    or saves one for a task already forgotten.
  - *It is erased with the task:* by retention when the task is kept, or
    at once when it is forgotten once read.
- **A task's retention class** says what becomes of it once it has ended
  (../tasks/README.md, section 3):
  - *kept:* its records stay to the deployment's retention horizon, and
    whole while none is set (`../../protocol/store/README.md`, 6.1). jig's chat
    template keeps, so closed conversations stay readable;
  - *forgotten once read:* once it has ended, its requester has read its
    result and, for a conversation, its turns, and no live task names it,
    its record and its read position are erased in one decision, which
    tells every connector the task is erased (`../connectors.md`, 3.4), so
    no restart sees one without the other, and nothing read comes back
    unread. The `Seen` after which it is read forgets it, while the
    party's inbox holds it (the bound below), in the `Seen`'s own
    decision: the mark and the erase commit together, so no restart sees
    the task read and not forgotten. A `Seen` over results loads nothing
    for it: the inbox's entry for an unread result carries what its
    forget needs (the task's class, its label and listing, and, for a
    conversation, its last turn and whether its turns are read). A
    `Seen` over an ended conversation's turns loads its record and its
    position before it is decided, and forgets in that decision.
    Retention's next pass forgets the others: one the inbox no longer
    held when it was read, or one a live task named then. Its other
    remains go at retention's next pass (`../../protocol/store/loads.md`, 4, and `../../protocol/store/README.md`, 6.1
    ). A task whose requester is not a party is kept, whatever its
    class: no party reads it.
- **Unread counts.** A conversation's unread turns are counted from its
  requester's position and carried on its task line; `me` counts what
  waits for the party, and their unread updates
  (`../../client/domain/contract/documents.md`, 1).
- **Across projects,** since a party who works in several decides in one
  place; a client filters it by project, kind or root
  (`../../client/domain/contract/README.md`, section 4, and 5.1). It is not a task
  query: proposals, escalations and questions are entries, not tasks.
- **What is addressed to a role** is in the inbox of every party holding
  it, until one of them acts, when it leaves every one.
- **Bounded:** the people child domain holds each party's entries up to a
  limit; a client pages through the rest, loaded from the store. What
  needs the party is counted exactly, since it is derived from live
  tasks; updates unread are counted up to a limit.

## 6. Chats

- **A chat is a template,** jig's default for a conversation with an
  agent, sugar over the primitives (../model.md, 7.3): template 1, label 1,
  in the application's configuration, which an application keeps,
  changes or drops. Filled in, it is a `Create` of one agent task, its
  spec the party's first words, flagged conversation and kept, with the
  fields `../../protocol/configuration.md`, 3.1 gives template 1. Its
  executor is an agent under the application's charter for chats, which
  a project's policy may override: its instructions for chats, its
  models, the tools it gives, and a wake policy that wakes it on its
  requester's words and on its delegates' results.
- **The engine never expands or checks it.** The client fills the
  template, as `me` serves it merged with the project's policy's
  overrides (../authority.md, section 6; `../../client/domain/contract/requests.md`,
  6), into an ordinary `Create`, which the engine checks as one
  written by hand (`acts.md`, 1; `../../protocol/configuration.md`, 3.1). A client
  that ignores it makes only its own lists inconsistent.
- **Permissions stay primitive.** A party may hold a chat when their role
  may create with the chat charter, within its spend and depth, and their
  escalations have a holder (`acts.md`, 1); no permission names a template
  (../authority.md, section 6).
- **Turn by turn, live.** The party's words, each a `Message`, wake it,
  or reach its live run at once; its replies stream to the client as the
  run reports them, and each turn is committed as it ends (../engine/executors.md,
  2), so the conversation the client shows is the transcript in the
  store.
- **Waiting, then parked.** A chat waits for the next words holding its
  host's slot; idle past its charter's threshold it parks, and the next
  words resume it from its transcript, on any host its charter allows.
- **What a chat does:** answers; delegates; hands work to a procedure; or
  proposes a goal, which its requester accepts in the conversation, as a
  proposal in their inbox, and which is theirs once accepted, not the
  chat's (../tasks/proposals.md). It answers its
  delegates' questions, which stay with it, and asks its person a
  question of its own by a person task addressed to them, in their
  inbox (../tasks/messages.md, 5).
- **What it waits on** is the engine's, computed with the tree and the
  inbox (../tasks/README.md, section 3); **its status** (its phase, its run and
  what it waits on, read together) is a fact jig's helpers derive from
  its documents (`../../client/domain/client/helpers.md`), never the
  engine's state.
- **A party's chats** are a task query by requester and label 1, live
  and ended (`../../client/domain/contract/README.md`, 4.1), each line carrying its
  unread count (section 5).
- **Closing, through a helper.** jig's helper for chats, in `jig-client`
  (`../../client/domain/client/helpers.md`), composes one compound
  request (`acts.md`, 1): each live delegate cancelled or moved to the party, as
  they say, then the chat cancelled with the template's grace, which
  asks it to finish, as an amendment it is woken by, and cancels it if
  it has not ended when the grace ends (../tasks/changes.md). Any client
  may compose the same request itself; the engine knows no close.
- **Closed by the deployment,** where no client is there: a connector
  granted it makes the deployment's `Close` (`acts.md`, 3), the same close with
  every delegate cancelled, none moved, since no party says which to
  keep. The core composes it from the conversation's live delegates in
  the decision that admits it, and the application gives the grace,
  its template's: the engine still reads no template.
- **Closed with its project:** a project's deletion (`acts.md`, 2) closes each
  of its live conversations as the deployment's close does, every
  delegate cancelled, with the grace its `RemoveProject` carries. A
  client gives the chat template's, as its helper gives a close's
  (`shapes::remove_project`, `../../client/domain/client/helpers.md`),
  and an application deleting by the deployment's act gives its own
  chat template's (temper's playground: temper's `protocol/setup/playground.md`
  ): here too the engine reads no template.

## 7. Person tasks

A task whose executor is a party, or a role (../model.md, 3.2):

- **Made by** an agent's `delegate` (a choice between two reports), a
  procedure (an approval it waits for), or a party's `Create`.
- **Its contract** is a choice among the options its spec lists, an
  approval of a state its procedure names, or an answer in words; and,
  always, that it cannot be done, with a reason, which is a failure
  (../tasks/README.md, section 3).
- **An approval** is of the state as its procedure named it: bytes the
  core carries with the contract and never reads, and a part its
  connector shows the state by. The answer names the state it approves;
  declining is "cannot do it". It is admitted as any answer is, naming
  the person task's revision, and is refused as moved unless its state
  equals the contract's, byte for byte: the core compares the two, and
  never reads what they say. The result reaches the procedure that
  asked, as any delegate's does, and its connector reads the state.
- **Shown** with its spec, the results it depends on, and what its
  contract asks; the party's `Answer` is its result.
- **Addressed to a role,** it waits for any party holding it; one who
  takes it (`Take`) holds it, so two do not answer at once, and may hand
  it back (`HandBack`). A service may hold a role a person task is
  addressed to, and answer it through its requests.
- **Stalled,** past its bound, it is news for its requester, which may
  amend, cancel or address it to someone else.
- **Kept** by a dependency an amendment added (../tasks/lifecycle.md, 8), it stays
  in its party's inbox, shown waiting on its dependencies, and an
  `Answer` is refused, naming them; once they are done, it is answered
  as ever. A take stands while it is kept. A task that depends on a
  person task goes on once the person task is answered.

## 8. Notifications

- **In a client,** live: a signed-in party's inbox changes reach their
  open watches through the views, once their commits are durable.
- **Beyond the client:** a page to the person on call, a mail, a chat
  system's message, each as a connector of the application's whose
  effects are messages to parties, keyed and made once like any other
  (../connectors.md). The core says what waits for whom; the application's
  policy says which entries notify, and through which connector.

## 9. Below the domain

What the clients' component, on the engine's side, and the client below
its surface owe this domain:

- **Requests, reads and watches,** as jig's messages and the
  application's, carried to the component by the application's
  transport, which the component never sees
  (`../../client/domain/contract/README.md`, section 8; `../../protocol/client/README.md`).
- **Credentials:** minted on a printed link's secret, a service's
  credential, or the application's server's `Mint`; each presented one
  verified to its sign-in; a sign-in's digest erased when the core says
  it ended or expired. The domain names a sign-in by its number
  (`../../protocol/client/sign-in.md`).
- **Sign-in expiry timers** account for both clocks as section 3 says.
  A root omitting an otherwise empty closure still checks its current
  deadlines; refused admission leaves an expiry due (../root.md, section 3).
- **Turns** carried as the bytes the domain keeps, decoded below the
  client's surface by `jig-protocol-transcript`
  (`../../client/domain/client/conversations.md`).
- **Keys** for requests, made by the client, carried with each request
  (`../../client/domain/client/requests.md`).

## 10. The world

The people child domain has its world, with its parent scripted. In the
core's world (../engine/README.md, section 9), scripted parties sign in and act
through operations: a member creates a chat from jig's template, which
proposes a goal they cannot fund, routed to a maintainer who accepts it;
a member's `Create` beyond their authority is refused, naming what it
lacked and that a proposal could be accepted, and then proposed and
accepted; a party's root is refused where their role decides no
escalation and the policy names no escalation role; a person answers a
choice between two reports; two maintainers decide one proposal, and the
second is told; a person stops a run, then writes to its task and
releases it in one compound request; a conversation with two live
delegates is closed, one cancelled and one moved to its person, and
cancelled with a grace it finishes within, and another outlives its
grace; a compound request is refused at its second operation, and
nothing of it is made; a conversation ends, and its read position is
read back from the store after a restart; a task forgotten once read is
erased with its read position; an observer is refused a request; a
service creates a goal its role allows; an ordinary deciding request
sent twice is made once across a restart; roles are seeded from an
adopted resource's parties; an application's server mints a credential
for an identity it verified, and a mint naming a provider not of kind
application, the
deployment's among them, is refused; a person task given a dependency by
its requester, its answer refused naming it, then answered once the
dependency is done; a member's amendment adding a dependency beyond
their reach, refused; a person signing in again by a fresh link while
holding a live sign-in of their own keeps it, its number and expiry,
with nothing written, and signs in so more times than the deployment
has room for sign-ins; and one holding another person's sign-in, or
their own ended or expired between the clients' check and people's
answer, signs in afresh; a sign-in past the deployment's room is
answered full, with its room and its earliest saved expiry, nothing
written, while one holding their own live sign-in keeps it, and signs in
once a sign-in ends; a sign-in finding the room full while one sign-in
is past its wall expiry, its timer not yet run, ends that one and signs
in; a new identity past the people table's room is answered busy; a
connector granted the deployment's acts closes a person's conversation
with two live delegates, both cancelled and the conversation cancelled
with a grace it finishes within, removes one of the deployment's
accounts, adds a key as the deployment's, and adopts one of its own
resources; an act sent again under its key, after a restart between
its commit and its answer, is answered from its saved answer with
nothing made twice, and one the restart lost is made then; acts are
refused, nothing made, while the configuration names no first owner,
of a kind their connector is not granted, and on a subject outside
their act's: a person's account, a task that is no conversation,
another connector's resource; an owner's deletion of a project with a
live task is refused, naming its live roots, and a member's is refused,
nothing deleted; the deployment's act deletes a project with a live
chat and its delegates, with a grace, and a grace past the bound is
refused, nothing deleted; the delegates are cancelled and
the chat asked to finish, cancelled when the grace ends, then all
erased whole with their transcripts, its own rows erased and each
connector asked to release it until it answers, and only its spend
left in the period's totals; the deleted project refuses every
request, act and watch as one never made, but its read answers gone,
leaves every `me`, and its
name is taken again by a new project of a new number; a deletion cut
by a restart after any of its commits ends the same, with its grace; a
connector granted it deletes a project, and its act sent again is
answered removed while the deletion lasts and unknown after; an
owner's sent again is answered removed after it as while it lasts.

In jig's conformance world (../testing.md, section 5), a project is deleted
in a case for each family of jig's store, the core's, the tasks', the
people's, the notes' and the accounts', and for the application's,
each cut at every commit.

Its referee: every request is answered once, after its commit; a
compound request is made whole or not at all; no request is acted on
beyond the party's role, and none becomes a proposal unasked; no party
becomes a root's requester while its escalations have no holder; no
act in the deployment's name is made but by a connector granted its
kind, while a first owner is configured, on its act's subject; what
waits for a role leaves every inbox once one party acts on it; a
person's words reach their task, or the task ends; nothing read comes
back unread, across a restart or a task's end; a project being deleted
takes no new task; a deletion a restart cut is resumed; and, once a
deletion has ended, no row names its project or one of its tasks but
what outlives it (`acts.md`, 2).

## 11. Open questions

- **A person's sign-ins listed and each ended** (3), browser-approved
  ones among them: `../../protocol/client/README.md`, section 12. Approval between
  hosts also waits there; people still receives an ordinary sign-in.

- **Acting as a person:** whether an approval should also be posted on a
  system as the person who gave it, so that the system's own protection
  requires it independently. jig holds no provider's credential; an
  application's sign-in may keep one, and its connector would then need
  to know which party gave the approval.
- **Finer cuts among a project's parties:** whether a chat and its
  transcript are its requester's alone, or what an observer reads.
  Default: a role in the project reads all of it (`public-reading.md`, 4).
- **Roles that span projects,** as the inbox does.
- **Who deletes a project:** its owners, and the deployment by its act
  (`acts.md`, 2, 3). The deployment's owners over any project is later.
  Default: as said.
- **A deleted project's spend:** kept in the deployment's and its
  parties' totals for the period, never per task (`acts.md`, 2). Default: as
  said.
- **A public board's ended goals across a restart or before publication:**
  a store index of a project's ended goals, newest first, that the public
  index loads once at start and at publication (`public-reading.md`, 9). Default: none; the
  board's ended goals are those ended while the index kept the project,
  and its page states the publication and restart limits. A connector's
  kept record does not widen which tasks the index hands it (`public-reading.md`, 3).
- **A human publication capability:** a way for a person to publish
  another configured name they wrote, that an agent holding their
  credential could not use, since a credential proves who sent a
  request and not who wrote it (`public-reading.md`, 3). Default: none; public documents
  carry only task titles and the project's name as text.
- **Project 0 in the checks:** the deployment's roster is kept as
  project 0's (section 3), and no write makes a project 0, so each read
  that asks whether a project is open sets it aside first. Whether the
  deployment takes a value of its own, so that no check can forget it.
  Default: as said.
- **People's tokens and the core's flights:** a people request's token
  and a core flight's handle can share a value, so each crossing between
  them translates. Whether each takes a type of its own, so that a
  crossing left untranslated does not compile. Default: as said.
- **An omitted model:** an authority that omits its models translates
  to none in authority's terms, and that comes back as a stated none,
  so the two mappings are not each other's inverse; no decision
  composes them on an omitted value. Whether authority keeps an
  omission of its own. Default: as said.
