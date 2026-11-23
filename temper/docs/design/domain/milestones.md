# Milestones

Provisional, 2026-11-16, from `../draft/milestones.md`, sections 3, 6,
7 and 8. A milestone is a set of goals, as a goal is a set of changes:
a name, the goals in it, and done once every one of them is done. Each
goal lands on its own, by its own way. temper offers the primitive and
nothing around it: a person or their scripts make milestones through
`temper ctl` (`../client/ctl/milestones.md`). Underneath, a milestone
is jig's gathering (jig's `domain/tasks/gatherings.md`). What the pages
read and draw of it, the states of section 4 of the draft and its
section 5, is `../client/pages/milestone.md` and the pages it names.

## 1. In one page

- **A tracked task of its own identity:** jig's gathering, made from
  the stock's template 8, labelled 4 (section 2). Its tree, its root,
  its pool, Pause and Close are jig's.
- **Its spec's words** are its name, then its words (section 3). Its
  maker is its requester, the party whose `Create` made it.
- **Its goals are its delegates,** made by a party with jig's
  `Delegate` beneath it, funded from its budget, each a goal of
  template 2 that may wait on other tasks (section 4).
- **Its goals ask its maker:** a goal's question to its requester, the
  milestone, reaches the maker's inbox through jig, and `decided` judges
  for the maker (section 5).
- **Whole is jig's `Conclude`;** it ends done once whole and every goal
  in it has ended, complete or ended by its goals' endings, or closed
  when cancelled (section 6).
- **Reads:** milestones are a task query by template 8's label; its
  facts its task's document; its goals its delegates; its totals the
  forge family's read `Milestone` (section 7), from jig's records and
  the forge's milestone row (section 8).
- **The status agent** reads the live and newly ended milestones, and
  names a goal by its milestone (section 9).
- **Kept out of goals:** no goal list, count, way, forge issue or public
  goal takes a milestone (section 10).

## 2. Its identity

- **Template 8, `milestone`,** in the stock
  (`../protocol/configuration/stock.md`): executor chosen; a report,
  template 1's bound; the creator's authority, `shallower` 1, the
  deepest a party may give a task it makes (jig's `domain/authority.md`,
  section 5; a template of `shallower` 0 can't be filled: jig's
  `client/domain/client/helpers.md`), so that a goal beneath it is one
  level shallower than a goal its maker starts alone, within the stock's
  32 levels; the
  creator's budget, given; no deadline; tracked at priority 1;
  `ended_by` `Requester`, not a conversation, listed, kept; label 4; no
  grace, LLM choice or resources. jig checks it as a gathering's
  template (jig's `domain/tasks/gatherings.md`, section 2), and a
  creator fills its executor with the gathering's,
  `Procedure { connector: RECURRING, code: GATHERING }`.
- **Label 4** marks a milestone, beside 1, a chat, 2, a goal, and 3, a
  standing role (`agent/standing.md`, section 2). Lists find milestones
  by template 8's label as served in `me`, as Goals find goals by
  template 2's (`../client/pages/work.md`, 1, *Goals*).
- **A milestone is a task whose executor is jig's gathering.** temper
  makes no other gathering, so the executor is the test every part
  applies to one task, the engine's and the client's alike, with jig's
  constants and no settings read; the label serves the lists.
- **Its tracked slot:** it takes one of jig's live tracked tasks and
  counts among its project's goals for jig's limits, as a goal does
  (jig's `domain/tasks/README.md`, section 6), and nowhere of temper's
  (section 10).
- **Its policy:** a project's policy may override template 8's budget
  and priority, as any template's (jig's `protocol/configuration-templates.md`,
  1); the flags, label and tracking are its identity.

## 3. Its name, words and maker

- **Its spec's words:** its name on the first line; then, where it has
  words, a blank line and its words. So its chip's title is its name
  (jig's documents' first line).
- **Its name,** for a person: 1 to 60 characters, Unicode scalar values,
  on one line, with no control character. `temper ctl` refuses another
  before sending (`../client/ctl/milestones.md`, 2). "Store shape".
- **Its words,** optional, what it is for: "The store's new shape, from
  its draft". Their bound is the spec's.
- **`milestones::facts`,** one pure function of the client domain, from
  a milestone's task document (jig's `client/domain/contract/documents.md`,
  1): `Facts { name, words, maker, whole, made }`. `name` the spec's
  first line; `words` what follows the first blank line, none where
  nothing does; `maker` its requester, a party; `whole`, `Option<Wall>`,
  its document's `concluded.at`; `made` its document's `made`. A
  document whose spec has no blank line after the first holds no words;
  a first line past 60 characters, made by other means than `temper
  ctl`, is kept whole as the name. `temper ctl` and the pages read it
  the same.
- **Its maker** is its requester, the party whose `Create` made it.
  Whoever adds goals, a maintainer among them, never changes it.

## 4. Its goals

- **A goal joins at its start,** and belongs to one milestone at most:
  a party's `Delegate` beneath the milestone, with one task, a goal
  filled from template 2 (`goal start --milestone`,
  `../client/ctl/milestones.md`, 3). jig refuses a batch beneath a
  milestone once it is whole, naming when (*concluded*), and never moves
  a goal into or out of one (jig's `domain/tasks/gatherings.md`, 3).
- **Its requester and funder are the milestone;** it is the
  milestone's delegate, its depth one below it. Its person, whom it
  asks, is the milestone's maker (section 5). It decides, lands and
  reports as any goal of its way does, and keeps its forge issue
  (`forge/issues.md`).
- **Its authority** is template 2's fill from the party's creating
  offer, narrowed to what the milestone has left: delegation depth at
  most the milestone's less one; its delegation's tasks at most what the
  milestone's tasks have left less one, its task document's `tasks_left`
  (jig's `client/domain/contract/documents.md`, 1), and none where its
  depth is zero; its deadline no later than the milestone's where it has
  one. jig counts every task the milestone's subtree makes against its
  allowance, reserving none, and admits a delegate whose tasks and one
  more fit what is left (jig's `domain/authority.md`, 5), so a goal
  narrowed to what is left less one leaves the next goal room, each
  later goal's allowance one less, as a change's is narrowed to its
  goal's (`agent/tools-change.md`, 1). jig checks
  it against both the party's role and what the milestone has left,
  refusing at admission (jig's `domain/authority-checks.md`, 1.4); the tool
  prints that refusal.
- **Its budget** is carved from the milestone's when the goal is made,
  waiting or not: an amount, or `AsAllowed`, resolved against what the
  milestone has left (jig's `domain/authority-checks.md`, 1.4). So the
  milestone's budget must hold every goal made at once. `AsAllowed`
  means what it means for any goal, all that fits: a goal started so
  takes all the milestone has left, within the role's spend, and a
  second started so after it is refused for the milestone's budget. A
  script that starts several goals in one milestone gives each its
  amount, `--budget N` (the user, 2026-11-17: no hidden share, which
  would give a script's goals less and less in an order nobody chose).
- **Its waits:** its batch's dependencies, each a task the party
  reaches, a goal or a person task, in a milestone or not. One done adds
  no wait; one failed or cancelled refuses the start, naming how it
  ended; each judged at admission (jig's `domain/tasks/README.md`,
  section 4). temper judges none of them.
- **Its deadline** counts from when it is made, waiting included. A
  goal may have none where its template and the milestone allow it:
  template 2 takes the creator's deadline or none, and a milestone has
  none unless its maker gave one. A goal beneath a milestone with a
  deadline ends by it.

## 5. Its goals ask its maker

- **A question:** a goal whose requester is a milestone is a goal a
  task requested: its brief says `[Requested by task M]`, and it asks
  its requester with `message`, in the question form, as a goal a chat
  calls asks its chat (`agent/agents.md`, section 2). jig hands the
  question to the milestone's requester, the maker, as an entry of
  their inbox (jig's `domain/tasks/gatherings.md`, section 4), a goal's
  question under Needs you as any; their answer reaches the goal as its
  requester's.
- **`decided`** judges for the maker: the person a goal's `decided`
  reads Away for and writes its row under is, for a goal whose requester
  is a milestone, the milestone's requester where that is a person
  (`forge/attention.md`, section 5, *The caller's person*), as it is a
  chat's person for a goal the chat calls. Its Ask says to ask the requester with `message`, as for
  any task requester holding `message`. Its Decided for you rows reach
  the maker as any goal's.
- **Escalations and proposals** pass the milestone to its requester,
  as a procedure's delegates' do (jig's `domain/tasks/gatherings.md`,
  section 4): nothing of temper's.
- **A milestone made by the deployment** has no person: its goals'
  questions wait in its inbox and `decided` writes no row, as for any
  task requester with no person.

## 6. Its end

- **Whole** is its maker's word that every goal is in: jig's
  `Conclude`, kept for good as its document's `concluded`. Whole can't
  be undone.
- **It ends done** in the decision where it is whole, no goal of it
  lives and it is not held: at `Conclude` with no live goal, or when its
  last live goal ends. Its report's words are jig's,
  `Concluded: D done, F failed, C cancelled.`.
- **Its outcome,** from its ending and its goals' endings: *complete*,
  ended done with no goal failed; *ended*, ended done with some failed;
  *closed*, cancelled, by Close or a cancel of its tree. A goal closed
  (cancelled) never keeps it from completing, and nor do the reds after
  landing its goals owe (section 7, `reds`; `red-findings.md`, section
  10).
- **Its count,** "7 / 8": its goals done, then its goals done, failed
  and live, the closed ones left out of both, as jig's task line counts
  `delegates_done` and `delegates_total`.
- **Pause, Resume and Close** are a task's: a placed hold over its tree
  and its lifting, and a cancel of its tree (`../client/pages/task.md`,
  section 4). What landed stays.

## 7. Reads

- **The milestones:** a task query of the project by template 8's
  label, tracked: its live group watched, its ended group read by when
  each ended, as Goals reads goals (`../client/pages/work.md`, 1,
  *Goals*). `temper ctl milestone list` reads one group a call.
- **A milestone's facts:** its task's document, read by jig's
  `Read::Task`, and `milestones::facts` (section 3).
- **Its goals:** its delegates, live and ended, by jig's task query with
  its number as their parent; both groups come in task number, the
  order they joined, and merge so, the live row winning a number both
  hold, as a goal's page merges its tasks (`../client/pages/goal.md`,
  section 2). The ended group is read again as its mark grows.
- **On a list's rows:** the pages draw a listed milestone's facts, goals
  and totals from its row's milestone part, and a goal's milestone from
  its row's goal part, with no read of their own
  (`../client/forge/parts.md`, 3 and 5); a milestone's page reads them as
  above (`../client/pages/milestone.md`, 3).
- **A goal's milestone:** `milestones::of`, one pure function of the
  client domain, from a goal's task line: its goal part's `milestone`,
  the goal's requester where that is a milestone, which the engine names
  (`../client/forge/parts.md`, 3). A goal's requester alone groups it,
  with no read of its tree.
- **Its totals,** the forge family's read `Milestone`
  (`../client/forge/records.md`): `MilestoneRead { project, task }`, in
  the project's scope by a party with a role there, answered with
  `MilestoneTotals` or `not_milestone`, for a task that is not a
  milestone of the project, or one whose ended record is gone.
  `MilestoneTotals`:
  - `task`, u64;
  - `done`, `failed`, `closed`, u32: its goals ended done, failed and
    cancelled, jig's count of its delegates' endings;
  - `live`, u32: its goals live;
  - `ended`, option of a wall: when it ended, its completion where it
    ended done;
  - `last_landing`, option of `MilestoneLanding { at, goal, change }`:
    the forge's (section 8), none where none is known;
  - `spent`, u64: its own spend, its ended goals' settled spend, and its
    live goals' own and settled spend below them, never a reservation:
    `spent + spent_below` of the milestone, plus `spent + spent_below`
    of each live goal (jig's `domain/authority-budgets.md`, section 1);
  - `reds`, u32: the uncleared red findings of its project blamed on
    its goals, each once, whatever their state (`red-findings.md`,
    section 10), the forge's `Domain::reds_owed`.

  A live milestone's are composed at the root from jig's live records
  and the forge's row; an ended one's from its ended record as jig
  stored it, or the forge's row (section 8). Its `reds` are counted at
  each read from the forge's findings, live or ended, and kept nowhere.
  A page that has not read them says what it holds, and never
  "complete".

## 8. The forge's milestone row

What the forge keeps of a milestone, since jig keeps no landing:

- **Keyed by its task,** in a stored family of temper's store, one row
  a milestone: its `project`; its `name`, its spec's first line, at
  most 256 bytes, cut at a UTF-8 boundary, written with the row, so
  that an ended milestone is named without jig's record;
  `last_landing`, an option of
  `MilestoneLanding { at: Wall, goal: u64, change: u64 }`, and `ended`,
  an option of its end: when, its outcome (section 6), its counts done,
  failed and closed, and its `spent`, as section 7 sums them at the end.
- **Its last landing:** each landing into a repository's landing branch
  (`forge/adoption.md`, 8) of a change whose goal is in a milestone sets the row's
  `last_landing` to that landing's time, the goal and the change, where
  it is newer than the row's. A change knows its milestone from its
  registration: the root names its goal, its nearest tracked ancestor
  (`forge_route`'s `tracked_ancestor`), and that goal's requester where
  it is a milestone; the change's row keeps both, the milestone with a
  default of none.
- **Its end:** the decision that ends a milestone gives the forge its
  end, from the tracked tasks' projection the root sends the forge for
  goals (`forge/issues.md`), which carries a milestone marked as one: the
  forge records its row's `ended` and makes it no issue.
- **Bounded:** at most 256 rows a project, the row of the milestone that
  ended first dropped first, a live milestone's never: where a project's
  live milestones pass 256 (jig's `project_goals` is configurable), it
  keeps every live row and no ended one, the bound being the greater of
  256 and its live count, and a restore loads such a set as it is; it
  falls back to 256 as those milestones end. A dropped row
  reads as none known: `Milestone` answers an ended milestone whose row
  is gone from its ended record, with no landing.
- **Read by** `Milestone` (section 7) and the status agent's input
  (section 9), in memory as the forge's other rows are.

## 9. The status agent's input

- **Its live goals** (`forge/attention.md`, section 3, *Its words*): a
  goal in a milestone is named by its milestone first, `NAME › TITLE`;
  a milestone is never one of its live goals.
- **Its milestones,** after its live goals: the project's live
  milestones, then those that ended since the last brief, each a line,
  at most 8 of each, the newest activity first:

  ```
  Milestones:
  - Store shape: whole; 7 of 8 goals done; last landing 2026-11-16T21:38Z
  - Search: complete at 2026-11-16T21:40Z; 5 of 5 goals done, 1 closed; last landing 2026-11-16T21:40Z; 2 reds owed
  - Moves: open; 2 of 4 goals done, 1 failed
  ```

  A live milestone's word is `open` or `whole`; an ended one's
  `complete`, `ended` or `closed`, with `at` and its end, UTC to the
  minute. Then `D of N goals done`, its count (section 6), `, F
  failed` where some failed, `, C closed` where some were; then `; last
  landing TIME` where one is known; then `; R reds owed`, `; 1 red
  owed` for one, where its `reds` (section 7) are above zero. The part
  is absent when the project has none.
- **Its instructions** gain a sentence: "Name a goal by its milestone
  where it has one, and say when a milestone completed and when its
  last goal landed." (`forge/attention.md`, section 3, *Its
  instructions*).

## 10. Kept out of goals

A milestone is tracked, so each part that took every tracked task for a
goal takes a milestone's executor out:

- **Ways:** a way is a template that is tracked and not ended by its
  requester (`../client/pages/work.md`, 1, *Ways*), so template 4, ended
  by its closer, stays one and template 8 is never one, in the
  start sheet, the forge's default way, the engine's start of a goal or
  a row's way.
- **Forge issues:** a milestone has no issue; its goals keep theirs
  (section 8).
- **The status agent's live goals** (section 9), **Ask temper's goals**
  (`agent/ask-temper.md`), and **a public change's goal**, which is its
  outermost tracked ancestor that is not a milestone
  (`../client/forge/README.md`, section 9).
- **Goals and its counts, Home's region and launchpad, findings'
  routes:** these read template 2's label or label 2, so need nothing.

## 11. Its world

Focused stories, each where its owner's are:

- **jig's tasks and core worlds:** a gathering's template checked; a
  delegate's question to it reaching its person's inbox and answered
  there (jig's `domain/tasks/gatherings.md`, section 6).
- **The stock's tests:** template 8's fields and its check.
- **temper's engine domain world** (`temper/tests/engine/domain`): a
  goal beneath a gathering whose `decided` writes its row for the maker;
  its landing setting the milestone's last landing; the milestone's end
  recorded with no issue; `Milestone` answered live and ended; the
  status words naming milestones and a goal by its milestone; the reds
  its goals owe counted and said, a complete milestone staying complete
  (`red-findings.md`, section 11).
- **The client's tools' harness and the joined client world**
  (`temper/tests/client`): `temper ctl`'s milestone commands and `goal
  start`'s flags (`../client/ctl/milestones.md`, section 5).
- **The client domain's focused tests:** `milestones::facts` and
  `milestones::of`; a way never template 8.

## 12. Open questions

- **A goal's question when a maintainer added it:** it asks the maker,
  not the maintainer who started it. Default: the maker, as the draft
  says; the history names who added it.
- **The forge's rows past 256 a project.** Default: an ended
  milestone's dropped first, read again from jig's ended record without
  its landing.
