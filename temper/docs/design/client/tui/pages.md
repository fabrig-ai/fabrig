# The TUI's pages

Three screens, each a tree of the TUI's view, built from the root and
the TUI's own state: signing in, while the link is signed out; once
signed in, or while the TUI signs in from the engine's link (below), the
launchpad, the root at `Home { project }` (`launchpad.md`); and a chat, the root at
`Chat { task }` (`README.md`, 3, *Its pages*), with its draft beside it where it
has one (*A design session*, below). Signing in has a framed top, the
region `tui-top`, titled "temper", and a bottom row, the region
`tui-bottom`, spread: what closes the TUI, and the link's state. The
launchpad and a chat share their foot: the box, and under it the quiet
line or a list (*The foot*, below). The launchpad and a chat are drawn
from the user's hand-made frames of the look request `tui-launchpad`
(2026-11-08), at 80 by 24 in the dark theme; their words are the
browser's chat terminal's (`../pages/chat/terminal.md`, 1), and a chat reads
as `../../draft/chat-reading.md` says. Every frame below is illustrative:
its words and look are refined on the recording (`../../draft/tui.md`,
section 5). In front of a box, the welcome stands in the no-project
launchpad's place, the playground's project says so, Ctrl+C and
`/quit` ask first while work runs, and the box's state and `/log` join
the foot and the commands (`box.md`).

- **The TUI's state,** `temper-client-tui`'s `Tui`, holds what the root
  does not: the address, the link field's value, the no-project command
  composer's value, the sign-in's progress
  (asked, answered, lost, or the TUI's own refusal of what was pasted),
  whether it signs in from the engine's link and whether that ended,
  whether the link was signed out after a sign-in, and whether the
  credential's last write failed; the window's columns, for what a
  narrow row leaves out; the question set aside by "Tell it something
  else" (*The chat's question*); the notice standing (*The notice*);
  the theme chosen, the terminal's scheme Auto reads, and whether the
  theme's list stands (*`/theme`*); the left-off chat selected on the
  launchpad, by its task (`launchpad.md`, 4); the reply whose choices
  were set aside, and the chat a goal's start was asked from (*A design
  session*); and whether it is on a
  box (`README.md`, 2, *On a box*). The root stays the one source of the
  link's state, of `me`, of the pages and their reads, of the box's
  words with a project and of the terminal's look, lists and notes.
- **Its words:** where the TUI says what `temper-client-view` says, it
  says the view's phrases (`temper-client-view`'s `words`): the link's
  states, "Live", "Connecting", "Offline", "Signed out" and "Out of
  date" (`../view.md`, 1); a link used, expired or not this temper's, and the
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
  "Signed out", "Out of date", each in its token's paint (`theme.md`). On the
  launchpad and a chat it ends the quiet line.

**Signing in**, illustrative:

```
╭─ temper ─────────────────────────────────────╮
│ Sign in this terminal to 127.0.0.1:8367      │
│ Paste the link temper printed, then Enter.   │
│ Link: ••••••••••••••••••••••••••             │
│ Signing in…                                  │
╰──────────────────────────────────────────────╯

 Ctrl+C to quit                     ● Signed out
```

- **The paste:** one text field, "Link", a secret, drawn masked (skein's
  `ui/screen.md`, 5.4), that submits on Enter, its placeholder the
  link's shape at the TUI's address, and the focus asked for it as the
  screen opens. A paste fills it whole (skein's `ui/screen.md`, 6.2). A
  `q` types in it, so the bottom row says "Ctrl+C to quit".
- **Enter** checks the field with jig's parser of the printed link
  (`jig-client-protocol-http`'s `printed`, under the default mapping's
  prefix `/jig`; `../README.md`, section 4), and that its `HOST:PORT` is the TUI's
  address:
  - not a link: "That is not a sign-in link: paste the whole link
    temper printed.";
  - a link for another address: "That link is for HOST:PORT. To sign
    in there, run temper http://HOST:PORT."; nothing is sent;
  - a link for this address: the host asks the root `SignIn { with:
    Link { secret, held } }`, held the credential the directory holds,
    as `temper-box open` does (`../README.md`, section 4), and the field is emptied; the
    secret stays in the TUI's state only until then.
- **While it signs in:** "Signing in…". `jig-client` sends the link
  again by itself, after a backoff, when the engine answers not ready
  or busy, or no answer comes (jig's `client/domain/client/README.md`, section
  9), and the TUI says which: "Signing in: the engine is busy; trying
  again.", "Signing in: the engine is starting or stopping; trying
  again.", "Signing in: no answer from the engine; trying again.".
- **Refused:** a link used, expired or not this temper's, or the
  sign-ins full, in the view's words (`../view.md`, 1); a link refused otherwise,
  "This link was refused. Use the fresh one temper printed.". The field
  waits for another.
- **Room for the browser's approval:** the paste is the last way of
  signing in on the screen, under its own lines, so that the approval
  (`../../draft/tui.md`, 4.2) is laid above it.
- **Signed out elsewhere:** the link signed out after a sign-in, by any
  means (a sign-out from another client, the sign-in ended or expired,
  a held credential the engine does not know): the token is erased
  (`README.md`, 3), and the screen shows again under the view's "Signed out.".

**Signed in from the engine's link** (`README.md`, 2, *The engine's link, kept at
rest*), with nothing pasted and no key pressed:

- **When:** the link is signed out, the TUI has not closed, it has an
  engine's link file to read (`Start.local`), and its note for the
  address doesn't say signed out: at its start with no credential kept,
  and again when a sign-in expires (*Expired is not signed out*, below).
  The host asks the duty `local` and, while it signs in from the file,
  the view draws the launchpad's "Connecting to HOST:PORT", the state
  "Connecting": no sign-in screen is drawn and no approval is asked.
- **The file read:** none, a link that `jig-client-protocol-http`'s
  `printed` doesn't read under the prefix `/jig`, or a link for another
  address than the TUI's, and the TUI falls back. A link for its address:
  the host asks the root `SignIn { with: Link { secret, held: None } }`,
  as for a paste (`../README.md`, section 4), and keeps nothing of the secret but its
  SHA-256, to tell a fresh link from it.
- **Signed in:** the launchpad, as by a link pasted; the credential
  kept (`README.md`, 3), and the note `Until` the sign-in's expiry.
- **Answered `link_used` or `link_expired`:** the engine draws the next
  local link when a TUI spends one, and a sign-in by a spent or expired
  one draws a fresh one when none is live, which the engine keeps in the
  file and never prints (jig's `protocol/client/sign-in.md`, *The
  local link*; `README.md`, 2). So the host reads the file again a second later,
  at most twice for one need, and signs in by a link whose digest
  differs from the one answered; the same link read again waits another
  second, within those two reads. This is the way when the file's link
  expired, or another TUI spent the link this one read. A browser signed
  in by the printed link, before or after, leaves the file's link
  live.
- **Any other end** (refused, `link_unknown`, not ready, busy, full, no
  answer; but in front, not ready and busy (`box.md`) wait for jig's resend),
  the reads spent, or the file read finding none: the TUI falls
  back to the sign-in screen as it is with no file, which asks the
  browser's approval (`../README.md`, 4.1) and takes the paste. It says nothing of the
  file: no refusal line is drawn for the file's link, and a fallback
  after an expiry shows the screen under the view's "Signed out.", as
  any sign-in ended. The approval's take replaces a link's sign-in that
  `jig-client` still sends again by itself (`../README.md`, 4.1, *A pasted link, sent*).
- **Expired is not signed out.** The engine answers both alike, the
  link signed out, so the note tells them apart, from the expiry the
  sign-in's answer gave (`README.md`, 3, *The note*), on the engine's own machine's
  wall clock. The link signed out after a sign-in, or a held credential
  refused, at or past the note's `Until` less a minute's slack, is an
  expiry: the TUI signs in from the file on its own, as above. Before it,
  the sign-in ended: a sign-out here or elsewhere, or an engine at that
  address that never made it. The TUI notes `SignedOut` and never signs
  in from the file at that address again until a person's act, the paste
  or the browser's approval, signs it in, whose sign-in notes `Until`
  once more. With no note for the address, the TUI has not signed in
  there, and the file is read.
- **Never shown:** the file's secret is never drawn, logged or kept in
  the TUI's state; a recording keeps it in the person's private file, and
  `add` redacts it (`journeys.md`, section 5).

**The launchpad**, its header, Where you left off, Needs you and its
walk, Start and `/brief`'s note, across the person's projects, is
`launchpad.md`'s.

**The foot,** the launchpad's and a chat's, from the bottom up: the
quiet line, or a list in its place; the box between two rules; on a
chat, the running line above them.

- **The running line,** the region `tui-working`: on a chat, while the
  browser's terminal draws its running line (`../pages/chat/terminal.md`, 1, *The running line*;
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
  then two spaces and the terminal's phrase, as `../../draft/surfaces.md`,
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
    for the launchpad's left-off chats (`launchpad.md`, 4), and
    `q` types: the first screen's "q to quit" is gone. Ctrl+C closes the
    TUI from anywhere (`README.md`, 4).
  - *Enter:* before the sign-in lands, the words kept and the notice
    "Still signing in" (*Typed before the sign-in lands*); on the
    launchpad's empty box with a left-off chat
    selected, that chat opened (`launchpad.md`, 4); else the box's
    words, by what they are, in this order: a
    command's beginning, as *The commands* say; on a chat whose question
    stands, its answer in words (*The chat's question*); with no
    project, nothing; otherwise sent: on the launchpad `Act::StartChat`,
    which asks Ask temper in the project and opens the chat it makes
    (`../pages/home.md`, 1, *In the TUI*), on a chat `Act::Send`. A refusal or a
    failure the root keeps for the send is drawn under the box, in
    `tui-refused`, in the view's words.
  - *No new line:* a terminal reads Shift+Enter as Enter, so the box
    takes one paragraph; words wrap as the box's lines (`../README.md`, section 5).
- **Typed before the sign-in lands,** the user's request of 2026-11-09,
  after a key typed on the welcome while the TUI signed in was drawn
  nowhere and then thrown away:
  - *Whenever the box is drawn, keys go to the box:* the welcome in
    front, the launchpad's "Connecting to HOST:PORT" while the TUI signs
    in from the engine's or the handed link, the launchpad and a chat.
    The link's field takes keys only while the sign-in screen is drawn,
    `temper_client_tui::sign_in_drawn(root, tui)`, the one test the
    view's choice of screen and the host's routing of a key both read
    (`README.md`, 3, `Typed`).
  - *The sign-in never clears the box:* the link's changes leave its
    words (*The box*), and the host's going to the launchpad from the
    start's address once `me` names a project (`README.md`, 3, *Its pages*) carries
    the no-project box's words into Home's composer, `Act::ChatWords`,
    so words typed before `me` was read stay in the box.
  - *Before it lands,* while the link is signed out or the root has not
    yet read `me` (`temper_client_tui::landing(root)`): the box's words
    open no list, and Enter keeps them, sends and runs nothing, and the
    notice says "Still signing in" (*The notice*). Once it has landed,
    the words are acted on at the person's next Enter, as any words are.
- **The quiet line,** the region `tui-bottom`, spread, as the sign-in's
  bottom row (`../../draft/surfaces.md`, 11.1: the project, the model and
  effort, the link's state, and "N need you · n to walk"):
  - *at its left,* the project's name (`tui-hint`), " · ", the picker's
    closed line, "Claude Opus `../pages/accounts.md`, 2 · high" (`shared::closed_line`,
    `tui-hint`), " · ", then "●" and the link's state in its token's
    paint (`pages.md`, *The link's state*); with no picker (no project, no
    usable account) the closed line and its " · " are left out, and with
    no project the project and its " · " too; in a design session, the
    project's name, " · ", "drafter" (`tui-hint`), " · " and the closed
    line, the link's state after them only while it is not Live (*A
    design session*, *The quiet line*);
  - *at its right,* each left out when absent, " · " between two (`tui-faint`):
    in front of a box its state and "playground" (`box.md`); "paused · a
    message resumes it" on a chat the person paused (`tui-hint`); on a
    chat whose draft can show, the pane's parts (*A design session*,
    *The quiet line*); while
    something needs the person, "1 needs you", "3 need you", or "3+
    need you" where the read is cut (`tui-signal`): on the launchpad
    across the person's projects, as its Needs you and its walk are
    (`Session::count_across`, `launchpad.md`, 5), on a chat the rail's
    count of the project (`Session::count`, `../pages/needs-you.md`, 1);
    and, on the launchpad with that count, "n to walk" (`tui-hint`);
    while a notice stands, the notice alone, in place of them all
    (*The notice*).

  The chat's spend is the browser's alone.
- **A narrow terminal:** the header's address goes first (above); then,
  where the quiet line's two halves, two spaces between them and its
  padding are wider than the window's columns, it leaves out "n to walk"
  and its " · ", on a chat the pane's goal part and then its Ctrl-P
  words, each with its " · " (*A design session*), then the closed line
  and its " · ", and skein cuts what
  still does not fit from the left half's end; the rule above a chat's
  box leaves out the place, then the project (*A chat*, *The rule above
  the box*). A notice goes before the line's own words (*The notice*).
- **The notice,** the user's request of 2026-11-09: a one-line notice
  of the TUI's, at the right end of the quiet line, never in the
  conversation. It is the place the box's state uses in front (`box.md`),
  made general:
  - *What:* `temper_client_tui::Notice`, one of an unknown command's,
    "Unknown command /WORDS · /help lists them", WORDS the box's words
    after "/" (*The commands*); "Still signing in" (*Typed before the
    sign-in lands*); the theme taken's (*`/theme`*); and a goal
    proposed's (*A design session*, *`/goal`*). Its words are
    the TUI's own (`tui-signal`).
  - *Where:* the quiet line's right half, alone: the box's state,
    "playground", "paused · a message resumes it", the count and "n to
    walk" give way to it for its time, then stand again. It stands on
    the launchpad, the welcome and a chat alike, and on the sign-in
    screen not at all.
  - *How long:* 6 seconds, a newer one replacing it, timed by the host's
    dues (`README.md`, 3, *The notice's time*); a send, a key or another screen
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
    (`box.md`).

**The commands**, the browser terminal's (`../pages/chat/terminal.md`, 1,
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
  while the document has a section with a heading too (*A design
  session*, *`/goal`*); with no project `/theme`,
  `/help` and `/quit`. Never the root's `/draft` on the launchpad, where
  the TUI's own `/draft` stands (below), nor `/font`,
  the browser's alone. On the launchpad `/model` and `/effort` change
  Home's composer's choice, which the chat its box starts takes
  (`Act::StartChat`, `../pages/home.md`, 1). `/close` and `/start-over` are
  offered only on a chat with no delegate: a close that asks its
  delegates' fates is the web's sheet (`../README.md`, section 5).
- **The TUI's own commands** (`../../draft/surfaces.md`, 11.4), which the
  root's `Command` does not hold and which never reach the root as
  commands, as `/quit` and `/new` don't: offered on the launchpad with a
  project, `/draft` while the project offers drafts (`launchpad.md`, 7), `/change`
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
    design session as the browser's new chat with a draft does (`../pages/README.md`, *A
    new chat*, *With a draft*): the root to `NewChat { draft: true }`,
    its words, `Act::StartChat`, so the domain sends WORDS, a blank line
    and "Write this up as a draft beside us." from template 6 where `me`
    lists it, and from the project's chat template otherwise; the chat
    opens as a chat started from the launchpad is, its pane at half (*A
    design session*). A refusal is drawn under the box, as `/change`'s.
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
    argument, as the approval's tab is opened (`../README.md`, 4.1): on the launchpad
    the selected row's page, a chat's (`/chats/T`) or the goal's
    (`/tasks/T`), else the project's Home (`/projects/P`); on a chat
    its page, or, where a goal was started from its draft, that goal's
    (`/tasks/T`, *A design session*, *A linked goal*). Only the page's address: never a link, a code or a
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
  "Close temper", in front "Close temper and stop the box" (`box.md`), and
  its question while work runs as `box.md` asks it. `/exit` sent whole is
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
  press: `/quit` closes the TUI (`README.md`, 4) and never reaches the root;
  `/new` opens the launchpad (`README.md`, 3), whose next question starts a new
  Ask temper chat; `/goal` opens the root's sheet by `Act::OpenStart`
  and its prompt (*A design session*, *`/goal`*), never `Run`; any
  other is the root's `terminal::Act::Run {
  command }`, so that its list opens, its note is written or its act is
  taken, as in the browser's terminal. The box is cleared.
- **Words no command of the TUI's names:** words that begin with "/",
  hold no white space and begin no command the TUI offers there are
  cleared from the box (`terminal::Act::Dismiss`), nothing is sent, and
  the notice says "Unknown command /WORDS · /help lists them" (*The
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
    out: connecting an account is the web's (`../../draft/tui.md`,
    section 2).
  - `/effort`: "Effort. A change applies from the next turn."; the
    chosen model's efforts (`shared::efforts`), `choice::Act::Effort`.
  - `/chats`: "Chats"; the person's live chats in the project, newest
    activity first, the open one left out, each "TN" and its title
    (`shared::chats`), read when the list opens as the browser's is;
    taking one is `Act::Go` to its `Chat { task }`, followed as `README.md`, 3
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
  terminal doesn't say" where the terminal says no scheme (`theme.md`, *The
  scheme*); "Light"; and "Dark", with no words (`tui-option`); the
  theme chosen active. While it stands the box keeps the focus: Up and
  Down move the active choice, Enter takes it, Esc closes the list, and
  words typed close it.
  - *Taking one,* `Event::Theme { choice }`: the TUI draws in it at
    once (`README.md`, 3, *The look*), keeps it for the next run (`theme.md`, *The theme
    chosen*), lowers the list, and gives the notice "Theme: auto, light
    as your terminal says.", "Theme: auto, dark as your terminal
    says.", "Theme: auto, dark: your terminal doesn't say.", "Theme:
    light." or "Theme: dark." (*The notice*).
  - *Its state:* the TUI's, `Tui::theme_list`, as the quit question's
    is (`box.md`), and never the root's: the root's lists and notes don't
    hold it.
  - *The browser's terminal's* `/theme` is the root's and flips, as
    before (`../pages/chat/terminal.md`, 1).

**A chat**, illustrative (the request's 03, with the look request
`tui-chat-title-below`'s rule):

```
❯ What landed overnight?

▸ Listed changes, read 3 changes  6 s

✦ Three changes landed between 22:10 and 06:40:

  1. csv-export: the report's columns, the first of its three tasks.
  2. csv-export: the export button on the report page.
  3. fix-login: a session no longer ends when the forge restarts.

  csv-export's last task waits on you: whether the file separates its
  values with a comma or a▌

✦ Working
────────────────────────────────────────────── Hello / What landed… T2 on box ──
❯ Reply
────────────────────────────────────────────────────────────────────────────────
 / for commands · 1 needs you                   Claude Opus 5.5 · high · ● Live
```

- **Its chat:** the root's, `Chat { task }`: its task, its thread, the
  call under way, its delegations and its proposals, as the root's chat
  page holds them (`../pages/README.md`, `Chat { task }`; `pages().chat()`).
- **The rule above the box** carries the chat's title, aligned to its
  end, as Claude Code's is: the box region's level-1 heading, which
  skein draws in its top rule (skein's `ui/screen.md`, 5.2), its parts
  each aligned to the end (`tui-rule-title`), so that they end two
  glyphs across before the rule's end: the project's name (`tui-name`),
  " / " (`tui-faint`), the title (`tui-name`), " T2", the chat's number,
  and " on " (`tui-hint`), then the engine's place (`tui-address`). The
  look request `tui-chat-title-below` (2026-11-08) moved it there from a
  top line, freeing the screen's first row for the conversation; no
  "✦" goes with it.
  - *The title:* the chat's short heading as the browser's bar shows it
    (`../pages/chat/README.md`, 1, *A chat's heading is short*; `shared::heading`), cut to at
    most 20 characters: whole where it has 20 or fewer; else its words
    from the start, as many as fit with "…" after them in 20, its
    trailing spaces dropped, then "…" ("What can you tell…"); where its
    first word alone does not fit, its first 19 characters and "…". A
    character is a Unicode scalar value.
  - *The place,* `temper_client_tui::place(address, boxed)`, the one
    function that words it, which `temper box`'s own header may call
    too: on a box (`README.md`, 2, *On a box*) "box" where the address's port is
    8367, `temper-box init`'s default, and "box:PORT" at another port; not
    on a box, `HOST:PORT`, the address as it is.
  - *A narrow terminal:* while the title's parts and six columns are
    wider than the window's columns, the place and its " on " are left
    out first, then the project and its " / "; skein leaves out what
    still does not fit, whole (skein's `ui/screen.md`, 5.2), and the rule
    stands bare.
  - *While a list stands* under the box (the commands, a question), the
    rule above the box keeps the title; on the launchpad the box's rule
    has no title, and its framed header stays as it is.
- **The conversation,** in the body, from the screen's first row: a log (skein's `ui/README.md`, 3.2) with
  the chat's end request (`Chat::to_end`), holding one list, an item an
  entry, in the thread's order as the browser's terminal reads it (`../pages/chat/terminal.md`, 1, *The
  conversation*), with a blank row between two entries
  (`tui-spaced`). Each
  entry's glyph and its words' indent are its token's look (`theme.md`):
  - *the person's words,* the chat's request and each words line a
    person sent: "❯", then each paragraph (`tui-said`): on the band
    across the row, the mark dim, where the terminal answered its
    background; else in the dim colour after an accent "❯", as Claude
    Code draws its prompts (`theme.md`, *The band*);
  - *a delegate's words:* "✦", "TN · " and its words (`tui-agent`);
  - *a run of calls,* as the browser's terminal reads one (`../pages/chat/conversation.md`, 1, *its
    calls*): "▸", its name and, two spaces after, its description, the
    run's time, "1 failed · 29 s" in `tui-run-failed`, or the call under
    way with "…" while it runs (`tui-run`), from the view's run line
    (`shared::Run`, `shared::RunLine`), so that it counts what has ended
    and names the call being made. Its rows are not drawn (`../README.md`, section 5);
  - *the agent's texts:* "✦", then each text read as Markdown by
    skein-ui's reader, its blocks two columns in (`tui-agent`), inside
    a group under `tui-bullets`, whose mark "•" takes the place of the
    entry's "✦" before every unnumbered item under it, nested ones too
    (skein's `ui/screen.md`, 3): a Markdown bullet is "•", never the
    agent's own mark "✦" (the user's request of 2026-11-08); numbered
    lists are numbered, as skein's reader orders them. The words being
    written are read inside the same group;
  - *the words being written:* the turn being written, whole, read by
    `markdown::read_growing` (skein's `ui/markdown.md`), its end a
    small block after its last word, a reversed space (`tui-end`), in
    the browser's "▍" place. A closed block is read again to the same
    nodes and stays as it was; the committed turn draws the same blocks
    in the same place, and only the block goes;
  - *the three dots:* while the chat works and the turn being written
    has no word and no call yet, "✦" and "…" (`tui-dots`), as the
    browser's dots stand;
  - *a card,* one line each, after the conversation's lines in the
    order the page holds them: a delegated task (`Chat::delegations`,
    its task as `Chat::delegation` reads it) "✦", "TN · KIND", its title
    and its state (`shared::delegation_line`); a proposal in the chat's
    subtree (`Chat::cards`) "✦", "Proposal" (`tui-signal`) and its title
    (`shared::proposal_line`). A card carries no act: a proposal is
    accepted on the web. Placing each at the call that made it is later
    TUI work (`../README.md`, section 5);
  - *a view,* printed, not drawn (`../../draft/surfaces.md`, 11.5; the
    user, 2026-11-08): an agent's committed text is split at its `view`
    blocks by the domain's `views::parts`, as the browser's is (`../pages/chat/conversation.md`, 3),
    its text parts read as Markdown and each view printed in its place,
    inside the entry's group, as lines of the terminal's own text with no
    picture, from the chat page's read of its address
    (`Chat::view`, `../pages/chat/conversation.md`, 3, *Read when shown*). The words being written are
    read as they are until the turn commits. `view::printed`, the one
    function that prints a view, is what a chat's document calls too
    where the TUI draws one:
    - *a path:* a stop a paragraph, in the derivation's order (the
      wire's `stops`): its mark, then a space and its words as the
      derivation gives them, which the TUI never spells (they change
      with the page How work runs, `../../draft/surfaces.md`, 13). The mark
      is the web's for its actor: a ring `○` for a person
      (`tui-signal`, the person's colour alone), a four-point star `✦`
      for an agent (no paint, the ink, as `../../draft/surfaces.md`, 13,
      turns it), a small square `▪` for code (`tui-hint`). Code's stops
      stand four columns in, as they stand below the line on the web,
      their words in `tui-hint`. A branch's bar closes its part, four
      columns in: `┃` (`tui-faint`) and the branch's name, its words.
      Illustrative, a way:

      ```
      You approve
      An agent makes each change; nothing reaches main until you approve it.

      ○ you ask
      ✦ produce
          ▪ check
      ✦ repair
          ▪ check
      ○ you approve
          ▪ update
      ✦ resolve
          ▪ merge
          ┃ main
      ```

    - *a way:* its name (`tui-name`), and " · now" (`tui-hint`) on the
      way the project runs, as the browser says it (`ViewNow`); its line
      (`tui-hint`); a blank row; then its path;
    - *a proposal:* "Before" (`tui-head`) and its first path, a blank
      row, then "After" (`tui-head`) and its second, the view's words
      (`ViewBefore`, `ViewAfter`);
    - *a charter or a rule:* its card: its `name` (`tui-name`), its
      `who` (`tui-hint`), then its `facts` as terms, each its label and
      its value (skein's `Term` and `Description`);
    - *a `not_drawn`:* its words, a paragraph;
    - *asking or failed:* "Drawing ADDRESS…" (`tui-hint`) until the read
      answers, then the view's "Couldn't draw ADDRESS." where it was
      refused or failed, the browser's words (`ViewAsking`,
      `ViewFailed`);
    - *nothing to press:* a printed view carries no act, and a stop's
      card stays the web's (`../pages/ask-temper.md`, 1, Ask temper's page).
  - *not drawn,* as in the browser's terminal: the speakers' names, the
    turns' times, a turn's opaque blocks, a run's beginning and end;
    and, the TUI's, a document's write, since the pane shows the draft
    and marks what the turn wrote (*A design session*), and "Being
    sent" and "Sent", the conversation showing the words.
  The terminal's notes follow the conversation, each in `tui-note`.
- **Reading back:** PgUp and PgDn page the body by its rows while the
  box keeps the focus (skein's `ui/screen.md`, 6.2); the body follows the
  conversation's end as it grows, until the person pages up, and again
  once they page down to it (`../view.md`, 2 there). The window is the one the
  chat's page reads as it opens: earlier pages are not read as the head
  nears, since skein's screen reports no log edge yet (its section 10),
  and there is no "Latest ↓" (`../README.md`, section 5).
- **The foot:** the running line, the box, "Reply", and the quiet line,
  "paused · a message resumes it" on a chat the person paused. On a
  chat that has ended the box takes commands alone, and words that are
  no command note "This chat is closed. /new starts another.", the
  root's note (`../pages/chat/terminal.md`, 1, *The box*).

**The chat's question**, illustrative (the request's 04):

```
❯ Answer it with a comma

────────────────────────────────────────────── Hello / What landed… T2 on box ──
❯
────────────────────────────────────────────────────────────────────────────────
Answer csv-export's question with "Comma"?
❯ 1. Answer Comma
  2. Answer Semicolon
  3. Tell it something else
```

- **Which:** while a chat waits on the person, the root reads its
  subtree's questions (`../pages/chat/terminal.md`, 1, *A goal's question*), and the TUI draws the
  first the browser's terminal would, one the person may answer
  (`shared::question`), in the region `tui-list`, while the box is
  empty, no other list stands and the person has not set it aside.
- **Its list:** an options menu after the box, its question line as the
  head (`tui-list-head`), then its parsed choices numbered "1. ", "2. ",
  the recommended one first and active and the others in their order,
  then "Tell it something else". With no parsed choice, its question
  line and "Tell it something else" alone.
- **Choosing** answers it as Needs you does (`needs::Act::Choose`, the
  choice's words).
- **"Tell it something else,"** or Esc, or words typed, sets the list
  aside for that question: the box shows "Reply" again, and while the
  question stands, Enter on the box's words that are no command answers
  it in those words, as Needs you's reply does (`needs::Act::OpenReply`,
  `Act::CardWords`, `needs::Act::SendReply`), and clears the box. A
  refusal is drawn under the box in the view's words. Another question
  draws its list again.
- **Departs from the browser's terminal,** which keeps the choices'
  order, marks the recommended one selected where it stands and ends
  with "Later": the TUI puts the recommended choice first and "Tell it
  something else" last, as `../../draft/tui.md`, section 3 asks, and has no
  Later, since the box is there for an answer in the person's words.

**A design session**, illustrative (`tui/whole-story/design-session.html`,
at 80 by 24, the drafter's choices set aside by the "1" typed):

```
❯ Snapshots: can vsr's state transfer   │ raft-to-vsr.md
  carry cowsql's WAL pages?             │
                                        │ 4. Snapshots and state transfer  new
✦ Mostly. vsr's checkpoint is the       │
  host's to make, so cowsql keeps its   │ 4.1 cowsql keeps its WAL page
  page format. I've written it as 4.1   │ format; vsr's checkpoint is the
  and 4.2, beside.                      │ host's.
                                        │
▸ Read 6 files in vsr, 4 in cowsql  41 s│ 4.2 A witness keeps the log and
                                        │ never a checkpoint.
✦ One point is yours. While a cluster   │
  moves, run raft and vsr side by side, │ 5. What changes for users
  or move it at once?                   │
                                        │ 5.1 Nothing in the SQL API.
  1. Move it at once: no mixed state    │
  2. Side by side, raft the fallback    │
  3. Tell me something else             │
                                        │
───────────────────────────────────────────── cowsql / Raft to vsr T12 on box ──
❯ 1
────────────────────────────────────────────────────────────────────────────────
 cowsql · drafter · Opus 5.5 · xhigh                 pane: half · Ctrl-P widens
```

- **What it is:** a chat whose chip's executor is template 6's charter,
  `pages::chats::drafting(me, project, executor)` (`../pages/README.md`, *A design
  session*; `../../domain/agent/agents.md`, 4), read as any chat (*A chat*,
  above), with its draft beside the conversation, the drafter's
  questions as prompts, `/goal`, and a quiet line of its own
  (`../../draft/surfaces.md`, 11.2; `../../draft/tui-design-session.md`). Any
  other chat with a document has the pane, its marks, its replies'
  choices, `/goal` and the quiet line's pane parts too, but not
  "drafter".
- **Starting one:** `/draft WORDS` on the launchpad (*The TUI's own
  commands*), template 6 where `me` lists it and a chat otherwise, as
  the browser's new chat with a draft; it opens with the pane at half,
  as the domain's start leaves it (`Mode::Split`, `../pages/README.md`, *A new chat*).
- **Sittings:** a session is kept (`../../domain/agent/agents.md`, 4). Opened
  again, from Where you left off (`launchpad.md`, 4), `/chats N` or
  `/chats`, it is the root's chat as any chat is, its conversation and
  draft read as they stand, and the domain opens a design session with
  its pane at half (`../pages/README.md`, *A design session*); Where you left
  off says ", a design session" for one.
- **The pane's states,** the chat page's mode as the root keeps it
  (`pages::chat::Chat::mode`, `../pages/README.md`, *Its mode*), of which the TUI keeps
  nothing: `Mode::Chat`, folded, the conversation alone; `Mode::Split`,
  half, the conversation in the body and the draft in the side region
  `tui-pane` beside it (`theme.md`; skein's `ui/screen.md`, 5.2, *A side
  region*); `Mode::Draft`, all, the draft in the body in the
  conversation's place, the conversation not drawn. The foot (the
  running line, the box, a list or the quiet line) stands under each.
  The pane can show while the chat has a document or writes one
  (`Chat::draft`, `Chat::writing`); with neither, every mode draws as
  folded.
  - *Too narrow for two halves:* below 80 columns,
    `temper_client_tui::HALVES`, half shows as all and the quiet line
    says "pane: all": at 80 each half keeps 38 columns of words, the
    conversation's after its two-column indent and the pane's after its
    divider and padding, the narrowest the halves read at.
  - *Ctrl-P* cycles them on a chat whose pane can show, whatever the box
    holds and whatever list stands: the host reads the key before
    skein's key reader, as it reads `n` (`temper_client_tui::pane_key(view)`,
    from the tree last built), and the TUI's event is the root's
    `Act::Mode` with the next state after the one shown: folded to half,
    or to all where too narrow; half to all; all to folded. With no pane
    to show, Ctrl-P does nothing.
  - *`/draft`* on such a chat is the root's `Command::Draft`, as the
    browser's terminal's "Draft": the draft shown at half, or hidden
    (*Offered*).
  - *The body as the state changes:* entering all, the host sets the
    body's scroll to its first row and stops following its end (skein's
    `ui/screen.md`, 5.2, `Focus::scroll` and `Focus::follow`); leaving
    all, it follows the conversation's end again: the draft reads from
    its head, the conversation from its newest words. PgUp and PgDn page
    the body in each state; at half the pane shows what fits and does
    not scroll (skein's `ui/screen.md`, 5.2, *A side region*), all
    being the place to read the draft whole.
- **The draft,** at half and at all alike, from the chat's document as
  the web reads it (`../pages/README.md`, *A chat's document*: the same reads, no new
  watch), in `temper-client-tui`'s `view/pane.rs`:
  - *its head:* the document's name, the slug of its title and ".md"
    (`document::slug`), as "raft-to-vsr" and ".md" make the name (`tui-name`); while the chat
    writes a draft and none is read yet, "untitled.md" and " · writing"
    (`tui-hint`); then a blank row;
  - *its sections:* the body split at each `#` and `##` line outside a
    fence (`document::sections`), each read as Markdown by skein-ui's
    reader as an agent's text is, a blank row between two; at half,
    from the first section marked new, else from the first, those
    before it left out, since a side region shows its first rows; at
    all, every section from the first;
  - *"new":* two spaces and "new" (`tui-signal`) after the heading of
    each section the last turn wrote, on its row. The last turn is the
    conversation after the person's last words; its writes, each
    successful `note` or `document` call on the chat's document there
    (`document::tool_write`), in order. A first write marks every
    section; a later one, `NoteChange::Later { added, removed, first }`,
    marks the sections holding its lines `first` to `first + added − 1`,
    or the one holding `first` where it added none, each line moved by
    the turn's later writes above it, `added − removed` each. A turn
    that wrote nothing marks none, and the marks stand until the next
    turn's writes. `temper_client_tui::marked(body, writes)`, pure,
    gives the sections' indexes. No diff is drawn.
  - *a flow* (`skein/docs/design/ui/markdown.md`, 1) prints as its fenced
    text: a fenced block whose info string is `mermaid` lays as a code
    block of its lines, never a drawing, in the pane and in all alone;
    the conversation reads as it does.
  - *empty:* while the chat writes a draft and none is read yet, the
    head and "The draft appears here as you write it together."
    (`tui-hint`), the browser's words (`../pages/README.md`, *Empty*).
  - *not drawn:* the web's tint, its note "Changed by T4 just now" and
    "Show"; the path's `docs/design/draft/`; and in the conversation a
    write's line (*A chat*, *not drawn*).
- **A reply's choices,** as the drafter asks (its instructions,
  `temper/crates/temper-service/src/stock/drafter.txt`: one question at
  a time, as numbered choices, the recommendation first with its
  reason, "Tell me something else" last):
  - *Which:* the chat's last committed agent text, with no words of the
    person's after it and the chat not working, whose text ends, past
    white space, with a numbered list of the items 1 to N, N at least 2,
    whose last item's words are "Tell me something else" or "Tell it
    something else", a full stop after them allowed;
    `temper_client_tui::reply_choices(text)`, pure, reads it. Any other
    reply reads as today.
  - *Each choice,* the items 1 to N − 1: the item's first line, its
    Markdown `**` and `__` dropped, cut at the first ` — `, ` – `, ` - `
    or `: ` into its words, before, and its reason, after, trimmed;
    with none of them, the whole is its words.
  - *Drawn* as the chat's question is (above): the list under the box
    in the quiet line's place, with no head, since the reply's question
    stands just above it; its choices numbered "1. ", "2. ", the first,
    the recommendation, active, each its words as its name and its
    reason as its words (`tui-option`); then "N. Tell it something
    else", the TUI's words (*The chat's question*). A held question of
    the chat's wins over it, and any list a command or a walk opens
    stands in its place. The reply itself prints in the conversation as
    written, its list included.
  - *Choosing,* Enter on the active choice, or Enter on the box's words
    when they are a choice's number alone ("1"): the TUI's event
    `Event::Answer { words }`, the choice's words sent as the person's
    message, `Act::MessageWords` then `Act::Send`, and the box cleared.
  - *Set aside:* "Tell it something else", by Enter or its number, Esc,
    or words typed that are no choice's number: `Event::ReplyAside {
    turn }`, the list gone for that reply, the box the person's own, its
    words sent as any message; `Tui` keeps the reply's turn, and the
    list stands again only for a newer reply.
- **`/goal`,** the root's `Command::Goal` where *Offered* says, starts a
  goal from the draft through the domain's start sheet, as the web's
  "Start goal…" does (`../pages/README.md`, *Starting a goal from a chat's document*;
  `../pages/goal.md`, 3):
  - *Pressed or sent:* the host makes `Act::OpenStart { draft: Some(PATH),
    chat: Some(TASK) }`, PATH the draft's path,
    `pages::chat::sheet::path(&document::slug(title))`, never
    `terminal::Act::Run`, so that the root's sheet opens with the chat's
    checks and the project's preferred way; `Tui` keeps the chat.
  - *Its prompt,* while the root's sheet stands (`Pages::start`), as a
    command's list (*A command's list*): its head "Start a goal from
    FILE. How should it run?" (`tui-list-head`), FILE the path's last
    segment; its choices the project's ways, kind 7's tracked templates
    (`temper_client_view::pages::goals::ways(me, project, words)`),
    numbered, the sheet's chosen way, the preferred one, first and
    active, the others in number order, each its name and its line as
    its words; then "Cancel". Esc is Cancel, `Act::StartCancel`.
  - *A way chosen:* `Event::Way { template }`, `Act::StartWay {
    template }` then `Act::StartGoal`, the sheet's name its default and
    its asking "As the project says" (`../pages/goal.md`, 3). A refusal, or nothing to
    fund it (`StartGoal::error`, `start_goal::unfunded`), is drawn under
    the box in `tui-refused`, in the view's words for the sheet's error,
    and the prompt stays.
  - *Made:* the root opens the goal's `Task { task }`; the TUI has no
    goal's page, so the host, while `Tui` keeps the chat, opens that
    chat again (`Act::Go { address: Chat { task } }`) and lets it go.
    *Proposed:* the sheet closes, and the notice says "Proposed: the goal
    starts once it is accepted." (*The notice*).
- **A linked goal,** the newest goal started from this chat's draft
  (`pages::chat::Chat::goal_from`, its chip `Chat::goal_chip`): `/goal`
  is no longer offered, the quiet line names the goal, and `/open` opens
  its page (*The TUI's own commands*).
- **The quiet line,** on a chat whose pane can show (*The foot*, *The
  quiet line*):
  - *at its left,* in a design session: the project's name, " · ",
    "drafter" (`tui-hint`), " · " and the closed line, "Opus `../pages/accounts.md`, 2 ·
    xhigh"; the link's state after them, " · ● Offline", only while it
    is not Live; in any other chat, as before;
  - *at its right,* after "paused · a message resumes it" where the
    person paused the chat: the pane's state, "pane: folded", "pane:
    half" or "pane: all", as shown; " · " and what the next Ctrl-P does,
    "Ctrl-P opens", "Ctrl-P widens" or "Ctrl-P folds"; then the goal's
    part, " · " and "/goal starts a goal from the draft" while `/goal`
    is offered, or "goal TITLE · /open opens it" while a goal is linked,
    TITLE its chip's title cut at 20 characters as the rule's title is;
    all in `tui-hint`; then the count of what needs the person, as
    before;
  - *narrow:* the goal's part goes first, then the Ctrl-P words, each
    with its " · ", then as *A narrow terminal* says.
- **Where it departs from its mockup** (`tui/whole-story/design-session.html`;
  `../../draft/surfaces.md`, 13), each for a reason:
  - the chat's title is in the rule above the box, "cowsql / Raft to
    vsr T12 on box", where the mockup's framed header says "temper ·
    cowsql · raft-to-vsr · a design session": the look request
    `tui-chat-title-below` moved a chat's title there, and "drafter" in
    the quiet line says what it is;
  - the pane's head is the name alone, with no "draft" at its end and
    no rule under it, and "new" stands two spaces after a section's
    heading rather than flush with the pane's end: a side region lays
    lines, nothing flush beside other words on a row (skein's
    `ui/screen.md`, 5.2); and "new" marks a `#` or `##` section, the
    document's parts as the domain splits them, where the mockup marks
    a numbered paragraph;
  - the drafter's choices stand under the box as a prompt, as the
    request asks (`../../draft/tui-design-session.md`), as well as in the
    reply, where the mockup prints them in the reply alone; and the
    last says "Tell it something else", the TUI's words;
  - the quiet line at half says "Ctrl-P widens", where the mockup has
    "Ctrl-P folds", since its words say what the next press does; at 80
    columns the goal's part is left out, which the mockup's 110 hold;
    and the link's state shows while it is not Live, which the mockup,
    live, leaves out;
  - the halves split the window evenly, 55 and 55 columns at 110,
    where the mockup gives the conversation 56 and the pane 54.
