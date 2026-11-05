+++
id = "lru"
part = "lru"
kind = "feature"
needs = []
files = ["skein/crates/skein-lib/src/lru.rs", "skein/crates/skein-lib/src/lib.rs", "skein/crates/skein-lib/src/tests/containers.rs", "skein/tests/lib/tests/suite/memory.rs", "skein/crates/skein-forge/src/forgejo/plans/tables.rs", "skein/crates/skein-forge/src/github/plans/tables.rs", "skein/crates/skein-forge/src/users.rs", "skein/docs/design/lib.status.md", "skein/docs/design/forge.status.md", "skein/docs/development/debt/D006-lru-tables.md"]
effort = "high"
minutes = 50
+++

# skein: one bounded least-recently-used table in lib, and the forge's tables on it

## Goal

Pay `skein/docs/development/debt/D006-lru-tables.md`. `skein_lib::Lru` is one priced container, and both dialects'
repository tables and their logins' table stand on it.

Acceptance:

- `rg -c 'fn tick\(&mut self\) -> u64' skein/crates` finds nothing.
- `rg -n 'used|oldest' skein/crates/skein-forge/src/forgejo/plans/tables.rs skein/crates/skein-forge/src/github/plans/tables.rs
  skein/crates/skein-forge/src/users.rs` finds nothing about use order.
- `Tables`' public methods keep their signatures in both dialects, and every test of theirs passes unchanged:
  `a_full_table_drops_the_entry_used_longest_ago`, `a_full_user_table_keeps_the_most_recent_login_and_replaces_in_place`,
  `github_plans_tables_evict_least_recently_used`, `logged_fill_keeps_its_row_recent_for_table_eviction`.
- The memory ceilings of `skein-forge-machine-world` (`full_*_holders_stay_under_the_ceiling`) pass unmoved. The second tree
  costs a few nodes in tables of at most 16 rows. If one fails, stop and ask, with the measured peak.
- Production lines fall across lib and forge together, `Lru` included. Cap for `lru.rs`'s production: 75 lines.
- The item is deleted, and `lib.status.md`'s `5: Lru` and `forge.status.md`'s `4.2:` entry about the tables leave.
  `lib.status.md`'s `8:` and `forge.status.md`'s `4.4:` are another task's.

## Inputs

- `temper/docs/plan/debt-17/notes/common.md`
- `skein/docs/development/debt/D006-lru-tables.md`
- `skein/docs/design/lib.md`, 5 and 5.1 (this cut wrote `Lru`); `skein/docs/design/forge.md`, 4.2 and 4.4;
  `skein/crates/skein-lib/src/deadlines.rs`, the precedent: a `Set` of `(order, key)` beside a `Map`

## Owns and may change

- **Owns:** the files in this spec's header.
- **May change, an export line only:** `skein/crates/skein-lib/src/lib.rs`, with its header doc's list.
- **Two deliberate changes, neither reachable:** Forgejo's `Tables` at a repository capacity of 0 kept panicking on
  `"room after eviction"`, and keeps nothing now, as GitHub's and `Users` do; `skein-forge-connection` asks for 0 only with no
  forge of the dialect, and refuses it otherwise. Forgejo's clock panicked at 2^64 uses, and saturates now. Say both in the
  commit. Each table has its own clock now where a `Tables` had one: the order within a table doesn't move.
- `RepositoryEntry` and `UserEntry` go. Nothing outside these files names them, in any tree: check, and if something does,
  stop and ask. `pub use crate::users::UserEntry` goes with them.
- `users.rs` stays as the root's module that `forge.md`, 4.4 and `forge-github.md` name: `pub type Users = Lru<(u16, u64), Box<[u8]>>`,
  with its doc line. If an alias reads worse in a caller than the plain type, keep the module and say so in the report.
- `skein/docs/design/forge-github.md` is read by tests; it needs no change, and this task does not touch it.

## Steps

1. `skein-lib/src/lru.rs`:

   ```rust
   /// A bounded table that drops the entry used longest ago (lib.md, 5).
   #[derive(Debug)]
   pub struct Lru<K, V> { by_use: Set<(u64, K)>, by_key: Map<K, (u64, V)>, clock: u64 }

   impl<K: Ord + Copy, V> Lru<K, V> {
       #[must_use] pub fn with_capacity(capacity: u32) -> Lru<K, V>;
       #[must_use] pub fn price(capacity: u32) -> Option<u64>;
       #[must_use] pub fn len(&self) -> u32;
       #[must_use] pub fn is_empty(&self) -> bool;
       /// The value under `key`, now the most recently used.
       pub fn get(&mut self, key: K) -> Option<&V>;
       /// Keeps `value`, most recently used; hands back what left (lib.md, 5).
       pub fn insert(&mut self, key: K, value: V) -> Option<(K, V)>;
       pub fn remove(&mut self, key: K) -> Option<V>;
   }
   impl<K: Hash + Ord, V: Hash> Hash for Lru<K, V>;  // lib.md, 5.1
   ```

   The clock uses `saturating_add`. Nothing else goes on it unless a caller needs it.

2. Tests, in `skein-lib/src/tests/containers.rs` as tables: eviction by use; a `get` that saves an entry; replacing evicts
   nothing and hands back the old value; `remove`; capacity 0 keeps nothing and hands the pair back; `Hash` alike however an
   order was reached, and unlike for another order. The exhausted clock needs the private field: an inline `#[cfg(test)] mod
   tests` in `lru.rs`, setting it near `u64::MAX`, shows equal uses falling to the least key. Break each of the use order,
   the capacity 0 and the saturation by hand.

3. `skein/tests/lib/tests/suite/memory.rs`: an `Lru` case beside `Deadlines`'s, holding `price` against the counting
   allocator over the same capacities.

4. Both `plans/tables.rs` hold `repositories: Lru<api::Repository, Repository>` and `users: Users`, with no clock; each
   method is one line. Their module docs cite `forge.md`, 4.2, for what's cited as 3.5.

5. `heavy cargo nextest run -p skein-lib -p skein-lib-tests -p skein-forge -p skein-forge-connection -p skein-forge-machine-world`,
   then `heavy cargo check --tests -p skein-fake-forge-github -p skein-fake-peers`, and the suites of `skein/tests/forge/github`,
   named by its manifest.

6. The status lines, then delete the item. `cargo fmt`; `python3 -I -B skein/scripts/old_forms.py`; `heavy cargo clippy -p
   skein-lib -p skein-lib-tests -p skein-forge --all-targets -- -D warnings`; the citations check. Commit and push.

## Report

The common note's items, what you broke by hand and what failed, and the three memory peaks if your run prints them.
