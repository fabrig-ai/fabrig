# Settling

2026-11-24. How the store puts each map value at another layout again
at the binary's while it serves, so that plans at open and conversions
at read stay few, and a layout no value holds leaves the registry
(temper's `draft/layouts-both-ways.md`, 8.2). It puts what a settling
job would have (`jobs.md`, 5), without holding the open.

## 1. What settles

- **A value settles** where it is a value of the map of a registered
  value type, held in the map (`logs.md`, 2: never a log-held key), whose
  head is the id of a layout other than the binary's that the open
  planned (`layouts.md`, 5.3), older or later (`layouts.md`, 5.6),
  whether or not its plan keeps. Its put is the value as the read edge
  gives it (`layouts.md`, 4), at the binary's layout, its head the
  binary's id. A value the plan refuses is left as stored.
- **What doesn't:** the frames and the values a task log holds, which
  nothing writes again (`transcripts.md`, 6; `logs.md`, 8); a written
  job's layouts, which its job converts at open (`jobs.md`, 4); the
  secrets' store and every family outside the registry
  (`layouts.md`, 1).

## 2. The walk

- **One walk per start,** in memory, never a row: from the first value
  type, in the registry's key order, holding a value that settles,
  through each of its ranges in key order, then the next. A value type
  the census shows holds no value at a layout other than the binary's
  (`layouts.md`, 5.3) is passed over. A start begins it again from the
  first; a value already put is at the binary's layout, so a walk cut
  short costs a page read again.
- **It starts once the store serves,** after the start's wires commit
  (`records.md`, 3), and ends at the last range. A store with nothing
  to settle runs none.
- **A page at a time:** the walk reads the next rows of its ranges as
  stored, at most the store's page bytes (`opening.md`, 2, step 4),
  holding one page, and hands its values on as the commits it rides
  take them (3). It reads as any read of the map does, within the
  component's reads in flight. A commit still waiting between the
  component and skein-kv when the page is read counts as one since the
  read (3), so the page is never trusted over a write it may not hold.

## 3. Riding the root's commits

The root's commits are the store's only commits while it serves, each
one skein-kv commit numbered as the root numbers it (`commits.md`,
section 1). Settling makes none of its own: as orphan erases do, it
adds its puts to the commits the component submits for the root.

- **What a commit carries:** after the root's own writes become
  operations, the page's next values that settle, in key order, while
  the commit stays within the bounds the root's commits are sized by:
  its operations, and its bytes within the store's commit edge
  (`records.md`, 4), at most 32 values a commit. A value too large for
  what a commit leaves waits for one that leaves it room; the walk
  stays at it.
- **Never over a newer write:** a key that the commit writes, or that
  any commit since its page was read wrote, is dropped from the page:
  the root's write stands. So a settled put always carries the value
  its key held just before the commit it rides, converted, and the
  root reads the same value before and after it.
- **No effect of its own:** the commit is the root's, its number and
  its outcome the root's decision's. A settled put fails or survives
  with it; a crash loses none of what a durable commit holds.
- **Idle stores wait:** with no commit of the root, nothing settles.

## 4. Retiring what it empties

- **A layout emptied:** once the walk has passed the last range of a
  value type, each layout of it other than the binary's that the walk
  found no value at, having put every one it met, is retired: its
  `Layout` row erased, with each `Text` no other row names, in the next
  commit it rides, as the open's retirement does (`layouts.md`, 5.4).
  A layout one of whose values the plan refused, or the walk left
  waiting, stays.
- **Why the walk proves it:** only the binary writes the map while it
  serves, and at its own layout, so behind the walk's cursor no value
  at another layout comes back; a key the walk dropped (3) was written
  at the binary's layout or erased.
- **The order stays:** a retired layout keeps its place in its value
  type's order (`layouts.md`, 5.6). Its plan, still in memory, is read
  by no value, and the next open plans none for it.
- **After settling,** the registry holds the binary's layouts of every
  value type of the map, beside the layouts a log or the frames may
  still hold.

## 5. Bounds

- **Memory:** one page of the store's page bytes, the read edge's
  buffer and workspace (`layouts.md`, 7), and the walk's cursor.
- **Per commit:** at most 32 values, within the operations and bytes
  the root's commits are sized by, so no startup check moves and the
  store never answers a commit busy (`commits.md`, section 1).

## 6. Tests

- **Step tests,** in `jig-protocol-store`, over `jig-test-layouts`'
  fixture families: a store holding values at two older layouts and a
  later one, opened by a binary and served, settles every value over
  the commits the test makes, each value reading the same before and
  after, and the registry then holding the binary's layout alone; a
  commit carries at most 32 settled values and stays within its bounds;
  a value written by the root between a page's read and the commit
  that would settle it keeps the root's value; a value its plan
  refuses is left, its layout kept; a log-held value type and the
  frames settle nothing; a restart's walk begins again and finds
  nothing left to put; a store with nothing to settle reads no page.
- **The store's world,** `jig-store-world`: commits crashed while
  settled puts ride them, the referee finding each value as the
  acknowledged commits left it.

## 7. Open questions

- **An idle store** settles nothing. A store that commits rarely could
  settle on a timer of its own, by a commit the root would number.
