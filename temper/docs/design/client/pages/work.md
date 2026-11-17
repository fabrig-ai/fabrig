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
  - **Its reads** are the four lists it joins, the first three each as its
    own page reads it: the goals' (*Goals*: the goals' query, its live group
    watched and its
  
    ended group read, a report per row, the project's needs);
    the changes' (*Changes*: the subject `changes`, and a `task` per row of its queues,
    its `landed` and its `landed_default`); the person's chats' (*The Chats page*: the chats' query and its
    documents' notes); and the project's landed drafts, the subject `drafts`
    (`../forge/records.md`, *A project's landed drafts*), a watch refused or
    ended counting as answered with none; and, to name who signed, the project's people, jig's
    `Read::Project { project }` on opening and again on each `me`
    delivery while none is pending, as Team reads them (`team.md`, 1,
    *Its reads*), or the rail's read of it where one serves. Work under each filter, `Goals` and
    `Changes` open the same four, and a move among them keeps them open, as a
    move of Goals' filter keeps its reads; leaving them for any other page
    closes them. With the goals' read, the project's milestones and their
    drawn milestones' reads open and close (`milestone.md`, 1).
  - **Its head,** on each of its filters: the kicker "PROJECT · Work"; the
    level-1 heading, counted from the list's sections (below): "N in
    flight", "N drafts" ("One draft") and "N landed this week", the parts at
    zero left out, the rest joined by ", ", the first capitalised, the
    numbers in words from one to nine and in digits from ten, as *Changes*'
    heading has them: "Four in flight, two drafts, 23 landed this week.";
    with all three at zero, "Nothing in flight yet."; no heading until the
    four reads have answered. Then the lede: "Everything on PROJECT, in the
    words of the agent doing it. A goal opens to its tasks and changes; a
    chat that started work sits with what it started; a draft waits for its
    goal.", its second sentence beginning "A milestone opens to its goals,
    and a goal to its tasks and changes;" while the project lists a
    milestone. Then, in `small`, the landing's line, *Changes*' one function,
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
    - "In flight · N", "newest activity first": a card for each live
      milestone drawn (`milestone.md`, 1) and for each live goal in none;
      for each change on its way to landing that no listed goal or chat holds;
      and for each chat with a change on its way, by the rules below; a
      milestone's card counts once in N and in the heading's "in flight";
    - "Drafts · N", "a goal starts from each": a card for each draft being
      written and for each landed draft listed;
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
    plain facts that `work::facts(root, project)` gathers from the four
    reads. It places each item by its own facts, never by the project's
    settings or its way, so that a move of the settings hides nothing that
    exists (`../../draft/surfaces.md`, sections 5 and 12):
    - a change belongs to the goal its task's `goal` names, else to the chat
      at the root of its `lineage_chips` on the project's chat template, as
      *Changes*' *Where a change came from* reads it, else to nothing; while
      its task is unread, to nothing;
    - a goal belongs to the milestone `Milestones::of` names, where that
      milestone is drawn (`milestone.md`, 1), else to nothing; a live goal
      that belongs to a live milestone is placed on that milestone's card
      alone, as a line, and is no card of its own
      (`../../draft/milestones.md`, 5.2);
    - a change on its way that belongs to a listed goal is drawn on that
      goal's card alone, or, where the goal is on a milestone's card, under
      the goal's line there alone; one that belongs to a listed chat on that
      chat's card alone; any other change on its way is a card of its own;
    - a chat is in In flight while a change on its way belongs to it; its card
      holds every listed change that belongs to it, on its way or landed;
    - a draft being written is a chat of the person's with a document
      (`chat/document.md`, 1); a draft's chat with a change on its way is in
      In flight too;
    - a landed draft is a row of the subject `drafts`, in Drafts while it is
      listed, ready and with no goal (`../../domain/forge/drafts.md`, 1),
      whoever wrote it; never in In flight;
    - every live item, a live milestone drawn, a live goal, a change on its
      way, a chat with one, a draft being written and a landed draft listed,
      is placed exactly once in each section it belongs to, and at least
      once, a goal's line on its milestone's card being its place;
    - In flight's order is the newest activity first: a goal's or a chat's
      `last_activity`, a milestone's the newest of its own and its goals',
      as Home orders its rows (`home.md`, 1, *Goals*), a change's
      `ready_since`; Drafts' the newest first, a
      draft being written by its chat's `last_activity`, a landed draft by its
      `at`; a tie the lower task number first, drafts being written before
      landed ones, and landed ones by forge, owner, name, then path.
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
    the card: "✓", its title, " · signed by NAMES" while it kept
    sign-offs (*Who signed*), and "landed CLOCK"; or "·", its title and
    "checking" or "queued 2nd", in *Changes*' words. Then, while it has a
    report, its byline: "Written by the goal at CLOCK", its report's `at`.
  - **A milestone's card** (`../../draft/milestones.md`, 5.2), as the mockup
    `../web/home-for-a-person/work.html` draws it, `temper-work-card` and
    `temper-ms-card`:
    - *its from line:* the tag "milestone", `temper-work-milestone`, on the
      ink ground in the paper's colour; its name, its chip's title, a link to
      its `Task`; in the way's place, "8 goals · whole" once its document
      says whole, else "6 goals so far", N its count's total (`milestone.md`,
      2), "1 goal"; and "212 spent of 400", its totals' `spent` and its
      line's `budget`, left out with no budget or before its totals are
      read; before its document is read, no "whole" or "so far";
    - *its headline,* `temper-work-quote`, and *its strip,* as on Home
      (`milestone.md`, 2);
    - *a line per goal started and live,* not waiting, in the order they
      joined, `temper-work-row` and `temper-ms-live`: the mark "▸"; its
      title, a link to its goal's `Task`; at its right, in the byline's
      style, its state's word and its count, "running · 5 / 7", as its row
      says them (*Goals*); then each change of that goal on its way, a
      goal's card's change row (`temper-ms-change`), indented;
    - *the goals waiting,* live and in `ChipPhase::Waiting`, in a
      `Role::Disclosure` `temper-ms-fold` named "3 wait", "1 waits",
      folded, each a line `temper-ms-waits`: the mark "·", its title a
      link to its goal, and its wait's words, "waits on C4" (*Goals*);
    - *the goals ended,* in one disclosure, folded, named by their counts
      joined by " · ", the parts at zero left out: "N done" and, with one
      done, ", the last NAME at CLOCK", NAME the id or title of the one
      ended done last (`milestone.md`, 2, *Its headline*) and CLOCK its end
      in the clock form; "N failed"; "N closed": "7 done, the last S8 at Sat
      21:40 · 1 failed". It opens to their lines, newest end first, each
      `temper-ms-landed`: "✓" for done, "×" failed, "–" closed; its title,
      a link; its state's word, "landed Sat 21:40", "failed", "closed".
      Its counts are of the goals the milestone's read holds.

    With no goal, its headline says so and nothing follows the strip. In
    the terminal it prints as a goal's card does: its from line one
    numbered link, its headline, the strip's line, then its lines, the
    disclosures' rows under their names.
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
  - **A landed draft's card** (`../../domain/forge/drafts.md`), as
    `work.html`'s witness catch-up: its from line, the tag "draft" in the
    person's colour, then, joined by " · ", FILE, its path's last segment;
    "landed in DIR", DIR its path less that segment, ending `/`, or at the
    repository's root its repository's name and `/`; and "ready since
    CLOCK", its `at` in the clock form: "FILE · landed in docs/ · ready
    since Fri 14:10". Then its title, when it has one. Then,
    while the project has a tracked template (*Ways*), its choices, the
    mockup's `.choices`:
    - while kind 7's `way` (`settings.md`, 2) is one of them, "Start a goal:
      WAY", `.choice.rec`, WAY that way's name (*Ways*), with the `small`
      "PROJECT’s way", which opens the start sheet for the draft and starts
      the goal at once, as the sheet's "Start goal" would (`goal.md`, 3, *From
      a landed draft*); then "Another way", `.choice`, which opens the sheet
      alone;
    - otherwise "Start a goal…", `.choice.rec`, which opens the sheet alone.

    It names no repository, commit or chat: `temper ctl draft list` does
    (`../ctl/areas.md`, area 7).
  - **A landed row:** its landing's time in the mono face, in the clock
    form; then a goal's row its goal's title, the title of the goal's chip
    in its change's `lineage_chips`, else its change's title (a goal's
    landing takes the goal's title as its words, `../../domain/forge/branches.md`,
    2), a link to the goal's `Task`; a change's row its change's title, a
    link to its change's `Task`; then " · signed by NAMES", in the
    byline's style, while the `landed_default` row the row was made from
    kept sign-offs, a goal's row its newest landing's (*Who signed*);
    then, on a goal's row whose change's task document, as *Changes* reads
    it, has a milestone in its lineage (`milestones::lineage`,
    `milestone.md`, 1), " · in Store shape" in the byline's style, its
    milestone's name, and " · in Store shape, now complete" on the row whose
    change is that milestone's totals' `last_landing` where the milestone is
    complete (`milestone.md`, 2) and its `ended` is in the same seven days
    (`../../draft/milestones.md`, 5.2); a milestone is never a row of its own; and
    at its right its landing's commit,
    its first seven hex digits as a short commit is written
    (`../web/public.md`, 3, *A short commit*), after "a chat with you · " on a
    chat's change's row.
  - **Who signed** (`../../domain/forge/change/gates.md`, 2, *Sign-offs
    kept at landing*), as the whole story's `team/work.html` draws it: a
    landed row's `signed`, from the changes' `landed` or `landed_default`
    row it was made from, named as Team names people (`team.md`, 1):
    "you" for the person reading, first, then the others in `signed`'s
    order, each the member's name, or its login where the name is empty;
    then, as one more item, the parties the project's document doesn't
    list, "one other" or "N others", N in words from two to nine, then
    digits. Two items are joined by " and ", more by ", " with " and "
    before the last: "signed by you", "signed by you and Sam", "signed by
    you, Sam and Kit", "signed by Sam and one other", "signed by two
    others". While the project's people are unread, refused or failed, no
    row says who signed. One pure function of the domain's,
    `temper_client_domain::pages::work::signed`, gives the words.
  - **Under Drafts,** Work's head and filters, then the Drafts section alone.
  - **Before an answer and with nothing:** a section claims nothing until its
    reads have answered (`README.md`'s intro). With all three answered and nothing
    listed, All is its head alone, "Nothing in flight yet."; Drafts reached
    by its address with none says "None.", as Goals' filter does.
  - **In the terminal** the page prints as lines: its head; the filters as
    numbered links; each card a numbered link with its from line and its
    headline, its rows under it, but a landed draft's, which opens no page:
    its from line and its title, then its choices as numbered buttons; each
    landed row a numbered link.
  - **What no read gives,** a chat's delegates other than its changes, and
    who made a change, is section 2's (*Work's later parts*).
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
    "Under way" running, planning, paused, waits and waits for room, "Ended"
    landed, closed and failed.
  - **"More goals",** a quiet button after the rows while the ended page has
    more, reading the next ended page in its place as "more closed chats"; the
    heading's numbers keep counting the rows read before it.
  - **Its milestones** (`../../draft/milestones.md`, 5.2, *Its filters*): a
    goal in a drawn milestone (`Milestones::of`, `milestone.md`, 1) is drawn
    in its milestone's group, `temper-goals-group`: the milestone's row as
    Home draws it (`home.md`, 1, *Goals*), then the rows of its goals that the
    filter shows, in the list's order, inset 16 px with a rule at their left
    in the line's colour. The group stands where the first of those rows
    would; a milestone none of whose goals the filter shows is not drawn, and
    a goal whose milestone is not drawn is a row of its own. The intro's and
    the filters' numbers count goals, never milestones. In the terminal the
    milestone's row prints as on Home, its goals' rows under it.

  What it reads: the query `TaskQuery { project, label: the template's,
  tracked, by priority }`, the label template `shapes::GOAL`'s, its live group
  watched and its ended group read by when each ended, as `Chats`; `listed`
  puts goals in the project's index. The rail's facts that the project
  has goals or milestones take this watch and read, and the milestones'
  list's, while the page holds them, with no narrower twin beside them
  (`milestone.md`, 1, *`Domain::has_goals`*). One read per row, one at a time, as the
  inbox's cards: its report (`goal.md`, 1, *A goal's report*). The needs are `inbox`'s
  group for the project, each row's `selection` naming its goals, read
  once as the page opens and again only as their marks move: `me`'s
  `needs_total` moving, or the inbox's mark, the rail's standing watch of
  `inbox` (`needs-you.md`) delivering again, which the engine does only
  once it has marked the party's inbox changed (jig's
  `client/domain/contract/README.md`, 5.2). That watch's first delivery
  is no move; each after it is, the first once it opens again after a
  drop among them; and no delivery of the goals' is one.

  **A row** is one link to the goal's `Task`, public as `goals::row`, since
  Home's area will reuse it: its top line the chip's title (its spec's first
  line) at the left and at the right, in `temper-goals-progress`, its bar,
  "6 / 9 · " and its state's word; then its
  headline. "6 / 9" is done, then total, of the goal's direct delegates, not
  counting cancelled ones, as its task line counts them (jig's
  `client/domain/contract/documents.md`, 1), true past one page; it is left out, with
  its " · " and its bar, while there are none. An ended row is marked.
  The same row is Home's and this list's (`../../draft/home-for-a-person.md`, 3):
  - *Its bar,* the browser's alone, as the mockup
    `../web/home-for-a-person/home.html` draws it: the goal's page's
    `temper-goal-bar` (`goal.md`, 2), 84 px wide in the row, its parts 4 px
    tall with a 2 px radius, a part a task of the total: the done ones in
    ink (`temper-goal-bar-landed`), then, while the goal is live and done
    is short of total, one in grey (`temper-goal-bar-running`), the rest in
    the line's colour. So "5 / 6 · running" has five in ink and one grey.
    It counts tasks done, whatever they did, so it is approximate, as the
    headline's count is (2, *A goal row's count of landings*); the goal's
    page keeps the exact one. Its name is "5 done of 6"; the terminal draws
    none, the count's words saying it.
  - *Its title,* past 80 characters, is cut at the last space that leaves
    it at most 80 with "…", or at a character's edge where there is none:
    one function of the view's parts, which the chats' heading's cut
    (`chat::short_title`, at 72) shares.

  **A goal's state** is a pure function, `goals::state`, the first that matches:

  | State | When | Word |
  |---|---|---|
  | needs | a needs row's `selection` names the goal | "needs funds" where the goal's own chip is held for `HoldCategory::Exceeded`, else "needs you" |
  | paused | live, held by a placed hold of mode Stop over its own subtree | "paused by you" |
  | waits | live, its chip's phase `ChipPhase::Waiting` listing a dependency, or with `more` | "waits for you" where a dependency is the reader's own need (below); else "waits on NAME" for one, "waits on NAME and NAME" for two whose lines are held, "waits on N tasks" for any other, N its listed dependencies, "N+" with `more` |
  | waits for room | live, its phase `ChipPhase::Waiting` with no dependency listed and `holds` | "waits for its resources"; "waits for room" once a project names a capacity (`../../draft/milestones.md`, section 11), which none does yet |
  | planning | live, no report, no delegates | "planning" |
  | running | any other live | "running" |
  | landed | ended, done | "landed " and its end in the clock form (`../view.md`, 6) |
  | closed | ended, cancelled | "closed" |
  | failed | ended, failed | "failed" |

  A row's **headline** is its report's. With no report: "Planning from the
  draft." planning; "N of M tasks done." running or paused, N and M in
  number words to nine, digits after, its first word capitalised, "Five
  of six tasks done.", "Five of 12 tasks done.", "None of six tasks
  done." for none (`../../draft/home-for-a-person.md`, 3), N its direct
  delegates ended done, whatever they did, since `TaskLine` counts no landings
  (`../README.md`, section 5); "Landed." landed;
  "Closed by you." closed; "Failed." failed; a goal that needs the person
  takes running's words, or planning's with no delegates. A goal that waits
  has no report yet, so its headline is its wait as a sentence, its first
  word capitalised: "Waits on S6.", "Waits on S5 and S6.", "Waits on 3
  tasks.", "Waits for you.", "Waits for its resources."
  (`../../draft/milestones.md`, section 4), on every page that draws its row;
  a report, where one is read, still wins.

  **A wait's names:** NAME is the dependency's id where its title has one
  (*A goal's id*, below), else its title cut at 40 characters by the row's
  cut, from the lines the page holds: the goals' live rows and ended page,
  and, on a page that reads them, its milestones' goals
  (`milestone.md`, 1); "task N" where none holds it, and two not both held
  say "2 tasks". A dependency is the reader's own need where it is the own
  task of a row of the needs' read, the first of that row's `selection`
  goals, the rest being its ancestors (jig's `InboxSelection`): a person
  task addressed to the reader, as a goal asks its person. `goals::state`
  decides a wait from the chip alone, `State::Waits(Wait)`, `Wait`
  carrying the first two listed dependencies, their count and `more`, and
  `State::WaitsForRoom`; the view words them from what the page holds.

  **A goal's id,** `goals::id(title: &[u8]) -> Option<&[u8]>`, one pure
  function of the domain's: its title's leading run of one to three ASCII
  capital letters then one to three ASCII digits, where the title ends
  there or goes on with a character that is neither a letter nor a digit:
  "S7" in "S7 history in task logs", "K10" in "K10 · prepare reconnect
  statements once", "H1" in "H1, Home's parts"; none in "Fix 3 bugs", "s7
  history" or "SS7a". A wait (above), a milestone's strip and its headline
  name a goal by its id where it has one, else by its title
  (`milestone.md`, 2).

  **Ways** are one public function, `goals::ways`, of the view, the project's
  templates with `tracked` set and not ended by their requester, in number
  order, each with a name and a line: so a milestone's template 8, ended by
  its requester, is never a way (`../../domain/milestones.md`, section 10), and
  every part that picks a way, the start sheet, the forge's default way,
  the engine's start of a goal and a row's way, takes the same rule.
  Template `goal`: "Task by task", "The same plan; each task lands on its own
  once checked." Template `goal for a draft`: "Plan, tasks, land whole", "A plan
  from the draft, its tasks on a goal branch, landed as one change once all are
  checked." Any other: its own name, no line. The filter's and the page's
  words are drawn in the terminal as a page of its own, heading, filters as
  numbered buttons, a row a numbered link with its state and headline.

## 2. Open questions

- **Work's later parts** (1, *Work*; `../../draft/surfaces.md`, section 5):
  what a chat's delegates did other than its changes; and who made a
  change. No read gives either. Default: a chat's card holds its changes,
  and a change's card names no maker.
- **A goal's card from a landed draft** (1, *Work*): `work.html` has a
  goal's from line say "from your draft, Thursday"; the card names no
  draft. Default: so; a landed draft's row names its goal, which a later
  card may read.
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
