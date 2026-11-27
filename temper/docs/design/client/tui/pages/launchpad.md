# The TUI's launchpad

The TUI's first screen once signed in, or while it signs in from the
engine's link: the root at `Home { project }` (`../host.md`, 1, *Its
pages*), drawn from `../../../draft/surfaces.md`, 11.1. Its mockup is `../whole-story/launchpad.html`, with
`../whole-story/pair/` and `../whole-story/team/` beside it, whose words,
parts and order bind and whose figures don't. It shares its foot, the
box and the quiet line or a list, with a chat, and its commands are the
TUI's (`README.md`, *The foot* and *The commands*). Its words are said as
`README.md`, *Its words* has them: the view's shared surface,
`temper_client_view::shared`, wherever the browser says the same thing.
Every frame below is illustrative: its words and look are refined on
the recording (`../../../draft/tui.md`, section 5).

## 1. The screen

Illustrative, a person in two projects, Hello and shop:

```

  ▄█▀▀   temper 0.0.0 (b948f40d6df1)
 ▄█▄█▄█  127.0.0.1:8367

 4 changes landed across two projects. Two things need you.
 Hello: CSV export is in review. shop: Coupons are two of three in.
                                   written by the status agent 07:42 · /brief

 Where you left off
   Hello   Away mode, a chat · today 08:12 · waiting on you
   shop    coupons, a goal · Fri 18:40 · Working

 Needs you
   ? Hello   csv-export: comma or semicolon? · asked 20 min ago
   ✓ shop    Expired coupons say why · asked yesterday        /needs walks them

 Start
   /change   one change, run as Hello's way says
   /ask      Ask temper

────────────────────────────────────────────────────────────────────────────────
❯ Ask for a change, or ask temper.  / for commands
────────────────────────────────────────────────────────────────────────────────
 ● Live · Hello · Claude Opus 5.5 · high                    2 need you · /needs
```

- **Its parts, in order:** the header (`README.md`, 3); then the
  sections (`tui-spaced`, a blank row between two, and one between the
  header and the first): the headline (3), Where you left off (4), Needs
  you (5), Team on PROJECT (7) and Start (7); then the notes (7); then
  the foot (`README.md`, *The foot*).
- **Where it departs from its mockups,** each for a reason: the brief is
  one a project, joined, not one across projects written per person
  (`../../../draft/surfaces.md`, 15, its default; `../../pages/home.md`, 1, *Home on All
  projects*), and so the headline section holds Home on All projects' headline over
  the joined briefs, and `/brief` the briefs whole (3), since the engine
  keeps a sentence and a lede for each project and no more
  (`../../../domain/forge/attention.md`, 3); the header is two lines,
  the mark, the build and the place, and names no project (`README.md`,
  3), where the mockups frame the project and the deployment, the
  project named on the quiet line: the user's request `tui-14` of
  2026-11-15; what
  is open in a row is its state as Home words it; a need's row says when
  it asked, "asked 20 min ago", and not who, since no read names the
  asker (8); the quiet line names the link's state, "● Live", as the
  Pair mockup does, first, at the row's start, where the mockup has it
  after the model: the user's request `tui-21` of 2026-11-15 has it in
  one place on every screen (`README.md`, *The link's state*); the sections keep the at-scale mockup's order, Where
  you left off before Needs you; "Landed this week" (Pair's) is not
  drawn, not being in `../../../draft/surfaces.md`, 11.1; Team on
  PROJECT's columns are its rows' names, the areas each holds and its
  line, where the mockup words an area as a phrase ("tests and proof"),
  since an area's name is a label's (`../../../domain/areas.md`, 2), and its
  rows are at most four (7); and Where you left off lists no talk with a
  role, which no read lists (4).

## 2. Its reads, across the person's projects

- **Its project:** the root's, `Home { project }`. Start, the box and
  the quiet line are that project's: the box asks
  Ask temper in it with Home's composer and picker (`../../pages/home.md`, 1,
  *In the TUI*), and `/change` starts its chat. The brief, Where you
  left off and Needs you are the person's, across their projects, as
  `../../../draft/surfaces.md`, 11.1 has them.
- **Asked by the TUI alone:** the domain's
  `boundary::Event::Launchpad`, which the TUI's host gives the root
  once, right after `LookKept` follows `Start` (`../host.md`, 1, *The
  look*). The domain keeps it for the root's life. No other shell gives
  it, so the browser's Home, and `temper-box open`'s, are unmoved.
- **Home, with it given,** holds beside its own reads (the project's
  status, its composer and picker, its chats, which start Ask temper's
  chat, and its goals):
  - *the statuses across projects:* Home on All projects' reads with
    their statuses (`../../pages/home.md`, 1, *Home on All projects*, *Its
    reads*): the subject `status` of each of the first
    `all_home::ACROSS` projects of `me`, in its order, and `Looked` once
    each, so that each project's `previous` and `seen` move as its Home's
    entry moves them. The root's project is read once, by Home's own
    status and its own `Looked`: one watch and one `Looked` a project.
    `all_home::facts(root, wall)`, and so `all_home::intro` and
    `all_home::brief`, answer on this Home as on `AllHome`.
  - *the inbox unfiltered:* `Filter::ALL` in place of the project's
    filter, so that the needs (`pages().needs().needs(inbox,
    approvals)`) and `shared::need_prompts` hold every project's open
    cards and every `approvals` row, in Needs you's order, as on
    `Inbox { filter: ALL }` (`../../pages/needs-you.md`, 1, *An approval*). Home's
    own facts then say its needs `Unknown`, since they read the
    project's filter; the TUI draws none of Home's facts' needs.
  - *the decided documents* that a need's reason reads
    (`../../pages/needs-you.md`, 1, *Why it reached you*): the shared
    *Decided for you* store's, within its four projects (`../../pages/home.md`,
    1, *Decided for you*), taken as the launchpad's projects, the root's
    project first (*the team's reads*, below); so the projects of the
    first needs walked have theirs.
  - *the left-off reads,* `pages::launchpad::LeftOff`: two watches of
    jig's `Tasks` with no project, which jig answers with the person's
    own tasks across the projects they are in: the person's chats,
    requester the person, the chat template's label and the order
    `Activity`; and the person's goals, requester the person, the goal
    template's label, `tracked` and the order `Activity`; each window the
    client's `rows` limit, as Home's chats read theirs. The labels are
    the root project's templates' (`jig_client::shapes::template` of
    `shapes::CHAT` and `shapes::GOAL`), which every project shares: the
    stock's chat, Ask temper and drafting templates carry the chat's
    label.
  - *the team's reads,* for Team on PROJECT (7), of the root's project
    alone: Team's roles' watch and the project's document, which every
    project page reads (`../../pages/team.md`, 1, *Its reads*); each listed
    role's Now, read as Team reads them, which Team alone read before;
    and the project's findings document, for its areas and triager
    (`../../pages/talk.md`, 3), Needs you's for the project: the projects
    whose findings and decided documents the launchpad reads are the
    root's project first, then the open needs' distinct projects in their
    order, then `me`'s order, to four, so a walk's need of a fourth
    project beside them says no reason.

  Leaving Home leaves them all, as leaving `AllHome` does.
- **What crosses to the TUI,** the domain's public surface:
  `boundary::Event::Launchpad`, a variant with no fields;
  `Pages::left_off(&self) -> Option<&launchpad::LeftOff>`, `Some` on a
  Home that holds them; `LeftOff::chats(&self, client: &Client) ->
  Option<&c::Tasks>` and `LeftOff::goals(&self, client: &Client) ->
  Option<&c::Tasks>`, each `None` until its watch has delivered;
  `all_home::facts`, `intro` and `brief` as above; and, for the quiet
  line's count, `Session::count_across` (`README.md`, *The quiet line*);
  for Team on PROJECT (7), `team::shown_at(root, project, wall)`'s
  `roles`, each with its Now, and its reader's `Person` with what waits
  on them, and `team::areas(root, project)`.
- **The view's words across projects,** in `temper_client_view::shared`,
  which the AllHome page's builder calls too, so that the two say them
  alike: `shared::all_home_headline(w: &Words, facts: &all_home::Facts,
  now: Wall) -> (Box<[u8]>, Option<Wall>)`, Home on All projects'
  headline, its landed sentence across projects then its needs sentence
  (`../../pages/home.md`, 1, *Home on All projects*, *the headline*), with the
  instant its words next change; and `shared::joined_brief(w: &Words,
  brief: &all_home::Brief) -> Box<[u8]>`, its lede's words, each joined
  brief's project's name, ": " and its sentence, joined by a space (*the
  lede*).
- **A need's prompt** (`shared::NeedPrompt`) carries, beside its object,
  project, mark, line and act: `since: Option<Wall>`, when the need began
  waiting, a card's wait as the inbox's page says it and an `approvals`
  row's `since`; and `why: Option<Box<[u8]>>`, its reason line,
  `shared::why` of its need, `None` where that gives none.
- **Each drawn at the next frame after it changes;** the TUI derives
  none of them again.

## 3. The headline

The launchpad's first section, under the header (`README.md`, 3), with
no head: what the framed header held inside it before the user's request
`tui-14` of 2026-11-15, its parts in this order, each left out when
absent, and the section left out when none is drawn. The project's name
is the quiet line's, and the address the header's second line.

- *Home on All projects' headline,* once the inbox has answered: its
  landed sentence across projects and its needs sentence
  (`shared::all_home_headline`), "4 changes landed across two projects.
  Two things need you.", one paragraph, never a brief's sentence, since
  across projects the headline is code's; before any project's status is
  shown, the needs sentence alone. While the inbox has not answered,
  nothing (`../../pages/README.md`: no page claims empty
  before its read answers).
- *with a brief new to the person* in any project (`all_home::brief`):
  the joined briefs, newest first, at most four, one paragraph
  (`shared::joined_brief`), "Hello: CSV export is in review. shop:
  Coupons are two of three in."; then the byline "written by the status
  agent HH:MM · /brief" (`tui-byline`, aligned to the end), HH:MM the
  newest joined brief's `at` in the view's clock (`../../view.md`, 6,
  *clock*), and " · /brief" in the byline's paint: `/brief` prints the
  briefs whole (7). With none new, neither paragraph nor byline.
- *before `me`:* "Connecting to PLACE", PLACE as the header's second
  line names it (`README.md`, 3), the state "Connecting";
  *out of date,* in front of a box: the view's "This temper is out of
  date: run the temper the engine runs.", the state "Out of date"; out
  of front, the screen of a build refused stands in the launchpad's
  place (`refused.md`);
- *a sign-in not kept,* when the credential's write failed: the line
  "This sign-in could not be kept: the next temper asks for a link
  again." (`tui-refused`).
- *in front of a box,* the playground's project has no brief and no
  headline: its banner stands in their place (`../box.md`).

## 4. Where you left off

The first of the sections: the term "Where you left off" (`tui-head`)
and under it the conversations the person can go back to, across their
projects, at most three, newest activity first, the lower task first
between two of one time: of the person's chats (`LeftOff::chats`), those
that have not ended, and never one this run closed by `/close`, left
out from the frame the TUI goes back to the launchpad on, before its row
reads ended (`chat.md`, section 1, *Closing*); and the person's newest goal, of their goals
(`LeftOff::goals`), the one not ended with the newest last activity,
among the rows the read holds. Each is a row, an unnumbered item of a
list under `tui-left-off`, whose mark is a space, so that its words
stand two columns in:

- *its project,* the row's project's name in `me` (`tui-name`), padded
  with spaces to the longest among the section's and Needs you's rows
  and three more, so that the two sections' words start in one column;
- *what it is and when:* its title, ", a design session" for a chat
  that is one (its chip's executor read by `pages::chats::drafting(me,
  project, executor)`, its row's project, `../../pages/README.md`, *A design
  session*), else ", a chat", or ", a goal", " · " and its last activity
  in the view's *dated* form (`../../view.md`, 6: "today 08:12" today, "Fri
  18:40" in the week, the date before), then " · " and its state as
  Home's Chats row words it (the view's `shared::chat_state`), what is
  open in it, all but the title in `tui-hint`;
- *no number* (the user's word of 2026-11-27): a chat's place among the
  section's chats, 1 the newest, is what `/chats N` opens (`README.md`,
  *The TUI's own commands*), and no row draws it. The row's words are
  cut at a word with "…" at the row's end.

Left out with no such conversation, and while neither read has
answered. A chat that writes a draft and is no design session says ", a
chat". A talk with a role (`../../../draft/surfaces.md`, 11.1 and 11.3) is
not listed: no read lists the talks a person has open (`../../pages/talk.md`,
4), and `/talk NAME` opens a role, and its open talk with it
(`README.md`, 2, *`/talk`*).

Its rows are the first of the launchpad's list: Up from the box reaches
the newest, and Enter on a chat's row opens it (7, *The list*).

## 5. Needs you

The next section: the term "Needs you" and a row a need, across the
person's projects, at most five, in Needs you's order (`Session::needs`,
the inbox's cards then the forge's approvals, those marked later last),
each as the view's `shared::need_prompts` reads it:

- *its mark* (`tui-signal`): "?" for a question, "$" for money, a task
  held on its budget, "✓" for a sign-off, a change waiting for the
  person's review on the forge, and "•" for any other kind (a proposal,
  another hold, a person's task). A kind the engine adds later takes its
  mark then (`../../../draft/surfaces.md`, 11.1);
- *its project,* padded as Where you left off's (4);
- *its words* as Home's lines say them, "TITLE: SHORT"
  (`shared::need_line`);
- *when it asked,* " · " and "asked AGO" (`tui-hint`), AGO its `since`
  in the view's *ago* form (`../../view.md`, 6): "asked 20 min ago", "asked
  just now", "asked at 08:12", "asked yesterday", "asked Tue"; left out
  with no `since`. It is cut with the row's words, from their end.

The section's last row ends with "/needs walks them" (`tui-hint`) at the
row's end, the row's words cut at a word with "…" where they and two
spaces would not leave it room, and left out where that would leave the
words fewer than 24 columns. Left out while nothing needs the
person, as the quiet line's count is, and while the needs' read has not
answered. A row says no reason: its reason is the walk's (6).

## 6. Walking what needs you

`/needs`, or Enter on a need's row of the launchpad's list (7, *The
list*), which shows that need first: a prompt a need, under the box in the quiet line's
place (`tui-list`, an options menu after the box, its head a line of
its own, `tui-list-head`), one after another in Needs you's order
(`../../../draft/tui.md`, section 3, *Needs from elsewhere*), across the
person's projects:

- *its reason,* under the head, a line of its own (`tui-why`): the
  prompt's `why`, "Reached you because it is an implementation choice,
  which asks you unless you’re away." (`../../pages/needs-you.md`, 1, *Why it
  reached you*); left out where it has none, and drawn once its
  project's decided document answers;
- *a question* the person may answer: as the chat's question
  (`chat.md`, 2, *The chat's question*), its question line the head, its
  choices numbered, the recommended one first and active, "Tell it
  something else", then "Next";
- *a sign-off:* its row's words the head; "1. Review on FORGE ↗", the
  browser's words, which opens the forge's review page through the
  opener (`shared::need_prompts`' `Review`, the page the browser's card
  links), then "2. Next";
- *money and any other kind:* its row's words the head; "1. Open it on
  the web", which opens Needs you for the need's project on the web
  (`/needs?project=P`, `address::print`) through the opener, then "2. Next".

A choice taken, or "Next", shows the next need the walk has not shown;
with none left the walk ends and the TUI notes "Nothing else needs you."
(`tui-note`) until the next send or screen. Esc ends the walk. "Tell it
something else" sets the question aside: the box's next words answer it,
as a chat's question's words do, and the walk goes on after the send.
Words typed while a prompt stands end the walk, the box keeping them.
`n` types on every screen, the user's word of 2026-11-27: no bare
letter starts the walk. On a chat a chat's question is its own.

## 7. Its other parts

- **No project yet:** while `me` names no project the root stays at its
  start's address (`../host.md`, 1): the border holds "temper" alone, and
  under it the view's "No project yet" and the TUI's "Setting temper up
  makes your first project: open ORIGIN in a browser.", ORIGIN the
  origin printed, `http://HOST:PORT` or `https://NAME` (`../README.md`, 1). The
  box takes commands alone, its placeholder "/ for commands", and Enter
  on other words sends nothing: Ask temper's chat is a project's, by its
  template (`../../../draft/ask-temper.md`, section 9), so with no project
  there is no one to ask. In front of a box the welcome stands in its
  place (`../box.md`).
- **Team on PROJECT** (`../../../draft/surfaces.md`, 11.1: "each role, its
  area and its Now in a line"), after Needs you, while the launchpad has
  a project with a live standing role, or an area its person holds: the
  term "Team on PROJECT" (`tui-head`), PROJECT the root's project's name,
  and under it a row each, an unnumbered item whose mark is a space, as
  Where you left off's (4), at most four:
  - *its rows,* the project's live standing roles in Team's order
    (`team::shown_at`'s `roles`, `../../pages/team.md`, 1, *Standing roles*),
    then, while the person holds an area of the project, theirs, which
    keeps its place where the roles fill the rest;
  - *its name:* a role's name (`tui-name`), or "You" on the person's,
    padded with spaces to the longest of the section's names and three
    more;
  - *its areas:* the names of the areas it holds (`team::areas`;
    `../../pages/talk.md`, 3), in the areas' order, joined by ", ", then
    "triage" where it is the project's triager (`tui-hint`), padded to the
    longest of the section's and three more; a column left out where no
    row holds one;
  - *its line* (`tui-hint`): a role's, its Now's first line, as Team's row
    reads it; the person's, what waits on them in the project, as Team's
    person line says it, "Two questions wait on you." (the view's
    `shared::person_line`, `../../pages/team.md`, 1, *A person's line*);
    none while unread.

  The row's words are cut at a word with "…" at the row's end. Left out
  while the roles' watch has not answered, and with no row.
- **Start,** the next section, while the launchpad has a project: the
  term "Start" (`tui-head`) and a row each for the TUI's commands that
  start work, its name (`tui-name`) padded with spaces to the longer and
  three more, then its words (`tui-hint`): "/draft", "a design session
  that ends in a draft and a goal", while the project offers drafts
  (`pages::drafts_offered(root, project)`, `../../pages/README.md`, *A chat's
  document*); "/change", "one change, run as PROJECT's way says";
  "/talk", "a role, about its area", while the project has a live
  standing role (`README.md`, 2, *`/talk`*); "/ask", "Ask temper"; and
  in front of a box, while `/web` is offered, "/web", "Open temper in
  your browser" (`../box.md`, *`/web`, temper in the browser*).
- **The list** (the user's word of 2026-11-27: the sections are a list
  above the box): the rows of Where you left off (4), Needs you (5), Team
  on PROJECT (a role's row: the person's "You" acts on nothing and is
  none of it) and Start, in that order; the headline and the notes are
  none of it. The box is the only node of the launchpad a person acts
  on (`README.md`, *The box*); the list's keys are the host's, read
  before skein's key reader (`../host.md`, 1, *A key*), Ctrl-p and
  Ctrl-n as Up and Down (skein's `ui/screen.md`, 6.2, *Emacs's keys*).
  `/chats` still opens any live chat of the project; what needs the
  person is walked, answered in its chat or on the web.
  - *Into it:* Up on the box with its caret on the box's first line,
    the box empty or not, while no list stands under the box, moves the
    focus to the list's first row, the newest conversation, so that Up
    then Enter reopens it. Skein moves no caret between a text area's
    rows (skein's `ui/screen.md`, 10, *A text area's rows*), so until it
    does, Up anywhere in the box is on its first line. With no row, Up
    is as it was; Down on the box enters nothing.
  - *Drawn:* the row with the focus carries `tui-picked`: the focus's
    mark `❯` in its list mark's place, a need's own mark kept after it,
    and its words in the accent, across the band where the theme has one
    (`../theme.md`). The box keeps its words, never cleared, drawn
    without the focus: no caret, its words dim (`tui-hint`). The quiet
    line's right half says "Enter opens · Tab sections · Esc back to the
    box" (`README.md`, 1, *The quiet line*).
  - *Moving:* Up and Down move a row at a time, falling through from
    one section to the next; Up at the first row stays, and Down past
    the last row goes back to the box. Tab moves to the next section's
    first row, staying in the last section; Shift-Tab to the section
    before's first row, and in the first section to its first row.
    PgUp and PgDn page the body, as anywhere.
  - *Enter, or Right,* acts on the row, and the focus goes back to the
    box: a chat's or a design session's opens it, `Act::Go` to its
    `Chat { task }`, followed as a chat chosen from `/chats` is, whatever
    its project; the goal's opens its page on the web, as `/open` does;
    a need's starts the walk from it (6); a role's opens a talk with
    it, `Act::Go` to its `Talk { project, task }`, as `/talk`'s choice
    does (`README.md`, section 2, *`/talk`*); a Start row does what its
    command pressed does: `/draft`, `/change` and `/ask` go to their
    place not yet started, `/talk` opens its list, `/web` the browser.
  - *Out of it:* Esc goes back to the box and its words; so does a key
    that types (a character, a paste, Backspace or Delete), which then
    acts in the box, typing adding to its words, the caret at their end.
    Words that begin with "/", typed so, keep the row picked as the
    object in focus, `tui-picked` still drawn, so that `/open` opens its
    page (`README.md`, section 2, *`/open`*); other words, Esc on a
    command's list and leaving the launchpad clear it. A row no longer
    drawn, a chat ended or a need answered, returns the focus to the
    box. Left and any other key do nothing in the list.
  - *Its state,* `Tui`'s: the row picked, by what it is (a chat's, the
    goal's or a role's task, a need's object, a Start row's command),
    and whether the list holds the focus; the view's event `Picked {
    row }` (`../host.md`, 1).
- **`/brief`'s note:** the briefs whole, after the root's notes in the
  sections, until the next send or screen: for each joined brief, newest
  first, its project's name and ": " then its sentence, and its lede, a
  paragraph each; then the byline. Offered while the header holds a
  byline (`README.md`, *The TUI's own commands*).
- **The terminal's notes** (`../../pages/chat/terminal.md`, 1, *The terminal's
  notes*) follow the sections, each in `tui-note`.

## 8. Open questions

- **Stop asking in the TUI** (6; `../../../draft/what-reaches-you.md`,
  section 5): the walk's reason line offers neither Stop asking nor Keep
  asking, whose offers, words and Apply or Propose are Needs you's on the
  web. Default: the web's; "Open it on the web" reaches it for a kind
  the walk can't answer, and a question's reason says why alone. Owner:
  temper/tui.
- **A need's asker on its row** (5): "asked by the change" or "asked by
  Kit", as the Pair and Team mockups draw it, once a read names who asked
  (`../../pages/needs-you.md`, 1, *The card*: the goal, role or asker joins
  the web's from line as their reads come). Default: the age alone.
- **A chat's question's reason** (`chat.md`, 2, *The chat's question*): a
  chat's page watches no decided document, so its question says no
  reason. Default: none until a chat's page reads its project's.
- **One brief across projects, written per person**
  (`../../../draft/surfaces.md`, 15): default one a project, joined, as Home on
  All projects takes it (3).
