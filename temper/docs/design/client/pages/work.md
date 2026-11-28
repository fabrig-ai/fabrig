# Work and Goals

## 1. The page

- **Work** (`../../draft/surfaces.md`, section 5; the whole story's
  `web/whole-story/work.html`, and `pair/work.html` and `team/work.html`
  beside it for what Pair and Team hold): `Work { project, filter }`, at
  `/projects/<project>/work`, one list of the project's work, under way and
  lately done, a project page in the rail's frame, a column as Home's, at
  most 720 px wide and centred, its parts 60 px apart, 32 on a phone. Its
  filter, `WorkFilter`, All or Drafts, is in its address (`../web/addresses.md`,
  1). Goals and Changes are Work's filters at their own addresses,
  `Goals { project, filter }` and `Changes { project }` (1, *Goals*;
  `../forge/pages.md`, *Changes*), and Chats is reached from it (`chats.md`, 1).
  Choosing a filter is a move.
  - **Its reads** are the four lists it joins, the first three each as its
    own page reads it: the goals' (*Goals*: the goals' query, its live group
    watched and its ended group read, each row with its goal part, and the
    project's needs);
    the changes' (*Changes*: the subject `changes`, each row naming where it
    came from and who signed it, `../forge/records.md`, *A project's
    changes*); the person's chats' (*The Chats page*: the chats' query, each
    row with its chat part); and the project's landed drafts, the subject `drafts`
    (`../forge/records.md`, *A project's landed drafts*), a watch refused or
    ended counting as answered with none. No read is made per row
    (`../forge/parts.md`, 5). Work under each filter, `Goals` and
    `Changes` open the same four, and a move among them keeps them open, as a
    move of Goals' filter keeps its reads; leaving them for any other page
    closes them. With the goals' read, the project's milestones open and
    close, each row with its milestone part (`milestone.md`, 1).
  - **Its budget** (`../../draft/page-round-trips.md`, section 2): a cold
    open of Work, a reload among them, under each of its filters (All and
    Drafts, Goals' four, Changes), is drawn whole in two waves and two
    requests after its page and bundle, whatever the project's size: the
    stream, whose hello brings `me` (`../root.md`, 1.4), then one batch
    (jig's `protocol/client/default-http.md`, 6) carrying every read and
    watch the frame and the page open.
    - *A wave* is every exchange the client has sent that waits for an
      answer, all answered at once; a page drawn whole that sends nothing
      more has taken its last. *Requests* are the stream's opening and
      each exchange, a batch one.
    - *Drawn whole* is every row of the page's windows, with everything
      its card draws, and the frame's rail and bar.
    - *Outside it:* a person's acts ("More goals", opening a goal,
      choosing a filter); values the stream brings later; a lost
      connection's retries; answers busy; and, on a reload, the keyed
      requests the small store kept, which leave on their own routes
      beside the batch, with their keys, and hold nothing of the page.
    - *One constant beside the page:* `temper_client_domain::pages::work::BUDGET`,
      a `temper_client_domain::pages::Budget { waves: u32, requests: u32 }`,
      `Budget { waves: 2, requests: 2 }`, the record a later page's budget
      takes too (`../../draft/page-round-trips.md`, section 10).
    - *What holds it.* Nothing leaves before the hello but the stream
      (`../root.md`, 1.4); a reload's kept keyed requests leave once
      the hello's `me` confirms their party (jig's
      `client/domain/client/small-store.md`, 1), beside the batch. Nothing waits for
      another answer: no read per row (`../forge/parts.md`, 5); no ended
      group, needs or setting read again on opening (`../forge/parts.md`,
      6; *Goals*, below; `settings.md`, 1); and nothing the page doesn't
      draw: no `Subject::Chips` watch of the chats' goals, which the
      Chats page alone draws (`chats.md`, 1), and no `deployment` watch,
      which only a page judging a model usable reads
      (`../forge/family.md`, 7). The session's own reads and watches
      (`../root.md`, 1.4) and the project's settings' reads stay
      open whatever the page and ride in the same batch: the frame draws
      from most of them, and closing the others on Work only to open them
      again on the next page would cost requests that keeping them
      doesn't.
    - *The page's members in the batch* come in its sections' order, the
      same under every filter since the filters share their reads: the
      goals' watch, their ended read and the needs' read; the milestones'
      watch and ended read; the chats' watch and ended read; the
      changes' watch; then the drafts' watch. The frame's and the
      session's come before or after them.
    - *Its test,* focused, in `temper-client-view`'s suite: the linked
      client root (`temper_client_domain::Domain::new`) and the view, with
      an engine double that answers every exchange of a wave at once and
      sends each opened watch's first value over the stream, counts a cold
      open's waves and requests until it is drawn whole and sends nothing
      more: on a small project and on one at the client's bounds (32
      goals live and ended, 16 chats, 16 milestones with their goals, 64
      changes queued and landed, 64 landed drafts, or each list's bound
      where the client's limits give another), both within `BUDGET` and
      equal; under each filter, Goals' Needs among them; read by the
      deployment's owner with a playground running, the frame's most; and
      a reload with kept keyed requests, the page within `BUDGET` and the
      kept requests on their own routes beside it.
  - **Its head,** on each of its filters: the kicker "PROJECT · Work"; the
    level-1 heading, counted from the list's sections (below): "N in
    flight", "N drafts" ("One draft") and "N landed this week", the parts at
    zero left out, the rest joined by ", ", the first capitalised, the
    numbers in words from one to nine and in digits from ten, as *Changes*'
    heading has them: "Four in flight, two drafts, 23 landed this week.";
    with all three at zero, "Nothing in flight yet.". The heading waits
    only for its counts, every list its sections count having answered;
    until then its line is held, empty and named nothing, at the
    heading's height in the browser, so that nothing below it moves when
    it comes (`../../draft/page-round-trips.md`, section 6); the terminal
    prints no line for it. The kicker, the lede and the filters draw at
    once. Then the lede: "Everything on PROJECT, in the
    words of the agent doing it. A goal opens to its tasks and changes; a
    chat that started work sits with what it started; a draft waits for its
    goal.", its second sentence beginning "A milestone opens to its goals,
    and a goal to its tasks and changes;" while the project lists a
    milestone. Then, in `small`, the landing's line, *Changes*' one function,
    "3 queued · last landed 6 min ago", a branch's "main red" in the signal's
    colour; nothing while the changes' watch hasn't answered.
  - **Its filters:** a navigation named "Work's filters", chips as the
    mockup's `.chips`, each a link: "All"; "Drafts · N"; "Goals · N";
    "Changes · N"; "Waits"; "Chats · N". The current one is current, on the signal's
    ground. A filter with nothing under it isn't drawn
    (`../../draft/surfaces.md`, section 12): Drafts while a draft is listed,
    Goals while the goals' reads list a goal, live or ended, Changes while
    a change is queued or landed recently, Chats while a chat started work;
    All always, and the current one always. "Waits" is drawn always and
    counts nothing: it goes to `Waits { project, filter }`, a page that
    keeps its own head and read (`waits.md`, 1), and Work reads no waits,
    so its budget stays as it is. Kind 7's switches Goals and
    Changes (`settings.md`, 2) hide their filters, as overrides that hide a filter its rule
    would show and never show one it would hide (`../../draft/surfaces.md`,
    section 12); until kind 7 is read, or with its read refused, neither hides,
    and what they hide stays under All. N is what its list holds:
    Drafts' cards, the goals' rows read, the changes queued and landed, the
    chats with work. "Chats" goes to `Chats { filter: All }`, the person's
    chats, a page that keeps its own head (`chats.md`, 1).
  - **All** is three sections, each drawn only while it holds a card or a
    row, each with its head, the mockup's `.shead`, its title and a quiet
    line at its right. **They draw top down, each as its lists come**
    (`../../draft/page-round-trips.md`, section 6): a section draws once
    its own lists and those of every section above it have answered, so
    that nothing the person is reading moves when a later list arrives,
    and no section waits for a list below it or for the frame's reads.
    In flight's lists are the goals' (their live watch and ended read, and
    the needs' read, which words a goal's wait and state), the
    milestones' and the chats' (each a live watch and an ended read) and
    the changes'; Drafts' are the chats' and the drafts'; Landed this
    week's the changes' and the milestones'. A list answered refused,
    ended or failed counts as answered with none, as the drafts' watch
    does (*Its reads*). So In flight draws without waiting for the
    drafts' watch, and Drafts and Landed this week once it has answered
    too. **A row is whole when first drawn:** what its card draws comes
    with its list, a goal's headline, a milestone, a change's provenance
    and its signers' names among them (`../forge/parts.md`, 5;
    `../forge/records.md`, *A project's changes*), so that no part fills
    in later and moves a row; a part not made leaves its row as
    `../forge/parts.md`, 5 says, until its next delivery. **One stable
    list:** a live goal of a drawn milestone is a line on that milestone's
    card alone, never also a card of its own, whichever of its list's
    deliveries made its part (`Milestones::of`, `milestone.md`, 1, *A
    goal's milestone*), and a goal's headline and its "Written by the goal
    at …" byline, a milestone's count, state, strip and goal lines are what
    their parts last said. So In flight's N and the heading's numbers stand
    through a delivery whose parts are not made, and reads answering again
    in any order, with nothing changed in the store, draw the same text
    (`README.md`'s opening).
    - "In flight · N", "newest activity first": a card for each live
      milestone drawn (`milestone.md`, 1) and for each live goal in none;
      for each change on its way to landing that no listed goal or chat holds;
      and for each chat with a change on its way, by the rules below; a
      milestone's card counts once in N and in the heading's "in flight";
    - "Drafts · N", "a goal starts from each": a card for each draft being
      written and for each landed draft listed;
    - "Landed this week · N", with "All", to `Changes { project }`, at its
      right: the work that reached a landing branch in the seven days
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
    plain facts that `work::facts(root, project)` gathers from the lists
    that have answered, saying which have; `place` gives a section only
    once its lists and those above it have (*All*), and the filters'
    counts and the heading's once all have. It places each item by its own facts, never by the project's
    settings or its way, so that a move of the settings hides nothing that
    exists (`../../draft/surfaces.md`, sections 5 and 12):
    - a change belongs to the goal its row's `goal` names, else to the chat
      its row's `chat` names, as *Changes*' *Where a change came from* reads
      them, else to nothing; while its row names neither, to nothing;
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
    form; then a goal's row its goal's title, its row's `goal`'s title,
    else its change's title (a goal's
    landing takes the goal's title as its words, `../../domain/forge/branches-closer.md`,
    1), a link to the goal's `Task`; a change's row its change's title, a
    link to its change's `Task`; then " · signed by NAMES", in the
    byline's style, while the `landed_default` row the row was made from
    kept sign-offs, a goal's row its newest landing's (*Who signed*);
    then, on a goal's row whose row names a `milestone`
    (`../forge/records.md`, *A project's changes*), " · in Store shape" in
    the byline's style, its milestone's title, and " · in Store shape, now complete" on the row whose
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
    row it was made from, named as the engine names them in that row
    (`../forge/records.md`, *A project's changes*, `ChangesSigned`'s
    `name`), as Team names people (`team.md`, 1):
    "you" for the person reading, first, then the others in `signed`'s
    order, each its `name`;
    then, as one more item, the parties with no `name`, the project's
    document not listing them, "one other" or "N others", N in words from two to nine, then
    digits. Two items are joined by " and ", more by ", " with " and "
    before the last: "signed by you", "signed by you and Sam", "signed by
    you, Sam and Kit", "signed by Sam and one other", "signed by two
    others". Work reads no project's people for it. One pure function of the domain's,
    `temper_client_domain::pages::work::signed`, gives the words.
  - **Under Drafts,** Work's head and filters, then the Drafts section alone.
  - **Before an answer and with nothing:** a section claims nothing until its
    lists have answered (`README.md`'s intro; *All*, above). With every list
    answered and nothing listed, All is its head alone, "Nothing in flight
    yet."; Drafts reached by its address with none says "None.", once the
    chats' and the drafts' lists have answered, as Goals' filter does.
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
  `?state=live` or `?state=ended` (`../web/addresses.md`, 1). Choosing one is a move.
  A goal is a tracked agent's task (`goal.md`, 2) at its own `Task` address, so that
  Goals is the list and draws no goal page. Its parts, in order:
  - **Its intro:** under Work's head and filters, the heading, at
    level 2: "5 goals, 3 under way.", the first number
    every row and the second those live and those needing the person; "1 goal,
    1 under way." in the singular; "No goals yet." with none; "More than N
    goals, M under way." while rows remain unread. The heading and the
    rows draw once the goals' live watch and ended read, the milestones'
    list and the needs' read have answered, each refused or failed
    counting as answered with none, so that a row's state, its wait's
    words and its milestone's group are whole when first drawn
    (`../../draft/page-round-trips.md`, section 6); before, the
    heading's line is held as Work's is (*Its head*). Then "A goal turns a draft
    into landed work, its own way: W.", W the project's ways in order, lower
    case, joined by ", " (*Ways,* below).
  - **Its filters:** "All", "Needs you", "Under way", "Ended", the chosen one
    pressed; at the section head's right, the `.link` "Start one from a draft",
    opening `goal.md`, 3's sheet with no draft, while the project has a tracked
    template. Under them, the rows, or, once its lists have answered (*Its intro*), "None.";
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
  tracked, by priority, parts: [GOAL_PART] }`, the label template `shapes::GOAL`'s, its live group
  watched and its ended group read by when each ended, as `Chats`, and read
  again only as its mark grows (`../forge/parts.md`, 6); `listed`
  puts goals in the project's index. The rail's facts that the project
  has goals or milestones take this watch and read, and the milestones'
  list's, while the page holds them, with no narrower twin beside them
  (`milestone.md`, 1, *`Domain::has_goals`*). Each row's goal part carries its
  report's headline and when it was written (`goal.md`, 1, *A goal's
  report*; `../forge/parts.md`, 3), with no read per row. The needs are `inbox`'s
  group for the project, each row's `selection` naming its goals, read
  once as the page opens and again only as their marks move: `me`'s
  `needs_total` moving, or the inbox's mark, the rail's standing watch of
  `inbox` (`needs-you.md`) delivering again, which the engine does only
  once it has marked the party's inbox changed (jig's
  `client/domain/contract/watches.md`, 1.2). That watch's first delivery
  is no move; each after it is, the first once it opens again after a
  drop among them; and no delivery of the goals' is one. A read of them
  that answers without its value keeps the `selection` the last one gave,
  so a goal's "waiting on you" stands through it (`README.md`'s opening);
  a row's headline is its goal part's as last decoded
  (`../forge/parts.md`, 5).

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
    one function of the view's parts, `shared::row_title`, which the
    chats' heading's cut (`chat::short_title`, at 72) shares, and which
    every line, row and card naming a task by its title takes
    (`shared-behaviour.md`, 1, *A task's title on a line*): Work's chat cards,
    landed drafts, nested rows and from lines, and a milestone group's
    rows, as this row.

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
  landing log (`../../domain/after-landing-storage.md`, section 1), a ring of
  `landings_kept` landings per repository, through `landed_default`'s
  `forge.changes_landed` rows, so a busy week's landings into goal branches
  can push its older landing-branch landings out of the ring, and more than
  `forge.changes_landed` of them show as that many. Default: so, nothing
  stored added (the person, 2026-11-11); or the log keeps landing-branch
  landings apart, a stored field.
- **A goal row's count of landings** (1, *Goals*; `goal.md`, 2): `TaskLine` counts a goal's
  delegates ended done whatever they did, so a row and the goal's headline say "N of
  M tasks done.", and only the goal's own rows, whose chips carry their results, say
  "landed". Default: so; or jig's `Endings` counts change results, a stored and a wire
  field.
