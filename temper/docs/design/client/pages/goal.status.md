# goal: status

2026-11-11. Where the tree's code lags or departs from [goal.md](goal.md).

- **Departs:** 3: a party whose `me` gives the project no creating bound still
  sees a chat's "Start goal…" (`temper-client-view`'s `pages/chat_sheet.rs`, `bar`),
  and a press opens nothing (`pages/start_goal.rs`, `open`;
  `no_creating_or_empty_document_opens_no_sheet`), where the section has `NotOffered`
  said in jig's words. No reason is recorded.

- **Design departs:** 2, from the request `ui-goal-page` of 2026-11-05, decided
  by its plan's cut and its proof. A goal's bar is `figure.bar` in the browser
  and `Role::Image` in the terminal, where the request has `div.bar` and
  `Role::Image` for both: skein-ui's `Role::Image` is an image, with no children.
  The planning dots are spans in a `Figure`, not `.typing`'s `<i>`s. A live goal
  with no tasks yet headlines as planning, where the request's words give "0 of 0
  tasks landed." A dependency outside the goal's rows says "waits on task N",
  where the request has its title alone, and the row has none. A goal's counts
  search at most 8 pages for its own line and are then what the page holds, where
  the request has them true past one page (`../README.md`, section 5, *A goal's own line*).
