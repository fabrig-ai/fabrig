+++
id = "store-step"
part = "orders"
kind = "feature"
needs = []
files = ["hearth/crates/hearth-protocol-builder/src/lib.rs", "hearth/crates/hearth-protocol-builder/src/tests/resolve.rs"]
effort = "high"
minutes = 25
+++

# The store step gets its named predicate

## Goal

`hearth-protocol-builder`'s `up` reads `!above.is_empty()`, of the whole queue, and means "this store step answered". It holds today because `calls` is usually empty there, and it goes wrong once `calls` is uncapped and holds earlier events. It becomes the named predicate of `hearth/docs/design/domain/render.md`, 8.5, with a test that shows the difference.

Acceptance:

- `up` calls `store_answered`, in the form `notes/contracts.md` gives, and `git grep -n 'above.is_empty()' -- hearth/crates/hearth-protocol-builder/src/lib.rs` no longer lists that line. The `!above.is_empty()` of `Mode::Listing` and `Mode::Hunks` in `work_pending` read a machine's own queue, and stay.
- A new test, `a_store_event_that_answers_nothing_leaves_the_index_unsettled`, passes and bites (below).
- `orders::what_a_call_prints_and_stores_does_not_depend_on_its_queues` passes as it stands: its resolving row checks that a kept index's terminal store answer comes before the next index opens.
- Every test of `hearth-protocol-builder` and `hearth-builder-world`'s focused suite passes.

## Inputs

- `hearth/docs/plan/hearth-memory-6/notes/common.md`
- `hearth/docs/plan/hearth-memory-6/notes/contracts.md`, "`hearth-protocol-builder/src/lib.rs` after `store-step`"
- `hearth/docs/design/domain/render.md`, 8.5: the store step's order
- `hearth/docs/development/debt/D001-room-checks-and-max-out.md`, "The service-order search", its last paragraph, and "What the service round must finish"
- `hearth/crates/hearth-protocol-builder/src/indices.rs`: `waiting`, `arrival` and `settled`
- `hearth/crates/hearth-protocol-builder/src/store.rs`: `Store::up`, where a keep's `Renamed` and `Synced` ask for the next step and give nothing above

## Owns and may change

- **Owns:** the two files under `files`, and in `lib.rs` only `up` and the new function. `debt-26` changes other hunks of `lib.rs` (`Current`'s accessors and `finish`), and a wider edit costs a wider merge.
- **Reader files, pre-approved:** `hearth/crates/hearth-protocol-builder/src/tests/fixtures/indices.rs` or `src/tests/fixtures.rs`, to add a small helper the test needs, and nothing else there. `Layer`'s queues stay at capacity 1.
- **If the test can't reach that state:** a test-owned `Queue::uncapped()` for `above`, with `up` called directly, should do, as tests in this crate have the private fields. If the keep's steps can't be reached in the crate's tests at reasonable size, write the smallest state that shows the difference, and say so in the report. A test that can't bite is a `Question:`.

## Steps

1. Add `store_answered` and use it in `up`.
2. In `src/tests/resolve.rs`, the test, through `Protocol` in `Current::Resolving` so that `up` itself runs, with an index at `State::Arrival` (`a_kept_index_is_synced_renamed_across_then_cache_synced_then_recorded`, in `src/tests/indices.rs`, shows the way to that state, and `Layer::resolve` in `src/tests/fixtures.rs` to the resolving one):

   ```rust
   #[test]
   fn a_store_event_that_answers_nothing_leaves_the_index_unsettled() {
       // `above` already holds an earlier event, as an uncapped `calls` can.
       // A keep's `Renamed` asks for its sync and gives nothing above.
       // The index stays at its arrival: `arrival()` is still `Some`, and no next index opens.
   }
   ```

3. **Bite:** put `!above.is_empty()` back in `up`, see this test fail because the index settled, and restore it. Then make `store_answered` always true, see `orders::what_a_call_prints_and_stores_does_not_depend_on_its_queues` fail on its two milestones, as D001 says it does, and restore it. If that second one doesn't fail, it's a finding: report it.
4. `heavy cargo clippy -p hearth-protocol-builder --all-targets -- -D warnings`, the tests above, and `heavy cargo check -p hearth-service-builder --tests`.
5. Commit and push.

## Report

The rules broken and the failures seen, what ran, the CPU seconds of the new test, and what the second bite showed.
