# LLM accounts: uses and account selection

## 1. Uses: whose work an account pays for

### 1.1 The payer

A run's task has one funder, and the chain of funders ends at a party's
pool or a project's period (authority-budgets.md, section 1). That end is the
run's **payer**: that pool's party, or that period's project. A recurring
allotment pays as its project. Financial links decide it, independently of
the requester and delegation tree; a Move or refunding changes it for every
descendant funded through the changed task. Prospective checks follow
staged financial links before live ones, within the tasks child's bounds,
and refuse unknown or cyclic task chains without writing anything.

### 1.2 Uses

- **A plan's account, by default,** pays for its holder's own work and
  for the automatic work of each project in which its holder is the only
  party with a role. A person alone on a deployment is the only party of
  every project, so everything runs on their subscription, with nothing
  to set.
- **A plan's account's uses may name more,** in its holder's word: a
  service, as an agent of theirs that connects with its own identity
  does (people/README.md, section 3), whose work then runs on its holder's
  subscription; or less than the default. A person other than its
  holder, and a project with another party, are refused in a plan's
  account's uses, naming the LLM provider's rule (providers.md, 2.6 and
  3.4).
- **A key's account's uses** are its holder's own work by default, a
  person's; the deployment's key pays for every payer. A key's holder
  may name any party or project in its uses: a key is the customer's to
  provision for its own authorized users, billed to the key's owner
  (providers.md, 3.1). A key that should pay for more than a plan's
  account may is an account of its own.
- **A project in a key's uses** pays for all the work inside that
  project: the project's own automatic work, and every party's work
  there, whoever the payer. It pays for nothing outside it, the same
  party's work in another project included. The payer stays the payer:
  spend and budgets are counted to it as for any account (authority-budgets.md,
  section 1). A key whose uses name one project alone thus serves that
  project and no other. A plan account's uses are unchanged: a project
  named there covers only the project's own work, since a plan may not
  name a project with another party (above). Tested by
  `a_key_naming_a_project_pays_for_a_persons_work_in_it_and_nowhere_else`
  and `a_plan_naming_a_project_still_does_not_cover_another_persons_work_in_it`
  (`jig/crates/jig-domain-core/src/tests/runs/coverage.rs`).
- **A shared project's automatic work** is so paid by a key: the
  deployment's, or a person's whose uses name the project. No plan pays
  for it (above).
- **A person's automatic work in a project** (authority.md, 6.2), funded
  from their pool there, is not the project's: its payer is the person,
  so it is their own work, and a plan of theirs pays for it as for their
  chats, whoever else the project holds. temper's brief is such work,
  written for each person who looks (temper's
  `domain/forge/attention.md`, 3). Tested by
  `a_pool_seeds_work_is_its_persons_and_their_plan_pays_in_a_shared_project`
  (`jig/crates/jig-domain-core/src/tests/runs/coverage.rs`).
- **An agent that brings its own LLM** pays for its own completions;
  jig's accounts are not involved.

### 1.3 Choosing the account

Selection uses runtime accounts and their kept source identities (`accounts.md`, 3.1).
Configuration supplies provider membership and model choices; it gives no
account a preferred number or an endpoint default.

- **A task given no LLM choice takes the first model it can use.** Each
  check of its run (tasks/README.md, section 3) takes, among its charter's
  models, main then alternatives, within its authority:
  1. the first of the LLM provider of the account it keeps, while that
     account still serves its payer (or, for a key, its project), usable
     or not: **a task keeps what it's using,** so an account added later
     moves nothing, and a limited one is waited for, below;
  2. else the first whose LLM provider has an account usable now for its
     payer that allows the model (`accounts-lists.md`, 1), its source usable on a connected host
     where the host rule allows it (`accounts-sources.md`, 1), the readiness the accounts document
     shows (`../client/domain/contract/documents.md`, 1);
  3. else the first whose LLM provider has an account serving its payer,
     so that its wait names that account;
  4. else the charter's first, its run waiting with no account to name,
     and its document naming no model (`../client/domain/contract/documents.md`, 1).

  The others stay its sub-agents' models, in their order. Nobody chooses:
  a deployment with one provider's account runs that provider's model. A
  task given a choice is never moved: it waits for what it was given,
  saying why.
- **With the run's check.** As the core checks that a task's run may
  start (authority-checks.md, 1.3; engine/executors.md, 1, step 1), it takes, for its
  main model's LLM provider, the accounts whose uses cover the payer,
  or, for a key, the task's project (1.2):
  the account its task keeps, or its creator named, if it has one; else
  the payer's own, in their order, then those that name it, then the
  deployment's. It takes the first with a source usable (`accounts-sources.md`, 1) on a host
  with a free slot of a kind the charter allows, and that the host rule
  allows: a host that keeps a person's program logins takes only runs
  that person's accounts pay for (`accounts-sources.md`, 1). The claim's commit records the
  account and the source (engine/executors.md, 1, step 3), and the run is placed
  there.
- **Account, source, host and endpoint, together.** The endpoint a run
  takes is its chosen source's (`accounts-sources.md`, 1), never its model's stored one. The
  run's check, the readiness the accounts document shows, the offer,
  `Core::account_llm` and a claim's check again each take a model's
  source by `accounts-sources.md`, 1's rule and check a host for that source's endpoint, its
  presence and its bounds, so that they agree. The claim's charter names
  that endpoint for each of its models, the program's for a program
  source and the provider's `direct` one for a sign-in, a key or *none*,
  and smith runs the grant it is given there (smith's
  `protocol/programs.md`, section 3). A stored model keeps its endpoint.
- **The task keeps it,** from run to run, so a conversation stays on one
  account's cache and replay. Its creator names an account when it is
  created or amended (tasks/README.md, section 3) only to want one in
  particular, within the account's uses.
- **Holding a new task to one account.** `Core::account_llm(charter,
  models, payer, project, account)` is the choice that wants one account
  in particular for a creator who names no model: the charter's first
  model, in its order within `models`, that the offer (engine/views.md, 1)
  holds with that account; else the first whose LLM provider is the
  account's and whose candidates for the payer hold it, so that its wait
  names that account; each model at its default effort, as a task given
  no choice takes it, and the account its creator's preference. `None`
  where the account serves none of them. It is an observation, as the
  offer is: a task's own checks decide, and a task made with it waits for
  that account rather than moving.
- **A change of payer chooses again.** When a commit changes a task's
  payer (a move to another requester, a widening or an acceptance
  funded by another party: tasks/changes.md; authority-budgets.md, section
  1) or its main model's LLM provider, its next run's check takes the
  kept account only if its uses still cover the payer (or, for a key,
  the task's project) and its LLM
  provider is still the main model's, and chooses again otherwise.
- **A task whose account is unusable waits** for it, saying why
  (authority-checks.md, 1.3). An account whose program login no host with room
  holds usable, none busy holding one either, waits as that account's,
  naming the source, not as a host's. A limited task never moves to another of its
  holder's accounts: it waits for its own, as its LLM provider's limit
  asks, and a choice made again takes no other account of that holder
  and LLM provider while the kept one is limited. An amendment naming
  another account of that LLM provider is refused while its account is
  limited (providers.md, 2.5 and 3.4).
- **Its sub-agents,** on the direct backend, use the run's account for
  its main model's LLM provider, and for each other LLM provider a
  sub-agent's model uses, one account taken the same way at the run's
  check, which the claim records too. A sub-agent's model whose LLM
  provider has no usable account serving the payer is left out of that
  run's models (smith's `domain/run.md`, 5.3). A run on a program
  backend has smith's sub-agents, inspect only, each a program session
  of its own on main's account, and the program's own are withheld
  (smith's `protocol/programs.md`, 5.10). A charter's models of main's
  provider share its grant, so each takes main's way, on the endpoint
  main's source takes for it; a model of another provider takes a
  sign-in, a key or *none* of its own, on its provider's `direct`
  endpoint where its own endpoint names a program, and is left out where
  that provider names no `direct`.
- **Replay stays with its account.** A provider's opaque replay
  (reasoning, signatures and item ids) is kept only for the account that
  made it: a task whose account changes resumes its conversation carried
  over as for another LLM provider (smith's `protocol/transcript.md`,
  section 4).

The task's creator preference and the account kept by a successful claim are
separate. Clearing the former retains the latter. Current payer, membership,
uses and provider facts are joined at every check; a context retained during
preparation is not authority for a later claim. Account/source waits retain the
source's typed reason and optional reset, separately from host and application
credential waits. With no reset, no polling deadline is invented.

Recurring templates check explicit hints against the project's payer at start,
regardless of their members' declared funders, and again before each occurrence.
If an account has disappeared, changed provider or ceased to serve the project,
the occurrence is created and its next run waits, naming the account and its
typed admission reason in the Task document. This exceptional wait is rechecked
from current facts; it does not silently choose a different account.

The grant number passed as smith's account is the selected **source** number.
Changing sources within one billing account therefore conservatively causes
smith's carry-over too: it preserves plain history, discards opaque replay and
compacts on the new LLM before opening its new window (smith's
`protocol/transcript.md`, section 4). Jig keeps the committed conversation and
passes the actual new grant. A separate stable billing-account replay identity
is a later decision for smith.
