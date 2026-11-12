# The TUI's pages

Three screens, each a tree of the TUI's view, built from the root and
the TUI's own state: signing in, while the link is signed out; once
signed in, or while the TUI signs in from the engine's link (below), the
launchpad, the root at `Home { project }`; and a chat, the root at
`Chat { task }` (`../README.md`, 3, *Its pages*), with its draft beside it where it
has one (`design-session.md`, section 1). Signing in has a framed top, the
region `tui-top`, titled "temper", and a bottom row, the region
`tui-bottom`, spread: what closes the TUI, and the link's state. The
launchpad and a chat share their foot: the box, and under it the quiet
line or a list (`README.md`, section 1, below). The launchpad and a chat are drawn
from the user's hand-made frames of the look request `tui-launchpad`
(2026-11-08), at 80 by 24 in the dark theme; their words are the
browser's chat terminal's (`../../pages/chat/terminal.md`, 1), and a chat reads
as `../../../draft/chat-reading.md` says. Every frame below is illustrative:
its words and look are refined on the recording (`../../../draft/tui.md`,
section 5). In front of a box, the welcome stands in the no-project
launchpad's place, the playground's project says so, Ctrl+C and
`/quit` ask first while work runs, and the box's state and `/log` join
the foot and the commands (`../box.md`).

- **The TUI's state,** `temper-client-tui`'s `Tui`, holds what the root
  does not: the address, the link field's value, the no-project command
  composer's value, the sign-in's progress
  (asked, answered, lost, or the TUI's own refusal of what was pasted),
  whether it signs in from the engine's link and whether that ended,
  whether the link was signed out after a sign-in, and whether the
  credential's last write failed; the window's columns, for what a
  narrow row leaves out; the question set aside by "Tell it something
  else" (`chat.md`, section 2); the notice standing (`README.md`, section 1, *The notice*);
  the theme chosen, the terminal's scheme Auto reads, and whether the
  theme's list stands (`README.md`, section 2, *`/theme`*); the left-off chat selected on the
  launchpad, by its task (`launchpad.md`, 4, *Where you left off*); the reply whose choices
  were set aside, and the chat a goal's start was asked from (`design-session.md`, section 1); and whether it is on a
  box (`../README.md`, 2, *On a box*). The root stays the one source of the
  link's state, of `me`, of the pages and their reads, of the box's
  words with a project and of the terminal's look, lists and notes.
- **Its words:** where the TUI says what `temper-client-view` says, it
  says the view's phrases (`temper-client-view`'s `words`): the link's
  states, "Live", "Connecting", "Offline", "Signed out" and "Out of
  date" (`../../view.md`, 1); a link used, expired or not this temper's, and the
  sign-ins full; "No project yet"; "This temper is out of date: run
  the temper the engine runs."; "Signed out.". On the launchpad and a
  chat it calls the view's shared surface, `temper_client_view::shared`,
  for what the browser's terminal says of the same thing, and never the
  view's builders: Home's headline, a chat's state as Home's rows word
  it, a need's line, a run's line, the running line's words, the
  picker's closed line, a chat's short heading, the commands' words and
  when each is offered, the lists' heads and choices, a confirmation's
  question and acts, the chat's question, a card's line, the notes'
  words and the terminal's own phrases ("/ for commands", "✦", "❯").
  Its other words are its own phrases, under an owner of its own
  (skein's `ui/README.md`, section 9).
- **The link's state** is the bottom row's last child, a status in the
  view's words after the TUI's own `●`: "Live" once `me` is read on a
  live link, "Connecting" while it starts or before `me`, "Offline",
  "Signed out", "Out of date", each in its token's paint (`../theme.md`). On the
  launchpad and a chat it ends the quiet line.

- `sign-in.md`: signing in and renewing from the engine's link.
- `launchpad.md`: the launchpad, its rows and walking what needs you.
- `chat.md`: a chat's conversation and its question.
- `design-session.md`: the draft pane, reply choices and starting a goal.

## 1. The foot

The launchpad's and a chat's, from the bottom up: the
quiet line, or a list in its place; the box between two rules; on a
chat, the running line above them.

- **The running line,** the region `tui-working`: on a chat, while the
  browser's terminal draws its running line (`../../pages/chat/terminal.md`, 1, *The running line*;
  `shared::running`), "✦" and its words, "Working", "waiting on 1
  delegate", "Waiting for an account" (`tui-running`), with no clock and
  no pulse.
- **The box,** the region `tui-box`, framed by two rules across the
  window and no sides (skein's `ui/screen.md`, 5.2): one text area
  (`tui-field`) that submits on Enter, named "Ask temper" on the launchpad
  and "Message" on a chat, as the browser's composers are, its name the
  tree's and not laid (skein's `ui/screen.md`, 3, `label`); the focus's
  mark `❯` before it. Its placeholder: "Ask for a change, or ask
  temper.  / for commands" on the launchpad with a project (the words
  then two spaces and the terminal's phrase, as `../../../draft/surfaces.md`,
  11.1 and its Pair mockup have it), "/ for commands" with no project, "Reply" on a chat, and on a chat
  that has ended "/new starts another chat", as the browser's terminal
  keeps its box for commands there; none while a list stands under it.
  - *With a project its words are the root's:* Home's composer's on the launchpad
    (`Act::ChatWords`) and the chat's on a chat (`Act::MessageWords`),
    so that what the domain reads of them, a command's beginning, a
    send, a list's closing, is the browser terminal's. Its value is
    always those words; its write count moves whenever they change by
    anything but the person's typing (a command cleared them, a chat
    opened).
    With no project, the TUI owns the box's words separately from the
    sign-in field; typing, command completion, the footer and submission
    use that same value. Typing advances the TUI's revision without
    advancing the box's write count. Executing a command, dismissing its
    list and navigation clear the value and advance the write count; the
    link's changes, its sign-in and sign-out among them, leave the value
    alone (*Typed before the sign-in lands*).
  - *The focus:* the box asks for it as each screen opens and keeps it.
    It is the only node a person acts on outside a list, so Tab, Up and
    Down leave the focus where it is (skein's `ui/screen.md`, 6.2), but
    for the launchpad's left-off chats (`launchpad.md`, 4, *Where you left off*), and
    `q` types: the first screen's "q to quit" is gone. Ctrl+C closes the
    TUI from anywhere (`../README.md`, 4).
  - *Enter:* before the sign-in lands, the words kept and the notice
    "Still signing in" (*Typed before the sign-in lands*); on the
    launchpad's empty box with a left-off chat
    selected, that chat opened (`launchpad.md`, 4, *Where you left off*); else the box's
    words, by what they are, in this order: a
    command's beginning, as `README.md`, section 2 says; on a chat whose question
    stands, its answer in words (`chat.md`, section 2); with no
    project, nothing; otherwise sent: on the launchpad `Act::StartChat`,
    which asks Ask temper in the project and opens the chat it makes
    (`../../pages/home.md`, 1, *In the TUI*), on a chat `Act::Send`. A refusal or a
    failure the root keeps for the send is drawn under the box, in
    `tui-refused`, in the view's words.
  - *No new line:* a terminal reads Shift+Enter as Enter, so the box
    takes one paragraph; words wrap as the box's lines (`../../README.md`, section 5).
- **Typed before the sign-in lands,** the user's request of 2026-11-09,
  after a key typed on the welcome while the TUI signed in was drawn
  nowhere and then thrown away:
  - *Whenever the box is drawn, keys go to the box:* the welcome in
    front, the launchpad's "Connecting to HOST:PORT" while the TUI signs
    in from the engine's or the handed link, the launchpad and a chat.
    The link's field takes keys only while the sign-in screen is drawn,
    `temper_client_tui::sign_in_drawn(root, tui)`, the one test the
    view's choice of screen and the host's routing of a key both read
    (`../README.md`, 3, `Typed`).
  - *The sign-in never clears the box:* the link's changes leave its
    words (*The box*), and the host's going to the launchpad from the
    start's address once `me` names a project (`../README.md`, 3, *Its pages*) carries
    the no-project box's words into Home's composer, `Act::ChatWords`,
    so words typed before `me` was read stay in the box.
  - *Before it lands,* while the link is signed out or the root has not
    yet read `me` (`temper_client_tui::landing(root)`): the box's words
    open no list, and Enter keeps them, sends and runs nothing, and the
    notice says "Still signing in" (*The notice*). Once it has landed,
    the words are acted on at the person's next Enter, as any words are.
- **The quiet line,** the region `tui-bottom`, spread, as the sign-in's
  bottom row (`../../../draft/surfaces.md`, 11.1: the project, the model and
  effort, the link's state, and "N need you · n to walk"):
  - *at its left,* the project's name (`tui-hint`), " · ", the picker's
    closed line, "Claude Opus `../../pages/accounts.md`, 2 · high" (`shared::closed_line`,
    `tui-hint`), " · ", then "●" and the link's state in its token's
    paint (`README.md`, *The link's state*); with no picker (no project, no
    usable account) the closed line and its " · " are left out, and with
    no project the project and its " · " too; in a design session, the
    project's name, " · ", "drafter" (`tui-hint`), " · " and the closed
    line, the link's state after them only while it is not Live (`design-session.md`, section 1, *The quiet line*);
  - *at its right,* each left out when absent, " · " between two (`tui-faint`):
    in front of a box its state and "playground" (`../box.md`); "paused · a
    message resumes it" on a chat the person paused (`tui-hint`); on a
    chat whose draft can show, the pane's parts (`design-session.md`, section 1,
    *The quiet line*); while
    something needs the person, "1 needs you", "3 need you", or "3+
    need you" where the read is cut (`tui-signal`): on the launchpad
    across the person's projects, as its Needs you and its walk are
    (`Session::count_across`, `launchpad.md`, 5), on a chat the rail's
    count of the project (`Session::count`, `../../pages/needs-you.md`, 1);
    and, on the launchpad with that count, "n to walk" (`tui-hint`);
    while a notice stands, the notice alone, in place of them all
    (*The notice*).

  The chat's spend is the browser's alone.
- **A narrow terminal:** the header's address goes first (above); then,
  where the quiet line's two halves, two spaces between them and its
  padding are wider than the window's columns, it leaves out "n to walk"
  and its " · ", on a chat the pane's goal part and then its Ctrl-P
  words, each with its " · " (`design-session.md`, section 1), then the closed line
  and its " · ", and skein cuts what
  still does not fit from the left half's end; the rule above a chat's
  box leaves out the place, then the project (`chat.md`, section 1, *The rule above
  the box*). A notice goes before the line's own words (*The notice*).
- **The notice,** the user's request of 2026-11-09: a one-line notice
  of the TUI's, at the right end of the quiet line, never in the
  conversation. It is the place the box's state uses in front (`../box.md`),
  made general:
  - *What:* `temper_client_tui::Notice`, one of an unknown command's,
    "Unknown command /WORDS · /help lists them", WORDS the box's words
    after "/" (section 2); "Still signing in" (*Typed before the
    sign-in lands*); the theme taken's (section 2, *`/theme`*); and a goal
    proposed's (`design-session.md`, section 1, *`/goal`*). Its words are
    the TUI's own (`tui-signal`).
  - *Where:* the quiet line's right half, alone: the box's state,
    "playground", "paused · a message resumes it", the count and "n to
    walk" give way to it for its time, then stand again. It stands on
    the launchpad, the welcome and a chat alike, and on the sign-in
    screen not at all.
  - *How long:* 6 seconds, a newer one replacing it, timed by the host's
    dues (`../README.md`, 3, *The notice's time*); a send, a key or another screen
    leave it standing.
  - *A narrow window:* where the quiet line's left half, two spaces, the
    notice and its padding are wider than the window's columns, the
    notice goes before the line's own words: the notice first, at the
    row's start, then " · " and the left half, which skein cuts from its
    end.
  - *Not a notice:* a note of more than a line stays in the
    conversation, `/help`'s and `/log`'s among them, and so do `/brief`'s
    note, `/open`'s "Open ADDRESS in your browser.", which holds an
    address to copy, and the box's give up note, which stands for good
    (`../box.md`).

## 2. The commands

The browser terminal's (`../../pages/chat/terminal.md`, 1,
*Commands*), illustrative (the request's 02):

```
────────────────────────────────────────────────────────────────────────────────
❯ /
────────────────────────────────────────────────────────────────────────────────
❯ /model   Change the model, from the next turn
  /new     Start a new chat
  /chats   Open another chat
  /theme   Auto, light or dark
  /help    Commands and keys
  /quit    Close temper
```

- **Offered:** on the launchpad `/model` and `/effort`, then `/new`,
  `/chats`, `/theme`, `/help` and `/quit`; on a
  chat `/model`, `/effort`, `/draft`, `/pause`, `/close`, `/start-over`
  and `/goal`, then
  `/new`, `/chats`, `/theme`, `/help` and `/quit`; each of the first
  where the view says the browser's terminal offers it
  (`shared::offered`): `/model` while the picker is changeable, and
  `/effort` while it is and its chosen model has efforts, wherever
  `/model` is, as the browser's terminal offers it wherever its picker
  is (the user's request of 2026-11-08); `/draft` there only while the
  chat has a document or writes one (`Chat::draft`, `Chat::writing`),
  so it shows or hides the draft and never adds one; and `/goal` only
  while the document has a section with a heading too (`design-session.md`, section 1, *`/goal`*); with no project `/theme`,
  `/help` and `/quit`. Never the root's `/draft` on the launchpad, where
  the TUI's own `/draft` stands (below), nor `/font`,
  the browser's alone. On the launchpad `/model` and `/effort` change
  Home's composer's choice, which the chat its box starts takes
  (`Act::StartChat`, `../../pages/home.md`, 1). `/close` and `/start-over` are
  offered only on a chat with no delegate: a close that asks its
  delegates' fates is the web's sheet (`../../README.md`, section 5).
- **The TUI's own commands** (`../../../draft/surfaces.md`, 11.4), which the
  root's `Command` does not hold and which never reach the root as
  commands, as `/quit` and `/new` don't: offered on the launchpad with a
  project, `/draft` while the project offers drafts (`launchpad.md`, 7, *Start*), `/change`
  and `/ask` before the root's, and `/brief` while
  the header holds a brief, `/needs` while Needs you stands and `/open`
  after `/chats`; on a chat `/open` after `/chats`. So the launchpad's
  order is `/draft`, `/change`, `/ask`, `/model`, `/effort`, `/new`, `/chats`,
  `/brief`, `/needs`, `/open`, `/theme`, `/help`, `/quit`. `/theme` is
  the TUI's own wherever it is offered (*`/theme`*), in the place the
  root's held. Their words,
  the TUI's: "A design session that ends in a draft and a goal", "One
  change, run as the project's way says", "Ask temper",
  "The brief, whole", "Walk what needs you", "Open this on the web",
  "Auto, light or dark".
  - *`/draft`* on the launchpad, pressed or sent alone, leaves "/draft "
    in the box for the words; sent with words, `/draft WORDS` starts a
    design session as the browser's new chat with a draft does (`../../pages/README.md`, *A
    new chat*, *With a draft*): the root to `NewChat { draft: true }`,
    its words, `Act::StartChat`, so the domain sends WORDS, a blank line
    and "Write this up as a draft beside us." from template 6 where `me`
    lists it, and from the project's chat template otherwise; the chat
    opens as a chat started from the launchpad is, its pane at half (`design-session.md`, section 1). A refusal is drawn under the box, as `/change`'s.
  - *`/change` and `/ask`,* pressed or sent alone, leave "/change " or
    "/ask " in the box for the words; sent with words, `/change WORDS`
    starts a chat in the project, the one change's (the project's chat
    template, as the browser's "+ New chat" starts it: the root to
    `NewChat { draft: false }`, its words, `Act::StartChat`), opened as
    a chat started from the launchpad is, and run as the project's way
    says; `/ask WORDS` asks Ask temper, as the box's words do. A refusal
    is drawn under the box, as a send's, and the launchpad stays.
  - *`/brief`* writes `/brief`'s note (`launchpad.md`, 7).
  - *`/needs`* walks what needs the person, as `n` does.
  - *`/open`* opens the object in focus on the web, the opener given
    `http://HOST:PORT` and the page's path (`address::print`) as one
    argument, as the approval's tab is opened (`../../README.md`, 4.1): on the launchpad
    the selected row's page, a chat's (`/chats/T`) or the goal's
    (`/tasks/T`), else the project's Home (`/projects/P`); on a chat
    its page, or, where a goal was started from its draft, that goal's
    (`/tasks/T`, `design-session.md`, section 1, *A linked goal*). Only the page's address: never a link, a code or a
    secret. Where no browser opened (no opener, or it failed), the TUI
    notes "Open ADDRESS in your browser." until the next send or screen.
  - *`/chats N`,* sent on the launchpad, opens the Nth chat of Where you
    left off (`launchpad.md`, 4), whatever its project, as Enter on it
    does; a number with no chat there gives the notice an unknown
    command gives. `/chats` alone opens the root's list, the project's
    chats, newest first.
- **Their words:** each command's name and the browser terminal's words
  (`shared::description`), but the closing command's, and the TUI's own
  commands' (above).
- **`/quit`,** the user's request of 2026-11-09: the root's
  `Command::Exit` is named `/quit` in the TUI, `temper_client_tui::QUIT`,
  listed, completed and in `/help`'s note under that name, its words
  "Close temper", in front "Close temper and stop the box" (`../box.md`), and
  its question while work runs as `../box.md` asks it. `/exit` sent whole is
  the same command, never listed or completed, and its beginnings begin
  no command of its own. The browser's terminal keeps `/exit`: the
  view's `shared` words are unchanged, and the TUI names the command
  itself.
- **The list:** the box's words beginning with "/" and holding no white
  space open the commands under the box in the quiet line's place, the
  region `tui-list`: an options menu after the box (skein's `ui/README.md`,
  3.2, *options*), named "Commands", with no head, each command offered
  whose name begins with the words, in the order *Offered* and *The
  TUI's own commands* give, its name and, as its description, its words (`tui-command`), the
  first active. While it stands the box keeps the focus: Up and Down
  move the active command, Enter presses it, Tab completes its name
  (skein's `ui/screen.md`, 6.2), Esc closes the list and clears the
  words (`terminal::Act::Dismiss`). With no command beginning with the
  words, no list stands.
- **Pressed,** or sent whole, which with its list standing is the same
  press: `/quit` closes the TUI (`../README.md`, 4) and never reaches the root;
  `/new` opens the launchpad (`../README.md`, 3), whose next question starts a new
  Ask temper chat; `/goal` opens the root's sheet by `Act::OpenStart`
  and its prompt (`design-session.md`, section 1, *`/goal`*), never `Run`; any
  other is the root's `terminal::Act::Run {
  command }`, so that its list opens, its note is written or its act is
  taken, as in the browser's terminal. The box is cleared.
- **Words no command of the TUI's names:** words that begin with "/",
  hold no white space and begin no command the TUI offers there are
  cleared from the box (`terminal::Act::Dismiss`), nothing is sent, and
  the notice says "Unknown command /WORDS · /help lists them" (section 1, *The
  notice*), in the TUI's own words, never a note in the conversation.
  So `/font`, which the root knows, is unknown here.
- **A command's list,** under the box in the quiet line's place, an
  options menu after it, its head a line of its own at the row's start
  (`tui-list-head`), its choices its items, the current one active;
  taking one closes it, Esc closes it, and words typed close it (the
  root's):
  - `/model`: the view's head, "Model. A change applies from the next
    turn."; the picker's models, each its shown name and, as its words,
    what pays and "default" on the default's (`shared::models`), taking
    one `choice::Act::Model`. A provider with no model, which the
    browser's terminal lists with "Connect a plan or key ›", is left
    out: connecting an account is the web's (`../../../draft/tui.md`,
    section 2).
  - `/effort`: "Effort. A change applies from the next turn."; the
    chosen model's efforts (`shared::efforts`), `choice::Act::Effort`.
  - `/chats`: "Chats"; the person's live chats in the project, newest
    activity first, the open one left out, each "TN" and its title
    (`shared::chats`), read when the list opens as the browser's is;
    taking one is `Act::Go` to its `Chat { task }`, followed as `../README.md`, 3
    says.
  - `/close` and `/start-over`: the confirmation's question as the head
    and its two acts as choices, "Close" or "Start over" first, then
    "Keep it open" or "Keep this one" (`shared::confirmation`):
    `Act::Confirm` and `Act::Cancel`.
- **`/help`'s note:** the commands the TUI offers there, each name
  padded with spaces to the longest and two more, then its words; then
  its keys, a line each: "Enter sends", "Tab completes a command", "PgUp
  and PgDn read back", "Esc closes a list", on the launchpad while Needs
  you stands "n walks what needs you", and "Ctrl+C closes temper". The
  root keeps the note (`terminal::Note::Help`); its words are the TUI's.
- **`/theme`,** the user's request of 2026-11-09, since a flip on the
  terminal's own ground showed as almost nothing: the TUI's own command,
  `Own::Theme`, never the root's `Command::Theme` in the TUI. Pressed,
  or sent whole or begun, it opens the theme's list under the box in
  the quiet line's place, as a command's list stands, the box's words
  cleared: its head (`tui-list-head`) "Theme. Auto follows your
  terminal."; then "Auto", its words what Auto reads now, "light, as
  your terminal says", "dark, as your terminal says", or "dark: your
  terminal doesn't say" where the terminal says no scheme (`../theme.md`, *The
  scheme*); "Light"; and "Dark", with no words (`tui-option`); the
  theme chosen active. While it stands the box keeps the focus: Up and
  Down move the active choice, Enter takes it, Esc closes the list, and
  words typed close it.
  - *Taking one,* `Event::Theme { choice }`: the TUI draws in it at
    once (`../README.md`, 3, *The look*), keeps it for the next run (`../theme.md`, *The theme
    chosen*), lowers the list, and gives the notice "Theme: auto, light
    as your terminal says.", "Theme: auto, dark as your terminal
    says.", "Theme: auto, dark: your terminal doesn't say.", "Theme:
    light." or "Theme: dark." (section 1, *The notice*).
  - *Its state:* the TUI's, `Tui::theme_list`, as the quit question's
    is (`../box.md`), and never the root's: the root's lists and notes don't
    hold it.
  - *The browser's terminal's* `/theme` is the root's and flips, as
    before (`../../pages/chat/terminal.md`, 1).
