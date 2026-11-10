# Loads

## 1. Pages

- **Same step.** skein-kv serves a page from memory in the step that asks
  (skein's `data/kv.md`, section 2), and the component answers in that step
  too, through its outputs, never by recursion. Only transcripts' bytes
  take io (`transcripts.md`). The store answers `Restore` with `Event::Page` or `Event::PageRefused`.
- **The cursor is exclusive.** A page's cursor is the last key it
  returned; the next page starts just past it, and an absent cursor at
  the interval's beginning. skein-kv's ranges start inclusively, and its
  next key is the first it did not return, so the component asks from
  the cursor followed by a zero byte, the least key above it.
- **Two bounds.** skein-kv bounds a page by encoded bytes, the asker by
  rows and decoded bytes. The component keeps the longest prefix that
  fits both.
- **A restart's page holds any row a writer commits:** its byte bound
  is at least the largest row a writer can commit, never the on-demand
  read allowance (`core.load_bytes`), which bounds a party's or a run's
  loads alone. So every record its writers wrote restores
  (`../../domain/engine/restart.md`), and a restart page that comes back
  empty without moving its cursor is a bug and panics, never a loop.
- **A row that does not decode** stops the engine, as a failed commit
  does: the store holds bytes it did not write, or that broke since.

## 2. The restart's ranges

The core's restart script (`../../domain/engine/restart.md`) asks the
root for one step at a time, and the root asks the store:

- **the core's live ranges,** in the order the core restores them: the
  deployment's header first, then the settings' live ranges (defaults,
  changes, held revisions, charter versions, pending proposals' sets),
  which every later record is judged against, then the accounts' family
  in three intervals, its issued program names first (tag 4), then its
  counter, accounts and sources (stopping before the installation's
  identifier, which the component keeps), then its accounts' facts (tag 5),
  which name the accounts before them, then people's families (live
  conversations' read positions among them, ended ones never), then the
  tasks child's live families, then the core's own live rows (goals'
  projections, run proofs, calls, agents' conversations). It is a fixed
  list of intervals, in that order, paged and handed up typed. The root
  has the accounts child want its sources only once these are restored
  (`../../domain/accounts.md`, 3.5);
- **the inbox's census,** a step of its own once people's read positions
  are restored: the family of ended read positions, one row per ended
  conversation not yet erased (4), then the results' index from each
  party's read position over results (`README.md`, 4.4), parties in the census's
  order; each row joined to its ended task's record by a point read;
  never every ended task (`README.md`, section 10). It then reads each unanswered
  question's activity line by the position its live task keeps (`README.md`, 10.7),
  to rebuild the core's bounded, volatile question facts: the original
  time and words, never another durable record. The census:
  - hands up an ended read position, then its task's record, and a
    result's row only as its task's record;
  - never parts a position from its record: a page counts both against
    the asker's rows and bytes, and ends before a pair that does not
    fit, so a root asks the census for at least two rows;
  - goes on from the last index row's key. The census places each key
    it admits in the walk's order (`Census::interval`), so that a root
    that keeps its own cursors finds the interval to go on from (`README.md`, 4.3);
  - restores each ended task once per party, whichever part names it:
    neither part's progress skips a row of the other;
- **each connector's live ranges,** as the application's byte intervals,
  handed up as bytes for its codecs;
- **a key's place in the walk:** a restart's request names its ranges
  in order, and their intervals are numbered across the request in
  that order (`README.md`, 4.3), a cursor naming the interval it is in. The order a
  walk hands rows up is (interval, key), not the keys' own: an interval
  may start below the one before it (the accounts' issued names, tag 4,
  before their counter, tag 0). jig gives each key its place, so that a
  root that keeps its own cursors never compares raw keys or keeps a
  copy of the intervals. The place is the domain's, over typed keys,
  pure, beside the ranges the core names (`jig-domain-core`, so a
  domain crate pages with no io below it); the component's byte
  intervals and cursors follow it:

  ```rust
  pub fn RestoreRange::interval(ranges: &[RestoreRange], key: &Key) -> Option<u32>  // jig-domain-core; None: no interval of the walk holds it
  pub fn intervals::cursor(ranges: &[RestoreRange], key: &[u8]) -> Option<Cursor>   // jig-protocol-store, the same place over encoded keys
  ```

  `Census::interval` is the census's case of the first. The second
  finds the interval by containment in the list the walk builds, and
  `README.md`, 4.3's test pins the two together over generated keys, for each range
  alone, a census, and `[Core, Tasks]`. A page's rows ascend by place,
  a later interval or the same interval and a higher key, which the
  tests assert and no start re-checks (`../../domain/engine/restart.md`
  ). A root's "after" for a fresh request is the place of its last
  key, and a row at or below it is one already handed up;
- **what the core never restores** (turns, terminals, history, ended
  tasks, notes' entries) is never read at start.

## 3. Parties' reads

A party's read (`../../client/domain/contract/README.md`, section 4) names a
scope, a position, a direction and its bounds. The component serves each
scope as intervals over jig's families, and joins each row in the same
step by point reads, those its scope's rows carry: its owner task, live
or ended; its requester ancestry, within the domain's depth limit; its
links; the record it relates to. A task's document joins all its links.
A turn joins its owner alone; an inbox entry joins its owner, the record
it relates to and its requester ancestry for selection, without the
owner's other task links. A task's activity joins its owner alone, once
a page, and each turn's line its turn (below).

| Scope | Served from |
|---|---|
| a task, a person task, a result | the task's record, by key, with its last attempt's terminal |
| a proposal | a party's: its record, by key, with its decision; a task's: the row of its proposer's history that keeps it |
| an escalation | its decision, by the revision its task's record names, with the hold's milestone |
| a question, a reply | the task's activity line, by its position, with the line that answers a question, where its answer was heard (below), or the turn a reply was (from the transcript, `transcripts.md`, 3) |
| a turn | the task's transcript (`transcripts.md`, 3) |
| a task's activity, newest first, or the page beside one of its lines | the activity family, read backwards or, for the lines after a position, forwards (4); each turn's line with its turn, from the transcript (`transcripts.md`, 3); its owner task joined once for the page, and no row's ancestry or links, since an activity line names its task alone |
| a task's history, or a subtree's | the history and milestone families; a subtree's by the index of its delegates, task by task: the task's own rows, then each delegate's in that index's order, each task's rows in its own history's order |
| a task's ended delegates | the index of ended delegates by requester |
| an inbox group | the index of the party's inbox, by time: what needs them oldest first, updates newest first; a group, and the filter, select within it |
| a task query's live tasks | not the store's: the tasks child's live tasks, scanned in memory |
| a task query's ended tasks, naming a requester party | the index of ended tasks by requester, from the party, and the label where the query names one |
| a task query's ended tasks, naming a requester task or a parent | the index of ended delegates by requester |
| a task query's ended tasks, naming a project and none of the above | the index of ended tasks by project, from the project, and the label where the query names one: listed tasks only |
| a project's notes, a scope's notes | the notes' lines of the project's scopes, or of the one scope, filtered by description, each with its entry: by scope, then by name within it, as the lines are keyed |
| a project's spend | the funders' periods of the project |

- **A task query's ended part** (`../../client/domain/contract/README.md`, 4.1)
  is served from one index, chosen by the query's keys in the table's
  order: a requester party; a requester task or a parent; a project. A
  query naming none of these has no ended part, and its page says so.
  The keys beyond the index's prefix (the label, where it is not in it;
  the root; the phase class; the executor kind; tracked) filter its
  rows. Its rows come in the index's order: within a label, newest
  ended first from the two ended indexes; by task from the ended
  delegates', whatever order the query asks of its live part.
- **A task query's row joins** its task's record, by key: the ended
  record, or its summary once retention has shrunk it (`README.md`, 6.1); and, for
  a task flagged conversation, its requester's ended read position, by
  a point read (4).
- **A page goes on from the last row it examined,** in its scope's
  order, not from the last it returned. A page that skips rows (an index
  row whose owner is gone, 4) or leaves them out (an inbox group, a
  description's filter, a task query's filter beyond its index's
  prefix) goes on past them; a page that reached its bound
  and returned none is cut, not ended, whichever its scope and whichever
  bound it reached (rows, bytes or joins), and carries the position it
  reached, so the client reads on from the row it did not take. A page
  cut by its join bound before it examined any row is the exception: the
  row at that position needs more joins than any read allows, so a read
  there would be cut the same way, and the read answers *too linked*,
  naming the bound, never an empty page to be read again. A
  subtree's history is in its scope's order, task by task,
  its position the task and the row.
- **Joins are point reads** in the same step, priced within the read's
  bounds: a page's rows times the joins each may need. An owner lookup
  is charged one join, whether the owner is found live, found ended
  after its live record is missed, or not found: a miss decodes
  nothing, so the second probe needs no join of its own.
  A turn's is read from its transcript, and answers once its frame is read.
  An activity page's joins are its owner's, once, and one for each turn, so its
  first row needs at most the owner's and a turn's.
- **A page of a task's activity carries its turns.** Each turn's line
  comes with its turn: its spend, the last message it read, its time,
  and smith's bytes. A turn and its line commit together, so a page's
  turn lines are the transcript's frames in the same order, one each,
  and the component reads them in one walk (`transcripts.md`, 3). A turn counts its
  bytes with its line's against the read's byte bound, and one join:
  - a turn whose line and bytes together pass the read's whole byte
    bound comes with its size in place of its bytes, and counts without
    them, so no page comes back empty for a turn;
  - one that fits the bound but not what is left of the page ends the
    page before its line, and the next page starts with it;
  - a frame that does not read whole (`transcripts.md`, 3) comes as unread, and ends
    the page.
- **Single reads go by key.** Where a task's record does not name the
  key a single read needs (a question's line and its answer's, an
  escalation's hold, a task's proposal in its proposer's history), the
  domain names it there, or keeps an index (`README.md`, section 10).
- **An answer is read where it was heard** (`../../domain/engine/executors.md`, 2;
  who may read it, `../../client/domain/contract/README.md`, section 4): the
  question's record names the answer by its task and position, and the
  read joins it by a point read, as the record it relates to.
- **A read past its bounds** is cut and says so, as every load is
  (`../../domain/engine/README.md`, 5.3).

## 4. Indexes

skein-kv has no secondary index (skein's `data/kv.md`, section 1). Where a
scope's order is not a family's key order, the domain keeps an index: a
family of small rows its owner saves and erases in the same decision as
the records they point at, with keys reversed in time where a page reads
newest first. Each index is its owner's, tested in its world. `README.md`, section 10
 lists those the domain owes.

**The ended indexes,** the tasks child's, each saved in the decision
that ends a task (`../../domain/tasks/README.md`, section 3):

- *ended by requester:* key `(party, label, ended at reversed, task)`, a
  row for every ended task whose requester is a party;
- *ended by project:* key `(project, label, ended at reversed, task)`, a
  row only for an ended task flagged *listed*. The flag is the task's,
  set from its template when it was made, so the engine reads no
  template to index.

Each serves a task query's ended part from its prefix, the party or the
project, then the label (3). A filter beyond the prefix scans the
index's pages, which may come back cut. A task moved to a party
(`../../domain/tasks/changes.md`) is indexed by the requester it has
when it ends.

**Read positions over conversations,** the people child's: one per task
flagged *conversation*, its requester's, while that requester is a
party (`../../domain/people/README.md`, section 5):

- *a live conversation's,* in a family keyed `(party, task)`, loaded at
  start, so their bound in memory stays the live conversations, one
  each;
- *an ended one's:* the decision that ends the task moves its position
  to the family of ended read positions, keyed `(task, party)`, read on
  demand. A read of the conversation and a task query's row join it by
  a point read (3); the inbox's census reads the family through
  (2);
- *erased with the task:* by retention, as it shrinks a kept task
  (`README.md`, 6.1); or in the decision that forgets a task forgotten once read
  (below).

**An owner erased first.** When a task's record is erased before the
records its index rows name, as a task forgotten once read is
(`../../domain/people/README.md`, section 5; `README.md`, 6.1), its decision erases the rows
its record names whole: its result's update, its rows in the ended
indexes, and its ended read position. Its replies' updates stay, since a
conversation's replies are unbounded, a decision is bounded, and the
task's record names only its newest; they go when retention erases the
reply lines they name (`README.md`, 6.1). Until then a page skips a row whose owner
is gone, counting it within its bounds, so nothing of a forgotten task
shows and no reply read comes back unread.

**Reading backwards:** a family whose positions are dense (activity)
reads newest first by computing forward intervals from its last
position, which its owner task's record names; a family read newest
first by time is keyed reversed. skein-kv's load runs forwards only
(skein's `data/kv.md`, section 12). The lines after a position (a page beside
a line, `../../client/domain/contract/README.md`, section 4) walk the same dense positions
upward, from the one after it, stopping at the same bounds.
