# The setups' catalogue

Provisional, 2026-11-12, revised 2026-11-16, from `../../draft/landed-qa.md`, sections 5,
6 and 13, milestones Q3 to Q8 of `../../draft/landed-qa.execution.md`.
The recipes kept in `temper/docs/development/qa/setups/`, what each
reaches, on which forge, and where its scripts come from. Their format
and walk are `../../client/recipe.md`; how a pass restores, makes again
and adds to them is `places.md`.

## 1. In one page

- **On the fakes alone** (section 2): every setup's forge is the box's
  fake Forgejo elsewhere, connected by `project-landing` as its admin
  through ctl, keeping its repositories, pull requests and checks in the
  box's data and so in every copy. No setup sets up the box's own
  Forgejo, so no place runs a Forgejo or a runner.
- **The first recipes** (section 2), from the journeys' setups: the
  welcome, `project-landing` and `chat-draft`, on the box's generated
  scripts.
- **Ctl alone** (section 2): `ctl-hello`, from a fresh box by ctl lines
  alone, Hello on the fake Forgejo and the box's fake model the person's
  account, its scripts a review's approving verdict, so that a ctl
  case's change lands by the project's stock rule; it walks in a ctl
  session.
- **The TUI's** (section 3): the TUI's journeys start from
  `project-landing`, which serves their starting screens; `tui-chat`
  adds a chat asked and answered from the TUI's launchpad.
- **The twins** (section 4): B0's walk replayed as setups, its
  scripts its twin's with takes, so that its numbers and heads are the
  box's own: `widget`, `b0-plan` and `b0-landed`; B1's, its gate
  committed by `forge` lines, `b1-plan` and `b1-landed` (section 4.4).
- **A deployment's** (section 5): `turn-landing`, a project landing by
  `turn` at a `batch` of two with a gate the fake Forgejo dispatches,
  at a deployment's place, with scripts that carry two goals' changes
  to ready together; its engine serves the place's bundle, and
  `turn-landing-clients` sees its state through the web client and the
  TUI on the deployment's engine (5.1); `stock-landing` is `ctl-hello`'s
  state at a deployment, landing by the stock rule (5.2).
- **Grown by passes** (`places.md`, 6): a recipe a pass adds lands with
  its request; this document names the catalogue's roots, not every
  recipe.
- **One catalogue of either kind** (section 7): every setup in one
  directory, its kind its chain's root's, each box setup weighed by what
  of a box it needs (`kinds.md`, 7), and triage offering the setups of
  the kinds the places run (`kinds.md`, 6).

## 2. The first recipes

- **Their forge** is the box's fake Forgejo elsewhere
  (`../../protocol/setup/box.md`, 5), the person's
  (`../../draft/landed-qa.md`, 6): its state kept in the box's data,
  its repositories reached by the box's fake git, its CI a head's own,
  `check` passing unless the head's `.fake-forge/ci` says otherwise
  (`../fakes.md`, 2). `project-landing` connects it as its admin, with
  its first repository, in one ctl line that names the box's values
  rather than writing them (`../../client/drive.md`, 3.4).
- **Through ctl, not the web,** since the web's connect takes the
  admin's token itself, which a recipe never names, and gives no git
  program, so the box's agents would reach no fake forge's git; ctl
  takes the token's file and `--git-program`, both named by the box's
  values. The web journey's "Run on this machine" stays the journeys'
  (`../../client/web/journeys.md`, 8.2): QA's state is the one it
  reaches, Hello with `work/hello` and ChatGPT, on the fake Forgejo in
  place of the box's own.
- **No `scripts.json`** of their own, so the box's generated scripts
  answer, strict (`../../protocol/setup/box.md`, 5).

| Recipe | After | Its lines | Reaches |
|---|---|---|---|
| `welcome` | none | its check alone | a fresh box's welcome, nothing connected |
| `project-landing` | `welcome` | `ctl project make Hello`; `ctl forge connect forgejo-admin {forgejo.admin} --first work/hello --project Hello --api {forgejo.api} --git {forgejo.git} --git-program {git}`, waited until `work/hello` is adopted; the fake ChatGPT plan signed in through the web client, as a person signs a model in once a forge is connected; its check a `web see` of Hello's home naming `work/hello` | Hello, `work/hello` its home on the box's fake Forgejo, the fake ChatGPT plan signed in |
| `chat-draft` | `project-landing` | the web journey's, unchanged | Hello's first chat with its first Away mode draft written |
| `ctl-hello` | none | `project-landing`'s three ctl lines; `ctl account add-key 1 {model.key} --key qa-model`; `ctl account list --until usable`; its check, after `check`, `ctl project show Hello` and `ctl account list --until usable` | Hello, `work/hello` its home on the box's fake Forgejo, the box's fake model the person's account, and any change's review answered approving |

- **`ctl-hello`, ctl alone:** the catalogue's first recipe whose chain
  holds no `web` and no `tui` line, so that triage with neither surface
  configured offers it (`triage.md`, 3) and its walk is a ctl session
  (`../../client/recipe.md`, 5.1; `../../client/drive.md`, 2.6). A root,
  since `welcome`'s check and `project-landing`'s sign-in are web lines.
  Its `recipe`, whole:

  ```text
  recipe 1
  ctl project make Hello --key qa-hello-project
  ctl forge connect forgejo-admin {forgejo.admin} --first work/hello --project Hello --api {forgejo.api} --git {forgejo.git} --git-program {git} --key qa-hello
  ctl forge list --until made:1
  ctl account add-key 1 {model.key} --key qa-model
  ctl account list --until usable
  check
  ctl project show Hello
  ctl account list --until usable
  ```

- **The fake model as the person's account:** provider 1, OpenAI, whose
  endpoint the box's fake model serves; its key the box's fake model
  key, named by its file alone (`../../client/drive.md`, 3.4), never
  read. Without `--deployment`, a key-only account of the signed-in
  person (`../../client/ctl/areas.md`, area 5), which serves the
  person's own work: a change the person opens is admitted as their
  batch and its reviews spend their pool
  (`../../domain/forge/change/opened.md`, 3), so its review runs on it,
  while work no person funds finds no account and waits rather than
  failing under strict. With OpenAI's alone, the reviewer's charter
  takes `gpt-6.1-sol`, which the fake model serves.
- **Its scripts:** `scripts.json` holds `cued` alone, one script: B0's
  twin's review (4.3), cued by `Review this change at the named head`,
  its four turns the twin's, `read_pull`, `read_pull_files` and
  `read_checks`, each taking the pull request's number and head from
  the review's brief (skein's `testing/fakes/llm.md`, 2.6), then
  `finish` with verdict `1`, approving, all but its text, which reads
  "Approve. QA's scripted review read the pull request, the files it
  changes and its checks." No `asked` and no `otherwise`: under strict
  any other turn fails, naming its request (skein's
  `testing/fakes/llm.md`, 2.5), so a case whose change needs a model's
  turn beyond its review, a goal's or a chat's, gives those scripts in
  a child recipe (`places.md`, 6).
- **What it serves:** ctl's `change`, `landing`, `repository` and
  `project` groups on a change that lands by the stock rule, gate 1's
  review approving: a branch a case's `forge push` makes and
  `ctl change open` opens (`../../draft/mechanical-changes.md`), and any
  change the person funds whose only model turns are its review's.

## 3. The TUI's

- **Their starting screens** are `project-landing`'s, as the TUI's
  journeys start from it (`../../client/tui/journeys.md`, 9): a TUI
  case's lines start the TUI at its first `tui` line, signed in by the
  drive session (`../../client/drive.md`, 3.3), so a case on the
  launchpad starts from `project-landing` with no recipe of its own.
- **`tui-chat`,** after `project-landing`, with no `scripts.json`: the
  steps of the `launchpad-chat` journey's walk past its approval
  (`../../client/tui/journeys/launchpad-chat/walk`), as drive lines, from
  `web reload "/projects/1"`, as the journey's setup ends, to the fake
  model's answer, a `tui see` of it that waits, then `web reload "/"`,
  a step that is no `see`, so that the answer's `see` stays a step
  (`../../client/recipe.md`, 3.1); its check `tui see "/chats 1"`, the
  chat's row on the launchpad of the TUI the check starts afresh after
  the walk's `tui stop` (`../../client/recipe.md`, 5.1;
  `../../client/tui/pages/launchpad.md`, 4). It reaches Hello
  with an Ask temper chat asked and answered from the TUI, and serves
  the TUI's chat screen (`../../client/tui/pages/chat.md`) and the web
  client's view of a chat the TUI started.
- **A TUI state** a later case needs is a pass's to add, as any other
  (`places.md`, 6): its lines are `tui` lines, and its check `tui see`
  or `web see` lines, read on the root page and a TUI started afresh
  (`../../client/recipe.md`, 5.1).

## 4. The twins

### 4.1 What a twin setup is

- **A real run replayed** (`../../draft/landed-qa.md`, 6): the run's
  agents say what the real model said, from its twin's scripts
  (`../real-loop.md`, the ladder's twins), and the person's acts are
  its recipes' lines, in the walk's words, through ctl as the walk made
  them (`../../../development/acceptance.md`, 6 and 7). Its parent is
  the state the walk started from.
- **On the box's fake Forgejo,** as every setup (section 2): the walk's
  ChatGPT, with the fake Forgejo in place of the walk's owned one, its
  repository, pull requests and CI the fake's. A head's CI is what it
  holds (`../fakes.md`, 2): B0's green CI is a head that declares
  nothing, and B1's failed CI and its repair are the gate's declaration
  (4.4), as the fixtures' own replays judge them
  (`../real-loop.md`, the ladder's twins).
- **Strict:** every turn the box asks for is the twin's or fails
  (`../../protocol/setup/box.md`, 5), so a twin that no longer covers a
  run blocks the cases on it, naming its scripts, and is never a
  finding against temper (`../../draft/landed-qa.md`, 6).

### 4.2 What differs on the box

- **Numbers and ids the walk's box gave and this one does not:** the
  tasks' numbers, since the parent's state holds tasks of its own; the
  pull request's, numbered by the box's fake Forgejo; and every head
  and merge, made by the fake's git. Each is a take (skein's
  `testing/fakes/llm.md`, 2.6): the twin's calls and texts name what
  the box's conversation holds, from a tool's output or the agent's
  brief, rather than what the walk's held.
- **Made by `temper-twin`,** never by hand: its `--takes FILE`
  (`../fakes.md`, 1) writes each take into the turns that use it, from
  a `takes.json` kept beside the twin's fixture, and the fixture's
  `regenerate.sh` passes it, so a twin made again keeps its takes.
- **Proven by the twin's fuzzy story:** the fixture, with its takes,
  is what `suite::fuzzy::green_light` replays (`../real-loop.md`, the
  ladder's twins), so each take is found in the conversation smith
  writes and the story reaches its end as before. Whether the takes
  find the box's values is the walk after landing's
  (`../../draft/landed-qa.execution.md`, 1).
- **One source:** a twin setup's `scripts.json` is its fixture's
  `cued` and `asked`, member for member, with no `otherwise`, so that
  under strict a call no cue selects fails; a focused test holds the
  two equal.

### 4.3 B0's

From `temper/tests/real-loop/tests/fixtures/b0/`, the rung's walk
(`../../../development/acceptance.md`, 6):

| Recipe | After | Its lines | Reaches |
|---|---|---|---|
| `widget` | `project-landing` | `ctl project make widget`; `ctl repository make --project widget --forge 1 playground/widget`, waited until adopted; then `forge commit forgejo playground/widget` of the B0 fixture's `forge.json` tree, `README.md`, `src/main.rs` and `.forgejo/workflows/check.yml`, from its `files/`, each as the tree holds it, then `forge checks forgejo playground/widget`; its check a `web see` of widget's project | the project widget, `playground/widget` its home on the fake Forgejo holding the widget the rungs start from, beside Hello (`../../../development/acceptance.md`, 5) |
| `b0-plan` | `widget` | the person's request, `person.json`'s `request`, in a chat of widget's project through ctl, waited until the work's question is in the inbox; its check a `web see` of the question's words | the work's plan asked and waiting for its go-ahead, nothing pushed |
| `b0-landed` | `b0-plan` | the person's answer, `person.json`'s `answer`, through ctl's inbox, waited until the chat's last word; its check a `web see` of `person.json`'s `last_word` | the change landed on `main` with CI green and an agent's review passed, the chat saying so |

- **`widget`'s tree** is the fixture's, committed by the person rather
  than given by `--playground`, which only an owned Forgejo takes; the
  starter workflow `repository make` gives stays beside it, a file the
  fake's CI does not read.
- **`b0-plan`'s `scripts.json`** is B0's twin's; `b0-landed` has none
  of its own, since its chain gives its parent's.
- **Their pages:** the chat, Needs you and its question, the goal and
  its change, Work and the project's Home at each state; a case on a
  later page of the same goal starts from the nearest.

### 4.4 B1's

From `temper/tests/real-loop/tests/fixtures/b1/`, the rung's walk
(`../../../development/acceptance.md`, 7), from `widget`:

| Recipe | After | Its lines | Reaches |
|---|---|---|---|
| `b1-plan` | `widget` | first the person's gate: `forge commit forgejo playground/widget` (`../../client/drive.md`, 3.5) with the walk's message, "CI: each pull request adds changes/<its number>.md", of three files from its `files/`: the two gate files of the fixture's `forge.json` tree, its workflow `ci.yml` under `.forgejo/workflows/` and its contributing guide, each as the tree holds it, and `.fake-forge/ci`, reading `pull-file changes/{pull}.md`; then `forge checks forgejo playground/widget`; then the person's request, `person.json`'s `request`, in a chat of widget's project through ctl, waited until the work's question is in the inbox; its check a `web see` of the question's words | widget gated on `main`, its push's checks settled, and the work's plan asked and waiting for its go-ahead, nothing pushed |
| `b1-landed` | `b1-plan` | the person's answer, `person.json`'s `answer`, through ctl's inbox, waited until the chat's last word; its check a `web see` of the chat's last word, as the twin's last chat turn gives it | the change landed on `main` after its first CI failed and temper's repair passed on the same pull request, an agent's review passed, the chat saying so |

- **The gate is the twin's,** not today's text of
  `../../../development/acceptance.md`, 7: the fixture's walk met the
  gate its `forge.json` holds, and its repair writes what that gate
  checks, so a later gate the twin never met would fail it. A focused
  test holds the recipe's two gate files equal to the fixture's.
- **`.fake-forge/ci` stands for the gate's `changes` job** on the fake
  (skein's `testing/fakes/forge.md`, 2.3): a pull request's head fails
  `check` until it holds `changes/<its number>.md`, so the twin's first
  head fails and its repair passes, as the fixture's own replay judges
  them with its `changes` context; the push's head, no pull request's,
  is skipped, as the walk's `changes` job was on a push. The fake
  reports one context, `check`, where the walk's runner reported two.
- **`b1-plan`'s `scripts.json`** is B1's twin's; `b1-landed` has none
  of its own.
- **Their pages:** as B0's, and the change's failed CI, its repair and
  the repaired head on the same pull request between the two.

## 5. A deployment's

The catalogue's first deployment setup, `turn-landing`
(`../../draft/landed-qa.md`, 5 and 10): a project on the place's fake
Forgejo landing by `turn` at a `batch` of two, its gate a workflow the
fake Forgejo dispatches, so that landing by turn, which no box can be
set to, is judged first, and landing by turn's cases (merge-queue M2's
and M3's, `../../draft/merge-queue.execution.md`) run through the
machinery alone. A root recipe, at a deployment's place
(`deployment.md`).

- **Its `config/`** (`deployment.md`, 2): the engine alone, one worker
  of two slots at least, its agent on the fake model, as the cluster's
  fixture writes them (`../../protocol/cluster.md`, 5), each address,
  directory and program a `{place.NAME}`; the fake Forgejo's document
  with no seeded repository, its admin token under `{place.data}`, its
  made repositories' CI `{"declared": ".fake-forge/ci"}`, its
  `workflows` the gate, `[{"file": "gate.yml", "jobs": ["gate"],
  "cue": {"declared": ".fake-forge/gate"}}]`, and its `manual_merges`
  `{"allow": true, "autodetect": true}` (`../fakes.md`, 2); the fake
  model's document, its outputs under `{place.data}`. The engine's
  `application.clients.bundle` is `"{place.bundle}"`, so that its
  clients' listener serves the place's bundle, the kept build's, to the
  session's browser and the TUI's approval (`deployment.md`, 2), at
  every walk and session, ctl's among them.
- **Its lines,** ctl's and the deployment's alone: the fake model's key
  added as the deployment's account (`ctl account add-key ... {model.key}
  --deployment`) and the charters' models set to the fake model's, as
  the cluster's story does; the project Hello and the fake Forgejo
  connected as its admin with its first repository `work/hello`, by the
  same ctl line as `project-landing`'s (section 2), waited until it is
  adopted; then `deployment patch engine landing.json`, whose file sets
  Hello's landing by `turn`, `batch` 2, its repository's gate `gate`,
  workflow `gate.yml`, blocking, on the forge and repository numbers
  those lines made (`deployment.md`, 6.2; `../../domain/landing.md`,
  13.1); its check, after `check`, ctl's reads of the project, its
  forge and its repository. The forge's read waits until it is
  connected, `ctl forge list --until connected:1`, as the place's own
  reads do: the engine the patch starts again, and the one a walk from
  the setup's copy starts, each connects the forge afresh, showing it
  connecting meanwhile, and its repositories are read only once it is
  connected (`../../client/ctl/tools.md`, 4), never after a sleep.
- **Its gate by content:** a dispatched `gate.yml` at a landing's top
  passes unless the top's tree holds `.fake-forge/gate` reading `fail`,
  so a case makes a batch red by a member that writes that file, and
  green by members that leave it (`../../draft/landed-qa.md`, 6 and 10).
- **Its `scripts.json`:** the fake model's scripts for its cases' goals,
  matched by their cues and taking their numbers and heads from the
  conversation (skein's `testing/fakes/llm.md`, 2.4 to 2.6): two goals
  started on Hello whose changes into `main` are ready at one forming
  decision, so that they land in one landing of two
  (`../../domain/landing.md`, 9), by a latch the person releases
  through ctl, as the owned Forgejo's red narrowed keeps its changes
  together (`../process-tier.md`, landing by turn on a real Forgejo), and their
  reviews; each change's content its own file, a member made red by its
  `.fake-forge/gate`. The same scripts are the real loop's fixture for
  the story that proves them (section 6).
- **Its README** says the state and the ctl groups it serves: `landing`,
  `change`, `goal`, `task` and `repository`.
- **Ctl alone,** so that its walk is a deployment's ctl session
  (`../../client/recipe.md`, 5.1) and triage with ctl alone configured
  offers it (`triage.md`, 3).

### 5.1 Seen through the web client and the TUI

`turn-landing-clients`, after `turn-landing`: the same state, Hello
landing by turn, seen through the web client and the TUI on the
deployment's engine (`deployment.md`, 6.1), so that a landing's web and
tui cases on landing by turn start from a setup that proves both reach
the deployment, and the catalogue holds a deployment's setup of web and
tui lines (`../../draft/landed-qa.execution.md`, Q7).

- **Its lines:** steps that open Hello's home in the web client and
  read its repository there, and start the TUI and read its launchpad;
  then `check`, a ctl read of the project, and the web client's root
  page and the TUI's launchpad read again, after the walk's `web reload
  "/"` and `tui stop` (`../../client/recipe.md`, 5.1). Its words are
  those the same pages show at a box for the same state, from
  `project-landing`'s and `tui-chat`'s lines (section 2 and 3), the
  page's own design (`../../client/web/pages/`,
  `../../client/tui/pages/`) settling any doubt. By default:

  ```text
  recipe 1
  after "turn-landing"
  web reload "/projects/1"
  web see "work/hello is in Hello"
  tui see "Ask temper"
  check
  ctl project show Hello
  web see "Nothing needs you."
  tui see "Ask temper"
  ```

- **No `scripts.json`, `files/` or `config/`** of its own: its chain's
  scripts are `turn-landing`'s, and no line of it asks the fake model.
- **Its walk** is a deployment's full session, its bundle the place's
  (`../../client/recipe.md`, 5.1; `places.md`, 4), from `turn-landing`'s
  copy or a fresh place.
- **Its README** says the state and the pages and screens it serves:
  the web client's home and Hello's project home, and the TUI's
  launchpad, for landing by turn's web and tui cases.

### 5.2 `stock-landing`

`ctl-hello`'s state (section 2) at a deployment's place: Hello on the
place's fake Forgejo with `work/hello` its home, the fake model the
person's own key account, and any change's review answered approving,
so that a change the person opens lands by Hello's stock rule. A root,
so that a place of the deployment's kind serves ctl cases on a change
that lands by the stock rule, which `turn-landing` does not
(section 7; the user, 2026-11-16). It and `ctl-hello` are one setup at
the two kinds of place, not two concepts: the same lines, the same
scripts and the same state, each at a place of its kind.

- **Its `config/`:** `turn-landing`'s five documents, file for file
  (section 5): the fake Forgejo's gate workflow and manual merges are
  declared there but never dispatched, since no `deployment patch`
  gives Hello a landing by turn, and the engine's
  `application.clients.bundle` is the place's bundle.
- **Its `recipe`:** `ctl-hello`'s, byte for byte: `ctl account add-key
  1 {model.key} --key qa-model` names provider 1, which is the fake
  model's provider in `turn-landing`'s engine document as OpenAI's is at
  a box, and every other line names values a deployment serves
  (`deployment.md`, 6.1). Without `--deployment`, the account is the
  person's alone, as at a box: work no person funds finds no account
  and waits rather than failing under strict.
- **Its `scripts.json`:** `ctl-hello`'s, byte for byte: B0's twin's
  review, approving.
- **No `files/`** and no `deployment patch` line: Hello lands by the
  stock rule.
- **Its README** says the state, that it is `ctl-hello`'s at a
  deployment, and the ctl groups it serves: `change`, `landing`,
  `repository` and `project`.
- **Ctl alone,** so its walk is a deployment's ctl session
  (`../../client/recipe.md`, 5.1) and triage with ctl alone configured
  offers it where a deployment's place is configured (`kinds.md`, 6).

## 6. Their tests

Focused, with no box: every recipe of the catalogue reads, its chain
and hash, as `recipe list` gives them (`../../client/recipe.md`, 7); no
recipe's lines set up the box's own Forgejo, no `forge setup` act and
no press of "Run on this machine"; `project-landing` connects the fake
Forgejo through the box's values; a twin setup's `scripts.json` reads in
the box's `--scripts` form and equals its fixture's `cued` and `asked`;
`widget`'s files equal the B0 fixture's tree; `b1-plan`'s `forge commit`
names the B1 fixture's gate files, its files equal to theirs, and its
`.fake-forge/ci` reads `pull-file changes/{pull}.md`. `ctl-hello` reads
and hashes as a root whose chain holds no `web` and no `tui` line; its
walk from a fresh box is a ctl session given its steps and then its
check, with no `web reload "/"` and no `tui stop`, its `{model.key}`
resolved from the box's line and kept as written; its `scripts.json`
reads in the box's `--scripts` form and its one script is B0's twin's
review but for the verdict's text. `turn-landing`'s
`config/` holds its five documents, each rendered with a place's names
(`deployment.md`, 2), `{place.bundle}` among them, refusing none and
parsing as its JSON shape, its engine's `application.clients.bundle`
the place's bundle; its engine's document, so rendered, passes every
check the engine's start makes, so that a figure the start ties across
its documents is held here and not first at a walk: its
`application.limits` carry `forge.judge_projects` 8, jig's
`authority.projects` in `standard`, and `forge.client.op_bytes` the
engine's `tasks.result_bytes`, 4,096, the bound the root holds it to,
as init's document does (`../../protocol/setup/init.md`, 1); its
forge document declares the gate and the manual merges; its patch file
sets `turn`, `batch` 2 and the gate; its `scripts.json` reads in the
fake model's `--scripts` form and equals its real-loop fixture's. In
the real loop's world, those scripts carry two goals' changes to ready
together and land them in one landing of two, by `turn` at `batch` 2,
the gate dispatched on the fake Forgejo and green, and a member holding
`.fake-forge/gate` reading `fail` turns it red
(`../../draft/landed-qa.md`, 11). `turn-landing-clients` reads and
hashes as a deployment's chain after `turn-landing`, holding web and tui
lines; its walk is planned as a deployment's full session given
`turn-landing`'s `config/`, `--ports` and `--bundle`, with the walk's
`web reload "/"` and `tui stop` before its check. `stock-landing` reads
and hashes as a deployment's root whose chain holds no `web` and no
`tui` line, its walk planned as a deployment's ctl session given its
`config/` and `--ports`; its `recipe` and `scripts.json` equal
`ctl-hello`'s and its `config/` equals `turn-landing`'s, file for file.
The walks are the walk's after landing (`../../draft/landed-qa.execution.md`, 1).

## 7. Each box setup weighed

`../../draft/landed-qa.md`, 13 keeps a box's setup that needs nothing of
a box as the box's until the one framework weighs each (`kinds.md`, 7).
A box's setup needs a box where its state, or a step of its chain,
reaches what only a box gives: the box's welcome, the fake ChatGPT plan
signed in through it, the scripts the box generates for its fake model,
or a B0 or B1 rung walked on a box (`../../../development/acceptance.md`,
6 and 7).

| Setup | What of a box it needs | Weighed |
|---|---|---|
| `welcome` | the box's welcome, its check | the box's |
| `project-landing` | the fake ChatGPT plan signed in at the box's welcome, after `welcome` | the box's |
| `chat-draft` | the box's generated scripts, which answer its chat and write its draft, after `project-landing` | the box's |
| `tui-chat` | the box's generated scripts, which answer its chat, after `project-landing` | the box's |
| `widget` | `project-landing`'s chain | the box's |
| `b0-plan`, `b0-landed` | B0's rung on a box, after `widget` | the box's |
| `b1-plan`, `b1-landed` | B1's rung on a box, after `widget` | the box's |
| `ctl-hello` | nothing: ctl lines alone, each value it names served at a deployment too (`deployment.md`, 6.1), its scripts its own | the box's, and its state at a deployment's place is `stock-landing` (5.2) |

So every box setup stays the box's, for QA of the box itself (the user,
2026-11-16: moving `ctl-hello` would take the box's only setup of ctl
lines alone, which its ctl passes start from). The one that needs
nothing of a box is reached at a deployment's place too, by
`stock-landing`, so that ctl cases on a change landing by the stock
rule have a setup at either kind: `ctl-hello` and `stock-landing` are
one setup in two places, read as one, the same lines and scripts at a
place of each kind. A deployment's setups start from a root of their
own, `config/` and all (section 5).

## 8. Open questions

- **A twin's later states:** B0's plan and landing are its setups. A
  state between, CI running or a review under way, is a stopped box mid-run.
  Default: none until a case needs one, then the pass's own (`places.md`, 6).
