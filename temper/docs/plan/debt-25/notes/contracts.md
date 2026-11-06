# debt-25: the conformance kit's binding shapes

For jig D030 and D034, whose tasks meet in `jig/testing/jig-conformance`. Line numbers are at this plan's base: find the code by name.

## Where things are

The items' paths are out of date. The kit is `jig/testing/jig-conformance` (`harness.rs`, `referee.rs`, `core.rs`, `core/*`). The core's adapter is `jig/testing/jig-core-kit/src/conformance.rs`, which holds `store_view` and `Observer::durable`. There are four adopters, each implementing `Application`:

| Adopter | Package | File |
|---|---|---|
| `Testing` | `jig-conformance-world` | `jig/tests/conformance/src/lib.rs` |
| `Ops` | `jig-ops-world` | `jig/tests/ops/src/conformance/mod.rs` |
| `OpsApplication` | `jig-ops-world` | `jig/tests/ops/src/joint/engine.rs` |
| temper's | `temper-conformance-world` | `temper/tests/conformance/src/lib.rs` |

`jig/testing/jig-client-scripted-engine/src/application.rs` has an `Application` of its own, which this plan doesn't touch.

Every trait edit breaks all four, so each trait task carries temper's lines that name the item, edited for that alone. That's this plan's only crossing into temper.

**Test packages for a trait task:** `jig-conformance`, `jig-conformance-world`, `jig-ops-world`, `temper-conformance-world`. A task that touches `jig-core-kit` adds `jig-core-kit` and `jig-core-world`.

## `Application` when D030's tasks end

22 functions and 10 types become 19 and 7. The rest stands, so that `jig/docs/design/domain/testing.status.md`, 5 reads true:

```rust
pub trait Application {
    type Domain; type Store: Durable; type Config; type Systems;
    type Peers; type Event: Debug; type Delivery: Debug;
    // W is <Self::Store as Durable>::Write. Input<E> has no row type.
    fn build(&Config) -> Domain;  fn store(&Config) -> Store;
    fn systems(&Config, u64) -> Systems;  fn peers(&Config, u64) -> Peers;
    fn policy(&Peers) -> &Policy;  fn start(&Config, &mut Store) -> Vec<Event>;
    fn cold(&mut Peers, &Store, Clock);  fn timers(&Config) -> Vec<Event>;
    fn step(&mut Domain, &Config, Clock, Input<Event>);
    fn release(&mut Domain, &Config, Clock) -> Vec<Output<W, Delivery>>;
    fn inbound(&mut Peers, &Store, Clock, &mut Event);
    fn deliver(&mut Peers, &mut Systems, &mut Store, Clock, Delivery) -> Vec<Input<Event>>;
    fn poll(&mut Peers, &mut Systems, &mut Store, Clock, bool) -> Vec<Input<Event>>;
    fn submitted(&mut Peers, &Store, Clock) {}
    fn observed(&mut Peers, &Systems, &Store, Clock) -> Vec<Observed>;
    fn ready(&Domain) -> bool;  fn quiescent(&Domain) -> bool;
    fn reclaim(&mut Domain);  fn maximum(&Config) -> u64;
}
```

This sketches the items' shapes. Each function keeps the receiver and parameter names it has today, and its doc comment.

**No lower, on purpose:**
- `Harness<A>` needs a trait (`jig/docs/design/domain/testing.md`, 1).
- `type Domain = root::Domain` rules out a kit trait on a root (the orphan rule), and step code holds no traits.
- Decoding reads `jig-protocol-charter`, which links smith, and every application links this kit.
- The first period and `Drained` differ between adopters in events the referee sees.

Sub-traits or an enum-argument hook would lower the count and pay nothing: don't.

## `Outside`, the typed comparison

In `jig/tests/conformance/src/replay.rs`, replacing `outside`'s string:

```rust
#[derive(PartialEq, Eq, Debug)]
pub struct Outside { /* owned: systems, peers, the store's four fields, referee, clock: u64, outcome, trace: Vec<String> */ }
pub fn outside(world: &World) -> Outside;
impl Outside {
    /// The field that differs and its first differing `{:#?}` line.
    pub fn first_difference(&self, other: &Self) -> Option<String>;
}
```

It is owned, because the test's `drop(resumed)` stays before the fresh run. `outside` may take the `World` by value and move its fields out, if cloning them would need more derives; the test then asserts `outcome` first, in the same order of checks. It compares the store's four fields, which are what the store's `Debug` shows (`jig/testing/jig-test-store-files/src/store.rs`), and it compares trace lines, not `enabled`.
