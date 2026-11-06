# ui-owner-menu: the plan

The request: `inputs/request.md`. Two tasks, one after the other, 130 implementer-minutes.

| part, task | what | needs | minutes |
|---|---|---|---|
| `menu-outside` | skein: a press outside an open menu closes it, the press still landing, and the focus returns to its button | none | 40 |
| `person-menu` | temper: the rail's foot row opens a menu with "Accounts" and "Sign out"; the tab's Sign out at a phone's width alone; the stories, the gallery, the prints, the proof | `menu-outside` | 90 |

**Critical path:** `menu-outside`, then `person-menu`. They run in series because `person-menu`'s browser story presses outside the menu, and a missing generic mechanism goes into skein before its consumer.

## What was decided

- **`Role::Menu`, not a disclosure:** the request asks for a button that opens a menu, and the rail's "+ New chat" already has skein's: ARIA's menu button, its arrows, Escape, an item's press closing it and the focus back on its button. A `Role::Disclosure` with `FoldsAway`, as the switcher's menu is, would give the press outside without skein's change, but its summary is no button, and it returns no focus on a press outside. The look is the switcher's either way.
- **The gap is skein's:** a menu closes today only on Escape or an item's press. So the "+ New chat" menu gains the press outside too, a change the request doesn't name and the same rule wants.
- **`page::Event` gains `PressOutside`:** a public interface of skein's page part, for a press the mapping can't name a node for. No one outside `skein-ui-web` matches it.
- **A press that took the focus keeps it:** the request says the focus returns to the row. A press on a field or another control leaves it there, or a press outside would steal what it just landed on. A press on nothing returns it.
- **The terminal's unchanged by the shell's match:** the view draws the menu for `Shell::Browser` alone. A `Role::Menu` in the terminal would print two "Sign out"s on the Accounts tab and move every number of the rail.
- **The tab's form stays in the tree and the sheet hides it over 760 px:** the view doesn't know a width, and a phone needs it. So a browser at a phone's width shows the row, its login and the failure line as today.
- **A project-less Accounts page has nothing to change:** the request says it keeps Sign out "there", at the tab's foot. In the code that page has no such row: its Sign out is the banner's (`temper/docs/design/client/README.md`, 3.5). It stays the banner's.
- **The browser story is the existing one:** `signing_in_reloading_reconnecting_and_signing_out_keep_the_cookie_contract` already signs out, reloads and stays out, and it is the test `ui-browser-red` held. It gains the menu's steps and keeps its assertions, with no new server's time on the tier's budget. It signs out from the Accounts page, so its press meets accounts arriving as before.
- **Written by this cut:** `client/README.md`, 3.5 and 5.3; `browser.md`, 2.5, 5 and 7.2; a stale line of `journeys.md`, 3.4, that said temper's view draws no menu; the note of departures, in `greenfield/README.md` and `README.status.md`. The mock's pages stay as drawn.

## Catching up with `ui-rename-focus`

The request has it land first. When this was cut, its `focus-target` was merged on its trunk and `rename-row` was not, and neither was on main. A plan can't need another plan's task, so `notes/common.md` states its rules for both tasks and tells each to merge `origin/main` at its start if main holds them by then. No code here builds on its code. If it lands later, fab's catch-up brings it to the trunk and the end check runs with it. `skein/docs/design/STATUS.md` stays untouched here: `menu-outside`'s **Not yet** opens and closes on this trunk, and that plan rewrites the same line.

## Overlaps with the drafts beside this one

None of these could be avoided, and each is in different lines but where it says otherwise:

- **`ui-rename-focus`:** `skein-ui-web`'s `page.rs` and `page/web.rs` (its focus ops; here the press), `ui-web.md`, 7 (other bullets), `folds_away.rs` (this adds at the end, and leaves its fixture alone), `frame.rs` (its switcher; here the foot), `tests/rail.rs`, `temper.css`, `README.md`, 5.3, and `README.status.md`.
- **`look-round-6/log-end`:** `page.rs`, `page/web.rs` and `tests/page/events.rs`. This adds beside the menu's tests, not at the file's end.
- **`look-round-6/*`, `look-round-7/sign-in-wait-page`, `ui-final-walk/fix-look`, `ui-walk-large`:** `temper.css` (under this plan's marker), `look.rs` (its token array's end meets theirs: the same lines, renumbered on a catch-up), `temper-client-view/src/tests/**` (a file of its own, and one `mod` line), the gallery and the journeys' prints (regenerated on a catch-up, as the request says).
- **`look-round-6/reading`:** `browser.md` and `journeys.md`.
- **`debt-25`, `box-link-first`, `configuring-words`, `chat-reading-*`:** no file in common.
