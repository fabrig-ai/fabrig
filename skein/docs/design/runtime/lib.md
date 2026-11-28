# lib

Provisional, 2026-10-03, revised 2026-10-23. The design of `skein-lib`:
the building blocks every step crate shares, in skein and in every
service built on it. The mechanics are those of ../../foundation/programming-model.md;
this document says what lib holds and what each part promises.

## 1. In one page

- **Everything step code needs and the subset does not give it.** Typed
  handles and the slabs that issue them, containers, the join of two maps
  and the search of a sorted slice (`lib-containers.md`, 1.2, 1.3), cursors over
  owned bytes, secrets' bytes, base64 and SHA-256, the stream vocabulary
  and its intake, deadlines, time, randomness, the environment a step
  reads, and the digest of a state that replay compares.
- **Limits at the edges, growth inside.** Containers grow as they fill;
  an edge's container keeps its cap and refuses past it. A capped
  container's `price(capacity)` prices its bookkeeping too; sums
  of those prices are estimates, not process-wide bounds
  (../../foundation/programming-model.md, 6.3). The queues and journal inside a service
  have no cap of their own (`lib-containers.md`, section 1, and `lib.md`, section 11).
- **Written once, tested hardest.** Every layer of every service stands on
  lib. Application code does not hand-roll a data structure: what is
  missing goes into lib.
- **The subset, with a few allowances.** lib is step code, `no_std` with
  `alloc`, with the allowances of ../../foundation/programming-model.md, 10.2: generic
  types, lifetime parameters on its cursors, hand-written impls of std
  traits for its own types, `Vec`, `VecDeque` and B-trees inside its
  containers, and `while` loops bounded by a container's capacity. Where
  a module builds on a type that is out, it says so with a scoped
  `#[expect(clippy::disallowed_types, reason = "…")]`.

## 2. In skein

lib depends on nothing. Every other crate, in skein and in a service,
depends on it. It is the only part of skein a service's domain uses.

**Built for WebAssembly too.** lib, `skein-codec`, `skein-http` and
`skein-ui` build for `wasm32-unknown-unknown` as well as for the host:
none of them depends on io, the shell or a platform's API. A client
compiled to WebAssembly, an application's in a browser over skein's
browser parts (../ui/web/README.md), links lib's containers and base64, the
generated codecs, HTTP's client and the event-stream reader (http.md,
section 2), and skein-ui's tree, view and words (../ui/README.md), and its shell
hands them what its platform streams. Nothing in them assumes a 64-bit
`usize`: a conversion that could fail on a 32-bit target is checked.
skein's gate checks them for that target with the other portable
crates (../testing/README.md, section 10).

**`enum Lifecycle { Live, Closing, Aborting, Closed }`** is the component
lifecycle its owner drives (../../foundation/programming-model.md, 5.2).
`Live` admits new work; `Closing` refuses it as closed, closes what the
component keeps, and lets each held request end with its own terminal;
`Aborting` refuses new work as closed, cancels held requests and aborts
what it keeps. Another close changes nothing; an abort moves a live or
closing component to `Aborting`, where another abort changes nothing.
`Closed` means everything below has settled and the component has
reported its last event, `Closed`, once; its owner sends it nothing more.

## 3. Handles and slabs

- **`Id<T>`** is a slot index plus a generation, typed by the entity it
  names, and `Copy`. It is never reused for another entity: a slot whose
  generation would wrap is never used again. `id.token()` and
  `Id::<T>::from_token(t)` convert to and from the name that crosses a
  boundary.
- **`Slab<T>`** holds one entity kind and grows as it fills, costing
  what it holds. It starts with no slots allocated; growth preserves slot
  reuse order and generations. A kind created from outside has a cap
  from the limits;
  a kind that admitted work makes has no cap of its own
  (../../foundation/programming-model.md, 6.1).
  - `insert` returns the new `Id`, or hands the value back when the slab
    is full: the refusal.
  - `get` and `get_mut` check the generation and return `None` for a
    handle whose slot holds another entity now.
  - `get2_mut` looks up two entities at once, for the rare step that
    needs both, and fails on equal handles (../../foundation/programming-model.md, 5.1).
  - `retire` marks a slot; `reclaim`, at the iteration's reclaim point,
    frees every retired one, so an entity can be looked up until the
    iteration ends.
  - **`Slab::iter(&self) -> Iter<'_, T>`** returns its entities with
    their handles in slot order, including those retired since the last
    `reclaim`: their handles still reach them.
- **`struct Iter<'a, T>`**, re-exported as **`SlabIter`**, is the iterator
  `Slab::iter` returns: its `Iterator::Item` is `(Id<T>, &'a T)`, and it
  visits entities retired but not yet reclaimed as well as the others.

## 4. Names across boundaries

- **`Token`** is an opaque `u64`, the only name that crosses a boundary.
  A layer makes one from its own handle and turns it back only for tokens
  it issued, in the record variant it issued them for.
- **`ReplyTo`** wraps a token, and is neither `Copy` nor `Clone`, and
  `#[must_use]`: replying consumes it, so a second reply does not
  compile. A missing one is the simulator's to catch.

`Name<N>` is a name of at most `N` bytes held inline, checked once when it is made, against its maker's grammar, and copied freely after: a source's or a method's name, never text a person reads.

A token's top four bits may tag its namespace (`Token::tagged`, `tag`, `untagged`), for a root that hands tokens to several children.

## 8. Time, deadlines and randomness

- **`Time`** is nanoseconds on a monotonic clock, in a `u64`, read by the
  shell or the simulator. **`Duration`** is a span, with checked and
  saturating arithmetic. **`Wall`** is wall-clock time, nanoseconds since
  the Unix epoch, a type of its own with no arithmetic on spans, so it is
  never mixed with `Time` and never arms a deadline.
- **`Civil`** is a date of the proleptic Gregorian calendar, in UTC: a
  year, a month and a day, whose public fields may describe no date.
  `Wall::civil` is the one conversion of a wall time to its date, which
  every `Wall` has. `Civil::valid` says whether a calendar holds it, and
  `Civil::days` counts its days from the epoch, signed, or gives `None`
  for no date, so a time just outside `Wall` may still come in by its
  offset. `Civil::wall` is its midnight, or `None` outside `Wall`.
  `Civil::month_name` is a month's English three-byte name. The texts of
  a date are their readers': RFC 3339 and the HTTP date's reading in
  skein-forge (../forge/dialects.md, 1.4), IMF-fixdate's writing in skein-http
  (http.md, 5.4), and `YYYY-MM-DD` in skein-ui (../ui/README.md, 9).
- **`earliest(left: Option<Time>, right: Option<Time>) -> Option<Time>`**
  is a `const fn` returning the earlier of two optional deadlines, `None`
  only when both are: an absent deadline never falls due.
- **`Deadlines`** is a layer's own timer table, bounded: `arm` a deadline
  for a key, `cancel` it, ask for the `next` one, and `expire` those that
  have passed, each removed before its handler runs. It is a pair of
  B-trees, priced at capacity.
- **`Rng`** is a seeded generator held in a layer's state: `next_u64`,
  `below`, `between`, `chance`. It is never used for secrets.
- **`CryptoRng`** is a cryptographic generator held in a layer's state,
  ChaCha20 keyed by a 32-byte seed the shell draws from the kernel at
  start, or a world from its own seed: `fill` writes fresh bytes into a
  bounded buffer. Whatever must not be guessed is drawn from it: secrets,
  OAuth states and PKCE verifiers, sign-ins' secrets. Each generator is
  made from the seed and a stream number its owner gives (ChaCha20's
  nonce), distinct for every generator of a process, so several drawn from
  one seed never yield the same bytes. It is neither `Copy` nor `Clone`,
  since a copy would yield its bytes again: its owner lends it by `&mut`,
  and a component that keeps a generator of its own is made one, from the
  seed, on a stream of its own.
- **`FixedStart`** is the instant and seed a process starts from when a
  tool walks it on fixed time, so that two walks read the same times and
  draw the same values: `pub struct FixedStart { pub wall: Wall, pub
  seed: u64 }`. Its text is the wall's whole seconds since the epoch in
  decimal, a comma and the seed in decimal, `1767690000,7`, with no sign,
  space or other byte and no leading zero but in `0` itself;
  `FixedStart::read(text: &[u8]) -> Option<FixedStart>` reads it, and
  refuses a number past its type, the seconds past `Wall` once made
  nanoseconds. `FixedStart::NOW: Time` is one hour, 3,600 seconds: the
  monotonic reading every process on a fixed start starts at.
  `draw(&self, stream: u64, out: &mut [u8])` fills `out` from a
  `CryptoRng` keyed by the seed's eight little-endian bytes and 24 zero
  bytes, on `stream`: what the process would have drawn from the kernel
  at its start, a cryptographic seed or an identifier, one stream for
  each. Its draws are known to anyone who knows the start, so nothing
  that guards a real account starts from one.
- **`FixedClock`** is the clock of a process on a fixed start, over the
  host's monotonic readings, `native`, that its caller gives: it reads
  the start's `NOW` and wall, and moves only to the deadlines it is asked
  for, so that a process's time is its own timers'. `FixedClock::new(start:
  &FixedStart, native: Time) -> FixedClock`; `until(&mut self, deadline:
  Option<Time>, native: Time) -> Option<Time>` keeps the deadline asked
  and gives the native instant to wait until for it: `native` plus the
  deadline less the clock's reading, saturating, the instant given when
  that deadline was first asked if it is asked again unchanged, and
  `None` for none, which forgets it; `read(&mut self, native: Time) ->
  (Time, Wall)` moves the clock to the deadline asked once `native` has
  reached its instant, then gives the clock's reading and its wall, the
  start's wall plus the span from `NOW`. It never moves back, and a
  deadline at or before its reading leaves it where it is.

## 9. The environment

**`Env<L>`** is what a step reads besides its own state: `now`, `wall`,
and its layer's `limits: L`. A step gets it behind a shared borrow.
`env.with(limits)` hands a child step the same clocks with its own limits.

## 10. Testing

Step tests drive each container through its operations, at and past a
cap where it has one: a full edge queue, a stale handle, a slot whose
generation would wrap, a refusal at the entrance, a deadline that fires
as it is cancelled, a scan cut at every byte. The step tests are
`src/tests.rs`, a module for each area of this document under
`src/tests/`: handles, containers, bytes, streams, and time.

The byte search is also compared with a naive one, and the intake with a
plain reference, over random cases drawn from a seed: a few hundred in
the step tests, and 20,000 from the same seeds in the fuzzy suite, in
`tests/lib` (../../foundation/testing-strategy.md, 8).

The join (`lib-containers.md`, 1.2) is held to a naive one, every key of either map looked up
in both and taken in key order, and the bisection (`lib-containers.md`, 1.3) to a linear scan
for the first item not below the key, the same way: random pairs of maps
with shared, disjoint and interleaved keys and values that differ or
agree; random sorted slices with repeated keys, each searched for every
key it holds and for keys between, below and above them. The step tests
also join empty maps, one empty and one not, and maps whose keys are all
shared; search slices of every length from 0 to 64; and drive a bisection
over `usize::MAX` indices answered `Less` every time and `Greater` every
time, each ending within `PROBES` with `end()` at `usize::MAX` and at 0.
Their step tests are in `src/tests/containers.rs`.

SHA-256 is held, in the step tests, to FIPS 180-4's short examples (the
empty message, `abc`, the 448-bit and 896-bit messages) and to messages
at a block's edges (55, 56, 63, 64 and 65 bytes) against the one-shot
form, each fed whole and at a few splits. The fuzzy suite adds the
million `a`s, sweeps every length from 0 to 200 bytes, each split at
every point, and adds 20,000 random messages, each split at a pair of
points drawn at random. HMAC is held to RFC 4231's test cases, each fed
whole and at a few splits in the step tests, and split at every point in
the fuzzy suite; `equal` to equal and unequal inputs of the same and of
different lengths. State digests are held to their contents: two
containers that reach the same contents by different histories (a map
filled in two orders, a queue that wrapped, a slab whose slots were
reused in the same order) digest alike, a change to one entry changes
the digest, and an integer written as `usize` digests as the same `u64`.
Their step tests are `src/tests/sha256.rs` and `src/tests/digest.rs`.

The request table's step tests are `src/tests/requests.rs`; its independent
obligation model, replay and seeded sweep are in `tests/lib/src/request_table.rs`
and `tests/lib/tests/suite/{request_table,fuzzy/request_table}.rs`.

Each capped container's allocation price is checked against the counting
allocator (../testing/service-tiers.md, 1), in `tests/lib/tests/suite/memory.rs`, in the
focused suite. Every capped container is built at capacities from 0 to
300, filled to them, emptied and filled again, every operation a step of
the meter, and what it held of its own is never more than its price:
`price(capacity)`, including `Slab::price` and `Deadlines::price`, or
`RequestTable::price(&limits)` for a request table.
What an item owns is its owner's to count, so the items own no heap; they
come in several sizes and alignments, up to 64 bytes, as a container's
price depends on both. An input moved into a step is accounted for as
../testing/service-tiers.md, 1 states. What an operation hands
out (an item taken, a delivery, a list moved into a box) is no longer
the container's, and the meter takes it off.

- **A slab, a queue, a list, a stack and an intake** grow as they fill (an allocated list excepted);
  capped queues and intakes grow exactly, never past their cap. The
  counting allocator checks each byte price against what is allocated,
  and scenarios hold measured peaks under named ceilings with headroom
  (../../foundation/testing-strategy.md, 6).
- **A map, a set and a deadline table** are filled in order, each leaf
  left behind holding six entries, then thinned to five a leaf, the fewest
  a leaf holds: a tree within a few nodes of the most its capacity allows,
  and driven at random. Their
  allocation price counts every node as an internal one, the larger kind, so at
  a capacity of 12 or more they hold from about 30% of it, for small
  entries, whose nodes are mostly edges, to 96%, for 64-byte keys and
  values. Below 12, a tree is at most one leaf, and the price counts
  up to three nodes.

## 11. The journal

`Journal<W, O>` is a commit barrier for writes and outputs. It is
generic over both and inspects neither. It belongs in lib because it is a
data structure, like a queue; a service does not hand-roll one
(../../foundation/programming-model.md, 10.2).

- **Admission:** `Journal::uncapped(release, durable)` keeps commits,
  held outputs and the door in queues that grow as they fill (`lib-containers.md`, section 1
  ); `release` is the most one release call moves, `durable` the last
  commit number the store holds, from which numbering goes on.
  `JournalLimits` holds only `release`, the per-call output limit.
  `restore_numbering(durable)` restores the numbering a restart finds
  later than the journal was made, before its first commit: numbers then
  begin after `durable`, and what it holds and has at its door keeps its
  place and stays ready. Called once a commit was numbered, or with a
  `durable` below the journal's, it is the owner's bug and an assertion.
  `decide()` opens a decision with no room to reserve, which cannot be
  refused; `Decision::push_write` and `push_held` add without a
  `Result`. New external work waits at the root's watermark
  (../../foundation/programming-model.md, 7.1), never on room in the journal.
- **One decision:** a `#[must_use]` value, not `Clone`, collects its
  writes and held outputs.
- **Commit and tags:** accepting a decision makes at most one numbered
  commit, holding all its writes. Numbers begin after `durable` (at one
  for a new journal) and increase in order. A writing decision accepted
  past number `u64::MAX` stops the journal, as an answer out of order
  does: `durable` is what the store holds, which no panic answers
  (../../foundation/programming-model.md, 10.4), so it makes no commit, the
  decision's held outputs never leave, and `release` answers `Stopped`.
  A decision without a write makes no
  commit. Each held output
  follows that decision's commit, or the last commit made if there was
  no write; it leaves only once that commit is durable. A commit answer
  makes that commit and every earlier one durable. Answers outside the
  order the store sends commits are the owner's bug and stop the
  journal.
- **The door and release:** `now(output)` admits an untagged output that
  decides nothing and is ready to leave at once. These outputs leave in
  their own order. Held outputs
  leave in the order they were collected, after their tags are durable.
  Each release call moves at most its configured count. The journal does
  not arm a deadline or advance commits by itself.
- **Stopping:** a root that stops of its own calls `Journal::stop()`,
  with no argument and no result. Nothing held or at the door leaves
  again, and `release` answers `Stopped`.
- **Failure:** `failed(number)` reports a failed sent commit. No output
  tagged with it or a later commit leaves, and the journal reports that
  it has stopped. It admits no more decisions or outputs.
- **Memory:** the journal costs what it holds: its queues grow as they
  fill and reserve nothing. Values it holds own their payload bytes;
  their owner meters those where they enter (../../foundation/programming-model.md, 6.1).
- **The backlog:** `backlog()` counts the commits made and not yet
  answered, the outputs held and those at the door, in entries, which a
  root compares with its watermark before it admits new work.

The journal uses generic types and lib's containers. It defines no
traits, takes no closures, and uses no `dyn` (../../foundation/programming-model.md,
10.2).

## 12. The request table

`RequestTable<O, R>` keeps bounded keyed requests until their answers are
final. It is generic over the asker's owner value and the requests it
holds, and inspects neither owners, requests nor answers. It belongs in
lib because it is a data structure with a lifecycle, like a queue or the journal; a service does not hand-roll one
(../../foundation/programming-model.md, 10.2). jig's client is its one user outside skein
(`jig/docs/design/client/domain/root.md`, 4), and lib keeps it all the same: it
is generic over the owner and the request, as no jig step crate may be
(`jig/AGENTS.md`), and a crate of its own would hold this one module and add a
manifest to save nothing.

An entry lives through three lifecycles at once, one enum each
(../../foundation/programming-model.md, 5.4): its sending (parked, ready, in flight, retrying,
retired); its keeping (not yet saved, saved, or restored, which needs known
retention and has nothing to save until its first sending); and its progress
(nothing to say, a change not taken, or the last taken). Its age's anchor is a
value learned once, read beside all three, and is no phase.

- **Owner:** each request keeps the asker's own value, of a `Copy` type,
  held and handed back with progress and a final answer, never read. An
  asker composing several parts names each owner with a variant of its
  own enum, carrying the part's token, or naming a restored request whose
  asker is gone. It can then route by the variant to the part that asked
  without keeping a routing table of its own. The owner is not required
  to be a token: a token's holder never interprets it (section 2).
- **Keys:** a key is drawn from the seed given at construction, or given
  by the asker (a digest of a name, say). A drawn key is never reused for
  another request. A given key is the asker's: the table admits it while
  it holds no entry under it. An entry holds its key from its admission
  to its reclaim, retired or not. A restored request keeps its key.
  Exhausting the key space refuses admission (`Full`) rather than reusing
  a key.
- **Scope:** each request keeps an opaque number its asker gives. Only
  requests whose scope is confirmed may be sent.
- **Kept before sent:** each admitted request emits its record for the
  asker's store: its key, scope, request and first sending's wall time,
  if it has been sent. Its first sending updates that record. Records
  leave before the sends they cover, in order; each take emits at most
  its configured count. Retiring a request emits its erase.
- **Not kept:** a request whose record the asker's store did not write
  is retired by its key and the number of its admission (`not_kept`),
  whatever its state: parked, ready, in flight or retrying. The asker hands a send over only once the records taken
  before it are stored, so such a request was never sent. It is never
  sent; an answer for its attempt changes nothing. Its erase is emitted,
  as any retired request's, since an earlier write may hold its record.
  Its owner value is handed back, as a final answer's, with no progress,
  and the asker answers it itself. A key that names no request, or a
  retired one, hands back nothing. So does an admission number earlier
  than the one the key holds: a stale "not kept" never retires a later
  holder of the same given key.
- **One attempt:** a request has at most one attempt in flight. Each
  attempt has its own fresh token, independent of the owner value.
  An answer for a superseded or retired attempt changes nothing.
- **Envelopes:** the protocol layer says which kind of answer came.
  *Final* retires the request and hands back its owner value, so the
  asker routes the answer unopened. *Again* schedules a retry under the
  same key. *Lost* does likewise while the link is up, and parks while
  it is down. *Signed out* parks until the request's scope is confirmed
  again. Retry spans start at a configured first span, double to a
  configured most, and are jittered from the seed.
- **Parking and resuming:** a down link parks requests and sends
  nothing. An up link resumes the confirmed scope's parked requests;
  confirming a scope permits only that scope's requests. Changing the
  scope parks every other scope's requests.
- **Retention:** the asker tells the table how long answers are kept.
  A request first sent longer ago than that span less the configured
  margin is never sent again: it is retired, erased, and its owner told
  that its outcome is unknown. The saved first sending is a wall time,
  so the cutoff survives a restart; retries and the cutoff use monotonic
  deadlines computed from the supplied clocks (../../foundation/programming-model.md,
  section 9). A restored request is not sent until retention is known.
- **Restoration:** records are admitted at startup within the same
  limits, keeping their keys and first sending times. They begin parked
  until the link is up and their scope is confirmed. The asker supplies
  their owner values and declared sizes again.
- **Progress:** changes to in flight, retrying, parked or unknown leave
  with the owner value, for display. The table never reads the answer
  or decides how progress is shown.
- **Admission and memory:** capacities are fixed at construction. `ask`
  and `restore` refuse with a reason, handing back the request and
  changing nothing: `Full` (past the held-request count or the sum of
  owners' declared bytes, or no key left to draw) or `Held` (the key is
  held). A restore of a held key is refused as `Held`. Each admission, by
  `ask` or by `restore`, has a number, which they return with the key.
  Numbers increase with each admission. `not_kept` names the key and the
  number; every other call stays by key: an asker holds one admission per
  key at a time. The table does not measure a request. `price(&limits)`
  prices its containers, owner values and bookkeeping from the limits
  with checked arithmetic; the owner prices the payload bytes it
  declares separately.

The asker decides each request's scope and contents, supplies its owner
value and declared size, and tells the table the link, confirmed scope,
retention and clocks. The protocol layer classifies envelopes; final
answers go directly to their owner. The table decides keys, attempts,
backoff, parking, retention cutoffs, store records and progress.

The table uses generic types and lib's own containers. It defines no
traits, takes no closures, and uses no `dyn` (../../foundation/programming-model.md,
10.2).

## 13. Not built yet

What of this document is not built is in [`lib.status.md`](lib.status.md).

## 14. Open questions

- **Remaining decoder allocation and scalar costs** (`lib-containers.md`, 1; `lib-bytes.md`, 1):
  `temper/docs/development/limits-fuzzy.md` records actual list growth,
  scalar reads and scalar writes after the append-bookkeeping repair.
  Eager decoder allocation needs a different entrance/allocation contract
  from grow-as-filled lists; coalesced writes must preserve field order,
  whole-put refusal and partial progress on overflow. Default: keep the
  required allocations, reads and writes until a profile supports that
  mechanism, with no change to wire bytes, bounds or errors. Smaller
  conversion costs stay below the account's roughly 2% stop rule.
- **temper's side of 4 and 8:** temper's six hand-written calendars, in
  `temper-box`, `temper-client-domain`, `temper-client-view` and
  `temper-client-tools`, move onto `Wall::civil` and `Civil`, and its
  hand-set token tags onto `Token::tagged`, in its own tree. They wait with
  no debt item to hold them.
- **The carriers left:** `lib-bytes.md`, section 1's rule goes crate by crate, each when a
  draft asks for it; `lib.status.md` names those it has not reached. The
  stream's records and `Intake` would close the most at once, since every
  head, body and frame passes through them.
- **Widening's other shapes:** a `u32` written through `.ok()`, `?` or
  `is_err` can never fail, so `usize_of` (`lib-bytes.md`, section 1) could take it too, as
  `len_u64` could `skein-fake-checkout`'s two lengths, should it take
  `skein-lib`. `lib.status.md` names what stands, and
  `skein/scripts/old_forms.py`'s `widening` holds the `.expect(..)` form at its
  floor.
