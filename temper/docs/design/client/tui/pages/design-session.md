# A design session

## 1. A design session

Illustrative (`tui/whole-story/design-session.html`,
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
  `pages::chats::drafting(me, project, executor)` (`../../pages/README.md`, *A design
  session*; `../../../domain/agent/agents.md`, 4), read as any chat (`chat.md`, section 1,
  above), with its draft beside the conversation, the drafter's
  questions as prompts, `/goal`, and a quiet line of its own
  (`../../../draft/surfaces.md`, 11.2; `../../../draft/tui-design-session.md`). Any
  other chat with a document has the pane, its marks, its replies'
  choices, `/goal` and the quiet line's pane parts too, but not
  "drafter".
- **Starting one:** `/draft WORDS` on the launchpad (`README.md`, section 2, *The TUI's own
  commands*), template 6 where `me` lists it and a chat otherwise, as
  the browser's new chat with a draft; it opens with the pane at half,
  as the domain's start leaves it (`Mode::Split`, `../../pages/README.md`, *A new chat*).
- **Sittings:** a session is kept (`../../../domain/agent/agents.md`, 4). Opened
  again, from `launchpad.md`, section 1, *Where you left off*, `/chats N` or `/chats`, it is the
  root's chat as any chat is, its conversation and draft read as they
  stand, and the domain opens a design session with its pane at half
  (`../../pages/README.md`, *A design session*); `launchpad.md`, section 1, *Where you left off* says ", a design
  session" for one (`launchpad.md`, section 1).
- **The pane's states,** the chat page's mode as the root keeps it
  (`pages::chat::Chat::mode`, `../../pages/README.md`, *Its mode*), of which the TUI keeps
  nothing: `Mode::Chat`, folded, the conversation alone; `Mode::Split`,
  half, the conversation in the body and the draft in the side region
  `tui-pane` beside it (`../theme.md`; skein's `ui/screen.md`, 5.2, *A side
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
    (`README.md`, section 2, *Offered*).
  - *The body as the state changes:* entering all, the host sets the
    body's scroll to its first row and stops following its end (skein's
    `ui/screen.md`, 5.2, `Focus::scroll` and `Focus::follow`); leaving
    all, it follows the conversation's end again: the draft reads from
    its head, the conversation from its newest words. PgUp and PgDn page
    the body in each state; at half the pane shows what fits and does
    not scroll (skein's `ui/screen.md`, 5.2, *A side region*), all
    being the place to read the draft whole.
- **The draft,** at half and at all alike, from the chat's document as
  the web reads it (`../../pages/README.md`, *A chat's document*: the same reads, no new
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
    (`tui-hint`), the browser's words (`../../pages/README.md`, *Empty*).
  - *not drawn:* the web's tint, its note "Changed by T4 just now" and
    "Show"; the path's `docs/design/draft/`; and in the conversation a
    write's line (`chat.md`, section 1, *not drawn*).
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
    else", the TUI's words (`chat.md`, section 2). A held question of
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
- **`/goal`,** the root's `Command::Goal` where `README.md`, section 2, *Offered* says, starts a
  goal from the draft through the domain's start sheet, as the web's
  "Start goal…" does (`../../pages/README.md`, *Starting a goal from a chat's document*;
  `../../pages/goal.md`, 3):
  - *Pressed or sent:* the host makes `Act::OpenStart { draft: Some(PATH),
    chat: Some(TASK) }`, PATH the draft's path,
    `pages::chat::sheet::path(&document::slug(title))`, never
    `terminal::Act::Run`, so that the root's sheet opens with the chat's
    checks and the project's preferred way; `Tui` keeps the chat.
  - *Its prompt,* while the root's sheet stands (`Pages::start`), as a
    command's list (`README.md`, section 2, *A command's list*): its head "Start a goal from
    FILE. How should it run?" (`tui-list-head`), FILE the path's last
    segment; its choices the project's ways, kind 7's tracked templates
    (`temper_client_view::pages::goals::ways(me, project, words)`),
    numbered, the sheet's chosen way, the preferred one, first and
    active, the others in number order, each its name and its line as
    its words; then "Cancel". Esc is Cancel, `Act::StartCancel`.
  - *A way chosen:* `Event::Way { template }`, `Act::StartWay {
    template }` then `Act::StartGoal`, the sheet's name its default and
    its asking "As the project says" (`../../pages/goal.md`, 3). A refusal, or nothing to
    fund it (`StartGoal::error`, `start_goal::unfunded`), is drawn under
    the box in `tui-refused`, in the view's words for the sheet's error,
    and the prompt stays.
  - *Made:* the root opens the goal's `Task { task }`; the TUI has no
    goal's page, so the host, while `Tui` keeps the chat, opens that
    chat again (`Act::Go { address: Chat { task } }`) and lets it go.
    *Proposed:* the sheet closes, and the notice says "Proposed: the goal
    starts once it is accepted." (`README.md`, section 1, *The notice*).
- **A linked goal,** the newest goal started from this chat's draft
  (`pages::chat::Chat::goal_from`, its chip `Chat::goal_chip`): `/goal`
  is no longer offered, the quiet line names the goal, and `/open` opens
  its page (`README.md`, section 2, *The TUI's own commands*).
- **The quiet line,** on a chat whose pane can show (`README.md`, section 1, *The
  quiet line*):
  - *at its left,* in a design session: the project's name, " · ",
    "drafter" (`tui-hint`), " · " and the closed line, "Opus `../../pages/accounts.md`, 2 ·
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
    with its " · ", then as `README.md`, section 1, *A narrow terminal* says.
- **Where it departs from its mockup** (`tui/whole-story/design-session.html`;
  `../../../draft/surfaces.md`, 13), each for a reason:
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
    request asks (`../../../draft/tui-design-session.md`), as well as in the
    reply, where the mockup prints them in the reply alone; and the
    last says "Tell it something else", the TUI's words;
  - the quiet line at half says "Ctrl-P widens", where the mockup has
    "Ctrl-P folds", since its words say what the next press does; at 80
    columns the goal's part is left out, which the mockup's 110 hold;
    and the link's state shows while it is not Live, which the mockup,
    live, leaves out;
  - the halves split the window evenly, 55 and 55 columns at 110,
    where the mockup gives the conversation 56 and the pane 54.
