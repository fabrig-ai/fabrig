+++
id = "reuse-counted"
part = "reuse-counted"
kind = "feature"
needs = []
files = ["skein/testing/skein-heap/src/lib.rs", "skein/tests/heap/tests/suite/span.rs", "skein/crates/skein-codec/Cargo.toml", "skein/crates/skein-codec/src/tests.rs", "skein/crates/skein-codec/src/value.rs", "skein/crates/skein-codec/src/patch/run/tests.rs", "Cargo.lock", "skein/docs/design/codec.status.md", "skein/docs/design/testing.status.md", "skein/docs/design/STATUS.md"]
effort = "medium"
minutes = 75
+++

## Goal

skein-codec's patcher tests show that a reused pass neither grows nor
copies a table by counting the allocations around it, with skein-heap's
span under the counting allocator, and by comparing the tables' rooms;
no test reads an address. `nested_walks_grow_both_stacks_and_reuse_them`
then passes in one process whatever ran before it.

**What the cut found** (confirm it, step 3): the walk stack's buffer
legitimately moves. `Build::room_nodes` (`patch/run.rs`) sets a stack's
room with `List::with_capacity(room)`, which only sets the list's cap:
a `List` allocates as it fills (`skein/docs/design/lib.md`, 5;
`skein-lib/src/list.rs`, `push`, growth floor 4). `start` pushes the
root's frame, so the buffer is 4 frames when the test reads `before`;
the carry's walk reaches its fifth frame and `reserve_exact` grows the
buffer to the room, 5. That `realloc` keeps the address when glibc can
grow the block in place (a fresh process) and moves it when the next
block is taken (after other tests). The room, 5, never changes. This is
growth as the record needs it (codec.md, 11.7), not a failed reuse; the
test read its "before" too early and by address.

Acceptance:

- `skein_heap::Span` gains `made(&self) -> u64`: the allocations its
  thread made since `Span::start`, a reallocation as one (testing.md, 5).
  A new test in `skein/tests/heap/tests/suite/span.rs` pins it.
- skein-codec's unit-test binary declares the counting allocator, and
  `skein-heap` is one of its dev-dependencies.
- In `skein/crates/skein-codec/src/patch/run/tests.rs` nothing reads an
  address: `rg -n 'as_ptr|allocation\(' skein/crates/skein-codec/src/patch/run/tests.rs`
  prints nothing, and `Value::allocation` (`value.rs`, test-only) is
  gone. Every check that compared `allocations(...)` or
  `.allocation()` by address now compares rooms and counts the reused
  pass's allocations, none, as the sketches below say.
- `nested_walks_grow_both_stacks_and_reuse_them` checks that the first
  nested walk stays in the rooms its start reserved (rooms only), then
  that a second pass as deep, start to finish, allocates nothing.
- The lib binary passes in one process: `heavy cargo test -p
  skein-codec --lib -- --test-threads=1`, its listed order, and the
  other orders step 6 gives.
- The status lines this task closes are gone (step 7).

## Inputs

- `temper/docs/plan/skein-walk-reuse/notes/common.md`: every rule there
  holds.
- `skein/docs/design/codec.md`, 11.7 (*Kept and growing tables*,
  *Per record*) and 11.8 (*Memory*: capacities and the span's count,
  never an address; a stack's buffer grows as the walk deepens). The cut
  wrote 11.8's lines; build them.
- `skein/docs/design/testing.md`, 5 (*Reuse is counted, never read from
  an address*: `Span::made`). The cut wrote it; build it.
- `skein/docs/design/lib.md`, 5: a `List` allocates as it fills.
- `skein/docs/foundation/testing-strategy.md`, 8 (*A binary is a
  process*): why the order matters.
- The precedent for a step crate's unit tests under the counting
  allocator: `skein/crates/skein-forge-connection/src/tests.rs` and its
  `Cargo.toml`'s dev-dependencies.

## Owns and may change

- Owns: the files in the header.
- Readers and neighbours:
  - `Span` gains a private field and a method; nothing outside
    `skein-heap` constructs a `Span` or names its fields, so no reader
    changes. `Grown` doesn't change.
  - `skein/crates/skein-codec/src/patch/run.rs`,
    `patch/run/walk.rs`, `patch/run/build.rs`: only under the option
    below for a pass that allocates where 11.7 says it reuses.
- Options, each pre-approved:
  - **A call that allocates one table by design while the test shows
    the others kept** (`written_nesting_...`'s start of the larger old
    record, which fills its replacement entries for the first time;
    `refused_growth_...`'s first smaller start after the refusal, whose
    walk buffer the refused decode never reached): compare the kept
    tables' rooms there, and count from the next whole pass, which
    allocates nothing. A comment names the table that allocates.
  - **A whole reused pass that counts an allocation this spec says it
    doesn't:** find the table. If the patcher replaces or regrows a
    table 11.7 says it keeps, that's the patcher's fault: fix it in the
    run files above, and report it first. If it is a buffer filling for
    the first time, start the count one pass later and say so in a
    comment and the report.
  - **Another skein-codec unit test that fails under the counting
    allocator** (it only counts; `System` allocates): if the test leaned
    on an address or the system allocator's own behaviour, give it the
    same treatment and name it; otherwise it is a `Question:`.
  - **The other orders (step 6):** if `RUSTC_BOOTSTRAP` can't reach the
    test binary through `heavy`, run the fallback step 6 gives and say
    so.

## Steps

1. On the base, once: `heavy cargo test -p skein-codec --lib --
   --test-threads=1`. Quote the result, pass or the request's failure;
   either is fine (the heap decides).
2. skein-heap: `Span` keeps the thread's `made` count beside `start`,
   read after `warm_runtime()` as `live` is, and:

   ```rust
   /// The allocations this thread made since the start, a reallocation
   /// as one: none shows a table kept rather than replaced (testing.md, 5).
   #[must_use]
   pub fn made(&self) -> u64
   ```

   (`wrapping_sub`, as the counts wrap.) The module doc's *Around a
   call* says a span also counts them. In `span.rs`, a test, for
   example `a_span_counts_the_allocations_its_thread_made`: a vector
   made before the span; inside it, `made()` is 0, then a
   `reserve_exact` past its capacity (one) and a box made and dropped
   (one) give 2, and a drop adds none. Break it once (return 0, or count
   frees) and see it fail.
3. skein-codec: `skein-heap.workspace = true` under
   `[dev-dependencies]`; in `src/tests.rs`,

   ```rust
   // The patcher's step tests count what a reused pass allocates (codec.md, 11.8).
   #[global_allocator]
   static HEAP: skein_heap::Counting = skein_heap::Counting;
   ```

   Confirm the cause before changing the tests: with a temporary span
   around the first nested pass's carry in
   `nested_walks_grow_both_stacks_and_reuse_them`, report its count (the
   cut expects at least one, the walk buffer's growth from 4 to 5
   frames), then remove it.
4. `patch/run/tests.rs`: `Allocations` and `allocations` become rooms
   alone, and a helper counts a call:

   ```rust
   /// The rooms of the run's tables (codec.md, 11.7): what reuse keeps.
   #[derive(Debug, PartialEq, Eq)]
   struct Rooms {
       entries: [u32; 2], // the old value's and the reserve's, sorted
       out: u32,
       walk: u32,
       opened: u32,
   }
   fn rooms(patcher: &Patcher) -> Rooms
   /// What `call` returns, and the allocations it made on this thread
   /// (testing.md, 5).
   fn counted<T>(call: impl FnOnce() -> T) -> (T, u64)
   ```

   `assert_run_rooms` reads `rooms`. Each reuse check becomes: the
   record's box made before the span (`list_bytes(..)`, `WIRE.into()`,
   `Box::from(..)` are the caller's), then one `counted` around the
   reused pass, `start` through `finish` (`finish` lends the output and
   allocates nothing), asserting `made == 0` with a message saying what
   is reused, and `rooms` equal where the test compared allocations
   before. The tests: `small_records_hold_at_most_twice_their_run_price`,
   `tiny_ceilings_allocate_only_their_configured_room` (its start alone),
   `nested_walks_grow_both_stacks_and_reuse_them`,
   `written_nesting_grows_live_stacks_and_reuses_grown_old_entries`,
   `a_larger_next_pass_grows_tables_and_smaller_passes_reuse_them`,
   `refused_growth_keeps_its_room_for_smaller_records`. Keep each test's
   name, fixtures and other assertions.

   The nested test's shape:

   ```rust
   patcher.start(plan, D, WIRE.into()).expect("nested old");
   let reserved = rooms(&patcher);
   carry_nested(&mut patcher); // stop, carry the list, Done, finish == WIRE
   assert_eq!(rooms(&patcher), reserved, "the nested walk stays in the rooms its start reserved");
   assert!(patcher.build.walk.capacity() > 2);
   assert!(patcher.build.opened.capacity() > 2);
   // The first walk grew the stacks' buffers as it deepened; one as deep reuses them.
   let wire: Box<[u8]> = WIRE.into();
   let ((), made) = counted(|| {
       patcher.start(plan, D, wire).expect("nested again");
       carry_nested(&mut patcher);
   });
   assert_eq!(made, 0, "a nested walk as deep as the last reuses its stacks");
   assert_eq!(rooms(&patcher), reserved);
   ```

   Remove `Value::allocation` from `value.rs`; tests read `room()`.
5. Make the new checks bite, once each, and restore: in `Build::clear_stacks`,
   replace the walk stack with a fresh list of the same room
   (`self.walk = List::with_capacity(self.walk.capacity())`): the
   nested test and `small_records_...` fail on their count, in any
   order. Report the failures.
6. The orders, through `heavy`, after `cargo fmt`:
   - listed: `heavy cargo test -p skein-codec --lib -- --test-threads=1`;
   - others: libtest sorts tests by name and has no reverse, so three
     shuffled orders, `RUSTC_BOOTSTRAP=1 heavy cargo test -p skein-codec
     --lib -- --test-threads=1 -Z unstable-options --shuffle-seed N`
     for N = 1, 2, 3 (it rebuilds skein-codec's few crates once);
   - fallback, if the shuffle can't run: the six tests above first in a
     fresh process (`-- --test-threads=1 patch::run::tests`) and each
     alone (`--exact`).
   Report each command and its count.
7. Close the status lines: remove the `- **Not yet:** 11.8: ...` bullet
   from `skein/docs/design/codec.status.md`, leaving its date line and
   `No gap.`; remove the `- **Not yet:** 5: ...` bullet whole from
   `skein/docs/design/testing.status.md`, nothing else there; in
   `skein/docs/design/STATUS.md` put back the sentence
   ``README.md`, `codec.md`, `world.md`, `fake-checkout.md`,
   `oauth-issuer.md` and `ui-text.md` have no gap.`` (wrapped as its
   neighbours are) in place of the one naming the codec's step tests.
8. Once: `cargo fmt`; `python3 -I -B skein/scripts/old_forms.py`;
   `heavy cargo clippy -p skein-codec -p skein-heap -p skein-heap-tests
   --all-targets -- -D warnings`; `heavy cargo nextest run -p
   skein-codec -p skein-heap -p skein-heap-tests`. The change is test
   code and one additive method, so skein-heap's and skein-codec's
   dependents don't run (a departure from the implementer's text,
   stated). Report the skein-codec lib binary's time in that run beside
   its time on the base (step 1's run or a nextest run of the base):
   the counting allocator now runs under all its tests.
9. Commit (`skein-codec: count a reused pass's allocations, never an
   address`, saying why: the reuse checks depended on what ran before
   in one process), run the citations check (`notes/common.md`), push.

## Report

The implementer's report, with: step 1's base result; step 3's count
for the first nested carry, and whether it confirms the cut's cause;
each test changed and the check that replaced its address; each option
taken; step 5's break and the failures seen; step 6's commands and
counts; the lib binary's time before and after.
