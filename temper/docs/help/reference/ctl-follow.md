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
