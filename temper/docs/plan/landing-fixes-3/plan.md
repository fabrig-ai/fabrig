# landing-fixes-3

What the first real landing through temper met (`inputs/request.md`). Three ranked items, at most three tasks.

## Parts and tasks

| Request's item | Task | State |
|---|---|---|
| 1. A spec's report can't fit `finish` | not yet released | being cut |
| 2. A receipt pads the head | `head-words` | released |
| 3. A compare says too large on a small change | `compare-words` | released |

Item 1 ranks first but goes out last, because it needs the most reading. Items 2 and 3 need nothing from it.

## What the cut decided

- **Item 2:** the receipt's cause holds. The survey also found `CI failed at head` and `CI passed at head` padding a head, and `head-words` fixes those too. Every other printer is width-aware or prints a short prefix.
- **Item 3:** the refusal changes and the bounds don't. The request's cause is half of it: `forge.client.rows` is 8, and a compare also holds both whole sides of every changed file within `forge.client.answer_bytes`, so one line changed in a long file is refused too. The refusal already named `read_pull_files`, but it never said the ids were read and it pointed at a host bound that wouldn't help.

## Overlaps

`head-words` touches `temper-engine-domain/src/route/forge_route.rs`, which `landing-fixes-2/repair-cause` and `update-recovers/update-hold-words` are also changing. The edit is confined to the two cancel reasons.

## Critical path

None yet: the released tasks need nothing and nothing needs them.
