# first-run-projects: plan

`temper/docs/design/draft/first-run-projects.md`, in its section 8's order (`inputs/request.md`):
no seeded project and the playground's Demo (sections 2 and 3); the wizard without adoption,
Ready making "Default", a project with no repository highlighted (4 and 5); the banner's switcher
(7); the first chat, its interim (6). The cut writes the design and releases it with its specs.

## Design written by this cut

- Step 1: `temper/docs/design/protocol/setup.md`, 8.3 (its projects: none seeded; a deployment
  seeded before keeps its project 1), 8.4 (the start's act 2 makes `Demo`), 13.4 and 15 (project
  numbers); `temper/docs/design/protocol/configuration.md`, section 8, 13.1 and 13.6, step 1
  (`temper init` seeds none); `temper/docs/design/client/README.md`, 5.6 (its project named by its
  record); `journeys.md`, 6 (the welcome-playground journey's project); `domain/testing.md`'s
  project numbers; the help pages `box.md`, `project.md`, `policy.md`, `deployment.md` and
  `playground.md` (no seeded project; `Demo`). Statuses: setup (Not yet 8.3, 8.4), client README (Not yet 5.6); the draft's
  status loses its 2 and 3 entries for them and records section 9's two questions as Open, with
  the interim readings `inputs/request.md` gives.

- Step 3's skein part: `skein/docs/design/ui.md`, 6.1, section 14 and 16's mapping, and
  `skein/docs/design/ui-web.md`, section 7: Escape inside a form, as inside a dialog, so the
  banner's rename in place is cancelled by it. Statuses: skein `ui`, `ui-web` (Not yet), skein's
  `STATUS.md`.

## Parts and tasks so far

| Part | Task | Needs | Tree |
|---|---|---|---|
| no-seed | no-seeded-project | | temper (init's template; service and process tests counting projects) |
| demo | demo-project | | temper (engine's playground route, client view's project names) |
| form-escape | skein-form-escape | | skein (skein-ui's outline, skein-ui-web's page model) |

## Critical path

no-seeded-project, demo-project and skein-form-escape, side by side; the rest follows as its batches release.

## Overlaps with the plans beside this one

- `frame.rs` (`temper-client-view`): demo-project edits one call's arguments in
  `project_buttons`, which the look plan `look-welcome-chats` redraws as its switcher.
