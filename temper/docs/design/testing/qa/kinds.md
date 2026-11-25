# A place's kinds

Provisional, 2026-11-16, from `../../draft/archive/landed-qa.md`, sections 2, 5
and 10, milestone Q8 of `../../draft/archive/landed-qa.execution.md`. One
framework for a place's two kinds, a box (`../../protocol/setup/box-backends.md`,
1) and a deployment (`deployment.md`): what a place does is one set of
operations, install, start, stop, save, restore and the lines' values,
each written once, in the drive session or the QA tool, for either
kind; what differs between the kinds is a value of the kind, in one
table (`kinds.md`, section 3). The box stays a kind of its own, for QA of the box
itself: its welcome, its playground and `temper box`
(`../../draft/archive/landed-qa.md`, 2 and 10).

## 1. In one page

- **The operations** (section 2): install, restore and save are the QA
  tool's (`places.md`, 3 and 4); start, the lines' values, stop and the
  exits are the drive session's (`../../client/drive.md`, 2 and 3).
- **The kinds' table** (section 3): each kind's value for each
  operation, the only place the two kinds differ.
- **Ports by use** (section 4): the nine positions stay shared, each
  named by what binds it at each kind.
- **In the drive session** (section 5): one place, of either kind,
  behind one interface; the session's own steps, ctl's sign-in, the
  browser's start, the TUI, the lines and the clients' end, written once.
- **In the QA tool** (section 6): a place's kind answers its values in
  one place; a copy leaves out each kind's pinned agent; triage offers
  the setups of the kinds the places run.
- **One catalogue** (section 7): every setup, of either kind, in one
  directory and one document, each box setup weighed (`setups.md`, 7).
- **What stays** (section 8): the drive tool's command line, answers and
  exits, `qa.json`, the copies and the recipes, so that a place's build
  and the QA tool of another commit still speak.

## 2. The operations

| Operation | What | Whose | Where |
|---|---|---|---|
| install | the pass's build into the place's `bin/` and `bundle/` | the QA tool | `places.md`, 3: one path for either kind, since a kept build holds the box's and the deployment's executables alike (`builds.md`, 3) |
| restore | a setup's copy unpacked into the place's scratch directory, but for what a start pins | the QA tool | `places.md`, 4 |
| start | the place's processes started from `bin/` and the scratch directory, each ready; its first link and origin | the drive session | `../../client/drive.md`, 2.3; `deployment.md`, 4 and 5 |
| values | a ctl line's named values and a forge line's API and token file | the drive session | `../../client/drive.md`, 3.4 and 3.5; `deployment.md`, 6.1 |
| its own lines | lines only its kind takes | the drive session | `deployment.md`, 6.2 |
| stop | the place's processes stopped in order, so the scratch directory holds a stopped place | the drive session | `../../client/drive.md`, 2.4; `deployment.md`, 7 |
| exits | a store refused by name, status 3, or the place failed, status 4 | the drive session | `../../client/drive.md`, 2.5; `deployment.md`, 7 |
| save | `config/` and `data/` archived, but for what a start pins | the QA tool | `places.md`, 4 |

## 3. The kinds' table

| | A box | A deployment |
|---|---|---|
| its scratch directory at a place | `box/` | `deployment/` |
| its setups | `fresh`, and every chain whose root holds no `config/` | every chain whose root holds `config/`; no `fresh` (`places.md`, 4) |
| the drive session's flag | none | `--deployment ROOT/config`, ROOT the chain's root (`places.md`, 4 and 5) |
| prepared | by `temper-box` itself: a new sandbox where `config/` and `data/` are absent | `deployment.md`, 4 |
| its processes, in start order | `temper-box --fake-backends` | the fake Forgejo, the fake model, the engine, the worker (`deployment.md`, 5) |
| ready | the box's first stdout line starting `http://`, within 60 seconds | each process's ready line within its bound, the engine's first `http://` stdout line among the engine's |
| its first link, and the fresh link a spent one brings | the box's stdout | the engine's stdout |
| a start failed | the box's exit, or a stderr line holding `; giving up` or `; restarting ` | a process's exit, or a ready line past its bound |
| `--ports` | optional, given to the box as `TEMPER_INTERNAL_FAKE_BOX_PORTS` | required, rendered as `{place.NAME}` (section 4) |
| `--bundle` | required, given to the box | optional, the engine's `{place.bundle}` (`deployment.md`, 2) |
| its values | the newest of the box's opening lines in `session/box.log` (`../../client/drive.md`, 3.4) | the place and its rendered documents (`deployment.md`, 6.1) |
| its own lines | none: `failed: line N: a box takes no deployment line`, unkept | `deployment patch` (`deployment.md`, 6.2) |
| a ctl session's refusal of a web or tui line | `this session takes ctl and forge lines alone` | `this session takes ctl, forge and deployment lines alone` |
| a line once it has exited | fails, `the box exited` (`../../client/drive.md`, 3.7) | fails, `the deployment exited` (`deployment.md`, 6.2) |
| its stop | `SIGTERM`, within 60 seconds | the worker and the engine within 60 seconds each, then the fake model and the fake Forgejo within 30 each |
| its logs in `session/` | `box.log`, and serve's log under the box's logs | `forgejo.log`, `model.log`, `engine.log`, `worker.log` |
| refused by name, status 3 | serve's log line starting `temper serve: `, or a fake forge's starting `temper-box fake forge: state `, naming a path under the scratch directory | the engine's log line starting `temper-engine serve: `, or the fake Forgejo's starting `temper-box fake forge: state `, naming a path under the scratch directory |
| failed, status 4 | `box failed: WHY`, then the last 20 lines of `session/box.log` and of serve's log | `deployment failed: WHY`, then the last 20 lines of each process's log |
| the pinned agent a copy leaves out | `data/box/fake-backends/data/workspaces/agent` | `data/worker/workspaces/agent` |

Everything else is one path for both: ctl signed in from a file
holding the first link, the browser's start on the first link's origin,
the TUI there, every ctl, forge, web and tui line, the kept lines, the
clients' end before the place's stop, the archive, the install, the
holder and its socket.

## 4. Ports by use

`qa.json`'s `ports` (`README.md`, 3.2), the drive session's `--ports`
(`../../client/drive.md`, 2.1) and `TEMPER_INTERNAL_FAKE_BOX_PORTS`
(`../../protocol/setup/box-backends.md`, 1) keep one list of nine, in the box's
order, so every place's configuration and every copy made at it stay as
they are. Each position is named by what binds it:

| Position | Its name | A box binds | A deployment binds |
|---|---|---|---|
| 1 | `clients` | its clients' listener | the engine's clients' listener, `{place.clients}` |
| 2 | `owned` | its owned Forgejo | the engine's workers' listener, `{place.workers}` |
| 3 | `forge` | init's forge | nothing |
| 4 | `llm` | init's model | nothing |
| 5 | `webhooks` | its webhooks' listener | the engine's webhooks' listener, `{place.webhooks}` |
| 6 | `github` | its fake GitHub | nothing |
| 7 | `oauth` | its fake OAuth | nothing |
| 8 | `model` | its fake model | the fake model, `{place.model}` |
| 9 | `forgejo` | its fake Forgejo elsewhere | the fake Forgejo, `{place.forgejo}` |

In the drive session the list is one record, its positions read by
these names alone: the box is given the list whole, and a deployment's
`{place.NAME}` values read their positions through it. A kind that
binds a position by a use the other's name does not say names it in
this table, as position 2 does. So the ports are named by what binds
them, and no format moves.

## 5. In the drive session

- **One place per session,** its kind from the command line as now:
  `--deployment DIR` a deployment's, else a box's
  (`../../client/drive.md`, 2.1).
- **Behind one interface:** the session's logic holds its place as one
  value of either kind, a closed enum of the two, never a trait object,
  a generic or a closure, as a contract names none
  (`skein/docs/foundation/programming-model.md`, 10.3). The session asks the place to
  prepare and start; gives it each of its processes' lines and exits;
  takes from it, once it is ready, its first link and its origin; asks
  it for a line's values and a forge line's hands, and to carry out its
  own lines; asks it to stop; and gives it its logs to classify into an
  exit. Each kind answers from section 3's table.
- **The session's own steps, once,** on what the place gives, for either
  kind: ctl signed in from a file holding the place's first link
  (`../../client/drive.md`, 2.3, step 2); but in a ctl session, the
  browser's start on the place's origin, the spent link opened, the
  fresh link taken from the place's stdout, the root page's main region
  awaited, the spent link's 401 not counted (step 3); the TUI on the
  place's origin (3.3); every line (section 3 there); and at the end
  the TUI ended and the browser closed, then the place stopped (2.4).
  So the browser's start and the clients' end, which Q7's deployment
  session repeated around its own processes, are one path, and the box
  and the deployment differ only where section 3 says.
- **One vocabulary of processes:** the box is one of the session's
  processes as the fake Forgejo, the fake model, the engine and the
  worker are: each started by the same act, its output given to the
  place a line at a time, stopped by the same act, its group killed past
  its bound; the place classifies the logs it names.
- **What a place's kind adds stays its own:** a deployment's
  preparation, documents and `deployment patch` (`deployment.md`, 2 to
  6) are the deployment's, and a patch line's stop and start again go
  through the place's own stop and start of the engine and the worker.

## 6. In the QA tool

- **The kind answers its values:** a place's kind, from `qa.json`
  (`README.md`, 3.2), gives its scratch directory's name, whether it
  offers `fresh`, the pinned agent a copy leaves out and the drive
  session's flag (section 3), each from one place in the tool, so that
  a reader asks the kind rather than testing it. The tool keeps a kind
  of its own beside the drive session's, since it links none of the
  drive tool's code (`places.md`, 8): a place's build reads its own
  recipes and speaks its own lines.
- **Each kind's pinned agent left out:** a copy saved at a deployment's
  place leaves out `data/worker/workspaces/agent`, as one at a box's
  leaves out `data/box/fake-backends/data/workspaces/agent`, and a
  restore at either never unpacks its kind's from a copy that holds it
  (`places.md`, 4). So a deployment's older copy that holds one restores
  without it, and its worker pins the place's build at its start.
- **One reader of a chain at a place:** `pass setup`'s chain and the
  holder's, whose root gives the deployment's `config/` and whose
  members give the scripts (`places.md`, 4 and 5), are read by one
  function, its kind the chain's root's.
- **The catalogue by the places' kinds:** triage offers the setups of
  the kinds `qa.json`'s places run (`triage.md`, 3): `fresh` and the
  box's setups only where a box's place is configured, a deployment's
  setups only where a deployment's place is. A case on a setup of a kind
  no place runs would never be taken (`ledger.md`, 4.8), so triage never
  offers one, and the case's check refuses it as a setup not offered
  (`ledger.md`, 4.3).

## 7. One catalogue

- **One directory,** `temper/docs/development/qa/setups/`, holds every
  setup of either kind, its kind its chain's root's
  (`../../client/recipe.md`, 2); `setups.md` lists each with its kind.
- **Each box setup weighed** (`../../draft/archive/landed-qa.md`, 13): every
  box setup stays the box's, for QA of the box itself, and the one that
  needs nothing of a box, `ctl-hello`, is reached at a deployment's
  place too, by `stock-landing`: one setup in two places, the same lines
  and scripts at a place of each kind (`setups.md`, 5.2 and 7; the
  user, 2026-11-16).

## 8. What stays as it is

- **The drive tool's command line,** its answers, its exits and their
  words, and the kept lines (`../../client/drive.md`, 2 to 4): the QA
  tool runs the place's own build of it (`places.md`, 8), which may be
  older or newer than the tool, so nothing between them moves.
- **`qa.json`** (`README.md`, 3.2), its places' `kind` and `ports` among
  it.
- **The copies:** their archive and their names (`places.md`, 4), so a
  copy made before restores.
- **The recipes,** their format, their chain and their hash
  (`../../client/recipe.md`, 2 to 4).

## 9. Its tests

Focused, with no process, no box and no browser:

- **The drive session's one path,** over injected events, for each
  kind: the session's start reaches `ready` through the same client
  steps, ctl signed in from the place's first link and, but in a ctl
  session, the spent link opened, the fresh link from the place's
  stdout, the root page's main region; the end closes the TUI and the
  browser before the place's first stop; each kind's processes, values,
  refusals and words as section 3 has them. The existing tests of the
  box's and the deployment's sessions keep their expectations.
- **Ports by use:** the box given the nine in their order; a
  deployment's `{place.clients}`, `{place.workers}`, `{place.webhooks}`,
  `{place.model}` and `{place.forgejo}` the positions section 4 names.
- **The QA tool:** a copy saved at a deployment's place without
  `data/worker/workspaces/agent`, the rest of `data/` with it, and a
  copy holding one restored without it; at a box's place, the box's as
  before; the holder's and `pass setup`'s chain read alike; triage's
  catalogue with places of one kind offering that kind's setups alone,
  `fresh` only with a box's place, and with places of both kinds every
  setup.

The walk after landing makes every setup again at a place of its kind
and runs a pass at each kind (`../../draft/archive/landed-qa.execution.md`, Q8).

## 10. Later

- **A third kind,** should a place ever run what neither a box nor a
  deployment does: a column of section 3's table, a name for each
  position it binds in section 4's.

## 11. Open questions

- **A place's ports named in `qa.json`:** the nine positions by name,
  rather than a list. Default: the list, as section 4 has it, until a
  kind binds a use no position names.
