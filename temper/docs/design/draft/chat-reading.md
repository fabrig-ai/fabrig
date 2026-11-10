# Reading a chat

Draft, 2026-11-06, from a session with the user that day, who tried a
box on real accounts and recorded what they saw. A chat showed "a lot of
unuseful text ... ('Earlier text omitted', tool calls not being
coalesced/aggregated in a single short line with minimal real-estate)",
its text streamed unlike Claude's TUI, "Older" was in the way ("just
PgUp should show older lines ... the default size of the content being
shown should be larger"), and after enough "Older" the conversation's
end was gone. Shown a mock of what follows, the user said: "Exactly what
I'd like to see. Go for it", and took each recommendation here. The look
request `chat-work` draws it (`../client/journeys/chat-work/`). This
draft carries what the look cannot.

## 1. In one page

- **A run of calls is one line in words,** "Read 7 docs, checked settings
  3×  29 s", and opens to a row a call (section 2).
- **The answer only grows** while it is written, whole, and takes its
  committed place without moving. Reasoning is not drawn (section 3).
- **A committed turn keeps its calls' names,** whatever their results
  weigh, and a conversation says nothing of what was cut (section 4).
- **A conversation is read by scrolling:** no "Older", PgUp and PgDn from
  anywhere, a window bounded by size, the newest lines never lost, and
  "Latest ↓" while the end is out of view (section 5).
- **A numbered list is a numbered list** (section 6).

## 2. The line

- **A run:** every call between two of the agent's texts, across the
  turns that made them. A text, a person's words or a card ends it.
- **Its words:** one phrase a tool, in the order first called, joined by
  commas, the first capitalised: "Read 7 docs, checked settings 3×".
  - A tool with words of its own counts what it touched, not its calls:
    seven documents read through thirteen calls are "Read 7 docs". The
    words are the view's table, tool by tool, starting from the tools a
    chat calls most.
  - Any other tool is its name, counted: "read_file ×2".
  - One call alone reads the same way: "Read the page hold".
  - A failed call adds ", 1 failed", in the signal's colour.
- **Its time:** from the turn that opened the run to the turn that ended
  it, as the committed turns say; no time where they do not say.
- **While it runs:** the same line counts what has ended and says the
  call being made, from the run's `call`: "checking template 2…".
- **Opened:** a row a call: the tool, its argument in a few words, and
  its outcome's first line, cut to the row. Never a result's body.
- **What stays apart:** a call with a card of its own (a delegate, a
  change, a goal, a proposal; `../client/README.md`, 5.7) keeps its card,
  and a document's write keeps its line and "Show". Neither counts in a
  run.
- **The terminal** prints the line and, opened, its rows.

## 3. The answer while it is written

- **Whole:** the client keeps the turn being written up to `long_text`
  (131 KB today), not `text`, the 512 bytes of a field. Today a long
  answer is seen through a 512-byte tail that moves, and that is most of
  what looked strange to the user.
- **Drawn once:** a block closed by a blank line is read once and stays
  on the page unchanged. Only the last block is read again as words
  arrive.
- **No raw markers:** the last block is read as if it were closed. A bold
  still open is bold; a list's or a heading's marker still being written
  ("4.", "##") waits for its words; a code fence still open is code.
  skein's reader shows a delimiter never closed as text
  (`skein/docs/design/ui.md`, section 10), so it takes the growing case
  as an argument.
- **Its end:** a small block after the last word, in place of today's
  "Writing...".
- **Committed without moving:** the committed turn's blocks are the same
  blocks, so its place on the page does not change. Only the end's mark
  goes.
- **Before the first call or word,** the three dots, as today.
- **Reasoning:** a turn's opaque blocks ("Content in 3", "2164 bytes") are
  not drawn in a conversation, and nothing stands in their place.

## 4. What a committed turn keeps

Today's `turn_text` (2048 bytes) is shared by the turn's texts, its calls'
names, inputs and outcomes, and its results' summaries, an outcome taken
before its input (jig's `client/domain/client/README.md`, section 11). A few
results use it up, and the later calls arrive with no names: "read_docs ×2
· ". The answer in the user's recording, 1978 bytes, was cut by it.

- **The order changes:** the calls' names first, then the texts, then
  inputs, and outcomes and results' summaries last. A name is never cut.
- **The budget grows** to `long_text`, so an answer commits whole, as it
  was written. `turn_bytes` and the window's bytes hold it.
- **An outcome, on the way to a conversation, is its first line.** The
  page draws no more, and the rest is what used the budget. Where the
  view reads a task's number from a result's summary (a delegate's card),
  that summary stays whole.
- **No notice:** "Earlier text omitted" leaves the conversation. A text
  still cut at the bound ends in "…", where it was cut.

## 5. Reading

- **No "Older".** Earlier lines load when the conversation's head comes
  within one and a half screens of the view, whatever brought it there:
  PgUp, Home, a wheel, a finger. A generic mechanism, so skein first: a
  log node says, as an act, that its head is near.
- **The same rule opens a chat:** a page opened at the end reads earlier
  pages until the head is that far off. A long chat opens with at least a
  screen and a half; a short one shows whole.
- **The position stays:** lines loaded above leave the words being read
  where they were on the screen, in every browser.
- **While loading:** "Loading earlier messages…", faint and centred at
  the head, only while a read is out. Nothing once the chat's start is
  shown. A failed read says so there and tries again as the head nears.
- **Keys:** PgUp and PgDn scroll the conversation from anywhere on the
  page, the composer included, a menu or dialog open excepted. In the
  terminal, the same.
- **A window bounded by size:** the conversation's lines fill
  `document_bytes` (1 MB today). The 16 lines go, and the `lines` bound
  with them, or it grows far past what the bytes hold.
- **The newest are never lost:** past the bound, the lines furthest from
  where the person reads go, at either end, whole, and come back when
  scrolled to, through the thread's `newer` as through its `older` (jig's
  `client/domain/client/conversations.md`). Nothing says how many left.
- **"Latest ↓":** a small pill over the pane's foot while the end is out
  of view. Pressed, or on the person sending, the view goes to the end,
  read again if the window let it go.
- **New words while reading back** do not move the view.

## 6. Numbered lists

skein's reader reads an ordered list as a bullet list whose items begin
with their number, so a numbered item shows a bullet and "1. ". A list
node says it is ordered and from what number; the web shell draws
`<ol start>`, and the terminal the numbers.

## 7. Open questions

- **The line's verbs:** the table's first tools and their words are the
  landing's, to be read by the user on a real chat.
- **The turn's time,** when committed turns give only when each ended: a
  run of one turn has no start to measure from.
