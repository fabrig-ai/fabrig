# Team

## 1. The page

- **Team** (`../../draft/surfaces.md`, section 6; the whole story's
  `web/whole-story/team.html`, and `team/team.html` beside it for a
  project of three people): `Team { project }`, at
  `/projects/<project>/team`, a page of the project's frame. It draws a
  project's people and what waits on each, from what the engine holds
  (`../../draft/whole-story.execution.md`, section 2): a project's areas and
  their owners, a role's card and "Hand an area to a role" are not
  designed here, and nothing stands in their place.
  - *Its intro:* the kicker "PROJECT · Team"; the headline "Who owns what
    on PROJECT"; the lede, how many people the project has: "Just you.",
    "Two of you.", "Three of you.", "Four of you.", then "N of you." in
    digits, and "N or more of you." while the project's document is cut;
    where the person reading isn't one of them, as a deployment's owner
    may not be, "One person.", "Two people.", "Three people.", "Four
    people.", "N people." and "N or more people.".
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
    them is read; what a person signed while the changes' watch has
    answered and their count is above zero; the triager's line while a
    triager is read. No part is drawn empty, and each person is drawn once.
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
  - *In the rail* (`../view.md`, 3): "Team", after "Work", while the project has two
    people or more or a standing role, current on `Team`. Its count of
    people is the project's document's, read on every project page for
    the rail. On a phone, whose tab bar has no Team, the switcher's menu
    holds "Team for PROJECT" before "How work runs for PROJECT", while
    the rail shows Team (`../../draft/surfaces.md`, section 10).
  - *No act:* inviting a person, handing an area to a role and a role's
    acts are not Team's yet. The terminal prints its parts, as any page.

## 2. Open questions

- **Team after landing** (1; `../view.md`, 3): temper/ui compares main with
  the intro and People of the whole story's `web/whole-story/team/team.html`,
  and the rail's Team in each page's `nav.rail`, through `../web/gallery.md`, 2's
  `team`, `team-alone` and `night-team`, at 1440, 1000 and 390 px, by day
  and by night, with the switcher's menu open on a phone; and walks Team on
  a box with two people, each reading what waits on the other. Owner:
  temper/ui. Default: each difference a finding filed back as a repair.
- **Team's later parts** (1; `../../draft/surfaces.md`, sections 6
  and 12; `../../draft/whole-story.execution.md`, P4b and P4c): a role's card
  (R1); a project's areas and their owners, a person's areas on their box,
  "Hand an area to a role", "Talk to it" and the lede on how the project's
  triager hands what comes in to an area's owner (R2, R3); the triager as
  the draft's Who routes, a chip under the areas that reads what comes in
  against their words, changed by proposal, in place of *Team*'s line at
  People's foot, once the areas are drawn (R2); what a person can still
  sign and asked for, on their line, as
  `web/whole-story/team/team.html` draws it (N5); and what waits
  on another person on another's task or on a role they hold, once a read
  gives another party's inbox. Each joins *Its parts* with the read that
  gives it. Default: none is drawn until it is built, and a person's line
  is what *Team* counts.
- **A week's sign-offs past the rows** (1, *What each signed*): the
  count reads the changes' `landed_default` rows, at most
  `forge.changes_landed` (8 in `small`) across the project's
  repositories, from landing logs of `landings_kept` landings each
  (`../../domain/after-landing.md`, section 10), so a busy week's count
  is a floor, as Work's Landed this week is (`work.md`, 2). Default: so,
  nothing stored added; or the changes' document counts each person's
  sign-offs over every landing its logs keep, a wire field.
