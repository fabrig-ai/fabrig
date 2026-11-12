# Requests

## 1. The keyed request

- **The key** is sixteen bytes, scoped to the party: the same key from
  two of their sign-ins is one request (`../../../domain/people/acts.md`, 1).
  The client draws it, or the application gives it as a name (below).
- **Answered once durable:** the answer waits for the commit of the
  decision it reports, as every output that someone outside could act on
  does (`../../../domain/engine/README.md`, 5.2).
- **Ordinary answers kept for a retention:** an ordinary deciding
  request's durable original answer is saved under its key, success and
  permanent routed refusal alike, and a request sent again within the
  retention gets it again. `me` says how long the retention is, so a
  client never sends a request again past it
  (../client/requests.md). Busy and not ready are never kept: the
  client sends the request again, under its key, but a request carrying
  a value (below).
- **Capacity is engine-wide:** retained answers plus pending ordinary
  requests' reserved answer places, never per party. Retained replay or
  conflict and pending joins or conflicts precede new admission. No
  unexpired ordinary answer is evicted to admit another key.
- **`Seen` keeps no answer:** a top-level `Seen` bypasses
  completed-answer capacity, using the bounded pending, flight,
  waiter and load places, with uncapped journal and output queues,
  without reserving an answer place. It saves no `Answer` on success or
  routed refusal. While pending, an identical request joins and a
  different ask conflicts as usual.
  Keeping no answer, its key is free after completion: a retry is
  processed against current subject and authority, and a different ask under that key is a new
  request (`../../../domain/people/acts.md`, 1).
- **One held key, one ask:** a key sent again with a different ask is
  refused as a conflict while pending or retained.
- **A name** is a key the application gives in place of one the client
  draws, for a request that comes from outside the application
  (sixteen bytes of a digest of an agent's key, say). The engine treats
  it as any key: a name sent again is answered as above, within the
  retention for an ordinary deciding request, and one sent with another
  ask under a pending or retained key is refused as a conflict,
  which for a name is no bug of the client's. The application builds a
  named request from its own inputs and `me` alone, never from a clock,
  so that the name sent again makes the same ask: a template's deadline
  goes as a span (2). A client holds one request under a name at a
  time: a `Send` under a name its request table holds is answered held
  at once, and nothing is kept or sent (../client/requests.md).
- **A compound request is one keyed request** (4): one key, one ask,
  one answer.
- **A request carrying a value,** such as an LLM account's key added or
  replaced (2), or an application's request's credential, is never
  kept by the client (../client/requests.md): it carries a name, goes
  once, and its caller asks again under that name. The
  engine keeps the value in its secrets' store before the domain sees
  the request, handing the domain only a reference to the value and
  its digest (`../../../protocol/client/README.md`, section 3).
  Its name is its caller's, never made from the value, so that a name
  carries no secret. It is never in a compound request.
- **A request whose answer carries a value,** an LLM account moved out
  (2), goes as one carrying a value does: never kept by the client,
  sent once under a name, never in a compound request. The value is in
  the first answer alone, to the exchange that asked; the answer kept
  for the key holds none, so a second answer under the name carries
  none, and the client answers it *lost* (`../../../protocol/client/README.md`,
  section 3).

## 2. jig's requests

The plumbing operations, by their names on the wire
(`../../../protocol/client/wire.md`, section 1): what each names, and what it
becomes. Their rules are the domain's (`../../../domain/people/acts.md`, 1
and 2; `../../../domain/tasks/README.md`, section 4 and `../../../domain/tasks/changes.md` and `../../../domain/tasks/proposals.md`). A party and an
agent ask the same operations and pass the same check, from different
sources: the party's role and pool, or the task's authority and what it
has left (`../../../domain/authority.md`, 8.1 and 8.4).

| Operation | Names | Becomes |
|---|---|---|
| `Create` | a project; a batch: for each task its executor (an agent's charter, a procedure and its parameters, a person, or a role, which is the project's: roles are per project, `../../../domain/people/README.md`, section 1), spec, result contract (a report, a change or a verdict: an approval is a procedure's, `../../../domain/people/README.md`, section 7), authority, whose spend and deadline are the task's budget and deadline (`../../../domain/authority.md`, section 3), that budget as named there or `AsAllowed`, that deadline a wall or a span (below), wake policy, or none for its executor's default (`../../../domain/tasks/messages.md`, 3), as an agent's delegate may leave it, dependencies and inputs, the grants named after it (`../../../domain/authority.md`, section 4), tracked and priority, label and flags, and its LLM choice (its main model with its effort, the models its run's sub-agents may use with theirs, and an account, any of them left out for its charter's default, and optionally `later`, a main model with its effort and sub-agents with theirs for every run after its first activation's: `../../../domain/tasks/README.md`, section 3); and the dependencies among its tasks. The other resources a task will write are those its connectors name from its executor and spec (`../../../domain/tasks/README.md`, section 4) | the batch checked whole and made at once (`../../../domain/tasks/README.md`, section 4), its tasks numbered consecutively in the batch's order, the first and the last answered, and the `AsAllowed` task, if any, with what it was given |
| `Propose` | a project; the action: a batch, or an amendment (a widening of a task's authority among them); a party makes no effect (`../../../domain/people/acts.md`, 1); or, in the deployment's scope, a change of settings, as `ChangeSettings` names it but for whether running work moves (`../../../domain/engine/settings.md`, 6); a reason | a proposal to whoever may accept it, its number answered (5) |
| `Withdraw` | the party's own pending proposal | withdrawn |
| `Message` | a task; the words | words in its inbox, their message number answered |
| `Talk` | a task that stands; the words | its words as a `Message`'s, and a talk opened or carried on (`../../../domain/tasks/standing.md`, 5.6), their message number answered |
| `Leave` | a task that stands | its talk ended, and its session (`../../../domain/tasks/standing.md`, 5.6) |
| `Amend` | a task; its spec's revision, or a person task's for its address; the change: spec and parameters, instructions, authority, wake policy, its dependencies as a whole list (those it names anew added, those it leaves out removed: `../../../domain/tasks/changes.md`), a person task's address, tracked and priority, its LLM choice, which its next run takes, replacing the task's whole, its `later` with it, so one without `later` leaves the task none; a reason | amended; an active task given a pending dependency kept until it is done (`../../../domain/tasks/lifecycle.md`, 8). A change of priority alone names no revision: the first commit decides |
| `Cancel` | a task; a reason; a grace, or none | it and its tree closing; with a grace, the task first hears that it is asked to finish, as it hears an amendment, and is cancelled when the grace ends unless it ended first |
| `Release` | a held task; its hold's revision; for an uncertain effect that cannot be recovered, whether to make it again | its hold lifted, judged afresh |
| `Stop` | a task | its run cancelled, the task held for the party: a placed hold of the party's, stop, over the task's subtree (`../../../domain/tasks/lifecycle.md`, 10) |
| `Hold` | a mode, `Stop` or `Drain`; a scope, `Subtree(task)` or `Project`; a reason | a placed hold, its number and revision answered (`../../../domain/tasks/lifecycle.md`, 10) |
| `Lift` | a placed hold by its number; its revision; `with_tasks` | the hold ended; with `with_tasks`, each task it held that nothing else holds released |
| `Move` | a task the party may amend; a reason; or a live task the party may amend as its new requester | the party its requester, its reservation moved to their pool (`../../../domain/tasks/changes.md`); or that task its requester |
| `Succeed` | an ended task in the party's tree, a live task in their tree | the live task the ended one's successor (`../../../domain/tasks/changes.md`) |
| `Decide` | a proposal, by its proposer and number: accept, for a proposal of settings with whether running work moves, reject with a reason, or pass up; or an escalation, by its held task and its hold's revision: release, leave held with a reason, or pass up | as `../../../domain/tasks/proposals.md` has it; or, with a retained decision visible to the party, decided first, by whom and how |
| `Answer` | a question, by its task and message, with the words; or a person task, by its revision, with the answer its contract asks (a choice, words, or the state an approval approves) or that it cannot be done | an answer to the question; the person task's result |
| `Take`, `HandBack` | a person task addressed to a role the party holds | held by them, or back to the role |
| `Seen` | how far the party has read: their results, or one conversation's turns | their durable read position advanced to `max(old, through)`; after completion a retry checks current subject and authority and may return `Unknown` after forget or `Role` after authority loss, never recreating history or replaying a completed answer (1; `../../../domain/people/README.md`, section 5) |
| `ChangeModel` | the deployment, its scope (`README.md`, 5.5); the act: a model's number and whether a picker shows it, or a provider and a name to add | one change of the `model` setting at the newest revision, running work staying, decided and answered as `ChangeSettings`, and never stale (`../../../domain/engine/settings.md`, 11) |
| `ChangeSettings` | the deployment, its scope (`README.md`, 5.5); the revision the party saw; the items, at most one per setting, each a setting (a charter, a template or a policy by number, the rules, an application's kind, scope and number, or a project's own policy by its project) and what becomes of it: set to a value whole, a charter's instructions and its parts' as text, each part's `on` as its reasons (`../../../domain/tasks/messages.md`, 6), or, for a charter with a default, set to an overlay over it (`../../../domain/engine/overlays.md`), removed, or reset to its default, a project's policy set alone and at most one, decided by that project's owners and, beside the deployment's items, by a party who owns both; whether running work stays or moves (`../../../domain/engine/settings.md`, 2) | the settings changed in one decision, the new revision's number answered; the deployment's `settings` and every `me` whose templates changed marked. Not among a compound request's operations (4) |
| `MoveToNewest` | the deployment, its scope (`README.md`, 5.5); a revision held | every live task on it that may move set to the newest revision in one decision; answered with the count of those moved and of those that could not, which finish on it (`../../../domain/engine/settings.md`, 4); a revision not held refused as unknown; the newest answered with none moved, nothing written |
| `Restore` | the deployment, its scope (`README.md`, 5.5); a revision the history keeps; a reason | a proposal of the settings as that revision left them, against the newest, its items jig's kinds, an application's and each project's policy a later revision set, its number answered, with what it leaves out: the settings and the policy of each project deleted since, by kind, project and number (`../../../domain/engine/settings.md`, 10); forgotten, naming the oldest kept; empty, for the newest; stale; too large |
| `MakeProject` | the deployment, its scope (`README.md`, 5.5); the new project's name (`../../../domain/model.md`, 6.1); the number of a policy among the settings, the newest's (`../../../domain/engine/settings.md`, 1) | a project, its policy a copy of that one, its maker its owner and only party (`../../../domain/people/acts.md`, 2), its number answered; the maker's `me` gains it. Not among a compound request's operations (4) |
| `RemoveProject` | a project, by its owner; a grace, or none, within the tasks child's bound, as a `Cancel`'s: the chat template's, as `shapes::remove_project` gives it (../client/helpers.md) | the project closed in the decision that admits it, answered removed with its number, marked deleted, and deleted whole in the steps that follow (`../../../domain/people/acts.md`, 2): every `me` loses it, its read answers gone (`README.md`, section 4), and every other request and watch on it is refused unknown, as on a project that never was. A project with a live task is refused *live tasks*, naming its live roots (`README.md`, section 3), and nothing deleted, so the grace finds no conversation to close. Sent again under its key, it is answered removed, while the deletion lasts and after it, within the answers' retention; a project already being deleted, under another key, is refused unknown; a grace past its bound is refused, a limit, and nothing deleted. Not among a compound request's operations (4) |
| `SetVisibility` | a project, by its owner; public or private | the project public or private from that decision (`../../../domain/people/public-reading.md`, 2); every `me` of its parties marked. Project 0, and a project being deleted, refused |
| the project's own | notes, by the revision recalled; its name (`../../../domain/model.md`, 6.1); the policy, its templates' overrides among them (6), and its ceiling, edited whole within the deployment's rules (`../../../domain/authority.md`, section 6), so that rules widened by a change of settings reach a project its owners widen; its spend per period, which resizes its open period (`../../../domain/authority.md`, section 7); roles, a role's spend per period alone (`RoleSpend`) among their edits, resizing its parties' open pools; a party's pool; services, a service's credential answered once; as `../../../domain/people/acts.md`, 1 and 2 | the note or the project changed |
| adopting a resource | the application's own request (3), its form the application's; the engine's root makes the people request on the party's behalf | the project changed |
| the party's LLM accounts | as `../../../domain/accounts.md`, section 5 has them: signing in with an LLM provider; finishing the party's sign-in under way with the address its browser was sent to, carrying it as its value (1), of at most the bytes `me` states (`documents.md`, 1); signing a source out; adding or replacing a key, carrying its value (1), of at most the bytes `me` states (`documents.md`, 1), and removing one; declaring a program's login under a name; checking a program source's login; reading a provider's lists again (`ReadLists`, a provider, reading each account of it the party may act on that has a list, answered with how many, the reads ending on the `accounts` watch; `../../../domain/accounts.md`, 3.7); an account's place among the holder's and its sources' order; its uses; resuming a limited source; removing an account, which signs each of its sources out and erases its keys; moving an account out, saying it *written* by the move's number, and moving one in, carrying the moved file as its value (1; `../../../domain/accounts.md`, section 6). The party's own accounts, or, for an owner, the deployment's | an account made, renewed or changed. A sign-in is answered done with the address to open in a browser, and ends on the `accounts` watch (`README.md`, 5.1), made, renewed or refused; a finish is answered *finishing* once the sign-in has taken the address, and the sign-in ends on the watch; a sign-out's answer says whether the provider confirmed revocation, and names each host that did not sign a program's login out; a program's login declared is answered with the source's name, by which the application's own command signs the program in on each host; a move out is answered *moving out*, naming the account and the move's number, with the moved record to write, once (1); *written* is answered done; a move in is answered *moved in*, naming the account and whether it renewed one |

- **A task's fields cross as data** (`../../../domain/tasks/README.md`,
  section 3): *tracked* and *priority*, which the engine acts on; a
  *label*, an opaque 32-bit number the application gives, to which jig
  gives no meaning and no rule of the engine's keys; and flags, each
  with engine semantics: *conversation* (its turns are entries for its
  requester, with their read position and an unread count), *listed*
  (once ended, it is found by its project and label, `README.md`, 4.1), and its
  *retention class*, *kept* or *forgotten once read*. A `Create` names them, and a
  task keeps them; documents carry them, with the two the engine keeps
  on every task: its *last activity* and what it is *waiting on* (`documents.md`, 1).
- **`AsAllowed`** is a budget given as "as much as the creator's role and
  pool allow at admission, capped by the role's spend." The pool's
  amount left is a live join no client can compute without racing, so
  the engine resolves it, and the answer says what was given. At most one
  task of a batch gives it; it is resolved after the batch's named
  amounts are counted, and refused as lacking budget where it would give
  nothing (`../../../domain/tasks/README.md`, section 4). It and a deadline given
  as a span are the plumbing templates need (6).
- **A deadline given as a span** is "this long after the task is
  made". The engine makes it a wall from the wall time of the decision
  that makes the task, a `Create`'s admission or the acceptance of the
  proposal that asks for it, as it checks the batch
  (`../../../domain/authority.md`, section 7), and the task keeps that
  wall. A request so carries no clock of its sender's, and is the same
  ask each time it is sent, under its key or its name (1).
- **Where a party may reach.** A party's `Create` may depend on a live
  task, or take an ended task's result as an input, in their tree, or
  any task of the project for a maintainer; its tasks' requester is the
  party. An agent reaches its
  delegates and the tasks it was introduced to (`../../../domain/tasks/messages.md`,
  5). An `Amend` that adds a dependency reaches as far as a `Create`
  would, and below the task it amends (`../../../domain/tasks/changes.md`
  ).
- **A party's root has someone to escalate to.** A party may create a
  root task only when its escalations have a holder: their role covers
  escalations, or the project's policy names an escalation role. It is
  refused otherwise, saying so (`README.md`, section 3), as a proposal nobody may
  accept is (5).
- **What stays asymmetric:** procedures decide nothing
  (`../../../domain/tasks/proposals.md`); only parties execute person
  tasks, stop runs, move tasks and run the project.
- **An approval names its state.** A person task's contract may be an
  approval of a state its procedure names, which its connector shows by
  a part (`documents.md`, 3). The answer names the state it approves; declining it is
  "cannot do it", with a reason. It is admitted naming the person task's
  revision, as every answer is, and refused as moved unless the state it
  names equals the contract's, byte for byte: the engine compares the
  state and never reads it. The result reaches the procedure that asked,
  whose connector reads it.
- **A watch is not a keyed request.** It changes nothing and needs no
  answer kept: it is admitted on the party's sign-in and role, and ends
  when its client leaves (`README.md`, section 5).

Authority payloads carry an optional model selector: ALL or an explicit
list of at most 64 unique, nonempty UTF-8 names, each at most 4,096 bytes.
The client may receive fewer bytes per name under its whole text bound;
an identity beyond that bound is refused whole. An empty list is explicit denial. Omission follows the Create, Amend and
role defaults in `../../../domain/authority.md`, 3. Both public authority
forms carry this selector. Authority documents also carry `cut`; only a
whole document can supply a replacement. Names are copied literally in
both directions, and current availability is checked after retained-key
lookup (`../../../protocol/store/records.md`, 1).

The program operations use the account-change request: declaration
carries the provider, optional joined account, and portable source name; check
carries the source number. Declaration returns that number and whole name. Check
answers when dispatched, while inventory facts arrive on the `accounts` and
`deployment` watches.
Sign-out and account removal retain the configured names of workers that failed,
were not in contact, or lost contact; replay preserves those original names.
Neither request carries a secret or digest.

## 3. An application's requests

A request about an application's own object (silence this alert, extend
this environment) is the application's (`../../../domain/people/acts.md`, 1):

- **keyed alike,** in the same key space, by the same request table in
  the client (../client/requests.md);
- **admitted by the people child** (`../../../domain/people/acts.md`, 1) by
  key, as jig's are: duplicates joined, the party's sign-in checked,
  and, by the scope the request names as a read or a subject does
  (`README.md`, 5.5), their role in its project, their role in the deployment, or
  nothing more for the party's own; a key sent again with another
  request is told apart by a digest of the request's encoding, which
  the protocol layer gives the domain with it;
- **refused at admission with jig's refusal:** a request the people
  child refuses there (its party's role in the request's scope, a key
  sent again with another request, a limit), or one whose sign-in has
  ended, is answered as jig's are (`README.md`, section 3) and never reaches its
  connector. The client hands that answer to the application under the
  request's key and owner, which shows it as a refusal, never as its
  object gone;
- **routed by the engine's root,** by the owner it names, to the
  connector it concerns, which decides it within the same decision.
  What it asks of the core on the party's behalf (an amendment, a task,
  a release) is judged as the party's own request would be, against
  their role and authority; what it changes of its own is the
  connector's to allow, by its policy's roles;
- **answered once durable,** once admitted, with an outcome of the
  connector's own: only an admitted request is the connector's to
  decide. The people child keeps the key and a token; the connector
  keeps the outcome under that token, durably, for the retention, and
  gives it again when the request is sent again, as the core asks a
  connector for any value it does not own (`../../../domain/model.md`,
  section 5; `../../../domain/connectors.md`, 5.5). The one exception: a
  connector whose own table of kept outcomes is full refuses the
  admitted request at once, by the decision the people child reserved
  for it (`../../../domain/people/acts.md`, 1): a `Limit` of kind
  `ApplicationAnswers` naming the connector and its bound. The people
  child keeps that refusal itself, durably, as it keeps its admission
  refusals, and gives it again for the key, after a restart too, for
  the retention; the connector keeps nothing of it. A later ask under
  the same key gets that refusal, even once the table has room: a new
  ask takes a new key;
- **never its own acts:** a connector acts on the core on a party's
  behalf only within that party's request. What it asks of the core
  with no request, where no client is there (a person's conversation
  closed at an end no client saw, an act a restart left undone), is the
  deployment's act (`../../../domain/people/acts.md`, 3): granted to it by
  configuration, keyed in its own key space, and answered to it, never
  to a client.

## 4. A compound request

- **Some decisions are made of several operations,** committed together
  or not at all: closing a conversation whose delegates are live, each
  delegate cancelled or moved to the party and the conversation then
  cancelled with a grace; reordering several tracked tasks at once;
  amending a held task and releasing it; words to a task the party
  stopped, and its release.
- **One keyed request, `Compound { project, operations }`:** a list of
  operations, admitted in order and committed in one decision, all or
  none. It has one key and one answer: done, with each operation's
  outcome in order (below); or refused, naming the first operation that
  failed, by its place in the list, and why.
- **What it may hold:** `Cancel`, with or without a grace; `Move`;
  `Amend`, priority included; `Release`; `Message`; `Stop`. Placing
  and lifting holds, creating,
  proposing, withdrawing, deciding, answering, taking, handing back,
  marking seen and the project's own are requests of their own.
- **Admitted as the people child admits it** (`../../../domain/people/acts.md`,
  1): in order, each operation as it would be alone after those
  before it, with one pool counted across them; every operation's task
  in the request's project; at most *operations per request*, refused
  at admission past it as a limit (`README.md`, section 3), enough to close a
  conversation with every delegate live, or reorder every goal of a
  project, in one request (`../../../protocol/limits.md`
  ).
- **One check is the wire's own:** each operation on the wire names a
  project, and the people child's operations name none, since their
  compound names it once. So the clients' protocol layer compares the
  two numbers, before anything else of the request and before a later
  operation fails to decode, and answers one that differs refused, as
  malformed, naming its place: a request that disagrees with itself,
  which never reaches the people child. Whether each operation's task
  is in the request's project stays the people child's.
- **Revisions, as alone:** each operation names its revision where it
  would alone (`documents.md`, 2), so a `Release` names its hold's beside words or an
  amendment; a priority-only `Amend` names none, the first commit
  deciding; a `Message` names a task and words, and releases nothing.
- **Each operation's outcome: made, with a message number.** Its
  operations take consecutive message numbers, one each, in the
  request's order, and the answer names the first: the operation at
  place *i* took the first plus *i*. A `Message`'s words have that
  number in its task's inbox, as a lone `Message`'s answer names
  theirs. What another operation adds to its task's inbox, if anything,
  has its number too; one that adds nothing leaves its number unused.
  So a client finds each operation's number by its place alone, without
  knowing what each kind adds.
- **Nothing more of an operation is answered,** as nothing more is when
  it is alone. What it changed (a grace's end, a spec's revision, a
  requester and the reservation moved, a hold lifted) is in its task's
  documents, as after the operation alone, and a later decision names
  the revision a document gave (`documents.md`, 2).
- **Its worst case** is `../../../domain/engine/README.md`, section 7's.
- **A client never chains requests** for what the person sees as one
  act, so it never has to undo the first when the second is refused.

## 5. Proposing is explicit

- **A request beyond authority is refused,** for a party as for an
  agent. A `Create` or an `Amend` whose authority, budget, deadline or
  grants pass what the party may give alone is refused as lacking
  authority: what it lacked and by how much, and whether a proposal of
  it could be accepted (`README.md`, section 3); to whom it would go, the offer says
  (`documents.md`, 3). Nothing is proposed on the party's behalf.
- **`Propose` asks.** The party proposes the action, which waits for the
  roles the project's policy names for its kind
  (`../../../domain/tasks/proposals.md`). Where the policy names none that
  may accept it, it is refused at admission as lacking authority, saying
  that no proposal could be accepted: never a proposal that waits for
  nobody (`../../../domain/people/acts.md`, 1). A proposal once made, or one a
  task makes, waits as `../../../domain/tasks/proposals.md` says.
- **A party's proposal is the party's:** its proposer is the party, an
  amendment's as a batch's, so its decision, and a rejection's reason,
  reach them.
- **Why:** a person is never asked by accident, the reason
  `../../../domain/engine/executors.md`, 3 gives for agents, and a service is a
  program. Proposing when beyond is sugar: a helper that sends
  `Propose` where the offer says a draft is beyond the party (`documents.md`, 3).

## 6. Templates

- **A template is a recipe for a task of one shape:** the defaults that
  turn "a chat with these words" into a complete `Create`. It is data,
  numbered and never renumbered, in the application's configuration.
  What each carries, jig's two defaults (chat, template 1; goal,
  template 2), and what a project's policy may override, are
  `../../../protocol/configuration.md`'s, 3.1, with their JSON in its
  section 10.
- **The engine never expands or checks one.** It keeps the
  configuration's templates and each policy's overrides only to serve
  each project's merged templates in `me` (`documents.md`, 1). The client fills a
  template into an ordinary `Create` or `Propose`, a deadline's span
  sent as a span (2), and the engine checks that request as it checks
  one written by hand: a template grants nothing, and a client that
  ignores one only makes its own lists inconsistent, while authority and
  limits still hold.
- **Permissions stay primitive,** with no "may use template", and a
  task keeps the values it was made with
  (`../../../protocol/configuration.md`, 3.1).
