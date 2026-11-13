# A chat's document

## 1. Its document

- **A chat's document** (the greenfield mock's `chat.html`; the look request
  `configure-chat-draft`, for every chat): a chat keeps one Markdown document
  beside its conversation, which its agent writes (`../../../domain/agent/agents.md`, 3).
  It is the lowest-numbered note in the chat's own scope whose description
  does not begin `work:`; working notes are never drawn, and a chat with only
  working notes has no document. Its description is its title, its body Markdown,
  its name the mock's `slug` of the title, 28 characters, plus `.md`:
  `away-mode.md`. There are no file drafts: a document
  is always a chat's. Ask temper's agent writes it with `document`
  (`../../../domain/agent/ask-temper.md`, 3), whose answer's first line is a `note` answer's:
  wherever this part names a `note` call, a `document` call is read alike, by
  that line.
  - *Offered:* `pages::drafts_offered(root, project)` is that project's kind 7
    `drafts` (`../settings.md`, 2), true in the stock. It gates the Drafts filter, the Chats
    lede's words, "Start goal…", the rail's "Goal", and the view group and "Write it
    up from this chat" on a chat that is no Ask temper chat with no document. An Ask
    temper chat offers a draft whatever `drafts` says:
    `pages::chat_offers_draft(root, task)`, which the bar, the pane and the domain's taking of
    the write-up all read, is true on an Ask temper chat and otherwise `drafts_offered`.
  - *A design session* (`../../../domain/agent/agents.md`, 4): a chat whose chip's executor
    is the charter of template 6 as `me` lists it, as an Ask temper chat is
    template 5's (`../home.md`, 1, *To begin*), `pages::chats::drafting(me, project, executor)`.
    It reads as any chat with a document, its bar's modes, its tool lines and
    "Start goal…" with `../goal.md`, 3's ways, and is offered a draft as any chat is.
    Opened from a list or a link, it opens in Split rather than Chat, its document
    read as any chat's, until the person picks a mode on the page.
  - *Read live, with no new watch:* `Read::Notes { scope: NoteScope::Task {
    project, task }, description: None, after: None }`, on the same retries as an
    activity read, once the task's document has arrived, and again when the
    thread shows a `note` call not seen before and when the run's state moves off
    a call to `note`. One read is out at a time; a read wanted while one is out
    is asked once more on its answer. The page watches the task and its run, and
    only the task's own run writes the note, so nothing else moves it. The newest
    answer replaces the document, and a read refused or empty leaves it as it
    was.
  - *Its mode:* `Mode { Chat, Split, Draft }`, the page's state for that chat, kept
    while the page is open, with or without a document; a chat opened again starts in Chat, unless `NewChat {
    draft }` opened it in Split or it is a design session (above). Nothing stores it; the terminal's look, which is
    another thing, is kept per browser (`terminal.md`, 1).
  - *The tool line:* a `note` call whose answer parses (`../../../domain/agent/agents.md`,
    3) is a `.turn-tool` in the conversation: the pen, "Wrote" for a first
    write, else "Edited"; `.file` NAME; a `.turn-tool` detail, "64 lines" or "+18
    −6"; "Show", a button. It isn't counted in a run, and ends one; a failed one is a call of its
    run. The domain parses the words, as
    `delegation::results` does. "Show" moves to Split (on a phone, Draft) unless
    already in Split or Draft, and moves the focus to the section holding that
    call's `first`, which brings it into view and flashes it (`.doc-sec.flash`,
    `@keyframes flash`).
  - *The bar* (`README.md`, 1, *A chat's bar*; the look request `chat-bar`,
    2026-11-12): at its far right, `.seg`, a group named "The
    draft", "Chat", "Split" and "Draft", each its icon (`chat.svg`, `split.svg`,
    `doc.svg`), the chosen `aria-pressed`, on every chat with a document, and on
    one with none while `chat_offers_draft` and the chat hasn't ended: where
    "+ Draft" stood, which no bar draws now. On `NewChat`, while
    `drafts_offered`, the same group: Chat is `NewChat { draft: false }`, Split
    and Draft `NewChat { draft: true }` (`README.md`, 2, *With a draft*). `Ask`,
    before an Ask temper chat's first message, has nothing to write up and
    draws none. `Ctrl+\`, skein's primary key (⌘ on a Mac), steps through
    those shown, the next mode's button carrying the shortcut. On
    a phone, Chat and Draft alone; Split left on a wide screen shows as Draft.
    Before the group, `.btn.primary.small` "Start goal…" once the document has a section,
    on a chat of a project with a home repository with no goal linked; with a
    linked goal, a link `.btn.small` "Goal TITLE →" to `Task { task }`. Not on an
    Ask temper chat, which has "Start over" in their place, before Pause and
    Close, on every Ask temper chat that hasn't ended, with or without a
    document: it asks "Start over?", "A new
    Ask temper chat starts. This one stays in Chats." through `crate::confirm`
    (`../../README.md`, 3.6, *The sheet*), closes the chat with
    the chat's own close, which stays in Chats, Closed, its conversation and
    document kept, and opens `Ask { project }`. The composer's hint gains
    " · Ctrl+\ switches the draft" while the group shows.
  - *The layout,* one for both kinds of chat, from `configure-chat-draft`'s rules:
    `temper-mode-chat`, `-split` and `-draft` on the session. Chat: the
    conversation alone. Split: the conversation and the document in two halves, the
    document on white behind a rule, the rail folded to a 56-pixel strip holding the
    star, a link to `Home`, and "Show the rail", drawn as `files/menu.svg`, which
    opens the rail over the page until a press outside it or Escape. Draft: the
    document alone, the rail back, the composer kept at the foot on white. The pane
    keeps a scrollbar's gutter on both edges, so that its column and the composer
    share a centre. On a phone the document takes the screen's width. In the
    terminal, in Chat, the conversation alone; in Split or Draft, the document after
    it as a region named its title.
  - *The pane,* `.temper-doc`, `.temper-doc-inner`: a column of at most 640 pixels.
    Its line, `.temper-doc-path`: at its left `docs/design/draft/NAME.md`, or on an
    Ask temper chat "T7 · not proposed yet", "Proposal 12 · waits for an owner" or
    "Accepted at 14:03", from its proposal cards; at its right "edited 1 min ago",
    `TimeForm::Ago`, lower case. The body is split at each `#` and `##` line, each
    section read by `skein_ui::markdown::read(.., 1, b)` into a slot, `.temper-doc-sec`, keyed
    by its heading and its place among its namesakes, so that a change keeps the others'
    nodes. `#`, level 2, is drawn as the mock's `.doc-inner h1`, and `##`, level 3, as its `h2`; the first section's first
    paragraph as `.doc-muted`. Tables as `configure-chat-draft`'s. A fenced block
    reads as code, but for a flow (`skein/docs/design/ui/markdown.md`, 1), which is drawn.
    temper's page routes skein-ui's owner to its drawing code and gives its names
    beside temper's, so a flow draws in any Markdown a person reads here, a turn's as
    a document's. Its look is `../../web/look.md`'s.
  - *The tint,* `.temper-doc-changed`: the section holding the newest `note` call's
    `first`, or for a first write the first `##` section; under it `.temper-doc-note`
    "Added by T4 3 min ago" or "Changed by T4 just now".
  - *Empty,* Split or Draft on a chat with no document: the pane, a region named
    "Draft"; its path's left, on an Ask temper chat "TN · no draft yet", on another
    chat the mock's `untitled.md`; its right empty, or "writing" once the write-up
    is asked; `.temper-doc-empty` "The draft appears here as you write it
    together."; under it `.temper-doc-acts` with "Write it up from this chat",
    `.temper-doc-write-up`, while the chat hasn't ended and no write-up is asked.
    `NewChat { draft: true }` draws the empty pane as before, its path
    `untitled.md` and "writing", with no button: its first message asks the
    write-up (`README.md`, 2, *With a draft*).
  - *"Write it up from this chat"* (the look request `chat-bar`): sends the
    chat the message "Write this up as a draft beside us.", the person's own,
    shown in the chat as any other: one act, one request (`../../README.md`,
    3.6). It is the domain's write-up act, `AddDraft`: the chat is writing,
    the pane's right says so and the button goes, the mode it was pressed in
    kept (from Chat, the terminal look's "+ Draft", it goes to Split,
    `terminal.md`, 1, *The top line*); refused, nothing is writing, the button
    stays and the composer says why, as for a message. On a chat only this
    act sends those words: choosing a mode sends nothing. The `.meta-line`,
    when the thread is empty: "A chat that writes NAME. Say what it should
    hold."
- **Starting a goal from a chat's document** (the greenfield mock's `startSheet` and
  `startGoal`): "Start goal…" opens the one sheet to start a goal (`../goal.md`, 3), given
  `docs/design/draft/NAME.md`, the chat and a snapshot of the document's body; its
  "Name" is the slug without `.md`, and its ways and default are `../goal.md`, 3's.
  - *Sent:* its words, within `long_text` on the client's side and `tasks.spec_bytes`
    on the engine's, refused in place if over, are "NAME", a blank line, "Implement
    this draft. First commit it, as it stands, at docs/design/draft/SLUG.md.", a blank
    line, the body, a blank line and the line `goal`'s root gives a goal, so that
    charter 1's instructions read it as a goal (`../../../domain/agent/tools.md`, 2), a
    constant the root and the client reach, in `temper-client-protocol-wire`; then
    `../goal.md`, 3's `Draft:` and `Chat:` lines, and `Asks: always` where it is chosen.
  - *Answered:* `../goal.md`, 3's ends; on `Judged::Create` the fact `GoalFrom` is sent (`../../forge/records.md`)
    and the goal's `Task { task }` opens.
  - *After:* the fact's chat shows `.turn-tool`: pen, "Started goal", `.file` TITLE, its
    state, "Open", placed among the turns by the fact's `at`. Goals an
    agent started with the `goal` tool keep their line in the conversation (`../../view.md`, 5).

## 2. Open questions

- **A document's drawing after landing** (`../README.md`): temper/ui looks at a flow
  in a chat and in its document, including both layouts and the narrow
  fallback (`../../web/README.md`, section 3). Default: skein's acceptance cases
  and temper's host test hold the scene's bytes; the rendered comparison
  follows landing.

- **The fresh project's draft flow after landing** (`../README.md`, `../../view.md`, 3, `../settings.md`, 2):
  checking an Ask temper chat with drafts off, and a fresh chat's draft,
  goal and navigation on a real box. The view tables and `chat-draft`'s
  fake-model journey cover the offers and flow. Default: browser, walk and
  render checks follow landing, under the `ui-drafts-default` request;
  they are not conditions for completing its plan.

- **The sheet the request `ui-chat-draft` of 2026-11-05 draws** (`../README.md`, `../goal.md`, 3): its
  "Budget", in the deployment's unit, and its ways by their fixed words, "Plan, tasks,
  land whole", "Task by task" and "Plan, then wait for me", the last sent by
  `shapes::set_agreed_goal_from`, which stands built with nothing sending it; and a
  proposed goal's line above the composer. Default: `../goal.md`, 3's one sheet, its budget the
  creating bound's, its ways the project's templates, a proposal its toast.
