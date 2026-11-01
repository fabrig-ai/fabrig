# Progress: the box designer (part `box-groups`)

final: tasks/box-pipe.md
final: tasks/bx-machine.md
final: tasks/bx-sweep.md
final: tasks/bx-kinds.md
final: tasks/bx-wire.md
final: tasks/bx-commands.md
final: tasks/bx-script-forge.md
final: tasks/bx-init.md
final: tasks/bx-stories.md
final: tasks/bx-story-github.md

## Parts

- `box-groups`: the box's groups by kind from group files, `remove`/`removed`,
  the forge's directory removed and the cache kept, the machine's members,
  `temper init` naming no forge, and the box's process stories (draft 3 "The
  box knows kinds", 4 "Its box group stopped and removed", 6.4, 8; contracts,
  3 and 6).

## Tasks and needs

| Task | Needs | Minutes |
|---|---|---|
| `box-pipe` | none | 60 |
| `bx-machine` | none | 50 |
| `bx-sweep` | box-pipe | 60 |
| `bx-kinds` | box-pipe, bx-machine | 50 |
| `bx-wire` | bx-kinds, bx-sweep | 50 |
| `bx-commands` | bx-kinds | 40 |
| `bx-script-forge` | bx-commands | 40 |
| `bx-init` | bx-wire, bx-commands, jig-acts, fr-make-forge, fr-setup, pg-start | 60 |
| `bx-stories` | bx-wire, bx-commands, bx-script-forge, bx-init, rm-remove-forge, fr-make-forge, pd-delete, pg-reset | 60 |
| `bx-story-github` | bx-init, fr-make-forge, forge-asks | 45 |

## Critical path

Within the part: `box-pipe` (60) -> `bx-sweep` (60) -> `bx-wire` (50) ->
`bx-init` (60) -> `bx-stories` (60): 290 implementer-minutes, `bx-init` and
`bx-stories` also waiting on the engine's, jig's and the playground's tasks
named above. In parallel: `bx-machine` (50) -> `bx-kinds` (50) ->
`bx-commands` (40) -> `bx-script-forge` (40), joining at `bx-stories`.
What other parts wait on: `box-pipe` (the pipe's lines, for serve's box link:
`fr-make-forge`, `rm-remove-forge`) and `bx-machine` (`application.owned`
and `application.demo.fake`, for `fr-setup` and `pg-start`), both needing
nothing.

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
  not recorded. The box reads only the file's `playground` and
  `setup.directory` members; the rest is the steps' own (`temper forge prepare
  DIRS N` reads entry N through `temper_forge_setup::file::read`, the one
  reader).
- One forge of a kind at a time: `start N` with the kind's group taken answers
  `failed N busy` (the machine's one port per kind); groups may share a
  listener, an always step may not share one with any other step.
- `remove N` answers, besides `removed N`, as `stop` is refused: `failed N
  busy`, `failed N stopping`, and `failed N files not removed` on a file error.
- A removed group that is not a kind's group (the document's, or one the binary
  read from its file at the box's start) is gone for the box's life.
- The forge's directory is removed by the box itself with kernel operations
  (moved into `box/trash/N`, then swept as the worker's `Sweep` sweeps), not by
  a new `temper` command: a new command row needs a help page, which is the
  lead's. A directory outside the data directory is named and left.
- An owned forge's started notice for a kind's group is built from the
  template (`application.owned`), its label renumbered with the group; an owned
  forge made by `SetUp` takes `application.owned`'s `directory` as written
  (contracts 6: its record is made "from it"), so the notice's header paths
  are its own.
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
- `temper box --fake-backends` keeps its fake GitHub forge 2 and managed
  Forgejo 4 as seeded forges of its effective document (D1): `bx-init` makes
  its rewrite add them to a template that names none.
- The playground designer's contract problem (scripts name forge 1) is
  `bx-script-forge`: `temper fake llm DIRS` finds the playground's forge from
  its group file and moves every forge-1 resource tag of its scripts to it.
- The owned Forgejo set up afresh from the cache after a removal (draft 6.4)
  needs no code of its own: the new forge's file gives its `fetch`, which
  reuses a cached binary of the pinned digest (setup.md 6.6), and its
  `prepare` makes the removed directory anew; `bx-stories` asserts it.
- The process stories are two tasks: `bx-stories` (in a new
  `tests/box_stories/` module of `tests/process.rs`, so it shares no file with
  another part's stories) and `bx-story-github` (a new walk of the fake
  backends).

## Contract problems

- Contracts 3 gives `remove`'s one answer, `removed FORGE`; the box also answers
  `failed FORGE busy|stopping|files not removed` (as `stop` and `start` are
  refused under 7.1's one-ask rule). Proposed: a kept `BoxRemove` answered
  `failed` stays kept and is sent again (D4's rule for an unanswered act);
  serve's box link reads `removed` as the answer and `failed` as not yet.
- `ids.md` gives `bx-machine` "`temper init`'s template naming no forge, with
  those members and the grants"; it cannot need nothing and do so (Decisions,
  above). Cut: `bx-machine` is the readers and types, needing nothing;
  `bx-init` is the template. A task that needs the template naming no forge
  names `bx-init`, not `bx-machine`.
- Contracts 3: "`ENTRY` is the forge's setup entry in exactly the form of one
  `application.setup[]` entry". The box and every command read it with
  `temper_forge_setup::file::read` (`bx-kinds`), which refuses an owned entry
  without `"start": "asked"` and any top-level member but `setup` and
  `playground`; serve's writer (`fr-make-forge`) writes the paths as the
  document has them (relative to the data or cache directory, or absolute).

done
