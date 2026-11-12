# A chat

## 1. A chat

Illustrative (the request's 03, with the look request
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
  page holds them (`../../pages/README.md`, `Chat { task }`; `pages().chat()`).
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
    (`../../pages/chat/README.md`, 1, *A chat's heading is short*; `shared::heading`), cut to at
    most 20 characters: whole where it has 20 or fewer; else its words
    from the start, as many as fit with "…" after them in 20, its
    trailing spaces dropped, then "…" ("What can you tell…"); where its
    first word alone does not fit, its first 19 characters and "…". A
    character is a Unicode scalar value.
  - *The place,* `temper_client_tui::place(address, boxed)`, the one
    function that words it, which `temper box`'s own header may call
    too: on a box (`../README.md`, 2, *On a box*) "box" where the address's port is
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
  entry, in the thread's order as the browser's terminal reads it (`../../pages/chat/terminal.md`, 1, *The
  conversation*), with a blank row between two entries
  (`tui-spaced`). Each
  entry's glyph and its words' indent are its token's look (`../theme.md`):
  - *the person's words,* the chat's request and each words line a
    person sent: "❯", then each paragraph (`tui-said`): on the band
    across the row, the mark dim, where the terminal answered its
    background; else in the dim colour after an accent "❯", as Claude
    Code draws its prompts (`../theme.md`, *The band*);
  - *a delegate's words:* "✦", "TN · " and its words (`tui-agent`);
  - *a run of calls,* as the browser's terminal reads one (`../../pages/chat/conversation.md`, 1, *its
    calls*): "▸", its name and, two spaces after, its description, the
    run's time, "1 failed · 29 s" in `tui-run-failed`, or the call under
    way with "…" while it runs (`tui-run`), from the view's run line
    (`shared::Run`, `shared::RunLine`), so that it counts what has ended
    and names the call being made. Its rows are not drawn (`../../README.md`, section 5);
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
    TUI work (`../../README.md`, section 5);
  - *a view,* printed, not drawn (`../../../draft/surfaces.md`, 11.5; the
    user, 2026-11-08): an agent's committed text is split at its `view`
    blocks by the domain's `views::parts`, as the browser's is (`../../pages/chat/conversation.md`, 3),
    its text parts read as Markdown and each view printed in its place,
    inside the entry's group, as lines of the terminal's own text with no
    picture, from the chat page's read of its address
    (`Chat::view`, `../../pages/chat/conversation.md`, 3, *Read when shown*). The words being written are
    read as they are until the turn commits. `view::printed`, the one
    function that prints a view, is what a chat's document calls too
    where the TUI draws one:
    - *a path:* a stop a paragraph, in the derivation's order (the
      wire's `stops`): its mark, then a space and its words as the
      derivation gives them, which the TUI never spells (they change
      with the page How work runs, `../../../draft/surfaces.md`, 13),
      then, where it has them, a space and its small words in brackets,
      as the web's text says them, "you (if protocol core)"
      (`../../../domain/forge/change/branch-path.md`, *Drawn apart*). The mark
      is the web's for its actor: a ring `○` for a person
      (`tui-signal`, the person's colour alone), a four-point star `✦`
      for an agent (no paint, the ink, as `../../../draft/surfaces.md`, 13,
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
      card stays the web's (`../../pages/ask-temper.md`, 1, Ask temper's page).
  - *not drawn,* as in the browser's terminal: the speakers' names, the
    turns' times, a turn's opaque blocks, a run's beginning and end;
    and, the TUI's, a document's write, since the pane shows the draft
    and marks what the turn wrote (`design-session.md`, section 1), and "Being
    sent" and "Sent", the conversation showing the words.
  The terminal's notes follow the conversation, each in `tui-note`.
- **Reading back:** PgUp and PgDn page the body by its rows while the
  box keeps the focus (skein's `ui/screen.md`, 6.2); the body follows the
  conversation's end as it grows, until the person pages up, and again
  once they page down to it (`../../view.md`, 2 there). The window is the one the
  chat's page reads as it opens: earlier pages are not read as the head
  nears, since skein's screen reports no log edge yet (its section 10),
  and there is no "Latest ↓" (`../../README.md`, section 5).
- **The foot:** the running line, the box, "Reply", and the quiet line,
  "paused · a message resumes it" on a chat the person paused. On a
  chat that has ended the box takes commands alone, and words that are
  no command note "This chat is closed. /new starts another.", the
  root's note (`../../pages/chat/terminal.md`, 1, *The box*).

## 2. The chat's question

Illustrative (the request's 04):

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
  subtree's questions (`../../pages/chat/terminal.md`, 1, *A goal's question*), and the TUI draws the
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
  something else" last, as `../../../draft/tui.md`, section 3 asks, and has no
  Later, since the box is there for an answer in the person's words.
