+++
id = "machine"
part = "machine"
kind = "migration"
needs = ["mmdebstrap-orders"]
files = ["hearth/crates/hearth-protocol-mmdebstrap/**", "hearth/tests/mmdebstrap/**", "hearth/scripts/old-forms.txt"]
effort = "high"
minutes = 30
+++

# The mmdebstrap machine, uncapped

## Goal

`hearth-protocol-mmdebstrap` is the second machine off the old form. Its `MaxOut` and `UP_MAX_OUT` go, the queues beside its machines are uncapped, and
only the edges refuse. What hearth does doesn't change.

Acceptance:

- `git grep -nE 'MaxOut|MAX_OUT' -- hearth/crates/hearth-protocol-mmdebstrap hearth/tests/mmdebstrap` prints nothing, and neither does
  `git grep -n 'UP_MAX_OUT' -- hearth/crates/hearth-protocol-builder`.
- `git grep -n 'Queue::with_capacity' -- hearth/crates/hearth-protocol-mmdebstrap hearth/tests/mmdebstrap hearth/crates/hearth-protocol-builder/src/resolve.rs hearth/crates/hearth-protocol-builder/src/indices.rs`
  prints one line, `Tail::new`'s, and `git grep -n '\.room()' -- hearth/crates/hearth-protocol-mmdebstrap` prints one, `admit`'s.
- `hearth/tests/mmdebstrap/tests/suite/orders.rs` and `generated_failures_keep_the_reference_s_tail_and_cause` pass unchanged.
- `hearth/scripts/old-forms.txt`'s `max_out` falls by 13, with `--write`.
- Every test of `hearth-protocol-mmdebstrap`, `hearth-mmdebstrap-world`, `hearth-protocol-builder` and `hearth-builder-world`'s focused suite passes, and
  `hearth-conformance-world`, `hearth-render-world`, `hearth-service-builder` and `hearth` build their tests.

## Inputs

- `hearth/docs/plan/hearth-memory-3/notes/common.md`
- `skein/docs/foundation/programming-model.md`, 6.1, 6.3, 7.1 and 7.2
- `hearth/docs/design/domain/render.md`, 5.3 and 8.5
- `skein/crates/skein-lib/src/queue.rs`: `Queue::uncapped`, whose first push takes four items' room
- `skein/docs/foundation/testing-strategy.md`, 6, for a ceiling

## Owns and may change

- **Owns:** the files under `files`.
- **`hearth/crates/hearth-protocol-builder/src/resolve.rs` and `indices.rs`:** the lines of `## Steps`, 4, and nothing else. `indices.rs`'s `self.intake.room()` stays:
  it's a capacity, and D001's table keeps it. `client.rs`'s and `read.rs`'s `Decode` queues stay: those are the config and records machines', later rounds'.
- **Nothing else reads the constants.** A survey of the trunk found one reader outside the crate and its world, `indices.rs`'s loop, so `hearth-service-builder`
  should need no change. If it does, make the smallest change that keeps it compiling and behaving as it does, and name in the commit and the report what the
  service round must finish.
- **`tail.rs` doesn't change.** `notes/common.md`, "What doesn't move".
- **The bites stand.** `suite/orders.rs`'s queues were uncapped from the start, and `mmdebstrap-orders` broke each rule, so this task doesn't break them again.
- **Ceilings, pre-approved:** the uncapped queues' first push takes more. If `an_output_reader_at_its_limits_holds_no_more_than_its_ceiling`,
  `a_stanza_reader_at_its_limits_holds_no_more_than_its_ceiling` or a ceiling in `hearth/tests/builder/tests/suite/memory.rs` is passed, measure the peak, set the ceiling a
  quarter above it rounded up to two significant digits, and give the old peak, the new one and the queues that moved it in the commit. A ceiling that isn't passed stays.
  Any other test that fails is a finding first: read it before changing anything.
- **If the plan's order proves wrong,** so that the service must move first, stop and ask: a `Question:`, with what forced it.

## Steps

1. **The constants.** Remove `MaxOut` from `lib.rs`, and `UP_MAX_OUT` and the `MaxOut` import from `output.rs` and `packages.rs`.

2. **The crate's tests.** `src/tests/fixtures.rs`: all four queues in `opened` and `opened_list` to `skein_lib::Queue::uncapped()`. `src/tests/packages.rs`: each
   `for _ in 0..packages::UP_MAX_OUT.above` to `for _ in 0..above.len()`.

3. **The world.** `hearth/tests/mmdebstrap/src/world.rs`: every `Queue::with_capacity` in `read_output`, `read_output_metered`, `read_index` and `read_index_metered`
   to `Queue::uncapped()`. Their pops stay as they are.

4. **The queues beside the machines,** each to `Queue::uncapped()`:
   - `resolve.rs`: `pending` in `Resolve::new`, `requests` in the `Spawned` arm, and `seen` and `requests` in `stream`. The `if let ... .pop()` beside them stay: a
     delivery gives one of each, and `suite/orders.rs` holds that.
   - `indices.rs`: `demands` in the `State::Index` arm, and `found` and `demands` in `read`. `read`'s loop becomes `for _ in 0..found.len()`, its arms as they are.

5. **Run the readers' tests:** this changes what a full queue does, so run the packages of the acceptance, not only the crate. `cargo fmt`; clippy on
   `hearth-protocol-mmdebstrap`, `hearth-mmdebstrap-world` and `hearth-protocol-builder`.

6. `notes/common.md`, "The end of every task", and `heavy cargo nextest run -p fabrig-boundary --test outward`. Commit, with any ceiling's figures and anything the
   service round must finish, and push.

## Report

The first line; the count before and after; each ceiling moved, with its peaks and why, or that none moved; the tests run and how many; anything the service
round must finish.
