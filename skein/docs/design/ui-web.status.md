# ui-web: status

2026-11-06. Where skein's code lags or departs from [ui-web.md](ui-web.md).

- **Not yet:** 7: *Keys:* a key's scroll changes a log's following only when the platform's scroll event comes, not in the same apply. *A log:* a pointer's press on a button *to log end* takes the focus.
- **Departs:** 7: a press outside an open menu returns the focus to its button
  unless that press took it to another control, where the code asks only whether
  the focus left the menu (`skein-ui-web`'s `page.rs`, `close_menu_outside`, and
  `page/web.rs`, `press_outside`). An element that took the focus by request or by
  a removal, the main among them, keeps its `tabindex="-1"`, so a press on its own
  inert words takes the focus there, on no control, and the button doesn't get it
  back. No reason is recorded; seen by reading, and no test presses past a menu
  with such an element under the press.
