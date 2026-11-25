# The store: testing

## 1. Testing

Each within jig's suites' budgets.

- **The real store in the domain worlds.** The worlds run the component
  over skein-kv's in-memory mode, in place of a fake of the map, so every
  story runs the real keys, codecs, intervals, paging and joins. No
  world fakes the store's component: the real one runs in each
  (skein/docs/foundation/testing-strategy.md, section 4).
- **The restart oracle** (`../../domain/engine/world.md`, section 1): every
  world story ends with a cold restart over its store that restores
  with every link made and reads back what it wrote. The cross-record
  rules a restore once checked are proved there, writer by writer,
  never re-judged at a start.
- **Faults come from the world.** The world holds commits between the
  root and the component, in order: it delays them to make the store
  slow, and fails one to stop the engine. A crash keeps what reached the
  store and loses what the world held.
- **Transcripts' files have a stand-in** in the domain worlds: an
  in-memory file system answering the component's file requests, with
  the same rule after a crash. A crash keeps both in-memory stores and
  the stand-in, and the next engine's component is made over them,
  forgetting the reads they kept (`skein/docs/design/data/kv.md`, 6.5),
  and opens as `opening.md` says.
- **The store's world,** `tests/store`: the component over skein-kv and
  files on skein's simulated, crashing disk, a scripted root committing,
  loading, reading and appending turns, crashed at every file operation
  and sync. Its referee: after recovery, the last acknowledged commit, or
  that and one whole uncertain one, never part of one; no turn readable
  past its commit and none missing before it; numbers that match the
  header; pages equal to a model's.
- **The core's world on the simulated disk,** for a decision's
  durability across families (`../../domain/engine/decisions.md`, 1.1 and `../../domain/engine/restart.md`): the
  core world's directory-backed store (`jig-test-store-files`), killed
  at a chosen file operation of a chosen commit, and the engine started
  again through its link pass. Its rows: a kill at each operation of a
  decision's commit (after each record's bytes, mid-frame, between
  families), for a decision that makes a task and the person task
  linking a person, and one that makes a person and a task linking
  it; a second kill at each operation of the restart that follows the
  first, and of the commits after it; a kill at each step of a
  snapshot with commits racing it; a kill with several commits in
  flight. Its referee: the start opens, never refused and never a
  panic, with the last acknowledged decision, or that and one whole
  uncertain one, never part of one.
- **Generations and jobs,** in the store's world: fixture owner sets
  registering new layouts and running declared jobs at the same levels,
  checked, opened `pending`, switched only when the root asks, never
  before, and a root that ends before asking leaving the old generation
  active; stores below the floor, behind the binary's levels or without
  a registry refused before any upgrade; a second start refused `Busy`
  while one holds the lock; the
  copy's secrets private; the copy's snapshot and sealed segments
  linked to the previous generation's, each file with two links, its
  newest segment and its secrets copied with one, and a link the file
  system refuses copied in its place; going back before the copy's
  first commit of its own, after job commits past skein-kv's snapshot
  bound replaced the copy's linked snapshot, opening the previous
  generation at its last commit with its records as they were; a cut
  at every write, sync, link and rename of the
  first start, the copy, the registry's and jobs' commits, the manifest, the switch
  and the drop, each
  run from its own start, the referee finding one whole generation
  active, the other strays removed, and its records the upgraded or the
  old, never a mix.
- **The transcripts' framing's golden bytes,** as skein-kv keeps its
  own (skein's `data/kv.md`, section 9): a task's file of frames made from
  fixed turns (a window turn, a turn at the frame's bound, two
  attempts), kept beside the component's tests and never regenerated
  by the test that reads it; the writer's file compared with it byte
  for byte, and the reader walking it from both ends to the same turns.
  A change to the framing (`transcripts.md`, 1: the frame's length at both ends and its
  checksum, around the body) fails it, naming the format. The framing
  stays as it is: a change designs how the store reaches it in that
  landing (temper's `draft/archive/store-floor.md`, section 5). The core's
  `TurnRecord` and smith's turn move by their layouts
  (`transcripts.md`, 6).
- **Rehearsing and going back** (`opening-rehearsal.md`, 1), in the store's world over
  fixture sets: a rehearsal reporting each job and its counts and the restores whole,
  `A` still active and its maps unchanged, and the next start removing
  the scratch; one meeting a refusal, reporting it in a start's words;
  going back after an upgraded start that failed before its first commit
  of its own, the next start opening `A` as it was; refused once the
  upgraded generation has committed, with no previous generation, and
  while another process holds the lock, each changing nothing.
- **The deployment's layout's golden listing:** a first start's directory,
  each name and each file's kind and mode, never regenerated by the test
  that reads it. A change to a name or to where a generation keeps its
  maps fails it, naming the layout: from the first release a later build
  reads this layout as it finds it (temper's `draft/v0.1.0.md`, section 5).
  The manifest and `active` bytes are checked separately, with fixed-rule
  values built from the schema's text for the manifest, and a generation's
  number in decimal, no leading zero, and a newline for `active`. The
  manifest's fields are the component's family's, at its fixed version.
- **Transcripts converted as read,** most of it at the lowest layer.
  Step tests in `jig-protocol-store` over fixture frames' families (a
  body family and a turn family, `jig-test-layouts`): frames written at
  each fixture layout, in a file and in a turns log, read by every kind
  of `transcripts.md`, 3, each handing on the bytes the binary's codec
  writes for the same values, and the file's bytes unchanged; a body
  and a turn each at another layout, and both; the ids before the
  registry, the body at the core's version and the turn at each kept
  version, seeded and read; bounds judged on converted sizes; an id
  with no row, and a value its plan refuses, an unread frame; a frames'
  layout with no plan refusing the open; a frame written at the
  binary's ids by a later layout's binary; over skein-io's fault
  points, a cut at each step of a read changes no file. Fuzzed: a
  frame's decode over random fixture frames at random layouts. The core's closed
  window starts fresh with its tail, counted as a transition
  (`../../domain/engine/executors-transcripts.md`, 1); no map patch closes it at open.
  Generation, upgrade, check and frame scenarios run in the focused
  `jig-store-world::suite` binary, with short cases as data rows and a few
  cuts chosen for what a cut there can lose (`../../domain/testing.md`, 9). Their
  whole sweeps are under `fuzzy::` in that binary. The copy's
  mutation cuts belong to `fuzzy::upgrades::drop_copy_and_switch_recover_after_every_mutation`;
  `fuzzy::upgrades::cuts_across_check_and_switch_recover_whole_maps` starts
  at the seal's sync and cuts every operation through the check and switch.
  Frame scenarios kept above step tests prove durable files across
  restart; decoder-only duplicates stay in the step tests.
- **Step tests:** each key encoder against the derived order; each
  range's intervals against its predicate; each codec both ways, with
  fixed-rule values built from schema text or generated records built by
  their public fields, not stored codec bytes (skein's `data/codec/golden.md`, 1);
  each root's largest value decoded within its heap bound and released,
  at the ceilings if it fits the memory test's cap, otherwise at lower
  limits with its encoded and decoded bounds at the ceilings pinned
  (`jig/tests/store-codecs/tests/suite/largest_memory.rs`); refusals at open
  (format, deployment, sizes, numbers).
  Direct transcript pages either advance through a nonempty prefix or
  terminate `Unread` when no turn fits. The copy's check refuses a row
  the restart cannot load (`Check::Load`), naming its key: the active
  generation and source stay unchanged, and the next start sweeps the
  abandoned copy.
- **Fuzz targets** for each family's decoder and a frame's. Family seeds
  are full fixed-rule values built from their schema texts, not files.
- **On the real ring,** the reference binary (`../service.md`, section 7)
  killed around each phase of a turn's commit and started again. A killed
  process keeps the page cache, so this checks integration, not power
  loss (skein's `data/kv.md`, section 9).
