# Restoring temper's store

This document holds the store's restart and restore protocol, split from [`store.md`](store.md).

## 1. The restart

- **One step, one interval.** The core's restart script asks for the
  forge connector's step (`RestoreConnector`, jig's `domain/engine/restart.md`
  ). The root then pages through `Range::Forge`,
  restores each row into the forge (its `Event::Restore`), and ends the
  step with its `Restored`.
- **The routing maps `Range::Forge`** to one interval: every key under
  temper's tag whose family byte is below 128, which it asks jig's
  component to restore as the
  application's interval (jig's `protocol/store/loads.md`, 2, and `jig/docs/design/protocol/store/README.md`, 7
  ). The routing encodes the root's cursor with temper's key encoder,
  decodes each row with temper's codecs before the root sees it, and
  keeps the longest prefix of the page within the root's rows and
  decoded bytes, its cursor the last key kept.
- **Family by family,** in the variants' order, since the family's byte
  leads each key. The forge restores each row into its table and
  reconciles all at `Restored`, accepting any row order and joining
  a change's two parts (`store.md`, section 2, families 47 and 48) there.
- **Every row the predicate admits.** `Range::contains` admits every
  forge key below byte 128 and no core key; a test checks the interval against it on
  generated keys of both (`store.md`, section 6).
- **A row this build cannot read refuses the start, by name.** A row of
  temper's families that does not decode with this build's codecs, or
  whose record's key is not its row's (`store.md`, section 3), stops the restart
  before the root hears of it, and serve's start is refused as a store
  that refuses to open is: its words on serve's stderr after `temper
  serve: `, then exit 1, with nothing more read. The words:

  ```text
  temper serve: The store in DIR holds a NAME record, family N, that this build cannot read, and the start stopped there. Start the build that wrote it again, or, to start afresh, stop the service, move `active`, `generations/` and `transcripts/` out of DIR together, and start again: nothing is deleted, and the next start makes a new, empty store.
  ```

  DIR is the deployment's directory, as the other refusals name it,
  NAME the family's variant in `store.md`, section 2 and N its number. A store
  below the floor is refused earlier (`floor.md`, 4),
  so QA knows which named build to use or copy to make again
  (`../draft/archive/landed-qa.md`, 5). It is jig's row that
  does not decode, which stops the engine (jig's
  `protocol/store/loads.md`, 1), said by name.
- **Nothing else is read** but the landing history: no load, get or
  party's read of temper's families follows the restart, but the
  forge's loads of the families from byte 128 on, `Range::History`,
  routed to jig's application loads (`../domain/landing-history.md`,
  4.3). `Range::contains` admits their keys for `Range::History` alone,
  and `Range::Forge` none of them.
- **A row of `Making` restored** comes back waiting if it was making,
  its `copying` kept: nothing runs from the restore itself, and the
  setup's pull starts it once its forge is connected
  (`../domain/forge/adoption.md`, 9).
- **The core's ranges are jig's, filtered.** jig's restart ranges
  (jig's `protocol/store/keys.md`, 1.3, and `jig/docs/design/protocol/store/loads.md`, 2) are coarser than the root's
  core ranges. The routing asks for the jig ranges holding a root range,
  keeping each row the root's range admits: `CoreRange::Tasks` (the tasks child's live
  families and the goals' projections, core rows) is jig's `[Core, Tasks]`,
  in that order, also the keys' order; `CoreRange::{Calls, RunProofs}`
  are jig's `[Core]`. A root cursor is a key of its range, naming the jig
  interval holding it. If the filter drops every row while jig's walk
  goes on, it asks again from jig's cursor before the root hears of it.
  Thus a continuing page keeps a row, its cursor the last key kept.
- **The census is jig's walk, taken as it comes.** `Range::Census` is
  jig's census (jig's `protocol/store/loads.md`, 2): pages hold ended
  read positions and ended tasks in walk order, each task once per index
  row naming it. The cursor is the last index row's key, placed in the
  walk by jig's `Census::interval`. temper keeps no copy of the walk's order.
- **The root checks the census's rows, and keeps none.** Each row is an
  ended task or ended read position; the cursor is a census-admitted key past
  the page's start in walk order. An empty page may go on. The root
  neither pairs, orders nor dedupes rows, handing each to jig's core as
  it comes. jig's component never parts a position from its task, so
  the root's `loads.rows` is at least 2, checked at start. A cut census
  page stops the restart, as every cut restart page does.

## 2. Open questions

None.
