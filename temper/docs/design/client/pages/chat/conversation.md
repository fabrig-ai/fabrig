# A chat's conversation

## 1. Its conversation

- **A conversation's turns** are named "You" when the turn is the
  person's own, "Temper" when it is the agent's, and by the party's
  name otherwise (`chat.html`), on every page that draws a
  conversation. A chip's executor keeps jig's "Agent". On a chat's
  page each turn's time follows its name, "HH:MM", under the day's
  rule (`../../view.md`, 6): a line's time for the agent's words and the person's,
  and for the request the conversation's first line's; words not yet
  read by the engine have none.
- **A chat's conversation** is the chat's own, drawn from jig's thread
  (jig's `client/domain/objects.md`, section 10), never its runs'
  mechanics (`chat.html`, the mockups' first run):
  - *its request,* first, under "You" (or the requester's name): the
    chat's specification's words (its document's `spec`), a paragraph
    for each of its lines, where the window holds the conversation's
    start, its first line with no older page; a chat whose spec is
    empty has no such turn, and opens on its first words line;
  - *the agent's words,* each assistant message's text under "Temper",
    whole within the receiving bounds (`../../view.md`, 4), read as Markdown by
    `skein-ui`'s reader (skein's `ui/markdown.md`): paragraphs, lists,
    quotes, code, and tables at the top level. A browser draws a table
    as the mockups' (`md-table`), and leaves out a header row whose
    cells are all empty, as they do. A provider's opaque block, a turn's
    reasoning, is not drawn, and nothing stands in its place. A
    conversation says nothing of what was cut: no notice, at a text's end,
    a turn's or a window's;
  - *the words being written,* the turn being written, under "Temper",
    whole from its start up to `long_text` (`../../view.md`, 4), a page opened late
    from what the engine retains (`../../README.md`, section 5), read by `skein-ui`'s
    `read_growing` (skein's `ui/markdown.md`): a block a blank line
    has closed stays on the page as it was, only the last changes as
    words arrive, and no marker shows being written. A small block,
    "▍", in `temper-writing-end`, stands after the last word. The
    committed turn draws the same blocks in the same place, and only
    the block goes. Before its first word the turn draws nothing, and
    the three dots stand (below);
  - *its calls,* where they fall among the agent's texts: each
    delegation as its task's card (`../../view.md`, 5), a document's write as its tool
    line (`document.md`, 1, *The tool line*), and every other call in *a run*: every
    call between two of the agent's texts, across the turns that made
    them, drawn once as one line. A text, a person's words, a card, a
    document's write or any other drawn line ends a run; a line the chat
    hides does not. A call that a card stands for is neither counted,
    named nor listed. The line is one disclosure (skein's `ui/README.md`, its
    disclosure), `temper-work-run`, in a turn of its own:
    - *its name,* one phrase a tool, in the order first called, joined
      by ", ", the first capitalised unless it is a tool's own name:
      "Read 7 docs, checked settings 3×", "read_file ×2". The phrases are
      the view's table, one function (`pages/chat_line.rs`), tool by tool,
      starting from the tools a chat calls most. A tool in it counts what
      it touched, not its calls: `read_docs` is "read N docs", its distinct
      topics, each search one, a call whose input can't be read one
      ("read 1 doc"); `settings` is "checked settings N×", once "checked
      settings"; successful `propose_settings` calls are "proposed a change N×", once
      "proposed a change"; failed calls are "failed to propose a change N×",
      once "failed to propose a change", grouped separately from successes
      in the order each outcome first occurs. Under way it is "proposing a change", none of them
      naming its input; `read` and `Read` are "read N files", `list` and `Glob`
      "listed N folders", `write` and `Write` "wrote N files", `edit` and
      `Edit` "edited N files" and `remove` "removed N files", each its
      distinct arguments, a call whose input can't be read one; `search`
      and `Grep` are "searched the files N×", once "searched the files";
      `shell` and `Bash` are "ran N commands", each call one; `see`, Ask
      temper's read (`../../../draft/ask-temper.md`, 5.1), is a phrase for each
      first word of its address, in the order first called, as the table
      below has it. Any other
      tool is its name, counted: "read_file ×2", once "read_file". A run of
      one call alone says what it touched: "Read the page hold", "Searched
      the docs for WORDS", "Checked template 2", "Read src/main.rs",
      "Searched for WORDS", "Ran cargo test", its counted phrase where the
      input can't be read ("Read 1 file"); any other tool still its name.
      `see`'s phrases, each first word of its address (the input's first
      text member) a phrase of its own, as if a tool of its own; "N×"
      once is no count; another first word, or an address that can't be
      read, is `see`'s name, counted, "see ×2":

      | First word | Counted | One call alone | Under way |
      |---|---|---|---|
      | `changes` | "listed changes", N× | "Listed changes" | "listing changes" |
      | `change` | "read N changes", its distinct numbers | "Read change N" | "reading change N" |
      | `tasks` | "listed tasks", N× | "Listed tasks" | "listing tasks" |
      | `task` | "read N tasks", its distinct numbers | "Read task N" | "reading task N" |
      | `goals` | "listed goals", N× | "Listed goals" | "listing goals" |
      | `inbox` | "checked the inbox", N× | "Checked the inbox" | "checking the inbox" |
      | `me` | "checked your role", N× | "Checked your role" | "checking your role" |
      | `project` | "read the project", N× | "Read the project" | "reading the project" |
      | `settings` | "checked settings", N× | "Checked settings" | "checking settings" |
      | `ways` | "read the ways", N× | "Read the ways" | "reading the ways" |
      | `revisions`, `revision` | "read the history", N× | "Read the history" | "reading the history" |
      | `proposal` | "read N proposals", its distinct addresses | "Read proposal ID" | "reading proposal ID" |
      | `help` | "read N docs", each page and each search one | "Read the page NAME", "Searched the docs for WORDS", "Read the docs" | "reading NAME", "searching the docs for WORDS" |
      | `file` | "read N files", its distinct paths | "Read PATH" | "reading PATH" |
      | `more` | no phrase of its own where the run has another `see`; else "read on", N× | "Read on" | "reading on" |

      So `see changes` then `see change 1`, `2` and `3` read "Listed
      changes, read 3 changes";
    - *its description,* faint: the time the run took, then nothing else.
      "29 s", under a minute in whole seconds, then "4 min": from the
      line before the run's first turn to the run's last turn. A committed
      turn says only when it ended, so the line before it says when it
      began: any line of the task's activity, drawn or not (a turn, the
      person's words, the run's beginning). A run with no such line, at the
      window's head or the second run of one turn, measures from its first
      turn's end. Under a second has no time and no description;
    - *a failed call* adds "1 failed" ("N failed") before the time in the
      description, "1 failed · 29 s", the description then in the signal's
      colour, `temper-work-failed`, and read after the name with a comma
      the stylesheet draws;
    - *while it runs:* the conversation's last run, while the chat's
      status is active (`chats::working`) or an agent's run is out (`../task.md`, 2),
      has `temper-work-running` and no time. It counts what has ended, and
      its description is the call under way, from the run's `call` (jig's
      `RunState`), the phrase's present form in faint words: "checking
      template 2", "reading index", "reading src/main.rs", "searching for
      WORDS", "running cargo test"; any other tool, and a call whose input
      can't be read, its name. The stylesheet adds the "…", and the
      terminal, which has none, writes it. With no call under way, no
      description. A call under way with none ended is the line alone, its
      name the present form capitalised;
    - *opened,* `temper-work-calls`, a row a call in order, the call under
      way last: a paragraph named "TOOL ARGUMENT: OUTCOME", the tool in
      code, the argument in a few words (`read_docs`: "index", "change,
      from 1438", a search's words; `settings`: "the inventory", "charter
      1", "history"; any other tool: its input's first text or number
      member, "task 3", a text's first line; a live call's input cut at its
      bound as far as its first member closes; nothing where the input
      can't be read), and, in
      `temper-work-out`, its outcome's first line, a failed call's
      "failed: LINE" in the signal's colour. Never a result's body. A row
      is cut at 160 bytes, and a name at 256 at a phrase's end, with "…".
    A browser shows it folded until the person opens it. The terminal
    prints the name, then its description after a colon, and its rows
    under it, as text; the fake person finds them whole;
  - *the person's words,* the thread's words and each words line a
    person sent (jig's `OwnerActivity::Words`, by a person), under
    "You" or the party's name, a paragraph for each line, with "Being
    sent" or "Sent" until the engine has it, and no word once read;
    an answer the person gave in the inbox to a question a delegate
    asked is not among them: the answer is the asker's, in its
    activity (`../../../domain/agent/agents.md`, 2), and the conversation shows
    the notice the chat heard as one line under "You", "You answered
    question N in your inbox". The notice carries the answer words cut
    at 256 bytes on a UTF-8 boundary and marked when cut
    (`../../../domain/agent/README.md`, section 4);
  - *a delegate's words,* each words line a task sent the chat (jig's
    `OwnerActivity::Words`, by a task), a delegate's question among
    them, under the task's name, "TN", on the agent's side, a paragraph
    for each line;
  - *not drawn:* a run's beginning and end, the task's milestones, a
    turn's opaque blocks (a model's reasoning, with nothing in their
    place), and every message a turn's person, tool or system speaks, which are
    the run's inputs: its opening, its calls' results, and what woke
    it, a result delivered among them. A result block still anchors
    its task's card where the call that made it is not in the window
    (`../../view.md`, 5), drawn on the agent's side, never under "You";
  - *its end,* once the chat has ended, after the conversation: one
    line, "Closed at" and the time it ended ("Closed at 21:05", `../../view.md`, 6's
    clock), the status's word (`README.md`, 1's chats), then, for a report,
    " · its report, “REPORT”", the
    report's words whole; a failed chat's line is "Failed", its time
    and its reason ("Failed 21:05 · REASON"); a chat cancelled by the
    party that requested it was closed, and its line is "Closed at" and its
    time ("Closed at 21:05"), with no reason; one cancelled by anyone else
    is "Cancelled", its time and its reason ("Cancelled 21:05 ·
    REASON"). Who closed it is the ending's `by` against the chat's
    requester, both in its document, never the reason's words. The
    result is not drawn again.

  Its look is the greenfield mock's `.thread` and `.thread-inner`: a column
  720 px wide and centred, gap 20, whose scroll stays at the end while the
  person was at the end. The conversation is still a log.
  - *The person's and the agent's turns* are `.turn-you` and `.turn-agent`,
    the agent's Markdown with its first and last margins zero. A turn's
    name and clock are hidden from sight and stay for readers and the
    terminal, but "Being sent" and "Sent": 12 px, faint, right aligned under
    the bubble. The day's line is hidden; a marker is `.meta-line`.
  - *A delegate's words* look like `.turn-agent`, with "TN" kept visible,
    12 px and faint, above them.
  - *A call's line,* `.turn-tool`, opens with the document's icon where it
    is a run's and the pen where it is a write's. A run's line is muted, its
    description faint, with a "›" that turns when it opens; open, its rows
    are indented 24 px, a row wraps at a phone's width, the tool 12 px mono,
    the argument muted, the outcome faint. A delegated change and a goal are
    one line each (`../../view.md`, 5).
  - *While it works:* three dots, `.typing`, at the end of the log, named
    "Writing", while the chat's status is Starting, Writing, Working or a
    tool's, the window's last line is not provisional text holding a word, and no run is
    running at the conversation's end. The terminal
    prints nothing new for it. *The words being written* look like
    `.turn-agent`, their name hidden as a committed turn's, their "▍"
    faint and in the line of the last word; the terminal prints the "▍"
    as it prints a status. A numbered list's `ol` is 22 px in, its
    markers faint.
  - *A card in the conversation* (a proposal, a held task, a question's
    answer) is `.proposal`: white, radius 12, its shadow, 20 px 22 px; a
    kind's title in `.proposal-title`; `.proposal-split` and
    `.proposal-state`; its first button `.btn.primary.small`, the rest
    `.btn.quiet.small`.
  - *Reading back:* "Latest", `temper-latest`, a browser's alone: a small
    pill, centred 12 px over the pane's foot, its " ↓" the stylesheet's, on
    a button *to log end* (skein's `ui/README.md`, 3.2) beside the log, shown while
    the log is away from its end (`data-log-away`, skein's `ui/web/README.md`, 5).
    At a phone's width it stands in a row of its own below the log instead,
    so nothing lies beneath it and it covers none of a card's controls.
    Pressed, the view returns to the end and it goes. Where the window has
    let the newest lines go the log continues and the pill stays visible,
    even at the window's foot. A press on a chat reads the end again;
    on an agent's page it reads the next later page (2).
    Sending on a chat returns to the end too. New words arriving while
    reading back don't move the view.
  - *"Loading earlier messages…",* `temper-thread-loading`, faint and
    centred at the head of a chat's conversation or an agent's, a status
    named "Loading earlier messages", only while an older read is out.
    A failed read says "Couldn't load earlier messages" there, in the same
    look; the next move toward the head tries again (2). Nothing once
    the chat's start is shown.
  - *Earlier lines* load as the log's head nears the view (2), with no
    "Older" button, on a chat's page and an agent's. *Its end* has
    `.meta-line`'s look, and an ended chat has no composer.

## 2. Reading a conversation

A chat's conversation and an agent's task's (`../task.md`, 2) are read by scrolling
(`../../../draft/chat-reading.md`, section 5). jig's thread holds the window
(jig's `client/domain/client/conversations.md`), and skein's page part
measures it (skein's `ui/web/README.md`, section 5).

- **The log is bound** while an earlier page remains or the window has let
  its newest lines go, its act decoding to `Older` at its head and `Newer` at
  its foot. There is no "Older" button. Earlier lines load when the head
  comes within one and a half screens, whatever brought it there. The same
  rule opens a chat: a long one opens with at least a screen and a half,
  a short one whole.
- **Opening history:** a chat follows its first page's older cursor without a
  scroll event, by the rule above: a short conversation reaches its opening
  question even when the reply has already ended, and a long one stops once
  a screen and a half is held, its older cursor kept for the next read, which
  a browser's log asks for as its head comes near, saying "Loading earlier
  messages" as any read does. The domain measures no screen, so a screen and
  a half is a weight: the held lines weighing 2,880 bytes or more, a screen
  and a half of an 80 by 24 terminal, by jig's weight of a line (its bytes,
  and `LINE_ROOM` for it and for each message, block, object and part it
  holds; jig's `client/domain/client/conversations.md`). Initial reads stop at
  whichever comes first: the history's end, that weight held, the window's
  row or byte bound, a lasting read failure or a cursor that does not
  advance. A page that would exceed the window leaves its held
  newest rows and older cursor intact. Empty pages spend the same bounded
  initial read count as nonempty pages, at most the window's row capacity.
  Transient failures retry the same read under the conversation's retry rule.
- **The window** fills `document_bytes`, with no count of lines: its `lines`
  is what its bytes hold (`thread_limits`). `Limits::window_bytes` names a
  narrower window, of at least `long_text` and at most `document_bytes`, where
  a bundle asks for one: the browser tier's alone (`../../web/testing.md`, 3). The
  page's bounds are priced from `document_bytes` either way (`../../view.md`, 4). Reading
  earlier past the window lets the newest whole lines go,
  and reading later the earliest; they come back when scrolled to, and nothing
  says how many left.
- **While a window has let its newest go** the log *continues* (skein's
  `ui/README.md`, 3.2), so it isn't followed and "Latest" shows, even at its foot.
  New words and a run's live text wait: they show when the end is back.
- **A delivery that makes the view stale** rebuilds the semantic page tree,
  rereads the held Markdown and compares the old and new trees across the
  window. Unchanged builds are gated; the diff patches changed DOM nodes in
  place. No parsed Markdown or conversation subtree is retained. Following
  reads aggregate log geometry; holding an away reader can scan entries for
  its anchors, including on a patch outside that log (skein's `ui/web/README.md`, 5).
- **Each line's outer node is keyed,** a turn by its name, a run of calls by
  its first turn's, a person's words by their message, an activity line by its
  position. So a page read in above is an insert, nothing already drawn is
  rewritten, and the page part can keep the words where they are.
- **A chat's log head holds nothing that stays while lines arrive below it,** but
  the status below: the page part holds the first entry in view. So the context
  line (`../welcome.md`, 2) draws once the window holds the chat's start, and a day's label is
  keyed by its date and the first line under it, so that a page of the same day
  read in above replaces it. An agent's task's log keeps its brief, run heading
  and own held card before its lines (`../task.md`, 2); their effect on reaching earlier
  lines is `../../README.md`, section 5's *An agent's page opening at its end*.
- **While a read is out** the log is *busy*, and so it stays after a failure,
  which the page part then leaves alone until the person moves. At the head:
  "Loading earlier messages…" while an earlier page is out, "Couldn't load
  earlier messages" in the same look after one failed, and nothing once the
  chat's start is shown. A read at the foot says nothing, but a chat's first read,
  failed with no line to show, says the same at the head. An answer the client
  cannot decode is such a failure (jig's `client/domain/client/README.md`, section 13): it
  asks nothing again until the person moves, or a new line is committed, and the
  page's other lines and acts go on. Lost exchanges and busy reads keep their
  transient retries.
- **"Latest":** the view returns to the window's foot. On a chat where the
  window let the end go, a press reads the end again (`Thread::latest`),
  which replaces the window. A person sending on a chat does the same
  through the log's end request,
  which `Chat` counts. An agent's page has the same pill; while its window
  has let the end go, a press asks for the next later page (`Thread::newer`),
  and the log, following, asks for each next until its end is back (skein's
  `ui/web/README.md`, section 5). Its log has no end request, so sending there does
  not request the end.
- **The terminal:** the log is bound and continues as in a browser, with no
  measures: `up` reads earlier and `down` later (skein's `ui/text.md`, section
  6), and each prints the whole window again. These are lines followed by
  Enter, as are the escape sequences PgUp and PgDn make, not raw keys.
  It opens with the bounded initial history above. The line under its bound log's name says
  `(up, down: more lines)`; there is no pill, and sending on a chat reads
  the end again.

## 3. Views in a reply

A view is a picture temper's code draws from what is stored
(`../../../draft/ask-temper.md`, 6.1 and 6.2): placed by address, never drawn by
hand. Its stops come from the engine's one derivation
(`../../../domain/forge/change/branch-path.md`), so a reply, Ask temper's page and the
terminal draw the same path.

- **The block:** in a turn's text or the chat's document, a fenced block
  at the top level, its info string `view` and its one line the view's
  address. The client splits the text at such blocks before reading the
  rest as Markdown (skein's `ui/markdown.md`), so that a `view` block
  is never shown as code; one indented, inside another block or with more
  than one line stays code.
- **The addresses** of this milestone: `way NAME`, a stock way by its name
  as `ways` gives it, any case, over the chat's project; `path OWNER/NAME
  [BRANCH]`, a repository of the chat's project, its landing branch when
  left out; `proposal tN/K`, a waiting proposal's path before and after
  over the project's home repository's landing branch; `charter N`, a
  role; `rule K/N`, a landing rule, K 1 for the deployment's and 2 for
  the chat's project's, N its number; `roles`, every agent the project's
  work runs, each its charter's role card (`../../../domain/forge/change/branch-path.md`,
  *The project's roles*). A path or a charter takes
  `@revision N`, drawn when N is the newest revision; an older one is not
  drawn yet, and says so.
- **Read when shown:** the chat page reads each address it shows once,
  through the forge's read `View` in the chat's project (`../../forge/records.md`), equal
  addresses sharing one read, and again when the session's `settings`
  watch names a newer revision. Until it answers, the block says
  "Drawing VIEW…"; a read refused or failed says in a line that the view
  could not be drawn, and the address.
- **Drawn:** a path as temper's own drawing kind beside skein's flows,
  in fabrig.ai's language as the look request `ask-temper-ways` draws it:
  its stops on a line, a person's and an agent's above it, code's below,
  the line rising to each judgement and falling back, the dotted line
  no merge tool crosses, each branch's bar, the landing branch's at its
  end; each stop its mark, an agent's a four-point star, a person's a
  ring, code's a small square, and its words; a stop made only when
  needed (`needed`) its mark outlined, and a stop's small words
  (`over`) over its own, smaller and fainter (`../../../domain/forge/change/branch-path.md`
  , *Drawn apart*). Who acts is the mark alone, the colours the
  look's (`../../web/look.md`): a person's ring the signal's, an agent's
  star ink, code's square grey. Under the path, on Ask temper's page and
  How work runs, its legend: an agent, an agent when needed, a person,
  code, and the dotted line, "no merge tool above". A way draws its name and
  line over its path; a proposal its two paths, "Before" and "After"; a
  charter or a rule its card, `who`, `name` and `facts`, as a list of
  terms; `roles` each role's card so, in its order; a `not_drawn` its words. The drawing's fallback, which the terminal
  prints, is a stop a line, its words and its actor in words ("produce,
  an agent"), then ", when needed" for a needed stop and its small words
  in brackets: "repair, an agent, when needed (tests, review)", "merge,
  code (into goal)".
- **Its layout,** of the box's width `W` alone:
  - *across,* where `W` is 560 or more and its stops would stand at
    least 48 pixels apart in it (`W − 80` over the gaps between the stops
    before the last bar): a scene `A` wide and 140 high, `A` being `W`
    from 720 up and 720 under it, so that in a box under 720 the scene
    says its width and is scaled to the box, keeping its proportions, as
    the look request's picture is, 658 by 127.94 in a box 658 wide
    (skein's `ui/scenes.md`, 2): the dotted line
    at 72; a person's and an agent's marks at 40, their words over them
    at 22; code's at 106, its words under it at 131; the stops before the
    last bar evenly from 12 to `A − 68`, the first's words anchored at its
    start, the others' at their middle; a goal's branch's bar in a stop's
    place, dimmer, its words over the dotted line at 88; the landing
    branch's bar at `A − 8`, its words ending there at 88. The line runs
    along the base row through code's stops and the bars, rising to each
    judgement over 0.42 of a step and falling back over as much, a cubic
    curve each way, and ends at the landing branch's bar: the base row
    in the line's colour, each rise painted by a fade (skein's `ui/scenes.md`,
    2) from its mark's colour at the marks' row, through a fainter
    tone at the dotted line, to the line's colour at the base row, as the
    look request's fade is, a person's rises in the signal's and an
    agent's in ink, each tone one mark of the scene; a stop's small
    words over its own at 9, centred on the stop, whichever side of the
    line it stands.
  - *down the page,* narrower, in a column 360 pixels wide centred in the
    box, or, in a box under 360, a scene 360 wide that says its width and
    is scaled to the box, keeping its proportions, as the look request's
    narrow picture is: the dotted line down its middle at 178; a person's and an
    agent's marks at its right, at 206, their words after them; code's
    at its left, at 150, its words before them; a stop's small words
    under its own, 12 lower, on the same side; a stop every 40 pixels
    from 16, the landing branch's bar across the line's foot 38 under the
    last stop, its words before it; the scene's height its own, 14 under
    that bar (skein's `ui/scenes.md`, 2); the line's fades run across the
    column, from the marks' side to code's.
- **Pressed,** on Ask temper's page and How work runs alone (`../README.md`; `../settings.md`, 2): each stop but a bar is a
  group whose transparent area, 60 by 52 above the line and 60 by 44
  below it across, a row 28 high down the page, carries its act, so that
  a press anywhere on it opens the stop's card (skein's `ui/scenes.md`, 3);
  the open stop has a pale ring of 15 behind its mark and its words in
  ink, bold, and the scene grows at its foot by what is 14 pixels in the
  box (14 × `A` / `W` across, and 14 × 360 / `W` down the page in a box
  under 360, rounded to a whole pixel of the scene; 14 down the page
  otherwise), so that the card, following the drawing with no gap of
  its own (`../README.md`), stands 14 pixels under it as the request's does: across,
  that room holds the card's caret, a point under the open stop 6 pixels
  of the box high and 12 wide, its base on the scene's foot, where the
  card's top edge is; down the page there is no caret. Its fallback's lines are then buttons doing the same,
  expanded while their card shows, for the keyboard, a screen reader and
  the fake person. In a reply nothing is pressed.
- **Left to M3b** (`../../../draft/ask-temper.execution.md`): a stop pressed
  in a reply opening its card, the views `queue`, `task N tree` and
  `task N timeline`. The TUI prints views as lines of its own text
  (`../../tui/pages/chat.md`, 1, *a view*; `../../../draft/surfaces.md`, 11.5).

## 4. Open questions

- **Views after landing and M3b** (3): temper/ui compares a path, a
  way, a proposal's two paths and a charter's or a rule's card in a reply and in the
  chat's document at 1440, 1000 and 420 px, across and down the page,
  against the look request's pages, and walks a real Ask temper
  chat asked to show a way. M3b adds a stop pressed in a reply opening
  its card and the views `queue`, `task N tree` and `task N timeline`
  (`../../../draft/ask-temper.execution.md`, M3b); the TUI prints views and
  draws none (`../../tui/pages/chat.md`, 1). A view
  at an older revision, and a decided proposal's paths, wait for a read of
  the settings as a revision left them. Default: the focused tables and
  gallery samples, which establish no rendered look.

- **The line's words** (1, *its calls*; `../../../draft/chat-reading.md`, 7): the user
  reads the table's verbs on a real chat. `read_file` and the forge's other reads
  stay their names, as the draft's example has it, and so do `recall` and `message`,
  whose first members are a number or a body, which read badly as an argument.
  Default: the table as `../README.md` has it.
- **A run's time** (1, *its calls*; `../../../draft/chat-reading.md`, 7): a committed
  turn says only when it ended. Storing when it began would change a stored format
  and a protocol, for a figure the line before it gives nearly as well: it holds the
  model's writing of any text that opened the turn, and at a window's head or for a
  turn's second run is often no time at all. The user reads it on a real chat.
  Default: from the line before, as `../README.md` has it.
- **Which decision closed a chat** (`../../README.md`, 3.6, *Closed ends on one line*):
  `Activity::Decided` carries a revision alone, so the view can't tell the closing's
  from an earlier decision. Telling them apart takes a new fact of a protocol, the
  user's. Default: a chat its requester closed hides every "Decision made".
- **An answer past what the engine retains** (1, *the words being written*;
  `../../../draft/chat-reading.md`, 3): a page following it from its start shows it whole
  up to `long_text`. One opened later shows the end the engine retains, 65,536
  bytes, until the turn commits, since one delivery stops at the same bound (jig's
  `client/domain/client/README.md`, section 15). A gap between suffix snapshots keeps
  the held prefix incomplete until a full-origin snapshot or commitment, even
  below `long_text`; recovering the missing provisional bytes needs a read
  the thread does not ask for. Default: keep the prefix and its reading position.
- **A turn too large for a page** (`../../view.md`, 4; `../../../draft/chat-reading.md`, 4): a turn past
  `document_bytes` on the wire, 1,048,576 bytes, reads as too large, with no calls'
  names (jig's `client/domain/client/README.md`, section 15). Default: its line says so,
  and the tools read it by its parts.
- **A Reject on a chat's card** (`../settings.md`, 2; the request `ui-chat-accept` of 2026-11-06,
  which counts Reject among the card's acts): the mock's `chat.html` draws "Accept" and
  "Change something" alone, and the reason's box stands on the card in *Needs you*.
  Drawing one takes a view change and that box; its ask already goes under the
  proposal's own `project`. Default: none; a person rejects in *Needs you*, and
  asking for a change has the agent propose again (`../settings.md`, 2).
- **Calls naming their proposals** (`../README.md`, `../settings.md`, 2): filling a committed call's
  `objects` needs the core's turn record, its archived form and a mapping from
  the call's position to the proposal's number; proposals from a verdict or a
  procedure have no call. Default: keep the call anchor where one is supplied,
  and otherwise place the card by its making line, or its waiting time, as `../settings.md`, 2
  specifies. Filling those call objects is separate work.
- **Latest on a phone, checked by a press** (`../README.md`; the request `ui-proposal-2`
  of 2026-11-06): after landing, check that Latest's row leaves a card's
  controls reachable at a phone's width. Default: the stylesheet assertions
  hold its place in flow in both chats and agent pages; measuring and pressing
  in a browser follow landing.
- **The real-model proposal-card walk** (`../settings.md`, 2; the request `ui-proposal-card`
  of 2026-11-06): QA repeats the two-proposal, rejection from *Needs you* and
  reload walk at 1440 and 420 pixels on a real model after landing. Default:
  that check follows the fake-box render proof; scripted stories alone do not
  settle what a person meets with a real model.
- **The first use of a plan,** in the terminal: a card in the
  conversation, or a card that waits to be dismissed, another kind of
  `Dismissed`, keyed by its account (`../../../draft/to-begin.md`, section 5).
  Default: a card in the conversation (`../accounts.md`, 2), with nothing to dismiss.
- **A delivery beside a long conversation** (2;
  `../../../draft/chat-reading.md`, 3): retaining parsed closed blocks for the
  browser and TUI, or a conversation subtree, needs invalidation rules
  and proofs; reducing page bookkeeping and
  away-reader anchor scans needs the same. Owner: temper/ui for the shared
  reader. Default: rebuild and diff, keeping
  DOM identities. The measured whole-bound delivery cost is in
  `temper/docs/development/browser-budget.md`; the away-reader scan's share
  remains untimed. The browser tier's smaller window leaves a person's cost
  at the whole bound in place.
- **A turn with a call before its first text** (`../README.md`, `conversation.md`, 2;
  `../../../draft/chat-reading.md`, 3): keeping that text's nodes through commitment
  needs its key independent of the preceding call or card's place. Default:
  keep the current block keys until this ordering is reproduced; the
  `ui-pgup` request uses a render driver at 1440 and 420 px for its proof,
  with no new browser story, since the tier is at its budget.
- **The rest of a run's line** (`../README.md`, `../task.md`, 2; `../../../draft/chat-reading.md`, 2 and 7): words
  for tools past the table, which are the user's to read on a real chat. Default: any
  other tool is its name. A run of one turn has its time as *A run's time* says, and an
  agent's page has a running run and Latest as a chat's, sending not pressing it.
- **Which person messages are the person's** (`../README.md`): jig's turns keep no
  provenance, so a chat draws none of a turn's person messages, and
  the person's words from the thread's words alone. A charter whose
  person's words reach the chat only inside a turn would show none of
  them. Default: as drawn; a turn's message kind from smith's record
  is the candidate, jig's.
- **Seen while no one looks:** a chat's page open in a tab the person
  is not looking at, or in a terminal left open, marks the turns that
  arrive as seen (`../README.md`). Default: open is seen; the page's visibility,
  an input of the browser's shell, is the candidate.
- **A delegation's tasks, keyed** (`../../view.md`, 5): jig's thread keeps a
  `delegate` call's `objects` empty, so the domain reads the tasks
  from temper's acknowledgement and, at a result, from temper's inbox
  words: words of the engine's that the client depends on, each marked
  so where it is written and where it is read. Default: the objects
  when jig's thread keys them (a jig change and its repin), the words
  meanwhile, both ends temper's and joined by the process tier's first
  run (`../../testing.md`).
- **A change's live stages** on its card (`../../view.md`, 5; `../../forge/README.md`, 2.1): the
  steps are a stand-in, ticked from the result. The engine publishing
  the change's step as its task's part, which the forge's side reads,
  is the candidate, about 1.5 to 3 thousand lines. Default: the
  stand-in. The change's task page draws its live step from its panel
  (`../task.md`, 3); the card does not watch it.
- **"Landed into main":** the
  branch a change lands into is a parameter of the forge's procedure,
  which jig's contract does not type, and the landing words name the
  merge commit alone. Default: "Landed · merge commit …"; the result's words
  naming the branch are the candidate.
- **"CI passed" on a card** says what the stand-in's steps say: a pull
  request merged by someone else is landed like any landing
  (`../../forge/README.md`, 5.2), CI or not. Default: the words, until the live
  stages.
