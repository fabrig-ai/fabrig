# Projects

Kind: concept

A project is where work happens: the repositories adopted into it, one of them its home, the parties with a role in it, its own policy, and the period its budget comes from.

## What it is

- **Its number or its name** names it in ctl's calls: `1` or `Project
  1`. `me` lists your projects, each `N · NAME · ROLE`.
- **Its repositories,** adopted from the deployment's forges, each
  owned, a fork or context (temper help repository). A project may
  adopt several, on several forges.
- **Its home** is one of them, chosen at its adoption: the repository
  where temper keeps the project's goals, one issue each (temper help
  goal). A project has at most one home, and a context repository is
  never one. Every task of the project may read its home, whether or
  not it may write there.
- **Its members** are the parties with a role in it: owner,
  maintainer, member, observer, or a numbered role its policy defines
  (temper help party).
- **Its policy** is its own: what its work may do and spend, and what
  each role may start (temper help policy).
- **Its period** holds what its work may spend; each member's pool is
  drawn from it (temper help budget).

## How it changes

- **Made:** `temper init` makes none. Setting up the box's Forgejo,
  or connecting a Forgejo with a first repository, makes a new
  project, named after the repository, unless you name one that
  exists; the playground's start makes `Demo`. In the client, the
  welcome's Ready makes `Default`, and the banner's project switcher
  makes one by the name you give it. A new project's policy is a copy
  of policy 1 as it is then.
- **Renamed:** its owners rename it in place from the banner's project
  switcher; the new name shows wherever the project's does.
- **Given repositories:** `repository adopt` adopts one into a project
  that exists; the first one adopted becomes its home unless the
  adoption says otherwise.
- **Made public or private:** an owner uses `project amend PROJECT
  --visibility public --key KEY` to open its public pages, and
  `--visibility private` to make it private again. Anyone may read a
  public project's name and its tasks' structure and titles without
  signing in. The rest of their specifications and their words stay
  private.
- **Deleted:** its owners delete it with Delete on its settings in the
  client, confirmed by its name; the playground's reset deletes
  `Demo`. It takes no new work at once and leaves every list;
  its live tasks are cancelled, then erased, its own settings and roles
  with them, and its repositories leave it. Its name is free again; its
  number is never given again.

## What temper guarantees

- **One home:** a second home is refused, and the refusal prints the
  adoption to make instead.
- **Its policy is its own:** a change of policy 1 reaches the projects
  made after it alone.
- **A deletion ends whole:** one a restart cut goes on at the next
  start, and no start serves a half-deleted project.
- **A goal shows in the home:** each tracked task of the project is an
  issue there, updated as the task changes (temper help goal).

## Acting on it

- **ctl:** `me` lists your projects; `forge setup` and `forge connect`
  can make one with its first repository (temper help forge);
  `repository adopt` adds a repository; `task list`, `inbox` and
  `model list` read one project's, and `chat start` starts a chat in
  one; `project show` reads a project: whether it is public and its
  public page's path, its homes, its roles with their
  parties, its spends, its open period and each party's pool in it;
  `project fund` raises its period spend or a role's;
  `project amend PROJECT --visibility public|private --key KEY`
  changes its visibility. The public page is a path under the
  deployment's address.
- **The client:** Delete on a project's settings deletes it, for its
  owners.
- **temper's runs** work in their task's project: `change` writes to
  one of its repositories, and the forge's reads name one by
  `owner/name`.

## Limits

- Only an owner changes visibility; a member and project 0 are
  refused. A public project's pages show its name and task titles,
  never the rest of a specification, instructions, inputs, results,
  conversations, or people's names.
- ctl's group `project` reads a project and changes its spends and visibility:
  no call renames a project or changes its roles.
- temper does not make a project from ctl but with its first
  repository, or the playground's.
- ctl has no call that deletes a project.
- A deleted project's spend stays in the period's totals.
- temper has no call that moves a task or a repository to another
  project.
- A project's period lasts a week: each week the next opens at its
  period spend then, and `project fund` raises it at once (temper help
  weekly-budget).
