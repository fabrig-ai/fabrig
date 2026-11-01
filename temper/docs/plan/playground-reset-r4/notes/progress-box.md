# Progress: the box designer (part `box-groups`)

final: tasks/box-pipe.md
final: tasks/bx-machine.md

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

- `bx-machine` reads and types the machine members but leaves `temper init`'s
  template as it is; the template naming no forge, with `owned`, `demo.fake` and
  the new grants, is `bx-init`. Reason: the grants `cancel` and
  `remove_project` are unknown until `jig-acts`; a template with no forge needs
  `SetUp` and the playground's start making records first (else a first run has
  no forge and no playground), and `application.forges` refuses an empty list
  today; engine worlds read the template (`temper/tests/engine/domain/tests/fixtures/stock.rs`).
  A task that needs a first run's document to name no forge names `bx-init`.
- `application.demo.forge` stays required beside the new optional `fake` until
  the playground part's task that removes the demo; the machine members'
  `listen` may equal a seeded setup entry's of its own kind (D1).

## Contract problems

- Contracts 3 gives `remove`'s one answer, `removed FORGE`; the box also answers
  `failed FORGE busy|stopping|files not removed` (as `stop` and `start` are
  refused under 7.1's one-ask rule). Proposed: a kept `BoxRemove` answered
  `failed` stays kept and is sent again (D4's rule for an unanswered act);
  serve's box link reads `removed` as the answer and `failed` as not yet.
