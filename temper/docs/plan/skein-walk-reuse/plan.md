# skein-walk-reuse: the plan

Since the gate runs each test binary in one process, skein-codec's
`nested_walks_grow_both_stacks_and_reuse_them` fails depending on what
ran before it (`inputs/request.md`): the walk stack's room is the same,
5, but its buffer's address moved. One task, in skein.

**The cause.** The buffer legitimately moves. `Build::room_nodes` sets a
stack's room with `List::with_capacity(room)`, which only sets the
list's cap; a `List` allocates as it fills (`skein/docs/design/lib.md`,
5), from a floor of 4. `start` pushes the root's frame, so the buffer
holds 4 frames when the test reads its "before"; the carry's walk
reaches a fifth frame and `reserve_exact` grows the buffer to the room.
glibc's `realloc` grows the block in place in a fresh process and moves
it once other tests have taken the next block. That is growth as a
record needs it (`skein/docs/design/codec.md`, 11.7), not a failed
reuse: the test read too early, and by address. The task confirms it
with a count before changing the tests.

**The fix.** The reuse checks stop reading addresses: they compare the
tables' rooms and count the allocations around each reused pass, which
must be none, with skein-heap's span under the counting allocator. An
address can't show a copy reliably either way: an allocator may grow a
block in place or hand a freed block straight back. The count shows
both a growth and a same-size replacement, in any order.

The cut wrote the design first:

- `skein/docs/design/testing.md`, 5: *Reuse is counted, never read from
  an address*, and the span's count, `Span::made`.
- `skein/docs/design/codec.md`, 11.8 (*Memory*): capacities and the
  span's count show reuse, never an address; a stack's buffer grows as
  the walk deepens, so a pass reuses its stacks once a pass as deep has
  run.
- One **Not yet** entry each in `codec.status.md` (11.8) and
  `testing.status.md` (5), and `skein/docs/design/STATUS.md`'s first
  paragraph no longer lists `codec.md` as having no gap. The task puts
  all three back.

## Parts and tasks

| Part | Task | What | Needs | Effort | Minutes |
|---|---|---|---|---|---|
| `reuse-counted` | `reuse-counted` | `Span::made` and its test; skein-codec's unit tests under the counting allocator; the six reuse checks by rooms and count, the nested one reading after its first walk; `Value::allocation` gone; the cause confirmed by a count; a same-room stack replacement shown to fail the checks; the lib binary in one process in its listed order and three shuffled ones | | medium | 75 |

`notes/common.md` holds the rules the task keeps.

## Decisions

- **Count, not capacities alone.** The request allowed either. Rooms
  alone miss a table replaced at its size and a list's buffer
  regrowing within its room, which is what 11.8's step tests exist to
  catch. skein-heap already counts each thread's allocations (`made`);
  `Span::made` exposes it, an additive method: no reader of `Span` or
  `Grown` changes.
- **skein-codec's unit-test binary declares the counting allocator,**
  as `skein-forge-connection`'s does. The tests reach the patcher's
  private tables, so they stay unit tests.
- **The nested test keeps its two claims, each checked soundly:** the
  first walk stays in the rooms its start reserved (rooms), and a
  second pass as deep allocates nothing (count).
- **Where one table fills by design while the others are kept**
  (`written_nesting_...`'s larger old record, `refused_growth_...`'s
  first start after the refusal), rooms there and a count from the next
  pass, a pre-approved option.
- **Orders.** libtest sorts by name and has no reverse, so the task runs
  the listed order and three shuffled ones (`--shuffle-seed` under
  `RUSTC_BOOTSTRAP`), with a fallback if that can't reach the binary.
  Order-independence holds by construction: no check reads an address.
- **No test moves to a process of its own,** and no budget, tier,
  filter or override changes.

## Critical path

`reuse-counted`, about 75 minutes, alone.

## Risks after landing

- The counting allocator now runs under every skein-codec unit test; the
  task reports the lib binary's time before and after. It only counts,
  so the cost should be small.

## Overlaps with other drafts

None. No draft beside this one changes skein-codec, skein-heap,
`skein/tests/heap`, or `skein/docs/design/{codec,testing,STATUS}*.md`.
`Cargo.lock` gains one line in skein-codec's entry; the drafts beside
this one that touch `Cargo.lock` have merged.
