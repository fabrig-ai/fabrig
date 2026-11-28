# A project's charters, in temper

Provisional, 2026-11-28, from `../../draft/ways-charters.md`
(`../../draft/ways.execution.md`, W3). Which of a project's tasks run by
the charters its home repository keeps in `.temper/charters/`, how
temper gives them to jig's core, which review never takes them, and what
each run records of where its words came from. The mechanism is jig's
(`jig/docs/design/domain/engine/project-charters.md`); a charter file's
form and the check before it lands are `../../protocol/temper-dir.md`,
3.3 and 8; the terminal's preview and copy are
`../../client/ctl/tools-reads.md`, 1.

## 1. In one page

- **Five sources** (2) make a run's words: smith's harness, the charter,
  the deployment's lines, the repository's guide and the task.
- **A project's charters** (3) are its repository revision's charter
  files, where that revision holds `.temper/ways.toml`. A task of the
  deployment's charter NAME, in such a project, runs by the project's
  `NAME` where it has one, with its `settings` charter's settings and
  that charter's overlay over its words.
- **The root gives them to the core** (4) in the decision a revision
  moves, and at a start.
- **The floor's review keeps the deployment's reviewer** (5).
- **Each run records its sources** (6): the build, the settings
  revision, the project's set and each guide its agent read.

## 2. Five sources

| Source | Says | Read from |
|---|---|---|
| the harness | tools, the brief's frame, the checkout, delivery, the budget | smith's and temper's code (smith's `smith/docs/design/domain/run-start.md`, 1; `charter.md`) |
| the charter | how this agent works: its instructions and its parts | the project's charter of its name, where its set holds one (3); else the deployment's, the stock's text or one set whole (`charter.md`, 1) |
| the deployment's lines | what its machines are, and its own rules | the overlay of the charter whose settings the run runs by (jig's `jig/docs/design/domain/engine/overlays.md`) |
| the repository's guide | how this code is written | `AGENTS.md` at the root of each workspace directory, as smith finds it (smith's `smith/docs/design/domain/run-start.md`, 1.3) |
| the task | its words, its brief and its messages | the task (`charter.md`, 1) |

- **Sources, not an order:** smith lays out the prompt in its own order;
  the table says where each word comes from.
- **Outside on purpose:** the harness, which is code; the deployment's
  lines, which differ from one deployment to the next while the
  repository is the same on all; and every figure and grant, which are
  settings.

## 3. Which tasks

- **The switch is `ways.toml`:** a project's charters are the
  `.temper/charters/NAME.md` files of its repository revision
  (`../forge/temper-dir.md`, 3) where that revision's files hold
  `.temper/ways.toml`. A revision holding `charters/` without
  `ways.toml` gives none, so a project without `ways.toml` runs exactly
  as before.
- **Each file is the project's charter NAME,** run by the deployment's
  charter its front matter's `settings` names, or by the deployment's
  charter NAME where it names none (`../../protocol/temper-dir.md`,
  3.3).
- **Bound by name:** a task made in the project naming the deployment's
  charter NAME runs by the project's NAME, where its set holds one
  (jig's `jig/docs/design/domain/engine/project-charters.md`, 4): a
  change's producer, a goal's agent, a chat, a standing role. Its
  charter is then the one `settings` names, for every figure and grant.
- **A charter with no file** runs the deployment's charter, as today.
  Every task of W3's is work of no way; a way's own agents never take
  the stock's text (`../../draft/ways-charters.md`, 1), which the
  check of a way's charters keeps (`../../draft/ways.execution.md`, W4).
- **No project, no charters:** a trial runs in no project
  (`../trial.md`), and is never bound.

## 4. From the repository revision to the core

- **When a revision moves:** in the decision the forge saves a
  `DotTemperRevision` whose `number` moved (`../forge/temper-dir.md`,
  3), the root gives the core the project's set at that `number`:
  - where its `files` hold `.temper/ways.toml`, one charter for each
    `.temper/charters/NAME.md` among them, read by
    `../../protocol/temper-dir.md`, 3.3: its name, its `settings` or its
    own name, its instructions and its parts, each part's reasons as
    jig names them;
  - else, where the core holds a set for the project, an empty one, so
    its new tasks run as before; else nothing.
  - Its `source` is the home's name and the first 12 hex digits of the
    revision's commit: ``acme/shop at 1a2b3c4d5e6f``.
- **At a start,** once the core and the forge have restored, the root
  gives the core each project's set where the project's revision holds
  `ways.toml` and its `number` isn't the core's newest for the project,
  so a revision taken before this build gets its set at the first
  start.
- **Judged before:** the read judged the same files before the revision
  took them (`../../protocol/temper-dir.md`, 8), so a revision's charter
  files always read. One that doesn't is the build's fault: the root
  gives the project no set, and logs it naming the file.
- **A set the core refuses** (jig's
  `jig/docs/design/domain/engine/project-charters.md`, 6; only a limit
  lowered since the check can make one) leaves the project's new tasks
  on the set they had; the root logs the refusal, naming the project,
  the revision and the fault.

## 5. The floor's review

- **The deployment's reviewer, always:** an agent review a gate asks
  for is made asking for the deployment's words (jig's charter scope
  *deployment*, `jig/docs/design/domain/engine/project-charters.md`,
  4), so it runs kind 3's reviewer whatever the project's
  `.temper/charters/` holds (`../../draft/ways.md`, 5.4).
- **Why:** the floor is the owners' guard. A project's charters land
  under the project's rules alone, the owners' sign-off off by default
  (`../forge/temper-dir.md`, 5), so a charter a change can edit must
  never shape the review that judges that change.
- **Every gate's review, for now:** until a way's own gates run their
  way's charters (`../../draft/ways.execution.md`, W4), a plan's gate's
  review takes the deployment's reviewer too.

## 6. What a run records

- **Each attempt's sources** (jig's `jig/docs/design/domain/tasks/lifecycle.md`,
  12.1): the engine's build, as `temper-version`'s commit word names it
  and the store's floor keeps it; the settings revision its run is
  prepared under; the project's set its task records, its revision and
  its `source`; and each guide its agent read.
- **The guides come from the worker:** smith tells its host, once the
  run is prepared, each guide it read: its workspace directory, its
  path, the SHA-256 of the bytes it read, their count and whether the
  read was whole (smith's `smith/docs/design/domain/run-start.md`, 1.3).
  The worker names each directory's repository and the commit its
  checkout holds once prepared (`../worker/checkouts.md`), keeps the
  report from preparation, and gives it to jig's hub, which sends it as
  a `guides` record just before the attempt's answer, again wherever
  that answer is sent again, and frees it with the answer's
  acknowledgement (jig's `jig/docs/design/domain/hosts/hub.md`, 2,
  step 9). Its bytes are `temper-worker-wire`'s `Guides`, carried as the
  application's root of that name (`../../protocol/wires.md`, 6). The
  root steps the report onto the attempt as it reads the record, before
  the answer closes it. A running attempt therefore reads as unreported
  until it answers: the draft asks only that what a run read be read
  back while its records are kept, and a report kept with the answer is
  lost with it alone.
- **Why a record of its own:** an engine of the build before bounds the
  answer's `Left` by its own largest, so a report inside it would make
  that engine refuse the answer. `guides` is a kind the worker sends
  only where the engine's terms list it (jig's
  `protocol/channel/layouts.md`, 4.2): to an engine of the build before
  it sends none, and that attempt reads as unreported, as one run by a
  worker of the build before does.
- **Read back:** `temper ctl task runs TASK` prints each attempt's
  sources (`../../client/ctl/tools-reads.md`, 1). With the set's commit
  and each guide's, a person reads from the repository what the run was
  told, while the attempt's row is kept.

## 7. Upgrades

- **No version moves:** jig's families gain what jig's design says
  (`jig/docs/design/domain/engine/project-charters.md`, 9; its
  `domain/tasks/lifecycle.md`, 12.1); `jig-worker-wire` gains `guides`,
  an optional kind each side of either order goes without (6), and
  `temper-worker-wire` its `Guides` root; the clients' wire gains a setting
  read's project and an attempt's sources, the clients rebuilt with them
  (`../../protocol/wires.md`, 9).
- **dev.fabrig.ai** needs no step by hand: fabrig's `.temper/` holds no
  `ways.toml`, so no project of it has charters, and every task runs as
  before.

## 8. Testing

Focused, in the root's and the forge's tests, over the fake forge; a
story crossing the root in temper's world.

- **The set:** a revision with `ways.toml` and two charter files gives
  the core their set, `settings` as written or the file's own name; one
  without `ways.toml` gives none, or an empty one after a set; a start
  gives a revision taken before.
- **The binding, end to end:** a change's producer in a project whose
  `implementer.md` landed runs by its words and charter 2's overlay; a
  project without `ways.toml` runs the stock's; a gate's review runs the
  deployment's reviewer though the project holds
  `.temper/charters/reviewer.md`.
- **The sources:** an attempt records its build, settings revision and
  set; a worker's report adds its guides, once; an attempt with none
  reads unreported.

## 9. Later

- **A piece of work's set** for every task it starts, and charters
  composed in place from several files
  (`../../draft/ways.execution.md`, W4).
- **The preview on a page,** with the pages' pass
  (`../../draft/ways.md`, section 12).
- **Ask temper's review** of a project's charters after an upgrade
  (`../../draft/ways.md`, section 11).
