# view-navigation: status

2026-11-22. Where the tree's code lags or departs from [view-navigation.md](view-navigation.md).

- **Not yet:** 1: the rename's form asks the focus, and so does an open reply:
  `temper-client-view`'s `pages/ask.rs`, `reply_form`, calls `b.focus(0)` with no
  guard, where `pages/chat.rs`, `document.rs`, `settings.rs`, `new_chat.rs` and
  `ask_page.rs` ask only while no rename is open. So an owner who presses "Rename
  PROJECT" with a reply open on Home or Needs you meets skein-ui's
  `Builder::focus`, at most one per tree, which asserts. The `ui-rename-focus` task
  `rename-row` has opening the form on such a page not panic; no reason is recorded
  for this site. Seen by reading; no test holds it.

- **Design departs:** 1, from the greenfield target, by the request `ui-repo-link` of 2026-11-06: the mock's
  switcher row is one button holding the name and the small words; here the small words are a sibling of the
  button, since a link can't sit inside one, and they are a link with its "↗" where the repository has an address.

- **Design departs:** 1, from the greenfield target, by the request
  `ui-rename-focus` of 2026-11-06: closing the rename's form leaves the project
  menu open with the focus on its summary, where the mock draws the menu closed,
  since a disclosure's fold is skein's page part's (`skein/docs/design/ui/web/patches.md`, 1). The field does
  not select its name on opening, and no toast says "Renamed": the request names
  neither, and `README.md`, section 5 holds them. On a new chat or Ask temper's page the closed form
  leaves the focus on the page's own field, where the request has the opener or the
  summary, because that plan's cut keeps a page's own request. The field's hidden
  label says "Rename", where the mock's says "Project name": no one sees it, and
  that plan's cut took no new word for it.

- **Not yet:** 1, *The rail's parts*: no read gives the rail a standing
  role (`../draft/standing-roles.md`).

- **Design departs:** 1, from the greenfield target, by the request
  `ui-owner-menu` of 2026-11-06, at the user's word: the rail's foot row is the
  button of a menu with "Accounts" and "Sign out", where the mock draws the
  person's name as plain text, and a browser over 760 px signs out there rather
  than on the `Accounts` tab. `web/greenfield/README.md` notes it.
