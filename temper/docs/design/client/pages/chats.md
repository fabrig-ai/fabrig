# The Chats page

## 1. The page

- **The Chats page** (the greenfield mock's `chats.html`): `Chats { filter }`,
  a page of the frame and no pane, a main with the token `CHATS_PAGE`,
  drawn as the mock's `.main` and `.col`, in this order:
  - *Its head,* `.intro`: `.kicker` "PROJECT · chats"; the level-1
    `.headline` "Talk to an agent about OWNER/NAME.", OWNER/NAME the
    project's home repository, read as Home reads it, or "Talk to an
    agent." with none; `.lede` "A chat answers a question or changes the
    code.", or, while `drafts_offered`, "A chat answers a question or changes
    the code, and can write a draft beside it. A draft becomes a goal for bigger
    work."
  - *With no repository* (`home.md`, 1, *A project with no repository*), `.band.quiet-band`:
    `.tag` "No repository yet", "Chats in PROJECT talk and plan, but change
    no code until a repository is in." and a `.btn.small` link "Adopt a
    repository", to `Adopt { project }`.
  - *The list,* `.section`: `.section-head` with `.filters` at its left,
    "All", "Live", "Drafts" while `drafts_offered`, and "Closed",
    pressed-or-not buttons that filter in the page, reading nothing, the filter
    being in the page's address (`../web/README.md`, 2.3), a filter chosen a move, as
    the inbox's; and `.btn.small` "+ New chat" at its right, a link to `NewChat`.
    Drafts holds the rows whose chat has a document. The page reads
    `NoteScope::Tasks { project }` as it opens, bodies left out, and watches the
    person's facts; a note written later is seen on the next open.
  - *Its rows,* `.goal-rows` and `.goal-row`: the live group's lines by last
    activity and the ended page's by when each ended, merged newest first,
    a tie by the lower number first, each row one link: its number's, whose
    `::after` covers the row. A row's top line, `.top.mono`, holds at its
    left "T9", then for an Ask temper chat " · Ask temper", for a chat with a
    document " · writes away-mode.md", then for a goal the person started from it
    " · started goal TITLE", TITLE from the goal's chip by `Subject::Chips`, and at
    its right its status, " · " and its age. The status is the bar's (one word for
    one state in both places, and "Closed" for every ended chat, as the
    filter says it; the user's pick, 10-23), in lower case by the look
    alone (the terminal keeps its words), `.state-needs` while it waits on
    the person: the tokens `STATUS_YOURS` and `STATUS_WAITING`. Its age is
    Home's short form (`../view.md`, 6, *span*): when it ended or was last active.
    Under it `.line`, its title, `.goal-row.ended .line` on an ended chat.
    A row names no project: the page is one project's. Its own spend, "N
    spent", "1 delegate" or "N delegates" while it has live ones, and on an
    ended line whose retention is Kept "Closed. Its transcript is kept."
    stay in the view and are hidden in the browser; no other line claims
    that its transcript stays.
  - *With no line to show,* once its reads have answered, `.empty`, "None.",
    under every filter.
  - *"More closed chats",* under "All" or "Closed" while an older page of
    ended chats remains, a `.btn.quiet.small` under the rows, reads it, in
    place of the last.

  On a phone, as the mock: `.headline` 25 px, `.goal-row { padding: 12px 0
  }`, `.goal-row .line { font-size: 15px }`; the frame's top bar and tab bar
  stay.

## 2. Open questions

- **What a chat did, on its line** (the same request): its chat's line
  says "landed · 1 h" where the chat's delegates landed. The chats' list
  gives a chat's state, not what its delegates landed, so saying it takes
  one read per row. Default: the chat's state as `Chats` words it,
  "closed · 1 h".
