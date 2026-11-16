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
  tracked: Some(true), order: Priority }`, as `temper ctl milestone
  list` reads (`../../domain/milestones.md`, section 7): its live group
  watched with the client's rows window, and its ended group's first
  page read by when each ended, read again when the live watch changes.
  With no template 8 in `me`, it reads nothing and lists none.
- **Its drawn milestones:** each live milestone listed, and each ended
  one whose `ended_at` is within the seven days before the wall, at most
  16, `milestones::DRAWN`, the live first by last activity, then the
  ended by newest end. For each, `pages::milestones::Milestone` holds:
  - *its document,* `Read::Task { task }`, read again when its line's
    `changed` moves, and `milestones::facts` of it: its name, words,
    maker, whole and made;
  - *its goals,* its delegates, `TaskQuery { project, parent: Some(M),
    order: Number }`: the live group watched, the ended group read by
    number, its pages followed to at most 4, read again when the live
    watch changes and once after the milestone has ended; merged in task
    number, the order they joined, a live row winning a number both
    hold, as a goal's page merges its tasks (`goal.md`, 2); `more` where
    a page was left unread;
  - *its totals,* `MilestoneTotals`, the forge family's read `Milestone`
    (`../../domain/milestones.md`, section 7), read on opening, again when
    its line's `changed` moves, and once after it ends; `not_milestone`
    or a failed read holding none.

  A milestone's page holds one `Milestone` of its own, by the same type
  (section 3).
- **A goal's milestone,** `Milestones::of(line) -> Option<u64>`, from
  `milestones::of` (`../../domain/milestones.md`, section 7): a requester
  listed is its milestone; a task requester not listed is read once by
  number, `Read::Task`, and held, at most 16 held, the oldest dropped
  first, and is its milestone where its chip is one (`is`); any other
  requester, or one not yet read, gives none, so that a goal is drawn on
  its own until its requester is known.
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
  first page.

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
- **Its tag:** "milestone", `temper-ms-tag`, on the ink ground in the
  paper's colour, 11 px, 500, radius 4, padding 0 7 px, as the mockups
  draw it; printed in the terminal as "milestone · " before its name.
- **The view's parts:** one module of the view's, `milestones`, draws the
  strip, the tag, the state's word and style and the headline's words,
  which Home's row, Work's card, the Goals filter's group and a
  milestone's page share; the client's `temper.css` takes the strip's
  and the tag's rules from the mockups' `pages.css`.

## 3. Open questions

- **Room** (`../../draft/milestones.md`, section 11): no project names a
  capacity yet, so a goal held for its resources says "waits for its
  resources" and its segment is outlined as a wait. Default: so, until a
  capacity is designed; the mockups' hatched "room" segment comes with
  it.
- **A milestone's ended goals past four pages** (1): its goals' list
  says what it holds and `more`; its count and state are its totals'.
  Default: so.
