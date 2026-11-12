# temper ctl follow

Kind: reference

Follows: read a project's follows and their lag and parity.

## follow list

```
follow list --project PROJECT
  --project PROJECT: a project's name or number; required. The project
List a project's follows, their lag and parity
Prints: One line per follow: source and target branches, lag and parity; `no follows` when empty
--json: the record follows.
Key: none.
`follow show --project PROJECT FOLLOW` reads one whole.
```

## follow show

```
follow show --project PROJECT FOLLOW
  --project PROJECT: a project's name or number; required. The project
  FOLLOW (follow): a number from 1 to 4294967295; required. The follow's number
Read one follow whole
Prints: The setting, task, lag oldest first, skips and parity
--json: the record follow.
Key: none.
The lag and parity are the forge's retained facts.
```

## follow start

```
follow start --project PROJECT FOLLOW COMMIT --key KEY
  --project PROJECT: a project's name or number; required. The project
  FOLLOW (follow): a number from 1 to 4294967295; required. The follow's number
  COMMIT (after): text, at most 1048576 bytes; required. The landing's full lowercase commit id
Start a follow entry or lift its owner's hold
Prints: `kept`, or the entry and why the act was refused
--json: the record follow-act.
Key: --key KEY, required.
Gives the owner's word, lifts an owner's hold, or retries an ended entry.
```

## follow hold

```
follow hold --project PROJECT FOLLOW COMMIT --reason TEXT --key KEY
  --project PROJECT: a project's name or number; required. The project
  FOLLOW (follow): a number from 1 to 4294967295; required. The follow's number
  COMMIT (after): text, at most 1048576 bytes; required. The landing's full lowercase commit id
  --reason TEXT: text, at most 1048576 bytes, or - to read stdin; required. Why this entry waits or is skipped
Keep a follow entry from producing
Prints: `kept`, or the entry and why the act was refused
--json: the record follow-act.
Key: --key KEY, required.
The change must not have started producing; follow start lifts the hold.
```

## follow skip

```
follow skip --project PROJECT FOLLOW COMMIT --reason TEXT --key KEY
  --project PROJECT: a project's name or number; required. The project
  FOLLOW (follow): a number from 1 to 4294967295; required. The follow's number
  COMMIT (after): text, at most 1048576 bytes; required. The landing's full lowercase commit id
  --reason TEXT: text, at most 1048576 bytes, or - to read stdin; required. Why this entry waits or is skipped
Count a landing followed without its change
Prints: `kept`, or the entry and why the act was refused
--json: the record follow-act.
Key: --key KEY, required.
Cancels its change, keeps the reason among skips, and keeps the chain whole.
```

## follow merge

```
follow merge --project PROJECT FOLLOW COMMIT [COMMIT]... --key KEY
  --project PROJECT: a project's name or number; required. The project
  FOLLOW (follow): a number from 1 to 4294967295; required. The follow's number
  COMMIT (after): text, at most 1048576 bytes; required. The landing's full lowercase commit id
  COMMIT (afters): text, at most 1048576 bytes; repeated, at most 31. The next landings, oldest first
Make one change follow consecutive landings
Prints: `kept`, or the entry and why the act was refused
--json: the record follow-act.
Key: --key KEY, required.
Names two or more consecutive entries whose changes have not started producing.
```
