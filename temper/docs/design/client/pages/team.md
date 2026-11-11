# Team

## 1. The page

- **Team** (`../../draft/surfaces.md`, section 6; the whole story's
  `web/whole-story/team.html`, and `team/team.html` beside it for a
  project of three people): `Team { project, role }`, at
  `/projects/<project>/team`, and `/projects/<project>/team?role=N` while
  `role` is `Some(N)`, a page of the project's frame. It draws a project's
  people and what waits on each, its standing roles and one role's card,
  from what the engine holds (`../../draft/whole-story.execution.md`,
  section 2): a project's areas and their owners, "Hand an area to a
  role", and a card's "owns AREA", "Talk to it", "Give the area back" and
  weekly report are not designed here, and nothing stands in their place.
  - *Its intro:* the kicker "PROJECT · Team"; the headline "Who owns what
    on PROJECT"; the lede, how many people the project has: "Just you.",
    "Two of you.", "Three of you.", "Four of you.", then "N of you." in
    digits, and "N or more of you." while the project's document is cut;
    where the person reading isn't one of them, as a deployment's owner
    may not be, "One person.", "Two people.", "Three people.", "Four
    people.", "N people." and "N or more people.".
  - *Standing roles,* a section headed "Standing roles", after the intro
    and before People, while the project has a live standing role
    (`../../domain/agent/standing.md`): a row for each, in the order of
    their numbers, at most 16, the roles' watch's window, and under them,
    small and muted, "Only the first 16 are shown." while the watch says
    there are more. A row holds the role's name, its task's title (the
    first line of its spec's words), in an agent's chip, a link to `Team
    { project, role: Some(N) }`, current while its card is the one shown;
    its Now's first line, while one is read; and at its right what it has
    spent of its period's budget, "21 of 40", its budget's `spent` and
    `budget` as they come, unscaled, as a chat's bar gives them. This
    section stands where `../../draft/surfaces.md`, section 6's areas
    will, and a role is reached from it until they are drawn (section 2).
  - *A role's card* (`../../draft/surfaces.md`, section 6; `team.html`'s
    card), under Standing roles, for one role: the one `role` names while
    it is listed, else the first listed. Its parts, in this order, each
    drawn while it holds something:
    - *its head:* its name, a link to its task's page, `Task { task }`
      (`task.md`); under it, small and muted, "A standing role · since 2
      November · started by you": its task's `made` as a day, and its
      requester, "you" for the person reading, else as People names the
      member, else as the agent task page names a requester; while it is
      held, a line with the agent task page's words for its hold
      (`task.md`, 2). At the head's right, "Pause" while it isn't held,
      and "Resume" while it is paused alone, sent as a goal's page sends
      them (`task.md`, 4): Pause jig's `Stop` of the role at once, its
      toast "Paused: running tasks stop, nothing new starts."; Resume
      `Release` naming the hold's revision, its toast "Resumed.". A role
      held for any other cause has no Resume: Needs you has its need.
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

    On a wide page the card has two columns, its Now and its rules on the
    left, what wakes it and its sessions on the right, as `team.html`'s
    card; on a phone one, in that order.
  - *What wakes it,* from the role's task document's `subscriptions` and
    `wake` (jig's `client/domain/contract/README.md`, section 4; jig's
    `domain/tasks/messages.md`, 3 and 4), a row each, in the document's
    order:
    - *a timer with a period:* "A timer", and its period: a week, the
      weekday and clock of its `at`, "Mondays 06:00"; a day, "daily
      06:00"; any other, "every" and the period, "every 4 hours";
    - *a timer without one:* "A timer", and "once, at" and its `at` as the
      view says a time;
    - *another task's state:* "Task N held", "Task N's result", or "Task N
      held, or its result", by its `held` and `result`, N a link to the
      task's page; its class the wake policy's rule for the kind of
      message jig delivers it as;
    - *a connector's topics:* one row for all the forge's, at the place of
      the first, "News from the forge", or "News from the forge, on N
      topics"; one for each other connector's, "News from connector N";
      its class the policy's `news` rule. What a topic is, a landing on a
      branch, a check or a pull, temper's forge keeps and no client read
      gives (section 2).

    A class says the policy's rule: immediate, "at once"; paced, "every"
    and its span, "every 30 minutes"; batched, "N at a time, or after" and
    its age; never, "with its next wake". A span, a period or an age is
    said in its largest whole unit of minutes, hours, days and weeks:
    "every 90 minutes", "every 4 hours", and "every hour" or "every day"
    for one.
  - *Its sessions,* from the role's activity (jig's
    `domain/tasks/standing.md`, sections 5 and 7): a session is named by
    the attempt that began it; it begins at that attempt's `RunBegan`,
    which resumes nothing, and ends at the `SessionEnded` naming it, or
    is open while none has come. A row: its start's weekday and clock and
    its length, "Sun 01:52 · 49 min", or "Sun 01:52 · open"; what it did,
    its runs' last `Stood` words on one line, while any; and at its right
    its cost, the sum of its runs' `RunEnded` `spent`, as it comes. The
    read stops at a line older than the seven days, at the eighth
    session's start, or after 16 pages.
  - *People,* a section headed "People": a box for each of the project's
    members of kind person (jig's `domain/people/README.md`, section 4), the
    person reading first, then the others in the document's order, each
    once. A service is no one's teammate: Deployment's Members lists it,
    Team doesn't. A box holds the person's name, "You" for the person
    reading, else the member's name, or its login where the name is
    empty; under it, muted, their role in the project: "Owner",
    "Maintainer", "Member", "Observer", or "Role N" for a role the
    project's policy adds; then their line.
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
  - *The project's triager* (`../../domain/findings.md`, section 6), the
    owner of findings with no area, is a line at People's foot,
    "Findings with no area go to NAME.", once the project's settings
    name one. None does yet (`../../domain/landing.md`, 13.1, kind 4's
    `findings`), so no page draws it.
  - *Its parts* (`../../draft/surfaces.md`, section 12): one pure function,
    `pages::team::parts(&team::Facts) -> team::Parts`, which both shells
    draw from, over plain facts that `team::gather` makes from the reads:
    People while a person is listed; People's head's shared role while
    *People's head* says, as `Parts::shared`, an `Option<HeldRole>` that
    is never `Policy`; a person's line while what waits on
    them is read; the triager's line while a triager is read; Standing
    roles while a role is listed, and "Only the first 16 are shown." by
    the watch's `more`; a row's Now while it is read; the card while a
    role is listed, for the role *A role's card* names; its head with it,
    and Pause or Resume by its hold; each other part of the card while
    what it draws is read and holds something, and "Not all are shown."
    by the document's cut. No part is drawn empty, and each person and
    each role is drawn once.
  - *Its reads:* `me`, for the party and the project's name; jig's
    `Read::Project { project }` on opening, and again on each `me`
    delivery while none is pending, since a change of members moves `me`
    (jig's `client/domain/contract/README.md`, 5.1); the standing inbox watch
    and the person's approvals, as Needs you's; one `tasks{query}` watch
    for each other person read, while the page is open. Before the
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
      card and again when its line's last activity moves. A refused or
      failed read leaves its part undrawn.
  - *In the rail* (`../view.md`, 3): "Team", after "Work", while the project has two
    people or more or a standing role, current on `Team` whatever its
    `role`. Its count of people is the project's document's, and its
    count of roles the roles' watch's, both read on every project page for
    the rail. On a phone, whose tab bar has no Team, the switcher's menu
    holds "Team for PROJECT" before "How work runs for PROJECT", while
    the rail shows Team (`../../draft/surfaces.md`, section 10).
  - *Its acts:* a role's card's Pause and Resume (above). Inviting a
    person, handing an area to a role and a role's other acts are not
    Team's yet. The terminal prints its parts, as any page.

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
  then paused and resumed there. Owner: temper/ui. Default: each
  difference a finding filed back as a repair.
- **Team's later parts** (1; `../../draft/surfaces.md`, sections 6
  and 12; `../../draft/whole-story.execution.md`, P4c): a project's areas and their owners, a person's areas on their box,
  "Hand an area to a role", a card's "owns AREA", "Talk to it", "Give the
  area back" and weekly report, and the lede on how the project's
  triager hands what comes in to an area's owner (R2, R3), Standing
  roles folding into the areas' table, a role reached from its area's
  row; the triager as
  the draft's Who routes, a chip under the areas that reads what comes in
  against their words, changed by proposal, in place of *Team*'s line at
  People's foot, once the areas are drawn (R2); what a person can still
  sign, signed and asked for, on their line, as
  `web/whole-story/team/team.html` draws it (N2, N5, N6); and what waits
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
  report", which a timer doesn't say; what a session did, once a record
  keeps a line for it, where a run's finished words are the only one; and
  "written at 06:58, when its last session ended", once the client reads
  a session's record (jig's `domain/tasks/standing.md`, 7). Default: each
  as section 1 has it, the gap recorded in
  `../../draft/surfaces.status.md`, 6.
