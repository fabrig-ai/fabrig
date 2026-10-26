# Session 01: given keys in skein-lib's request table

skein's `docs/design/lib.md` 12 (landed at skein 52d6750) lets an asker give a request's key, a digest of a name say, and
asks the table to admit it only while no entry holds it. It also asks every refusal to give its reason, `Full` or `Held`,
handing back what it was given, and every admission to have a number, returned with its key, which `not_kept` names, so that
a stale "not kept" never retires a later holder of the same key. Today `RequestTable` draws every key from its seed, refuses
by handing back the bare request or record, numbers its admissions only in the private `serial`, and retires by key alone. 01.1
types the refusals, returns each admission's number and makes `not_kept` take it; 01.2 adds the given key. The downstream
repository's tools plan builds its send under a named key on both, after its own repin onto this session's landing. Two
increments, in order; each lands alone.

## Read first

- **skein:** `AGENTS.md`; its gate `scripts/check.sh`; `docs/design/lib.md` 12 (the bullets "Keys", "Not kept" and
  "Admission and memory", and the closing paragraph); `docs/design/testing.md` 6 (where step tests and worlds go).
- **skein foundation:** `programming-model.md` 6.3 (the worst case), 7 (admission), 10.2 and 10.3 (generic types, no
  closures, exhaustive matches); `testing-strategy.md` 2.1 (step tests), 2.2 (domain worlds), 8 (two suites), 10 (a test's
  shape).
- **Plan:** the README's "Rules of this plan" (the check that skein names no downstream repository) and
  `code-organization.md` (the Don't list, and the command that counts an increment's production lines).
- **Code:** `crates/skein-lib/src/requests.rs` (`RequestTable`: `ask`, `restore`, `not_kept`, `reclaim`, and the private
  `serial`, `order`, `index`, `fresh_key` and `insert`); `crates/skein-lib/src/token.rs` (`Token`, the newtype mirrored);
  `crates/skein-lib/src/tests/requests.rs` (the table's step tests and their fixtures `Owner`, `owner`, `limits`,
  `FIRST_OWNER`, `SECOND_OWNER`, `live`, `sent`); `tests/lib/src/request_table.rs` (the obligation ledger's world: `World`,
  `Obligation`, `Terminal`, `Coverage`, `check`).

## What the design asks

- **lib.md 12, "Keys":** a key is drawn from the seed or given by the asker; a given key is admitted while the table holds no
  entry under it; an entry holds its key from its admission to its reclaim, retired or not; a restored request keeps its
  key. 01.2 builds the given key; the draw is unchanged.
- **lib.md 12, "Admission and memory":** `ask` and `restore` refuse with a reason, handing back the request and changing
  nothing: `Full` (past the held-request count or the declared bytes, or no key left to draw) or `Held` (the key is held);
  "A restore of a held key is refused as `Held`". Each admission, by `ask` or by `restore`, has a number, returned with the
  key, and numbers increase with each admission. `not_kept` names the key and the number; every other call stays by key.
  01.1 builds all of it.
- **lib.md 12, "Not kept":** an admission number earlier than the one the key holds hands back nothing. 01.1 makes
  `not_kept` take the number; 01.2's tests show it for a later holder of the same given key.
- **lib.md 12, closing paragraph:** generic types and lib's own containers; no traits, closures or `dyn`. The new types are
  plain data; no field is added to an entry or to the table, and `worst_case` is unchanged (programming-model.md 6.3).

## Choices this plan makes

The design names neither the given key's entry point, the refusal's type nor the number's type; this plan chooses them.
- **`RequestNumber(u64)`,** a newtype as `Token` is, with `new` and `raw`, so that an asker can carry it through its own
  store's report and back. It is `serial` after the admission: a table's first admission is 1. The table alone counts:
  `RequestRecord` does not carry it, and a table built at a restart numbers its restored records from 1 again.
- **`RequestAdmission { key, number }`,** plain data, returned by every admission, by `restore` too (its key is its
  record's).
- **`RequestRefused<T> { Full(T), Held(T) }`,** generic as `RequestAnswered<O>` is: `RequestRefused<R>` from `ask` and
  `ask_given`, `RequestRefused<RequestRecord<R>>` from `restore`. No method hands back the value whatever the reason: a
  caller matches both arms.
- **`ask_given`,** a method of its own beside `ask`, taking the key after the size. `ask` keeps its draw and its arguments;
  one method taking `Option<RequestKey>` was not chosen, since a drawn key is never `Held`.
- **`Held` before `Full`:** a held key is refused `Held` whatever the room. lib.md 12's "A restore of a held key is refused
  as `Held`" is read as unconditional, and kept for a given key: the asker learns at once that its key is taken. Otherwise
  `Full`, for each limit refused today: the count and the bytes (`fits`), the admission numbers (`serial` at `u64::MAX`),
  the attempt tokens (`ask` and `ask_given`, as `ask` today) and, for `ask`, no key left to draw (`fresh_key`).
- **`not_kept` retires only under the number its key holds now:** the key's entry (`index`) must be the entry `order` holds
  under that number. An earlier number, or any other, hands back nothing: numbers are never reused within a table, so no
  other number can name the key's holder. `order` is keyed by the admission's number already; `Entry` gains no field.

## Rules that bind it

- skein names no repository downstream of it, in code, comments, tests or commit bodies: the README's "Rules of this plan"
  holds the check. This file says "the downstream repositories".
- Every match over `RequestRefused`, `State`, `RequestOut` or the world's `Terminal` is an exhaustive `match`: skein's
  `clippy.toml` disallows `core::matches` and closure combinators (`Result::map` among them), `wildcard_enum_match_arm` is
  denied, in tests too; no lint exception in tests.
- `docs/design/STATUS.md` is not this session's: the designer keeps it and clears lib.md 12's item when the plan ends, and
  the plan's row is the planner's, then the supervisor's (development.md 3.3).
- `ask`, `restore` and `not_kept` change in place, with every user in skein in the same increment (the plan's Don't list).

## Increments

### 01.1 Typed refusals, and each admission's number
Scope: lib.md 12, "Admission and memory" and "Not kept": `ask` and `restore` refuse `Full` or `Held`, handing back the
request or the record and changing nothing; each admission's number returned with its key; `not_kept` by key and number.
Needs: nothing (skein's `main`).
Files: `crates/skein-lib/src/requests.rs` (the three types; `ask`, `restore`, `not_kept`, `insert`; the module doc and its
table); `crates/skein-lib/src/lib.rs` (the `requests::` re-export gains `RequestAdmission`, `RequestNumber` and
`RequestRefused`). The users, all in skein: `crates/skein-lib/src/tests/requests.rs` (34 `ask`, 8 `restore` and 6
`not_kept` calls at b4c6241; it declares `mod admissions;`); new, `crates/skein-lib/src/tests/requests/admissions.rs`;
`crates/skein-lib/src/tests/digest.rs` (`a_request_table_s_digest_changes_with_each_request_asked` compares two `ask`
results, and compiles unchanged); `tests/lib/src/request_table.rs`; `tests/lib/tests/memory.rs` (`request_table_memory`:
two `ask` calls and one `restore`). No hotspot. Downstream, the client's domain crate calls `ask`, `restore` and
`not_kept`: it moves at its own repin, in the downstream repository's tools plan, not here.
Shape: the types as sketched below. `insert` returns the `RequestNumber` it files in `order`. `ask` refuses
`Full(request)` wherever it returns `Err(request)` today, and returns `Ok(RequestAdmission { key, number })`. `restore`
refuses `Held(record)` when `contains(record.key)`, checked first, then `Full(record)` past `fits` or at
`serial == u64::MAX`, and returns its admission. `not_kept(key, number)` takes the entry's id from `index` as today, then
hands back `None` unless `self.order.get(&number.raw())` is `Some(&id)`; the rest is today's. The module doc says that each
admission is numbered and each refusal says why; its table gains `| absent | ask or restore past a limit | absent | refused
Full, the value handed back |` and `| any held state, retired included | restore of its key | unchanged | refused Held, the
record handed back |`, and the not-kept row's event becomes "not kept, with its admission's number"; a row says that another
number changes nothing. The table's `Hash` is unchanged.
Test shape: the new tests in `crates/skein-lib/src/tests/requests/admissions.rs`, a child of `tests/requests.rs`, so that
they use its fixtures by `super::` and that file stays below 800 lines (626 at b4c6241). The reference test:

```rust
//! Each admission's number and refusal's reason, and keys given by the asker (lib.md, 12).

use super::{FIRST_OWNER, Owner, SECOND_OWNER, limits, owner};
use crate::{RequestKey, RequestNumber, RequestRecord, RequestTable};

/// The scope `live` confirms.
const SCOPE: u64 = 7;
/// One declared byte, of the ten `limits` admits.
const SIZE: u64 = 1;
/// A record's key, as the asker's store hands it back at a restart.
const RESTORED: RequestKey = [42; 16];

#[test]
fn each_admission_by_ask_or_restore_is_numbered_one_above_the_last() {
    let mut table = RequestTable::<Owner, u64>::new(&limits(), 1);
    let first = table.ask(owner(FIRST_OWNER), SCOPE, SIZE, 10).expect("fits");
    let record = RequestRecord { key: RESTORED, scope: SCOPE, first_sent: None, request: 11 };
    let restored = table.restore(Owner::Restored(RESTORED), SIZE, record).expect("fits");
    let second = table.ask(owner(SECOND_OWNER), SCOPE, SIZE, 12).expect("fits");
    assert_eq!(
        [first.number, restored.number, second.number],
        [RequestNumber::new(1), RequestNumber::new(2), RequestNumber::new(3)],
        "admissions are numbered from 1, by ask and by restore alike"
    );
}
```

Beside it: `a_restore_of_a_held_key_is_refused_held_whatever_the_room` (limits of one request: a second restore under the
held key gives `Err(RequestRefused::Held(record))`, `len()` 1); `a_restore_past_the_count_is_refused_full_with_its_record`;
`not_kept_under_another_admission_s_number_hands_back_nothing` (two asks; each key under the other's number gives `None`,
and the next take still gives both `Save`s). The existing tests follow the new types: a bound key is the admission's
`.key`; `seeded_keys_are_unique_across_thousands_of_asks` compares whole admissions (`Ok(admission)`); the four `ask`
refusals expect `Err(RequestRefused::Full(..))`; `restoration_keeps_keys_and_waits_for_link_scope_and_retention` expects
`Err(RequestRefused::Held(record.clone()))`; each `not_kept` passes its admission's number. The world
(`tests/lib/src/request_table.rs`, at most 800 lines, 635 at b4c6241): `Obligation` gains `number`, and `World` the last
number admitted, `last: Option<RequestNumber>`, cleared by `restart`; every admission, by `World::ask` or by `restart`'s
`restore`, asserts its number above `last`; `World::ask` matches `Full(returned)` as today's `Err` and `Held(_)` as
`unreachable!("a drawn key is never held")`. A new action, `World::not_kept(pick, other)`: the held obligation at `pick`,
under its own number or the number of the one at `other` (with none held, nothing). The ledger expects its owner exactly when the obligation has no
terminal and no expiry witness (the ledger's mark that the table's `fire` retired it) and the number is its own; it then
sets `terminal` to a new `Terminal::NotKept` and `attempt` to `None`, and the existing checks carry the rest: no save, its
erase, no progress, a stale answer for its attempt. `check`'s draw gives the action slot 10, today's no-op, which moves to
slot 11 (`rng.below(12)`). `Coverage` gains `not_kept` (owners handed back) and `ignored` (a held key under another number),
each above 0 at 64 rounds. `memory.rs`'s calls take the admission's `.key`; its measure is unchanged. Ceilings: each unit
test under 20 ms; the world's focused test under 0.1 s (0.04 to 0.05 s at b4c6241), its fuzzy twin under 2 s (1.2 s).
Proof: world, `request_table_follows_the_obligation_model_and_replays` (tests/lib, focused) and its 4,096-round twin in the
fuzzy suite: the ledger, independent of the table, sees every admission numbered above the last, every refusal `Full`
exactly where count or bytes do not fit, and a not-kept call hand back its owner only under its admission's own number;
under 0.1 s and 2 s, in the gate. Nothing above the table exists in skein: the downstream repository's tools plan
connects it, at its repin onto this landing.
Don't: change `RequestRecord`, `RequestOut`, `RequestAnswered`, `RequestProgress`, `RequestLimits` or `worst_case`; add a
field to `Entry` or to `RequestTable`; add `ask_given` (01.2); change `answered`, `fire` or `reclaim`.
Size: at most 80 lines of production code (`requests.rs`, `lib.rs`), the users' tests aside.
Removes: `ask`'s `Result<RequestKey, R>`, `restore`'s `Result<(), RequestRecord<R>>`, and `not_kept(key)` by key alone.

### 01.2 A key given by the asker
Scope: lib.md 12, "Keys": a request admitted under the asker's key while no entry holds it, the key held from admission to
reclaim, retired or not, and refused `Held` otherwise, before `Full`; "Not kept": a stale "not kept" never retires a later
holder of the same given key.
Needs: the typed refusals and admission numbers, landed (01.1).
Files: `crates/skein-lib/src/requests.rs` (`ask_given` beside `ask`; the module doc and its table; the comment in
`fresh_key`); `crates/skein-lib/src/tests/requests/admissions.rs`; `tests/lib/src/request_table.rs`. No hotspot.
Shape: `ask_given(&mut self, owner, scope, size, key, request)` refuses `Held(request)` when `contains(key)`; then
`Full(request)` past `fits`, at `serial == u64::MAX` or with no fresh attempt, as `ask` does; otherwise it `insert`s
`RequestRecord { key, scope, first_sent: None, request }`, not restored, and returns `Ok(RequestAdmission { key, number })`.
`draws` does not move. `ask`'s draw already skips every held key (`fresh_key`); its comment says "Restored or given keys may
collide". The module doc's table gains `| absent | ask_given of an unheld key | parked or ready | save, then eligible send |`,
and 01.1's held row reads "restore or ask_given of its key".
Test shape: in `admissions.rs`, as 01.1's reference test, with a named given key (`const NAMED: RequestKey = [0xE1; 16];`):
`a_given_key_is_admitted_and_saved_under_that_key` (link down: the admission is `{ key: NAMED, number: 1 }`, the take's
`Save` holds `NAMED`); `a_given_key_is_refused_held_while_its_entry_is_retired` (after a `Final`, before `reclaim`);
`a_given_key_is_admitted_again_once_reclaimed_under_a_later_number`; `a_stale_not_kept_never_retires_a_later_holder_of_its_key`
(`not_kept(NAMED, first.number)` gives the first owner; erase taken and reclaimed; a second `ask_given(NAMED, ..)`; then
`not_kept(NAMED, first.number)` gives `None` and `owner(NAMED)` is the second owner); `a_held_given_key_is_refused_held_before_full`
(limits of one request); `new_keys_skip_a_held_given_key` (a twin table of the same seed shows the first drawn key; given
first, the next `ask` draws another, as `restored_work_waits_for_retention_and_new_keys_skip_held_collisions` does). The
world: `World::ask_given(name, scope, coverage)` asks under one of three given keys (`[0xE1; 16]` with its last byte the
name), each always with the same request, `GIVEN + name` (`const GIVEN: u64 = 1 << 32`, above every drawn request), of size
`request % 4` and owner `Owner::child(request)`. The ledger expects `Held(request)` exactly when `held` has the key, else
`Full(request)` exactly when it does not fit, else the given key's admission, numbered above `last`. `issued` keeps its
no-reuse assertion for drawn keys; a given key's entry carries the same request each time, so `decided` ("one key names one
effect") holds unchanged. `World` keeps, per given key, the number of its last reclaimed admission (`earlier:
BTreeMap<RequestKey, RequestNumber>`, filled by `reclaim`, cleared by `restart`); `not_kept` gains a third choice, that
number, for a held given key that has one, and the ledger expects nothing. `check`'s asks (slots 0 to 3) ask a given key
one time in four (`rng.chance(250)`, the name `rng.below(3)`). `Coverage` gains `given`, `held` and `stale`, each above 0 at
64 rounds. Ceilings as 01.1's.
Proof: world, `request_table_follows_the_obligation_model_and_replays` and its fuzzy twin, with given keys: the ledger sees
a given key admitted exactly while no entry holds it, `Held` told before `Full`, a key restored at a restart refusing its
own name until reclaimed, and an earlier admission's number handing back nothing; under 0.1 s and 2 s, in the gate.
Nothing above the table exists in skein: the downstream repository's tools plan's send under a named key connects it.
Don't: change `ask`, `draws` or `fresh_key`'s bound; move `draws` for a given key; add a type for a given key (it is a
`RequestKey`) or a field to `RequestRecord` saying how its key was made (a restored record keeps its key, drawn or given).
Size: at most 30 lines of production code (`requests.rs`).
Removes: nothing.

## Interface sketches

The table owns the numbers: it counts admissions, never reuses a number, and hands each out once, in its admission. The
asker keeps an admission's number beside its key until its store answers the record's write; `not_kept` is the one call
that takes it, and every other call stays by key. A refusal ends its call: nothing is admitted, numbered or drawn. The sketch
compiles under skein's workspace lints, `clippy.toml` and `rustfmt.toml`, with `clippy --all-targets -D warnings`, in a
scratch crate with stand-ins for `RequestKey`, `RequestRecord` and `RequestTable` and stub bodies.

```rust
// crates/skein-lib/src/requests.rs

/// One admission's number, by `ask`, `ask_given` or `restore`: 1 for a table's first, one more for each after.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct RequestNumber(u64);

impl RequestNumber {
    /// A number from its raw value, as `Token::new` makes a token.
    #[must_use]
    pub const fn new(raw: u64) -> RequestNumber;

    /// The raw value.
    #[must_use]
    pub const fn raw(self) -> u64;
}

/// An admitted request's key, drawn or given, and its admission's number; `not_kept` names both.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RequestAdmission {
    /// The key the entry holds from this admission to its reclaim.
    pub key: RequestKey,
    /// This admission's number.
    pub number: RequestNumber,
}

/// Why `ask`, `ask_given` or `restore` refused; it hands back what it was given, and the table is unchanged.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RequestRefused<T> {
    /// Past the held-request count or the sum of declared bytes, or no key, number or attempt left.
    Full(T),
    /// An entry holds the key, retired or not, until its reclaim; told before `Full`.
    Held(T),
}

impl<O: Copy, R> RequestTable<O, R> {
    /// Admit a request under a key drawn from the seed.
    pub fn ask(&mut self, owner: O, scope: u64, size: u64, request: R) -> Result<RequestAdmission, RequestRefused<R>>;

    /// Admit a request under the asker's own key, while no entry holds it.
    pub fn ask_given(
        &mut self,
        owner: O,
        scope: u64,
        size: u64,
        key: RequestKey,
        request: R,
    ) -> Result<RequestAdmission, RequestRefused<R>>;

    /// Restore a saved request under its own key, with its owner's current value and declared size.
    pub fn restore(
        &mut self,
        owner: O,
        size: u64,
        record: RequestRecord<R>,
    ) -> Result<RequestAdmission, RequestRefused<RequestRecord<R>>>;

    /// Retire the request admitted under `key` as `number`, whose record was not kept; its owner, once.
    pub fn not_kept(&mut self, key: RequestKey, number: RequestNumber) -> Option<O>;
}
```

01.1 lands every item but `ask_given`, which 01.2 adds.

## Questions to raise when reached

None: lib.md 12 answers each observable rule, and the choices above are this plan's.

## Done when

Each in `~/src/rust/skein`, after `git fetch`:
- `git show origin/main:crates/skein-lib/src/requests.rs | grep -q -F 'pub fn not_kept(&mut self, key: RequestKey, number: RequestNumber) -> Option<O>'`
- `git show origin/main:crates/skein-lib/src/requests.rs | grep -q -F 'pub fn ask_given('`
- `git show origin/main:crates/skein-lib/src/requests.rs | grep -q -F -- '-> Result<RequestAdmission, RequestRefused<RequestRecord<R>>>'`
- `git show origin/main:crates/skein-lib/src/requests.rs | grep -q -x -E ' +Held\(T\),'`
- `! git show origin/main:crates/skein-lib/src/requests.rs | grep -q -F -- '-> Result<RequestKey, R>'`
- `test "$(git show origin/main:crates/skein-lib/src/lib.rs | sed -n '/^pub use requests::{/,/};/p' | grep -o -w -E 'RequestAdmission|RequestNumber|RequestRefused' | sort -u | wc -l)" = 3`
