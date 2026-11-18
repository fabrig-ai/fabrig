# temper ctl change

Kind: reference

Changes: open one or read its way to landing.

## change show

```
change show TASK [--project PROJECT] [--since DIGEST] [--until CONDITION] [--timeout DURATION]
  TASK (task): a number from 1 to 18446744073709551615; required. The change task's number
  --project PROJECT: a project's name or number; optional. Its project; the task's when absent
  --since DIGEST: a digest, as a read prints it; optional. Wait until it differs from the read that printed DIGEST
  --until CONDITION: one of pull, ci:pending, ci:passed, ci:failed, held, landed, changed or cleared; optional. Wait until it holds: pull, ci:pending, ci:passed, ci:failed, held, landed, cleared, or changed
  --timeout DURATION: a duration: seconds, or a number with s, m or h, at most 3600 seconds; optional. How long a wait lasts; 300 seconds when absent, at most 3600
Read a change's way to landing: its pull request, CI and step
Prints: `change N`, its repository, branch and landing branch, its pull request, step and CI at its head; for an opened change, its source branch, the commit named or `its head`, and the head taking kept; its questions: the route, those taken of its bound, and each open question with where it stands; and a digest
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

## change open

```
change open --project PROJECT [--repository OWNER/NAME] --branch BRANCH [--at COMMIT] [--into BRANCH] --title TEXT [--body TEXT] (--budget AMOUNT | --as-allowed) --key KEY
  --project PROJECT: a project's name or number; required. The project
  --repository OWNER/NAME: text, at most 1048576 bytes; optional. Its repository; the project's only repository when absent
  --branch BRANCH: text, at most 1048576 bytes; optional. Required: the source branch, 1 to 1024 bytes
  --at COMMIT: text, at most 1048576 bytes; optional. 40 or 64 hexadecimal digits; its head when taking the change when absent
  --into BRANCH: text, at most 1048576 bytes; optional. The landing branch, 1 to 1024 bytes; the repository's default when absent
  --title TEXT: text, at most 1048576 bytes, or - to read stdin; optional. Required: the pull request's title, 1 to 256 bytes
  --body TEXT: text, at most 1048576 bytes, or - to read stdin; optional. The pull request's body after the opening line, 1 to 16384 bytes
  --budget AMOUNT: an amount with at most two decimals, as 130 or 1.25; optional. What reviews, resolutions and repairs may spend; or --as-allowed
  --as-allowed: a switch. Take what your role allows instead of a named budget
Open a change from a branch you pushed
Prints: `task N`, the change task made; a refusal names why nothing was made
--json: the record change_open.
Key: --key KEY, required.
Give --branch, --title, and either --budget or --as-allowed.
--repository is OWNER/NAME. --at is sent lowercase.
A replay under the same key answers the first. A change that fails later
was opened: read it with change show TASK.
```
