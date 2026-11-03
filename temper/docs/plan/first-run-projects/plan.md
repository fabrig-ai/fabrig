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

- Step 2 (a designer of this cut, `notes/contracts.md` K1 to K4):
  `temper/docs/design/protocol/setup.md`, 6.5, 6.7, 8.1, 8.3, 15, 16 (asked with no project, made
  unadopted, ctl's call alone adopts); `protocol/forge.md`, 2.1; `protocol/store.md`, 2
  (`FirstIntent`); `client/README.md`, 3.5 (the welcome whole per K2, Ready making "Default" per
  K1, With no project yet, A project with no repository per K4), 4.4 (the form filled from a
  project with no repository), 4.6 and 8's table (K3's wire), 5.6, 10; `client/forge.md`, 5.4;
  `client/tools.md`, 5.5 (ctl keeps `--project`); `help/forge.md`. Statuses: setup, forge,
  store, client README, forge, tools; the draft's status loses its 4 and 5 entries.
- Step 4: `client/README.md`, 3.5, "The first chat" (template 3 while `me` offers it, else the
  stock chat, so the offer moves to the configuring agent with nothing else changed).
- Step 3: `client/README.md`, 5.3, "Projects in the banner" (Rename for owners, its form with
  Save, Cancel and Escape; the new project's form; opening the made project); `help/project.md`.
  The switcher's look stays 5.1's and the look plan's.

## Parts and tasks so far

| Part | Task | Needs | Tree |
|---|---|---|---|
| no-seed | no-seeded-project | | temper (init's template; service and process tests counting projects) |
| demo | demo-project | | temper (engine's playground route, client view's project names) |
| form-escape | skein-form-escape | | skein (skein-ui's outline, skein-ui-web's page model) |
| wizard | welcome-ready | demo-project, no-seeded-project | temper (client domain and view: the welcome, ConnectForge, client world) |
| no-repository | no-repository | welcome-ready | temper (client view's chats page, domain's adoption form) |
| wizard | setup-project-goes | welcome-ready, no-repository, no-seeded-project | temper (client wire, store, engine forge, service, ctl, client domain encode) |
| first-chat | first-chat | no-repository | temper (client domain's pages, view's chats page) |
| switcher | switcher-acts | first-chat | temper (client domain: the switcher's state and acts) |
| switcher | switcher-view | switcher-acts, skein-form-escape | temper (client view's banner, terminal goldens) |

## Critical path

no-seeded-project and demo-project (step 1, landed or done), then welcome-ready (the longest,
about 120 minutes), no-repository, first-chat, switcher-acts and switcher-view, one after another
in temper's client crates, which one task at a time owns; setup-project-goes runs beside
first-chat in other files of the client domain (its `forge/` encoding) and its own crates.
skein-form-escape has landed. The switcher's tasks come last, as `inputs/request.md` asks.

## Decisions

- **Section 9's questions,** the interim readings `inputs/request.md` gives: a taken "Default"
  at Ready opens the existing one (K1); a second playground run makes "Demo" again. Both stand as
  **Open** in the draft's status.
- **The request's project goes** (the draft's 4, K3): from the client wire and from the store's
  `FirstIntent`, both schemas changed in place at their current versions; ctl keeps `--project`
  in its own run. Until the first release the store changes in place
  (`temper/docs/design/protocol/store.md`, section 4), so a store written before with a first
  repository's intent is not migrated.
- **The `welcome-set-up` journey** goes stale with the project field: welcome-ready deletes it, and
  `journeys.status.md` records it as walked anew later (`journeys.md`, 8), a person's walk.
- **Escape in a form** is skein's generic mechanism (`skein-form-escape`), before its temper use.

## Overlaps with the plans beside this one

- **look-welcome-chats** (the look of the welcome-playground journey's pages): its request also
  names the switcher's Rename and new project and the banner's "Set up temper →". This plan
  keeps to the behaviour the user gave it (`inputs/request.md`): the switcher's acts and their
  binding, released last (`switcher-acts`, `switcher-view`), its view binding to the look's
  switcher if the trunk holds it and to today's buttons otherwise. "Set up temper →" is that
  plan's (an existing act, `EndPlayground`, and its look): the draft's status keeps its 3 lag
  until it lands. Files both plans change: `temper-client-view`'s `frame.rs` (demo-project's
  one call; switcher-view's banner), `words.rs`, `pages/chats.rs` (the no-repository highlight,
  first-chat's offer), the terminal goldens; the client README's 3.5 and 5.x pages' sections.
- **withdrawn-forms** (`withdrawn-forms/temper-configs`, merged in its trunk) and
  **reference-m3** (`reference-m3/subagents-profile`, merged): both change
  `temper/crates/temper-init/src/tests/`, where no-seeded-project changes the template's projects
  test (other functions).
