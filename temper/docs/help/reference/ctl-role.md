# temper ctl role

Kind: reference

Standing roles: start one, list them, read one and its Now.

## role start

```
role start --project PROJECT --budget AMOUNT TEXT [--charter CHARTER] --key KEY
  --project PROJECT: a project's name or number; required. The project
  --budget AMOUNT: a number; required. The budget each period, in the deployment's unit
  TEXT (text): text, at most 1048576 bytes, or - to read stdin; required. What the role owns and how
  --charter CHARTER: a number from 1 to 4294967295; optional. The role's charter; 13 when absent
Start a standing role from template 7
Prints: `done: task N, role`, or `proposed: PROPOSAL, to RECIPIENT` when another party must agree
--json: the record role_create.
Key: --key KEY, required.
Template 7 supplies the standing flags, label, contract and models.
Within your bounds the role is created; beyond them it is proposed
when your role allows a proposal. It has no deadline.
```

## role list

```
role list --project PROJECT [--after CURSOR]
  --project PROJECT: a project's name or number; required. The project
  --after CURSOR: a cursor, as a read prints it; optional. The cursor a page printed, to read on
List a project's live standing roles and their Now headlines
Prints: A line per role: its number, charter, state, budget left and Now's first line, or `no Now yet`; then a continuation or `no more`
--json: the record roles.
Key: none.
Reads live tasks labelled 3 and the note described now in each
role's own scope. The lowest-numbered matching note is its Now.
```

## role show

```
role show TASK
  TASK (task): a number; required. The role's task number
Read a standing role's task document and its Now whole
Prints: The task document, then its whole Now with note number, revision and time, or `no Now yet`
--json: the record role.
Key: none.
The Now is the lowest-numbered note described now in the role's
own task scope. A supplied cut body is marked, never silently lost.
```
