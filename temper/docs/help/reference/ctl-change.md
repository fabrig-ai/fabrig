# temper ctl change

Kind: reference

Changes: read a change's way to landing.

## change show

```
change show TASK [--project PROJECT] [--since DIGEST] [--until CONDITION] [--timeout DURATION]
  TASK (task): a number from 1 to 18446744073709551615; required. The change task's number
  --project PROJECT: a project's name or number; optional. Its project; the task's when absent
  --since DIGEST: a digest, as a read prints it; optional. Wait until it differs from the read that printed DIGEST
  --until CONDITION: one of pull, ci:pending, ci:passed, ci:failed, held, landed, changed or cleared; optional. Wait until it holds: pull, ci:pending, ci:passed, ci:failed, held, landed, cleared, or changed
  --timeout DURATION: a duration: seconds, or a number with s, m or h, at most 3600 seconds; optional. How long a wait lasts; 300 seconds when absent, at most 3600
Read a change's way to landing: its pull request, CI and step
Prints: `change N`, its repository, branch and landing branch, its pull request, step and CI at its head; its questions: the route, those taken of its bound, and each open question with where it stands; and a digest
--json: the record change.
Key: none.
Outstanding landing-rule approval lists each requirement, counted people,
review heads, remaining count and unmapped reviews, then the pull for review.
Unread/incomplete evidence differs from zero qualifying approvals.
Next: `temper ctl DIR change show TASK --until landed`; held, `temper ctl DIR
task show N`; verifying/red, `temper ctl DIR change show TASK --until cleared`;
landed, its merge. No-change completion prints `result: completed, no change`
and that task-show command for its report.
Unmet waits: `not met: ended`, `not met: ended: cancelled` or
`not met: ended: failed` (exit 3); landing waits at a hold: `not met: held` (3).
Gone record: last 8 history events and the all-events read (0), up to 16 pages,
naming more. Empty history: record and history gone (2). JSON: events/read
or history: gone. A wait without a panel: `no change panel for task N` (2).
Read the forge for its diff, reviews and logs.
```
