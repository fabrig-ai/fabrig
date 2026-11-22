# Home on All projects

This document holds home on All projects, split from [`home.md`](home.md).

## 1. Home on All projects

- **Home on All projects** (`../../draft/surfaces.md`, section 3; the whole
  story's `home.html` and `phone-home.html`, `web/whole-story/`):
  `AllHome`, at `/home`, Home for every project the person is in, as
  `me` lists them and in its order, the first 16 read (`all_home::ACROSS`)
  and the rest left out. The root, `/`, opens it for a person in two
  projects or more or in none, and a project's Home for one in one
  (`../web/addresses.md`, 1). It draws in the open project's frame, as `Inbox`
  does: the rail and the tab bar are the open project's, or with none open
  `me`'s first project's, "Home" current (`../view-navigation.md`, 1). Going to it opens no
  project and moves none. With no project in `me` it draws its intro
  alone. It has no To begin, no Chats, no Goals and no foot: those are a
  project's Home's. One column of prose, in this order, each part drawn
  by its rule (`home.md`, 1, *A page's parts*):
  - *Its reads.* On entering, for each project read, the subject
    `status` (`../forge/records.md`) and `Looked` once, so that each project's `previous`
    and `seen` move as its Home's entry moves them; the inbox unfiltered
    and the subject `approvals`, as *Needs you* reads them across projects.
    Its `decided` documents and showing tasks use the shared *Decided for
    you* store (`home.md`, 1), with the same four-project and seven-task bounds.
    Leaving unwatches them all. A project's status counts once its look is
    answered and its document delivered, as on its Home; a watch refused or
    ended leaves that project's status unknown, and the rest stand.
    While Home on All projects is the open page, and not for the
    launchpad's reads (`boundary::Event::Launchpad`) or Needs you's, it
    also watches the forge's `follows` document
    (`../../domain/forge/follows.md`, section 7; `forge::follows::watch`)
    of each of the first four projects read, in `me`'s order
    (`all_home::FOLLOWED`, 4), each watch counted in the client's
    `watches`: a project's watch is held while the project is among those
    four, and dropped when it leaves them and on leaving the page. Four
    projects' statuses, decided and `follows` documents with the session's
    own watches hold 23, and 25 at most, of a shipped client's 26
    (`../../protocol/configuration/limits.md`, *The engine's watch rooms
    follow what it admits*). A document refused or ended draws none of its
    project's boxes, and the rest stand.
  - *The intro,* one function's composition, `all_home::intro`, so that a
    later part joins there:
    - *the kicker:* the day and time, as Home's; then, with the newest
      `previous` of the projects shown over a minute old, " · you last
      looked SPAN ago", SPAN Home's own word; then, with an Away spell of
      the person's on a project read whose `off` is after that look, " ·
      away SINCE to OFF", both by `../view.md`, 6's *clock*, the newest such spell;
    - *the headline:* the landed sentence across projects, then a space
      and the needs sentence, Home's words over every open card across
      projects, each open need once (`needs-you.md`, 1, *What the client
      keeps*, *The needs' rows*), "N+" where the needs' rows or
      `approvals` is cut. With N the sum
      over the projects shown of `landed` less `seen`, and P the number of
      them where it is above 0: "N changes landed across P projects.",
      P in number words to "nine", digits after; with P one, "N changes
      landed in PROJECT." or "One change landed in PROJECT."; for N 0,
      with a `last_landed` on any project shown, "The last change landed
      SPAN ago.", the newest, SPAN the kicker's word, "just now" under a
      minute; nothing with nothing landed. It never says a brief's
      `sentence`: across projects the headline is code's;
    - *the lede:* the briefs joined, `all_home::brief`: each project's
      brief that is new to the person by Home's rule (`home.md`, 1, *Home on any
      project*), newest `at` first, at most four, each its project's name
      and ": " in the strong weight, then the first sentence of its `lede`,
      up to and with the first ".", "!" or "?" that ends the lede or is
      followed by a space, the whole lede with none. The name says whose,
      since the person's brief in each project is written in that project's words
      (`../../draft/surfaces.md`, 15, its default);
    - *the byline:* "Each project's brief, its first line · the newest,
      vsr's, by the status agent at 07:52", vsr the newest joined brief's
      project's name with "'s", and 07:52 that brief's `at` by
      `../view.md`, 6's *clock*, in `temper-home-byline`: the same words
      whether one brief is joined or four. A project's Home keeps its own
      byline, "Written by the status agent at 07:52" (*Home on any
      project*). With no brief new, neither lede nor byline.

    Until a project's status is shown it counts nothing; while none is,
    the intro has its kicker's day and time and its needs sentence alone,
    the needs sentence once the inbox has answered.
  - *Needs you,* as a project's Home's (`home.md`, 1, *The cards*) across
    projects: the open cards of the unfiltered inbox with every
    `approvals` row, each card's from line with its project's chip; "All
    needs" and "N more" open `Inbox` unfiltered.
  - *Decided for you,* a region named "Decided for you", drawn while a
    row shows (`shared-behaviour.md`, 1, *Away*: a row shows for 24 hours from its `at`): its
    head the level-2 heading "Decided for you · N", N every row showing
    across the projects read, and at its right "What reaches you", to the
    open project's `Settings { project }`, whose How work runs tab holds
    what reaches the person; then at most seven rows, the newest kept,
    oldest first, each a `temper-decision` as a project's Home draws it
    (`shared-behaviour.md`, 1, *Away*, *Home, while a decision shows*): mono "PROJECT · TITLE ·
    01:05", then the question, what it took and why, and at its right what
    a project's Home has there, "Overturn" on a live task. Overturn is
    *Away*'s act under the row's project.
  - *Projects,* a region named "Projects", while the person is in two
    projects or more: its head the level-2 heading "Projects"; then
    `temper-projects`, a row a project read, in `me`'s order: its name, a
    link to its `Home`; and its health in chips: BRANCH, `temper-chip-good`,
    while its status is shown and not `red`; "BRANCH red",
    `temper-chip-signal`, while `red`; none while its status is unknown.
    BRANCH is its status's `branch`, "landing" where it is empty, as Home's
    foot says it. A project's Now, its tiers, a follow's parity and
    "owned" are not designed here (`home.md`, 1, *A page's parts*).
  - *What follows what* (`../../draft/surfaces.md`, section 3;
    `../../domain/forge/follows.md`, section 7;
    `../../domain/forge/dependencies.md`, section 4), a region named "What
    follows what", while a box is drawn (`home.md`, 1, *A page's parts*): its head
    the level-2 heading "What follows what", at its right, small and
    muted, "across projects"; then its boxes side by side, wrapping, as
    How work runs draws What follows a landing's (`settings.md`, 2): for
    each project whose `follows` document has answered, in `me`'s order,
    a box each of its follows, in the document's order, then a box each
    of its dependencies, in the document's order, which is their numbers'.
    Each box holds a chip, as How work runs' chips are, its title, in
    weight, and a small muted line:
    - *A follow:* the chip "follows SOURCE", code's mark (the path's,
      `chat/conversation.md`, 3), SOURCE its source repository as the
      document names it, `OWNER/NAME`; the title PROJECT, the document's
      project's name as `me` names it; the line "LAG · PARITY" in `temper
      ctl follow list`'s words (`../../domain/forge/follows.md`, section 7;
      `forge::follows::lag` and `forge::follows::parity`), "2 behind ·
      check passes at 9c04d7e1a2b3" or "caught up · no check yet"; and
      where the follow is stopped, a second line, "Stopped: WORDS.", as
      How work runs has it.
    - *A dependency:* the chip "depends on DEPENDENCY", with no mark,
      since nothing acts on it (`../../domain/forge/dependencies.md`, 3),
      DEPENDENCY the named project's name where `me` lists that project,
      else "project Q", as `forge::follows::dependency_line` names it
      (`../../domain/forge/dependencies.md`, 4); the title PROJECT; the
      line "Nothing follows its landings."

    So the pilot draws a box "follows acme/vsr", **vsr-rs**, "2 behind ·
    check passes at 9c04d7e1a2b3", then "depends on vsr-rs", **fabrig**,
    "Nothing follows its landings.". Each box is a group named by its
    title and chip, "vsr-rs follows acme/vsr", "fabrig depends on
    vsr-rs". Every follow and dependency a project read holds is drawn,
    whichever project its other end is: a follow's source is a
    repository, which the document doesn't tie to a project. Nothing in
    the region is a control: a follow and a dependency change by
    proposal, kinds 12 and 13, and a follow by its owner's acts in
    `temper ctl` (`../../domain/forge/follows.md`, section 5). A
    project's Home draws none: its follows are How work runs'
    (`settings.md`, 2).
  - *The Ask temper line,* `temper-ask-line` (the mockups' `.ask`), the
    page's last part: a field named "Ask temper", its placeholder "Ask
    temper about your projects", and the button "Ask". Enter or "Ask"
    with words sends `Act::AskWith { words }`, which opens `Ask { project
    }` on the project in view, the open project here, with the words in
    its composer, unsent, as if typed there; with none it does nothing.
  - *On a phone* (`phone-home.html`): the kicker "Sunday 08:04 · last
    looked 9 h ago", `../view.md`, 6's short span; the headline,
    "31 changes landed. Two things need you.", whose landed sentence names
    no project, whatever P: "N changes landed." or "One change landed.",
    and for N 0 the wide page's, then the needs sentence as above, a
    level-1 heading in the phone's form beside the wide one, as the
    kicker's two forms are; the lede as above; the byline "The status
    agent, 07:52"; the cards in their phone
    form; then, while a row shows, one line "Decided for you · N ›", a
    link to *Needs you* unfiltered, which lists them. No Projects, no
    What follows what and no Ask line: the phone asks from its Ask tab (`../../draft/surfaces.md`,
    10). The browser's alone.

  In the terminal the page reads as Home's does there: its parts keep
  their headings, each line one readable run of words, and the cards'
  choices print as buttons.

## 2. Open questions

- **What follows what across many projects** (`home.md`, 1, *Home on All
  projects*): Home on All projects watches at most four projects'
  `follows` documents, the first four read, within the client's watches;
  a person in more sees the others' follows on each project's How work
  runs (`settings.md`, 2), and the region says nothing of them. A forge
  document of the person's follows and dependencies across the
  deployment would lift the bound; default: four.
