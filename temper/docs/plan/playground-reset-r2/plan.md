# Plan: playground-reset-r2

The cut of `temper/docs/design/draft/playground-reset.md`, sections 3 to
7, into six parts and 68 tasks (`plan.toml`, `tasks/`). Where the code
stood: `status.md`. The contracts between the parts: `notes/contracts.md`;
the standing rules: `notes/common.md`, `notes/scope.md`; the lead's
answers to foreseen questions: `notes/answers.md`.

## Parts, in the draft's order

1. **forge-records** (draft 3): forges are the forge connector's records
   (`Forge`, `ForgeNumbers`), made by `SetUp`/`Connect`, numbered and never
   reused, the first repository's intent kept; skein's forge connection
   and setup host gain a runtime `Add`; temper-service builds every forge
   from a registry at start and when one is made; the document keeps the
   machine's (`application.ports`, `application.playground`,
   `application.github`); `temper init` names no forge. Forges the document
   still named stay seeds until `fr-retire` ends the bridge; box-less
   deployments and tests get forges by seed files or `Event::Seed`.
2. **box-groups** (draft 3 and 4, the box): `<data>/box/groups/N` read at
   start and at `start N`, steps by kind from `temper-forge-setup`, a fixed
   number of group slots, `remove N` (stop, `temper forge remove DIRS N`,
   `started/N`, then `groups/N`), the child commands reading `groups/N`;
   serve writes the file before `start N` and removes on a kept act.
3. **project-delete** (draft 5, jig): `RemoveProject` (deployment act and
   owners' ask), `Act::Cancel`; a core `Removal` row with stages Ending,
   Erasing, Notes, Rows; live tasks cancelled, then each ended task erased
   whole (the core pages them by its own reads, one per commit), then the
   project's rows, `ProjectRemoved` to connectors; resumed at each start;
   the referee counts rows naming a project (ledgers and retained answers
   excepted); one conformance case per family.
4. **forge-remove** (draft 4, 6.4): `RemoveForge` ends any forge at once
   (`forge_ended` general), erases its writer and flights, keeps
   `KeptAct`s (cancels, `RemoveGroup`, `EraseSecrets`) sent at each start,
   erases the record when whole; `ProjectRemoved` erases a project's
   repository rows; the service retires the forge; a set up started over.
5. **playground** (draft 6.1 to 6.3): `StartPlayground` makes the fake
   forge's record, its group, the playground's project, the scripted
   account (its uses that project alone) and the adoption;
   `EndPlayground`'s reset is the `Playground` row's steps (project,
   forge, account), resumed at each start; a playground found running at
   start is reset (draft 9's default); the demo's families and its end go.
6. **client**: the wire (family 12, in place), Forges on records with
   Remove, the welcome's three states and the Forge step's Start over and
   Go on, the playground's pages and one-line banner, "Leaving the
   playground…", temper's project settings page with Delete, ctl, the
   joined client world, the gallery and journeys.

## Decisions

- Forge definitions are the document's own JSON members, less numbers,
  carried as bytes by the domain and read by the service's existing
  readers (contracts 2), so the forge's format did not change, only its
  home.
- Section 9's defaults, all taken: a box restarted in the playground
  resets it; a removed forge's repository rows stay hidden until their
  project is deleted (the connector does not hear retention erase a
  task); deletion by owners and the deployment; spend kept; the Default
  project reused.
- `temper box --fake-backends` seeds nothing: a fresh box is empty; its
  fake GitHub is the machine's GitHub endpoint.
- A Forgejo elsewhere is not made from the client at runtime this run
  (seed files only); the playground's policy is a stock policy 2.
- Design documents are not amended: the draft stays a draft, cited by
  code; adopting it (its section 10) is the designers' and the user's.
- Questions for the user are in `plan.toml` with their defaults taken;
  no task waits for one.

## Critical path

- 68 tasks, 5020 minutes.
- Longest chain, 1035 implementer-minutes: cl-wire -> fr-ask -> fr-registry -> fr-setup -> fr-service -> fr-init -> fr-fixtures -> fr-retire -> fr-qa -> fr-process -> bx-process -> rm-process -> pg-process

  It runs through the service registry (`fr-registry`, `fr-setup`,
  `fr-service`), the bridge's end (`fr-init`, `fr-fixtures`, `fr-retire`)
  and the process tier, which the parts share and their needs serialize
  (`fr-process`, `bx-process`, `rm-process`, `pg-process`). The engine's
  chain (`bx-serve`, `rm-service`, `pg-start`, `pg-reset`, `pg-gone`) runs
  beside it, and jig's (`jig-acts` to `jp-delete`, about 405 minutes) and
  the client's (about 400 to `cl-welcome`) from the first minutes.

## Tasks

### forge-records (19 tasks, 1365 implementer-minutes)

| Task | Kind | Needs | Minutes |
|---|---|---|---|
| `fr-skein-setup` | feature | - | 50 |
| `fr-skein-connection` | feature | - | 60 |
| `forge-families` | interface | - | 80 |
| `fr-ask` | interface | `forge-families`, `cl-wire` | 60 |
| `fr-domain` | feature | `fr-ask` | 90 |
| `fr-registry` | interface | `fr-ask` | 90 |
| `fr-clients` | feature | `fr-registry` | 50 |
| `fr-workers` | feature | `fr-registry` | 60 |
| `fr-connections` | feature | `fr-registry`, `fr-skein-connection` | 75 |
| `fr-document` | feature | `fr-registry` | 90 |
| `fr-setup` | feature | `fr-registry`, `fr-skein-setup`, `fr-domain` | 90 |
| `fr-service` | feature | `fr-domain`, `fr-connections`, `fr-workers`, `fr-setup`, `fr-clients`, `fr-document` | 90 |
| `fr-init` | feature | `fr-service`, `jig-acts` | 60 |
| `fr-worlds` | migration | `fr-service` | 60 |
| `fr-fixtures` | migration | `fr-service`, `fr-init` | 60 |
| `fr-retire` | migration | `fr-worlds`, `fr-fixtures`, `fr-init`, `bx-box`, `bx-children` | 75 |
| `fr-fake-backends` | feature | `fr-retire` | 60 |
| `fr-qa` | migration | `fr-retire` | 75 |
| `fr-process` | migration | `fr-qa`, `fr-fake-backends`, `bx-serve` | 90 |

### box-groups (6 tasks, 465 implementer-minutes)

| Task | Kind | Needs | Minutes |
|---|---|---|---|
| `bx-steps` | feature | - | 60 |
| `bx-machine` | feature | - | 90 |
| `bx-children` | feature | `bx-steps` | 60 |
| `bx-box` | feature | `bx-steps`, `bx-machine` | 90 |
| `bx-serve` | feature | `fr-service`, `bx-steps` | 90 |
| `bx-process` | migration | `bx-box`, `bx-children`, `bx-serve`, `fr-process` | 75 |

### project-delete (16 tasks, 1150 implementer-minutes)

| Task | Kind | Needs | Minutes |
|---|---|---|---|
| `jig-acts` | interface | - | 75 |
| `jp-children` | feature | `jig-acts` | 45 |
| `jp-planner` | feature | `jig-acts` | 60 |
| `jp-tasks` | feature | `jig-acts` | 60 |
| `jp-row` | interface | `jig-acts` | 70 |
| `jp-scopes` | interface | `jig-acts` | 75 |
| `jig-ask` | interface | `jig-acts` | 80 |
| `jp-referee` | feature | `jig-acts` | 80 |
| `jp-proposals` | interface | `jp-scopes` | 30 |
| `jp-people` | feature | `jig-ask` | 75 |
| `jp-client` | feature | `jig-ask` | 80 |
| `jp-close` | feature | `jig-ask`, `jp-row`, `jp-tasks`, `jp-scopes` | 90 |
| `jp-finish` | feature | `jp-close`, `jp-planner`, `jp-proposals`, `jp-tasks`, `jp-people`, `jp-children`, `jp-referee` | 80 |
| `jp-erase` | feature | `jp-close`, `jp-planner`, `jp-scopes`, `jp-people`, `jp-referee` | 85 |
| `jp-delete` | feature | `jp-erase`, `jp-finish` | 75 |
| `jp-conformance` | feature | `jp-delete`, `jp-referee` | 90 |

### forge-remove (5 tasks, 405 implementer-minutes)

| Task | Kind | Needs | Minutes |
|---|---|---|---|
| `rm-domain` | feature | `forge-families`, `jig-acts`, `fr-domain`, `cl-wire` | 90 |
| `rm-project` | feature | `rm-domain` | 60 |
| `rm-world` | feature | `rm-project` | 75 |
| `rm-service` | feature | `rm-domain`, `fr-service`, `bx-serve` | 90 |
| `rm-process` | feature | `rm-service`, `bx-process`, `cl-tools` | 90 |

### playground (7 tasks, 535 implementer-minutes)

| Task | Kind | Needs | Minutes |
|---|---|---|---|
| `pg-start` | feature | `rm-domain`, `fr-service`, `bx-serve` | 90 |
| `pg-models` | feature | `pg-start`, `fr-init` | 70 |
| `pg-reset` | feature | `pg-start`, `rm-project`, `rm-service`, `jp-delete` | 90 |
| `pg-gone` | migration | `pg-reset`, `pg-models`, `rm-world` | 75 |
| `pg-world` | feature | `pg-gone` | 75 |
| `pg-script` | feature | `pg-start`, `pg-gone`, `bx-process` | 45 |
| `pg-process` | feature | `pg-gone`, `pg-script`, `rm-process` | 90 |

### client (15 tasks, 1100 implementer-minutes)

| Task | Kind | Needs | Minutes |
|---|---|---|---|
| `cl-wire` | interface | - | 90 |
| `cl-surface` | interface | `cl-wire` | 75 |
| `cl-remove` | feature | `cl-surface` | 60 |
| `cl-project` | feature | `cl-surface`, `jig-ask` | 70 |
| `cl-connect` | feature | `cl-surface` | 75 |
| `cl-play-domain` | feature | `cl-surface` | 85 |
| `cl-connect-view` | feature | `cl-connect`, `cl-remove` | 85 |
| `cl-tools` | feature | `cl-connect`, `cl-remove`, `cl-play-domain` | 75 |
| `cl-play-view` | feature | `cl-play-domain`, `cl-connect` | 85 |
| `cl-welcome` | feature | `cl-connect-view`, `cl-play-domain` | 75 |
| `cl-gallery` | feature | `cl-welcome`, `cl-play-view`, `cl-connect-view`, `cl-remove`, `cl-project` | 50 |
| `cl-world-forges` | feature | `rm-domain`, `cl-connect-view`, `cl-welcome`, `cl-tools` | 85 |
| `cl-world-playground` | feature | `pg-reset`, `cl-world-forges`, `cl-play-view` | 85 |
| `cl-world-project` | feature | `jp-delete`, `cl-project`, `cl-world-playground` | 45 |
| `cl-tidy` | migration | `cl-gallery`, `cl-world-playground`, `cl-world-project`, `cl-tools`, `pg-gone` | 60 |

