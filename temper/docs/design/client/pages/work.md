# Work and Goals

## 1. The page

- **Work** (`../../draft/surfaces.md`, section 5; the whole story's
  `web/whole-story/work.html`, and `pair/work.html` and `team/work.html`
  beside it for what Pair and Team hold): `Work { project, filter }`, at
  `/projects/<project>/work`, one list of the project's work, under way and
  lately done, a project page in the rail's frame, a column as Home's, at
  most 720 px wide and centred, its parts 60 px apart, 32 on a phone. Its
  filter, `WorkFilter`, All or Drafts, is in its address (`../web/README.md`,
  2.3). Goals and Changes are Work's filters at their own addresses,
  `Goals { project, filter }` and `Changes { project }` (1, *Goals*;
  `../forge/pages.md`, *Changes*), and Chats is reached from it (`chats.md`, 1).
  Choosing a filter is a move.
  - **Its reads** are the three lists it joins, each as its own page reads
    it: the goals' (*Goals*: the goals' query, its live group watched and its
    ended group read, a report per row, the project's needs); the changes'
    (*Changes*: the subject `changes`, and a `task` per row of its queues,
    its `landed` and its `landed_default`); and the
    person's chats' (*The Chats page*: the chats' query and its documents'
    notes). Work under each filter, `Goals` and `Changes` open the same three,
    and a move among them keeps them open, as a move of Goals' filter keeps its
    reads; leaving them for any other page closes them.
  - **Its head,** on each of its filters: the kicker "PROJECT · Work"; the
    level-1 heading, counted from the list's sections (below): "N in
    flight", "N drafts" ("One draft") and "N landed this week", the parts at
    zero left out, the rest joined by ", ", the first capitalised, the
    numbers in words from one to nine and in digits from ten, as *Changes*'
    heading has them: "Four in flight, two drafts, 23 landed this week.";
    with all three at zero, "Nothing in flight yet."; no heading until the
    three reads have answered. Then the lede: "Everything on PROJECT, in the
    words of the agent doing it. A goal opens to its tasks and changes; a
    chat that started work sits with what it started; a draft waits for its
    goal." Then, in `small`, the landing's line, *Changes*' one function,
    "3 queued · last landed 6 min ago", a branch's "main red" in the signal's
    colour; nothing while the changes' watch hasn't answered.
  - **Its filters:** a navigation named "Work's filters", chips as the
    mockup's `.chips`, each a link: "All"; "Drafts · N"; "Goals · N";
    "Changes · N"; "Chats · N". The current one is current, on the signal's
    ground. A filter with nothing under it isn't drawn
    (`../../draft/surfaces.md`, section 12): Drafts while a draft is listed,
    Goals while the goals' reads list a goal, live or ended, Changes while
    a change is queued or landed recently, Chats while a chat started work;
    All always, and the current one always. Kind 7's switches Goals and
    Changes (`settings.md`, 2) hide their filters, as overrides that hide a filter its rule
    would show and never show one it would hide (`../../draft/surfaces.md`,
    section 12); until kind 7 is read, or with its read refused, neither hides,
    and what they hide stays under All. N is what its list holds:
    Drafts' cards, the goals' rows read, the changes queued and landed, the
    chats with work. "Chats" goes to `Chats { filter: All }`, the person's
    chats, a page that keeps its own head (`chats.md`, 1).
  - **All** is three sections, each drawn only while it holds a card or a
    row, each with its head, the mockup's `.shead`, its title and a quiet
    line at its right:
    - "In flight · N", "newest activity first": a card for each live goal;
      for each change on its way to landing that no listed goal or chat holds;
      and for each chat with a change on its way, by the rules below;
    - "Drafts · N", "a goal starts from each": a card for each draft;
    - "Landed this week · N", with "All", to `Changes { project }`, at its
      right: the work that reached a default branch in the seven days
      before now, by the changes' `landed_default` rows
      (`../forge/records.md`, *A project's changes*), newest first: a row
      for each goal with a change landed into its repository's default
      branch, its goal's landing or any other of its changes, and a row
      for each such change outside a goal. A change landed into a goal's
      branch, or any branch other than its repository's default, is no
      row, since a person reads the section as what reached main (the
      person, 2026-11-11), where the whole story's mockup lists a row per
      change. N, and the heading's "landed this week", count the rows.
      After the rows, while it is above zero, a quiet line, in the
      byline's style, of the changes of goals landed into a branch other
      than their repository's default in the same seven days, by the
      `landed` rows: "Also one change landed into a goal branch." or
      "Also N changes landed into goal branches.", N in words from two to
      nine and in digits from ten, as the heading's numbers. With no row,
      the section, its quiet line with it, isn't drawn.
  - **What goes where** is one pure function of the domain's,
    `temper_client_domain::pages::work::place(&Facts, wall) -> Work`, over
    plain facts that `work::facts(root, project)` gathers from the three
    reads. It places each item by its own facts, never by the project's
    settings or its way, so that a move of the settings hides nothing that
    exists (`../../draft/surfaces.md`, sections 5 and 12):
    - a change belongs to the goal its task's `goal` names, else to the chat
      at the root of its `lineage_chips` on the project's chat template, as
      *Changes*' *Where a change came from* reads it, else to nothing; while
      its task is unread, to nothing;
    - a change on its way that belongs to a listed goal is drawn on that
      goal's card alone, and one that belongs to a listed chat on that chat's
      card alone; any other change on its way is a card of its own;
    - a chat is in In flight while a change on its way belongs to it; its card
      holds every listed change that belongs to it, on its way or landed;
    - a draft is a chat of the person's with a document (`chat/document.md`, 1);
      a draft's chat with a change on its way is in In flight
      too;
    - every live item, a live goal, a change on its way, a chat with one and
      a draft, is placed exactly once in each section it belongs to, and at
      least once;
    - In flight's order is the newest activity first: a goal's or a chat's
      `last_activity`, a change's `ready_since`; Drafts' the newest
      `last_activity` first; a tie the lower task number first.
    - Landed this week takes the `landed_default` rows whose `at` is in
      the seven days before `wall`, `wall` included, each once by its task:
      a row that belongs to a goal is that goal's row, and a goal has one
      row, at its newest such landing; any other is its change's row, a
      chat's change and one whose task is unread among them. Its order is
      the newest landing first, a tie the lower change's task first. Its
      quiet line counts the `landed` rows in the same window, each once by
      its task, whose `default_branch` is false and that belong to a goal,
      whether the goal is listed or not.
  - **A goal's card:** its from line: the tag "goal" in the agents' colour,
    the mockup's `.who.a`; its title, its chip's, a link to its `Task`; its
    way's name (1, *Ways*); and "31 of 50 spent", its task line's
    `budget` as the goal's page says it (`goal.md`, 2), left out with no budget. Then
    its headline, *Goals*' row's. Then, while it has delegates, its bar, done
    of total as *Goals*' "6 / 9" counts them. Then a row for each change on
    the card: "✓", its title and "landed CLOCK"; or "·", its title and
    "checking" or "queued 2nd", in *Changes*' words. Then, while it has a
    report, its byline: "Written by the goal at CLOCK", its report's `at`.
  - **A change's card:** the tag "change" in code's colour, `.who.c`; its
    title, a link to its `Task`; "checking" or "queued 2nd", in *Changes*'
    words.
  - **A chat's card:** the tag "chat" in the person's colour, `.who.p`; "with
    you", its start in the clock form (`../view.md`, 6), and its title in quotes, a link
    to its `Chat`; then a row for each change on the card, as a goal's.
  - **A draft's card:** the tag "draft" in the person's colour; its document's
    name, as the Chats page's row says what it writes, and "a chat with you",
    its last activity in the clock form; then its chat's title; then "Open
    the draft", to its `Chat`.
  - **A landed row:** its landing's time in the mono face, in the clock
    form; then a goal's row its goal's title, the title of the goal's chip
    in its change's `lineage_chips`, else its change's title (a goal's
    landing takes the goal's title as its words, `../../domain/forge/branches.md`,
    2), a link to the goal's `Task`; a change's row its change's title, a
    link to its change's `Task`; and at its right its landing's commit,
    its first seven hex digits as a short commit is written
    (`../web/public.md`, 3, *A short commit*), after "a chat with you · " on a
    chat's change's row.
  - **Under Drafts,** Work's head and filters, then the Drafts section alone.
  - **Before an answer and with nothing:** a section claims nothing until its
    reads have answered (`README.md`'s intro). With all three answered and nothing
    listed, All is its head alone, "Nothing in flight yet."; Drafts reached
    by its address with none says "None.", as Goals' filter does.
  - **In the terminal** the page prints as lines: its head; the filters as
    numbered links; each card a numbered link with its from line and its
    headline, its rows under it; each landed row a numbered link.
  - **What no read gives,** landed drafts ready to start a goal
    (`../../draft/surfaces.md`, section 5), a chat's delegates other than its
    changes, and who made a change, is `../README.md`, section 5's (*Work's later parts*).
- **Goals** (the greenfield mock's `goals.html`), at `/projects/<project>/goals`,
  in the project's frame, is Work under its Goals filter (1, *Work*):
  Work's head and filters, "Goals" current, take its kicker's place, and
  under them are the project's goals, the filter in its address:
  `GoalsFilter`, All, Needs, Live or Ended, whose query is `?state=needs`,
  `?state=live` or `?state=ended` (`../web/README.md`, 2.3). Choosing one is a move.
  A goal is a tracked agent's task (`goal.md`, 2) at its own `Task` address, so that
  Goals is the list and draws no goal page. Its parts, in order:
  - **Its intro:** under Work's head and filters, the heading, at
    level 2: "5 goals, 3 under way.", the first number
    every row and the second those live and those needing the person; "1 goal,
    1 under way." in the singular; "No goals yet." with none, once its reads
    have answered, and no heading before; "More than N
    goals, M under way." while rows remain unread. Then "A goal turns a draft
    into landed work, its own way: W.", W the project's ways in order, lower
    case, joined by ", " (*Ways,* below).
  - **Its filters:** "All", "Needs you", "Under way", "Ended", the chosen one
    pressed; at the section head's right, the `.link` "Start one from a draft",
    opening `goal.md`, 3's sheet with no draft, while the project has a tracked
    template. Under them, the rows, or, once its reads have answered, "None.";
    "Needs you" is state needs,
    "Under way" running, planning and paused, "Ended" landed, closed and
    failed.
  - **"More goals",** a quiet button after the rows while the ended page has
    more, reading the next ended page in its place as "more closed chats"; the
    heading's numbers keep counting the rows read before it.

  What it reads: the query `TaskQuery { project, label: the template's,
  tracked, by priority }`, the label template `shapes::GOAL`'s, its live group
  watched and its ended group read by when each ended, as `Chats`; `listed`
  puts goals in the project's index. One read per row, one at a time, as the
  inbox's cards: its report (`goal.md`, 1, *A goal's report*). The needs are `inbox`'s
  group for the project, read again as the watch or `me`'s `needs_total`
  moves, each row's `selection` naming its goals.

  **A row** is one link to the goal's `Task`, public as `goals::row`, since
  Home's area will reuse it: its top line the chip's title (its spec's first
  line) at the left and at the right "6 / 9 · " and its state's word; then its
  headline. "6 / 9" is done, then total, of the goal's direct delegates, not
  counting cancelled ones, as its task line counts them (jig's
  `client/domain/contract/documents.md`, 1), true past one page; it is left out, with
  its " · ", while there are none. An ended row is marked.

  **A goal's state** is a pure function, `goals::state`, the first that matches:

  | State | When | Word |
  |---|---|---|
  | needs | a needs row's `selection` names the goal | "needs funds" where the goal's own chip is held for `HoldCategory::Exceeded`, else "needs you" |
  | paused | live, held by a placed hold of mode Stop over its own subtree | "paused by you" |
  | planning | live, no report, no delegates | "planning" |
  | running | any other live | "running" |
  | landed | ended, done | "landed " and its end in the clock form (`../view.md`, 6) |
  | closed | ended, cancelled | "closed" |
  | failed | ended, failed | "failed" |

  A row's **headline** is its report's. With no report: "Planning from the
  draft." planning; "N of M tasks done." running or paused, N its direct
  delegates ended done, whatever they did, since `TaskLine` counts no landings
  (`../README.md`, section 5); "Landed." landed;
  "Closed by you." closed; "Failed." failed; a goal that needs the person
  takes running's words, or planning's with no delegates.

  **Ways** are one public function, `goals::ways`, of the view, the project's
  templates with `tracked` set, in number order, each with a name and a line.
  Template `goal`: "Task by task", "The same plan; each task lands on its own
  once checked." Template `goal for a draft`: "Plan, tasks, land whole", "A plan
  from the draft, its tasks on a goal branch, landed as one change once all are
  checked." Any other: its own name, no line. The filter's and the page's
  words are drawn in the terminal as a page of its own, heading, filters as
  numbered buttons, a row a numbered link with its state and headline.

## 2. Open questions

- **Work's later parts** (1, *Work*; `../../draft/surfaces.md`, section 5):
  a landed draft ready to start a goal, named with its repository, path
  and commit (`../../draft/whole-story.execution.md`, P8); what a chat's
  delegates did other than its changes; and who made a change. No read
  gives any of them. Default: a draft is a chat's document, a chat's card
  holds its changes, and a change's card names no maker.
- **A week's landings past the log** (1, *Work*): Landed this week reads the
  landing log (`../../domain/after-landing.md`, section 10), a ring of
  `landings_kept` landings per repository, through `landed_default`'s
  `forge.changes_landed` rows, so a busy week's landings into goal branches
  can push its older default-branch landings out of the ring, and more than
  `forge.changes_landed` of them show as that many. Default: so, nothing
  stored added (the person, 2026-11-11); or the log keeps default-branch
  landings apart, a stored field.
- **A goal row's count of landings** (1, *Goals*; `goal.md`, 2): `TaskLine` counts a goal's
  delegates ended done whatever they did, so a row and the goal's headline say "N of
  M tasks done.", and only the goal's own rows, whose chips carry their results, say
  "landed". Default: so; or jig's `Endings` counts change results, a stored and a wire
  field.
