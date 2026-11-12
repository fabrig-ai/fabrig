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
- **Its ways,** in `project-ways`, govern drafts in chats, Work's Goals and
  Changes filters and the way a goal starts by. The filters'
  switches hide no rail entry; their work stays under Work's All filter.
  Its `project-questions` setting gives a change's question route when its
  maker names none (temper help settings). Its `project-words` gives How work
  runs its headline and lede when the project runs a way of its own;
  Ask temper writes them with its proposals.
- **Its period** holds what its work may spend; each member's pool is
  drawn from it (temper help budget).

A project's `project-classes` setting says when agents decide, then tell you;
ask you; or ask you even when away. Four kinds always wait: money,
irreversible acts, rules and other. The project's way sets its classes;
you can change them as any setting (temper help settings).

## How it changes

- **Made:** `temper-box init` makes none. `project make NAME --key KEY` makes
  one from policy 1 and prints its number. Setting up the box's Forgejo,
  or connecting a Forgejo with a first repository, makes a new
  project, named after the repository, unless you name one that
  exists; the playground's start makes `Demo`. In the client, the
  welcome's Ready makes a project named for the first repository, or
  `My project` with none, and the rail's project switcher
  makes one by the name you give it. A new project's policy is a copy
  of policy 1 as it is then.
- **Renamed:** its owners rename it in place from the rail's project
  switcher; the new name shows wherever the project's does.
- **Given repositories:** `repository adopt` adopts one into a project
  that exists; the first one adopted becomes its home unless the
  adoption says otherwise.
- **Made public or private:** an owner uses `project amend PROJECT
  --visibility public --key KEY` to open its public pages, and
  `--visibility private` to make it private again. Anyone may read a
  public project's name and its tasks' structure and titles without
  signing in. The rest of their specifications and their words stay
  private. Anything said in this project's chats may show on its public pages,
  in the titles of its goals and tasks. On a public repository, a change's
  pull request shows what its agent wrote too.
  Opening visibility also needs anonymous admission configured;
  temper help open-public-reading walks every figure and the restart.
- **Deleted:** an owner uses `project delete PROJECT --key KEY`, with
  no confirmation, or Delete on How work runs in the client, confirmed
  by its name; the playground's reset deletes `Demo`. ctl can delete a
  fresh store's default project too. An owner's deletion is refused while
  it has live tasks, naming their live roots in the engine's words:
  `project delete: This project has live tasks; end them first: 12, 15`.
  Close its chats or cancel its roots, wait for them to end, then use a new
  key: the refused key keeps its refusal. Deletion takes no new work at
  once and leaves every list. Its tasks are erased, its settings and roles
  with them, and its repositories leave it. Its name is free again; its
  number is never given again. ctl prints `deleted: project 3, fabrig`,
  the answer's number and the name `me` gave at the call, or the number alone
  with no name. A number is sent even when absent from `me`; an unknown
  name sends nothing, exit 2, `unknown project`. Asking again with the same
  number and key gets the first answer after deletion as while it lasts,
  within the engine's retention. A restored request prints
  `kept: remove-project, deleted: project 3`. `project show 3` then prints
  `project 3 was deleted` on stderr, exit 2; an unknown project says `unknown`.

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

- **ctl:** `me` lists your projects; `project make NAME --key KEY` makes one; `forge setup` and `forge connect`
  can make one with its first repository (temper help forge);
  `repository adopt` adds a repository; `task list`, `inbox` and
  `model list` read one project's, and `chat start` starts a chat in
  one; `project show` reads a project: whether it is public and its
  public page's path, its homes, its roles with their
  parties, its spends, its open period and each party's pool in it;
  `project fund` raises its period spend or a role's;
  `project amend PROJECT --visibility public|private --key KEY`
  changes its visibility. The public page is a path under the
  deployment's address. `project delete PROJECT --key KEY` deletes it.
- **The client:** How work runs on the rail opens the project's process;
  on a phone, choose How work runs for PROJECT in the project switcher.
  Delete there deletes the project, for its owners. The switcher's
  Deployment opens Accounts, the first Deployment tab.
  Team on the rail shows the project’s people and what waits on each,
  from two people; on a phone, use Team for PROJECT in the switcher.
- **temper's runs** work in their task's project: `change` writes to
  one of its repositories, and the forge's reads name one by
  `owner/name`.

## Limits

For the configuration members that need a restart, read `temper help configuration`.

- Only an owner changes visibility; a member and project 0 are
  refused. A public project's pages show its name and task titles,
  never the rest of a specification, instructions, inputs, results,
  conversations, or people's names. A title can hold what was said in a
  chat; see **Made public or private** under **How it changes**.
- ctl's group `project` makes or removes a project, reads it and changes its spends and visibility:
  no call renames a project or changes its roles.
- `forge setup` sets up the one owned Forgejo with its first repository.
  For a second project, read `forge list`, use `project make NAME --key KEY`,
  then `repository make --project PROJECT --forge FORGE ORG/NAME --until adopted --key KEY`.
- Removal needs ownership: a member who is not its owner, or a project
  already being deleted, is refused (exit 2). A project not in your `me`
  sends nothing and exits 2. Reusing the key answers the same removal
  while deletion lasts.
- A deleted project's spend stays in the period's totals.
- temper has no call that moves a task or a repository to another
  project.
- A project's period lasts a week: each week the next opens at its
  period spend then, and `project fund` raises it at once (temper help
  weekly-budget).
