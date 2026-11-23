# patches: status

2026-11-23. Where the tree's code lags or departs from [patches.md](patches.md).

- **Departs:** 1: a press outside an open menu returns the focus to its button
  unless that press took it to another control, where the code asks only whether
  the focus left the menu (`skein-ui-web`'s `page.rs`, `close_menu_outside`, and
  `page/web.rs`, `press_outside`). An element that took the focus by request or by
  a removal, the main among them, keeps its `tabindex="-1"`, so a press on its own
  inert words takes the focus there, on no control, and the button doesn't get it
  back. No reason is recorded; seen by reading, and no test presses past a menu
  with such an element under the press.
- **Not yet:** 1: no run on record shows that tier within its budget with
  `log_reading`'s seven stories, from the plan `chat-reading-c`, which waits for its run
  after landing likewise.
