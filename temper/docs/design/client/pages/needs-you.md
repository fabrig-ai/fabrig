# Needs you

## 1. The page

### 1.1 The terminal inbox

Moved to [`needs-you-terminal.md`](needs-you-terminal.md), section 1.

### 1.2 The browser's cards

- **Needs you** (the user's greenfield mock's `needs.html`; `needsPage`):
  `Inbox { filter }` in the browser, its address and its filters as they
  were (`../web/addresses.md`, 1), across projects by default
  (`../../draft/surfaces.md`, section 4; the whole story's `needs.html` and
  `phone-need.html`): no filter menus and no "+ New chat". Updates
  leave the browser's page and no page replaces them; `Seen` marking
  stays as it is. Its unfiltered address, `/needs`, works across projects,
  each card's from line opening with its project's chip. On a wide page
  it is a list of the open cards (1, *What the client keeps*), every
  open need among them, whether its card is read or not, in
  their order, oldest first as the inbox orders them: the shown one, the
  one a Home row asked for, else the first, open as *The card*; each other
  closed, a `temper-need-row`, its from line and its SHORT (below), a
  held card's why it's held in its SHORT's place and a person task's
  none, since their SHORT is the title the from line names (*Its title
  once*, below), the
  whole a button that opens it (`needs::Act::Open`), closing the one open.
  A need whose card is not read (1, *What the client keeps*, *The needs'
  rows*) is a closed row drawn from its row alone: its from line, the
  project's chip on the unfiltered address, its task's TITLE from the
  row's task link, "TN" where the link holds no title, its kind and its
  age; no SHORT. Wherever this page draws a need's age, a closed row and
  its card's from line among them, "Overdue" follows it, in the
  signal's colour, where the need's row's bound is *overdue* and has
  passed by the view's clock (jig's
  `client/domain/contract/documents.md`, 1, *an inbox*;
  `../../draft/bounded-waits.md`, 8). Opening it reads its card, and it opens as *The card*
  once that read answers, closed until then. In the terminal's inbox
  such a need is its row alone, its kind's tag, "PROJECT › TN TITLE" and
  its wait, with "Open TN ↗".
  On a phone it is one card at a time, the shown one, as below. Until the
  inbox has answered, the column is empty, so that "Nothing needs you."
  never flashes, and what it said stands through each refresh after that.
  - *The column,* as the mock's `.col.narrow.flow`: 600 wide, centred,
    gap 32; 720 wide on a wide page's list.
  - *Its intro,* `temper-needs-intro`, while a card is open: the kicker
    "Needs you · all projects" unfiltered, "Needs you · PROJECT" filtered
    by a project; the level-1 heading, Home's needs sentence over the open
    needs (1, *What the client keeps*, *The open cards*' count), then with a Decided for you row showing " Four were decided for
    you." or " One was decided for you.", in number words to "Nine",
    digits after. What reaches the person, the lede `needs.html` draws, is
    `../../draft/what-reaches-you.md`, section 2's, and joins the intro with
    its classes.
  - *Its head,* on a wide page: the level-2 heading "Needs you · N", N in
    the signal colour, the headline's number (`home.md`, 1, *the
    headline*: every open need, or the findings' `Counted::total` where
    the sentence counts findings), "N+" where the read is cut, and at its right "oldest
    first", muted. Its place, "2 of 3": the total is what is answered of
    this session's page plus the open needs; where the read is cut, "2 of 3+",
    drawn in the phone's bar (*On a phone,* below).
  - *On a phone* (the whole story's `phone-need.html`;
    `../../draft/surfaces.md`, section 10), the browser's alone, at 760 px and
    under, while a card is open: the page's own top bar,
    `temper-needs-bar`, takes the frame's, whose switcher and Away chip are
    not drawn there, the night switch keeping its place at the right (`../view-navigation.md`, 1).
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
      (`../../draft/surfaces.md`, 4). *Its milestone* (`../../draft/milestones.md`,
      5.3): where the card's task is under a milestone, the milestone and
      the goal come before TITLE, after the chip: the milestone's name, a
      link to its `Task` (`temper-ms-from`), " › ", the goal's title, a link
      to its `Task`, and, where the card's task is not that goal, " › " and
      TITLE: "fabrig · Store shape › S7 history in task logs · a question ·
      16 min". They are its task's document's lineage (`milestones::lineage`,
      `milestone.md`, 1): the document the card holds where it is that
      task's, else that task's read once by number while the card shows, as
      Decided for you reads its rows' tasks (`home.md`, 1, **Decided for
      you**); until it is read, and for a proposal, which names no task's
      title, the line is as without one. Home's cards and the terminal's
      inbox name them the same, the terminal's "PROJECT › Store shape › TN
      TITLE";
    - `h1.flow-q`, the question line: 26px/1.35, 500, -.01em; 19px on a
      phone; none on a person task's card whose from line names its
      title (*Its title once*, below);
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
    the terminal's card), through the confirmation they have (`../root.md`, 1.6).
    A party's amendment is drawn as a task's: "Amend task N", its
    specification, and its authority before and after. Its maker's card,
    on this page and in a chat, adds Withdraw (plain), offered to the
    maker alone, as the document's acts offer it, and through its
    confirmation, "Withdraw this proposal?", which sends `Withdraw` from
    the card's key and project with no read of its own; a card whose
    read was refused offers it too, when its key names the signed-in
    party. Answered, the card leaves the rows with the entry. A
    held task: why it's held as its question (*Why it's held*, below),
    then its words, then Release (primary) and Leave held (plain, its
    reason's textarea as Reject's). Its title is its from line's, and
    its words are those past their first line (*Its title once*,
    below). A task held because a task it waits on failed or
    was cancelled (jig's `HoldReason::Dependency { task, how }`) says which
    and what releasing needs as its question (`../../draft/milestones.md`,
    5.3): "S6 is held: S5, which it waits on, failed. Drop or replace that
    dependency, then release it.", "was closed" for a cancelled one, and
    "ended" where the card holds no ending; each named by its id or title
    as a wait names it (`work.md`, 1, *A wait's names*), "task N" where its
    title is not held. A funds card: "S of L used." as the
    question, in the task page's money form, plain numbers, then its
    words, its hold's words, and Release and Leave held; there is no
    "Add $10". Its hold's
    words say what a person can do: accept a proposal for more budget, if one
    waits, then release it; otherwise close it and start the work again once
    their pool has room, which closing an open chat makes. A person task: its
    spec, its acts as plain buttons, its title its from line's (*Its
    title once*, below). *A person task in words,*
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
  - *Its title once* (`../../draft/home-for-a-person.md`, 2, *Kept for
    the thing's own page*): a card's body never repeats the title its
    from line names. Where the from line of a held, funds or person task
    card names its own task by its title (TITLE, or the goal's link
    where the card's task is that goal: *The card*, *Its milestone*),
    its body draws no title: no heading of it, no line of it as owner,
    and of its words only those past their first line, the title's,
    none where nothing but spaces, tabs, carriage returns and line
    feeds follows it; a cut read keeps its cut notice. Where the from
    line names another task in its place (a person task in words',
    "Task N", its requester), the body keeps its title. One function of
    the shared parts gives those words, beside `only_title`
    (`shared-behaviour.md`, 1, *A task's words after its title*). Home's cards keep
    the rule (`home.md`, 1, *The cards*), and so does a closed row
    (above). *In the terminal's inbox* a card has no from line: its tag
    line names "PROJECT › TN TITLE", and its body keeps the title as its
    heading, over its words whole, left out where they are only the
    title (`shared-behaviour.md`, 1, *A task's words after its title*).
  - *Why it's held:* a held card's question is why it's held, in one
    sentence for a person, on this page, on Home and in the terminal's
    inbox, by one function of the shared parts,
    `shared::hold_sentence`, from its task's hold (jig's `ChipHold`):
    - a dependency's: its question, as above;
    - a hold with words (`ChipHold::words`, the account its engine or
      connector kept), not empty: their first clause, up to the first
      "; " or line feed, its first letter upper-cased where it is an
      ASCII lower-case letter, then "." unless it ends in ".", "!", "?"
      or "…"; words cut before either end in "…" instead. "forge could
      not release this task's resources; resolve this reason, then task
      release 6978 --hold 1; or task cancel 6978 REASON" says "Forge
      could not release this task's resources."; an empty first clause
      says as a hold with none;
    - otherwise its reason's phrase, the words `cards::hold_phrase`
      gives its `HoldCategory` and its number, made a sentence the same
      way: "An effect failed permanently.", "The executor stopped
      progressing.";
    - with no hold in its document, "This task is held."

    A funds card's question stays "S of L used.", with its Budget
    phrase under it, the words that say what a person can do (*By its
    kind*). Past that, neither card draws its hold: the account whole,
    a reason's phrase beside it, "Hold revision N", the acts the
    account names (`task release`, `task cancel`) and its
    specification's parameters ("Parameter 1: 2") are its task's
    page's, whose own held card draws the hold (`task.md`, 2, *Its
    parts, in order*; 3, *Its hold*). In the terminal's inbox the
    question comes after the title, before the words; Release and Leave
    held follow, as on the card here.
  - *An approval,* a change waiting on a review on the forge: a row of the
    subject `approvals` (`../forge/records.md`), its kind "an approval", its question, context,
    size, review link and "Open" as `../forge/README.md`, 4.1 has them, the review link
    "Review on GitHub ↗" or "Review on Forgejo ↗" first and primary, one tap
    to the forge's review page (`../../draft/forge-identity.md`, section 8), and
    in the terminal's inbox "Later" too, its key
    `ObjectKey::Task` of the change, so that Later works on it as on any card.
    The open cards merge the inbox's needs and these rows in one order, with
    those marked later last and the rest ordered by severity, oldest first by
    their times within each severity, as *What the client keeps* says
    (`needs-you-state.md`, 1, *The open cards*).
    A filter by project takes the project's rows, one by goal the rows of the
    changes under that goal, one by a milestone the rows whose goal is one of
    its goals, as its `Milestone` reads them (`milestone.md`, 1), and one by kind none, since no kind of the
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
    sentence; a held task's or another person task's title, cut past 80
    characters as TITLE is, a funds card's "S of L used"; an approval's
    question. A held or person task's SHORT names its task where no from
    line does, as Home's *More* line and a goal's needs bands say it;
    under a from line, a closed row and Home's card say a held card's
    why it's held instead, and a person task's nothing (*Its title
    once*).
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
      you" (`rules`), "no kind of decision names it" (`other`), "it starts
      work on findings" (`findings`, `../../draft/home-for-a-person.md`,
      5); for a reason
      `set`, "it changes NAME", NAME the set's name in the project's classes
      (`settings.md`, 2), or "one of PROJECT’s path sets" until they are read. With
      `under` an item with words, " under “WORDS”" follows the subject. A
      reason `always` reads "Reached you because its goal always asks you."
      So: "Reached you because it changes vsr’s protocol core, which asks
      you, even when away.", and a triager's group's: "Reached you because
      it starts work on findings, which asks you unless you’re away."
    - *A question with no row,* its project's document delivered and not
      `cut`: "Reached you because it was asked of you directly.": no call
      judged it, so it came by its route (kind 6,
      `../../protocol/configuration/settings-kinds.md`, 1) or without the tool. With no
      row and the document cut, refused, ended or not yet delivered, no
      line. A row whose reach says decided, which an option past the call's
      leaves, reads as none.
    - *A proposal,* "Reached you because a proposal always waits for whoever
      may accept it."; *held,* "Reached you because a held task always waits
      for whoever may release it."; *funds,* "Reached you because money past
      a budget always waits for you." An approval, whose rule its context
      names, and a person task not in words, the person's own work, have
      no line.
    - *A finding the person holds* (1, *A finding that is the person's*;
      `../../draft/home-for-a-person.md`, 3), by *Tell temper*'s rule
      (*Where it goes*) read for the finding: with an area, "Reached you as
      AREA’s owner."; else, where the project's settings name a triager
      that stands and *Goes to* would name it "you" (the party `me`
      names), "Reached you as PROJECT’s triager: findings that no area
      owns come to you."; else "Reached you as PROJECT’s owner: findings
      that no area owns come to you." PROJECT is the project's name as
      `me` names it. Until the project's `findings` document is read, no
      line. No Stop asking: no reach brought it. It is drawn on this
      page's finding card over its acts, and on Home's (`home.md`, 1,
      **Findings, drawn**, *A finding's own card*).
    - *The domain's reason,* `pages::needs::Why`, gains `Finding {
      project: u32, holds: needs::Holds }`, `Holds` one of `Area { name:
      Box<[u8]> }`, `Triager` and `Owner`, and is compared whole
      (`PartialEq`), Home's *One why* reading two cards' lines as the same
      where their `Why`s are equal (`home.md`, 1). A finding's is
      `pages::findings::why(root: &dyn Root, need: findings::Need<'_>) ->
      Option<needs::Why>`, which gives any other card's by
      `pages::needs::why`.
    - *In the plural,* the line Home says once for several cards
      (`home.md`, 1, *One why*): "These reached you" for "Reached you",
      and its clause said of them: "it is a wording choice", "they are
      wording choices"; "it retries or splits a failing task", "they retry
      or split failing tasks"; "it is an implementation choice", "they are
      implementation choices"; "it changes only tests", "they change only
      tests"; "it adds a dependency", "they add dependencies"; "it departs
      from a landed design", "they depart from a landed design"; "it spends
      money past a budget", "they spend money past a budget"; "it can’t be
      undone", "they can’t be undone"; "it changes what reaches you",
      "they change what reaches you"; "no kind of decision names it", "no
      kind of decision names them"; "it starts work on findings", "they
      start work on findings"; "it changes NAME", "they change NAME";
      " under “WORDS”" and CLASS as they are. Then "These reached you
      because their goals always ask you.", "These reached you because
      they were asked of you directly.", "These reached you because
      proposals always wait for whoever may accept them.", "These reached
      you because held tasks always wait for whoever may release them.",
      "These reached you because money past a budget always waits for
      you.", and a finding's "These reached you as AREA’s owner.", "These
      reached you as PROJECT’s triager: findings that no area owns come to
      you." and "These reached you as PROJECT’s owner: findings that no
      area owns come to you." One function of the view's shared words,
      beside the singular's (`temper_client_view::shared::why`), gives
      either form.
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
      "departures from a landed design", "starting work on findings",
      "changes to NAME", with `under`'s words after it as in CLAUSE; so
      Decided for you's "Allowed: starting work on findings." (`home.md`,
      1).
    - *Starting work on findings,* `Decide(Kind(Findings))`, names who
      asks it, as `../../draft/home-for-a-person.md`, 5 words it: its
      offer reads "Let the triager start goals for findings, then tell
      you.", and its narrower rule on this page "Stop asking: let the
      triager start goals for findings", each with `under`'s words after
      "findings" as in CLAUSE. Only the triager's charter names the kind
      (`../../domain/areas.md`, 6, *Its groups*), so the words name it
      whoever asked; the offer moves the project's item, as any
      `Decide`'s. A Stop asking proposal's reason (*Its act*) says it as
      any kind's, by OBJECT.
    - *Its act:* an owner of the card's project (`me`'s row for it `Owner`)
      has "Apply": jig's `ChangeSettings` of one item, kind 8 of that
      project (`../../protocol/configuration/settings-kinds.md`, 1), set to the offer's
      table, running work staying, against the newest revision the person
      has, as `settings.md`, 2's controls send it, a `stale` refusal sent once more
      against the revision it names. Anyone else has "Propose": jig's
      `Propose` in the deployment's scope, a change of settings of that one
      item against the newest revision, its reason "Stop asking in PROJECT:
      let agents decide OBJECT, then tell whoever they would have asked.",
      which a party with a role in the project may make and the owners
      decide (jig's `domain/people/acts.md`, 1, and `jig/docs/design/domain/engine/settings-proposals.md`, 1);
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
    (`../root.md`, 1.6), and closes the offers where they are open. Answered, the toast "It keeps asking.
    Stop asking comes back in 7 days."; refused or lost, "temper couldn't
    keep that. Try Keep asking again."; *unknown,* the subject `dismissed`
    answers it. While it is out, and once kept, the line offers neither
    Keep asking nor Stop asking for that subject in that project, on every card and device,
    until 7 days after its `at`; a later Keep asking of the same replaces
    it (`../../domain/forge/attention.md`, 1).
### 1.3 Tell temper

Moved to [`needs-you-tell.md`](needs-you-tell.md), section 1.

### 1.4 What the client keeps

Moved to [`needs-you-state.md`](needs-you-state.md), section 1.

### 1.5 Only you

- **Only you** (`../../draft/what-reaches-you.md`, section 8;
  `../../draft/surfaces.md`, section 4; the whole story's `needs.html`): what
  only the person can do and no choice answers, listed apart on *Needs
  you*: a sign-in of their own that is signed out, or a login their account
  reads as expiring. Work goes on around it: the runs that need the account
  wait for it (`accounts-words.md`, 1, *A run's wait*), and nothing else does.
  - *Its read,* `pages::only_you`, one domain function every shell may
    call, from the `accounts` document every page holds (jig's
    `client/domain/contract/documents.md`, 1, and `jig/docs/design/client/domain/contract/watches.md`, 1.1), read and never kept. An item
    is the deciding source, the first supplied, of an account the person
    holds (`AccountHolder::Party`, the party `me` names), but the
    playground provider's (`../forge/family.md`, 7), as the rail's mark reads it (`../view-navigation.md`, 1), whose
    kind is a sign-in or a program's login, and which is signed out
    (`SignInAgain`) or reads `expiring`, signed out where both hold: the
    source whose state holds the account's work (`accounts-words.md`, 1, *The composer's
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
      (`accounts-words.md`, 1, *An account*: "ChatGPT sign-in", "Claude Code login NAME"):
      signed out, "Your ChatGPT sign-in is signed out. Sign in again?";
      expiring, "Your Claude Code login claude-alice is expiring. Sign in
      again?";
    - its act alone, "Sign in again", a link in the look of the
      recommended `.choice`, primary, with no "recommended": to `Accounts`,
      whose row holds the sign-in (`accounts-words.md`, 1, *Each state*). On a phone it is full
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
- **The browser's updates** (1; that request): they leave the
  browser's page and no page replaces them, `Seen` marking staying as it is.
  Default: the terminal's list alone draws them.
- **"Add $10" on a funds card** (the greenfield mock's `needs.html`; that
  request puts budgets last): an act that widens the budget in place. Default:
  Release and Leave held, the hold's own acts.
- **A chat's own question, as a card** (the mock's `n-license`; that request):
  a chat's question in its words stays in its conversation, with no inbox
  entry. Default: as it is.
- **Across projects, in one project** (1, *Its intro* and an item's
  "PROJECT ›"; the look request `one-project`, which settles the switcher
  alone, `../view-navigation.md`, 1, *All projects*): on `/needs` with one project, the
  kicker reads "Needs you · all projects" and each item "PROJECT ›",
  though the switcher names the project. A look question for the user.
  Default: as they are.
- **A maker's own pending proposals, listed** (1, *By its kind*; the
  draft `../../draft/proposal-reach.md`, 2): a party's proposal waits in
  the inbox of those who may decide it, so a maker who decides none of
  its own finds no card for it, and its Withdraw is reached from ctl's
  `proposal withdraw` alone. Default: no listing of a party's own
  proposals; the card's Withdraw is where the card is drawn.
