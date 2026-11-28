# temper's parts of task rows

Provisional, 2026-11-17. From `../../draft/page-round-trips.md`, sections
4, 5 and 8, milestone R2 of `../../draft/page-round-trips.execution.md`,
on jig's mechanism (jig's `client/domain/contract/parts.md`). It gives
temper's three kinds of part: what each holds and is made from, how the
engine makes them, what the pages ask and stop reading for them; and how
the pages read a list's ended group again by its mark. The changes'
rows naming their goal, chat, milestone and signers are `records.md`,
*A project's changes*; the trials' rows' run wait, `../../domain/trial.md`,
section 6.

## 1. In one page

- **Three kinds** (2), owner `FORGE`, at `CLIENT_VERSION`: a goal's, its
  newest report's headline, the milestone that gathers it and whether it
  holds a goal branch; a chat's,
  its document's description; a milestone's, whole or so far, its
  totals and its goals as its card draws them.
- **Their records** (3) are the forge family's, from its version 29.
- **Made by the engine's root** (4) from the core's live state, the
  forge's rows and loads of notes and ended tasks, keeping memory-only
  what saves loads, and marked by their inputs.
- **The pages ask them** (5): the goals', chats' and milestones' queries
  name their kind, and the reads per row they replace go.
- **An ended group is read again only when its mark grows** (6).

## 2. The kinds

`temper-client-protocol-wire` names each kind's number and bytes, one
constant each; the engine declares them in the core's configuration
(jig's `domain/engine/views.md`, 2, `PartKind`, its connector `FORGE`)
and the client names them in its queries' `parts`, owner `FORGE`,
version `CLIENT_VERSION`.

| Kind | Number | Bytes | Inputs | Holds |
|---|---|---|---|---|
| `GOAL_PART` | 1 | 1,024 | `notes` | its newest report's headline and when it was written; the milestone that gathers it; whether it holds a goal branch |
| `CHAT_PART` | 2 | 1,024 | `notes` | its document's description |
| `MILESTONE_PART` | 3 | 32,768 | `line`, `delegates`, `delegate_notes`, `records` | whole or so far; its totals; at most 16 of its goals, each its line and its report's headline; how many more |

- **Within the window:** the browser's and the terminal's windows are 16
  rows (jig's `client/domain/client/README.md`, section 11), so 16 rows,
  each with its line and a milestone's part at its bytes, fit the
  client's document bytes (1 MiB). temper's tools name no part.
- **Of any row:** a kind is made the same way whatever the row's task: a
  goal part of a task with no report holds none. A milestone part of a
  task that is no milestone of the row's project is `failed`,
  `unreadable`.

## 3. Its records

The forge family's, from version 29, beside its other records
(`records.md`), each in the forge's envelope at `CLIENT_VERSION`:

```
record GoalReport { headline: bytes 512 at: u64 }
versioned record GoalPart { report: option GoalReport milestone: option u64 own_branch: bool = false }
versioned record ChatPart { document: option bytes 512 }
# line: jig-client-protocol-wire's TaskLine, encoded at jig's VERSION, with no parts.
record MilestoneGoal { line: bytes 16384 headline: option bytes 512 }
versioned record MilestonePart { whole: option u64 totals: MilestoneTotals goals: list 16 MilestoneGoal more: u32 }
```

- **`GoalPart`:** `report`, the newest note in the task's scope whose
  description begins `report: ` (`../../domain/agent/agents.md`, *A
  goal's report*), newest by `changed`, then the greater number: its
  `headline`, the description past `report: ` to its first line's end,
  and `at`, the note's `changed`; none with no such note. `milestone`,
  the task's requester where that requester is a milestone of its
  project (`../../domain/milestones.md`, section 7, as the root's
  `forge_route::milestones::of` finds it), else none. `own_branch`,
  whether the task holds a goal branch (`../../domain/forge/branches.md`,
  1), as the forge domain's holds tell it (`holds_goal_branch`): a
  branch its repository's adoption prefix names as a goal's, under any
  root and number, held by the task, so that a goal beneath a milestone
  and a prefix of several segments count, and a goal whose changes land
  one by one holds none. It is the engine's fact, never read from a
  path, and made afresh with each answer, as the part's other fields
  are: a goal takes its hold as it is made and keeps it for its life
  (`../../domain/forge/branches.md`, 1), so no mark of its own moves it.
  A part of an engine before the field reads `false`.
- **`ChatPart`:** `document`, the description of the task's
  lowest-numbered note whose description doesn't begin `work:`
  (`../pages/chat/document.md`, 1), whole; none with none.
- **`MilestonePart`:**
  - `whole`, its document's `concluded.at`
    (`../../domain/milestones.md`, section 3), none while so far;
  - `totals`, as the read `Milestone` answers them (`records.md`, *A
    milestone's totals*; `../../domain/milestones.md`, section 7), exact
    over all its goals, its `reds` counted as the part is made and never
    kept with an ended milestone's cached totals
    (`../../domain/red-findings.md`, section 10);
  - `goals`, its delegates, the live first by priority, the lower first,
    then the lower number, then the ended by newest `ended_at`, then the
    lower number; at most 16, and only while the record fits its kind's
    bytes. Each is its `line`, as jig's task query of
    `TaskQuery { project, parent: Some(M) }` gives it, with no parts, and
    `headline`, the goal part's `report.headline` of that goal;
  - `more`, the goals it doesn't list: its totals' `done`, `failed`,
    `closed` and `live` together, less those it lists.

## 4. Made by the engine's root

- **Asked once per answer** (jig's `domain/engine/views.md`, 2): the
  core's `parts::Ask::Lines` for the forge's connector reaches
  `temper-engine-domain`'s root, which makes the parts in a module of
  its own, `route/parts.rs`, and answers `parts::Up::Lines`: in the same
  decision where all it needs is in memory, else after its loads. It
  encodes each record itself, synchronously, through a pure encoding in
  a crate below it (the root can't name the service's vocabulary, which
  names the root): the family's records as `temper-client-protocol-wire`
  encodes them, a milestone goal's line through jig's own translation of
  a `TaskLine` at jig's `VERSION`, made public where jig keeps it, never
  copied. The service's family vocabulary
  (`temper-engine-service-vocabulary`) re-exports that encoding and
  makes no second one.
- **From memory:** the core's live tasks (a goal's requester; a
  milestone's live delegates and their lines; a live milestone's
  `concluded`) and the forge's milestone rows, its totals made as
  `route/milestones.rs`'s `read` makes the read `Milestone`'s.
- **From loads,** through the root's archive reads (`archive.rs`), as its
  note reads and ended task reads load: a goal's notes by description in
  its scope's index; a chat's notes, its scope's first entries; a
  milestone's ended goals, its ended delegates' index (jig's
  `protocol/store/loads.md`, 3) joined as the core's ended read of
  `TaskQuery { parent: Some(M), phase: Ended }` joins them, its first
  four pages; an ended milestone's `concluded`, from its ended row. A
  load that can't be placed, the root's reads full, answers that part
  `failed`, `busy`; a load failing or finding nothing to make it from,
  `unreadable`.
- **Kept, memory-only:** the root keeps what it loaded, by task: a goal's
  newest report or none, a chat's document or none, a milestone's ended
  goals' lines and its `concluded`, with the ended count they were made
  at. At most 4,096 entries, the least recently used dropped first, and
  none across a restart. A note saved or erased in a decision the root
  routes drops the entry of the task in whose scope it is; a milestone's
  ended goals are made again when its ended count moves, and a goal's
  headline in them comes from that goal's own entry.
- **Marked by its inputs** (jig's `domain/engine/views.md`, 2): a goal's
  and a chat's part by their notes; a milestone's by its line, its
  goals' lines and their notes, by its forge row, and by its reds: the
  root sends the forge's `parts::Up::Marked { tasks }` naming M wherever
  the forge's row of M changes (its last landing, its end), and wherever
  a decision writes a finding whose blame names M, before the write or
  after (`../../domain/red-findings.md`, section 10).
- **Visibility** is its row's (jig's `client/domain/contract/parts.md`,
  4): a part reads only within its row's project.

## 5. What the pages ask

`temper_client_domain_forge::parts` holds the client's side: `asks`,
each kind's `PartAsk`, and `goal`, `chat` and `milestone`, each the
row's part of its kind decoded from its `TaskLine`: `Some` for a value
under `FORGE` at `CLIENT_VERSION` that decodes, `None` for anything else.

- **The goals' query** (`../pages/work.md`, 1, *Goals*) names `GOAL_PART`,
  its live watch and its ended reads alike. A row's headline and its
  byline take its part's report, a goal's milestone (`Milestones::of`,
  `../pages/milestone.md`, 1) its part's `milestone`. The report reads,
  one per row, go, and so do the reads of a goal's requester to learn
  whether it is a milestone. Home's rows are the same rows.
- **The chats' query** (`../pages/chats.md`, 1) names `CHAT_PART`; a row's
  document is its part's. The project's notes by page go.
- **The milestones' query** (`../pages/milestone.md`, 1) names
  `MILESTONE_PART`: a drawn milestone's whole, totals, goals and `more`
  are its part's, its goals drawn in the order they joined, as before.
  Its document read, its totals read, its goals' watch and their ended
  pages go, and a milestone's goals outside the project's goal windows
  still draw. A milestone's page (`../pages/milestone.md`, 3) lists every
  goal and keeps its own reads, but its goals' query names `GOAL_PART`,
  so its reports, one per goal, go.
- **A goal's page** (`../pages/goal.md`, 2): the query that finds its
  own line names `GOAL_PART`, and the page reads whether the goal lands
  whole from that line's part, `own_branch`. It keeps the part last
  decoded for its line as a list's row does (below); with none decoded
  yet, it draws as a goal with no branch. A milestone's page's own line
  asks no part.
- **A part not made,** `not_served`, `unknown` or `failed`, is no answer
  for its row (`../pages/README.md`'s opening): the row keeps the part
  last decoded for its task in the list that holds it, for as long as the
  page holds that list, and draws from it, its report, its milestone, its
  document, its totals and goals alike. A delivery or a read whose part
  decodes replaces it whole, one saying the goal has no report or no
  milestone included. The engine makes a list's rows with `failed`,
  `busy` parts where its loads are full (jig's
  `client/domain/contract/parts.md`), so a running project's list is
  delivered now with its parts and now without them; what the page
  draws doesn't move. Only a row whose part has never decoded in the held
  list is drawn as with no report, no document, or a milestone with
  nothing read. The client reads nothing in its place, and the row's next
  delivery or read brings it. The kept parts are bounded by the list's
  rows: a row that leaves the list takes its part with it, and leaving
  the list (`../pages/milestone.md`, 1, its lifetime) drops them all.

## 6. Ended groups by their mark

- **Read again only when the mark grows:** the goals', chats' and
  milestones' lists, `HasGoals`'s milestone fact
  (`../pages/milestone.md`, 1) and a milestone's page's goals (its
  section 3), read their ended group's first page on
  opening, and again only on a delivery of their live watch whose
  `ended_mark` is greater than that of the ended read they hold (jig's
  `client/domain/contract/parts.md`, 6): never for the watch's first
  delivery, nor for a live row's change alone.
- **A read under way** when the mark grows is followed by one more read
  once it answers, where its answer's mark is below the delivery's.
- **Older pages,** "More goals" and "More closed chats", are as before.

## 7. Open questions

- **Kept parts across a restart:** a restart makes every part from loads
  again. Default: nothing kept, as the draft's section 8 keeps nothing.
- **A goal page's own report** (`../pages/goal.md`): it reads the whole
  report, beyond a row's headline, as before. Default: it stays a read.
