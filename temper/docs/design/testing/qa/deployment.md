# A deployment at its place

Provisional, 2026-11-14, from `../../draft/landed-qa.md`, sections 2, 5,
6 and 10, milestone Q6 of `../../draft/landed-qa.execution.md`. A place
of the deployment's kind: temper's engine, worker and agent, each its own
process from its own document, with the fake Forgejo and the fake model
as processes of their own beside them, started from a setup's `config/`
at the place's ports, stopped in order, copied, and restored with a
newer build of each, as a redeploy does. The drive session starts and
stops it (`../../client/drive.md`, 2), the recipe walk reaches its
states (`../../client/recipe.md`, 5), and the place installs, saves and
restores it as it does a box (`places.md`). In Q6 it takes ctl, forge
and deployment lines alone; the web client and the TUI reach it in Q7.

## 1. In one page

- **Its documents** (section 2): a deployment's root recipe holds
  `config/`, five documents written as a deployment writes them, which
  name the place's ports, directories and programs as `{place.NAME}`.
- **Its directories** (section 3): the drive session's scratch
  directory, `config/`, `data/`, `cache/` and `session/`, each process's
  own under them; a copy holds `config/` and `data/`, as a box's does.
- **Prepared once** (section 4): the documents rendered, the worker's
  secret and its digest made; the fake git's link made at every start.
- **Started in order, stopped in reverse** (sections 5 and 7): the fake
  Forgejo, the fake model, the engine, the worker; ctl signed in from a
  file, no link printed anywhere.
- **Its lines** (section 6): ctl and forge lines with the deployment's
  values, and `deployment patch`, which changes a document between two
  starts, so a project's landing reaches what the recipe made.
- **A newer build** (section 8): every process from the place's `bin/`,
  the store patched forward, the fakes' state back.

## 2. Its documents

A deployment's root recipe holds `config/` (`../../client/recipe.md`,
2): the documents its processes start from, each JSON as a deployment
writes it, with the fakes in place of real backends
(`../../draft/landed-qa.md`, 5 and 6).

| File | Whose | Its shape |
|---|---|---|
| `engine.json` | the engine's | an engine alone (`../../protocol/cluster.md`, 1), as `temper-engine serve` takes it |
| `worker.json` | the worker's | a worker alone, one worker, as `temper-worker serve` takes it |
| `agent.json` | the worker's agent's | smith's agent configuration (`../../protocol/configuration/README.md`, 4) |
| `forgejo.json` | the fake Forgejo's | `temper-box fake forge --document`'s, with no `github` member (`../fakes.md`, 2) |
| `model.json` | the fake model's | `temper-box fake llm --document`'s (`../../protocol/setup/playground.md`) |

- **Each one required,** a regular file of at most 256 KiB, and nothing
  else in `config/`.
- **No credential:** no document holds a key, a token or a secret
  (`../../draft/landed-qa.md`, 5). The worker's secret and its digest are
  the place's (section 4); the model's key and the forge's admin token
  are outputs the fakes write where their documents say; accounts and
  forges are made by the recipe's lines, as a person makes them.
- **Two paths fixed:** the worker's document names its secret at its
  data path `secrets/worker.secret`, and the engine's names that
  worker's digest at its data path `secrets/worker.digest`, the files
  the place makes (section 4).
- **The place's names.** Within the documents' JSON string values alone,
  each `{place.NAME}` is replaced by the place's value when they are
  rendered (section 4); any other text stays as it is, a `{` not followed
  by `place.` among it, so that a cue's `{pull}` keeps its meaning:

  | Name | Its value |
  |---|---|
  | `{place.clients}` | `127.0.0.1:P`, the engine's clients' listener |
  | `{place.workers}` | `127.0.0.1:P`, the engine's workers' listener |
  | `{place.webhooks}` | `127.0.0.1:P`, the engine's webhooks' listener |
  | `{place.model}` | `127.0.0.1:P`, the fake model's listener |
  | `{place.forgejo}` | `127.0.0.1:P`, the fake Forgejo's listener |
  | `{place.config}`, `{place.data}`, `{place.cache}` | the scratch directory's `config/`, `data/` and `cache/`, absolute |
  | `{place.bin}` | the session's `--bin`, absolute; the worker's agent is `{place.bin}/temper-agent` |
  | `{place.git}` | the scratch directory's `data/bin/git`, absolute: the fake git program (section 4) |

  A `{place.NAME}` of no name here refuses the session, naming the
  document and the name.
- **Its ports** are the place's nine (`places.md`, 2), in the order
  `TEMPER_INTERNAL_FAKE_BOX_PORTS` lists a box's
  (`../../protocol/setup/box.md`, 5), each position bound by the
  deployment as a box binds it where they share a use:

  | Position | A box binds | A deployment binds |
  |---|---|---|
  | 1 | its clients' listener | `{place.clients}` |
  | 2 | its owned Forgejo | `{place.workers}` |
  | 5 | its webhooks' listener | `{place.webhooks}` |
  | 8 | its fake model | `{place.model}` |
  | 9 | its fake Forgejo elsewhere | `{place.forgejo}` |

  Positions 3, 4, 6 and 7 bind nothing at a deployment. So a place's
  `ports` in `qa.json` serve either kind, and a copy's documents and
  forge records name the place's ports, which no other place takes.
  A deployment takes no kernel's port: a session without `--ports`
  refuses.

## 3. Its directories

The drive session's scratch directory (`../../client/drive.md`, 2.2),
for a deployment:

| Path | What |
|---|---|
| `config/engine/temper.json` | the engine's document, rendered |
| `config/worker/temper.json`, `config/worker/agent.json` | the worker's and its agent's, rendered |
| `config/forgejo.json`, `config/model.json` | the fakes' documents, rendered |
| `data/engine/` | the engine's data directory: its store, `secrets/worker.digest` |
| `data/worker/` | the worker's: `secrets/worker.secret`, its workspaces, its programs |
| `data/forgejo/` | the fake Forgejo's kept state, `state`, and the outputs its document names there |
| `data/model/` | the fake model's outputs its document names there |
| `data/bin/git` | a link to `{place.bin}/temper-box`, the fake git program (`../fakes.md`, 3) |
| `cache/engine/`, `cache/worker/` | the engine's and the worker's caches |
| `session/` | the session's own (`../../client/drive.md`, 2.2): ctl's directory, and each process's output, `forgejo.log`, `model.log`, `engine.log`, `worker.log` |

- **A copy is `config/` and `data/`,** the deployment's whole data and
  configuration once every process has stopped, the fakes' state among
  it, archived as a box's are (`places.md`, 4). `cache/` stays at the
  place, as a box's does.
- **Private:** every directory the session makes there is 0o700 and
  every file 0o600, each mode set once it is made (`places.md`, 2).

## 4. Prepared

At the session's start, before any process:

- **Fresh,** with neither `config/` nor `data/`: the documents of
  `--deployment DIR` rendered with the place's names (section 2) into
  `config/` where section 3 puts them; `data/worker/secrets/worker.secret`
  made, 32 random bytes written as 64 lowercase hexadecimal digits and a
  newline, and `data/engine/secrets/worker.digest`, the SHA-256 of those
  64 digits written the same way, as `temper-box init` writes a
  standalone worker's pair.
- **Held,** with both: taken as they are and never rendered again, since
  a copy carries its documents and only its lines change them (section
  6.2); `--deployment DIR` is read for nothing more.
- **Either way,** `data/bin/git` made again, a link to
  `{place.bin}/temper-box`, so that it names the place's `bin/` whatever
  build is installed there.
- **One without the other** refuses the session, naming the one present.

## 5. Started

In this order, each from the session's `--bin`, its stdin closed, its
environment the session's (`../../client/drive.md`, 2.2), in a process
group of its own, its stdout and stderr read a line at a time into its
log in `session/`, but for the engine's stdout, which carries its
sign-in links alone (`../../protocol/cluster.md`, 3) and is written
nowhere. Each within its bound; a failure is the deployment's (section
7).

1. **The fake Forgejo:** `temper-box fake forge --document
   SCRATCH/config/forgejo.json --ready "listening " --state
   SCRATCH/data/forgejo/state`, ready at its stdout line starting
   `listening `, within 60 seconds. Its state lasts across its starts
   (`../fakes.md`, 2).
2. **The fake model:** `temper-box fake llm --document
   SCRATCH/config/model.json --ready "listening "`, with `--scripts
   SCRATCH/session/scripts.json` where the session was given scripts,
   its files joined into that one as the box joins its `--scripts`
   (`../../protocol/setup/box.md`, 5), and `--strict` as given; ready at
   `listening `, within 60 seconds.
3. **The engine:** `temper-engine serve --config SCRATCH/config/engine
   --data SCRATCH/data/engine --cache SCRATCH/cache/engine`, ready once
   its stderr has said `listening clients ` and `listening workers ` and
   its stdout has given its first line starting `http://`, within 120
   seconds.
4. **The worker:** `temper-worker serve --config SCRATCH/config/worker
   --data SCRATCH/data/worker --cache SCRATCH/cache/worker`, ready at its
   stderr's `linked `, within 120 seconds. It pins its agent's program,
   `{place.bin}/temper-agent`, as it starts (`../../protocol/binaries.md`,
   5).
5. **ctl, signed in from a file:** the engine's first link written to
   `session/link`, 0o600, then `temper ctl SCRATCH/session/ctl sign-in
   SCRATCH/session/link </dev/null` (`../../client/ctl/README.md`, 6.1),
   exit 0 within 60 seconds, then `session/link` removed. Every later
   link the engine prints is read and dropped. So no link is printed,
   logged, answered or kept, and no copy holds a sign-in
   (`../../draft/landed-qa.md`, 5 and 11).
6. **`ready`** on stdout, and the lines are read.

While a step waits, any process's exit ends the wait as the
deployment's failure.

## 6. Its lines

### 6.1 ctl's and the forge's

- **ctl lines** run as `../../client/drive.md`, 3.4 has them, the
  person the engine's first owner, with the deployment's values in place
  of the box's, read from the place and the rendered documents, never
  from a log:

  | Name | Its value at a deployment |
  |---|---|
  | `{forgejo.api}` | `127.0.0.1:P`, `{place.forgejo}`'s |
  | `{forgejo.git}` | `http://127.0.0.1:P`, the same |
  | `{forgejo.admin}` | the file the rendered `forgejo.json`'s `admin_token` names |
  | `{git}` | `{place.git}`'s |
  | `{model.key}` | the file the rendered `model.json` names for the key it writes |

- **forge lines** run as `../../client/drive.md`, 3.5 has them, on the
  same API and admin token.
- **web and tui lines** fail, `failed: line N: a deployment takes ctl,
  forge and deployment lines`, and are not kept: they reach a deployment
  in Q7 (`../../draft/landed-qa.execution.md`).

### 6.2 `deployment patch`

`deployment patch DOCUMENT PATH`, an owner's change to the deployment's
configuration between two of its starts (`../../draft/landed-qa.md`,
10): DOCUMENT `engine`, `worker` or `agent`; PATH a file of the files
directory as a `forge commit` line's is, the session's `--files DIR` or,
on a walk, the recipe's own `files/` (`../../client/drive.md`, 3.5), at
most 64 KiB.

1. **Read:** PATH as a JSON merge patch (RFC 7396), one JSON object, its
   string values rendered with the place's names (section 2).
2. **Stopped:** the worker, then the engine, as at the end (section 7),
   each exiting 0.
3. **Patched:** the patch applied to `config/engine/temper.json`,
   `config/worker/temper.json` or `config/worker/agent.json`, written to
   a temporary file beside it and renamed over it.
4. **Started again:** the engine, then the worker, as section 5's steps
   3 and 4. ctl's sign-in is the engine store's and outlives the
   restart, so the engine's new link is dropped.

It answers `| patched DOCUMENT` and `ok`. A patch that is no JSON
object, a result that is no JSON object, or a file missing fails the
line before anything stops, naming it. An engine or worker that does
not start again ends the deployment as failed (section 7): the line
fails, and every later one fails `failed: the deployment exited`. Its
bound is 300 seconds. It is kept as any line (`../../client/drive.md`,
4), so the recipe holds it, and its file in `files/`.

So a recipe gives a project's landing to what its own lines made, a
project and a repository whose numbers exist once those lines have run
(`../../domain/landing.md`, 13.1): `deployment patch engine
landing.json`, the file holding `{"application": {"landing":
{"projects": [...]}}}`, which a merge patch replaces whole. Once a
project's `by`, `batch` and gates are settings
(`../../protocol/configuration/settings.md`, 2), a ctl line changes them
instead.

## 7. Stopped, and its exits

- **At `end` or stdin's end:** the worker by `SIGTERM`, waited on within
  60 seconds; then the engine, within 60 seconds; then the fake model;
  then the fake Forgejo, which writes its state at its orderly stop
  (`../fakes.md`, 2), each within 30 seconds. A process past its bound
  has its group killed, and the stop has failed. After a clean stop,
  `config/` and `data/` hold a stopped deployment, ready to copy.
- **Its exits,** as `../../client/drive.md`, 2.5 has a box's:

  | Status | When | Its last lines on stdout |
  |---|---|---|
  | 0 | ended, every process stopped cleanly, whatever the lines answered | `ended` |
  | 3 | a store refused by name: the engine's log holds a line starting `temper-engine serve: ` that names a directory under the scratch directory (`../../protocol/store.md`, 5), or the fake Forgejo's a line starting `temper-box fake forge: state ` naming a file under it (`../fakes.md`, 2) | `refused: ` and that line |
  | 4 | the deployment failed: a process exited, panicked, or missed its ready line within its bound, ctl's sign-in failed, or a stop passed its bound | `deployment failed: WHY`, then the last 20 lines of each process's log, each prefixed `\| ` |
  | 1 | the session's own failure: its command line, a document missing, past its bound or no JSON, a `{place.NAME}` of no name, `config/` or `data/` alone | `failed: WHY`, on stderr |

  A place makes a copy refused by its store again from its recipe, as it
  does a box's (`places.md`, 4).

## 8. A newer build

- **Every process the newer build's:** a pass installs its build into
  the place's `bin/` before any process starts (`places.md`, 3), and
  every start runs from `bin/`: the engine, the worker and the agent it
  pins, the fakes and the fake git, whose link names `bin/temper-box`.
  The documents name the place's `bin/` and directories, never a kept
  build's, so a copy restores only at the place that made it, as a
  box's does.
- **Patched forward:** the engine patches its store as it starts, as a
  redeploy does (`../../protocol/store.md`, 5; `../../redeploy.md`); a
  store it has no patch for is refused by name, status 3, and the place
  makes the copy again. The fake Forgejo restores its state; a state it
  cannot read is refused by name, status 3, too.

## 9. Its code and tests

- **In `temper-client-journeys-tool`'s drive session**
  (`../../client/drive.md`, 5): a `deployment` module, its rendering, its
  preparation, its processes' plans and order, its values and its
  `deployment patch`, pure over bytes and events as the box's are; the
  session's io starts and stops them as it starts and stops a box.
- **Focused tests,** no process started:
  - rendering: each name replaced in string values alone, `{pull}` kept,
    an unknown `{place.X}` refused naming it and its document;
  - preparation: fresh, held, and one directory alone refused; the
    secret's and the digest's form; the link made again;
  - the start: the order and each ready line over injected events; ctl
    signed in from `session/link`, the link in no log, no answer and no
    program's argument; a process's exit before its ready line, status
    4; every later link dropped;
  - the stop's order and bounds; status 3 from the engine's refusal and
    from the forge's state;
  - the values from the rendered documents; `deployment patch`'s merge,
    its stop and start in order, its refusals before anything stops;
    web and tui lines failing, unkept.
- **The process tier's story** of a deployment at its place
  (`../process-tier.md`), written and compiled in Q6 and run after
  landing.

## 10. Later

- **The web client and the TUI** at a deployment (Q7), and a place's
  kinds behind one interface (Q8; `../../draft/landed-qa.md`, 10).
- **A fake GitHub, several workers, and program accounts** at a
  deployment: its worker's `temper-worker program-login` with the fake
  coding agent's command line beside it, once a setup needs them. Q6's
  setups' agents run on the fake model's scripts, as the twins' do
  (`setups.md`, 4).

## 11. Open questions

- **A deployment's ports by use:** positions shared with a box's nine
  (section 2). Default: shared, until Q8's one framework names a place's
  ports by what binds them.
