+++
id = "forge-tail"
part = "forge-tail"
kind = "feature"
needs = []
files = ["skein/crates/skein-forge-connection/src/grow.rs", "skein/crates/skein-forge-connection/src/connection.rs", "skein/crates/skein-forge-connection/src/tests/connection.rs", "skein/crates/skein-lib/src/tail.rs", "skein/tests/fake-peers/tests/forge/github.rs", "skein/tests/forge/github/src/runner.rs"]
effort = "medium"
minutes = 30
+++

# forge-connection's log tail stands on lib's `Tail`

## Goal

`grow::Tail` re-implements lib's ring. What it adds of its own is the runner's failure marker and whether bytes
were dropped: that stays, as a small `grow::LogTail` over `skein_lib::Tail`.

Acceptance:

- `rg -n 'struct Tail|grow: Grow' skein/crates/skein-forge-connection/src/grow.rs` finds nothing, and
  `grow.rs` holds `Grow`, `Full`, `LogTail` and `FailureEnd`.
- `skein_lib::Tail::into_boxed` exists, with a test across a wrap.
- A log read keeps the same bytes, the same `dropped` and the same phases: `heavy cargo nextest run -p
  skein-forge-connection`, `-p skein-lib`, `-p skein-fake-peers-tests --test forge` (find the test binary's name)
  and `-p github-conformance` pass.
- `grow.rs` ends at most 100 lines (136 at base).

## Inputs

- `temper/docs/plan/debt-13/notes/common.md`
- `skein/docs/development/debt/D013-forge-connection-parallel-lists.md`
- `skein/docs/design/lib.md`, 5 (containers grow as they fill) and 6
- `skein/docs/design/forge-connection.md`, 4.5 and 7.5

## Owns and may change

- **Owns:** `skein-forge-connection/src/grow.rs`, and `Receiving::Tail` and its two constructions in
  `connection.rs`. `skein-lib/src/tail.rs`: the new `into_boxed` and its test.
- **May change:** the tail's tests in `src/tests/connection.rs`; `skein/tests/fake-peers/tests/forge/github.rs` and
  `skein/tests/forge/github/src/runner.rs`, which read `grow::Tail`: the import and the two constructions alone.
- **Not to touch:** `Grow`, which stays as it is. `connection.rs` beyond `Receiving::Tail`: other tasks of this plan
  rework the rest.
- **Exception to the assertion rule, the cut's:** `the_tail_keeps_the_last_max_bytes_across_writes` asserts
  `tail.capacity()` after each write, which pins `grow::Tail`'s own growth. That mechanism goes: lib's `List` grows
  as it fills, and lib's tests pin that. The `capacities` column goes with it. Its `expected` and `dropped` columns
  stay as they are. Every other assertion stays.
- **Options:**
  - If a heap test or a world's ceiling moves, that's a change of behaviour: stop and ask.
  - If `Tail::new`'s first size turns out to be read by a test beyond the one above, keep the test's assertion and
    say so in the report.

## Steps

1. In `skein-lib/src/tail.rs`:

   ```rust
   /// The retained bytes in arrival order, in the ring's own buffer: no copy.
   #[must_use]
   pub fn into_boxed(self) -> Box<[u8]>;
   ```

   It rotates by `next`, as `grow::Tail::finish` rotates by `start`. So the finish of a log read makes no second
   buffer of the body's size. Test it beside `every_chunking_keeps_the_same_suffix_at_zero_and_wrap_boundaries`,
   against `to_boxed`, at every capacity and split that test walks.
2. In `grow.rs`:

   ```rust
   /// A body's last bytes, optionally ending at its first failed process (forge-connection.md, 4.5).
   #[derive(Debug)]
   pub struct LogTail {
       tail: skein_lib::Tail,
       dropped: bool,
       failure: Option<FailureEnd>,
   }
   impl LogTail {
       pub fn new(max: u32) -> Self;
       pub fn failed_step(max: u32) -> Self;
       pub fn write(&mut self, piece: &[u8]);
       pub fn dropped(&self) -> bool;
       pub fn finish(self) -> Box<[u8]>;
   }
   ```

   `write` keeps today's order for each byte: the failure marker first, then a byte past a failure is only counted
   as dropped. A byte that finds the ring full, or a ring of no capacity, sets `dropped`. `FailureEnd` is unchanged.
3. `connection.rs`: `Receiving::Tail { tail: LogTail, .. }`; `Tail::new(env.limits.chunk, max.min(self.raw_cap))` and
   `Tail::failed_step(env.limits.chunk, self.raw_cap)` lose their first argument.
4. The two worlds: `Tail::new(bytes, bytes)` and `Tail::failed_step(bytes, bytes)` become `LogTail::new(bytes)` and
   `LogTail::failed_step(bytes)`.
5. The tail's tests take the new spelling. `a_failed_step_tail_falls_back_and_reports_omission_at_tiny_bounds` already
   pins `dropped` at a cap of 0 and at a full ring: break `dropped`'s rule by hand once and see it red.
6. `cargo fmt`, `heavy cargo clippy -p skein-lib -p skein-forge-connection -p skein-fake-peers-tests -p
   github-conformance --all-targets -- -D warnings`, the tests above, commit, push.

## Report

What changed, the commit, what ran with counts, the rule broken and what failed, and the net lines, production and
tests apart.
