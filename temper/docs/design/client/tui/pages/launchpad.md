# The TUI's launchpad

The TUI's first screen once signed in, or while it signs in from the
engine's link: the root at `Home { project }` (`../README.md`, 3, *Its
pages*), drawn from `../../../draft/surfaces.md`, 11.1, less its team lines
and `/talk`. Its mockup is `../whole-story/launchpad.html`, with
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
╭─ temper · Hello ──────────────────────────────────────────── 127.0.0.1:8367 ─╮
│ 4 changes landed across two projects. Two things need you.                   │
│ Hello: CSV export is in review. shop: Coupons are two of three in.           │
│                                   written by the status agent 07:42 · /brief │
╰──────────────────────────────────────────────────────────────────────────────╯

 Where you left off
   Hello   Away mode, a chat · today 08:12 · waiting on you            /chats 1
   shop    coupons, a goal · Fri 18:40 · Working

 Needs you
   ? Hello   csv-export: comma or semicolon? · asked 20 min ago
   ✓ shop    Expired coupons say why · asked yesterday             n walks them

 Start
   /change   one change, run as Hello's way says
   /ask      Ask temper

────────────────────────────────────────────────────────────────────────────────
❯ Ask for a change, or ask temper.  / for commands
────────────────────────────────────────────────────────────────────────────────
 Hello · Claude Opus 5.5 · high · ● Live                 2 need you · n to walk
```

- **Its parts, in order:** the header (3); then the sections
  (`tui-spaced`, a blank row between two, and one between the header's
  frame and the first): Where you left off (4), Needs you (5) and Start
  (7); then the notes (7); then the foot (`README.md`, *The foot*).
- **Where it departs from its mockups,** each for a reason: the brief is
  one a project, joined, not one across projects written per person
  (`../../../draft/surfaces.md`, 15, its default; `../../pages/home.md`, 1, *Home on All
  projects*), and so the header holds Home on All projects' headline over
  the joined briefs, and `/brief` the briefs whole (3), since the engine
  keeps a sentence and a lede for each project and no more
  (`../../../domain/forge/attention.md`, 3); a chat's row ends with `/chats
  N`, numbering the rows as the open question's default numbers
  `/chats`' list (`../../../draft/surfaces.md`, 15), where the mockups draw
  `/chats` once, on the first row (`../../../draft/surfaces.status.md`); what
  is open in a row is its state as Home words it; a need's row says when
  it asked, "asked 20 min ago", and not who, since no read names the
  asker (8); the quiet line names the link's state, "● Live", as the
  Pair mockup does; the sections keep the at-scale mockup's order, Where
  you left off before Needs you; "Landed this week" (Pair's) and "Team
  on PROJECT" are not drawn, the first not in
  `../../../draft/surfaces.md`, 11.1, the second T3's.

## 2. Its reads, across the person's projects

- **Its project:** the root's, `Home { project }`. The header's title,
  Start, the box and the quiet line are that project's: the box asks
  Ask temper in it with Home's composer and picker (`../../pages/home.md`, 1,
  *In the TUI*), and `/change` starts its chat. The brief, Where you
  left off and Needs you are the person's, across their projects, as
  `../../../draft/surfaces.md`, 11.1 has them.
- **Asked by the TUI alone:** the domain's
  `boundary::Event::Launchpad`, which the TUI's host gives the root
  once, right after `LookKept` follows `Start` (`../README.md`, 3, *The
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
    1, *Decided for you*), taken as the distinct projects of the open
    needs in their order, then `me`'s order, to four; so the projects of
    the first needs walked have theirs.
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

  Leaving Home leaves them all, as leaving `AllHome` does.
- **What crosses to the TUI,** the domain's public surface:
  `boundary::Event::Launchpad`, a variant with no fields;
  `Pages::left_off(&self) -> Option<&launchpad::LeftOff>`, `Some` on a
  Home that holds them; `LeftOff::chats(&self, client: &Client) ->
  Option<&c::Tasks>` and `LeftOff::goals(&self, client: &Client) ->
  Option<&c::Tasks>`, each `None` until its watch has delivered;
  `all_home::facts`, `intro` and `brief` as above; and, for the quiet
  line's count, `Session::count_across` (`README.md`, *The quiet line*).
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

## 3. The header

The region `tui-top`, framed, its level-1 heading in its border:
"temper" (`tui-title`), " · " (`tui-faint`) and the project's name
(`tui-name`); and, at the border's end, `HOST:PORT` (`tui-address`,
aligned to the end), which skein leaves out first where the border lacks
the room for both (skein's `../ui/screen.md`, 5.2). Under its frame, a blank
row before the first section. Inside it:

- *Home on All projects' headline,* once the inbox has answered: its
  landed sentence across projects and its needs sentence
  (`shared::all_home_headline`), "4 changes landed across two projects.
  Two things need you.", one paragraph, never a brief's sentence, since
  across projects the headline is code's; before any project's status is
  shown, the needs sentence alone. While the inbox has not answered,
  nothing under the title (`../../pages/README.md`: no page claims empty
  before its read answers).
- *with a brief new to the person* in any project (`all_home::brief`):
  the joined briefs, newest first, at most four, one paragraph
  (`shared::joined_brief`), "Hello: CSV export is in review. shop:
  Coupons are two of three in."; then the byline "written by the status
  agent HH:MM · /brief" (`tui-byline`, aligned to the end), HH:MM the
  newest joined brief's `at` in the view's clock (`../../view.md`, 6,
  *clock*), and " · /brief" in the byline's paint: `/brief` prints the
  briefs whole (7). With none new, neither paragraph nor byline.
- *before `me`:* "Connecting to HOST:PORT", the state "Connecting";
  *out of date:* the view's "This temper is out of date: run the
  temper the engine runs.", the state "Out of date";
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
that have not ended; and the person's newest goal, of their goals
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
- *its number,* on a chat's row alone: "/chats N" (`tui-hint`) at the
  row's end, N the chat's place among the section's chats, 1 the
  newest, which `/chats N` opens (`README.md`, *The TUI's own commands*).
  The row's words are cut at a word with "…" where they and two spaces
  would not leave it room; where that would leave the words fewer than
  24 columns, the number is left out. A goal has no number: the TUI has
  no goal's page.

Left out with no such conversation, and while neither read has
answered. A chat that writes a draft and is no design session says ", a
chat"; a talk with a role says what it is when it is the TUI's
(`../../../draft/surfaces.md`, 11.3).

- *Up and Down,* while the box is empty and no list stands under it,
  move a selection through these rows, and the box keeps the focus:
  with none selected, either selects the newest; then Up selects the one
  above and Down the one below, staying at the first and the last. Each
  makes the view's `LeftOff { task: Some(TASK) }` (`../README.md`, 3), and
  the TUI's state keeps the task. With words in the box, or a list
  under it, the arrows are the box's or the list's, as before.
- *The selected chat* carries `tui-picked` too: the focus's mark `❯` in
  its mark's place and its title in the accent, as a list's active item
  is shown, across the band where the theme has one (`../theme.md`). A task
  that is no longer among the lines is no selection.
- *Enter,* on the empty box with a chat selected, opens it: the box's
  Enter makes `Act { act: Go { address: Chat { task } } }`, followed as
  a chat chosen from `/chats` is (`../README.md`, 3), whatever its project.
  With the goal selected, Enter opens the goal's page on the web, as
  `/open` does.
- *Cleared:* Esc, while a chat is selected and no list stands, makes
  `LeftOff { task: None }`; dismissing a command's list with Esc,
  ordinary words typed in the box and leaving the launchpad clear it
  too. While the box's words begin with "/", command composition keeps
  the selection so `/open` can open that row's page.

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

The section's last row ends with "n walks them" (`tui-hint`), placed and
left out as a chat's number is (4). Left out while nothing needs the
person, as the quiet line's count is, and while the needs' read has not
answered. A row says no reason: its reason is the walk's (6).

## 6. Walking what needs you

`n` on the launchpad's empty box while Needs you stands and no list
does, or `/needs`: a prompt a need, under the box in the quiet line's
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
  (`/?project=P`, `address::print`) through the opener, then "2. Next".

A choice taken, or "Next", shows the next need the walk has not shown;
with none left the walk ends and the TUI notes "Nothing else needs you."
(`tui-note`) until the next send or screen. Esc ends the walk. "Tell it
something else" sets the question aside: the box's next words answer it,
as a chat's question's words do, and the walk goes on after the send.
Words typed while a prompt stands end the walk; when `n` opened it and
nothing was taken, the box holds "n" before them, so that a message
beginning with n types as ever. On a chat `n` types: a chat's question
is its own.

## 7. Its other parts

- **No project yet:** while `me` names no project the root stays at its
  start's address (`../README.md`, 3): the border holds "temper" alone, and
  under it the view's "No project yet" and the TUI's "Setting temper up
  makes your first project: open http://HOST:PORT in a browser.". The
  box takes commands alone, its placeholder "/ for commands", and Enter
  on other words sends nothing: Ask temper's chat is a project's, by its
  template (`../../../draft/ask-temper.md`, section 9), so with no project
  there is no one to ask. In front of a box the welcome stands in its
  place (`../box.md`).
- **Start,** the next section, while the launchpad has a project: the
  term "Start" (`tui-head`) and a row each for the TUI's commands that
  start work, its name (`tui-name`) padded with spaces to the longer and
  three more, then its words (`tui-hint`): "/draft", "a design session
  that ends in a draft and a goal", while the project offers drafts
  (`pages::drafts_offered(root, project)`, `../../pages/README.md`, *A chat's
  document*); "/change", "one change, run as PROJECT's way says";
  "/ask", "Ask temper". `/talk` joins it when the TUI has it
  (`../../../draft/surfaces.md`, 11.3).
- **The box keeps the keys:** the box is the only node of the launchpad
  a person acts on, so the keys stay in it (`README.md`, *The box*), but
  Up, Down, Enter and Esc on the left-off rows and `n` while it is empty
  (4 and 6). `/chats` opens any live chat of the project; what needs the
  person is walked, answered in its chat or on the web.
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
