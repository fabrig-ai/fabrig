# temper ctl project

Kind: reference

Projects: make or delete one, read periods and pools, set spend and visibility.

## project show

```
project show PROJECT
  PROJECT (project): a project's name or number; required. The project
Read a project, its public page, roles and spends, its open period and each party's pool
Prints: The project, whether it is public and its public page's path, its homes and spend per period, each role, its open period and each pool
--json: the record project.
Key: none.
Amounts are in the deployment's unit. Spent includes what ended tasks spent. A missing next start or open period says so. A deleted project prints project N was deleted on stderr, exit 2; an unknown one prints unknown.
```

## project fund

```
project fund PROJECT AMOUNT [--role ROLE] --key KEY
  PROJECT (project): a project's name or number; required. The project
  AMOUNT (amount): a number from 0 to 18446744073709551615; required. The spend per period, in the deployment's unit
  --role ROLE: a number from 0 to 4294967295; optional. Set this role's spend per period
Set a project's spend per period, or a role's, resizing the open period at once
Prints: The spend edit as answered; then now: the open period's budget and what is left, with --role its pools too
--json: the record project_fund.
Key: --key KEY, required.
An owner's chats are funded from their pool, and pools reserve from the project's period. Raise both with two calls: project fund PROJECT AMOUNT first, then project fund PROJECT AMOUNT --role ROLE. A role's raise past the period's room is refused, exit 2, naming what the period has left. A cut keeps each budget at what it has used. A key replay prints the original edit, then today's balances marked now:. JSON has answer (project, optional role, period_spend) and now (the project read).
```

## project amend

```
project amend PROJECT --visibility public|private --key KEY
  PROJECT (project): a project's name or number; required. The project
  --visibility public|private: one of public or private; required. Public reading, or private again
Make a project readable by anyone without signing in, or private again
Prints: `project 1: public` and, made public, `page: /public/projects/3f2a9c01d4e5b6a7`, then what may show on stderr; made private, `project 1: private`
--json: the record project_amend.
Key: --key KEY, required.
Anything said in this project's chats may show on its public pages, in the titles of its goals and tasks. On a public repository, a change's pull request shows what its agent wrote too. Nothing asks first; made public, it prints these words on stderr. Only an owner changes visibility. A member and project 0 are refused, exit 2. The public page is a path under the deployment's address, from me's public identifier. A key replay prints the first answer.
```

## project make

```
project make NAME --key KEY
  NAME (name): text, at most 1048576 bytes; required. The new project's name
Make a project from policy 1
Prints: The new project's number
--json: the record make-project.
Key: --key KEY, required.
Copies policy 1 as it is now. A key replay returns the first project's number. Add a repository with repository make --project PROJECT --forge FORGE ORG/NAME --until adopted --key KEY.
```

## project delete

```
project delete PROJECT --key KEY
  PROJECT (project): a project's name or number; required. The project
Delete your own project, closing it at once and deleting it whole after
Prints: `deleted: project 3, fabrig`, the answer's number and the name me gave at the call; the number alone with no name
--json: the record project_removed.
Key: --key KEY, required.
Only an owner deletes a project, a fresh store's default project among them. Live tasks refuse it in jig's words, naming the live roots: end them first, then use a new key. No confirmation. A number is sent as given, in me or not; a name is resolved through me, and one not there sends nothing, exit 2, unknown project. The ask has no grace and uses no chat template. A member who is not its owner, an unknown project, or one already being deleted under another key is refused, exit 2. A key replay prints the same deletion after it ends as while it lasts, within the engine's retention. A restored request prints kept: remove-project, deleted: project N, with no name. project show PROJECT then says project N was deleted, exit 2.
```
