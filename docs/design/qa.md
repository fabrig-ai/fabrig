# QA of temper

Provisional, 2026-10-26, with the user. What a pass of QA after landing
(agents' `docs/design/qa-flow.md`) needs of temper: whom it plays, the
deployment it drives, the hands it holds, how it reads the forge and how
it checks an outcome. Who marks, chooses, starts and triages is
agents' `qa-flow.md`; the pass's steps and report are agents'
`roles/qa.md`. This document says only what those leave to the product,
and a pass reads it after its brief. The lab's binding and experiments were retired for this process on 10-27. What is still open is section 8.

## 1. In one page

- **A pass is one of temper's users:** an agent working with temper for
  a person. It has what any such agent has: temper's help (`temper
  help`, `temper ctl help`), `temper ctl`, and the tools it chooses for
  the systems temper works with (section 2).
- **temper is agnostic about those tools.** It serves its own part, and
  names the rest by id and link; how a user reaches the forge is the
  user's choice (the user, 10-26). Our passes choose `curl` (section 5).
- **temper says what it is doing, why, and where; the forge shows what
  is there.** A change's step, hold and landing are temper's; its diff,
  reviews, CI logs and branches are the forge's (`client/forge.md`,
  section 1).
- **The deployment is a box,** on its fake backends first, then on real
  ones a rung at a time, as the brief names (section 3).
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
  signed in by the box's printed link (`client/tools.md`, section 9).
  ctl is its tool for temper; what it uses for anything else is its
  own, as for any of temper's users.
- **temper's part, through ctl:** tasks, chats and their activity; a
  change's way to landing, its step in words, its hold, its place in a
  queue, its gates' verdicts and, landed, its merge commit, read with
  `change show TASK`; the forges and their repositories. Each comes with the ids
  and links that find it on the forge (`client/tools.md`, 5.5, area 6).
- **The forge's part, on the forge:** a pull request's diff, files and
  reviews; CI's runs, jobs and logs; branches and their heads. A part
  temper does not serve says "Shown on the forge.", with the resource
  in the forge's words (`client/tools.md`, 5.2).
- **What temper owns on the forge changes through temper:** its
  branches, its pull requests and their landing. What a person does on
  the forge (a review, an approval at a head, a comment, a push to a
  branch of their own) they do there as themselves, and temper reads it
  (`client/forge.md`, sections 4 and 6).
- **temper's own agents are another user.** A chat or a coder reads the
  forge only through its host's read tools, within its authority, since
  a worker holds no forge credential (`domain/agent.md`, 4.3). A pass
  tests that as one of temper's features, as any other.

## 3. The deployment

- **A box in the pass's scratch directory,** started as the walk
  `box` says (`temper help box`), with the flags of the brief's rung,
  and stopped by the pass before it ends.
- **The rungs:**

  | Rung | Forges | Model | Accounts held |
  |---|---|---|---|
  | fakes | `--fake-backends` (`protocol/setup.md`, 7.5): GitHub and a Forgejo elsewhere faked; Forgejo here real, from the box's cache | the fake model, by the key file the box prints | none |
  | a real model | Forgejo here, set up through ctl, as the walk `box` sets it up; the box walked from its start to the account's export | ChatGPT, a held login imported and exported back, or a key read from a private file (`client/tools.md`, 5.5, area 5); the box's agents on `gpt-6.1-sol` at effort high, its budget raised if needed (below) | the model's |
  | GitHub | GitHub, through an App kept for QA, on a repository kept for it | as the rung below | the App's key and its installation |
  | Claude | Forgejo here, as a real model's rung | Claude Code on a person's own login, a held configuration directory used in place (`protocol/credentials.md`, 3) | the model's |

- **The real model's agents run `gpt-6.1-sol` at effort high** (the
  user, 10-27), set as a person sets a box's model, before its start
  (the walk `box`'s first step): `gpt-6-luna`'s model entry renamed
  `gpt-6.1-sol` in place, in `temper.json` (the provider's list and
  `engine.llm`'s models, where it stays the first, ahead of `demo`) and
  in the agent's configuration, its window and prices kept (a unit is a
  thousand tokens on either model), its output 32,000, since high
  effort's reasoning counts against it, and its efforts declared
  `["high", "medium", "low"]`, the first being what a task with no
  choice of its own takes. Every charter takes the first configured
  model as its main model, and the stock's spends are sized from it at
  the start (`protocol/configuration.md`, 13.4): C 1,081, a run's K
  4,324, a chat 103,776, a project's period 207,552. After the start,
  `temper ctl settings show charter 1` names `gpt-6.1-sol`, and the box's
  spends are those; a box that shows otherwise is a setup to redo, not
  a finding. A chat is started with `--model gpt-6.1-sol --effort high`
  where its model is named; template 1's choice (`settings apply`,
  `{"main": {"model": "gpt-6.1-sol", "effort": "high"}}`) sets it for
  every chat of a running box.
- **Its room is raised beside it,** in `temper.json` before the start:
  `engine.limits`' overrides `"people.requests": 128` and
  `application.limits`' overrides `"root.loads.loads": 32`. The small
  profile keeps each deciding request's answer a day in 32 places, which
  one busy pass fills, after which every keyed request answers busy
  (jig's `domain/people.md`, 5.1); and its 2 load slots are too few for
  a restart with held work loaded. These are a QA box's, as the budget
  is (below); the product's own values are the user's, settled apart.
- **One configuration, checked before the first start.** Every change
  above (the model, its output and efforts, the room) is made in
  `CONFIG` after `temper init` and before the box's first start, with
  the limits a 32,000 output needs (a turn larger than init's stock
  holds: its turn, transcript and load bytes raised to the figures
  `protocol/configuration.md` names for it). Then, before the first
  start and before every restart, the pass or walk checks it, and stops
  if any line prints other than `true`, reporting a setup to redo, never
  a finding:

  ```
  jq -e '.engine.llm.models[0] | .name == "gpt-6.1-sol" and .output == 32000 and .efforts[0] == "high"' CONFIG/temper.json
  jq -e '.engine.limits.overrides["people.requests"] == 128 and .application.limits.overrides["root.loads.loads"] == 32' CONFIG/temper.json
  jq -e '.endpoints[0].models[0] | .name == "gpt-6.1-sol" and .output == 32000' CONFIG/agent.json
  ```

  A QA brief carries these three lines, and a box set up by an earlier
  `main` than this section's is set up again.
- **A budget never stops a walk** (the user, 10-27, "raise the budget if
  needed"): the stock holds a rung in arithmetic (a 100k-token turn is
  about 100 units against a run's K of 4,324), but a walk raises its
  box's anyway, as a person would, and says the figures in its report.
  Raising a period does not refill a project already made, so the walk
  raises right after its sign-in, before `forge setup` makes its
  project, in one `settings apply` edited from `settings show rules
  --json` and `settings show policy 1 --json`: the rules' spend 207,552
  and period spend 449,696; policy 1's ceiling's spend 207,552 and
  period spend 449,696, the project's period spend 449,696, role 0's
  authority spend 207,552 and its period spend 415,104. The period
  stays larger than role 0's pool, so that the deployment's own work (a
  landing task, a tier run, a queue's repair) is still funded. Nothing
  per run; the chat's template stays as allowed. A walk that meets a
  made project short funds it instead: `project fund P AMOUNT`, then
  `project fund P AMOUNT --role 0`, the project first, since the
  owner's pool reserves from its period; each refills the open period
  at once, and `project show P` reads what is left. The stock's own
  defaults are not a QA box's to change (temper#16).
- **Down before up:** the director opens a rung once the rung below has
  run clean at a `main` no older, and a rung's open findings hold the
  rungs above it (`draft/qa-live.md`, 3.2, without the lab's verdicts).
- **Real accounts are the user's,** held on the machine `qa-flow.md`
  names, used by path and never read, and given back before the box
  stops: an imported login exported to its file. Their spend is
  `qa-flow.md`'s open question (its section 9).

## 4. The hands

| Hand | Who | For | Given as |
|---|---|---|---|
| ctl | the deployment's first owner, the person | every act on temper, and temper's account of it | the box's printed sign-in link, on stdin to `ctl sign-in -` |
| the observer | the forge's person, with reads alone: on Forgejo, `read:repository`, `read:issue`, `read:organization` and `read:user` | section 5's reads | a header file the box names |
| the person on the forge | the forge's person, with a token that writes | what a person does there, as a case's step written in its plan: a review, an approval at a head, a comment, a push, a branch moved under temper | a header file the box names |

- **Never:** temper's bot or its tokens, the preparation's
  `admin_token`, temper's store or its HTTP API, a fake's internals or
  its observations. A value the box prints for a person to paste (a
  model key, a fake's admin token, a PEM) reaches temper through ctl,
  as the welcome's paste does.
- **What the box prints** today: its sign-in link, and on fake backends
  the fakes' addresses and the files a person pastes (the walk `box`,
  on fake backends). For the forge's two hands it
  is to print, for each forge it sets up or fakes, the forge's API and
  the person (section 7):

  ```text
  HH:MM:SSZ Forge 3 (Forgejo): api http://ADDR/api/v1
  HH:MM:SSZ Forge 3 (Forgejo): person LOGIN, read header FILE, write header FILE
  ```

  Each header file holds one line, `Authorization: token TOKEN`, private
  to the box's user, so that a pass passes a token by its path and never
  reads it.
- **Until it does,** on the box's Forgejo here, the pass signs in on
  Forgejo's web as the person with `person_password`, changes it as
  Forgejo asks (`protocol/setup.md`, 6.5), and makes the two tokens as a
  person makes them; on the fake Forgejo elsewhere, the admin token file
  the box prints serves both. It reports the minutes under Automation.

## 5. Reading the forge

Our passes read Forgejo with `curl`, the observer's header by its path:

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
  reads Forgejo 17. A read the fake refuses is a finding against the
  fake, routed to skein's owner.
- **The person's writes** are Forgejo's API with the write header, as
  the case's plan names them; a pass writes only what a case needs.
- **GitHub's reads,** on its rung: GitHub's REST API by `curl` at
  `https://api.github.com`, with the person's header file, a
  fine-grained token kept for QA on the repository kept for it, by its
  path:

  | What | Read |
  |---|---|
  | a change's pull request; its reviews; its files | `GET /repos/O/R/pulls/N`; `.../pulls/N/reviews`; `.../pulls/N/files` |
  | CI at a head: check runs; commit statuses | `GET /repos/O/R/commits/SHA/check-runs`; `.../commits/SHA/status` |
  | the Actions runs at a head; a run's jobs | `GET /repos/O/R/actions/runs?head_sha=SHA`; `.../actions/runs/ID/jobs` |
  | a job's log | `GET /repos/O/R/actions/jobs/ID/logs`, a redirect `curl -L` follows |
  | a branch and its head; the branches under a prefix | `GET /repos/O/R/branches/B`; `GET /repos/O/R/git/matching-refs/heads/PREFIX` |
  | a file at a commit | `GET /repos/O/R/contents/PATH?ref=SHA`, with `Accept: application/vnd.github.raw` |

  The box prints no forge hands for GitHub, which has no person of the
  box's: the person's header file is the brief's. Its 5,000 requests an
  hour are the person's own, apart from temper's installation's, so a
  pass reads at a person's pace, a wait on temper's side between reads,
  never a loop of reads.

## 6. Checking an outcome

- **Each case, one loop:** act through ctl and wait on it
  (`client/tools.md`, 5.4); read temper's account; follow its ids and
  links to the forge; compare.
- **A case's line says both,** what temper said and what the forge
  showed, in agents' `roles/qa.md`, section 3's shape.
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
- **Not a finding:** that the pass read the forge, or how.
- **Known gaps:** what the design has and `STATUS.md` lists as not yet
  built goes under Blocked, once, with what the pass did instead.

## 7. What it asks of temper

- **The box's forge hands** (section 4): for each forge it sets up or
  fakes, its API's address, the person's login, and a read and a write
  token as header files, printed with its other lines
  (`protocol/setup.md`, 7.3).
- **ctl's help** in these terms: the forge read as the agent chooses,
  and what temper owns there changed only through temper
  (`client/tools.md`, 11.5).

## 8. Open questions

- **The GitHub rung's App:** made by the user for QA, or from temper's
  manifest. An installation that exists already is accepted with no
  browser, by `forge accept FORGE INSTALLATION` (`client/tools.md`, 5.5,
  area 6).
- **Real accounts' holding and spend:** `qa-flow.md`, section 9.
- **Holding section 5 to the fake:** a step test that decodes each read
  with skein-forge's Forgejo requests, as the help's ctl lines are held
  to ctl's parser (`protocol/documentation.md`, section 1). Default:
  none, until a pass meets a read the fake refuses.
- **The browser:** a pass drives ctl; driving temper's pages waits in
  `draft/qa-web.md`, whose lab process is retired.
