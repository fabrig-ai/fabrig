# An owned forge's repositories

This document holds an owned forge's repositories, split from [`owned.md`](owned.md).

## 1. The first repository

- **Asked with the set up, with no project.** `SetUp`'s `first` is one
  of `Empty { org, name, playground, sample }`, an organization (default
  `work`) and a repository (default `hello`), made new with the starter
  workflow alone, or with the playground's copy's files (`playground`)
  or the sample's (`sample`), below; `Copies { addresses }`, at most 16, each
  as `../../draft/repository-copies.md`, 5.4, has it; or `Nothing`. It
  names no project: `Empty`'s repository is made unadopted, and its
  adoption is a later act's, into a project the person has
  (`../../draft/first-run-projects.md`, 4). `Copies` and `Nothing` make no
  repository at set up: the forge connects with no `Create`, and the
  copies wait for a project to go into (`../../client/pages/README.md`). The
  forge domain keeps it in the forge's record (../forge/engine.md, 1), the set
  up's intent: one per forge, written in the decision that admits
  `SetUp`, and replaced by the next `SetUp` it admits. `temper ctl forge
  setup` and a managed connect's `first` ask `Empty` alone, the sample
  never; the welcome's sheet asks the sample, `Copies` or `Nothing`, and
  no longer an empty repository (`../../client/pages/welcome.md`, 1).
- **Asked at a managed connect.** `ForgejoAdmin.first` carries the same
  organization and repository, with no project, and the person's runner label
  (default `docker` in the client and ctl), kept in the forge's record
  as `SetUp`'s are. After Reconcile and its
  `AdminOrigin` write are kept, Create makes the repository
  before the admin token is kept and the forge connects (`reconcile.md`, 5). A missing
  organization is owned by the admin Create has verified. On a forge
  served the listener Create adds the repository's hook, and the forge
  connects with its secret; on another host it adds none (`reconcile.md`, 5).
  The forge's row carries `admin`, the string form's admin-connect offer;
  `first.label`, the runner label; and `ended_stop`, an optional
  `AdminStop`: `NotAdmin`, `Refused { status, stage: SetupStage }`,
  `Unmarked` or `Other`. `SetupStage` names admin, bot, token, org,
  repository, file, app, installation or state. `ended` is the separate
  diagnostic; the client chooses its stop's words from `ended_stop`.
- **Made after reconcile.** At the box's `started FORGE`, or at a start
  while the record keeps an intent whose repository is not made, once
  `TakeOver` and `Reconcile` are done (`owned.md`, 5), the setup host runs
  `Create { forge, org, repository, files }` (skein's `forge/setup/forgejo.md`,
  1.9) by the admin's token: the org, private and owned by the person;
  the repository, private, with Forgejo's README on `main`; the bot a
  collaborator with `write`, and its hook on the webhooks' listener;
  then the starter workflow, below. Then the forge connects, as before
  (`owned.md`, 5). Each step finds what is already there and leaves it, so a set
  up asked again finishes what the last one left. The engine's limits
  for it are 3 files of at most 4 KiB each: the starter workflow alone;
  for the playground's copy, its `src/main.rs` beside it; for the
  sample, its three (below) (skein's `forge/setup/forgejo.md`, 1.1).
  The bound was 2 until the sample, and is raised rather than the files
  carried another way: Create is the one way the design offers to put
  files in a first repository, and its plan is built and checked in the
  engine's process (`temper-service`'s setup), on no wire and in no
  stored format, so that no version moves with it; the intent, which is
  kept, names the sample, never its files.
- **The starter workflow,** `.forgejo/workflows/ci.yml`, its commit
  the bot's, run on a push to `main` and on each pull, on the owned
  runner's label, `temper-host`. At a managed connect it uses `first.label`
  instead: 1–64 ASCII letters, digits, `.`, `_` or `-`, checked at the wire
  and again before the YAML is made. The workflow is otherwise the same:

  ```
  on:
    push:
      branches: [main]
    pull_request:
  jobs:
    check:
      runs-on: temper-host
      steps:
        - run: echo "temper's starter check passed; replace this step with your own checks"
  ```

  Its push is the repository's first CI run, on the owned runner: the
  owned forge enables Actions (`owned.md`, 2), and Forgejo gives a repository
  made through its API the Actions unit (skein's `forge/setup/forgejo.md`,
  section 2). The step needs no network and no action from elsewhere.
- **The playground's copy,** asked with `SetUp`'s `playground`
  (`../../client/ctl/areas.md`, `forge setup --playground`): the first
  repository is `playground/widget`, its organization and name fixed,
  made with the playground's `src/main.rs` (`playground.md`, `temper-fake`'s
  constant, one source for both) beside the starter workflow, in one
  Create, so that the playground's three suggested requests (`playground.md`) are
  asked of a real forge and a real model. Forgejo's README stays, since
  Create keeps a file already there (skein's `forge/setup/forgejo.md`, 1.9);
  the fake's `check.yml` is not copied, since the box's runner runs the
  starter. ctl's call adopts it, as below, into a new project named
  `widget` unless the call names one. A set up asked again takes its own request's fields, with
  the flag or without: a first repository already made stays, and the
  new one is made beside it.
- **The sample,** asked with `Empty`'s `sample` by the welcome's sheet,
  on the welcome and on Settings' Connect a forge
  (`../../client/pages/welcome.md`, 1, *Forge*; the user, 2026-11-19,
  the look request `local-forge`): the first repository is `work/hello`,
  the names the sheet sends and shows none of, made in one Create with
  three files, in this order: `hello`, a short POSIX `sh` script that
  prints a greeting; `test.sh`, its tests; and the sample's workflow at
  the starter's path, last, so that its push, the repository's first CI
  run, finds the other two. Forgejo's README is the fourth: Create keeps
  a file already there (skein's `forge/setup/forgejo.md`, 1.9), so the
  sample brings none of its own. It needs nothing the machine lacks,
  since temper's runs already call git and `/bin/sh`
  (`../../release.md`, section 4), and a change that breaks the greeting
  fails `sh test.sh`, so the check is real. Nothing in the engine or the
  client reads the files: they are `temper-forge-setup`'s constants, one
  each, beside the starter's, and these bytes, each ending in a newline:

  `hello`:

  ```
  #!/bin/sh
  # Prints a greeting, to the name given or to the world: sh hello [NAME]
  printf 'Hello, %s!\n' "${1:-world}"
  ```

  `test.sh`:

  ```
  #!/bin/sh
  # Checks hello's greetings: sh test.sh
  status=0
  check() {
      expected=$1
      shift
      actual=$(sh ./hello "$@")
      if [ "$actual" = "$expected" ]; then
          echo "ok: $expected"
      else
          echo "failed: expected '$expected', got '$actual'"
          status=1
      fi
  }
  check 'Hello, world!'
  check 'Hello, Ada!' Ada
  exit "$status"
  ```

  `.forgejo/workflows/ci.yml`, on the forge's label as the starter's
  (`temper-host` here), its commit the bot's:

  ```
  on:
    push:
      branches: [main]
    pull_request:
  jobs:
    check:
      runs-on: temper-host
      steps:
        - name: Check out
          env:
            TOKEN: ${{ github.token }}
          run: |
            git init -q .
            export GIT_CONFIG_COUNT=1 GIT_CONFIG_KEY_0=http.extraHeader GIT_CONFIG_VALUE_0="Authorization: token $TOKEN"
            git fetch -q --depth 1 "${GITHUB_SERVER_URL%/}/$GITHUB_REPOSITORY.git" "$GITHUB_SHA"
            git checkout -q FETCH_HEAD
        - name: Test
          run: sh test.sh
  ```

  The owned runner runs a job on the host (`owned.md`, 3), where its workspace starts
  empty and no action is fetched from elsewhere, so the workflow takes no
  `actions/checkout`: its first step fetches the commit it runs for by
  git, from the forge, with the job's own token in git's environment,
  never in its arguments. The files are made only with `sample` set; a
  request setting both `playground` and `sample`, which no client sends,
  is made as the playground's copy.
- **The row says it,** the intent, `starts_with`, and the forge's first
  repository beside its connection (`../../client/forge/deployment.md`;
  `../../client/forge/records.md`): with `Copies` or `Nothing` the first
  repository is none; else none; making `ORG/NAME` from the `SetUp`'s admission; made
  `ORG/NAME`, unadopted, once `Create` is done; or failed
  `ORG/NAME`, at the step that stopped (the admin, the org, the
  repository with its access, or the file); with the intent's
  `playground` and `sample`. A `Create` that stops
  blocks the connect as any step does (`owned.md`, 5): the forge is set up
  failed, its `why` naming the step, and "Set up again" asks with the
  same fields, the sample again where the row's `sample` says so. A restart reads the row's first repository from the
  record, making `ORG/NAME`, until `Create` answers again.
- **Adopted by a client's act, never by the set up.** The welcome's
  Ready, its last act, adopts it into the project that act makes
  (`../../client/pages/README.md`), as ctl's call does, below, and the
  project opens without waiting for it; `ConnectForge` adopts nothing:
  there the made repository waits, unadopted, for the person to adopt
  it into a project of theirs from the adoption form
  (`../../client/forge/family.md`, 4), one choice away. A repository the
  person's Ready left refused or unanswered waits the same way, Home
  offering it again.
  `temper ctl forge setup` and `forge connect … --first` hold their
  `--project` in their own run (`../../client/ctl/areas.md`): once the
  row says made, the call sends, under the same party, jig's
  `MakeProject` for a new project, its name and policy 1, the default
  document's (`init.md`, 3), when `--project` names no project, then `Adopt`:
  the project, the forge, `ORG`, `NAME`, landing on `main`, owned, the
  project's home, with CI. Nothing in the engine reads a project for
  it.
- **A restart before the repository is made** keeps the intent, since
  the record holds it: the start's sequence, or the box's `started`,
  makes the repository at the next connect, after reconcile (`owned.md`, 5).
- **A set up writes as it goes:** each act and plan keeps what it makes
  when it makes it, the forge's record and its intent at `SetUp`'s
  admission, each credential record as its plan writes it (`reconcile.md`, 1), and the
  box's group file and record as the group starts (`box.md`, 1). Nothing is
  staged in memory to be written at the end, and no write spans two
  stores.
- **Found later, it starts over.** A set up cut short, by a restart or a
  failure, leaves its forge kept but not connected, or its first
  repository not made: the welcome's Forge step shows it unfinished,
  "FORGE · not finished", with Start over (`../../client/pages/README.md`),
  which removes the forge (`../../domain/forge/adoption-connection.md`, 1) and opens the
  step's choices again. An owned Forgejo then sets up afresh, as a new
  forge numbered next: its binaries from the cache (`owned.md`, 6), which its
  removal keeps, and its directory prepared anew (`owned.md`, 3), the old one
  removed with its forge (`box.md`, 1).
- **GitHub goes on by default:** its App's records are kept as they come
  (`github.md`, 2), so the step offers Go on, to the App's installation, before
  Start over, which removes the forge and names the App on GitHub as the
  person's to delete (`../../domain/forge/adoption-connection.md`, 1).
- **An account added before** stays: it is the person's, not the set
  up's, and the Model step shows it ready (`../../client/pages/README.md`).
- **Its code:** the intent, in the forge's record, in
  `temper-engine-domain-forge`; `Create`'s start and its end in
  `temper-service`'s setup; the starter workflow's bytes, the sample's
  and the plan's limits in `temper-forge-setup`.

## 2. A repository made or copied

- **The service runs what the connector holds**
  (`../../domain/forge/adoption.md`, 5), in the setup's one pull
  (`temper-service`'s `lifecycle::advance`): on a forge connected with no
  writing run live, the connector's oldest `Waiting` row starts, the
  connector told so before the run's first event. A forge whose setup is
  excluded, ended or not connected keeps its rows waiting.
- **`Empty` runs `Create`** and **`Copy` runs `Copy`** (skein's
  `forge/setup/forgejo.md`, 1.9 and 1.10), each with the starter workflow alone
  in `files`, on the forge's label: the record's `first.label` when it
  has one, or else `temper-host` on an owned forge and `docker` on a
  managed one. The plan's limits are `owned-repositories.md`, 1's; `copy_deadline` is 30
  minutes, a constant of `small`'s.
- **`Copying`** is no credential record: the host's `Read` of it is
  answered from the row, and its `Write` becomes the connector's event,
  answered `Written` once that decision is kept.
- **Its end** goes to the connector: `Done` as made, with the id and the
  branch; `Stop` as failed, with its step, its why and what
  `Event::Said` said. An event the root is too busy to take is held and
  given again, since a row, unlike a view, is kept.
- **The other writing runs wait for it:** the service starts no
  reconcile, adoption's access or `Create` beside a run of a row, and
  one that finds it `Busy` waits as a held `Install` does. A copy may
  last 30 minutes, so a forge's restart and its reconcile wait as long.
- **Adopting** (`../../domain/forge/adoption.md`, 5) is asked by the same pull: a
  row `Made` or `Adopting` with no act in flight, on a connected forge,
  is given to the connector to act.

## 3. Open questions

None.
