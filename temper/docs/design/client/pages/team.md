# Team

## 1. The page

- **Team** (`../../draft/surfaces.md`, section 6; the whole story's
  `web/whole-story/team.html`, and `team/team.html` beside it for a
  project of three people): `Team { project, role }`, at
  `/projects/<project>/team`, and `/projects/<project>/team?role=N` while
  `role` is `Some(N)`, a page of the project's frame. It draws a project's
  areas and their owners (`../../domain/areas.md`), who routes what comes
  in, its people and what waits on each, its standing roles and one role's
  card, from what the engine holds (`../../draft/whole-story.execution.md`,
  section 2), and hands an area to a role or gives it back.
  - *Its intro:* the kicker "PROJECT · Team"; the headline "Who owns what
    on PROJECT"; the lede, how many people the project has: "Just you.",
    "Two of you.", "Three of you.", "Four of you.", then "N of you." in
    digits, and "N or more of you." while the project's document is cut;
    where the person reading isn't one of them, as a deployment's owner
    may not be, "One person.", "Two people.", "Three people.", "Four
    people.", "N people." and "N or more people.". While the areas'
    table is drawn (*Areas*, below), the lede goes on, as `team.html`'s
    does: "Each area has one owner. WHO hands each finding and red to its
    area's owner. An owner that can't decide asks you, as How work runs
    says.", WHO the holder of what has no area as *An owner's name*
    says it, its verb by it: "You hand", "Prover hands", "Sam hands",
    "The project's owners hand".
  - *Areas,* a section headed "Areas", after the intro, while the
    project's `findings` document has answered (*Its reads*): at its
    head's right, "+ Hand an area to a role" (*Hand an area to a role*,
    below), while the person reading holds a role in the project. While
    the project has an area or a triager set, a table, as `team.html`'s:
    the columns "Area", "What it covers", "Owner", "Their Now" and "This
    week"; a row for each of the document's `areas`, in their order; then
    a last row, "Anything else", "What fits none of these", whose owner
    is the document's `holder`, the holder of what has no area. With
    neither, in the table's place, small and muted: "No areas yet: the
    project's owners take every finding and red."
    - *A row:* the area's name as written, in the strong weight; its
      words as written; its owner, the row's holder as a chip in who-acts
      colours (`../../draft/surfaces.md`, 13), an agent's star for a task,
      a person's ring for a party or a role of people, its words *An
      owner's name*; while the row's holder is a role listed under the
      roles' watch, the chip links to `Team { project, role: Some(N) }`,
      and that role's first row is current while its card is shown. An area
      whose owner does not stand (`../../domain/areas.md`, section 3), its
      `holder` none, has in the chip's place, muted, the owner as written:
      "Task 61, ended", "Party 7, not of PROJECT", "Charter 15's role, not
      started", and nothing in its last two columns.
    - *Their Now:* for a listed role, its Now's first line, while one is
      read; for a party, their line, as their box says it (*A person's
      line*), while it is read; for any other holder, "–", muted.
    - *This week:* for a listed role, on its first row, what it has spent
      of its period's budget, as its row under Standing roles gives it,
      "21 of 40"; on each later row of the same role, muted, "in NAME's",
      NAME its title; for any other holder, "–", muted.
  - *Who routes,* under the table, while the table is drawn: small, the
    holder of what has no area as a chip, then "reads what comes in
    against these words and its rules, and hands each to its owner." for
    a task, and for people "reads what comes in against these words, and
    hands each to its owner.", in the second person for the reader, "You
    read … and hand …", and the plural for a role, "The project's owners
    read … and hand …"; then "Change by proposal", a link to Ask temper's
    page of the project, `Ask { project }`, as How work runs' "change by
    proposal" is (`settings.md`, 2): the triager is kind 15, set by a way
    or by proposal (`../../domain/areas.md`, section 6).
  - *An owner's name,* in a chip or a sentence: a task listed under the
    roles' watch, its title; another task, "Task N"; a party, "You" for
    the person reading, else as People names the member, else "Party N";
    a role of people, its plural, "The project's owners", "The project's
    maintainers", "The project's members", "The project's observers", or
    "Role N's people" for one the project's policy adds.
  - *Standing roles,* a section headed "Standing roles", after Areas
    and before People, while the project has a live standing role
    (`../../domain/agent/standing.md`) that holds no row of the areas'
    table, or the roles' watch says there are more than it lists; while
    the areas aren't read, every listed role holds none. A row for each
    such role, in the order of their numbers, at most 16, the roles'
    watch's window, and under them, small and muted, "Only the first 16
    are shown." while the watch says there are more. A row holds the
    role's name, its task's title (the first line of its spec's words),
    in an agent's chip, a link to `Team { project, role: Some(N) }`,
    current while its card is the one shown;
    its Now's first line, while one is read; and at its right what it has
    spent of its period's budget, "21 of 40", its budget's `spent` and
    `budget` as they come, unscaled, as a chat's bar gives them. A role
    holding a row of the areas' table is reached from its chip there, as
    `team.html`'s Prover is.
  - *A role's card* (`../../draft/surfaces.md`, section 6; `team.html`'s
    card), under Standing roles, or under Areas where Standing roles
    isn't drawn, for one role: the one `role` names while it is listed,
    else the first listed. Its parts, in this order, each drawn while it
    holds something:
    - *its head:* its name, a link to its task's page, `Task { task }`
      (`task.md`); under it, small and muted, "A standing role · since 2
      November · started by you": its task's `made` as a day, and its
      requester, "you" for the person reading, else as People names the
      member, else as the agent task page names a requester. While it
      holds rows of the areas' table, "since" follows what it owns: "A
      standing role · owns tests, and Anything else, since 2 November ·
      started by you", the names of the areas it holds in the table's
      order, then "Anything else" where it holds the last row, joined
      with ", " and ", and " before the last; the day is still its
      task's `made`, since nothing keeps when an area changed owner.
      While it is held, a line with the agent task page's words for its
      hold (`task.md`, 2). At the head's right, in this order: "Talk to
      it", the dark button, a link to its task's page, `Task { task }`,
      where its composer talks (`task.md`, 4, *A standing role's talk*),
      while its task's document offers `Talk`; "Pause" while it isn't
      held, and "Resume" while it is paused alone, sent as a goal's page
      sends them (`task.md`, 4): Pause jig's `Stop` of the role at once,
      its toast "Paused: running tasks stop, nothing new starts."; Resume
      `Release` naming the hold's revision, its toast "Resumed.". A role
      held for any other cause has no Resume: Needs you has its need.
      Then "Give the area back", or "Give the areas back" while it holds
      two areas or more, while it holds an area of the table but its last
      row and the person reading holds a role in the project (*Give the
      area back*, below).
    - *its Now:* the kicker "Its Now · written at 06:58", the note's
      `changed` as the view says a time, then its body as written: a
      line opening "- " a list item, the other lines paragraphs, nothing
      composed; a cut body ends "…". A role that has written none has no
      Now part.
    - *its rules:* the kicker "Its rules · change by proposal", "change by
      proposal" a link to its charter's page, `Role { project, charter }`
      (`settings.md`, 1, *A role's or a rule's page*); then its charter's
      "Asked to", as the `charter N` view gives it.
    - *what wakes it:* the kicker "Wakes on", then a row for each of its
      subscriptions, *What wakes it* below, its words at the left and its
      class at the right in code's chip; under them, small and muted,
      "Not all are shown." while the task's document is cut.
    - *its sessions this week:* the kicker "Sessions this week", then a
      row for each session begun in the seven days before now, newest
      first, at most eight (*Its sessions*, below); then, small and muted,
      "Each session starts fresh from its rules, its Now and what woke it,
      and ends by writing its Now." That line is drawn with the rows, never
      alone.
    - *its weekly report* (`../../domain/agent/standing.md`, 8): the
      kicker "Its weekly report · written Mon 06:00", the note's `changed`
      as a short weekday and a clock, as a session's row says its start,
      then its body drawn as the Now's is, nothing composed, a cut body
      ending "…". A role that has written none has no report part.

    On a wide page the card has two columns, its Now and its rules on the
    left, what wakes it, its sessions and its weekly report on the right,
    as `team.html`'s card; on a phone one, in that order.
  - *What wakes it,* from the role's task document's `subscriptions` and
    `wake` (jig's `client/domain/contract/README.md`, section 4; jig's
    `domain/tasks/messages.md`, 3 and 4), a row each, in the document's
    order:
    - *a timer with a period:* "A timer", and in its class's place its
      period: a week, the weekday and clock of its `at` where the view
      keeps time, "Mondays 06:00"; a day, "daily 06:00"; any other,
      "every" and the period, "every 4 hours"; while the policy's
      `timers` is unset, its words are "A timer, with its next wake";
    - *a timer without one:* as a timer with a period, with "once, at"
      and its `at` as the view says a clock time, "once, at 06:00" today,
      else with its date, in its period's place;
    - *another task's state:* "Task N held", "Task N's result", or "Task N
      held, or its result", by its `held` and `result`, N a link to the
      task's page; jig delivers either as a notice (jig's
      `domain/tasks/messages.md`, 1), so its class is the policy's
      `notices` rule;
    - *a connector's topics:* one row for all the forge's, connector
      `temper_client_protocol_wire::FORGE`, at the place of the first,
      "News from the forge", or "News from the forge, on N topics"; one
      for each other connector's, at the place of its first, "News from
      connector N"; its class the policy's `news` rule. What a topic is, a
      landing on a branch, a check or a pull, temper's forge keeps and no
      client read gives (section 2).

    A class says the policy's rule: immediate, "at once"; paced, "every"
    and its span, "every 30 minutes"; batched, "N at a time, or after" and
    its age, "5 at a time, or after 2 hours"; never, "with its next wake".
    A span, a period or an age is said in the largest of minutes, hours,
    days and weeks that it is a whole number of: "every 90 minutes",
    "every 4 hours", and "every hour" or "every day" for one; one that is
    no whole number of minutes in seconds, "every 45 seconds".
  - *Its sessions,* from the role's activity (jig's
    `domain/tasks/standing.md`, sections 5 and 7): a session holds the
    runs from the first `RunBegan` after a `SessionEnded`, or after none,
    up to the next `SessionEnded`, which jig commits at the claim that
    begins the next session where the session ended on a wake (`Quiet`,
    `Bound`, `Unoffered`, `Fresh`), so it can come hours after its last
    run, and at once where it was closed or its rules changed (`Closed`,
    `Rules`); the session is named by its first run's attempt. A row: its
    first run's start, as a short weekday and a clock, whatever the day,
    and its length, from that start to its last run's `RunEnded`, as the
    view words a span, "Sun 01:52 · 49 min"; while one of its runs is
    live, its `RunBegan` read and no `RunEnded` of its attempt after it,
    "Sun 01:52 · running"; what it did, the line of its last
    `NowWritten` (`../../domain/agent/standing.md`, 8), else its runs'
    last `Stood` words, on one line, their line breaks as spaces, while
    any; and at its right its
    cost, the sum of its ended runs' `RunEnded` `spent`, as it comes. The
    read runs from the newest line back, so every line after one read is
    read. A session is drawn once what comes before its first run is read:
    the `SessionEnded` before it, or the start of the role's activity, a
    page with no `before`; the session the read stops inside is not. The read
    stops at a `RunBegan` or a `SessionEnded` older than the seven days,
    once eight sessions are known, or after 16 pages.
  - *People,* a section headed "People": a box for each of the project's
    members of kind person (jig's `domain/people/README.md`, section 4), the
    person reading first, then the others in the document's order, each
    once. A service is no one's teammate: Deployment's Members lists it,
    Team doesn't. A box holds the person's name, "You" for the person
    reading, else the member's name, or its login where the name is
    empty; under it, muted, their role in the project: "Owner",
    "Maintainer", "Member", "Observer", or "Role N" for a role the
    project's policy adds, then, while they hold rows of the areas' table
    as a party, " · " and those rows' names in the table's order, joined
    with ", ", "Anything else" for its last row, as `team/team.html`'s
    "Owner · Payments"; then their line.
  - *People's head* holds, at its right and muted, the role the people
    share, while two people or more are listed, the project's document
    isn't cut, and every one listed holds the same role of the four
    named: "owners of PROJECT", "maintainers of PROJECT", "members of
    PROJECT" or "observers of PROJECT". Where their roles differ, where
    the one they share is a role the project's policy adds, where one
    person is listed or where the document is cut, the head holds its
    title alone, and each box says its own role. A count of them is the
    lede's, never the head's.
  - *A person's line* is what waits on them in the project, never a Now,
    which a person doesn't write: "Nothing waits on you.", "Two questions
    wait on you.", "One review waits on Sam.", and where kinds mix their
    sum, "Three things wait on you."; the count is a word from one to
    nine, as the line's first word, then digits, "12 questions wait on
    you."; a count from a cut list is a floor, in digits, as the lede's
    is, "2+ questions wait on Sam.". Its kinds are Needs you's: questions;
    reviews, a change waiting for the person's review on the forge;
    proposals; held tasks, an escalation that reached them; and person
    tasks.
    - *For the person reading:* Needs you's open entries in the project,
      as the rail counts them (`../view.md`, 3): the standing inbox's rows that show
      in the project, less those answered here, each by its group, and
      the person's approvals' rows of the project, as reviews.
    - *For another:* what the engine says waits on them on their own
      work: a `tasks{query}` watch of the project's live tasks they
      requested (jig's `client/domain/contract/README.md`, 4.1), each line whose
      `waits_on` is `Party` and names them counted once, by the kind of
      its oldest entry. A task with two entries waiting on them counts
      once, and what waits on them on another's task, or on a role they
      hold, isn't counted: no read gives another party's inbox (jig's
      `domain/people/README.md`, section 5). At most eight others are read, the
      first in People's order; a box past them has no line.
  - *What each signed* (`../../domain/forge/change/gates.md`, 2,
    *Sign-offs kept at landing*), as `web/whole-story/team/team.html`
    draws it: after a person's line, or alone where they have none,
    "Signed four last week.", the changes landed in the seven days
    before now, now included, whose sign-offs name the person, each
    change once: the subject `changes`' `landed_default` rows
    (`../forge/records.md`, *A project's changes*) whose `at` is in that
    window and whose `signed` holds the person's party. The count is a
    word from one to nine, then digits, "Signed 12 last week."; where
    `landed_default_more` is above zero and the oldest row listed is in
    the window, rows the document cut may hold more, so the count is a
    floor, in digits, "Signed 4+ last week.". At zero, nothing is said.
    Every person listed has it, those past the eight others read too.
  - *The project's triager* (`../../domain/areas.md`, section 6), the
    owner of what has no area, is the areas' last row and Who routes'
    chip; People has no line of it.
  - *Hand an area to a role,* the sheet "+ Hand an area to a role" opens
    (`../README.md`, 3.6, *The sheet*), headed "Hand an area to a role",
    its acts at its foot:
    - *Which area:* a choice of the project's areas, a line each, its
      name in the strong weight, its words, and muted its owner's name,
      the first chosen; then, while the project has fewer than 16, "A new
      area", which shows two fields, "Name" and "What it covers", judged
      as kind 14 judges them (`../../protocol/configuration/settings.md`,
      2): "A name is 1 to 32 lowercase letters, digits and -, starting
      with a letter.", "PROJECT has an area named NAME already." and "What
      it covers is 1 to 480 bytes, on one line.", each under its field
      once it is left or the act pressed.
    - *To which role:* a choice, a line each: each role the roles' watch
      lists, by its title, the first chosen; then "Prover, from the
      stock", "Owns its area's tests: triages their reds and flakes and
      makes the repairs.", and "Porter, from the stock", "Keeps its
      area's port in step with its source.", the stock's charters 15 and
      16 and their words (`../../protocol/configuration/stock.md`), each
      starting a new standing role, which shows the field "Its budget a
      week", whole units of the deployment's, above zero; then "Write
      one with Ask temper".
    - *Its acts:* "Not now", which closes it; and, for a role listed or
      from the stock, "Hand it over" for an owner of the project, else
      "Propose"; for "Write one with Ask temper", "Ask temper", which
      closes the sheet and opens Ask temper's page of the project with its
      words unsent in the composer, `Act::AskWith { words }` as Home's
      sends it (`home.md`, 1): "Write a
      standing role to own NAME on PROJECT: WORDS", or "… to own a new
      area on PROJECT." where the area is new and unnamed.
    - *What it sends:* a change of kind 14, the project's areas
      (`../../domain/areas.md`, section 2), whole: the record read at its
      newest revision, with the chosen area's owner changed, or the new
      area added last; the owner `task N` for a listed role and `charter
      15` or `charter 16` for one from the stock, standing once that role
      runs (`../../domain/areas.md`, section 3). An owner of the project
      sends it as `Ask::ChangeSettings` against that revision, running
      work staying, its toast "Handed to TITLE, as revision N."; anyone
      else as a proposal of settings, its reason "Hand NAME to TITLE",
      its toast "Proposed: it applies once an owner accepts it.", as Needs
      you's Stop asking sends either (`needs-you.md`, 1). A `stale`
      refusal is sent once more against the revision it names, as How
      work runs' controls are (`settings.md`, 2, *Its acts*). For a role
      from the stock, once the change is saved or proposed, the role is
      started from the stock's template 7, its charter 15 or 16, its words
      the role's name on a line, then "Owns the area NAME on PROJECT.",
      its budget the one given, created or proposed as the person's role
      judges it, as a way's act starts the roles it brings
      (`../../protocol/configuration/settings.md`, 8): "Prover\n\nOwns the
      area tests on vsr.". A refusal of either says why in the sheet, in
      jig's words, and keeps it open.
  - *Give the area back* opens the confirmation (`../README.md`, 3.6,
    *The sheet*): "Give tests back?", or "Give 2 areas back?", then
    "Its owner becomes you.", or "Their owner becomes Sam.", the role's
    person, its requester, as *An owner's name* says them, then "Keep it"
    or "Keep them", and "Give it back" or "Give them back" for an owner of
    the project, else "Propose". It sends a change of kind 14, whole, as
    *Hand an area to a role* does, each area the role holds as an owner
    of the record, `task N` or a `charter` standing as it, given the
    owner `party P`, P its requester: an owner's toast "Given back, as
    revision N.", anyone else's "Proposed: it applies once an owner
    accepts it.". The triager, the last row, is changed by proposal
    through Ask temper (*Who routes*), never given back here.
  - *Its parts* (`../../draft/surfaces.md`, section 12): one pure function,
    `pages::team::parts(&team::Facts) -> team::Parts`, which both shells
    draw from, over plain facts that `team::gather` makes from the reads:
    People while a person is listed; People's head's shared role while
    *People's head* says, as `Parts::shared`, an `Option<HeldRole>` that
    is never `Policy`; a person's line while what waits on
    them is read; what a person signed while the changes' watch has
    answered and their count is above zero; Areas while the `findings`
    document is read, its table and Who routes while the project has an
    area or a triager set, else the line in its place, and "+ Hand an
    area to a role" while the person reading holds a role in the project;
    a box's areas while they hold a row as a party; the lede's areas
    sentences with the table; Standing
    roles while a listed role holds no row of the table, or the watch's
    `more`, and "Only the first 16 are shown." by the watch's `more`; a
    row's Now while it is read; the card while a
    role is listed, for the role *A role's card* names; its head with it,
    "owns" by the rows it holds, "Talk to it" by its document's `Talk`
    offer, Pause or Resume by its hold, and "Give the area back" by the
    areas it holds; each other part of the card while
    what it draws is read and holds something, and "Not all are shown."
    by the document's cut. No part is drawn empty, each person and each
    role is drawn once in People and Standing roles, each area once in
    the table, and a role holding a row of the table is never a row of
    Standing roles.
  - *Its reads:* `me`, for the party and the project's name; jig's
    `Read::Project { project }` on opening, and again on each `me`
    delivery while none is pending, since a change of members moves `me`
    (jig's `client/domain/contract/README.md`, 5.1); the standing inbox watch
    and the person's approvals, as Needs you's; one `tasks{query}` watch
    for each other person read, and the subject `changes`, as Work
    watches it (`work.md`, 1, *Its reads*), while the page is open. Before the
    document answers, the intro, and "People" with no line under it; a
    refused or failed read says, under "People", "This project's people
    can't be read now."
    - *The roles' watch,* on every project page, for the rail and Team:
      one `tasks{query}` watch of the project's live tasks labelled 3
      (`../../domain/agent/standing.md`, section 2), by number, its
      window 16. Refused or failed, it counts no role and Standing roles
      isn't drawn.
    - *Each listed role's Now,* on Team: jig's `Read::Notes { scope:
      NoteScope::Task { project, task }, description: Some("now"), after:
      None }`, the lowest-numbered note whose description is `now`
      (jig's `domain/tasks/standing.md`, section 6), read when its row
      first comes and again when its line's last activity moves, since
      notes have no watch, as a goal's page reads its report (`goal.md`,
      1). None yet is a normal state, never an error.
    - *For the card:* a `Subject::Task { task }` watch of the role, for
      its head, its hold and what wakes it; the forge's `View` of
      `charter N` for its charter, as `Role`'s page reads it, again when
      the session's `settings` watch names a newer revision; and its
      activity, jig's `Read::Activity` pages, newest first, on opening the
      card and again when its line's last activity moves, reading back
      from the newest line until it reaches a line already read, by its
      `position`, and keeping what it read before; and its weekly report,
      jig's `Read::Notes` of its scope with `description: Some("report")`,
      the lowest-numbered note whose description is `report`
      (`../../domain/agent/standing.md`, 8), read as its Now is. A refused
      or failed read leaves its part undrawn.
    - *The areas,* on Team: the forge's subject `findings`
      (`../forge/records.md`), `FindingsSubject { project, state:
      Some(New), mine: false }`, while the page is open, for its `areas`,
      each with its owner and holder, its `triager` and its `holder`, the
      holder of what has no area; its rows are not drawn. The root marks
      it at each save of kind 14 or 15, so a hand-off shows when it is
      saved. Refused or failed, Areas isn't drawn and every listed role
      is a row of Standing roles.
    - *For the acts:* kind 14's setting, read at its newest revision when
      *Hand an area to a role* or *Give the area back* sends, its value
      decoded as `temper_client_protocol_wire::v1::ProjectAreas`; the
      roles' watch for the sheet's roles; and `me` for whether the person
      reading owns the project.
  - *In the rail* (`../view.md`, 3): "Team", after "Work", while the project has two
    people or more or a standing role, current on `Team` whatever its
    `role`. Its count of people is the project's document's, and its
    count of roles the roles' watch's, both read on every project page for
    the rail. On a phone, whose tab bar has no Team, the switcher's menu
    holds "Team for PROJECT" before "How work runs for PROJECT", while
    the rail shows Team (`../../draft/surfaces.md`, section 10).
  - *Its acts:* "+ Hand an area to a role" and its sheet; a role's card's
    "Talk to it", a link, Pause and Resume, and "Give the area back"
    (above). Inviting a person and a role's other acts are not Team's.
    The terminal prints its parts, as any page.

## 2. Open questions

- **Team after landing** (1; `../view.md`, 3): temper/ui compares main with
  the intro and People of the whole story's `web/whole-story/team/team.html`,
  and the rail's Team in each page's `nav.rail`, through `../web/gallery.md`, 2's
  `team`, `team-alone` and `night-team`, at 1440, 1000 and 390 px, by day
  and by night, with the switcher's menu open on a phone; and walks Team on
  a box with two people, each reading what waits on the other. The same
  for a role's card against `web/whole-story/team.html`'s, through
  `team-role` and `night-team-role`, and on a box a role started with
  `temper ctl role start`, its Now read on Team after its first session,
  then paused and resumed there. The same for the areas, Who routes and a
  card's "owns" against `team.html`, through `team-role`; and on a box an
  area handed to that role from Team's sheet, a red reaching it, and
  "Talk to it" opening its page, a talk there that adds a rule, recorded
  as the journey "Team to a role, then Talk" (`../web/journeys.md`).
  Owner: temper/ui. Default: each difference a finding filed back as a
  repair.
- **Team's later parts** (1; `../../draft/surfaces.md`, sections 6
  and 12): what a person can still
  sign and asked for, on their line, as
  `web/whole-story/team/team.html` draws it, once a read of what another
  person can still sign gives it, the `approvals` rows naming who can
  still sign only the changes their watcher sees
  (`../../domain/forge/change/questions.md`, 2, *Who can still sign*); and what waits
  on another person on another's task or on a role they hold, once a read
  gives another party's inbox. Each joins *Its parts* with the read that
  gives it. Default: none is drawn until it is built, and a person's line
  is what *Team* counts.
- **A role's card, where a read lacks** (1, *A role's card*): its rules as
  the words kept with its charter (`../../protocol/configuration/settings.md`,
  2, kind 9), once the `charter N` view carries them, where it has the
  "Asked to" fact, its instructions' first sentence; a forge topic in
  words, "A landing on vsr's main touching tests/", once a read gives
  temper's subscriptions of a task; what a timer is for, "Its weekly
  report", which a timer doesn't say; and
  "written at 06:58, when its last session ended", once the client reads
  a session's record (jig's `domain/tasks/standing.md`, 7). Default: each
  as section 1 has it, the gap recorded in
  `../../draft/surfaces.status.md`, 6.
- **A week's sign-offs past the rows** (1, *What each signed*): the
  count reads the changes' `landed_default` rows, at most
  `forge.changes_landed` (8 in `small`) across the project's
  repositories, from landing logs of `landings_kept` landings each
  (`../../domain/after-landing.md`, section 10), so a busy week's count
  is a floor, as Work's Landed this week is (`work.md`, 2). Default: so,
  nothing stored added; or the changes' document counts each person's
  sign-offs over every landing its logs keep, a wire field.
- **When a role took its area** (1, *A role's card*): "owns tests since
  DATE" says the role's start, since no record keeps when kind 14 named
  it. Default: so; or the forge keeps each area's owner's revision and
  its time with kind 14's value, a stored field.
- **`team/team.html`'s other parts** (1, *Areas*): its "Open" column, a
  count of each area's open findings, and its next step's card, "Hand an
  area to a role" with suggested areas, which `../../draft/surfaces.md`,
  section 6 doesn't name. Default: neither drawn; the count would come
  from the `findings` rows Team already watches, `new` alone today.
- **A role written with Ask temper, started from the web** (1, *Hand an
  area to a role*): Ask temper proposes a charter, and a person starts a
  role under it with `temper ctl role start --charter N`, since the sheet
  starts the stock's prover and porter alone. Default: so; or the sheet
  lists the deployment's charters that a standing role runs.
- **A stock role started twice** (1, *Hand an area to a role*): the
  sheet's "Prover, from the stock" starts a new role even where one
  under charter 15 runs, since the roles' watch doesn't say a role's
  charter; `charter 15` then stands as the lowest-numbered one. Default:
  so, the sheet listing the running role by its title first; or the
  roles' rows carry their charter, a wire field of jig's.
