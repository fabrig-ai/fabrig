# ui-owner-menu: the rules every task names

The request is `temper/docs/plan/ui-owner-menu/inputs/request.md`: read all of it, it is short, and its *For every task of this plan* binds you and is not repeated here.

## Departures from the implementer's text

- **Design text:** this plan's design lines are written, in the sections your spec cites. Your task closes its **Not yet** entry, in its last commit. A fix that makes another design line wrong edits that line and names it in your report. Nothing else under `docs/design/`, but the generated prints.
- **Decide, don't ask,** as the request says. A `Question:` is only for what truly blocks your task.
- **A proof's screenshots** stay outside the repository, in `~/src/fabrig/ui-owner-menu/<your task>/`; your report names them. The render script is throwaway, untracked and gone before the push.
- **A box and Chromium run directly,** outside `heavy`: they build nothing. Every build and test still goes through `heavy`.
- **Catch-up with `ui-rename-focus`:** the first thing you do, `git fetch origin`. If `origin/ui-rename-focus/focus-target` is an ancestor of `origin/main` and not of your base, `git merge origin/main` as your first commit, regenerating what the request says to. If `origin/main` doesn't hold it yet, go on and merge no other plan's trunk: no task here builds on its code, and fab's catch-up brings it to the trunk when it lands.

## The rules taken from `ui-rename-focus`

Its design lines are on `origin/ui-rename-focus/integration` until it lands (`skein/docs/design/ui-web.md`, 7 and 12; `temper/docs/design/client/README.md`, 5.3). Follow them whether or not your base holds them:

- A removal that takes the focus moves it to the nearest remaining ancestor that can take it, a disclosure on its summary, else the main. So Sign out's success, which removes the rail, needs no focus code of ours.
- A programmatic focus target (`tabindex="-1"`, there only to take the focus) draws no ring. A control a person reaches by keys keeps its ring: the row's button takes no `outline: 0`. A menu's item keeps what `temper.css` draws today, the hover's grey on focus (5.3).

## What crosses the two tasks

`menu-outside` makes this true, and `person-menu` builds on it with no skein code of its own: a press outside an open `Role::Menu` closes it, in skein's page part, with no step and nothing in temper's domain or view. The focus returns to the menu's button, unless that press took it to another control.
