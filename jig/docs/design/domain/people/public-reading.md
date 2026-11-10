# Public reading

Anyone may read a project its owners have made public, without signing
in: one party, fixed in code, reads for every visitor, a few documents
only, within a budget of its own behind every signed-in request. The
documents are public projections: types of their own, which hold only
what is added to them, and only structure and mechanics, with tasks'
titles and the project's name their only text. The application adds
its own beside jig's (3). How a public read travels is
`../../protocol/client/public-reads.md`, and over HTTP, 17.5.

## 1. The public party

- **One party for every visitor,** fixed in code: the public party. It
  is no row: no identity, provider, credential, sign-in, inbox, pool,
  `me` or place in a roster. The core names it by a value of its own,
  which no stored party, sign-in or connector takes, and no record
  stores.
- **Its role is fixed:** `public`, which grants one thing, the public
  reads of a public project (3) and of the deployment's public
  projects. It makes no request, opens no watch and no stream, reads
  none of jig's other documents and none of an application's other
  reads or subjects, and is the requester, funder, executor or holder of
  nothing. Every other exchange without a credential is signed out
  (`../../protocol/client/sign-in.md`), answered by the clients'
  component from the exchange alone, with no engine work.
- **No policy widens it.** `public` is no role a policy can name: it
  has no number and is no `Role` (`README.md`, section 4), and policies, rules,
  permissions and seeds name roles by number alone, so none can map a
  connector's permission to it, give it authority or a kind of
  proposal, or give it to a party (`acts.md`, 2). The public party's reads are
  admitted by the public index (9), never by a role's authority.
- **It writes nothing.** A public read is a read: it commits nothing,
  keeps no answer under a key, and names no party in any history.
- **Only the core decides,** as for every party: the clients'
  component forms a public read from the route alone, and the core
  admits it on the project's visibility (2) and the task's
  eligibility (7), never on anything the visitor sent but the read's
  own fields.
- **Its exceptions are its own:** no guard of a signed-in exchange is
  relaxed for it. A public read is a message of its own, on routes of
  its own (`../../protocol/client/default-http.md`, 5), and every check of the
  signed-in routes stands as without public reading.

## 2. A project's visibility

- **Private by default.** Every project is seeded and made private
  (`acts.md`, 2), and no configuration makes one public: a start opens nothing.
  Project 0, the deployment's roster, is never public. Visibility is a
  row of its own, kept only while a project is public
  (`../../protocol/store/README.md`, 4.4), so a store written before it holds
  none and every project in it reads private: no default makes one
  public.
- **Made public by its owners:** `SetVisibility { project, public }`,
  owner of the project alone; no policy gives it to another role. It is committed and answered once
  durable, as any request (`acts.md`, 1), and is a record of the project's, kept
  in the people family's project row (`../../protocol/store/README.md`, 4.4).
  Making a project public shows its name, its structure and its tasks'
  titles (3).
- **Every party of the project sees it:** the project's entry in `me`
  carries its public identifier while it is public, and none while it
  is private (`../../client/domain/contract/documents.md`, 1), so a client says
  so wherever it shows the project, and can give its public address.
- **Made private again,** the decision that commits it refuses every
  later public read of the project as unknown, and every public read of
  it still under way is answered unknown (10). It waits for none of
  them. What a caching proxy or a browser already holds lives out the
  freshness its answer was given (`../../protocol/client/default-http.md`, 5); that
  bounds later serving, and cannot withdraw a copy already downloaded.
- **Closed with the project:** a project being deleted answers every
  public read unknown from the decision that admits its deletion, as
  one never made (`acts.md`, 2), and its reads under way unknown alike.
- **Answered alike:** a project that is private, being deleted or never
  made, and a task that is not public (7), are one answer, unknown,
  reached by the same path and the same work, so a visitor learns
  nothing of what is not public (8).

## 3. What it reads

Each public document is a type of its own, made by a projection from
the public index (9); a field reaches it only when it is added
there. jig's hold structure and mechanics alone: public identifiers,
states, times, the shape of a tree, kinds, a goal's spend, task titles
and the project's name. Their records, and each field's source, are
`../../client/domain/contract/README.md`, section 10.

- **The deployment's public projects:** each public project's public
  identifier (8) and name, and the `max-age` the public answers
  carry, so a page knows how often to read again.
- **A project's board:** its public identifier and name; its public goals
  (7), live first by priority, then the most recently ended the
  index holds; each goal's line and its tree's lines below it, within
  the board's bounds, the lines past them folded into a count of
  public lines alone (9).
- **A task's card:** the line of one public task, its lineage up to
  its public goal as public identifiers, and its delegates' lines,
  folded past the bound.
- **A public line:** the task's public identifier; its title, the spec's
  first line cut at 256 bytes on a character boundary, saying whether
  cut; its parent's identifier, or none for a public goal, whose
  ancestry is never shown; whether it is
  a goal, and its priority; its executor's kind alone, never who
  or which; its phase, if held, the hold's kind without
  its reason, and if ended, how it ended, done, failed or cancelled,
  without its reason; when it was made, last changed, last active and
  ended, each to the second; a goal's spend, never a task's, and no
  budget; its public delegates' counts by phase.
- **No text but titles and the project's name.** The task's title and
  the project's name cross without an author flag or a human
  publication capability. Nothing else a person or an agent could
  have written crosses: no spec beyond its first line, instructions,
  inputs, results, words, questions and answers, turns and transcripts,
  notes, reason, label's meaning, proposal or escalation; no charter's,
  procedure's or role's name. No person's or party's name, login or
  number, no account, no resource's name beyond the links the
  application's projection adds. A human publication capability for
  the other configured names is Later (`README.md`, section 11).
- **A title can carry a chat's words.** A title is the first line of a
  spec an agent wrote from what a person said, so what was said in a
  chat can reach a stranger. Titles cross all the same: the user's
  decision, reaffirmed on 2026-11-06 (the request `ui-public-3`), on the
  condition that making a project public says so. temper's words, in
  each place it offers that (temper's `client/ctl/tools.md`, 3), are:
  "Anything said in this project's chats may show on its public pages,
  in the titles of its goals and tasks. On a public repository, a
  change's pull request shows what its agent wrote too."
  Its help and completed answer carry these words; an acknowledgement
  before publication is deferred in temper's `client/ctl/README.md`, section 11.
- **No internal number:** a project or a task is named by its public
  identifier alone (8), in every field and every route.
- **An application's own public documents** are its connectors' (the
  application's records, `../../protocol/client/public-reads.md`): a
  connector declares its public kinds, and is asked for one only once
  the core has admitted the read: the project public, and the task it
  names public (7). It is handed the project's public tasks and
  builds its document from them alone, by the same rule: a type of its
  own, private by default, no text, and every task it names one of
  those it was handed, named through the read's list of tasks
  (`../../protocol/client/public-reads.md`), which the core checks again
  before it answers (10). What a connector kept of a task past its
  end, such as temper's landed commit (temper's `client/forge/README.md`, 9.2),
  it gives only while that task is handed; keeping it makes nothing public.

## 4. Who sees what

Inside a project and outside it, and guests:

- **A party with a role in a project** reads and watches all of it;
  finer cuts among a project's parties (a chat its requester's
  alone) would be a role's, and are open (`README.md`, section 11).
- **Anyone else** reads only a public project's public documents, as the
  public party, and takes part in nothing.
- **No guest role.** Someone with no identity anywhere reads as the
  public party; to take part in a conversation they sign in, through
  the application's sign-in and a `Mint`, and are given a role (`README.md`, section 3
  ). jig keeps no role that sees one conversation and nothing else.

## 5. Its admission

The crowd is one party, bounded as jig bounds any party, and behind
every signed-in one. Every figure below is the deployment's
(`../../protocol/limits.md`), zero in every profile:

- **A room of its own:** public reads in flight, within a limit of the
  clients' component's, apart from the sign-ins' exchanges in flight;
  no signed-in exchange waits for one, and none of them takes a
  signed-in party's room (`../../protocol/client/README.md`, section 10).
- **A rate of reads:** a bucket of reads, refilled at the rate per
  second by the component's monotonic clock and holding at most its
  burst; each read admitted takes one.
- **A rate of bytes, reserved first:** a bucket of answer bytes,
  refilled likewise and holding at most its burst. Before a read is
  handed on, the component reserves its kind's largest answer, every
  kind's bound declared at start (`../../protocol/client/public-reads.md`);
  a read the bucket cannot cover is busy. Once answered, the bytes it
  did not send are given back; a read that ends any other way (unknown,
  busy at the root, its exchange gone) gives back all of them. The
  bucket never goes below empty, so a burst of reads at once is bounded
  by its burst, never by a debt.
- **A ceiling per answer:** no public answer is larger than the
  deployment's answer ceiling; every kind's declared bound is within
  it, and the ceiling within the bytes' burst, or the start is refused.
- **Behind every signed-in request:** the engine's root admits a public
  read only while its backlog is under the public watermark, lower than
  the one that admits every other new work, and only while the core
  holds fewer public reads than its own limit (../root.md, sections 3 and
  10). Load from signed-in parties refuses public reads first.
- **Protected capacity:** a public read loads nothing from the store,
  reading the public index alone (9); its work is bounded per read
  (9), so its share of the loop is bounded by its rate; its memory
  is its reservation and the index's; and over HTTP it holds a calls'
  room of its own, never a signed-in call's (`../../protocol/client/default-http.md`,
  5). Engine completions and signed-in reads never wait on one.
- **Busy first, never queued:** a public read past any of these is
  answered busy at once, saying nothing more; nothing waits, and the
  component never offers it again.
- **Off unless figured:** a rate of zero answers every public read
  busy, and is every profile's: a deployment that opens sets its
  figures, and the start checks how they relate
  (`../../protocol/limits.md`).
- **The proxy's own limits:** a reverse proxy before the engine caches
  the answers within their freshness, limits each address and admits
  public reads before they take its connections to the engine; jig
  trusts no address and counts none.
- **Durable before shown,** as every read's answer is (../engine/README.md, 5.2).

## 6. Its world

In the core's world: a project's owner makes it public, and the public
party reads its board and a delegate's card, with no text but their
titles and the project's name, and no party's name or internal number,
in them; a member's `SetVisibility` is refused; a goal made beneath a
conversation read as a public goal with no parent, the conversation
never named or counted; a delegate moved out of a goal's
tree leaving its board, its counts and its card, and one moved in
joining them; the project made private, and a public read answered
unknown, as a project never made and a private one are, with the same
work done before each answer; a conversation's delegate's card,
unknown; a deleted public project, unknown from its deletion's
decision; public reads past their rate, their bytes and their room
answered busy, while signed-in reads at the same moment are all
answered; the root's backlog past the public watermark, or the core's
public reads at their limit, refusing a public read and admitting a
signed-in one; a rate of zero refusing every public read; and every
exchange without a credential but a public read answered signed out.

Its races, each played both ways round: a project made private, a
project's deletion admitted, a task moved out of its goal's tree, and
a goal untracked, each while a public read of it is under way (an
application's document asked of its connector, an answer held until
durable), the read answered unknown; and a read whose reservation is
given back on every way it ends, the bytes' bucket whole after.

Its referee: the public party commits nothing and appears in no
record; no public document holds a field its type does not, a task
that is not public, or an internal number; no public read of a project
is answered after the decision that made the project private, began
its deletion, or took the task out of its public tree; no public read
is admitted while the root's backlog is at its public watermark, nor
holds a signed-in exchange's room; the bytes reserved and refunded sum
to the bytes answered.

## 7. Which tasks are public

One rule, the same for every field, list, count and connector:

- **A public goal** is a goal (../model.md, 6.2) of a public project, live
  or ended, whose parent, if any, is in no goal's tree: a goal made
  beneath a conversation is one, its parent not. A goal inside another
  goal's tree is a line of that tree.
- **A public task** is a public goal or a task below one, in its tree,
  that the public index holds (9). Nothing else is: a conversation
  and its delegates outside every goal's tree, a recurring task, a task
  of another project, a task past the index's bounds.
- **Ancestry stops at the public goal:** its line names no parent, and
  a card's lineage ends at it.
- **Every reference passes it:** a parent, a lineage, a delegate, a
  board's goal, an application's change, landing, member or queue
  entry. A value made from a set (a count by phase, a folded count, a
  position, a total) is made from the public members alone, filtered
  before it is counted or ordered; a value whose meaning would disclose
  what is not public, such as a place in a queue that holds tasks that
  are not, is not given at all.
- **It moves with the decisions:** a task made, moved (`acts.md`, 1), tracked or
  untracked, ended, or its project made public, private or deleted,
  changes what is public in the decision that commits it, and the
  public index follows in the same step.

## 8. Public identifiers

- **Opaque:** a public project and a public task are named by public
  identifiers, never by their numbers, which would show how many
  projects and tasks were made in between. An identifier is a keyed
  permutation of the number, 64 bits, apart for projects and tasks,
  written as 16 lowercase hexadecimal digits.
- **The key is the clients' component's,** a secret of the deployment's
  made at its first start and kept in the store's secrets' records as
  its other secrets are (`../../protocol/client/public-reads.md`). Only the
  component maps identifiers to numbers and back; the core and the
  connectors see numbers alone.
- **A stranger's identifier is unknown:** one that maps to no project,
  to a private one, or to a task that is not public takes the same path
  as an excluded one, a miss in the public index (9), and is answered
  alike.

## 9. The public index and the work of a read

- **An index of public data:** the core keeps, for each public project,
  its public goals and tasks: the live ones as references into its live
  tasks, and the most recently ended public goals, each with its tree's
  public lines as they ended, within the index's bounds (goals ended
  per project, lines per goal, depth). A read examines the index alone,
  never the live tasks of the deployment and never the store.
- **Built before it is read:** at start, once the live tasks are
  restored, and when a project is made public, the core builds the
  project's entry from its live tasks; until it has, a public read of
  the project is answered busy. Kept with the decisions (7) after.
- **Ended goals since it was built:** the ended goals the index holds
  are those that ended while it kept the project, frozen as they ended;
  the store keeps no index of a project's ended goals to load from
  (`../../protocol/store/loads.md`, 4), so a start, or a project made public,
  begins with none. Loading them is open (`README.md`, section 11). A page drawing a
  board says what this leaves out, so that its empty lists are not read
  as no work (temper's `client/web/public.md`, 3).
- **Bounded per read:** a board examines at most its goals' bound times
  its lines' bound, a card its lineage's depth and its delegates'
  bound; each answer's bytes are known from those bounds, which its
  kind declares, so the whole nested document is priced, not each list.
- **Truncated by rule:** a list past its bound keeps its first entries
  in its order (live goals by priority, ended by when they ended, lines
  in tree order) and folds the rest into a count of public tasks; an
  ended goal past the index's bound is no longer public, and its card
  unknown.
- **Its memory** is its bounds times the deployment's public projects,
  within the start's fit check (`../../protocol/limits.md`
  ).

## 10. A read under way

- **Rechecked before it is answered:** a public read's answer is made
  in the core's step that completes it, after every asynchronous part
  (its connector's document, its wait to be durable). In that step the
  core checks again that the project is public and not being deleted,
  and that every task the answer names is still public (7); if not,
  it answers unknown.
- **Never in revocation's way:** making a project private, deleting
  it, moving or untracking a task commit as they would with no reads
  under way; the reads they touch end unknown when they complete.
- **Its freshness is its snapshot's:** an answer carries when the step
  that made it decided, and the clients' end gives it what remains of
  the `max-age` from then (`../../protocol/client/default-http.md`, 5), never a
  fresh `max-age` for a late answer.
