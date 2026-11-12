# temper ctl role

Kind: reference

Standing roles: start, list, read their Now or report, and talk.

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
role show TASK [--report]
  TASK (task): a number; required. The role's task number
  --report: a switch. Read its weekly report instead of its Now
Read a standing role's task document and its Now or weekly report whole
Prints: The task document, its open talk, then its whole Now or report with note number, revision and time, or `no Now yet` / `no report yet`
--json: the record role.
Key: none.
The Now or report is the lowest-numbered note described now or
report in the role's own task scope. A supplied cut body is marked.
```

## role talk

```
role talk TASK TEXT [--timeout DURATION] --key KEY
  TASK (task): a number; required. The role's task number
  TEXT (text): text, at most 1048576 bytes, or - to read stdin; required. Words for the role
  --timeout DURATION: a duration: seconds, or a number with s, m or h, at most 3600 seconds; optional. How long to wait; 300 seconds when absent, at most 3600
role talk TASK --leave --key KEY
  TASK (task): a number; required. The role's task number
  --leave: a switch. End your talk and its session
Talk with a standing role, or leave your talk
Prints: The role's reply, or `left: task N`; refusals exit 2; a timeout, hold or end before its reply exits 3 with what came
--json: the record role_reply.
Key: --key KEY, required.
Your first words open a talk; later words continue it. Waits as
task show TASK --until replied, then reads the role's turns after
these words. Refusals: not a standing role; talking with PARTY;
no talk to leave. TEXT and --leave cannot be used together.
```
