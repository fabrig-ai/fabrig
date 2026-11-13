# Needs you

## 1. The page

- **The inbox in the terminal** (`inbox.html`; the browser's is *Needs
  you,* 1) shows what needs the person, then the
  updates, each entry its card (`../README.md`, section 1), under two headings, each
  with how many entries it lists beside it, the updates' as new ones
  since what is read leaves them (below): "Needs you 2", "Updates 2
  new", or "0"; "2+" and "2+ new" where the inbox's document says its
  list was cut (jig's `more`, an owner inbox's next page, a filtered
  page's `cut`), so that a count never says less than there is; with
  none, once that section's read has answered, "Nothing needs you." or "No
  updates.", and no count before it. Its first heading's
  row holds "+ New chat", a link to `NewChat` (`/chats/new`, its text area
  focused: no act of its own), then
  its filters as folded menus, in order: projects (only with two or more),
  goals, then Kind. Each menu names its choice: "All projects" or the
  project's name; "All goals" or "Goal · TN"; "Kind" or "Kind · Question".
  The project menu holds All projects and each project; the goal menu holds
  All goals and the Goal field; Kind holds All kinds and each kind. A choice
  moves to the filtered inbox's address and folds its menu (`../web/README.md`, 2.3).
- **A card that needs the person** begins with its kind's tag, then
  "PROJECT › TN TITLE", for its own task alone, and its wait at the right
  (`../view.md`, 6). A proposal holds no proposing task's title, so its line stops at
  TN; questions, held tasks and person tasks use their documents' titles.
  A batch proposal keeps the mockup's Why, Goal, Landing and Budget rows:
  its reason quoted, its executor and specification, its landing resources
  and grants with priority, and its funding once. Template and charter
  names are not supplied, so their existing numbered words remain. Its
  acts are "Accept", "Reject with reason" and "Open TN ↗"; a chat's card
  keeps "Open in inbox ↗". Under its words, its reason line, with Stop
  asking and its offers numbered as any control (1, *Why it
  reached you*); its project's `decided` document is one *Decided
  for you* watches, the unfiltered inbox watching `me`'s first projects.
- **A finding that is the person's** (`../../domain/areas.md`, section 5)
  is a card that needs them, from the forge's `findings` subject with
  `mine` and `new`, watched for each of `me`'s projects as `decided` is
  (`../forge/records.md`): its tag "Finding", then "PROJECT › P/N TITLE",
  and its wait since it was filed; its kind, severity and area, "no area"
  where it has none; the last hand-off's why, quoted, with who handed
  it; and its acts, "Hand on", an area's menu and a line of why, "Route
  to a goal", the project's live goals' menu, "Backlog" and "Drop with
  reason", each the forge's request, a refusal said in its words. Kind
  holds "Finding" among its choices. The card leaves once the finding
  leaves `new` or stops being theirs.
- **An update's card** has its heading as the link that opens it and marks
  a result seen. It repeats no project line or task-number link. The heading
  says "TN TITLE replied", "TN TITLE is done", or "TN TITLE ended";
  while its document is unread, the existing "Task N" words are the heading's
  link. Then the result's first line and the unread count and age (`../view.md`, 6).
  A reply has no preview: its document carries no reply's first line.
- **A question** in the inbox is a delegate's, asked of a task the person
  requested, their chat among them. It has no heading of its own: its line,
  then its words as written. "Answer in words" is folded until opened,
  holding "Answer words" and "Answer"; "Open TN ↗" opens the asked task.
  The answer goes to that task, which relays it to its delegate. "Pending"
  and "Answered" keep today's request outcomes, and a refusal stays under
  its card. This is the terminal's card; the browser's reads the
  question's words as *Needs you* (1) says.
- **Seen** is how far the person has read: jig's read positions (jig's
  `client/domain/pages.md`, section 3), one over their results and one
  over each chat's turns. An update is seen when it is opened, and no
  page offers "Mark seen":
  - *a chat's turns,* once its page shows them: as its window loads,
    and as turns arrive while it is open, through the newest its window
    holds;
  - *a result,* when the person follows its entry's link from the
    inbox: through that result, and so through every result before it,
    since jig keeps one position over them. A result opened from
    another page, or its entry left unopened, stays unread.

  A position only moves forward, with one `Seen` in flight for each;
  one refused or failed is sent again at the next move. What is read
  leaves the inbox's updates, and a task forgotten once read is erased
  then (jig's `domain/people/README.md`, section 5). The pages draw no mark of
  their own for what is unread, beyond the inbox's counts of it
  (above).
- **Needs you** (the user's greenfield mock's `needs.html`; `needsPage`):
  `Inbox { filter }` in the browser, its address and its filters as they
  were (`../web/README.md`, 2.3), across projects by default
  (`../../draft/surfaces.md`, section 4; the whole story's `needs.html` and
  `phone-need.html`): no filter menus and no "+ New chat". Updates
  leave the browser's page and no page replaces them; `Seen` marking
  stays as it is. Its unfiltered address, `/`, works across projects,
  each card's from line opening with its project's chip. On a wide page
  it is a list of the open cards (1, *What the client keeps*), in
  their order, oldest first as the inbox orders them: the shown one, the
  one a Home row asked for, else the first, open as *The card*; each other
  closed, a `temper-need-row`, its from line and its SHORT (below), the
  whole a button that opens it (`needs::Act::Open`), closing the one open.
  On a phone it is one card at a time, the shown one, as below. Until the
  inbox has answered, the column is empty, so that "Nothing needs you."
  never flashes, and what it said stands through each refresh after that.
  - *The column,* as the mock's `.col.narrow.flow`: 600 wide, centred,
    gap 32; 720 wide on a wide page's list.
  - *Its intro,* `temper-needs-intro`, while a card is open: the kicker
    "Needs you · all projects" unfiltered, "Needs you · PROJECT" filtered
    by a project; the level-1 heading, Home's needs sentence over the open
    cards, then with a Decided for you row showing " Four were decided for
    you." or " One was decided for you.", in number words to "Nine",
    digits after. What reaches the person, the lede `needs.html` draws, is
    `../../draft/what-reaches-you.md`, section 2's, and joins the intro with
    its classes.
  - *Its head,* on a wide page: the level-2 heading "Needs you · N", N in
    the signal colour, "N+" where the read is cut, and at its right "oldest
    first", muted. Its place, "2 of 3": the total is what is answered of
    this session's page plus the open; where the read is cut, "2 of 3+",
    drawn in the phone's bar (*On a phone,* below).
  - *On a phone* (the whole story's `phone-need.html`;
    `../../draft/surfaces.md`, section 10), the browser's alone, at 760 px and
    under, while a card is open: the page's own top bar,
    `temper-needs-bar`, takes the frame's, whose switcher and Away chip are
    not drawn there, the night switch keeping its place at the right (`../view.md`, 3).
    It holds "‹ Needs you", a link named "Needs you" to Home, where Done's
    "Home" goes (*Done,* below), the "‹" drawn by style; then its place,
    "1 of 2", muted, small. Over 760 px it is not drawn. Under it the open
    card alone: no intro, no head, no dots and no "Next:" line; its question
    19 px; its choices stacked, full width, each at least 44 px tall, its
    words at the left, the recommended first; then *Tell it something
    else*.
  - *The card,* on no ground (`.ask`, no background, no shadow, padding
    0), gap 16:
    - a `.mono` line, its from line: the project's chip, `temper-chip`, on
      the unfiltered address; TITLE, a link to the task, · kind · age, as
      Home's, " · later" with it; the kind "a question", "a proposal",
      "held", "a person task", "an approval", or "funds" for a held task
      whose reason is `HoldReason::Budget`. The goal or the role a need
      comes from and who asked it join the line as their reads come
      (`../../draft/surfaces.md`, 4);
    - `h1.flow-q`, the question line: 26px/1.35, 500, -.01em; 19px on a
      phone;
    - `.ctx`, `--gf-ink-2`, if it has a context;
    - a disclosure "What the agent saw", if the agent said: its summary
      width fit-content, 14px muted, its paragraph margin-top 6; closed
      at first;
    - `temper-why`, the reason line (*Why it reached you,* below), over
      the choices, as the whole story's `needs.html` draws its `.why`:
      13px, `--gf-signal-ink` on the signal ground, `--gf-signal-pale`,
      radius 6, padding 7px 10px, its acts `--gf-signal-ink`, underlined
      in `--gf-signal-line`; "Keep asking" and the narrower rule at its
      end, and the offers under it when open (*Stop asking,* below);
    - `.choices` in a column: min-height 54, padding 10 18, radius 12,
      16px; the recommended one, with none the first, primary, and on a
      question alone `.rec`, "recommended", at the far right, 12px, 400,
      opacity .72;
    - *Tell it something else,* `temper-need-tell`, on a question and a
      person task in words, always open under the choices: a textarea named
      "Or tell it something else", min-height 72, `--gf-line-2`, radius 8,
      `--gf-paper`, its placeholder the same words; its words are the
      card's reply, kept as a reply's are (1, *What the client keeps*),
      and go to the asker as its next message; "Send", primary and small,
      while it holds words, answers in them as a reply does. On a
      proposal, a held task or a person task other than one in words,
      "Open", quiet, to its task's or chat's page, and with no such page it
      is left out. The browser draws no "Later"; the terminal's inbox keeps
      it.
  - *By its kind.* A question reads its words by their line form
    (`../../domain/agent/agents.md`, 2); words in no such form are answered by
    *Tell it something else* alone. A proposal's question line is its reason's first
    sentence, without a heading's marks, its facts and rows as `.ctx` beneath, then Accept (primary,
    with no "recommended") and Reject (plain, its reason's textarea as on
    the terminal's card), through the confirmation they have (`../README.md`, 3.6). A
    held task: its title and its words, its facts as the terminal's card
    has them, then Release (primary) and Leave held (plain, its reason's
    textarea as Reject's). A funds card: "S of L used." as the
    question, in the task page's money form, plain numbers, its title as
    owner, then Release and Leave held; there is no "Add $10". Its hold's
    words say what a person can do: accept a proposal for more budget, if one
    waits, then release it; otherwise close it and start the work again once
    their pool has room, which closing an open chat makes. A person task: its
    title, its spec, its acts as plain buttons. *A person task in words,*
    one addressed to a person with a report as its contract, as a goal
    asks the person who started it (`../../domain/agent/agents.md`, 2), reads as a
    question: its spec's words by the same line form, its kind "a
    question", its choices and *Tell it something else*. Choosing or replying
    answers it in those words, naming the revision the card showed;
    Later keeps it open. With a task as requester, its line's TITLE is
    "Task N", N its requester, a link there, since the card reads no second
    task; another requester keeps the person task's title and link.
    Its answered line's title is its own. In the inbox's filter it stays
    a person task. On the terminal's inbox, Answer sends its words with
    the shown person revision. A role's task or a contract other than a
    report keeps the ordinary person-task card.
  - *An approval,* a change waiting on a review on the forge: a row of the
    subject `approvals` (`../forge/records.md`), its kind "an approval", its question, context,
    size, review link and "Open" as `../forge/README.md`, 4.1 has them, the review link
    "Review on GitHub ↗" or "Review on Forgejo ↗" first and primary, one tap
    to the forge's review page (`../../draft/forge-identity.md`, section 8), and
    in the terminal's inbox "Later" too, its key
    `ObjectKey::Task` of the change, so that Later works on it as on any card.
    The open cards are the inbox's needs and these rows together: the inbox's
    first, in its order, then the rows in theirs, and those marked later last.
    A filter by project takes the project's rows, one by goal the rows of the
    changes under that goal, and one by kind none, since no kind of the
    inbox's is theirs. They join once the inbox has answered, and until the
    `approvals` watch has delivered there are none. On the terminal's inbox
    there are none either.
  - *Whole,* since a person decides on these words. The engine's read of a
    person task cuts its words at `documents.text_bytes`, so a card whose words
    are cut reads the rest through `TaskText`, as a goal's page does (`goal.md`, 2), up to
    `long_text`, which bounds a task's words (`../view.md`, 4). Only the whole parses by the
    line form, so its context, "What the agent saw" and its choices draw once it
    has it all; until then the card reads as words in no such form, ending in
    `…`, answered by *Tell it something else* alone. A read that ended short is asked again at the
    inbox's next refresh. Words with more than 16 choices read as words in no such
    form, whole. A question by `message` has no such read: its words stay as the
    document has them.
  - *Cut,* what a card says while its words stay short of the whole: a person
    task's whose read ended, a `message` question's cut by the document's bound.
    Under the words, in `.ctx`: "This question is cut here." and the link "Read
    the rest", to its task's page (the requester's, for a person task in
    words), whose conversation holds them. No question is cut without it.
  - *SHORT,* what a closed row or a Home line says of a card: a
    question's line, or a person task in words', from the document's bounded
    words; a proposal's reason's first
    sentence; a held task's or another person task's title, a funds card's
    "S of L used"; an approval's question.
  - *Answered,* under it when any: the section title "Answered", then
    the `.answered` lines.
  - *Only you,* after them, and in the done state after "Home", while an
    item shows (1, **Only you**), before *Decided for you*.
  - *Decided for you,* after them, while a row shows (`home.md`, 1, **Decided for
    you**): the page's filter's project's rows, or, unfiltered, every
    watched project's.
  - *Done,* with none open, `.col.narrow` with no head: the kicker "Needs
    you", the level-1 heading "Nothing needs you.", the lede "Every
    question is answered. Work carries on; Home tells you what changed.",
    the `.answered` lines, and a plain button-link "Home": to `AllHome`
    while `me` lists two projects or more, else to the open project's
    `Home`, else the first project's.
  - *Its acts.* A choice answers and opens the next card. "Send" answers
    in words. On the terminal's inbox "Later" marks it and moves on, the
    toast saying "Later: it stays in Needs you, after the others."
  - *Why it reached you* (`../../draft/what-reaches-you.md`, section 5): one
    line of a card, in `temper-why`, over its choices on this page and
    under its question on Home's cards (`home.md`, 1, *The cards*), as the
    whole story's `needs.html` and `home.html` place it, from one domain function
    every shell calls (`pages::needs::why`), its words the view's shared ones
    (`temper_client_view::shared::why`), so that the TUI's walk says it
    too (`../tui/pages/launchpad.md`, 6). By the card's kind:
    - *A question,* by `message` or a person task in words. Its asking task
      is a question's sender where that is a task (`from`,
      `Requester::Task`), else the question's own task, and a person task's
      requester where that is a task. Its row is the newest `asked` row
      (`../forge/records.md`) of that task in the card's project's `decided` document whose
      `at` is at or before the card's `since` (a person task with none
      takes the newest). The row's reach says why: "Reached you because
      CLAUSE, which CLASS." CLASS is "asks you unless you’re away" for
      `ask`, "asks you, even when away" for `ask_away` and "always waits
      for you" for `wait`. CLAUSE names the reason's subject: for a reason
      `kind`, "it is a wording choice", "it retries or splits a failing
      task", "it is an implementation choice", "it changes only tests", "it
      adds a dependency", "it departs from a landed design", "it spends
      money past a budget", "it can’t be undone", "it changes what reaches
      you" (`rules`), "no kind of decision names it" (`other`); for a reason
      `set`, "it changes NAME", NAME the set's name in the project's classes
      (`settings.md`, 2), or "one of PROJECT’s path sets" until they are read. With
      `under` an item with words, " under “WORDS”" follows the subject. A
      reason `always` reads "Reached you because its goal always asks you."
      So: "Reached you because it changes vsr’s protocol core, which asks
      you, even when away."
    - *A question with no row,* its project's document delivered and not
      `cut`: "Reached you because it was asked of you directly.": no call
      judged it, so it came by its route (kind 6,
      `../../protocol/configuration/settings.md`, 2) or without the tool. With no
      row and the document cut, refused, ended or not yet delivered, no
      line. A row whose reach says decided, which an option past the call's
      leaves, reads as none.
    - *A proposal,* "Reached you because a proposal always waits for whoever
      may accept it."; *held,* "Reached you because a held task always waits
      for whoever may release it."; *funds,* "Reached you because money past
      a budget always waits for you." An approval, whose rule its context
      names, and a person task not in words, the person's own work, have
      no line.
    - *Its card waits:* a card of a question, on this page and on Home,
      draws once its project's `decided` document is delivered or its watch
      refused or ended, as the column waits for the inbox, so that a line
      never moves the choices under a finger.
  - *Stop asking,* at the reason line's end, offered where the reach asked and
    the kit offers something for it (`offers`, `../../domain/forge/attention.md`, 6:
    none for a fixed kind, for `always`, or where the table has no room),
    the card's project's classes are read (`settings.md`, 2), the person's dismissals
    are delivered, and no Keep asking of the reach's subject in that
    project is less than 7 days old; with none of it offered the line is
    its sentence alone.
    - *On this page,* as the whole story's `needs.html` draws it, the
      line's sentence is followed by two acts, buttons in the `.link`'s
      look, joined by " · ": "Keep asking" (*Keep asking,* below), then
      the narrower rule, "Stop asking: let agents decide RULE",
      `State::Expanded` while open. RULE names what the kit's first offer
      narrows to: for `Decide(subject)`, OBJECT (*Its offers in words,*
      below), so "Stop asking: let agents decide new dependencies"; for
      `Words(set)` or `Split(set)`, "some changes to NAME", NAME the set's
      name in the project's classes, so "Stop asking: let agents decide
      some changes to vsr’s protocol core", since the person words that
      narrowing, or takes the split, once it opens. On a phone the line is
      the same.
    - *On Home's cards* (`home.md`, 1, *The cards*) and the terminal's
      inbox (1, *A card that needs the person*), as the whole story's
      `home.html` draws it, the line ends in "Stop asking" alone, a button
      in the `.link`'s look, `State::Expanded` while open, and "Keep
      asking" stands under the offers.
    - *Its offers,* opened in place by the narrower rule or "Stop
      asking", under the line, `temper-why-offers`: a column, gap 8, a row
      an offer in the kit's order, its words at the left and its act at
      the right, `.btn.small`; then, on Home's cards and the terminal's
      inbox alone, "Keep asking", quiet and small. One is open at a time;
      pressing its act on the line again, another card's, an answer,
      Later, Keep asking or leaving the card closes it.
    - *Its offers in words,* each the narrowed table's change
      (`narrowed`): `Decide(subject)`, "Let agents decide OBJECT, then tell
      you."; `Words(set)`, "Let agents decide changes to NAME that are:",
      then under that sentence a `.text` field, "Which changes", its
      placeholder "fixes that keep the wire format", 1 to 120 bytes, the
      two together at the row's left, its act off until the field holds;
      `Split(set)`, "Let agents decide changes to PATTERNS, apart from the
      rest of NAME.", PATTERNS the matched patterns joined by ", ". OBJECT
      is the subject's plural: "wording choices", "retries and splits",
      "implementation choices", "test-only changes", "new dependencies",
      "departures from a landed design", "changes to NAME", with `under`'s
      words after it as in CLAUSE.
    - *Its act:* an owner of the card's project (`me`'s row for it `Owner`)
      has "Apply": jig's `ChangeSettings` of one item, kind 8 of that
      project (`../../protocol/configuration/settings.md`, 2), set to the offer's
      table, running work staying, against the newest revision the person
      has, as `settings.md`, 2's controls send it, a `stale` refusal sent once more
      against the revision it names. Anyone else has "Propose": jig's
      `Propose` in the deployment's scope, a change of settings of that one
      item against the newest revision, its reason "Stop asking in PROJECT:
      let agents decide OBJECT, then tell whoever they would have asked.",
      which a party with a role in the project may make and the owners
      decide (jig's `domain/people/acts.md`, 1, and `jig/docs/design/domain/engine/settings.md`, 6);
      it is the person's own proposal, never a procedure's. While one is out
      every act of the open offers, and the line's own, is off. Applied, the toast "Applied: the
      next one is decided and listed. This one still waits for you.";
      proposed, "Proposed: it applies once an owner accepts it. This one
      still waits for you."; refused, jig's words under the offers, as a
      settings row's refusal (`settings.md`, 2), the offers staying open. The need in
      hand waits for its answer whatever is applied: the rule changes what
      comes next, never what was asked.
  - *Keep asking,* on the line or under the offers, one act of the card's
    own, `stop_asking::Act::Keep { object }`, the card's object, whether or
    not its offers are open, sends the FORGE request `Dismiss { keep_asking(project,
    subject, at) }` (`../forge/records.md`), `subject` the card's reach's (the kind for a reason
    `kind`, the set for a reason `set`), `at` the wall, one act, one request
    (`../README.md`, 3.6), and closes the offers where they are open. Answered, the toast "It keeps asking.
    Stop asking comes back in 7 days."; refused or lost, "temper couldn't
    keep that. Try Keep asking again."; *unknown,* the subject `dismissed`
    answers it. While it is out, and once kept, the line offers neither
    Keep asking nor Stop asking for that subject in that project, on every card and device,
    until 7 days after its `at`; a later Keep asking of the same replaces
    it (`../../domain/forge/attention.md`, 1).
- **Tell temper** (`../../draft/surfaces.md`, section 9; the whole story's
  `needs.html` and `team/needs.html`, their foot, and `phone-tell.html`): a
  person's words about what's off, filed as a finding
  (`../../domain/findings.md`) in the project in view and sent to its owner
  (`../../domain/areas.md`, section 4): the owner of the area the page they
  were on gives (*Where it goes*), else the project's triager, which reads
  the words and the page and hands it to its area's owner, or keeps it
  (`../../domain/areas.md`, section 5), else the project's owners. It
  answers nothing: a question is Ask temper's.
  - *Where:* a region named "Tell temper" at the foot of a project's Needs
    you, its filter naming the project, after Decided for you, on a wide
    page; Needs you across projects has no project in view and draws none.
    On a phone, Ask temper's page's second act, the link "Tell temper" under
    its composer, to `Tell { project }` at `/projects/<project>/tell`, a
    page of the phone's Ask tab ("Ask" current, `../view.md`, 3) whose top
    bar holds "‹ Back", a link to Ask temper's page, then "Tell temper",
    muted, small, and the night switch. The terminal's `/tell` is
    `../../draft/surfaces.md`, 11.4's (`../tui/pages/README.md`, 2,
    *`/tell`*).
  - *The page you were on:* `Pages::was_on(&self) -> Option<&tell::About>`,
    the last page the person opened in the session but Needs you, `Tell`,
    Ask temper's page and an Ask temper chat, the pages Tell temper is
    reached from, kept as it was left: `tell::About { address: Address,
    title: Option<Box<[u8]>> }`, `title` the task's title for a page of a
    task, `Task`, `Talk` or `Chat`, and the role's for Team with a role's
    card, `Team { role: Some(task), .. }`, as the domain held it when the
    page was left, else none. Kept as *What the client keeps* is, gone on
    reload or sign-out. It is Tell temper's page only where it is the
    project's, its address naming the project or, for a page of a task, the
    task being the project's; else Tell temper has no page.
  - *Where it goes:* `tell::goes(root: &Domain, project: u32, about:
    Option<&tell::About>) -> Option<tell::Goes>`, which the browser and the
    terminal share: the project's `findings` document
    (`../forge/records.md`), the one Needs you's finding cards read
    (`pages::findings`), given to one pure rule, `tell::route(document:
    &w::Findings, about: Option<&tell::About>) -> tell::Goes`, and none
    until that document is read:

    ```rust
    pub enum Goes {
        Area { name: Box<[u8]>, holder: team::Holder },
        Triager { holder: team::Holder, areas: bool },
        Owners { areas: bool },
    }
    ```

    1. the page names a task, `Task { task }`, `Talk { task, .. }`, `Chat {
       task }` or `Team { role: Some(task), .. }`, and that task holds one of
       the project's areas: `Area`, the first it holds in kind 14's order,
       its name and holder;
    2. else the project's settings name a triager and it stands, the
       document's `holder` being what the `triager` gives (`task N` task N,
       `party N` party N, `role N` role N, `charter N` a task): `Triager`,
       that holder;
    3. else `Owners`, role 0 (`../../domain/areas.md`, section 4).

    `areas` says whether the project has an area to hand it to. So a word
    said on a standing role's page, its card or its talk goes straight to
    the area the role owns, with no agent's run; any other goes to the
    triager, who judges where it goes from the words and the page, as
    `../../draft/standing-roles.md`, section 4 has a role route by
    judgement. Routing by path stays later (its section 13).
  - *Its parts,* in this order: on a wide page the head "Tell temper",
    at its right "feedback becomes a finding", muted; on a phone the
    headline "Something off?" and the lede "Say it in your words. temper
    files it as a finding and sends it to the area's owner, who answers on
    it."; the field, a textarea named "Something off?" on a wide page and
    "What you saw" on a phone, two rows and four; "About PROJECT · the page
    you were on, TITLE" where Tell temper has a page, else "About PROJECT",
    TITLE the page's: a page of a task its kept title, else "Task N"; any
    other page its name as the rail, the tab bar or the page's own heading
    says it ("Team", "How work runs", "Work"); the "Goes to" line
    (*Goes to*, below); "File it", primary, while the field holds words;
    then "Ask temper instead", quiet, on a phone "It's a question: ask
    temper instead", which opens Ask temper on the project with the words
    in its composer, unsent, as the Ask temper line's `Act::AskWith` does
    (`home.md`, 1). The field's words are kept in `Pages` while the region
    or the page shows, as a chat's composer keeps its own, so that a redraw
    keeps them and "File it" reads them.
  - *Goes to,* by `Goes`, NAME a holder as Team names it (`team.md`, 1,
    *An owner's name*), "you" where it is the party `me` names:
    - `Area`: "Goes to NAME, AREA's owner"; the person's, "Goes to you, as
      AREA's owner".
    - `Triager`: "Goes to NAME, PROJECT's triager"; the person's, "Goes to
      you, the triager"; with `areas`, then ", who hands it to its area's
      owner".
    - `Owners`: "Goes to PROJECT's owners"; with `areas`, then ", who hand
      it to its area's owner".
  - *Its act:* "File it" files the words as a finding of the project's,
    the one act and one request of `../README.md`, 3.6, through the engine's filing
    (`../../domain/findings.md`, section 4), its `about` the page Tell temper
    has, else none; filed, the field empties and the toast says "Filed for
    NAME.", NAME as *Goes to* names the holder of the `Tell`'s `goes`, "you"
    the person's ("Filed for you."), `Owners` "PROJECT's owners", and
    "Filed." where it kept none; refused, the words stay and the refusal
    says why under the field. Words past the finding's bound are cut
    nowhere: "File it" is disabled while they are past it, saying how far.
  - *The domain's act,* which "File it" sends and the terminal's `/tell`
    sends (`../tui/pages/README.md`, 2, *`/tell`*): `pages::Act::Tell {
    project: u32, words: Box<[u8]>, about: Option<tell::About> }`, its
    words trimmed of white space at both ends, `about` the page they were
    on: the browser's `was_on` where it is the project's, the terminal's its
    screen. Empty, it does nothing; past 4,096 bytes, the finding's
    body's bound, it sends nothing and keeps `tell::State::Long { over }`,
    the bytes past it; else one `FileFinding` of the project
    (`../forge/records.md`), sent through the forge's family as a finding
    card's acts send theirs: kind `feedback`, severity `medium`, its area
    the `Area`'s name that `tell::goes` gives at the act, else none, so
    that the triager holds it, or the project's owners while none stands
    (`../../domain/areas.md`, section 4); its title the words' first line,
    cut at a character's edge to at most 200 bytes; its body the words
    whole, then, where `about` is given and the whole stays within 4,096
    bytes, a blank line and `Page: PATH`, PATH the address as
    `address::print` writes it, then ` (TITLE)` where a title is kept, so
    that the triager and the owner read the page as well as the words:

    ```
    The courier link opens in English for German customers.

    Page: /projects/2/talk/52 (Mailer)
    ```

    One at a time: a `Tell` while one is sent does nothing.
    - *What it keeps:* `Pages::tell(&self) -> Option<&tell::Tell>`, the
      last one's until the next, `tell::Tell { serial: u64, project: u32,
      words: Box<[u8]>, goes: Option<tell::Goes>, state: tell::State }`,
      `serial` counting the session's `Tell`s from 1, so that a shell says
      each outcome once; `goes` what `tell::goes` gave at the act, none
      where the document was not read; `tell::State` one of `Sending`,
      `Filed { number: u32 }` once the row has committed,
      `Refused(w::FindingRefusal)`, `Lost` where the request ended with no
      outcome or *unknown*, and `Long { over: u32 }`.
    - *Its words:* a refusal's are a finding card's, the view's
      `shared::finding_refusal(w: &Words, refusal: &w::FindingRefusal)
      -> Box<[u8]>`, which the card's own words call too.
  - *Shown* on a project's Needs you, on a wide page, and on its `Tell`
    page, once `tell::goes` answers for the project, its `findings`
    document read; nothing stands in its place before
    (`../../draft/whole-story.execution.md`, section 2). The `Tell` page
    holds the project's document as a filtered Needs you does
    (`pages::findings`).
- **What the client keeps** of what needs the person: in `Pages`, not on
  a page, for this session, gone on reload or on sign-out, none of it a
  stored record, and at most 64 each of the cards marked later, the answered
  lines, the refused answers and the proposals of settings, the oldest going
  first, a line still held never:
  - *Later,* the cards marked later. *The open cards* are the needs'
    groups as `Inbox` reads them, less those answered, in the inbox's
    order, those marked later last.
  - *Answered,* a bounded list: each card's object, project, title,
    label and when. The label is the choice's words, or `Replied: “…”`,
    its words cut at 57 characters plus "…" past 60. A line shows on
    *Needs you* for 10 minutes and on Home for 2, and an answered card
    leaves the open ones at once.
  - *At,* the card a Home row asked for, until it is answered or marked
    later, or the person leaves the page.
  - *The held send.* jig has no retraction, so a choice or a reply is
    held in the client for 5 seconds, a deadline of the domain's as
    `next_deadline` has. The card moves to answered at once, with the
    toast "Answered: LABEL" and its "Undo". Undo restores the card, a
    reply's words with it, unseen until the form opens, and sends
    nothing. At 5 seconds the `Answer` goes, its words the choice's
    or the reply's; after that its line shows no "Undo". A refused answer
    puts the card back with the refusal's words on it, as does an answer
    the engine did not keep. A tab closed inside the 5 seconds sends
    nothing and the question stays open. Accept, Reject, Release, Leave
    held, Take and Hand back send at once, with no Undo, and leave no
    answered line.
  - *A press stays under the finger.* When the inbox's document changes,
    a card still in it keeps the page it shows until its new read answers,
    and keeps it if that read fails. At a filtered address the cards stay
    as they are until the section reads have answered, a section whose read
    failed keeping its own. So a choice on the page stays pressable, in the
    same element, through the read: nothing a change of the inbox does draws
    a card without its choices, or takes one away, but the card leaving the
    rows. A press of a drawn choice that a bound refuses (its words, or 64
    answers already held) moves nothing and says so, in the toast "That
    answer wasn't sent. Press it again.": an answer never goes unsent with
    nothing shown.
  - *The toast,* one at a time, 4.5 seconds, with one optional act; drawn
    with the status role at the end of the frame, in a region named "Notices"
    (skein's `ui/README.md`, 12: a page's child is a landmark) that has
    no look of its own, in the mock's `.toast`:
    fixed, centred, bottom 28 (88 on a phone), an ink ground.
  - *The rail's count:* one standing `inbox` watch while a project's
    frame shows, the rail being what reads it, so that a client on no
    project's page asks nothing more; its needs' rows counted by the open
    project, less those answered, a row counting in each project it shows in
    (jig's `client/domain/contract/documents.md`, 1: `shows_in`), so that a proposal of
    settings counts in the projects it belongs to, and for one of the
    deployment's in each the person owns; "N+" where its document is cut, and no
    count before it is delivered. A card marked later still counts. The open
    project's rows of `approvals` add to it, "N+" where that document is cut too.
    On a page across projects (`../view.md`, 3, *All projects*) it counts
    what *Needs you* across them shows: each of the needs' rows once,
    whatever projects it shows in, less those answered, and every row of
    `approvals`, "N+" where either document is cut; so it is never the sum
    of the projects' counts, which counts a row of several projects in each.
    `me`'s `needs_total`, with every row of `approvals` added, is shown
    nowhere outside a project: the top bar offers no navigation (`../view.md`, 1).
  - *The approvals,* the forge side's watch of the subject `approvals` (`../forge/records.md`),
    held for the session as `dismissed` is, from when `me` is first delivered
    while the link is live: a watch refused or ended is kept as such, with no
    rows, and not opened again until the next sign-in.
  - *Stop asking,* the one open (1): its card's object,
    the words typed for a `Words` offer, the act out and its refusal, gone
    when it closes; and the Keep asking out, by its key, until answered.
- **Only you** (`../../draft/what-reaches-you.md`, section 8;
  `../../draft/surfaces.md`, section 4; the whole story's `needs.html`): what
  only the person can do and no choice answers, listed apart on *Needs
  you*: a sign-in of their own that is signed out, or a login their account
  reads as expiring. Work goes on around it: the runs that need the account
  wait for it (`accounts.md`, 2, *A run's wait*), and nothing else does.
  - *Its read,* `pages::only_you`, one domain function every shell may
    call, from the `accounts` document every page holds (jig's
    `client/domain/contract/documents.md`, 1, and `jig/docs/design/client/domain/contract/README.md`, 5.1), read and never kept. An item
    is the deciding source, the first supplied, of an account the person
    holds (`AccountHolder::Party`, the party `me` names), but the
    playground provider's (`../forge/family.md`, 7), as the rail's mark reads it (`../view.md`, 3), whose
    kind is a sign-in or a program's login, and which is signed out
    (`SignInAgain`) or reads `expiring`, signed out where both hold: the
    source whose state holds the account's work (`accounts.md`, 2, *The composer's
    line*). Items keep the document's order of accounts. A key, a source
    needing no credential and the deployment's accounts make none. With no
    document delivered there are none; a document `cut` gives the items it
    holds, its head counting them "N+".
  - *It shows* once the inbox has answered and an item is read (`home.md`, 1, *A page's
    parts*), on the unfiltered and a filtered page alike, since an account
    is no project's. It joins neither the cards' count, Needs you's head,
    the rail's count, Home's needs sentence nor Done's rule: Done holds with
    no card open, and Only you follows its "Home".
  - *The region,* named by its heading, after the cards' *Answered*, or
    Done's "Home", and before *Decided for you*, wherever the page draws
    that, on a phone too: a `section-head`, the level-2 heading "Only you ·
    1", at its right "work goes on around it", muted. Then
    `temper-only-item` an item, the mock's `.card`, gap 10, as `needs.html`
    draws it (`../../draft/surfaces.md`, 13):
    - its from line, the mock's `.from`, in the sans, 13 px and muted,
      where a need card's is in the mono: the chip `temper-chip`
      "Deployment", where Accounts is (`settings.md`, 1), then "an LLM
      account";
    - its question line, `.q`, 16.5 px, by its source as Accounts words it
      (`accounts.md`, 2, *An account*: "ChatGPT sign-in", "Claude Code login NAME"):
      signed out, "Your ChatGPT sign-in is signed out. Sign in again?";
      expiring, "Your Claude Code login claude-alice is expiring. Sign in
      again?";
    - its act alone, "Sign in again", a link in the look of the
      recommended `.choice`, primary, with no "recommended": to `Accounts`,
      whose row holds the sign-in (`accounts.md`, 2, *Each state*). On a phone it is full
      width and at least 44 px tall.
  - *Its acts' end:* signing in again renews the account of the same
    subject (jig's `domain/accounts.md`, 3.3), and the item goes once the
    document says so. Nothing else is sent from it.

## 2. Open questions

- **The approval flow after landing** (`README.md`, `../forge/records.md`, `settings.md`, 2): check the card's
  rendered look, the owned-Forgejo review link's sign-in, and a real human
  review on each forge. The joint-world Forgejo story holds the card and
  its files-page link in place of a browser step (`../../testing/checks.md`,
  3); GitHub's composed box story and live proof are separate
  (`../../testing/README.md`, section 7). Default: client and view tables,
  with no approval added to the terminal inbox in this round.
- **A message question read whole** (1, *Whole*, *Cut*): a message has
  no `TaskText` read for its words. Default: the document's bounded words,
  an explicit cut notice and a link to its task's conversation. Reading
  that question whole needs a read of its own or carrying its whole words.

- **The asker's lineage on a question's card** (`mockups/world-solo.js`):
  it needs the asker's documents on the inbox page. Default: the asked
  task's link (`README.md`). A question's choices are its words' (`../../domain/agent/agents.md`,
  2), and a retraction of a sent answer is jig's, which has none: the
  client holds an answer before it goes (`README.md`).
- **Later and the answered lines, kept** (1, *What the client keeps*; the
  request of 2026-11-05 for *Needs you* decides session order and no store
  record): a card marked later, or a line answered, surviving a reload or
  reaching another device, which takes a stored record. Default: this
  session's alone.
- **The browser's updates** (1; that request): they leave the
  browser's page and no page replaces them, `Seen` marking staying as it is.
  Default: the terminal's list alone draws them.
- **"Add $10" on a funds card** (the greenfield mock's `needs.html`; that
  request puts budgets last): an act that widens the budget in place. Default:
  Release and Leave held, the hold's own acts.
- **A chat's own question, as a card** (the mock's `n-license`; that request):
  a chat's question in its words stays in its conversation, with no inbox
  entry. Default: as it is.
- **A goal hearing a choice, as a story** (1, *What the client keeps*;
  `../../domain/agent/agents.md`, 2): that request asks one live walk of a scripted goal
  asking in the line form, a person pressing its recommended choice on Home and
  the goal hearing those words, whose scripts are throwaway. No world keeps it:
  the parser, the stock sentence, the session and the view each hold their own
  side. Default: those tests.
- **Tell temper's area, chosen by the person** (1, *Tell temper*, *Where it
  goes*; the whole story's `needs.html` and `phone-tell.html`, which say
  where a word goes and offer no other place): a control on the card
  naming another of the project's areas, or the triager, before "File it".
  Default: none; a word filed with the wrong area is handed on by its owner
  (`../../domain/areas.md`, section 5).
- **Tell temper judged before filing** (`../../draft/surfaces.md`, section
  9, "its area found from the page they were on and the words"; the
  mockups' "Goes to" naming an area from words alone): an agent's run
  reading the words as they are typed, to name the area before "File it".
  Default: the page's role alone names an area before filing, and the
  triager judges the words once filed, one short run an item
  (`../../domain/areas.md`, section 6).
- **What became of a word, in the brief** (`../../draft/surfaces.md`, section
  9, "what becomes of it returns in the brief"; `phone-tell.html`'s "It
  comes back in your Home's brief"): the status agent's words naming the
  findings a person filed through Tell temper and what became of them
  (`../../domain/forge/attention.md`, 3). Default: not drawn; the
  card says nothing of it, and the finding's issue carries its owner's
  answer.
