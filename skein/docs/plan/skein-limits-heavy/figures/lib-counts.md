# lib-counts: figures

All 28 `memory` tests were profiled before and after, alone in their own processes,
on the same x86-64 builder with the unoptimized test build and callgrind 3.24.0.
Both commands used `NEXTEST_DOUBLE_SPAWN=0`, the callgrind target runner,
`heavy cargo nextest run -p skein-lib-tests --test suite`,
`-E 'test(/^memory::/)' --test-threads=1`; only the profile output prefix changed.
The table gives each test process's `summary: Ir`, including startup; small
differences in unchanged tests are startup variation. All 28 passed in each run.

The instruction median of the 28 is between `a_queue_grows_to_its_capacity_exactly`
(rank 14) and `a_list_filled_to_its_capacity_stays_within_its_price` (rank 15).
Both stay: the former checks both push entry points, exact growth, refusal and order;
the latter fills, refuses, copies, clears and moves four item sizes at nine capacities.
`Metered::end` costs respectively 1,206,386 and 5,046,959 inclusive instructions.
The required queue-fill sample costs 12,754,407 there: the checks, rather than repeated setup.

| Row | Test | Before (Ir) | After (Ir) | Cause | What changed |
|---|---|---:|---:|---|---|
| `skein-lib-tests::suite` `memory` | `memory::a_deadline_table_holds_no_more_than_its_price` | 144,186,938 | 144,186,826 | `Deadlines::arm` 52,813,366; `Metered::end` 38,932,868 inclusive Ir | stays: per-operation price checks |
| `skein-lib-tests::suite` `memory` | `memory::a_journal_scenario_stays_under_its_peak_ceiling` | 1,417,947 | 1,417,712 | Process startup and counting-allocator initialization dominate | stays: no repeated large allocation |
| `skein-lib-tests::suite` `memory` | `memory::a_list_clone_holds_only_its_items_and_grows_as_it_fills` | 1,307,552 | 1,307,432 | Process startup and counting-allocator initialization dominate | stays: no repeated large allocation |
| `skein-lib-tests::suite` `memory` | `memory::a_list_filled_to_its_capacity_stays_within_its_price` | 9,460,427 | 9,460,251 | `Metered::end` 5,046,959 inclusive Ir; container operations | stays: per-operation price checks |
| `skein-lib-tests::suite` `memory` | `memory::a_list_grows_to_its_capacity_exactly` | 2,365,976 | 2,365,842 | `Metered::end` 618,549 inclusive Ir; container operations | stays: per-operation price checks |
| `skein-lib-tests::suite` `memory` | `memory::a_lru_holds_no_more_than_its_price` | 239,813,089 | 239,812,942 | `Lru::insert` 104,152,667; `Lru::remove` 66,608,792 inclusive Ir | stays: tree operations over the named capacity scenarios |
| `skein-lib-tests::suite` `memory` | `memory::a_map_holds_no_more_than_its_price` | 119,403,784 | 119,403,768 | `Metered::end` 42,035,703 inclusive Ir; container operations | stays: per-operation price checks |
| `skein-lib-tests::suite` `memory` | `memory::a_queue_filled_to_its_capacity_stays_within_its_price` | 20,737,342 | 20,737,152 | `Metered::end` 12,754,407 inclusive Ir; container operations | stays: per-operation price checks |
| `skein-lib-tests::suite` `memory` | `memory::a_queue_grows_to_its_capacity_exactly` | 3,364,663 | 3,364,531 | `Metered::end` 1,206,386 inclusive Ir; container operations | stays: per-operation price checks |
| `skein-lib-tests::suite` `memory` | `memory::a_queue_with_a_large_capacity_used_lightly_holds_what_it_holds` | 3,287,265,890 | 14,725,579 | A million metered turns: `Metered::end` 1,920,002,132 inclusive Ir | Turns 1,000,000 → 4,096; every step still metered |
| `skein-lib-tests::suite` `memory` | `memory::a_request_table_grows_within_its_held_and_peak_bounds` | 627,061,810 | 627,061,701 | `RequestTake::next_out` scans admission order: 324,144,260 inclusive Ir | stays: next cause about 0.047 CPU s; counts preserve named lifecycle checks |
| `skein-lib-tests::suite` `memory` | `memory::a_ring_used_lightly_holds_what_it_holds` | 3,287,267,961 | 14,727,308 | A million metered turns: `Metered::end` 1,920,002,132 inclusive Ir | Turns 1,000,000 → 4,096; every step still metered |
| `skein-lib-tests::suite` `memory` | `memory::a_set_holds_no_more_than_its_price` | 113,493,073 | 113,492,973 | `Metered::end` 38,619,824 inclusive Ir; container operations | stays: per-operation price checks |
| `skein-lib-tests::suite` `memory` | `memory::a_slab_grows_to_its_capacity_exactly` | 13,871,565 | 13,871,562 | Earlier-handle checks: `Slab::get` 2,410,493 inclusive Ir | stays: checks handles survive every growth |
| `skein-lib-tests::suite` `memory` | `memory::a_slab_grows_within_its_held_and_peak_bounds` | 29,532,249 | 29,532,115 | `Metered::end` 17,362,716 inclusive Ir; container operations | stays: per-operation price checks |
| `skein-lib-tests::suite` `memory` | `memory::a_stack_filled_to_its_capacity_stays_within_its_price` | 18,072,561 | 18,072,405 | `Metered::end` 11,004,269 inclusive Ir; container operations | stays: per-operation price checks |
| `skein-lib-tests::suite` `memory` | `memory::a_stack_grows_to_its_capacity_exactly` | 3,105,292 | 3,105,115 | `Metered::end` 1,196,168 inclusive Ir; container operations | stays: per-operation price checks |
| `skein-lib-tests::suite` `memory` | `memory::an_intake_filled_to_its_capacity_stays_within_its_price` | 16,894,834 | 16,894,780 | `Metered::end` 8,001,094 inclusive Ir; container operations | stays: per-operation price checks |
| `skein-lib-tests::suite` `memory` | `memory::an_intake_grows_to_its_capacity_exactly` | 2,826,433 | 2,826,326 | `Metered::end` 950,432 inclusive Ir; container operations | stays: per-operation price checks |
| `skein-lib-tests::suite` `memory` | `memory::an_intake_with_a_large_capacity_used_lightly_holds_what_it_holds` | 6,720,266,382 | 111,498,172 | A million metered deliveries: `Metered::end` 3,779,002,213; `Intake::meet` 1,893,000,172 inclusive Ir | Capacity 1,048,576 → 16,384; turns 1,000,000 → 16,384 |
| `skein-lib-tests::suite` `memory` | `memory::an_uncapped_list_allocates_only_as_items_arrive` | 1,274,582 | 1,274,467 | Process startup and counting-allocator initialization dominate | stays: no repeated large allocation |
| `skein-lib-tests::suite` `memory` | `memory::an_uncapped_map_allocates_only_as_entries_arrive` | 1,303,700 | 1,303,618 | Process startup and counting-allocator initialization dominate | stays: no repeated large allocation |
| `skein-lib-tests::suite` `memory` | `memory::an_unused_intake_holds_nothing` | 1,260,179 | 1,260,120 | Process startup and counting-allocator initialization dominate | stays: no repeated large allocation |
| `skein-lib-tests::suite` `memory` | `memory::an_unused_list_holds_nothing` | 1,260,074 | 1,259,982 | Process startup and counting-allocator initialization dominate | stays: no repeated large allocation |
| `skein-lib-tests::suite` `memory` | `memory::an_unused_queue_holds_nothing` | 1,260,218 | 1,260,159 | Process startup and counting-allocator initialization dominate | stays: no repeated large allocation |
| `skein-lib-tests::suite` `memory` | `memory::an_unused_slab_holds_nothing` | 1,261,410 | 1,261,365 | Process startup and counting-allocator initialization dominate | stays: no repeated large allocation |
| `skein-lib-tests::suite` `memory` | `memory::an_unused_stack_holds_nothing` | 1,261,380 | 1,260,026 | Process startup and counting-allocator initialization dominate | stays: no repeated large allocation |
| `skein-lib-tests::suite` `memory` | `memory::an_unused_uncapped_queue_holds_nothing` | 1,263,802 | 1,263,743 | Process startup and counting-allocator initialization dominate | stays: no repeated large allocation |

The three changed tests total 13,294,800,233 → 140,951,059 Ir (98.94% fewer).
Scaling their individual instruction reductions by the request's CPU figures
(0.336, 0.339 and 0.698 s) gives about 1.36 CPU s saved; this is an estimate,
not a new gate CPU measurement.

## Found and not fixed

- `RequestTake::next_out` in `skein/crates/skein-lib/src/requests.rs` repeatedly
  scans admission order while the request-table memory scenario drains bounded visits:
  324,144,260 / 627,061,810 = 51.69% of its inclusive instructions. A separate
  uninstrumented nextest run, through a Bash `time` runner, measured this test at
  0.089 user + 0.001 system CPU s. Eliminating the entire cause would save about
  0.0465 CPU s, below the stop rule. Fixing it would require preserving a traversal
  position across bounded visits while retaining output order and lifecycle behavior;
  no product change is made. The next-largest remaining test, LRU, measured
  0.062 CPU s; its largest product operation (`insert`, 43.43%) represents about
  0.027 CPU s, also below the rule. No repeated setup above the rule was found.
- `skein_heap::Meter` and `Counting` stay with `skein-world-costs`. In each original
  queue loop, `Meter::start` costs 740,000,740, `end` 783,000,783 and `check`
  862,001,074 inclusive Ir. In the intake, `Counting::alloc` costs 1,694,291,811
  and `dealloc` 1,488,221,584 inclusive Ir. These figures overlap caller costs;
  they must not be added to them. Optimizing them would require allocator/meter
  changes and its own accounting tests. Every turn remains metered here.
- Process startup remains in every profile, including the median and unused-container
  tests. Reducing it belongs to the separate process-cost work; the unused capacities
  and list-clone capacities allocate nothing up front and stay unchanged.

## Tests that went

None.

## Counts

- `memory::a_ring_used_lightly_holds_what_it_holds`: 1,000,000 → 4,096 turns.
  Two `u64` pushes and pops per turn pass 65,536 bytes, 64 ceilings, through a ring
  that can hold at most 128 such items: at least 63 wraps. A nonreclaiming ring
  crosses the 1,024-byte ceiling by turn 65. The compile-time inequality pins this
  margin. Dropping the second pop first failed the remaining FIFO comparison on
  turn 3 (`Some(1)` against `Some(2)`); suppressing only that comparison during
  the mutation let the meter fail at 1,536 bytes against 1,024, within 4,096 turns.
  Both pops and comparisons were restored.
- `memory::a_queue_with_a_large_capacity_used_lightly_holds_what_it_holds`:
  1,000,000 → 4,096 turns with the same shared constant and compile-time check.
  The capacity stays 15,000,000; the same throughput and per-turn 1,024-byte checks
  hold. Dropping the second pop failed FIFO at turn 3; suppressing only the first
  comparison during that mutation then failed the meter at 1,536 bytes against
  1,024. All assertions were restored.
- `memory::an_intake_with_a_large_capacity_used_lightly_holds_what_it_holds`:
  capacity 1,048,576 → 16,384; turns 1,000,000 → 16,384. Capacity is still sixteen
  ceilings, checked at compile time, so allocating it eagerly fails the first step.
  One constant supplies capacity, meter label and turns: two bytes per turn pass
  twice capacity through `append(...).expect("room for two bytes")`, rejecting
  lifetime-throughput accounting as before. With `Read::Fill(1)` and the comparison
  adjusted to the alternating delivered byte, one byte remained each turn: the meter
  failed at 1,536 bytes against 1,024. Separately, setting the constant to 1,024
  failed compilation with E0080 at the sixteen-ceilings assertion. Both restored.
- `memory::a_queue_grows_to_its_capacity_exactly` and
  `memory::a_list_filled_to_its_capacity_stays_within_its_price` (the median pair):
  counts unchanged; both push entry points and the fill/copy/clear/move checks stay.
- `memory::a_request_table_grows_within_its_held_and_peak_bounds` (next-costliest):
  counts unchanged; its nine capacities, restored/new paths and lifecycle checks stay.
- `memory::a_queue_filled_to_its_capacity_stays_within_its_price`: counts unchanged;
  four item sizes at nine capacities still fill, refuse and wrap.
- All other profiled tests: counts and capacities unchanged.
