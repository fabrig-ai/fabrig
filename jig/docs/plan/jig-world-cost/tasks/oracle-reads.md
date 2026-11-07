+++
id = "oracle-reads"
part = "oracle-reads"
kind = "feature"
needs = []
files = [
  "jig/testing/jig-core-kit/src/oracle.rs",
  "jig/testing/jig-test-store-files/src/store/readback.rs",
  "jig/testing/jig-test-store-files/src/store/tests.rs",
  "jig/crates/jig-protocol-store/src/pages/reader.rs",
  "jig/crates/jig-protocol-store/src/tests/readers.rs",
]
effort = "medium"
minutes = 50
+++

# The restart oracle reads the store twice where it read it four times, and a walk asks for pages

## Goal

`World::close` reads the whole store four times: `retained()` and `readback()` on each side of the restart, and `readback()` begins with its own `retained()`. And `walk_map` loads one row at a time, then asks again past it. Reuse what a read already holds, and ask the store for a page.

**"Every story ends with a cold restart" (`jig/docs/design/domain/engine.md`, 15) stays, and no story opts out.** This is its cost, not its place.

The two parts stand apart. If the page load can't keep what's below, land the first part alone and say why.

Acceptance:

1. `close` makes one `retained()` walk on each side of the restart. Each side's `readback` reads the inventory it was given. Every comparison in `close` is unchanged.
2. `Store::readback()` still exists for its other readers, with the same result.
3. `walk_map` loads pages. For every input it returns the same rows, the same cursors and the same failures as before, and `a_walk_by_pages_matches_a_walk_by_rows` shows it against the row-by-row walk kept in the test.
4. Every step test of `jig-protocol-store` passes unchanged, and every bound stays.
5. `component_limits` is unchanged.

## Inputs

- `jig/docs/design/domain/engine.md`, 6 and 15.
- `jig/docs/design/protocol/store.md`, 7.1 and 14. **That file is read by tests:** this task doesn't change it, and 7.1 stays true as it stands.
- skein's `kv.md`, section 2 (`skein/docs/design/kv.md`).
- `jig/docs/plan/jig-world-cost/notes/common.md`.

## Owns and may change

- **Owns:** the spec's `files`. **This is the plan's only task that edits `oracle.rs`.** `model-catalogue-2` may touch its `durable`, so stay away from that function.
- **Readers, read at the cut:**
  - `readback()`: `oracle.rs` and `jig-test-store-files/src/store/tests.rs`, which keep calling it.
  - `retained_page`: `readback.rs` and `jig-protocol-store`'s own tests.
  - `walk` and `walk_map`: `loads.rs`, `retained.rs`, `restore/reader.rs`, `patching/check.rs`, `patching/phase/walking.rs`, `patching/plan.rs`, `reads/erasure.rs`, `reads/paged.rs`, `reads/indexed.rs`, `reads/settings.rs`. Their signatures don't change, so none of them is edited.
  - `load_map`: `pages::load`, whose readers (`transcripts/cut.rs`, `transcripts/drop.rs`, `tests/open.rs`) read one row. **`load_map` keeps its one-row meaning.**
- **`jig-protocol-store` is step code:** the strict subset holds (`AGENTS.md`, 2): no unchecked arithmetic, no indexing, no `as`, no `_`.
- **`retained.rs` needs no change.** If you find a reason to change it, it's yours, and say so.
- **`component_limits`' `page: skein_kv::Page { rows: 2, .. }` stays.** The oracle already asks for the limit: `retained()` passes `component_limits.store.page`, and `retained_page` clamps to the same figure, so its pages are as large as the limits allow. Two rows is what makes each story cross pages (`store.md`, 14). In these worlds a load therefore holds at most two rows, so the walk's saving there is smaller than the oracle's: expected, not a shortfall.
- **Options:**
  - If `retained_page` can reuse a clone cheaply inside its loop, do it. Otherwise leave it.
  - If a store or world memory test goes red over the page a load holds, read the log. A bound that's truly passed means you leave `walk_map` alone and say why. Never raise a ceiling.
  - If the page load can't keep a byte, a cursor or a failure, the same: leave it and say why.

**Why a page load keeps them, read at the cut:** `skein_kv`'s `Map::page` stops at `max.rows` or `max.bytes` and returns the first key it didn't return as `next`. It refuses `TooLarge` only when the *first* row doesn't fit. So a row too large for a page, reached later in a load, comes first in the next load and is refused there, as today. `load` refuses `max.rows` above `limits.page.rows`, so the ask is capped at it.

## Steps

1. **`readback.rs`:** split `readback` without changing its body.

   ```rust
   pub fn readback(&mut self) -> BTreeMap<root::Key, root::Record> {
       let inventory = self.retained();
       self.readback_of(&inventory)
   }
   /// Read back an inventory `retained` just returned, with nothing changed between.
   pub fn readback_of(&mut self, inventory: &BTreeMap<Box<[u8]>, Box<[u8]>>) -> BTreeMap<root::Key, root::Record>
   ```
2. **`oracle.rs`, `close`:** `let records = self.store.readback_of(&inventory);` and `let recovered = self.store.readback_of(&restored);`. Nothing else in `close` changes. A test in `store/tests.rs`: `readback_of` agrees with `readback`.
3. **`reader.rs`:**

   ```rust
   pub(crate) fn load_map(store: &mut Store, map: KeptMap, range: Range) -> Result<pages::Page, Failure> {
       load_rows(store, map, range, 1)
   }
   fn load_rows(store: &mut Store, map: KeptMap, range: Range, rows: u32) -> Result<pages::Page, Failure>
   ```

   In `walk_map`, each load asks for the rows still wanted and one more, because the walk must see the row past a full page before it returns `next`: `capacity.saturating_sub(found.len()).saturating_add(1).min(limits.page.rows)`. Then **every row of the page goes through the same per-row steps, in the same order** as today's single row: `row_bytes`, the `found.len() == capacity` and byte-total cut, `last`, `past` with its failure at that row's site, `found.push`. `page.next.is_none()` ends the interval once the page's rows are done. Rows the cut leaves unread are dropped, as the next call asks again from the cursor. Keep the `for _ in 0..=capacity` bound on loads.
4. **`tests/readers.rs`:** `a_walk_by_pages_matches_a_walk_by_rows`. Keep today's loop in the test as a naive `walk_by_rows`, built on `load_map`. Over `raw_store` fixtures, for a few values each of `rows` (0, 1, 2, 3, and more than the store holds), the map's `page.rows` (1, 2, 8) and `page.bytes` (a cut between rows, and one row's), one interval and two with a gap, an empty interval, and each cursor the walk returned: same rows, same cursors, same `next`. A row too large for the page gives the same `Failure`. Keep the table to rows that can fail on a rule. Break it once by hand (drop the `+ 1`, or stop per-row checks at the page's end) and report what it showed.
5. Run narrow tests, then the implementer's closing steps. This is product code: run the focused tests of `jig-protocol-store`, `jig-test-store-files`, `jig-store-world`, `jig-core-kit` and `jig-core-world`, and the fuzzy test of `jig-store-world` if it has one (`--profile fuzzy -p jig-store-world`). Commit and push.

## Report

The implementer's report, plus: whether `walk_map` landed and, if not, why; the rule you broke and what the test showed; anything `close` still reads twice and why.
