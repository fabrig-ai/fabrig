# look-welcome-chats: plan

The 11 look requests on `origin/look-requests` (`inputs/request.md`), done as
`temper/docs/design/client/journeys.md`, 7 has it: the overlay's rules into `temper.css`, the
markup's changes into `temper-client-view`, and the journeys reprinted, so that
`welcome-playground/02.html` equals the requests' page but where a spec says why.

## Design written by this cut

- `temper/docs/design/client/README.md`: 3.5 (the chats page's row: its pane "Project", the
  level-1 heading "Chats", the composer beside it, its line under it, its placeholder and hint;
  the new chat's bar), 5.1 (the banner's switcher, "Set up temper" in its corner, no accounts'
  banner while the playground runs or is leaving, the playground's banner at the chats pane's
  foot), 5.6 (the banner's "Set up temper"; the playground's start in the pane, the suggested
  requests kept under the composer; the context line on the pane, OWNER/NAME alone), 11 (the
  suggested requests, the user's to revisit). Status: Not yet 3.5, 5.1, 5.6.
- `temper/docs/design/client/browser.md`, 2.5: the banner as one quiet line, the chats page as a
  terminal, the switcher in the phone menu, the chats' placeholder. Status: Not yet 2.5.
- `temper/docs/design/client/journeys.md`, 9, and `journeys/welcome-playground/README.md`:
  `02.html`'s banner line at the pane's foot. Status: Not yet 9.

## Parts and tasks so far

| Part | Task | Needs | Owns |
|---|---|---|---|
| sheet | sheet | | `temper.css` (and `task-pages.css` for the cascade) |

## Critical path

`sheet` runs alone from the first push; the view's tasks follow in the next batch, one writer of
`temper-client-view` at a time.

## Decisions

- **The switcher's Rename and "+ New project" are `first-run-projects`' step 3**
  (`temper/docs/design/draft/first-run-projects.md`, 7; that plan's request leaves the look to
  this plan and builds the behaviour behind it). This plan draws the switcher with the projects
  alone; `temper.css` carries the rules for Rename and New project, and `notes/contracts.md`
  records the requests' markup for them. Their two requests' trailers ride on `sheet`, which
  moves all they asked that is look.
- **The pane's "Chats" is the page's level-1 heading,** since the requests' title, "Chats ·
  temper", is the main's level-1 heading's (`journeys.md`, 5.3): the print has `h1` where the
  requests' page has `h2`.

## Overlaps with the plans beside this one

- `first-run-projects` (`demo-project`, merged on its trunk) changes `pages::project_name`'s
  arguments and its call in `frame.rs`'s `project_buttons`, which this plan's banner task
  redraws: a one-line conflict where the trunks meet. Its README 5.6 "Its names" and
  `journeys.md` 9's `welcome-playground` row (Playground to Demo) touch lines beside and on this
  plan's: the row is one line both change.
- `first-run-projects`' status (`temper/docs/design/draft/first-run-projects.status.md`, on
  `main`, not on this plan's trunk) lags at 3 for the banner's "Set up temper →", which this
  plan's README 5.1 and 5.6 now design; that entry is that plan's to drop.
