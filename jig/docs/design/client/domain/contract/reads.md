# The client and the engine: reads

## 1. Reads

A read asks for one document, or one page of a list, and is answered
once, with the document or why there is none. It decides nothing and
needs no key. Its answer is made from the engine's state as it is, which
may run ahead of the store by the commits in flight, so it is held as an
output of a decision that made no commit: tagged with the last commit
made, and released once that is durable (`../../../rails.md`, 1.2). What a
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
| `inbox`: a group of it, a page, filtered by project, kind or root | needs: live state, every page; updates: live state, then the store past the engine's limit |
| `tasks{query}`: a task query's page (1.1) | live state, then the store's indexes of ended tasks |
| `waits{project, kind, party, due}`: a project's live waits, a page (1.2) | live state |
| a project; its spend, a page; its notes, a page | live state; the store; the notes child |
| the deployment | live state |
| `settings`; `setting{kind, scope, number, revision}`: one setting's value whole, as the newest reads it, or, for a charter, as a held revision reads it, or, with `proposal`, as a pending proposal of settings would set it, or, of an application's kind, its state and size alone; `revision{n}`: a revision's record and its items, as it set them, an application's with their sizes alone; `revisions`: its records, a page, newest first | live state; the store's history, for a revision no longer held (`../../../domain/engine/settings-store.md`, 1) |
| an application's read, naming its owner, its scope and its window (`watches.md`, 1.5) | its owner, the connector, through the engine's root |

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
  delivers its retained document once, then ends the watch (`watches.md`, 1.2). A
  client waiting for a condition tests this first document too: a condition
  already true answers at once, and an ending alone proves no condition.
  A task read returns its live state or retained terminal history under
  the party's current visibility. A retained load
  waits for the preceding commits to be durable and checks current
  sign-in and visibility when it completes; it never restores a retired
  row into the live set (`../../../domain/engine/decisions.md`, 1.2 and 1.3).
- **A history page's rows** are a task's changes, its attempts and its
  holds (`../../../domain/tasks/lifecycle.md`, 12), in the store's
  order: for each task, its changes, then its attempts, then its holds.
  On jig's wire, from version 23, where they replace version 22's
  milestones:

  ```
  record AttemptLine { task: u64 attempt: u64 claimed: option u64 started: option u64 ended: option u64 end: option RunEnd phases: option u32 }
  record HoldLine { task: u64 position: u64 at: u64 before: PriorPhase why: HoldCategory source: option Part ending: option ChipEnding }
  record HistoryRowAttempt { value: AttemptLine }
  record HistoryRowHold { value: HoldLine }
  enum HistoryRow { change: boxed HistoryRowChange, attempt: HistoryRowAttempt, hold: HistoryRowHold }
  record EndedValueEscalation { task: Task decision: EscalationDecisionRecord hold: option HoldLine }
  ```

  An attempt's times are wall nanoseconds, none where the store did not
  record them (an attempt restored without its opening evidence,
  or still live); `end` is how its run ended, as the activity's
  `run_ended` line says it, `suspended` from version 40
  (`../../../domain/engine/executors.md`, 6.2); `phases` how many phase changes it took. A
  hold's `before`, `why`, `source` and `ending` are a held chip's
  (`../objects.md`, 5). An escalation's `hold` is the hold row at its
  task's latest hold. The activity's `milestone` line stays, its
  `position` the task's phase change it marks, which at a hold is that
  hold row's `position`.
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
  next, or say there is none (`../../../domain/engine/decisions.md`, 1.3).
- **A deleted project's read is gone.** A read of a project the
  people child marks deleted, while its deletion lasts and after it,
  answers *gone* to any signed-in party, where a project absent or
  hidden from them answers unknown; nothing else of it is read
  (`../../../domain/people/acts.md`, 2, *Marked deleted*). Its other
  reads, requests and watches are unknown, as a project never made.
- **Refused** for what the party may not see, as unknown; when the
  engine's loads are full, as busy; when the sign-in has ended, as
  signed out (../client/sign-in.md, section 1); in a version the engine does not
  serve, as version not served (`README.md`, section 6); where the row at the read's
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
  (`../../../domain/engine/executors-transcripts.md`, 1; `../../../protocol/store/loads.md`, 3).
- **A page beside a line** is how a window that let lines go reads them
  again: an activity line's position is a fact of the line, dense and
  never reused, so it names a place where no cursor was kept. Its answer
  is an activity page. Earlier, its `before` continues it as any page's
  does and its `after` is none; later, it holds the lines just after the
  position, its `after` says that more remain and its `before` is none. An
  empty page is the answer past either end. It is read with the access of
  `activity{n}`.
- **A turn** is named by its task, its attempt and its number within it
  (`watches.md`, 1.3). It is read once and never watched: a committed turn does not
  change. It is read with the access of `activity{n}`. Its answer is:
  - *the turn:* smith's bytes, which the client decodes as it decodes
    activity's turns (../client/conversations.md); to a party other than the
    person who requested its task, withheld: its preferences section and
    its private tools' calls projected as `../../../domain/engine/briefs.md`, 2.1
    says, here and on activity's pages alike;
  - *unknown,* when it is not there or the party may not see it;
  - *gone,* past its transcript's horizon, as a dropped transcript is
    (`../../../protocol/store/transcripts.md`, 5, and `../../../protocol/store/growth.md`, 1.1);
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
    for a restore (`../../../domain/engine/settings-application.md`, 1, and `../../../domain/engine/settings.md`, 10); an item of a
    project's own policy with the policy it set and the one it replaced;
  - *forgotten,* a revision made but erased past the history
    (`../../../domain/engine/settings-store.md`, 1), naming the oldest kept;
  - *unknown,* past the newest;
  - *busy,* as any read.
- **`revisions`** is the history a page at a time, newest first, from
  the newest or a cursor: each row a record without values, its items
  as settings and what became of each, cut at the page's bound with how
  many more. The last page says whether older revisions were forgotten.
  It is read, never watched: the `settings`' mark says a revision came.

### 1.1 Task queries

- **A task query** selects tasks by a fixed set of keys, any conjunction
  of them: project; requester (a party, or a task); root; parent;
  label; phase class (working, waiting, held, closing, ended); executor
  kind; tracked. It orders by number, ascending; last change or last
  activity, latest first; priority, ascending, tracked tasks before
  untracked; or ended at, latest first, its live group, which has no
  end, by number ascending. Ties go by number, ascending. It reads
  within a window, by cursor. Its rows are task lines (`documents.md`, 1). It is read
  a page at a time, and watched as `tasks{query}` (`watches.md`, 1.1). A list of a
  party's conversations, of a project's goals by priority, of a task's
  ended delegates, are each a task query.
- **Live first, then ended,** in two groups, each with its own cursor.
- **Parts and the ended mark:** a query may name an application's parts,
  which each row carries, and its answers carry its ended group's mark
  (`parts.md`).
- **Live tasks are scanned,** in the order asked: the tasks child keeps
  no secondary index, and its live tasks are bounded by its limits, so
  any conjunction of the keys costs no more than one scan. The fixed set
  bounds what marks a watch (`watches.md`, 1.2) and what crosses the wire, not the
  filtering.
- **Ended tasks are found through the store's indexes**
  (`../../../protocol/store/keys.md`, 1.4, and `../../../protocol/store/loads.md`, 3 and 4), in each index's order,
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
  (`../../../protocol/store/growth.md`, 1.1).

### 1.2 A project's waits

From `temper/docs/design/draft/bounded-waits.md`, 9: every live wait in
a project, nearest bound first, for a person following a stalled tree
and for the tools that replace the team's watchers.

- **Which waits:** every wait of each live task of the project that the
  party may read, as its document lists them (`../../../domain/engine/waits.md`,
  4), a procedure's reported ones among them. Each is listed once, at
  the task it waits at: a line's first wait, which may be a task's in
  its subtree, is not listed again at the line's task, so a wait shows
  at its own task and at no ancestor whose line summarizes it.
- **Filters,** any conjunction, each optional:
  - *kind:* one of the twelve (`../../../domain/engine/waits.md`, 3);
  - *party:* a party's waits: an entry waiting at the party or at a
    role it holds in the project, and a wait whose `hears` is the party
    or such a role. A task holder is no party's;
  - *due:* a duration; the waits whose bound is at or before the read's
    *now* plus it, those already past included. A wait with no bound is
    none's due.
- **Its now:** the engine's wall time as it serves the first page. The
  page carries it, and every later page of the same read carries and
  uses the first's, kept in its cursor, so that one read's pages order
  and filter by one instant.
- **Order,** in three groups:
  1. *overdue:* a wait whose bound's outcome is *overdue* and whose bound
     is at or before *now*;
  2. every other wait with a bound;
  3. the waits with no bound.

  The first two by bound, earliest first, then by task number; the third
  by task number. Waits of one task with the same place keep the task's
  own order (`../../../domain/engine/waits.md`, 4).
- **A row** is the task's chip and the wait whole, the record a task's
  document carries (`documents.md`, 1, *Its waits*). A page is its
  rows, its *now* and a cursor for the next, or none after the last.
- **Paged** as any page (`../../../domain/engine/decisions.md`, 1.3),
  within `documents.rows` rows and the wire's bytes. The cursor is
  opaque, as a task query's: it binds the project, the filters and the
  first page's *now*, and continues after the last row's place (its
  group, its bound, its task and its place in the task's order). A wait
  that comes or goes between pages is shown or skipped by its place, as
  a live task is by a task query's cursor; a cursor of another read is
  malformed.
- **Live tasks are scanned,** as a task query's live group is (1.1):
  one scan of the project's live tasks, each task's own waits computed
  as its document computes them, the page's window kept as the scan
  goes. No secondary index and no stored state; nothing of it reaches
  the store.
- **Read, not watched:** a wait's place moves with the clock as well as
  with commits, so a watch would need marks no commit makes. A client
  reads it again when it wants it current; each answer is current as it
  is made.
- **Refused** as any read (1): unknown for a project the party holds no
  role in, as a task query by project is.
