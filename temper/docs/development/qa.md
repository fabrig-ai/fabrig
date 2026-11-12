# QA of temper

Provisional, 2026-10-26, revised 2026-11-04. What temper offers an agent
driving it in a pass of QA after landing: whom it plays, the deployment
it drives, the hands it holds, how it reads the forge and how it checks
an outcome. The product's contracts it relies on are temper's design:
the box's forge hands (`../design/protocol/setup/box.md`, 3) and ctl's
help (`../design/client/ctl/testing.md`, 5). What is still open is
`qa.md`, section 8.

## 1. In one page

- **A pass is one of temper's users:** an agent working with temper for
  a person. It has what any such agent has: temper's help (`temper
  help`, `temper ctl help`), `temper ctl`, and the tools it chooses for
  the systems temper works with (section 2).
- **temper is agnostic about those tools.** It serves its own part, and
  names the rest by id and link; how a user reaches the forge is the
  user's choice. A pass here chooses `curl` (section 5).
- **temper says what it is doing, why, and where; the forge shows what
  is there.** A change's step, hold and landing are temper's; its diff,
  reviews, CI logs and branches are the forge's (`../design/client/forge/README.md`,
  section 1).
- **The deployment is a box,** on its fake backends or on real ones,
  each pair of backends a rung (section 3).
- **Three hands:** ctl as the signed-in person; the forge read as an
  observer, with reads alone; and the person on the forge, for what
  people do there. Never temper's bot or its tokens, its store, its HTTP
  API or a fake's internals (section 4).
- **The box hands each hand over,** printed as its other files are, so
  that a pass reads no design to set up (section 4).
- **A check compares two accounts,** temper's through ctl and the
  forge's through its API. A finding is temper not saying where to
  look, the two disagreeing, temper stale past its bound, help that
  does not get the pass there, or temper's words wrong; reading the
  forge is none (section 6).
- **What the design has and the code does not yet** is named once as
  blocked, never found again each pass (section 6).

## 2. Whom a pass plays

- **An agent working for a person,** with the person's credential,
  signed in by the box's printed link (`../design/client/ctl/README.md`, section 7).
  ctl is its tool for temper; what it uses for anything else is its
  own, as for any of temper's users.
- **temper's part, through ctl:** tasks, chats and their activity; a
  change's way to landing, its step in words, its hold, its place in a
  queue, its gates' verdicts and, landed, its merge commit, read with
  `change show TASK`; the forges and their repositories. Each comes with the ids
  and links that find it on the forge (`../design/client/ctl/areas.md`, area 6).
- **The forge's part, on the forge:** a pull request's diff, files and
  reviews; CI's runs, jobs and logs; branches and their heads. A part
  temper does not serve says "Shown on the forge.", with the resource
  in the forge's words (`../design/client/ctl/tools.md`, 2).
- **What temper owns on the forge changes through temper:** its
  branches, its pull requests and their landing. What a person does on
  the forge (a review, an approval at a head, a comment, a push to a
  branch of their own) they do there as themselves, and temper reads it
  (`../design/client/forge/README.md`, sections 4 and 5).
- **temper's own agents are another user.** A chat or a coder reads the
  forge only through its host's read tools, within its authority, since
  a worker holds no forge credential (`../design/domain/agent/tools.md`). A pass
  tests that as one of temper's features, as any other.

## 3. The deployment

- **A box in the pass's scratch directory,** started as the walk `box`
  says (`temper help box`), with the flags of its rung, and stopped by
  the pass before it ends. Each rung's stories are `acceptance.md`.
- **Landed QA's passes start elsewhere:** on the fakes' rung, at every
  landing, from a setup's copy restored at a place, or made again from
  its recipe, with the pass's build installed in the place's `bin/`
  (`../design/testing/qa/places.md`); their hands are this document's,
  with the pages and the TUI driven a line at a time
  (`../design/client/drive.md`).
- **The rungs:**

  | Rung | Forges | Model | What it needs |
  |---|---|---|---|
  | fakes | `--fake-backends` (`../design/protocol/setup/box.md`, 5): GitHub and a Forgejo elsewhere faked; Forgejo here real, from the box's cache | the fake model, by the key file the box prints | none |
  | a real model | Forgejo here, set up through ctl, as the walk `box` sets it up | ChatGPT, a login imported and exported back, or a key read from a private file (`../design/client/ctl/areas.md`, area 5) | the model's account |
  | GitHub | GitHub, through an App, on a repository the pass may change | as a real model's rung | the App's key and its installation, and a person's token on that repository |
  | Claude | Forgejo here, as a real model's rung | Claude Code on a person's own login, its configuration directory used in place (`../design/protocol/credentials.md`, 3) | the model's login |

- **The model is set before the first start.** Init's model entry is
  kept as it is (its window, prices and output), its efforts declared in
  `temper.json`'s `engine.llm` model, the first being what a task with
  no choice of its own takes. Every charter takes the first configured
  model as its main model, and the stock's spends are sized from it at
  the start (`../design/protocol/configuration/stock.md`). After the
  start, `temper ctl settings show charter 1` names the model; a box
  that shows otherwise is a setup to redo, not a finding. A chat is
  started with `--model MODEL --effort EFFORT` where its model is named;
  template 1's choice (`settings apply`, the whole choice: `{"main":
  {"model": MODEL, "effort": EFFORT}, "account": null, "subagents":
  []}`) sets it for every chat of a running box.
- **Its room is raised beside it,** in `temper.json` before the start,
  where a pass needs more than the profile gives: `engine.limits`'
  override `"people.requests"`, since each deciding request's answer is
  kept a day, which a busy pass fills, after which every keyed request
  answers busy (jig's `domain/people/acts.md`, 1); and
  `application.limits`' override `"root.loads.loads"`, since a restart
  with held work loaded needs a slot for it. Its retention stays a day,
  and the memory the start then names is raised as
  `../design/protocol/configuration/settings.md`, 7 says. These are a pass's,
  as the budget is; the product's own values are its design's.
- **One configuration, checked before every start.** Every change above
  is made in `CONFIG` after `temper init` and before the box's first
  start, and `memory_bytes` raised by what the start names when it
  refuses a short one. Before the first start and before every restart,
  a pass checks that `CONFIG` says what it set; one that differs is a
  setup to redo, never a finding.
- **Give back before you break it.** A case that aborts `serve`, kills
  the box or otherwise risks the deployment's restore first exports a
  login moved into the box (`account export`): what the deployment holds
  of its work (a task, an unanswered question) stays in its store, and a
  restart needs no model. It imports the login again only if a later
  case needs the model. A login a broken deployment still holds is a
  finding, said with the case that broke it.
- **A budget never stops a pass.** A pass raises its box's budget as a
  person would (`temper help budget`), and says the figures. Init's
  first start makes no project; after sign-in, `me` lists none. Before
  `forge setup`, the pass raises the deployment limits in one `settings apply` edited from
  `settings show rules --json` and `settings show policy 1 --json`: the
  rules' spend and period spend, policy 1's ceiling's spend and period
  spend, and role 0's authority spend and its period spend. The period
  stays larger than role 0's pool, so that the deployment's own work (a
  landing task, a tier run, a queue's repair) is still funded. Nothing
  per run; the chat's template stays as allowed. Policy 1 sizes future
  projects. The pass then runs `forge setup` without `--project`, as
  the box walk says: it creates `hello` and adopts `work/hello` as its
  home, printing `project: PROJECT · hello`. Use that printed number
  for later commands, 1 on a new box. `forge setup --playground` without
  `--project` instead creates `widget`, printing `project: 1 · widget`
  on a new box. `--project PROJECT` chooses an existing project.

  ```
  temper ctl DIR forge setup --org work --repository hello --key setup
  temper ctl DIR me
  temper ctl DIR project show PROJECT
  ```

  For an existing project, setting its `project-policy` immediately
  resizes the open period and parties' pools to its spends; a policy
  item may share a settings file with rules and other settings. For
  example, accepting project-policy `period_spend` 8200000 beside rules
  `period_spend` 8300000 leaves the open period's budget 8200000 in
  `project show`. Keep the values' other fields as read. Changing policy
  1 alone leaves an existing project's budgets as they are. To resize
  budgets separately, fund the project with distinct operation keys,
  the project first because the owner's pool reserves from its period:

  ```
  temper ctl DIR project fund PROJECT PERIOD --key budget-period
  temper ctl DIR project fund PROJECT POOL --role 0 --key budget-owner
  temper ctl DIR project show PROJECT
  ```

  `PROJECT` is setup's returned project, also listed by `me`; `PERIOD`
  and `POOL` are the pass's figures. `project show` prints the open period's
  budget and the owner's pool's, with their reserved, spent and left
  balances. These funding calls resize the open budgets at once.
- **Real accounts are the person's,** used by path and never read; a
  login moved into a box goes back before the box stops, an imported
  login exported to its file.

## 4. The hands

| Hand | Who | For | Given as |
|---|---|---|---|
| ctl | the deployment's first owner, the person | every act on temper, and temper's account of it | the box's printed sign-in link, on stdin to `ctl sign-in -` |
| the observer | the forge's person, with reads alone: on Forgejo, `read:repository`, `read:issue`, `read:organization` and `read:user` | section 5's reads | a header file the box names |
| the person on the forge | the forge's person, with a token that writes | what a person does there, as a case's step names it: a review, an approval at a head, a comment, a push, a branch moved under temper | a header file the box names |

- **Never:** temper's bot or its tokens, the preparation's
  `admin_token`, temper's store or its HTTP API, a fake's internals or
  its observations. A value the box prints for a person to paste (a
  model key, a fake's admin token, a PEM) reaches temper through ctl,
  as the welcome's paste does.
- **What the box prints:** its sign-in link; on fake backends the
  fakes' addresses and the files a person pastes (the walk `box`, on
  fake backends). An owned forge's terminal lines say:

  ```text
  HH:MM:SSZ Forge FORGE (Forgejo): starting
  HH:MM:SSZ Forge FORGE (Forgejo): started
  ```

  The forge's API and the person's login are in
  `DATA/box/logs/forgejo-FORGE.log`, each after its UTC time:

  ```text
  YYYY-MM-DDTHH:MM:SSZ Forgejo: api http://ADDR/api/v1
  YYYY-MM-DDTHH:MM:SSZ Forgejo: person LOGIN
  ```

  The person's two header files are `person_read.header` and
  `person_write.header` in `DATA/forge-FORGE/` (`temper help box`).
  `../design/protocol/setup/box.md`, 2 has the log name their paths too;
  that gap is recorded in `../design/protocol/setup/README.status.md`.
  Each header file holds one line, `Authorization: token TOKEN`, private
  to the box's user, so that a pass passes a token by its path and never
  reads it.

## 5. Reading the forge

A pass here reads Forgejo with `curl`, the observer's header by its
path:

```sh
curl -fsS -H @READ "$API/repos/OWNER/NAME/pulls/7"
```

| What | Read, under `API` |
|---|---|
| a change's pull request, by the number ctl prints | `GET /repos/O/R/pulls/N` |
| the pull request from a branch | `GET /repos/O/R/pulls/BASE/HEAD` |
| its reviews; its files | `GET /repos/O/R/pulls/N/reviews`; `GET /repos/O/R/pulls/N/files` |
| CI at a head, combined | `GET /repos/O/R/commits/SHA/status` |
| the Actions runs at a head | `GET /repos/O/R/actions/runs?head_sha=SHA` |
| a run's jobs; a job's log | `GET /repos/O/R/actions/runs/ID/jobs`; `GET /repos/O/R/actions/jobs/ID/logs` |
| a branch and its head | `GET /repos/O/R/branches/B` |
| a file at a commit | `GET /repos/O/R/raw/PATH?ref=SHA` |

- **Each read is one skein's fake forge serves,** decoded by
  skein-forge's Forgejo requests, so that a pass reads a fake as it
  reads Forgejo 17. A read the fake refuses is a finding against skein's
  fake.
- **The person's writes** are Forgejo's API with the write header, as
  the case names them; a pass writes only what a case needs.
- **GitHub's reads,** on its rung: GitHub's REST API by `curl` at
  `https://api.github.com`, with the person's header file, a
  fine-grained token on the pass's repository, by its path:

  | What | Read |
  |---|---|
  | a change's pull request; its reviews; its files | `GET /repos/O/R/pulls/N`; `.../pulls/N/reviews`; `.../pulls/N/files` |
  | CI at a head: check runs; commit statuses | `GET /repos/O/R/commits/SHA/check-runs`; `.../commits/SHA/status` |
  | the Actions runs at a head; a run's jobs | `GET /repos/O/R/actions/runs?head_sha=SHA`; `.../actions/runs/ID/jobs` |
  | a job's log | `GET /repos/O/R/actions/jobs/ID/logs`, a redirect `curl -L` follows |
  | a branch and its head; the branches under a prefix | `GET /repos/O/R/branches/B`; `GET /repos/O/R/git/matching-refs/heads/PREFIX` |
  | a file at a commit | `GET /repos/O/R/contents/PATH?ref=SHA`, with `Accept: application/vnd.github.raw` |

  The box prints no forge hands for GitHub, which has no person of the
  box's: the person's header file is the pass's own. Its 5,000 requests
  an hour are the person's own, apart from temper's installation's, so a
  pass reads at a person's pace, a wait on temper's side between reads,
  never a loop of reads.

## 6. Checking an outcome

- **Each case, one loop:** act through ctl and wait on it
  (`../design/client/ctl/tools.md`, 4); read temper's account; follow its ids and
  links to the forge; compare.
- **A case's line says both,** what temper said and what the forge
  showed.
- **A finding is one of:**
  - **No way to the forge:** an outcome temper reports with no id or
    link to its place there, such as "Landed" with no pull request.
  - **The two disagree:** landed in temper while `main`'s head is not
    its merge commit; CI green in temper and red on the forge.
  - **temper stale past its bound:** the forge shows what a wait on
    temper does not see within the wait's timeout.
  - **The help does not get there:** a step the pass could find only
    in the design or the source, or a refusal that does not name what
    is missing.
  - **temper's words wrong:** a state said that is not so.
  - **A start refused on a link, or one that panics on it** (jig's
    `domain/engine/restart.md`): a writer's bug; the finding names
    the record and the link from serve's log, with the case that wrote
    the record.
- **Not a finding:** that the pass read the forge, or how.
- **Known gaps:** what the design has and its status files list as not
  yet built is named once as blocked, with what the pass did instead.

## 7. What it asks of temper

- **The box's forge hands** (section 4): for each forge it sets up or
  fakes, its API's address, the person's login, and a read and a write
  token as header files, printed with its other lines
  (`../design/protocol/setup/box.md`, 3).
- **ctl's help** in these terms: the forge read as the agent chooses,
  and what temper owns there changed only through temper
  (`../design/client/ctl/testing.md`, 5).

## 8. Open questions

- **The GitHub rung's App:** made by the person for the pass, or from
  temper's manifest. An installation that exists already is accepted
  with no browser, by `forge accept FORGE INSTALLATION`
  (`../design/client/ctl/areas.md`, area 6).
- **Holding section 5 to the fake:** a step test that decodes each read
  with skein-forge's Forgejo requests, as the help's ctl lines are held
  to ctl's parser (`../design/protocol/documentation.md`, section 1). Default:
  none, until a pass meets a read the fake refuses.
- **The browser on a real rung:** a pass here drives ctl; landed QA
  drives the pages and the TUI on the fakes' rung through the drive
  session (`../design/client/drive.md`; `../design/testing/qa/places.md`,
  5). Default: a real rung's pass takes the same session once a case
  needs a real backend (`../design/draft/landed-qa.md`, 12).
