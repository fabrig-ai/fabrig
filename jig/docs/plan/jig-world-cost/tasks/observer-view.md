+++
id = "observer-view"
part = "observer-view"
kind = "feature"
needs = []
files = [
  "jig/testing/jig-conformance/src/core.rs",
  "jig/testing/jig-conformance/src/core/evidence.rs",
  "jig/testing/jig-conformance/src/core/intents.rs",
  "jig/testing/jig-conformance/src/core/named.rs",
  "jig/testing/jig-conformance/src/core/projects.rs",
  "jig/testing/jig-conformance/src/core/snapshot.rs",
  "jig/testing/jig-core-kit/src/conformance.rs",
  "jig/tests/ops/src/conformance/mod.rs",
  "jig/tests/ops/src/conformance/observations.rs",
  "jig/docs/development/debt/D034-world-replay-cost.md",
  "jig/docs/design/domain/testing.md",
  "jig/docs/design/domain/testing.status.md",
]
effort = "medium"
minutes = 75
+++

# The core observer makes a store view only for a reader

## Goal

One short core-world story builds `store_view` 91 times: for every core event in `inbound` (38), at each `submitted` (26) and at each new commit (26). Most of the first two read no row. Build a view only when something will read it, as `jig/docs/design/domain/testing.md`, 5 now says. Then, as far as it's simple, shrink `durable_at`'s whole `snapshot`.

**The view is made from the rows as they are and never kept.** D034's warning is why: `Got` changes rows outside commits, so `applied` doesn't name their state. A cache keyed on `applied` is the trap here.

**D034's keyed row feed isn't required,** and D034 stays open.

This task departs from the implementer's text in one way: **it writes design lines**, in `testing.md`, 10 and `testing.status.md`, 5, as well as D034, because their words depend on how far the `snapshot` part goes. Those three edits are its only design work, and the words are below.

Acceptance:

1. `jig_conformance::core::Observer::inbound` runs its view closure only for a party's request (`people::Event::Ask`), `core::Event::ProposeSettings` and a run's accepted proposal (`core::Event::NamedAction` with `DecideProposal` and `Accept`). `Observer::submitted` runs its only for a commit that erases `tasks::Key::Activity` or `tasks::Key::Ended`.
2. Tests show both: an event that reads no row and a commit that erases no task run no closure, and one of each that does run it exactly once.
3. Every adopter's behaviour holds: the focused suites of `jig-conformance`, `jig-core-kit`, `jig-core-world`, `jig-conformance-world`, `jig-ops-world`, `jig-client-protocol-world`, `jig-client-world`, `jig-engine-clients-world` and `jig-store-world`, and the fuzzy suites of `jig-core-world`, `jig-conformance-world` and `jig-ops-world`, pass.
4. `Store`'s `named` field and anything else this leaves unused are gone.
5. `activity_referee_rejects_changed_or_erased_lines` passes unchanged, and no referee rule changes.
6. D034, `testing.md`, 10 and `testing.status.md` say what's true after.

## Inputs

- `jig/docs/design/domain/testing.md`, 5 (the paragraph on an adapter's view, written by this cut) and 10.
- `jig/docs/design/domain/testing.status.md`, 5.
- `jig/docs/development/debt/D034-world-replay-cost.md`.
- `jig/docs/design/domain/engine.md`, 15, for why the observer reads what it reads.
- `jig/docs/plan/jig-world-cost/notes/common.md`.

## Owns and may change

- **Owns:** the spec's `files`.
- **Readers, read at the cut.** Only two adapters call what changes:
  - `jig-core-kit/src/conformance.rs`: the kit's `Observer::inbound` and `submitted` **keep their own signatures**, so `jig-core-kit/src/world.rs`, `world/admission.rs`, `world/store.rs` and `jig/tests/conformance/src/lib.rs` aren't edited.
  - `jig/tests/ops/src/conformance/mod.rs`, at its `inbound` and `submitted`.
- **Unchanged, so their readers aren't touched:** `Observer::durable`, `fresh_mark` and `cold`, on both layers. `jig/testing/jig-client-kit/src/joint/engine.rs` calls the kit's `durable`, and `jig/tests/ops/src/joint/engine.rs` has its own store: **both are in the plan's stay-out list.** Leave them. If a compile error points you there, your signature is wrong.
- **The other readers of `jig_conformance::core`:** `temper/tests/conformance/src/observations.rs`, `temper/tests/one-process/src/referee.rs` and `jig/tests/one-process/src/referee.rs` name its items but call neither method. Confirm with `heavy cargo check -p temper-conformance-world -p temper-one-process-world -p jig-one-process-world --tests`, and make only a compiler-forced edit there.
- **Options:**
  - Inside `jig_conformance::core`, how the store-free remainder is split is yours: `Makes::inbound` splits its `Act` arm from its `Ask` arm, and `Intents::inbound_core` keeps its order of observations. **Keep the order of observations exactly**: the referee sees them in order. If keeping it needs the view for an event outside acceptance 1's list, add that event to the list, the design paragraph and the report.
  - `snapshot`, see Step 5: if nothing there is simple, change nothing, and say so.
  - If an adopter's behaviour moves and you can't see why, land the kit's `inbound` half alone, say so, and leave the rest.

## Steps

1. **`core/evidence.rs`:** a helper both adapters and `view` use, and `Store` loses `named`.

   ```rust
   /// One submitted commit's core part (an adapter's `submitted` reads no other).
   #[must_use]
   pub fn commit<'a, W: 'a>(number: u64, writes: &'a [W], write: impl FnMut(&'a W) -> Write<'a>) -> Commit<'a>
   ```
2. **`core.rs`:**

   ```rust
   /// Observe inbound peer traffic, before the root may decide it. `store` runs
   /// only for an event whose check reads a row (testing.md, 5).
   pub fn inbound<'a>(&mut self, store: impl FnOnce() -> Store<'a>, now: u64, event: &mut core::Event);

   /// Observe the commit the store has just received, and the revisions it names.
   /// `store` runs only for a commit that erases a task's activity or an ended task.
   pub fn submitted<'a>(&mut self, commit: Option<Commit<'a>>, named: Vec<NamedRevision>, store: impl FnOnce() -> Store<'a>);
   ```

   `Intents::submitted` and `Named::submitted` take the commit's number and `named`, where they read `store.pending.back()` and `store.named` today.
3. **`jig-core-kit/src/conformance.rs`:** pull `store_view`'s two closures into named functions so `submitted` can use the write translation alone. `inbound` passes `|| store_view(store)`. `submitted` builds `named` from `store.pending.back()` as today, into a local `Vec`, and passes `store.pending.back().map(..)`, `named` and `|| store_view(store)`.
4. **`jig/tests/ops/src/conformance/`:** the same, with `observations::core_store`. Its write translation becomes a named function beside `core_store`. Ops passes an empty `named`, as it does today.
5. **`snapshot`, as far as it's simple:** read `durable_at` and `snapshot.rs`. Simple means: folding `durable_at`'s own repeated passes over `store.rows` into fewer, or moving what's cloned out of a view that's dropped anyway. Keep the order of observations. **Stop before anything keyed, anything that remembers rows between commits, or any change to the referee's walk.** Every one of D034's reasons against a row feed stands. Say in the report what you did or why you did nothing.
6. **Tests,** beside `activity_tests` in `jig-core-kit/src/conformance.rs` (it has the fixture; `observer.inner` is in reach):
   - `an_event_that_reads_no_row_makes_no_view`: `inbound` with a closure that panics, for an event outside the list. Then an `Ask` with a counting closure: it ran once.
   - `a_commit_that_erases_no_task_makes_no_view`: the same for `submitted`, with a `Write::Save` commit, then a `Write::Erase` of `tasks::Key::Ended`.

   Break each once by hand and report it. These are the rule's tests: keep each to the rows that can fail on it.
7. **Design and debt.** `testing.status.md`, 5: remove the two lines this cut added (`Observer::inbound` and `Observer::submitted` still take a view...). Rewrite the 5 entry about `durable_at` only if Step 5 changed what it says. `testing.md`, 10, *A commit's rows for the core's observer*: the words stay if `snapshot` is still whole; otherwise say what it does now. D034: from *What*, take out `Observer::inbound also builds the full view for every core event` and its two paths, keep the `Neighbours::busy_events` sentence, set **Checked** to today's `date -u`, and write what Step 5 did. The measured table stays, as what it says it is: one run each, before this change. **Status stays open.** Run `python3 -I -B tools/citations/citations.py --base origin/main`.
8. Narrow runs, then the implementer's closing steps. This changes no meaning but its reach is wide, so acceptance 3's list runs, one package at a time. Commit and push.

## Report

The implementer's report, plus: the view count for a short core story if you can read it from the code without measuring, otherwise `unknown`; what Step 5 did; each rule you broke and what the test showed; any event you added to the list.
