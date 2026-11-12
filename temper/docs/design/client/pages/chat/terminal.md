# A chat's terminal

## 1. Its terminal

- **A chat's terminal** (the look requests `chat-terminal`, `chat-terminal-font`,
  `chat-terminal-draft` and `chat-terminal-esc` of 2026-11-07, their pages
  `journeys/chat-terminal/`): a browser's other look of a chat's pages, `Chat`,
  `NewChat` and `Ask`, as a coding agent's terminal, full screen. It is the
  same page, its state, reads and acts, drawn otherwise: its commands are the
  page's own acts, and nothing new crosses to the engine but the reads named
  below, which other pages make already. The terminal shell (`../../line.md`) is
  another thing: it prints these pages as it always has.
  - *Its look, kept per browser:* `pages::terminal::Look { terminal, theme, font, night }`
    on `Pages`, the same on every chat the browser opens, so that another chat,
    `/chats`, `/new` and Start over open in it. `theme` is `Dark` or `Light`,
    `font` `DejaVu` or `Plex`; `night` is the night look of every page (`../../view.md`, 3, the
    night switch; `../../web/look.md`, *Night*), outside the terminal's own theme.
    A browser with no record of its own starts
    `{ terminal: false, theme: Dark, font: DejaVu, night: false }`: a first visit
    is day, whatever the system's scheme. The shell reads it at start
    from the kept store's record `look` (skein's `ui/web/README.md`, 8), four bytes,
    one for each field, in that order, `0` or `1`, `1` the terminal, `Light`,
    `Plex` and night; a record of three, the form before `night`, reads with
    `night` off, and a record of any other form is none. A change writes the record, in
    the step that made it, through a host duty, and a write refused changes
    nothing on the page. Another tab reads it at its own start.
    `terminal::Act::Night`, the night switch's press on any page, turns
    `night` and keeps the record as `Theme` does; it asks no full screen. The
    web shell sets the attribute `data-night` on the document's `html` while
    `night` is on and removes it otherwise, from the record before the first
    page is drawn and in the step a press turns it; the terminal shells draw no
    night.
  - *Entered:* for now, by nothing on the page: no bar draws ">_" ("Terminal,
    full screen"), on any chat, `NewChat` or `Ask`, on a phone either, now
    that temper has its own TUI (`../../tui/README.md`); the button needs
    care, and comes back later (the look request `no-terminal-button`,
    2026-11-12). On a chat the bar's other parts move into its place; on
    `Ask` the bar's right is empty. The look's code stays whole: a browser
    whose kept look says terminal still opens in it, and *Left* (below)
    still leaves it, so that no one is stranded; this request does not
    decide the look's future. `terminal::Act::Enter`, which the button gave
    and no control gives now, stays: it sets `terminal` and asks the shell for full
    screen in the same step (skein's `ui/web/README.md`, 9, *Full screen*), and
    draws the terminal in that step, before the browser answers: every terminal
    act, and a list's closing on any event, moves the pages' shown count, as
    each change the view draws does (`../../README.md`, section 3). Full screen refused, the
    terminal stays, not full screen.
  - *Left:* "Exit" and `/exit`, `terminal::Act::Leave`: `terminal` unset, full
    screen left. The browser leaving full screen (its Escape, a long Escape where
    the terminal holds the key, its own control) leaves the look as it is: the
    terminal stays, not full screen, and an open list closes (*Keys,* below). A
    reload, which leaves full screen, opens the terminal again, not full screen,
    since only a person's press may ask for it.
  - *Where it draws:* in a browser, on `Chat`, `NewChat` and `Ask` while
    `terminal` is set; every other page, and the terminal shell's, as before.
    The frame draws no rail and no tab bar around it: the page's main is the whole
    window. Its regions in order: the top line, the conversation (with the draft
    beside it), the running line, the box and its list, the quiet line.
  - *The top line:* the bar, on one line. At its left "✦", the project's name, a
    faint "/", the level-1 heading, named as the bar's, showing the title as the
    bar's does, and the chat's number "T2", on an Ask temper chat then a faint
    " · " and "Ask temper". At its right "Draft" and "Exit". On a
    phone the star and the title alone: the project, its "/" and the number go.
    "Draft" is a button pressed while the document shows: `Act::Mode` to Split,
    or back to Chat, its shortcut `Ctrl+\`; with no document, where
    `chat_offers_draft` and the chat hasn't ended, "+ Draft", `Act::AddDraft`,
    the write-up the browser's pane asks with "Write it up from this chat"
    (`document.md`, 1): the browser's bar no longer draws "+ Draft" (the look
    request `chat-bar`), and this look, kept as it is, does not decide that
    (*Entered*, above);
    otherwise none. Nothing else of the bar draws: its status, Pause or Resume,
    Close, Start over, "Start goal…", the mode group and the spend are commands
    and the quiet line's.
  - *The conversation:* the page's own, every line as the browser draws it, in the
    terminal's look (`../../web/look.md`, *A chat's terminal*): the person's words on
    a band after "❯", the agent's after "✦", calls folded under "▸", a note's
    write "✎ Wrote NAME · 30 lines Show". The speakers' names and the turns'
    times stay in the tree and the look hides them. A fold whose call failed shows
    its first failure's line on its folded row, "failed: WORDS", where today it
    says "1 failed". The column stays left, 100 characters at most.
  - *The running line,* above the box, while the chat works, its status one
    the browser draws working dots for (`conversation.md`, 1, *While it works*), whether or not
    the dots stand: the words being written and a call under way keep it. Or
    while the chat waits on its delegates or on an account: "✦",
    pulsing, then the bar's status words, with "Writing" presented as
    "Working" (`../../tui/pages/README.md`, 1); "waiting on 1 delegate" and "Waiting for
    an account" keep their words. It draws no clock.
  - *The box:* the composer, "❯" before its text area, its name, placeholder,
    hint's keys and bounds as a chat's (`composer.md`, 1, *Where the focus stands*), with no
    "Send". On a chat that has ended, which a browser draws with no composer, the
    terminal keeps the box for commands alone: its placeholder "/new starts another
    chat"; words that are no command note "This chat is closed. /new starts
    another." Under it, *the quiet line:* at its left "/ for commands", or "paused ·
    a message resumes it" on a chat the person paused (`README.md`, 1, *A person's pause*); at its
    right the picker's closed line, "Claude Opus `../accounts.md`, 2 · high", a button that opens
    `/model`'s list, then " · 778 spent", the chat's own spend. On a phone the spend
    goes. While a list is open the quiet line gives it its place.
  - *Commands,* typed in the box: words that begin with "/" and hold no white space
    open the list under the box (skein's `ui/README.md`, 3.2, *options*), named
    "Commands": each command the page offers now whose name begins with the words,
    in the table's order, its name and its words. Enter presses the active one, Tab
    completes its name, a click presses it. Enter on words that name no command
    sends nothing and notes "Unknown command /WORDS. /help lists them.". A command
    pressed or sent clears the box.

    | Command | Its words | What it does | Offered |
    |---|---|---|---|
    | `/model` | Change the model, from the next turn | its list (below) | where the picker is |
    | `/effort` | Change the effort | its list (below) | where the chosen model has efforts |
    | `/draft` | Show or hide the draft | as "Draft" | where "Draft" or "+ Draft" is |
    | `/pause` | Pause this chat; a message resumes it | `Act::Stop` | where Pause is |
    | `/close` | Close this chat; it stays in Chats | `Act::Close`, its confirmation as a list | where Close is |
    | `/start-over` | Start a new Ask temper chat | `Act::StartOver`, its confirmation as a list | where Start over is |
    | `/goal` | Start a goal from the draft | "Start goal…", its sheet | where "Start goal…" is |
    | `/new` | Start a new chat | `Act::Go` to `NewChat` | always |
    | `/chats` | Open another chat | its list (below) | always |
    | `/theme` | Dark or light | the other theme | always |
    | `/font` | DejaVu Sans Mono or IBM Plex Mono, to compare | the other font; notes "Font: NAME." | always |
    | `/help` | Commands and keys | notes the commands and keys | always |
    | `/exit` | Leave the terminal | `terminal::Act::Leave` | always |

  - *A command's list,* under the box in the commands' place, its head a line of
    its own, its choices the list's options, the current one selected; taking one
    closes it, Esc closes it, and words typed close it:
    - `/model`: "Model. A change applies from the next turn."; the picker's
      menu's models (`composer.md`, 1, *The work's account and model*), each its shown name and,
      muted, what pays ("your subscription"), "default" after it on the default's;
      a provider with no model its name and "Connect a plan or key ›", a link to
      `ConnectModel`. Taking a model is `choice::Act::Model`. "All models" stays the
      browser look's.
    - `/effort`: "Effort. A change applies from the next turn."; the chosen model's
      efforts, `choice::Act::Effort`.
    - `/chats`: "Chats"; the person's live chats in the project, newest activity
      first, each "TN" and its title, as the Chats page's Live filter reads them
      (`tasks{query}`, its first page), read when the list opens; taking one is
      `Act::Go` to its `Chat { task }`.
    - `/close` and `/start-over`: the confirmation's question as the head ("Close
      this chat? It stays in Chats.", "Start over? A new Ask temper chat starts.
      This one stays in Chats.") and its two acts as choices, "Close" or "Start
      over" first, then "Keep it open" or "Keep this one"; a confirmation that also
      asks its delegates' fates draws as the browser's sheet, over the terminal.
  - *The terminal's notes:* lines the page writes after the conversation, in the
    terminal's look: `/help`'s table of commands and keys, an unknown command's,
    `/font`'s, a closed chat's. They are the page's state, at most 8, the oldest
    going first, gone when the page closes; nothing sends or keeps them.
  - *Keys:* Enter sends, Shift+Enter makes a new line, `Ctrl+\` shows or hides the
    draft, PgUp and PgDn read back (skein's `ui/web/README.md`, 5), Esc closes a list and
    otherwise does nothing; in a list, the arrows move, Tab completes, Enter takes.
    In full screen, where the browser lets the terminal hold Escape (Chromium's), a
    short Esc is the terminal's and a long one leaves full screen; where it doesn't
    (Firefox, Safari), one Esc leaves full screen and closes an open list with it.
    The desktop's own shortcuts stay the browser's.
  - *The draft beside it:* in Split the document in the right half, behind a rule,
    on a ground of its own; on a phone, at 760 px or less, the document takes the
    conversation's place and the box stays at the foot, as Split shows as Draft
    there. It is the page's own pane (`document.md`, 1), its rules shared with
    the browser's look under the terminal's tokens (`../../web/look.md`). The pane's
    first paragraph, in both looks, reads its words as Markdown's inline runs,
    emphasis and strong among them, in its muted style.
  - *A goal's question,* where a delegate of the chat asked the person one: while
    the terminal draws a chat whose wait is on the person, the page reads the inbox
    of the chat's subtree, its questions (`Read::Inbox` with the chat as its root,
    as a tracked task's page reads its own), into the page's inbox, and draws its
    first question under the box as a list: its question line as the head, its
    parsed choices as options, the recommended one selected, then "Later";
    choosing answers it as Needs you does (`needs::Act::Choose`, `needs::Act::Later`).
    A question with no choices draws its line and "Answer in Needs you ›", a link to
    its card. The browser's look reads no inbox here.
  - *A proposal or an Accept card* in the conversation, where the browser's look
    draws its card: one line, its kicker ("Proposal") in the accent and its title, its
    acts after it ("Accept", "Reject…"), its facts folded under it as a call's are;
    Accept's confirmation as `/close`'s list. A delegation's card is one line too:
    "TN · KIND", its title, its state and "Open".

## 2. Open questions

- **The chat terminal after landing** (`../README.md`): temper/ui compares
  `chat-terminal`, `chat-terminal-font`, `chat-terminal-draft` and
  `chat-terminal-esc` with the shipped view at 1440, 1000 and 420 px,
  with menus and disclosures open and every state their messages name
  (`../../web/look-requests.md`, sections 4 and 5). Include both themes
  and fonts, the draft and its figure, questions, compact cards and
  closed chats. Check that applying `terminal::Act::Enter`, which no page
  control gives now, draws the terminal before any further input, including
  when full screen is refused, and that Exit
  and closing a list redraw at once with no command words in the box.
  Real keys check Chromium's short and long Escape,
  Exit, desktop shortcuts, Firefox and Safari's one-Escape fallback,
  and unavailable or refused keyboard lock, including insecure contexts.
  Default: the shared page's acts, bounded notes, kept look, focused
  tables and gallery states; compilation and print checks establish no
  rendered equality or real-key result. The departures in
  `../README.status.md` remain temper/ui's: budget words, settings
  proposal controls, case-sensitive commands and background focus.
