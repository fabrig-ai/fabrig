# lib: containers

## 1. Containers

| Container | What | Over |
|---|---|---|
| `Queue<T>` | the records between stages, first in first out | a ring buffer grown as it fills |
| `Lossy<T>` | facts a reader may miss: kept up to a cap, the rest dropped and counted | a capped `Queue` and a count |
| `List<T>` | a sequence a step appends to | a buffer costing what it holds |
| `Map<K, V>` | a table keyed by value, ordered by key | a B-tree, priced in tree nodes |
| `Set<K>` | a `Map` without values | a B-tree, priced in tree nodes |
| `Lru<K, V>` | a table that drops the entry used longest ago, the least key first among equal uses | a pair of B-trees, priced in tree nodes |
| `Stack<T>` | what a parser keeps instead of recursing | a buffer costing what it holds |

Each capped container refuses past its capacity, handing the value back
rather than panicking, with one exception: `Queue::push` on a full capped
queue is a bug, asserted; `Queue::try_push` is the refusing form.
`Queue::with_capacity` keeps its cap but allocates as it fills, growing
exactly and never past the cap. A B-tree allocates as it fills, so its
`price(capacity)` counts its nodes at capacity, not its entries.
`List`, `Queue`, `Intake`, `Map`, `Set`, `Lru`, `Stack` and `Buffer` give this
allocation price through `price(capacity)`.
The shared growth rule for `Queue`, `List`, `Stack`, `Slab`'s lists and
`Intake` grows storage to twice its length (its storage capacity for
`Intake`), at least the needed size and a floor of 4, never past its cap.

`Lru::get` makes an entry the most recently used. `Lru::insert` does too, and
hands back what left: the old value under that key, the entry it evicted, or at
capacity 0 the pair it was given, since a zero capacity keeps nothing. Its use
clock saturates; at the end of it, equal uses still fall to the least key.

`List::from_boxed` makes a list of owned items, its capacity their count,
and `List::into_boxed` moves them into a box of exactly their number.
`List::allocated(capacity)` allocates its storage once, at its capacity,
and never grows: a decoder's list, whose count has passed its limit and
what is left (`skein/docs/design/data/codec/README.md`, 4).
`List::swap_remove` takes one item out, the last taking its place, and
`List::iter_mut` hands each item to change in place.

**`Queue::clear(&mut self)`** drops every item, keeping the capacity and
the grown buffer: refilling to the previous length allocates nothing.

**`raise(&mut self, capacity: u32)`** on `Queue`, `List`, `Map`, `Set`,
`Lru`, `Slab` and `Deadlines` raises the container's capacity to `capacity` in place,
allocating nothing: what it holds stays, storage still grows by the
shared rule as it fills, and a slab's spent slots stay spent and its
handles valid. A capacity below the current one is a bug, asserted, as
a capacity never falls; an equal one changes nothing. It is how a
limit's raise reaches a table already made (temper's
`protocol/configuration/limit-settings.md`, 5).

**`Stack::clear(&mut self)`** drops every item, keeping the capacity and
the grown buffer: refilling to the previous depth allocates nothing.

**`Queue::uncapped()`, `Map::uncapped()` and `List::uncapped()`**
start empty and grow, with `u32::MAX` as their capacity. They hold the
hops, correlations and collections inside a service, never what a peer
can push in. An uncapped queue's `push` cannot fail; its allocation
price is for a chosen capacity, not a bound on the queue.

### 1.1 State digests

A seed replays to the same trace, and to the same digest of each
process's state (../../foundation/testing-strategy.md, section 6). lib gives the hasher,
and every type of its own that a state holds hashes what it holds, never
where it holds it.

- **`StateHasher`** is a `core::hash::Hasher` over SHA-256 (`lib-bytes.md`, 1.2), with
  no key: a digest is a function of the values hashed alone, the same in
  every run, on every machine and every target.
  - `write` hashes the bytes given; `write_u8` to `write_u128` and
    `write_i8` to `write_i128` hash the integer's little-endian bytes at
    its own width; `write_usize` and `write_isize` hash 8 bytes, so a
    32-bit build (`lib.md`, section 2) and a 64-bit one digest one state alike.
    std hashes a slice of `usize` or `isize` whole, through `hash_slice`,
    at the target's width, which no `Hasher` reaches: a state holds them
    as fields or in lib's containers, which hash item by item, and a type
    that keeps a slice of them hashes it item by item, by hand.
  - `digest(self) -> [u8; 32]` ends it. `finish()`, which `Hasher`
    requires, gives the digest's first 8 bytes as a big-endian `u64`,
    without ending it.
  - `state_digest(&value) -> [u8; 32]` hashes one value whole.
- **A state is digested through its types' derived `Hash`.** A service's
  state types derive it, as they derive `Debug`. A field that no later
  step reads, such as a measurement kept for display, may be left out by
  a hand-written impl whose comment says why.
- **lib's types hash what their later answers depend on:**

  | Type | Its `Hash` covers |
  |---|---|
  | `Slab<T>` | each slot in index order: free, with its generation; occupied, with its generation, whether it is retired, and its value; or spent. Then the order in which its free and retired slots will be handed out again |
  | `Map<K, V>`, `Set<K>` | the count, then each entry in key order |
  | `Lru<K, V>` | the count, then each entry from least to most recently used, never its clock |
  | `Queue<T>`, `List<T>`, `Stack<T>` | the count, then each item: a queue front to back, a list in order, a stack bottom to top |
  | `Deadlines<K>` | the count, then each armed deadline, its time and its key, in the order `expire` takes them |
  | `Journal<W, O>`, `RequestTable<O, R>` | their entries in their own order, and the counters, keys and attempts that decide what they emit next |
  | `Intake`, `Buffer` | the bytes it holds, in order |
  | `Id<T>`, `Token`, `Time`, `Duration`, `Wall`, `Civil`, `Secret`, `Rng` | their value |
  | `CryptoRng` | its stream number and how many bytes it has drawn, never its key |

- **Never hashed:** a container's capacity, which its owner fixed, and
  its layout in memory: a B-tree's nodes, a ring's head, a buffer's spare
  room, an address. Two states that would answer every later step alike
  digest alike, however each was reached. A `Slab` hashes all the slots
  it holds in index order, free ones included.
- **An environment retained by a world:** `Env<L>` derives `Hash` where
  `L: Hash`, over its clocks and limits. A step still receives it by
  shared borrow and keeps none (../../foundation/programming-model.md, section 3), but a
  world comparing its complete frozen state can include the environments
  it retains beside its domains. A service comparing only its domains,
  as the echo's does, leaves those environments outside that comparison.
- **No `Hash` for stream views:** `stream::Held` and `stream::Slice`, a
  buffer served as a stream's side below, which the step that makes it
  reads. A state that keeps a `Held` past its step hashes, by hand, what
  of it is left to read.
- **A digest reads the whole state,** so a world asks for one at the end
  of a run, and after each iteration only when it chooses to (../testing/service-tiers.md,
  section 1). No step code calls it.

### 1.2 Joining two maps

A step that compares two versions of one table, such as a part's rows
before and after a change, walks them together by key. Step code has no
closure to hand a generic walk and defines no generic item
(../../foundation/programming-model.md, 10.3), and a merge hand-rolled in
each table is what lib is for (`lib.md`, section 1), so lib joins two of its maps:

```rust
impl<K: Ord, V> Map<K, V> {
    pub fn join<'a>(&'a self, other: &'a Map<K, V>) -> Join<'a, K, V>;
}
// Join<'a, K, V>: Iterator<Item = (&'a K, Joined<'a, V>)>
pub enum Joined<'a, V> {
    First(&'a V),
    Second(&'a V),
    Both(&'a V, &'a V),
}
```

- **Each key of either map comes once, in key order,** with its value in
  the first map alone (`First`), in the second alone (`Second`), or in
  both (`Both`, the first's value, then the second's). A key in both is
  given as the first map's; the two are equal under `Ord`.
- **It holds a cursor on each map and allocates nothing.** Each step
  advances one cursor or both, so a join takes as many steps as the maps
  hold distinct keys. It borrows both maps while it runs.
- **`Joined` is `Copy` for every `V`,** and `Debug`, `PartialEq` and `Eq`
  where `V` is. `Join` and `Joined` are exported from lib's root beside
  `Map`.
- **Step code joins rows it already holds** by building a `Map` for each
  side, keyed by what the rows are compared by and borrowing them (a
  `Map<&[u8], &Row>` holds no copy of a key), then matching each
  `Joined` in a plain `for`:

  ```rust
  for (key, joined) in before.join(&after) {
      let change = match joined {
          Joined::First(row) => Change::Removed { key: bytes::copy_of(key), value: row.value.clone() },
          Joined::Second(row) => Change::Added { key: bytes::copy_of(key), value: row.value.clone() },
          Joined::Both(old, new) if old == new => continue,
          Joined::Both(old, new) => Change::Changed { key: bytes::copy_of(key), before: old.value.clone(), after: new.value.clone() },
      };
      changes.push(change).expect("a change for each key");
  }
  ```

### 1.3 Searching a sorted slice

A slice sorted once by a key that its items hold, and searched many
times, as a revision's listing is by path, keeps its shape: other code
builds it and reads it by position, and a `Map` beside it would hold every
key twice. lib can't read the key from an item (no closure, no trait:
../../foundation/programming-model.md, 10.3), so the search is one its
caller drives: lib keeps the bounds, and the step code compares.

```rust
pub struct Bisection { /* the indices still open, low..high */ }
impl Bisection {
    pub const PROBES: u32 = usize::BITS;
    pub const fn over(len: usize) -> Bisection;
    pub const fn probe(&self) -> Option<usize>;
    pub fn narrow(&mut self, order: core::cmp::Ordering);
    pub const fn end(&self) -> usize;
}
```

- **`over(len)` opens the indices `0..len`.** `probe()` names the next to
  compare, the middle of those open, `low + (high - low) / 2`, or `None`
  once none is open. `narrow(order)` takes how the item at that index
  orders against the key sought: `Less` leaves open the indices above it,
  `Equal` and `Greater` those below it. A `narrow` with none open is a
  bug, asserted.
- **`end()` is the first index whose item is not below the key:** where
  the key is, or where it would go, `len` past the last. Among equal keys
  it is the first. Once `probe()` has given `None`, the step code reads the
  item at `end()` and compares its key for equality; a search for the
  first item not below a bound, as task paths are found after `tasks/`,
  stops there.
- **A search over any length ends within `PROBES` probes:** each probe at
  least halves what is open, so `for _ in 0..Bisection::PROBES` bounds
  the step code's loop.
- **It holds two indices and allocates nothing.** It is `Copy`, and
  derives `Clone`, `PartialEq`, `Eq`, `Hash` and `Debug`.

```rust
let mut search = Bisection::over(entries.len());
for _ in 0..Bisection::PROBES {
    let Some(probe) = search.probe() else { break };
    let entry = entries.get(probe).expect("a probe within the slice");
    search.narrow(entry.key.as_ref().cmp(name));
}
let entry = entries.get(search.end())?;
if entry.key.as_ref() == name { Some(entry) } else { None }
```
