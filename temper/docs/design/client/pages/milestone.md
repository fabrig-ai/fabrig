# A milestone on the pages

Provisional, 2026-11-16, from `../../draft/milestones.md`, sections 4
and 5, milestone M3 of `../../draft/home-for-a-person.execution.md`. A
milestone is a set of goals (`../../domain/milestones.md`): this
document gives what the client reads of one and the words every page
draws it with (sections 1 and 2), and its own page (section 3). Home's
row of it is `home.md`, 1, *Goals*; Work's card and the Goals filter's
group are `work.md`, 1; Needs you's from line and filter are
`needs-you.md`, 1; a goal's "in NAME" is `goal.md`, 2. The look is the
mockups' `../web/home-for-a-person/`.

## 1. Its reads

- **Is a milestone:** `milestones::is(chip: &c::Chip) -> bool`, beside
  `milestones::facts` and `milestones::of` in the client domain's
  `milestones` module: the chip's executor is jig's gathering,
  `Procedure { connector: RECURRING, code: GATHERING }`
  (`../../domain/milestones.md`, section 2). `temper ctl`'s test of a
  milestone takes it too. Its template is the stock's 8, named once in
  that module, `milestones::TEMPLATE`, which `temper ctl`'s milestone
  query takes as well.
- **The project's milestones,** `pages::milestones::Milestones`, held by
  `Pages` beside the goals' shared read (`work.md`, 1, *Goals*) and with
  its lifetime: opened with it on Home, Work, Goals and Changes, kept
  across a move among them, closed on leaving them. Its query is
  `TaskQuery { project, label: TEMPLATE's label as me serves it,
  tracked: Some(true), order: Priority, parts: [MILESTONE_PART] }`, as
  `temper ctl milestone list` reads (`../../domain/milestones.md`, section
  7), less the part: its live group watched with the client's rows
  window, and its ended group's first page read by when each ended, read
  again only as its mark grows (`../forge/parts.md`, 6). With no
  template 8 in `me`, it reads nothing and lists none.
- **Its drawn milestones:** each live milestone listed, and each ended
  one whose `ended_at` is within the seven days before the wall, at most
  16, `milestones::DRAWN`, the live first by last activity, then the
  ended by newest end. For each, `pages::milestones::Milestone` holds,
  from its row alone, with no read of its own (`../forge/parts.md`, 3
  and 5):
  - *its facts,* what its card draws of them, from its line and its
    part: its name, its chip's title; its maker, its requester; whole,
    its part's `whole`. Its words and when it was made are its page's,
    which reads its document (section 3);
  - *its goals,* its part's `goals`, at most 16, each a goal's line and
    its report's headline, in task number, the order they joined; `more`,
    its part's;
  - *its totals,* its part's `totals`, the reds its goals owe among
    them (`../../domain/milestones.md`, section 7); none until a part of
    it has decoded.

  Each is its part's as last decoded while the list is held: a delivery
  whose part of it is not made keeps its facts, goals, headlines and
  totals as they were (`../forge/parts.md`, 5), so its count, its state's
  word and its strip don't fall back to "open" while nothing changed.

  A milestone's page holds its own, which lists every goal and reads
  them itself (section 3).
- **A goal's milestone,** `Milestones::of(line) -> Option<u64>`: its goal
  part's `milestone` (`../forge/parts.md`, 3), as last decoded while the
  goals' list is held (`../forge/parts.md`, 5); where no part of the goal
  has decoded, the drawn milestone whose part lists it among its goals;
  none otherwise, so that a goal is drawn on its own only while nothing
  read names its milestone. A decoded goal part saying it has none is
  none, whatever a milestone's part lists. No requester is read. Every
  page that groups goals asks `of`, Home's rows, Work's cards and Goals'
  groups alike, so a goal under a drawn milestone is drawn there alone,
  never also as a row of its own.
- **A task's milestone from its lineage,** `milestones::lineage(task:
  &c::Task) -> Option<Lineage>`, pure: of a task document's
  `lineage_chips`, the chip that is a milestone (`is`) is its milestone,
  and its goal the chip just beneath it, or the task's own chip where the
  task is the milestone's delegate. `Lineage { milestone, goal }` are
  chips, so their titles name them. A page that reads a task's document
  anyway (Decided for you's rows, a goal's page, a need's card) names
  its milestone with no read of the tree.
- **Under a need:** a milestone, or a goal in it, has a need of the
  reader's under it where the goals' needs read names it
  (`Goals::needs_person`), whose `selection` lists each need's task and
  its ancestors (jig's `InboxSelection`).
- **`Domain::has_goals`** reads both: the project has a goal, or lists a
  milestone, live or ended. `HasGoals` watches the milestones' query as
  it watches the goals', with a window of 1, and reads its ended group's
  first page, but where the page holds that query's list (Work's and
  Goals', and the milestones' on the pages that list them): then it
  takes the list's watch and its ended read, and opens neither of its
  own. **No narrower twin:** a watch whose subject differs from one the
  client holds only by a smaller window, or a read the page already
  makes, is served by the one held. The pages ask their lists before
  the rail's facts in each step, so a page that opens with its lists
  opens no twin to close; a fact keeps the value it last read until its
  own watch's first delivery, when the page that held its list is left.

## 2. Its words

Pure functions of `pages::milestones`, over a milestone's line, its
goals as merged (section 1), its totals where read and whether a need
of the reader's is under it; the view words them.

- **Its state,** `milestones::state(..) -> State`, the first that holds
  (`../../draft/milestones.md`, section 4):

  | State | When | Word |
  |---|---|---|
  | closed | its line ended cancelled | "closed" |
  | complete | ended done, its totals read with no goal failed | "complete" |
  | ended | ended done, its totals read with a goal failed | "ended" |
  | done | ended done, its totals not yet read | "done" |
  | needs | live, a need of the reader's under it | "needs you" |
  | paused | live, held by a placed hold of mode Stop over its own subtree, as a goal's paused (`work.md`, 1, *Goals*) | "paused by you" |
  | running | live, a goal in it live and started: not in `ChipPhase::Waiting` | "running" |
  | waits | live, goals in it live and none started | "waits" |
  | open | live, no goal in it live | "open" |

  So a page never says "complete" before its totals are read
  (`../../domain/milestones.md`, section 7).
- **Its count,** `milestones::count(..) -> Count { done, total }`,
  "7 / 8": from its totals where read, `done`, and `done + failed +
  live`; else from its goals held, those ended done, and those ended
  done, failed or live; closed goals out of both.
- **Its strip,** `milestones::strip(goals, needing: &[u64]) ->
  List<Segment>`, `Segment { task, mark }`: one per goal it counts, in
  the order they joined, its `Mark` `Done` (ended done), `Needs` (live,
  its number among `needing`, the goals with a need under them),
  `Live` (live and started), `Waits` (live and waiting) or `Failed`
  (ended failed). It is named "7 done of 8", its count. In the browser
  it is `temper-ms-strip`, as the mockups' `pages.css` draws it: a row of
  segments 6 px tall, radius 3, 3 px apart, each its goal's share of the
  width; done in ink, live in `--gf-goal-running`, needs in the signal's
  colour, waits outlined in `--gf-line-2`, failed in the line's colour
  struck through in `--gf-ink-2`; under each, its goal's id (`work.md`,
  1, *A goal's id*) where it has one, 10.5 px mono, faint, a live one's
  in ink and a needing one's in the signal's colour. In the terminal it
  is one line, a character a segment, "■" done, "▒" live or needs, "□"
  waits, "×" failed, then its name: "■■■■■■▒■ 7 done of 8".
- **Its headline,** `milestones::headline(..) -> Headline`, worded by
  the view, D and N its count:
  - the count's sentence: "D of N goals done.", "None of N goals
    done." for none, "goal" for an N of one;
  - complete: "All N goals done."; with none, "Complete, with no
    goals.";
  - ended: "D of N goals done; NAME failed." for one failed goal held,
    "; 2 failed." for more, or for one not held;
  - closed: "Closed by you; " and the count's sentence, its first word
    lower case unless "None": "Closed by you; 3 of 8 goals done.";
  - done: the count's sentence;
  - live with a goal live and started: the count's sentence, then its
    newest such goal by last activity, its NAME, ": " and its headline,
    its report's where read or its row's (`work.md`, 1, *Goals*), its
    closing full stop replaced by ", and one more running." or ", and N
    more running." where more are: "7 of 8 goals done. S7: 5 of its 7
    changes landed; the other 2 are building.";
  - live with none started: the count's sentence;
  - open with no goal live: "D goals done so far; more may join.", "1
    goal done so far; …"; with no goal at all, "No goals yet."

  NAME is the goal's id where its title has one, else its title cut at
  40 characters by the row's cut (`work.md`, 1, *Goals*).

  **Reds owed:** where its totals are read and their `reds`, R, are
  above zero (`../../domain/red-findings.md`, section 10), the
  headline's first sentence ends "; R reds after landing owed." in
  place of its full stop, "; 1 red after landing owed." for one, in
  every case above: "All 5 goals done; 2 reds after landing owed.",
  "7 of 8 goals done; 1 red after landing owed. S7: 5 of its 7 changes
  landed; the other 2 are building.", "D goals done so far; more may
  join; 1 red after landing owed." `Headline` carries them, `reds:
  u32`, beside its case, and its state's word is unchanged: a complete
  milestone owing reds is "complete". Every view that draws this
  headline draws them, its page, Home's row and Work's card among them.
- **Its tag:** "milestone", `temper-ms-tag`, on the ink ground in the
  paper's colour, 11 px, 500, radius 4, padding 0 7 px, as the mockups
  draw it; printed in the terminal as "milestone · " before its name.
- **The view's parts:** one module of the view's, `milestones`, draws the
  strip, the tag, the state's word and style and the headline's words,
  which Home's row, Work's card, the Goals filter's group and a
  milestone's page share; the client's `temper.css` takes the strip's
  and the tag's rules from the mockups' `pages.css`.

## 3. Its page

`Task { task }` of a milestone (`../../draft/milestones.md`, 5.4; the
mockup `../web/home-for-a-person/milestone.html`): a page shape of its
own, the task page's `Shape::Milestone`, which the task's document
dispatches by its chip (`milestones::is`) before any other shape, where a
procedure's task otherwise gets its header and plan (`task.md`, 3). It
borrows the goal page's header and two columns, their classes and look
(`goal.md`, 2).

- **What it reads,** beside the task's document and tree as any task
  page reads them: its own milestone, by the reads a drawn milestone made
  before its part (`../forge/parts.md`): its document's
  `milestones::facts`; its goals, `TaskQuery { project, parent: Some(M),
  order: Number, parts: [GOAL_PART] }`, the live group watched and the
  ended group read by number, its pages followed to at most 4, read again
  only as its mark grows (`../forge/parts.md`, 6), merged in task number,
  a live row winning a number both hold, as a goal's page merges its
  tasks (`goal.md`, 2), each goal's headline its goal part's, with no
  read per goal; its totals, the forge family's read `Milestone`, read
  on opening, again when its line's `changed` moves, and once after it
  ends; and its
  needs, the inbox's `Filter { project, goal: Some(task) }` as a goal's
  page opens it, which, the filter taking a root, holds every need under
  its goals, its read's `selection` saying which goal each is under.
- **The header,** `temper-goal-head`:
  - its meta, mono, items separated by "·": its name, bold; its state's
    word in its token (section 2); "7 of 8 goals done", its count; "212
    spent of 400", its totals' `spent` and its line's `budget`, left out
    with no budget or before its totals are read; "since " and its `made`
    in the clock form (`../view.md`, 6);
  - its headline (section 2) as the level-1 heading;
  - while live, its acts, `task.md`, 4's: "Pause" or "Resume", then
    "Close", with Close's confirmation. No Amend, no byline, no composer
    and no Chat link: a milestone runs no agent, so words go to one of its
    goals.
- **Needs you bands,** as a goal's page draws its own (`goal.md`, 2,
  *Needs you bands*), one per need under it, the first question whole and
  the rest by their first line; a band's QUESTION follows the goal it is
  under, its title and ": ", where its `selection` names one.
- **Goals,** in the report's place, `temper-goal-report`, headed "Goals":
  - a row per goal not ended, in the order they joined,
    `temper-ms-goal`: its line, `temper-goal-line`, its title a link to
    its `Task` and its state's word, a waiting goal's being what it waits
    on, "waits on S6" (`work.md`, 1, *Goals*), in its token; under it its
    headline, `temper-goal-facts`, its report's where read or its row's;
    then, where its line names a first wait (jig's
    `domain/engine/waits.md`, 4), `temper-goal-wait`: that wait in a
    wait's words (`waits.md`, 2), with its own bound and no hearer, led
    by "TN: " where it waits at a task under the goal, "T833: question 3
    at you · waiting 2 h · passes up in 20 min"
    (`../../draft/bounded-waits.md`, 9);
  - its ended goals in folds, `Role::Disclosure` and
    `temper-goal-landed-fold`, folded: "N done", "N failed" and "N
    closed", each while above zero, each holding its goals' lines, newest
    end first: the title, a link, and its state's word, "landed Sat
    21:40", read from its ended goals even after the milestone has ended;
  - "No goals yet." while open with none; "No goals." once whole with
    none; "More of its goals are not shown." as `temper-goal-facts` while
    its goals' read has `more`.
- **The side,** `temper-goal-side`, its sections headed as the goal
  page's are:
  - its strip, headed by its name, "7 done of 8" (section 2);
  - "What it is for": its words, where it has any;
  - "Landing": "Each goal lands on its own, once done.";
  - "Its goals": "Open: more goals may join." or "Whole: all its goals are
    in.", then "Made by NAME.", NAME "you" where the maker is the reader,
    else as Work names who signed (`work.md`, 1, *Who signed*), from the
    project's people where a read of them serves, else "person N", "task
    N" or "the deployment".
- **Before its reads answer,** it claims nothing: no count, state or
  headline before its line, goals and totals are held; the header shows
  its name once the document is read.
- **The terminal:** the view is shared and keeps its meaning: the header,
  the needs, "Goals" and its rows, each fold's rows under its name, then
  the side's sections, the strip as its line, "■■■■■■▒■ 7 done of 8".
- **On a phone:** the two columns wrap by the goal page's flex basis.

## 4. Open questions

- **Room** (`../../draft/milestones.md`, section 11): no project names a
  capacity yet, so a goal held for its resources says "waits for its
  resources" and its segment is outlined as a wait. Default: so, until a
  capacity is designed; the mockups' hatched "room" segment comes with
  it.
- **A milestone's ended goals past four pages** (1): its goals' list
  says what it holds and `more`; its count and state are its totals'.
  Default: so.
