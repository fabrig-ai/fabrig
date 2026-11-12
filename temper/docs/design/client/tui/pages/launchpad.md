# The launchpad

## 1. The launchpad

As drawn (`../../../draft/surfaces.md`, 11.1, less its
team lines and `/draft` and `/talk`; its mockup
`tui/whole-story/launchpad.html`, and `pair/` and `team/` beside it, whose
words, parts and order bind and whose figures don't), illustrative:

```
╭─ temper · Hello ──────────────────────────────────────────── 127.0.0.1:8367 ─╮
│ 3 changes landed overnight.                                                  │
│                                   written by the status agent 07:42 · /brief │
╰──────────────────────────────────────────────────────────────────────────────╯

 Where you left off
   Hello   Away mode, a chat · 08:12 · waiting on you                /chats 1
   Hello   csv-export, a goal · Fri 18:40 · Working

 Needs you
   ? Hello   csv-export: comma or semicolon?                       n walks them

 Start
   /change   one change, run as Hello's way says
   /ask      Ask temper

────────────────────────────────────────────────────────────────────────────────
❯ Ask for a change, or ask temper.  / for commands
────────────────────────────────────────────────────────────────────────────────
 Hello · Claude Opus 5.5 · high · ● Live                 1 needs you · n to walk
```

- **Where it departs from its mockup,** each for a reason: the brief is
  the project's, not one across projects, and so are the rows, since its
  root is a project's Home, not `AllHome` (`../../pages/home.md`, 1, *Home on All projects*;
  `../../../draft/surfaces.status.md`, 11);
  the header holds the brief's sentence and `/brief` its whole, since
  the engine keeps a sentence and a lede and no more (`../../../domain/forge/attention.md`,
  3); a chat's row ends with `/chats N`, numbering the rows as the
  open question's default numbers `/chats`' list (`../../../draft/surfaces.md`,
  15), where the mockups draw `/chats` once, on the first row
  (`../../../draft/surfaces.status.md`); what is open in a row is its state
  as Home words it; the quiet line names the link's state, "● Live", as the Pair
  mockup does; the sections keep the at-scale mockup's order, Where you
  left off before Needs you; "Landed this week" (Pair's) and "Team on
  PROJECT" are not drawn, the first not in `../../../draft/surfaces.md`, 11.1,
  the second T3's.

- **Its project:** the root's, `Home { project }` (`../README.md`, 3, *Its pages*).
  Home's reads are the launchpad's: the status agent's brief and what
  needs the person through `pages::home::facts(root, project, wall)`,
  the needs' cards through `pages().needs()`, read as the view's
  `shared::need_prompts` says them (*Needs you*, below), the person's
  chats through `pages().chats()` and the project's goals through
  `pages().goals()` (`../../pages/home.md`, 1). The TUI derives none of them again,
  and each is drawn at the next frame after it changes. They are the
  project's: the reads across projects are `AllHome`'s, its joined brief
  `pages::all_home::brief` (`../../pages/home.md`, 1, *Home on All projects*), which the
  launchpad does not take (`../../../draft/surfaces.status.md`, 11).
- **The header,** the region `tui-top`, framed, its level-1 heading in
  its border: "temper" (`tui-title`), " · " (`tui-faint`) and the
  project's name (`tui-name`); and, at the border's end, `HOST:PORT`
  (`tui-address`, aligned to the end), which skein leaves out first
  where the border lacks the room for both (skein's `ui/screen.md`,
  5.2). Inside it:
  - *with a brief new to the person* (`Facts::brief`, `../../pages/home.md`, 1, *Home on any
    project*): its sentence, one paragraph, then the byline "written by
    the status agent HH:MM" (`tui-byline`, aligned to the end), HH:MM its
    `at` in the view's clock (`../../view.md`, 6, *clock*), and, where its lede has
    words, " · /brief" after it, in the byline's paint: `/brief` prints
    the brief whole (`README.md`, section 2, *The TUI's own commands*, below);
  - *with none,* Home's headline, as the browser's Home says it, by the
    view's one function (`../../pages/home.md`, 1, *the headline*): the landed sentence and
    the needs sentence, "The last change landed 2 hours ago. Nothing
    needs you.", with no byline, since the computed Home has no prose of
    its own; while Home's facts have not answered, nothing under the
    title (`../../pages/README.md`: no page claims empty before its read answers);
  - *before `me`:* "Connecting to HOST:PORT", the state "Connecting";
    *out of date:* the view's "This temper is out of date: run the
    temper the engine runs.", the state "Out of date";
  - *a sign-in not kept,* when the credential's write failed: the line
    "This sign-in could not be kept: the next temper asks for a link
    again." (`tui-refused`).
- **Where you left off,** the first of the sections (`tui-spaced`, a
  blank row between two), the term "Where you left off" (`tui-head`)
  and under it the conversations the person can go back to, at most
  three, newest activity first, the lower task first between two of one
  time: of Home's chats (`Chats::live`, the person's chats by last
  activity), those that have not ended; and the person's newest goal, of
  Home's goals (`pages().goals()`, the project's tracked goals), the one
  not ended that the person asked for (its requester the person) with
  the newest last activity, among the rows the read holds. Each is a
  row, an unnumbered item of a list under `tui-left-off`, whose mark is a
  space, so that its words stand two columns in:
  - *its project,* the row's project's name in `me` (`tui-name`), padded
    with spaces to the longest among the section's and Needs you's rows
    and three more, so that the two sections' words start in one column;
  - *what it is and when:* its title, ", a design session" for a chat
    that is one (its chip's executor read by `pages::chats::drafting(me,
    project, executor)`, `../../pages/README.md`, *A design session*), else ", a chat", or
    ", a goal", " · " and
    its last activity in the view's clock (`../../view.md`, 6, *clock*: "08:12" today,
    "Fri 18:40" in the week, the date before), then " · " and its state
    as Home's Chats row words it (the view's `shared::chat_state`), what
    is open in it, all but the title in `tui-hint`;
  - *its number,* on a chat's row alone: "/chats N" (`tui-hint`) at the
    row's end, N the chat's place among the section's chats, 1 the
    newest, which `/chats N` opens (`README.md`, section 2, *The TUI's own commands*). The row's
    words are cut at a word with "…" where they and two spaces would not
    leave it room; where that would leave the words fewer than 24
    columns, the number is left out. A goal has no number: the TUI has no
    goal's page.

  Left out with no such conversation, and while the chats' read has not
  answered. A chat that writes a draft and is no design session says ",
  a chat"; a talk with a role says what it is when it is the TUI's
  (`../../../draft/surfaces.md`, 11.3).
  - *Up and Down,* while the box is empty and no list stands under it,
    move a selection through these rows, and the box keeps the focus:
    with none selected, either selects the newest; then Up selects the
    one above and Down the one below, staying at the first and the last.
    Each makes the view's `LeftOff { task: Some(TASK) }` (`../README.md`, 3), and the
    TUI's state keeps the task. With words in the box, or a list under
    it, the arrows are the box's or the list's, as before.
  - *The selected chat* carries `tui-picked` too: the focus's mark `❯`
    in its mark's place and its title in the accent, as a list's active
    item is shown, across the band where the theme has one (`../theme.md`). A
    task that is no longer among the lines is no selection.
  - *Enter,* on the empty box with a chat selected, opens it: the box's
    Enter makes `Act { act: Go { address: Chat { task } } }`, followed as
    a chat chosen from `/chats` is (`../README.md`, 3). With the goal selected, Enter
    opens the goal's page on the web, as `/open` does.
  - *Cleared:* Esc, while a chat is selected and no list stands, makes
    `LeftOff { task: None }`; dismissing a command's list with Esc,
    ordinary words typed in the box and leaving the launchpad clear it
    too. While the box's words begin with "/", command composition keeps
    the selection so `/open` can open that row's page.
- **Needs you,** the next section, the term "Needs you" and a row a
  need, at most five, in Home's order (`Session::needs`, the inbox's
  cards then the forge's approvals, those marked later last), each as
  the view's `shared::need_prompts` reads it: its mark (`tui-signal`),
  "?" for a question, "$" for money, a task held on its budget, "✓" for
  a sign-off, a change waiting for the person's review on the forge, and
  "•" for any other kind (a proposal, another hold, a person's task);
  its project, padded as *Where you left off*'s; then its words as
  Home's lines say them, "TITLE: SHORT" (`shared::need_line`). The
  section's last row ends with "n walks them" (`tui-hint`), placed and
  left out as a chat's number is. Left out while nothing needs the
  person, as the quiet line's count is, and while the needs' read has
  not answered. A kind the engine adds later takes its mark then
  (`../../../draft/surfaces.md`, 11.1).
- **Walking what needs you,** `n` on the launchpad's empty box while
  Needs you stands and no list does, or `/needs`: a prompt a need, under
  the box in the quiet line's place (`tui-list`, an options menu after
  the box, its head a line of its own, `tui-list-head`), one after
  another in Needs you's order (`../../../draft/tui.md`, section 3, *Needs from
  elsewhere*):
  - *a question* the person may answer: as the chat's question (`chat.md`, section 2, below), its question line the head, its choices
    numbered, the recommended one first and active, "Tell it something
    else", then "Next";
  - *a sign-off:* its row's words the head; "1. Review on FORGE ↗",
    the browser's words, which opens the forge's review page through the
    opener (`shared::need_prompts`' `Review`, the page the browser's card
    links), then "2. Next";
  - *money and any other kind:* its row's words the head; "1. Open it on
    the web", which opens Needs you for the project on the web (`/?project=P`,
    `address::print`) through the opener, then "2. Next".

  A choice taken, or "Next", shows the next need the walk has not shown;
  with none left the walk ends and the TUI notes "Nothing else needs
  you." (`tui-note`) until the next send or screen. Esc ends the walk.
  "Tell it something else" sets the question aside: the box's next words
  answer it, as a chat's question's words do, and the walk goes on after
  the send. Words typed while a prompt stands end the walk; when `n`
  opened it and nothing was taken, the box holds "n" before them, so
  that a message beginning with n types as ever. On a chat `n` types: a
  chat's question is its own.
- **No project yet:** while `me` names no project the root stays at its
  start's address (`../README.md`, 3): the border holds "temper" alone, and under it
  the view's "No project yet" and the TUI's "Setting temper up makes
  your first project: open http://HOST:PORT in a browser.". The box
  takes commands alone, its placeholder "/ for commands", and Enter on
  other words sends nothing: Ask temper's chat is a project's, by its
  template (`../../../draft/ask-temper.md`, section 9), so with no project
  there is no one to ask.
- **Start,** the next section, while the launchpad has a project: the
  term "Start" (`tui-head`) and a row each for the TUI's commands that
  start work, its name (`tui-name`) padded with spaces to the longer and
  three more, then its words (`tui-hint`): "/draft", "a design session
  that ends in a draft and a goal", while the project offers drafts
  (`pages::drafts_offered(root, project)`, `../../pages/README.md`, *A chat's document*);
  "/change", "one change, run as PROJECT's way says"; "/ask", "Ask
  temper". `/talk` joins it when the TUI has it
  (`../../../draft/surfaces.md`, 11.3).
- **The box keeps the keys:** the box is the only node of the launchpad
  a person acts on, so the keys stay in it (`README.md`, section 1, *The box*, below), but Up,
  Down, Enter and Esc on the left-off rows and `n` while it is empty
  (*Where you left off*, *Walking what needs you*). `/chats` opens any
  live chat; what needs the person is walked, answered in its chat or on
  the web.
- **`/brief`'s note:** the brief whole, its sentence and its lede, a
  paragraph each, then its byline, after the root's notes in the
  sections, until the next send or screen.
- **The terminal's notes** (`../../pages/chat/terminal.md`, 1, *The terminal's notes*) follow the
  sections, each in `tui-note`.
