+++
id = "counting"
part = "counting"
kind = "feature"
needs = []
files = [
  "skein/testing/skein-heap/src/lib.rs",
  "skein/tests/heap/tests/suite/refused.rs",
  "skein/docs/plan/skein-world-costs/figures/counting.md",
]
effort = "medium"
minutes = 75
+++

# `Counting`, with the same counts

## Goal

`Counting`'s `alloc` and `dealloc` are 38% of a median `jig-core-world` focused test, 33% of a `jig-client-protocol-world` test and 32% of a `jig-ops-world` fuzzy test, and `malloc` and `free` themselves are about 2%. The rest is thread-local reads and `Layout` checks in an unoptimized build. Make them cheaper at that cause, with every count, peak and hand-out exact.

Acceptance:

- `allocated` and `freed` each reach one thread-local once per call, and `with_highs` no longer reaches a second.
- `dealloc` checks no condition of a layout that `alloc` already held. `alloc` decides null with no more than one comparison past what `layout` already guarantees, and still answers null past what a layout can describe.
- `skein/tests/heap/tests/suite/meter.rs`, `runtime.rs` and `span.rs` have no diff, and pass.
- The new `refused` test passes and bites.
- `skein-heap`'s public surface doesn't change: `Counting`, `Meter`, `Measured`, `Span`, `Grown`, with the same signatures.
- What the crate's comments require still holds, and they say what's true: reaching the counters never allocates, never fails and never panics; they're const-initialised and without a destructor; nothing allocates while the highs are borrowed. `RUNTIME` stays as it is, apart from the allocator's paths: its destructor is its purpose.
- `figures/counting.md` holds the six tests' names, and each test's reading before and after, as `notes/measuring.md` says, with what's left of `alloc` and `dealloc` split by cause.

## Inputs

- `skein/docs/plan/skein-world-costs/inputs/request.md`, item 1.
- `skein/docs/foundation/testing-strategy.md`, 8 and 8.1.
- `skein/docs/design/testing.md`, 5, the paragraph on the counting allocator.
- `skein/docs/plan/skein-world-costs/notes/common.md`, `skein/docs/plan/skein-world-costs/notes/measuring.md`.

## Owns and may change

Owns the files in the header.

Neighbour: `skein/tests/heap/tests/suite/main.rs`, one line, `mod refused;`. Nothing else under `skein/tests/heap` changes.

Nothing outside `skein/testing/skein-heap/src/lib.rs` reads `LIVE`, `PEAK`, `MADE`, `PHASE` or `HIGHS`, so no reader changes. The 52 packages that depend on the crate use its public surface alone.

Options, each decided:

- **The highs:** in the one thread-local, beside the counters, so that a step's path reaches nothing twice. If that costs more than it saves on the paths outside a step, which take only the counters, keep them apart and say so with the numbers; `with_highs`'s path must still make no second reach of the counters.
- **`Cell` per field, or one `Cell` over a `Copy` struct:** whichever the profile shows cheaper. A `Cell` per field is the default.
- **An unchecked layout:** `Layout::from_size_align_unchecked` still runs its precondition check in an unoptimized build, as `ub_checks` follow the calling crate's debug assertions. So it may save little over `from_size_align`. Read the profile: on each path, use what's cheaper, and if `unsafe` saves nothing there, keep the safe form and say so in your report with the numbers. Any new `unsafe` stays inside `unsafe impl GlobalAlloc`'s methods with a `SAFETY` comment, or takes a scoped `#[expect(unsafe_code, reason = "...")]` on its function. The module's `# The unsafe` section says what's true at the end.
- **`dealloc`'s silent return,** the `let Some(..) else { return }` on `padded`: it goes, since `alloc` held the conditions. A branch that can't be taken stays out rather than becoming a `broken()`.
- **A cost this can't reach** (an unoptimized call the path can't avoid): a line in the figures with its share. No `[profile.dev.package.skein-heap]`.

## Steps

### 1. Before anything changes

Choose the six tests and read each on your base, as `notes/measuring.md` says. Write the names and the readings to `figures/counting.md`, and split `alloc` and `dealloc` by callee: `LocalKey` reads and writes, `padded`, `malloc` and `free`, the rest. That split decides the options above.

### 2. The shape

The outer layer binds; inside it the choices are yours.

```rust
/// A thread's counts. Const-initialised and without a destructor: reaching
/// them never allocates and never fails, so the allocator can use them.
/// Nothing allocates while it borrows the highs.
struct Counts {
    live: Cell<i64>,
    peak: Cell<i64>,
    // Allocations made so far, which numbers them from one.
    made: Cell<u64>,
    phase: Cell<Phase>,
    highs: RefCell<Highs>,
}

thread_local! {
    static COUNTS: Counts = const { Counts::NONE };
}

/// Reaches `COUNTS` once.
fn allocated(layout: Layout) -> u64;
/// Reaches `COUNTS` once.
fn freed(layout: Layout, number: u64);
```

- `Meter` and `Span` read through `COUNTS` too. They aren't hot, so a small private helper is fine there.
- A `const` assertion that `Counts` needs no drop (`core::mem::needs_drop`) holds "without a destructor" in the code, if it reads well.
- `padded` may split: what's known from `layout` alone (the offset, its size) from what `alloc` must check.
- A panic still never reaches an allocator's path: a broken invariant goes to `broken()`, and arithmetic wraps as it does today.

### 3. The `refused` test

`skein/tests/heap/tests/suite/refused.rs` holds "`alloc` answers null past what a layout can describe". `tests/heap` forbids `unsafe`, so it goes through `Vec`:

```rust
//! The allocator answers null past what a layout can describe, and counts nothing.

#[test]
fn a_block_past_what_a_layout_can_describe_is_refused_and_not_counted() {
    let most = usize::try_from(isize::MAX).expect("isize::MAX fits");
    for size in [most, most - 7, most - 8, most - 15] {
        let meter = skein_heap::Meter::new();
        let mut bytes: Vec<u8> = Vec::new();
        assert!(bytes.try_reserve_exact(size).is_err(), "{size} bytes");
        assert_eq!(meter.held(), 0, "{size} bytes");
    }
}
```

`most - 15` is the largest size whose padded layout is valid, where `malloc` itself refuses; the other three are past it, and it's `padded` that refuses. Add a row for a page-aligned element type if it costs little. Break the rule once by hand, by removing the comparison, and report what failed.

### 4. After

Read the six tests again, and fill in the figures' after arm.

### 5. Checks

Once:

- `cargo fmt`
- `python3 -I -B skein/scripts/old_forms.py`
- `heavy cargo clippy -p skein-heap -p skein-heap-tests --all-targets -- -D warnings`
- `heavy cargo nextest run -p skein-heap-tests`
- `heavy cargo nextest run -p skein-world-tests -p skein-oauth-accounts-world -p skein-forge-machine-world -p jig-store-world -p smith-hosts-world -p hearth-render-world`: the worlds that check memory by `Meter`, `Measured` and `Span`, across the trees.

A count that moves in any of them is yours, whatever its message says. Delete `target/world-costs/`, commit and push.

## Report

The six tests' totals before and after; what's left of `alloc` and `dealloc`, by cause and share, and what the code can't reach; the option you took for the highs, the `Cell`s and the unchecked layout, with the numbers behind each; the rule you broke for `refused`, and the failure seen.
