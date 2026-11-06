# chat-reading-d: the plan

The request: `inputs/request.md`, section 6 of `temper/docs/design/draft/chat-reading.md`. One task, 70 implementer-minutes, in skein alone.

| part, task | what | needs | minutes |
|---|---|---|---|
| `numbered-lists` | the reader, the tree's order, `<ol start>`, the terminal's numbers | none | 70 |

**Critical path:** `numbered-lists`. `walk = false`: the user reads it on a real chat.

The design it builds to is in this push: `skein/docs/design/ui.md`, 3.2, 4.1, 10, 16 and 18; `ui-web.md`, 6 and 7; `ui-text.md`, 5 and 8; their status files; and entry 6 of `temper/docs/design/draft/chat-reading.status.md` closed.

## What didn't hold, and the nearest thing taken

- **`<ol start>` alone can't show every number as written.** The draft gives a list one first number; the request wants its numbers as written; `1.` then `3.` in one `<ol>` would show 1 and 2. So a number that isn't the next opens another list from that number. Both hold, with no number on an item, and `1.`, `1.`, `1.` shows as written, three lists of one, where CommonMark would show 1, 2, 3.
- **An order isn't vocabulary by the criterion** (`skein/docs/design/ui.md`, 4.1). Of the platforms it counts, only HTML has an ordered list; ARIA, iOS and Android have none. So it isn't the light path. It goes in as a second exception, said in words like completed, with the draft as its authority, and iOS and Android draw the number before the item's words.

## What was decided

- **A line that merely starts with a number:** not of the form, always text (`3.14`, `3 apples`, `3) three`, `1.` alone, `007. x`, ten digits). Of the form within a paragraph, text too, as a wrapped sentence may start with a year: only `1.`, or the next number of the last numbered list closed, ends a paragraph. The second half is needed because this reader ends a list at a line under an item, the way agents write them: `1.`, a line under it, `2.` must still read 1 and 2.
- **What stays a list:** `2026. A year.` alone on its line is a list from 2026. The reader can't read a number's sense; it shows exactly as written, so nothing is lost.
- **A leading zero is text,** departing from CommonMark, which would show `007.` as 7: as written is the request's word.
- **A blank line between items makes two lists,** as today; each says its own number. Reading a line or a paragraph into its item is in `ui.md`'s Later.
- **Terminal:** `4. ` replaces `- ` and the other lines keep the bullet's two-space indent, so the shell's bounds change only by the number's ten bytes.
- **No browser tier:** `page/web.rs` is untouched, a changed tag already takes `Op::Replace`, and `<ol start>` is the browser's.

## What the request asked to have said

- **hearth:** skein's Markdown isn't on hearth's surface. `cargo tree -i skein-ui --workspace` names no hearth package.
- **temper's stylesheet,** left to the look's owner, as it falls in `temper-client-shell-web`:
  - **`temper/crates/temper-client-shell-web/temper.css:1323` hides the numbers.** `#app > .temper-session .temper-thread-inner .temper-card:not(.temper-task-card) li { display: flex; ... }` isn't scoped to a `ul`, and an `<li>` that isn't `display: list-item` draws no marker. A numbered list a card in a chat's thread reads from Markdown (`temper-client-view/src/cards.rs`, `document`) would show no numbers, where today its text holds them. `.temper-card ... ul` on line 1322 wants the same scoping thought. This comes from reading the sheet, not from a browser.
  - **`temper.css:1548`,** `#app > .temper-session .temper-doc-inner ul { margin: 0; padding-left: 20px; }`, has no `ol` beside it, so a numbered list on a document page takes the browser's margins and 40 px of padding.
  - **A chat's own text needs nothing:** the rules for an agent's words (`temper.css:253`, `254`, `639`) already name `ol`.

## Overlaps with the drafts beside this one

- **`chat-reading-a` (cut in flight):** its section 3 takes a growing case in `skein-ui/src/markdown.rs`, writes `ui.md`, section 10, and closes the entries beside the one closed here in `chat-reading.status.md`. It couldn't be avoided: both change the reader. `list_item` stays one function, the place a marker still being written ("4.") would wait for its words.
- **`ui-rename-focus/focus-target`:** edits `skein-ui-web/src/tests/page.rs` far from these tests, and `ui-web.status.md` and `STATUS.md`, which this task also brings back to `No gap.`.
- **`look-round-6`, `ui-final-walk/fix-look`, `ui-rename-focus/rename-row`** own `temper.css`: this plan changes nothing in it.
