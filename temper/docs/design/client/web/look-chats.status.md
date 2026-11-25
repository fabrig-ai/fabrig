# look-chats: status

2026-11-22. Where the tree's code lags or departs from [look-chats.md](look-chats.md).

- **Not yet:** the whole document: the answer of 2026-11-06 to the `ui-rename-focus` task
  `rename-row` asks a test that a long chat's document never exceeds its window at
  1280x720 and 1000x600 with "Older", since gone (`../pages/chat/conversation.md`, 2), pressed twice. What holds it is the sheet's
  rule (`temper-client-shell-web`'s `tests/suite/switcher.rs`,
  `scrolling_panes_contain_their_hidden_absolute_parts`), and no test measures the
  page; no reason is recorded. That test names the session's log and draft pane,
  and the session's thread with no log, which scrolls too, is not positioned. The walk
  of the request `ui-walk-3` of 2026-11-06 measured it by hand: the document at its
  window at each size, in Chat and Split, one pane scrolling, but with no earlier read
  ended (`../pages/chat/conversation.status.md`, 2).
- **Not yet:** the whole document: a numbered list read from Markdown
  (`temper/docs/design/draft/chat-reading.md`, 6) shows no numbers in a card of a
  chat's thread: `temper.css` makes `.temper-card:not(.temper-task-card) li` there
  `display: flex`, under no `ul`, and an item that is no `list-item` draws no
  marker. A proposal's reason reads there (`temper-client-view`'s `cards.rs`,
  `document`). On a document page `.temper-doc-inner ul` has no `ol` beside it, so
  a numbered list takes the browser's margins and padding. Both are seen by reading
  the sheet: no browser test draws a numbered list, the request `chat-reading-d`
  having the user read one on a real chat and leaving the sheet to the look's owner.
- **Departs:** the whole document: `Ctrl+\` keeps Chat → Split → Draft on a phone too,
  with Split displayed as Draft. The 2026-11-05 answer to
  `ui-chat-draft/document-pane` accepts this cycle without viewport state;
  only the visible buttons omit Split (`../pages/README.md`).
