# look-welcome-chats: contracts

The shapes this plan's tasks meet at, and what it leaves to `first-run-projects`, binding for the
specs that cite them.

## The style tokens

New tokens in `temper-client-view`'s `look.rs`, each a `pub const` beside the others, its name
appended to `NAMES.styles` in token order (the next free numbers on the trunk, in this order),
and its class the stylesheet's (`sheet`):

| Token | Class | Its node | Its task |
|---|---|---|---|
| `SIDEBAR` | `temper-sidebar` | the chats page's pane, a direct child of the chats page's main | `chats-pane` |
| `PROJECTS` | `temper-projects` | the banner's switcher, a disclosure inside "Menu", before the navigation | `banner-switcher` |
| `LEAVE_PLAYGROUND` | `temper-leave-playground` | the banner's "Set up temper" link, the corner's first child | `banner-switcher` |

The existing tokens keep their nodes: `CHATBAR`, `CONTEXT`, `REPOSITORY`, `HEADING`, `FILTERS`,
`EMPTY`, `ROWS`, `ACCOUNTS` (the playground's banner), `ENTRY`, `COMPOSER`, `HINT`, the
chooser's (`crate::choice`), `MENU`, `CORNER`, `NAVIGATION`, `PROJECT_EMPTY`.

## The chats page's tree

The tip of the requests' `02.html` (`7fba5d0fe3`), as the view draws it for every project
(`temper/docs/design/client/README.md`, 3.5), the playground's parts where it shows (4.8, 5.6):

```
main (MAIN; PAGE where it is today)
  aside "Project" (SIDEBAR)
    slot CHATBAR
      slot: paragraph "New chat", status STATUS_QUIET (its status word)
    [slot CONTEXT: paragraph: text REPOSITORY "OWNER/NAME"]        the playground's, while it runs
    [alert "The playground did not start: WHY.", button "Try again"]  once its start failed
    slot HEADING: heading level 1 "Chats", slot FILTERS: All, Live, Closed
    paragraph EMPTY "No chats yet." | list ROWS "Your chats" ...; [button "More closed chats"]
    [region "Playground" ACCOUNTS: the playground's banner]        while it runs or is leaving
  slot ENTRY: textarea "Start a chat" COMPOSER (placeholder "Describe a change"),
              paragraph HINT "enter to start · shift+enter for newline", button "Start chat",
              [refusal, outcome]
  slot: the chooser, `crate::choice::composer` (temper-choice)
  [list "Suggested requests"]                                       see `chats-pane`
```

## The banner's tree

```
banner BANNER
  link BRAND "temper"
  disclosure "Menu" MENU
    disclosure PROJECTS, named NAME, the open project's ("Projects" while none is open)
      button NAME (pressed: the open project), then a button for each other project
    | paragraph PROJECT_EMPTY "No project yet"                     with no project
    navigation "Main navigation" ...
    slot CORNER
      [link LEAVE_PLAYGROUND "Set up temper"]   EndPlayground; while the playground runs, not leaving
      region temper-link: the link's state ...
      status: the person's name; button "Sign out"
```

The requests' summary carries `aria-label="Project: NAME"` apart from its words; skein-ui-web
prints a disclosure's name as its summary's words alone, so the print has `<summary>NAME</summary>`
(`temper/docs/design/client/README.md`, 11, "The switcher's name").

## The switcher's markup, for `first-run-projects`

`first-run-projects`' step 3 (`temper/docs/design/draft/first-run-projects.md`, 7) builds Rename
in place and "+ New project" behind this switcher; this plan draws neither. The requests drew
them so (`7fba5d0fe3`'s `02.html`), and `temper.css` carries their rules (`sheet`), selecting
this markup:

```html
<details class="temper-projects"><summary aria-label="Project: Playground">Playground</summary>
<button type="button" aria-pressed="true">Playground</button>
<button type="button" aria-label="Rename Playground" class="temper-project-rename">Rename</button>
<button type="button">Project 1</button>
<form aria-label="New project" class="temper-project-new">
  <input type="text" aria-label="New project name" placeholder="New project" autocomplete="off">
  <button type="submit">Create</button>
</form>
</details>
```

Renaming, the open project's row and its Rename give way to one form,
`class="temper-project-rename-form"`, its name's field and "Save" (the requests' commit
`7fba5d0fe3`'s message: Enter saves, Escape cancels). The rules lay the open project's row as a
two-column grid (`::details-content`), Rename after the pressed button.
