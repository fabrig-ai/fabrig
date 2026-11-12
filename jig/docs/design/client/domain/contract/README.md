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
  accepted (section 3).
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
  (4.1).
- **Reads decide nothing, and show only what is durable.** A read is
  answered from the engine's live state or a page of its store, needs no
  key, and its answer waits until every commit it reflects is durable, so
  a party never sees state a crash would lose.
- **Watches are level-triggered.** A watch delivers its subject's current
  document whenever it has changed since the last delivery, one delivery
  in flight. Nothing is queued, so nothing is missed: a slow client sees
  fewer deliveries. A run's live text is the one stream, read from the
  offset the client has (section 5).
- **An application's requests and documents** cross on the same terms and
  the same messages, kept by the people child by key like jig's, each
  naming its owner by its number (../root.md, section 15), routed by the
  engine's root to that owner, and each read and subject naming its
  scope (a project, the deployment, or the party's own) and its window
  (5.5).
- **Versions per owner,** a version not served refused with a typed
  answer (section 6).
- **jig's promises stay below the API,** whatever the presentation: a
  decision names the revision it saw (`documents.md`, 2); an ordinary deciding request
  is made once, and every request is answered once durable (`requests.md`, 1); a
  document reflects only committed state (section 4, 5.2); admission and
  authority are the engine's (`requests.md`, 2, 5);
  a credential is its party's (section 2). Confirming a deciding act is
  the application's, with the facts jig gives as data (`documents.md`, 4); keeping a
  credential from being used across sites is the application's server's
  (section 8).

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
  secret, on a loopback listener, for one person's machine; a
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
  application (`../../../protocol/configuration.md`, section 3). The
  application decides how the credential travels to its client.
- **jig adds no "acting for."** A program relaying a person's decision
  (a bot on a chat platform) holds that person's own credential, from
  the application's sign-in, and its request is the person's, naming the
  revision it showed them.
- **A sign-in is numbered,** and every request, read and watch carries
  its credential below the domain, which hands the domain the sign-in's
  number; the engine refuses one whose sign-in has ended, and the client
  reports signed out to the application (../client/README.md, section 9).
- **`me`** (`documents.md`, 1) says who the party is, so a client never decides it
  from what it kept.

## 3. Answers

Every request has one answer, of one of these kinds:

- **done,** with its outcome: what was made (the first and the last of
  the tasks' numbers, consecutive in the batch's order, and the
  `AsAllowed` task, if any, with what it was given; a word's message
  number; a proposal's number and the role it was first admitted to, in the
  request's project; its present recipient is in its document. A replay keeps
  the original outcome's role; an account moved out, with its record, once: `requests.md`, 1), or what
  was decided; for a compound request, its
  operations made, in order, and the first of the message numbers they
  took, one each (`requests.md`, 4); and, with a retained decision visible to the party,
  who decided first, how and when. A task's automatically replaced settings
  proposal has no retained decision: deciding it answers `Unknown`, naming
  nothing and changing nothing (`../../../domain/tasks/proposals.md`).
  Accepting an ended settings proposal with a retained decision takes the
  running-work choice as a live acceptance does; that choice does not make
  a missing proposal malformed;
- **refused,** with its facts, and for a compound request the place of
  the first operation that failed:
  - *role:* the party's role here does not allow it;
  - *authority:* what was lacking, typed (a budget and how much is left,
    a grant, a kind of task, a depth, a deadline, tasks left, or a
    ceiling a policy edit passes: its holder, the deployment's rules, the
    project's policy or a role by number, the dimension (a spend per
    period, a spend, tools, delegation's tasks, depth or executors,
    grants, a deadline or notes), and, for a scalar, its value beside
    the edit's; a list is named without its contents,
    `../../../domain/authority.md`, section 6); and, where
    the request was judged against the party's role and pool and found
    beyond them (`../../../domain/authority.md`, 8.4), whether a proposal of
    it could be accepted (`requests.md`, 5). A refusal that another check decided
    (the request's shape, a reference, an uncertain effect's choice, its
    funding, the project's spend per period, an `AsAllowed` that would
    give nothing) makes no claim either way. Who would accept a proposal
    is the offer's to say (`documents.md`, 3), and the proposal's document's once it
    is made, not the refusal's;
  - *no one to escalate to:* a party's root task whose escalations would
    have no holder (`requests.md`, 2);
  - *limit:* which one (the deployment's projects, the project's live
    tasks, its goals, the operations per request, a grace past its bound,
    the inbox full, a party's words too long, an amendment too long, a
    key, a name or a pasted address past its bytes, the LLM accounts'
    bounds: accounts in the deployment, accounts per holder, sources per
    account, sign-ins at once, program sources per worker; and those that
    are no number: a duplicate, an empty batch, a reserved or duplicate
    role, a resource held, a hold's kind, a record after ready, a counter
    exhausted) and, where a number was met, its bound: a configured one
    with its configuration key, or a fixed one, so that a client names
    it, prints the bound, and says to raise the key only where one can;
    a client's own limits are the client's to say;
  - *provider's rule:* an LLM provider's terms forbid it, naming the
    rule (a plan's uses naming another person, or a project with
    another party; another account for a task its account limits:
    `../../../domain/accounts.md`, sections 4.2 and 4.3);
  - *account preference:* the account's number and why it cannot fund the
    prospective task: unknown, another provider, uses that do not serve the
    financial payer, or a different account while the kept account of that
    provider is limited. An otherwise valid unavailable account is accepted
    and waits; acceptance checks a proposal against its accepter's funding;
  - *moved:* the object's revision now, and what changed;
  - *ended:* the object has ended, and how;
  - *unknown:* nothing by that name the party may see;
  - *name taken:* another project holds the name a `MakeProject` or a
    rename gives, under simple case folding (`../../../domain/model.md`,
    6.1); it names no project, which the party may not see; or a live
    source holds the program source's name a declaration gives, or
    another holder's reservation does (`../../../domain/accounts.md`, 3.4),
    naming neither;
  - *not offered:* the deployment does not offer it (an executor it does
    not have, a charter its newest revision removed; for an account's
    act, what `../../../domain/accounts.md`, section 5, says: a declaration
    to an LLM provider offering the holder no program's login, a resume
    of a source that is not limited);
  - *stale:* a change or a proposal of settings names a revision other
    than the newest, which it names (`../../../domain/engine/settings.md`, 2);
  - *judged:* an application's connector refused an item of its kind of
    settings, naming the item and with its reason, in its own words
    (`../../../domain/engine/settings.md`, 5);
  - *full:* the settings' revisions or charter versions held would pass
    their limits, naming the oldest revision held and its live tasks,
    cut at their limit, which a `MoveToNewest` may move
    (`../../../domain/engine/settings.md`, 8);
  - *circle:* the batch or the dependencies an amendment adds would close
    a circle of waits, with the live tasks or among its own: the circle's
    tasks, in order, each waiting for the next, cut at its limit
    (`../../../domain/tasks/holds.md`, 7);
  - *live tasks:* an owner's `RemoveProject` of a project with live work:
    the project's live roots, by number, lowest first, at most 32, and
    how many more (`../../../domain/people/acts.md`, 2), in jig's words
    "This project has live tasks; end them first:", each client listing
    the numbers after them;
  - *dependency:* a dependency an amendment adds has failed or was
    cancelled, naming it and how it ended (`../../../domain/tasks/changes.md`
    );
  - *kept:* a person task's answer while it is kept, naming its pending
    dependencies (`../../../domain/tasks/lifecycle.md`, 8);
  - *not well formed:* which task or operation, and why (two tasks of a
    batch giving `AsAllowed`; the conversation flag on a task that is not
    an agent's; a proposed batch giving `AsAllowed`; a project's name out
    of shape, saying which rule; a change of settings whose settings
    would fail their check, naming the setting and its fault, one of
    the check's closed list (`../../../domain/engine/settings.md`, 1), or that
    changes nothing). A client words each refusal of a change from
    these facts, checking no setting itself;
  - *sign-in redirect:* a finish refused, saying why: *not waiting*
    (no sign-in of the party's under that number waits for its
    redirect: none, another party's, one that has taken one, ended,
    or begun before the engine's restart), *address* (not the
    sign-in's redirect: another scheme, host, port or path, a
    fragment, a field twice, past its bounds, or without its code or
    issued client id), *state* (another sign-in's: the sign-in still
    waits), or *too many* (it has refused a few for their state, and
    takes no more pastes, still waiting for its browser: skein's
    `oauth/accounts.md`, 5; `../../../domain/accounts.md`, section 5);
  - *key conflict:* the key names another ask, which is a client's bug
    where the client drew the key (`requests.md`, 1);
  - *account move:* an account's move refused, saying why: *not signed
    in* (no signed-in source holding a record that reads), *not ours*
    (a moved file of another application, or of no configured LLM
    provider's registration), *unreadable* (a moved file that does not
    decode, of a form or version this build does not read, or damaged),
    *held* (its subject held by another
    holder), *cancelled* (a *written* for a move no longer under way:
    the client removes the file it wrote), or *moving* (the source is
    out with a move, or back from one and not yet refreshed:
    `../../../domain/accounts.md`, section 6);
- **busy** or **not ready:** nothing was made and nothing kept; send it
  again, under its key, after a backoff. A busy says why when it is the
  table of retained answers that is full (`../../../domain/people/acts.md`,
  1): *answers full*, with how many it holds and when its oldest
  answer expires, so a client says so and when it will have room, never
  only that the request was not sent;
- **no answer** (the exchange lost: its deadline passed, or the
  transport failed before an answer): the outcome is unknown, so the
  request is sent again under its key after the same backoff; its key
  makes a repeat safe, since a request already taken answers with what
  was retained, and a sign-in by link repeats until the link's own
  expiry ends it;
- **signed out:** the sign-in has ended; sign in, then send it again;
- **version not served:** the engine does not serve the version the
  request's owner is encoded in. Nothing was read, made or kept. It
  names the owner and the ranges the engine serves (section 6).

A client hears one more, which no engine sends: **no answer**, when the
connection failed before one came; whether the engine has the request is
unknown, and the key makes sending it again safe.

How the client's link hands each answer to its request table is
`../../../protocol/client/README.md`'s, section 3; a request answered version not
served is held, kept and unsent, for its owner's version (../client/watches.md,
2).

## 4. Reads

A read asks for one document, or one page of a list, and is answered
once, with the document or why there is none. It decides nothing and
needs no key. Its answer is made from the engine's state as it is, which
may run ahead of the store by the commits in flight, so it is held as an
output of a decision that made no commit: tagged with the last commit
made, and released once that is durable (`../../../README.md`, 6.2). What a
party reads is therefore never lost by a crash.

| Read | From |
|---|---|
| `me` | live state |
| `task_text{n, field, from}`: specification words, effective instructions or kept report words, a UTF-8 chunk | live task source, or its retained owner row |
| `task{n}`; `proposal{key}`; an escalation; a question; a person task | live state, or the store once ended or decided |
| a result | the store, or live state while a stub names it |
| a task's history, a page, for it or its subtree | the store |
| `activity{n}`: a task's activity, a page, from its end | the store: words, run boundaries, turns, what it heard |
| `activity_beside{n, position, side}`: the page of a task's activity next to its line at `position`, earlier or later, that line left out | the store, as `activity{n}` |
| `turn{n, attempt, turn}`: one committed turn of a task's activity | the store: the task's transcript (`../../../protocol/store/transcripts.md`, 3) |
| `inbox`: a group of it, a page, filtered by project, kind or root | live state, then the store past the engine's limit |
| `tasks{query}`: a task query's page (4.1) | live state, then the store's indexes of ended tasks |
| a project; its spend, a page; its notes, a page | live state; the store; the notes child |
| the deployment | live state |
| `settings`; `setting{kind, scope, number, revision}`: one setting's value whole, as the newest reads it, or, for a charter, as a held revision reads it, or, with `proposal`, as a pending proposal of settings would set it, or, of an application's kind, its state and size alone; `revision{n}`: a revision's record and its items, as it set them, an application's with their sizes alone; `revisions`: its records, a page, newest first | live state; the store's history, for a revision no longer held (`../../../domain/engine/settings.md`, 7) |
| an application's read, naming its owner, its scope and its window (5.5) | its owner, the connector, through the engine's root |

- **Task source text** is read independently of its overview. `field` selects
  specification words, effective instructions or result words. Instructions are
  an agent's override, otherwise its charter at the task's recorded settings
  revision. Result words are a terminal Report, NoChange, Change or Verdict's
  stored words, including a report kept with cancellation, from a live terminal
  or retained owner row; other results and tasks without a report are unknown. The overview
  keeps its bounded preview and typed result metadata. Each answer gives `task`,
  `field`, `from`, `total`, `text` and `next`, byte offsets into that source.
  The chunk is at most `documents.text_bytes`, ends at a UTF-8 boundary and
  advances whenever `next` is present. A non-boundary or past-end offset is
  unknown; a text window too small for the next character is busy. Visibility,
  authentication and durable delivery are the task read's. Unavailable
  instructions are unknown. Each read uses current task state; after an amendment
  or settings move, restart at zero to read the new source consistently.

- **An ended task is a finished read.** Opening its task or tree watch
  delivers its retained document once, then ends the watch (5.2). A
  client waiting for a condition tests this first document too: a condition
  already true answers at once, and an ending alone proves no condition.
  A task read returns its live state or retained terminal history under
  the party's current visibility. A retained load
  waits for the preceding commits to be durable and checks current
  sign-in and visibility when it completes; it never restores a retired
  row into the live set (`../../../domain/engine/README.md`, 5.2 and 5.3).
- **Notes are read by scope.** A note's scope on the client's side is
  `Deployment`, `Project`, `Resource`, `Task { project, task }`, the
  engine's goal scope, which also holds a chat's own
  (`../../../domain/engine/briefs.md`, section 2), or `Party { party }`, a person's
  own preferences (`../../../domain/engine/briefs.md`, 2.1), each answered whole. A
  party's scope is read, and its entries edited and deleted, naming
  project 0, by that party alone, whatever their roles; any other party's
  read is unknown and their edit or deletion refused as unknown, an
  owner's included. `Tasks {
  project }` is a read of its own: every note of the project in a goal's
  scope, so that a client lists a project's documents with no read per task,
  each `Note`'s `body` empty and its `cut` set, a row's `scope` its own `Task`.
  It is for reads alone: an edit or a deletion naming it is refused. It takes no
  description. Its visibility is the project's, as any of its notes' is, and it
  pages as any. A client's `long_text` bounds a note's body and a task's words
  on its side (`../client/README.md`, section 11). A body is whole however its run wrote
  it, at once or by line ranges (`../../../domain/engine/briefs.md`, section 2): a read
  sees no parts, and nothing of a range reaches the wire.
- **Pages** are within a row and a byte limit, and carry a cursor for the
  next, or say there is none (`../../../domain/engine/README.md`, 5.3).
- **A deleted project's read is gone.** A read of a project the
  people child marks deleted, while its deletion lasts and after it,
  answers *gone* to any signed-in party, where a project absent or
  hidden from them answers unknown; nothing else of it is read
  (`../../../domain/people/acts.md`, 2, *Marked deleted*). Its other
  reads, requests and watches are unknown, as a project never made.
- **Refused** for what the party may not see, as unknown; when the
  engine's loads are full, as busy; when the sign-in has ended, as
  signed out (../client/README.md, section 9); in a version the engine does not
  serve, as version not served (section 6); where the row at the read's
  position needs more joins than any read allows, as too linked, naming
  the bound (`../../../protocol/store/loads.md`, 3), so a client never reads
  the same position again in vain.
- **An answer that cannot be encoded** within the client wire's bounds is
  answered malformed. At the client, an answer that cannot be decoded within its
  receiving bounds ends any jig read malformed, with the link's undecodable fact
  (../client/README.md, section 13). A lost exchange remains unreachable; decoding failure
  alone never asks for another attempt.
- **A question's answer** is read with the question, by whoever may read
  the question: a delegate is in its requester's project, and the
  answer's words are those of the task the question was put to
  (`../../../domain/engine/executors.md`, 2; `../../../protocol/store/loads.md`, 3).
- **A page beside a line** is how a window that let lines go reads them
  again: an activity line's position is a fact of the line, dense and
  never reused, so it names a place where no cursor was kept. Its answer
  is an activity page. Earlier, its `before` continues it as any page's
  does and its `after` is none; later, it holds the lines just after the
  position, its `after` says that more remain and its `before` is none. An
  empty page is the answer past either end. It is read with the access of
  `activity{n}`.
- **A turn** is named by its task, its attempt and its number within it
  (5.3). It is read once and never watched: a committed turn does not
  change. It is read with the access of `activity{n}`. Its answer is:
  - *the turn:* smith's bytes, which the client decodes as it decodes
    activity's turns (../client/conversations.md); to a party other than the
    person who requested its task, withheld: its preferences section and
    its private tools' calls projected as `../../../domain/engine/briefs.md`, 2.1
    says, here and on activity's pages alike;
  - *unknown,* when it is not there or the party may not see it;
  - *gone,* past its transcript's horizon, as a dropped transcript is
    (`../../../protocol/store/transcripts.md`, 5, and `../../../protocol/store/README.md`, 6.1);
  - *busy,* as any read;
  - *too large,* with its size in bytes: from the engine, past the
    single read's bound, the store's per-read byte budget
    (`../../../protocol/store/loads.md`, 3); from the client, past its limit
    for one document (../client/README.md, section 11). On activity's pages the
    client keeps the size the wire carries for a turn too large for a
    page (`documents.md`, 3);
  - *malformed,* when it does not decode, as a page's turn is (`documents.md`, 3).
- **A revision** is read by its number, by a party who may read the
  `settings` (`documents.md`, 1), a project's items by a party with a role there.
  `revision{n}`'s answer is:
  - *the revision:* its record (the revision it was made against, when,
    who decided it, a party, the release, the first start or a provider's list, the
    proposal it came from, whether it moved running work) and its
    items, each a setting by kind, scope and number and what became of
    it, set with the value whole, removed, or reset, and what it
    replaced (`../../../domain/engine/settings.md`, 10); an item's value
    past the single read's bound comes with its size instead, as too
    large, read whole by `setting{…}` while its revision is held; an
    item of an application's kinds with its size and the state it
    replaced alone, its connector keeping the values, those it replaced
    for a restore (`../../../domain/engine/settings.md`, 5 and 10); an item of a
    project's own policy with the policy it set and the one it replaced;
  - *forgotten,* a revision made but erased past the history
    (`../../../domain/engine/settings.md`, 7), naming the oldest kept;
  - *unknown,* past the newest;
  - *busy,* as any read.
- **`revisions`** is the history a page at a time, newest first, from
  the newest or a cursor: each row a record without values, its items
  as settings and what became of each, cut at the page's bound with how
  many more. The last page says whether older revisions were forgotten.
  It is read, never watched: the `settings`' mark says a revision came.

### 4.1 Task queries

- **A task query** selects tasks by a fixed set of keys, any conjunction
  of them: project; requester (a party, or a task); root; parent;
  label; phase class (working, waiting, held, closing, ended); executor
  kind; tracked. It orders by number, ascending; last change or last
  activity, latest first; priority, ascending, tracked tasks before
  untracked; or ended at, latest first, its live group, which has no
  end, by number ascending. Ties go by number, ascending. It reads
  within a window, by cursor. Its rows are task lines (`documents.md`, 1). It is read
  a page at a time, and watched as `tasks{query}` (5.1). A list of a
  party's conversations, of a project's goals by priority, of a task's
  ended delegates, are each a task query.
- **Live first, then ended,** in two groups, each with its own cursor.
- **Live tasks are scanned,** in the order asked: the tasks child keeps
  no secondary index, and its live tasks are bounded by its limits, so
  any conjunction of the keys costs no more than one scan. The fixed set
  bounds what marks a watch (5.2) and what crosses the wire, not the
  filtering.
- **Ended tasks are found through the store's indexes**
  (`../../../protocol/store/README.md`, 4.4, and `../../../protocol/store/loads.md`, 3 and 4), in each index's order,
  within a label the latest ended first:
  - *by requester:* every ended task whose requester is a party, by that
    party and label;
  - *by project:* only ended tasks flagged *listed*, by project and
    label;
  - *by requester task:* a task's ended delegates, in their family.

  A query's ended group is served by one index, chosen by its keys in
  this order: a requester party; a requester task or a parent; a
  project. Its other keys filter the index's pages as they are scanned,
  and a page so filtered may come back cut, saying so. A query naming
  none of these has no ended group, and its page says so.
- **What it does not serve:** full-text search
  (`../../../protocol/store/README.md`, section 11); ended tasks filtered by
  outcome, resource or spend; keys valued by a procedure's parameters,
  which belong in the application's own records naming the tasks; ended
  tracked tasks across projects.
- **Retention:** an ended task forgotten once read leaves the indexes,
  with its read position, once its requester, a party, has read its
  result and its turns, and no live task names it
  (`../../../domain/people/README.md`, section 5); a kept one stays to the
  deployment's retention horizon, and whole while none is set
  (`../../../protocol/store/README.md`, 6.1).

## 5. Watches

### 5.1 Subjects

| Subject | Its document | Who may watch |
|---|---|---|
| `me` | `me` | the party, always |
| `task{n}` | the task's document | a party who may see it |
| `tree{root}` | a task's live subtree, within a window | likewise |
| `chips` | a bounded set of tasks the client names, as chips | likewise, each |
| `inbox` | the party's own, across projects, within a window | the party |
| `tasks{query}` | the live task lines a query selects (4.1), in its order, within the watch's window, saying whether there are more | a party with a role in the project it names; a query naming no project, only the party, for tasks whose requester they are |
| `run{n}` | a task's live run: its attempt, its provisional text and current call (5.3) | a party who may see the task |
| `deployment` | hosts, accounts, capacity | a party with a deployment role |
| `settings` | the settings (`documents.md`, 1) | a party with a deployment role; a project's items, a party with a role there |
| `accounts` | the party's accounts (`documents.md`, 1): their own LLM accounts, their sources and states, their sign-ins and checks under way, and whether an account serving their work is *usable* and everything they asked is *settled* | the party, always |
| an application's subject, naming its owner, its scope and its window | the application's document | a party the scope admits (5.5); what of it they may see, as its owner, the application's connector, says |

The objects a client shows apart from these (a proposal, an escalation,
a question, a person task, a result, a task's activity, a project, a
note) are read (section 4), and read again when a watch that names them
says they moved. A turn is a read and not a subject: a committed turn
does not change, so it is read once (section 4).

Account source/uses and fleet changes invalidate accounts watches; membership
changes invalidate them too, including another member joining a sole-holder
project. The catalogue is immutable startup configuration and arrives on every
new `me` watch, including after restart.

### 5.2 Level-triggered

- **A delivery is a whole document,** the subject's current state within
  the watch's window, computed when the delivery is made, not when the
  change happened, and held until every commit it reflects is durable,
  as a read's answer is (section 4).
- **One delivery in flight per watch.** A change while one is in flight
  marks the watch changed; when the client has taken that delivery, the
  next is computed from the state as it is then. Changes in between are
  coalesced, never queued.
- **Nothing is missed:** the next delivery has every change. A slow
  client sees fewer deliveries, a fast one more; neither holds the engine
  up, and what the engine keeps per watch is a mark and a window, never a
  backlog.
- **The first delivery** is the subject's document when the watch is
  admitted. For a task or tree whose root has ended, the core loads its
  retained task facts through the bounded durable read path, checks the
  current sign-in and visibility, and delivers once. A tree still means
  the live subtree (`documents.md`, 1): its retained ended root has no live descendants,
  folded counts or continuation, whether it was a root or a delegate.
  The watch ends with the subject-gone reason after that delivery
  is taken or dropped. Missing or hidden retained facts end it without a
  delivery; retained facts too linked to load within the read's join
  bound end it without a delivery as too linked, naming the bound, as
  the task's read is refused (section 4; `../../../protocol/store/loads.md`,
  3), never as the subject gone; an ending alone is never terminal
  success. Other ended
  subjects are refused `Unknown`. No retired task is restored to serve it.
- **Ended,** each saying why: the client left; its subject ended and left
  the engine; the party may no longer see it; the sign-in ended; the
  engine is stopping.
- **Admitted** against the party's sign-in and what they may see, and the
  watches the engine allows per party, per stream
  (`../../../protocol/client/README.md`, section 10) and in all; past them, refused
  as busy. A watch holds its place from its admission until the engine has
  ended it: while its closing is in flight, and while its stream's end
  or its last delivery is (`../../../domain/engine/views.md`). A
  deployment allows each party, and each stream, at least the engine's
  watches one client opens at its limit, `me`'s included (../client/README.md,
  section 11), so that a party's one client within its limit never finds
  its room full. Busy is the room full for now, and the client opens the watch
  again after a backoff (../client/watches.md, 1), as a request refused busy is
  sent again (`README.md`, section 3).
- **A window past the engine's bounds is clamped,** at admission, never
  refused: the watch keeps the window the engine's rows admit, and each
  delivery is its document within it, saying there is more as its
  subject's document does (a tree's folded counts and its cursor; a task
  query's and an inbox's cursor; `documents.md`, 3). An application's window is
  clamped likewise before its connector is asked, and so is a live
  read's window. So busy is never a window's answer.
- **A document the engine cannot make within its bounds** for any window
  (a whole value past them, which its bounds are derived to rule out,
  `documents.md`, 3) is refused as unknown at admission, never as busy, since opening
  again would not mend it: the client ends the handle (../client/watches.md, 1).
  A delivery that cannot be made ends the watch, as its subject gone.
  An answer of its own is open (`README.md`, section 9).
- **An application's subject names its owner, its scope and its
  window,** as an application's read does: its owner by its number, the
  scope a project, the deployment, or the party's own (5.5). The core
  cannot read a connector's subject, so it admits on what it holds: the
  party's sign-in, their role in that scope, and the views' limits, as
  for jig's subjects. The connector, asked for the document, judges that
  the subject is in that scope and what the party may see of it, and
  answers unknown or not allowed otherwise. The engine's root only
  routes.
- **What marks a watch changed:** each decision's commit names the
  tasks, projects and parties it touched. A task touched marks the
  watches of that task, of every tree along its lineage, of chips naming
  it, and of every task query whose keys it matched before the decision
  or matches after it, so a task that leaves a query's set changes it
  too; a party's entries, role or pool touched mark their inbox and
  their `me`; a project's name or policy changed marks the `me` of every
  party with a role in it; an account's source, state, order or uses
  touched, a program's login reported on a host, a host connecting or
  leaving, or a sign-in or a check begun or ended, marks the `accounts` of
  the parties it concerns and the `deployment`'s, and their `me`, whose
  offer and previews read the same facts. Membership and effective-executor
  changes
  also invalidate affected `resolved_default` previews (`documents.md`, 1). Matching a
  task against the open queries is bounded by the watches the engine
  allows, each a fixed set of keys. A connector marks its own subjects,
  through the root, as its state changes.

### 5.3 A run's live text

The one stream: what a run writes before its turn commits is not state
the engine keeps (`../../../domain/engine/views.md`), but a person
reads it as it comes. Its deliveries decide nothing and follow no
commit, so they leave through the journal's door (`../../../README.md`,
6.3), marked provisional.

- **The run's document** is its attempt, the turn being written, named by
  its attempt, its number within it and its completion's retry as the
  host reports them, that retry's text so far, the call it is making, and the call being
  written (`writing`), with its provider id, tool, bounded input suffix,
  byte offsets and cut mark,
  within a bound on bytes. A call of the application's private tools
  carries its input, made or being written, to the task's requester alone;
  every other follower is sent its tool and no input
  (`../../../domain/engine/briefs.md`, 2.1).
- **Read from an offset:** each delivery carries the turn's name, the text
  after the offset the client has, and the offset it ends at; the client
  takes it and is sent the rest when there is more. A new turn's text,
  or a new retry's, starts at zero under its own name, so it never resets
  another turn's; a later retry replaces the earlier retries of its turn.
- **Committed turns are not in it.** A turn's committed form reaches the
  client only through its task's activity, read once the task's document
  says the activity has moved, and held until durable (section 4).
  `jig_client::thread` keeps a turn's provisional text until then
  (../client/conversations.md); the client's watch holds only the turn being
  written (../client/watches.md, 3).
- **Past its bound,** the text is cut from its start, at a character's
  boundary, and the delivery says so; the turn, once committed, has it
  all.
- **The watch follows the task,** not one attempt: a new run's text
  follows the last, and its beginning is in the document.
- **Only from a host that gives it.** A run's text and the call it is
  making come from hosts that hand them up (`../../../domain/hosts/hub.md`,
  5): an agent process gives text and the call being written; it gives
  no call being made. Its durable turns arrive as they commit.

### 5.4 An application's streams

An application's data that its people follow as it comes (a log
followed, a metric's latest points) is a stream of the application's:
a subject naming its owner, its scope and its window, read from an
offset, as a run's text is (5.3). Its deliveries decide nothing and are
never a document of the engine's committed state: they leave through
the journal's door (`../../../README.md`, 6.3), marked provisional, and an
application shows them marked so. It adds a kind of subject to the
application's own and a use of the journal's door, with no change to
5.1's subjects or 5.2's rules, and crosses on the client's one stream as
any watch's deliveries do (`../../../protocol/client/README.md`, section 5).

### 5.5 An application's scopes and windows

- **Scopes.** An application's request, read or subject names one:
  - *a project,* admitted on the party's role in it;
  - *the deployment,* admitted on their role in the deployment
    (`../../../domain/people/README.md`, section 3), as jig's `MakeProject` is
    (`requests.md`, 2);
  - *the party's own,* what is the signed-in party's across projects (an
    application's settings of theirs, the incidents they are paged
    for), admitted on the credential alone, and naming no party: the
    core names the party from the sign-in, as for `me`.

  The connector judges what any scope shows (5.2).
- **Windows.** An application's watches and reads carry a window, as
  jig's subjects do: bounded rows and bytes, at the cursor its owner
  gave, so that what the views keep per watch is a window and a mark,
  never a backlog, and every document fits the client's limits.
- **Streams** are 5.4's; **versions** are per owner (section 6).

## 6. Versions

- **Each owner's documents and requests are versioned,** jig's by jig and
  each application's by the application, and every message names its
  owner and its version (`../../../protocol/client/README.md`, section 6).
- **The engine serves a range of each owner's versions,** which the hello
  of its stream states; `me` carries none.
- **A version not served is refused with a typed answer,** *version not
  served* (section 3), naming the owner and the ranges the engine
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
   of it could be accepted (README.md, section 3); a party's proposal naming the
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
   (`../../../domain/people/README.md`, section 5; `../../../protocol/store/README.md`, 4.4).
7. **Templates served, never expanded** (`requests.md`, 6): the configuration's
   templates and each policy's overrides, kept only to serve each
   project's merged templates in `me`; an override naming no template,
   or one another override of the same policy names, refused when the
   policy is changed; a retained LLM choice incompatible with the
   effective executor refused at startup, policy change and restore, the
   store against the configuration (`../../../domain/engine/restart.md` and `../../../domain/engine/views.md`, section 1
   ;
   `../../../protocol/configuration.md`, 3.1).
8. **Credentials minted on the server's word:** a sign-in committed for
   an identity the application's server verified, its provider of kind
   application (section 2; `../../../domain/people/README.md`, section 3).
9. **An application's keyed requests,** admitted and kept by the people
   child as jig's are, told apart by a digest, judged as the party's when
   they ask the core for something, their outcomes kept by their
   connector for the retention (`requests.md`, 3; `../../../domain/people/acts.md`, 1;
   `../../../domain/connectors.md`, 5.5).
10. **Refusals with facts,** as section 3 lists them
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
    are taken (`../../../domain/engine/executors.md`, 2).
15. **The reads of section 4,** task queries among them (4.1), their
    answers held until the last commit made is durable
    (`../../../domain/engine/README.md`, 5.2 and 5.3).
16. **Level-triggered views** with the subjects of 5.1, what marks them
    changed, task queries by their keys among them (5.2), a run's text
    by turn, retry and offset, and an application's subjects through
    the same watches, each naming its scope, the party's own among them,
    and its window (5.5; `../../../domain/engine/views.md`). The
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
    with the *usable* and *settled* facts of the first, marked as 5.2
    says (`../../../domain/accounts.md`, sections 3 to 5;
    `../../../domain/people/acts.md`, 1).
19. **A task's LLM choice** (`requests.md`, 2): given at `Create`, changed by
    `Amend` for the next run, or, where it gives none, taken at each
    run's check, the first of the charter's models it can use
    (`../../../domain/accounts.md`, 4.3), within the authority's models; the
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

- **Reasons within account acts' not-offered answers** (section 3):
  key add or replace and sign-in start or finish can reach this answer
  through several core causes. Carrying each cause needs finer account
  refusals and their translation (`../../../domain/accounts.md`, section 7).
  Default: the existing refusal. Declarations and resumes keep the
  single meanings in section 3; clients can word those from their ask.

- **Browser-approved sign-ins beyond loopback** (2), and a person's
  sign-ins listed and each ended: `../../../protocol/client/README.md`, section 12.
  The built-in approval here signs in the approving browser's own party.

- **Refusal paths within charter parts** (`requests.md`, 2): a malformed part in a
  settings change is refused at its charter's request item as out of shape
  or past its count. Naming the part and its `on` member needs a refusal
  vocabulary beyond the charter-document paths. Default: the item's
  position and the existing shape or count refusal; configuration keeps
  its own part paths (`../../../protocol/configuration.md`, section 3).

- **Decisions on replaced task proposals** (requests.md, 2, and README.md, 3): automatically
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
- **Bundling parts:** whether a jig read's answer should carry the
  application's parts it names, put together by the engine's root, to
  save a client the second round trip.
- **Windows of a large tree:** how a tree watch's window follows what a
  person opened, so that a tree of hundreds of tasks costs what is shown.
- **Finer cuts among a project's parties**
  (`../../../domain/people/README.md`, section 11).
- **Search** over tasks, results, notes and transcripts, which the engine
  has only over notes' descriptions.
- **A document past its bounds:** a read whose answer cannot be encoded or
  decoded ends malformed (section 4). Whether the engine instead derives bounds
  so it cannot happen, as a path's segments are, or takes a client's bounds to
  page within them. A watch's opening is refused as unknown and an oversized
  delivery ends its watch (5.2); a watch's answer of its own remains open.
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
