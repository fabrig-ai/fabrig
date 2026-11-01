# Progress: the box designer (part `box-groups`)

final: tasks/box-pipe.md

## Parts

- `box-groups`: the box's groups by kind from group files, `remove`/`removed`,
  the machine's members, `temper init` naming no forge, and the box's process
  stories (draft 3 "The box knows kinds", 4 "Its box group stopped and
  removed", 6.4; contracts, 3 and 6).

## Critical path

(being written)

## Decisions

- The box runs a forge it was not started with in one of its **kinds' groups**:
  `Settings.vacant`, each kind's group built by the binary at the box's start
  numbered `u16::MAX` (an owned Forgejo's, the playground's), placed and sized
  as an asked group, filled at `start N` with `Group::numbered(N)`. Reason: the
  programming model has no callbacks, so the box cannot call
  temper-forge-setup's or temper-fake's builders at runtime, and its io,
  terminal and log bounds are sized at construction (setup.md 7.3, "The
  binary hands it the steps").
- A kind is told by `Group.recorded`: the playground's group is the one kind
  not recorded. The box reads only the file's `playground` member; the rest is
  the steps' own (`temper forge prepare DIRS N` reads entry N).
- One forge of a kind at a time: `start N` with the kind's group taken answers
  `failed N busy` (the machine's one port per kind).
- `remove N` answers, besides `removed N`, as `stop` is refused: `failed N
  busy`, `failed N stopping`, and `failed N files not removed` on a file error.
- A removed group that is not a kind's group (the document's, or one the binary
  read from its file at the box's start) is gone for the box's life.

## Contract problems

- Contracts 3 gives `remove`'s one answer, `removed FORGE`; the box also answers
  `failed FORGE busy|stopping|files not removed` (as `stop` and `start` are
  refused under 7.1's one-ask rule). Proposed: a kept `BoxRemove` answered
  `failed` stays kept and is sent again (D4's rule for an unanswered act);
  serve's box link reads `removed` as the answer and `failed` as not yet.
