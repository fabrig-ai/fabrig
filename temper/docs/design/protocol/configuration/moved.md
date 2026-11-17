# Members that moved

Provisional, 2026-11-16, from `../../draft/config.md`, 6.4 and 6.5
(`../../draft/config.execution.md`, N5). The members a document no
longer holds, where each went, and the constants that took the place of
those nobody changes. The document stays JSON, `temper.json`, until
`temper.toml` replaces it (`../../draft/config.md`, section 4); the
members that become its keys (`listen`, `public`, `owner`, `[internal]
bundle`) stay as they are until then.

## 1. In one page

- **Every moved member is refused,** naming its path and where it went
  (section 3), as an unknown member is (`README.md`, section 1). No build
  reads a member that moved, and none takes its value over
  (`../../draft/config.md`, section 8).
- **What nobody changes is a constant** (section 2): of jig's parts,
  those jig lets an application fix (jig's `protocol/configuration.md`,
  3.4); of temper's, a worker's workspaces and its git.
- **One table names them,** in `temper-service-core`'s startup: each
  moved member's path and its words. The refusals read it, and so do
  the `configuration` and `configuration-engine` reference pages, where
  each is one row of its shape saying that a start refuses it and where
  it went, as the members that left for the store are
  (`../documentation.md`, section 8).
- **What moved to the store or to set-up** is designed where it lives:
  the welcome, a setting (section 4); a forge's place, chosen when it's
  set up (section 5); a project's landing, temper's kinds 4 and 5
  (`../../domain/landing-settings.md`, 1.1), with the operator's hold that
  lets a deployment set them before anything lands
  (`../../domain/landing-settings.md`, 1.3); the engine's limits and
  its memory budget, two kinds of settings (`limit-settings.md`). A
  worker alone's `limits` and `application.limits` stay until its
  figures come from the engine (`../../draft/config.md`, 6.3).
- **What stays** for now: the members `README.md` and `engine.md` give
  that this document doesn't name, among them the OpenID provider,
  `engine.clients.provider`, which a managed Forgejo's sign-in through
  temper names (`../openid-provider.md`, 7), and the first owners,
  `engine.owners`. Each is the file's, or another milestone's.

## 2. The constants

temper passes jig's parser these as fixed (jig's
`protocol/configuration.md`, 3.4), in every shape, and decides the
worker's two itself:

| Member | Its constant |
|---|---|
| `engine.deployment` | the directory `state` in the data directory, as the owned example and dev.fabrig.ai have it (the user, 2026-11-16); the seed 7. A box made before by `temper-box init`, whose store is in `deployment`, moves that directory to `state` while stopped, or starts afresh |
| `engine.providers` | 1, the printed link's (`link`); 2, the deployment's own (`deployment`); 3, Google's sign-in (`application`), which `sign_in.google.provider` names |
| `engine.connectors` | `[1]`, the forge connector (`README.md`, section 2) |
| `engine.acts` | connector 1 may `remove_project`, `remove_account` and `adopt` |
| `engine.projects` | none: a first start seeds no project, and projects are made at runtime on policy 1 (`settings.md`, 5) |
| `engine.clients`' `heartbeat_ms`, `link`, `approval` | 10,000 ms; the printed link on; the approval on. Both stay on loopback alone, as the start's check of the clients' listener holds them (`engine.md`, `clients`) |
| `workers[].graces` | `grace_ms` 300,000, `drain_ms` 180,000, `agent_stop_ms` 5,000, `delivery_ms` 1,000, `save_ms` 1,000, as `temper-box init` writes them: a stop bound of 326 seconds with jig's link profile (`README.md`, section 3) |
| `workers[].programs` | `check_interval_ms` 60,000 |
| `application.workers[].root` | `workspaces` in the data directory, made private as the member's was (`README.md`, section 3) |
| `application.workers[].git` | the first executable `git` on the worker's own `PATH` at its start, checked as the member's path was; none found is `null`, as today (`README.md`, section 3) |

- **The fleet's grace is derived,** not configured: the workers' stop
  bound at the constant graces and the link's ping and silence, plus 30
  seconds, rounded up to a whole minute: 360 seconds with jig's link
  profile, as the stock has it today. A `fleet.grace` override above it
  is taken; one below it refuses the start, naming both
  (`../../draft/config.md`, section 7). The service manager's stop
  timeout stays above the stop bound (`temper help server`).
- **A test that needs another value** takes it from its world's own
  construction of the typed configuration, never from a document: a
  world that seeded a project through a document makes it at runtime, as
  an owner does, or builds its configuration in code; a cluster's worker
  whose git is a fake finds it first on its `PATH`.
- **`application.workers` goes whole:** with `root` and `git` constant,
  an entry has no member left, so the list is refused as a moved member
  is.

## 3. The table

Each row's words follow its path in the refusal, `<path>: <words>`, each
at most 120 bytes:

| Member | Words |
|---|---|
| `engine.deployment` | `moved: temper keeps the store in the data directory's state` |
| `engine.providers` | `moved: temper fixes its identity providers: 1 link, 2 deployment, 3 Google's` |
| `engine.connectors` | `moved: temper fixes its connector, the forge's, 1` |
| `engine.acts` | `moved: temper fixes the forge connector's acts` |
| `engine.projects` | `moved: projects are made at runtime; make one from the client or temper ctl` |
| `engine.clients.heartbeat_ms`, `.link`, `.approval` | `moved: temper fixes the clients' heartbeat, printed link and approval` |
| `workers[i].graces` | `moved: temper fixes a worker's graces (temper help server)` |
| `workers[i].programs` | `moved: temper fixes the login checks' interval` |
| `application.workers` | `moved: a worker's workspaces are the data directory's; its git is found on PATH` |
| `application.welcome` | `moved: the welcome is a setting; change it with temper ctl settings` |
| `application.landing` | `moved: a project's landing is settings, project-landing and project-gates; set them with temper ctl settings apply` |
| `application.machine` | `moved: a forge's place is chosen when it's set up, and kept with the forge` |
| `engine.limits` (engine and standalone) | `moved: jig's limits are the setting limits; ask temper, or use temper ctl settings apply` |
| `engine.memory_bytes` (engine and standalone) | `moved: the root's budget is transcript_memory, in the setting temper-limits` |
| `application.limits` (engine and standalone) | `moved: temper's limits are the setting temper-limits; ask temper, or use temper ctl settings apply` |
| `workers[i].memory_bytes`, a worker alone's `memory_bytes` | `moved: nothing read a worker's memory_bytes; remove it` |

The reference pages give the same words in each member's row, so a
person who meets a refusal finds where the member went.

## 4. The welcome

Designed in `settings.md`, section 2 (kind 20), from
`../../draft/config.md`, 6.5: a setting of the deployment, its default
from how the deployment was made.

## 5. A forge's place

Designed in `../setup/place.md`, from `../../draft/config.md`, 6.4:
chosen when the forge is set up, a free port on loopback and binaries
fetched, and kept in its record and its setup file; a host's own units
write the setup file once with `forge prepare`'s flags; a test's fake
GitHub is `application.test.github`.

## 6. Open questions

- **A managed Forgejo's sign-in through temper** keeps
  `engine.clients.provider` in the document; `../../draft/config.md`,
  section 4's file has no key for it. Default: the TOML file's milestone
  gives it one, or derives it as the owned forge's is.
