+++
id = "large-render"
part = "large"
kind = "feature"
needs = []
files = ["hearth/tests/builder/src/fixtures/large.rs", "hearth/tests/builder/src/fixtures.rs", "hearth/tests/builder/src/simulated.rs", "hearth/tests/builder/src/hosted.rs", "hearth/tests/builder/src/hosted/**", "hearth/tests/builder/tests/suite/memory.rs"]
effort = "high"
minutes = 60
+++

# A large render, under its own ceiling

## Goal

The 1 GiB figure stood for one real guarantee: the defaults fit a modest machine. That claim is kept as a measurement: a fixed, realistic large
render through the builder at the shipped limits, its peak under a ceiling set from what it measures.

Acceptance:

- `a_large_render_holds_no_more_than_its_ceiling` is in `hearth/tests/builder/tests/suite/memory.rs`, and passes in the focused suite.
- It runs `fixtures::large::scenario()` of `notes/contracts.md`, 4 at `hearth::limits()`.
- Its test asserts what makes the scenario large: the targets printed, and the entries and the documents the fake git served.
- Its ceiling follows `notes/common.md`, "A ceiling", and bites.
- It costs at most 1.5 CPU s, by nextest's time.
- **The commit says what "large" means here:** entries, documents and targets, with the peak and the ceiling.

## Inputs

- `hearth/docs/plan/hearth-memory-1/notes/common.md`
- `hearth/docs/plan/hearth-memory-1/notes/contracts.md`, 4
- `hearth/docs/design/domain/render.md`, 8.4
- `hearth/docs/design/domain/testing.md`, 8.4
- `skein/docs/foundation/testing-strategy.md`, 6

## Owns and may change

- **Owns:** the files under `files`.
- **`hearth/tests/builder/src/fixtures.rs`:** the `pub mod large;` line, and a fix `## Answers` allows.
- **`hearth/tests/builder/src/simulated.rs`, `hearth/tests/builder/src/hosted.rs`, `hearth/tests/builder/src/hosted/**`:** only a fix `## Answers` allows.
- **`hearth/tests/builder/tests/suite/memory.rs`:** only the new test. The four scenarios there keep their ceilings.
- **Options, each pre-approved:**
  - If the scenario costs over 1.5 CPU s, follow `## Answers`, in its order, and give the figures in the report.
  - If a shipped limit refuses the scenario, shrink the scenario rather than change the limit, and say so.
  - If its rendering shows a bug in hearth, stop and ask: the scenario isn't to change what hearth does.

## Steps

1. `hearth/tests/builder/src/fixtures/large.rs`, built on `fixtures::thin()` and `fixtures::THIN`, as `hearth/tests/builder/src/fixtures/bounds.rs`
   builds `queued()`:

   ```rust
   //! The large render: what "the defaults fit a modest machine" means (hearth/docs/design/domain/render.md, 8.4).
   pub const ENTRIES: usize = 4_096;
   pub const DOCUMENTS: usize = 512;
   pub const TARGETS: usize = 64;
   #[must_use]
   pub fn scenario() -> Scenario<'static>
   ```

   Make it read as a real repository: 64 hosts, each `hosts/<name>/<name>.json` and `hosts/<name>/<name>-prod.json`; tasks that each have a
   `task.json`, settings and templates, up to `DOCUMENTS` in all; and static files under the tasks' `tree/`, up to `ENTRIES` in all, of sizes
   drawn from a fixed table, not from a seed. Each target takes several of the tasks.
2. The test, in the shape of `queued_specs_hold_no_more_than_their_ceiling`:

   ```rust
   #[test]
   fn a_large_render_holds_no_more_than_its_ceiling() {
       const CEILING: u64 = ...;
       let outcome = simulated::checked_with(&fixtures::large::scenario(), bounds::SEED, hearth::limits(), CEILING);
       ...
   }
   ```
3. Measure the peak, and name the ceiling by `notes/common.md`, "A ceiling".
4. `cargo fmt`, clippy on `hearth-builder-world`, and focused tests of `hearth-builder-world`.
5. Commit, with what "large" means, and push.

## Report

The first line, the scenario's figures, its peak and ceiling, what bit, its CPU seconds, and which of the options you took.

## Answers

2026-11-05. **Asked:** at 2,048 entries, 256 documents and 64 targets the test costs 6.3 CPU s against 1.5, and another exact halving leaves too few
documents for 64 hosts: may entries fall further with a document floor, or may the work look outside the three files?

**Answer: find the cause first, then a floor on what shrinks. The 1.5 CPU s and the 64 targets stay.** Shrinking alone won't do it. In the run's own
table, halving entries and documents took 15.5 to 12.3 CPU s, static sizes moved nothing, and fewer tasks per target with shared configurations took
12.3 to 6.3. So the cost follows the work per target, not the listing; the first two rows' slope, on today's fixture, leaves about 3 CPU s at no entries at all: a rough line, but over
1.5. `737159af7e` on `hearth-memory-1/large-render` stays: continue from it, and its figures are the "before".

In this order, stopping as soon as the test is within 1.5 CPU s:

1. **Isolate the cause, and name it in the report.** Reading and timing anywhere in the repository is allowed; timing code is throwaway. Split the run:
   building the scenario and `Shared`, the render in the client, the hosted git and mmdebstrap processes, and the checks of `outcome`. Compare
   `Memory::Within` with `Memory::Unchecked`, which `simulated::run` gives without the checked allocator's bookkeeping. Compare a run of 8 targets, to
   see which of targets, documents and rendered templates the cost grows with, and whether it grows faster than linearly: dev builds leave hearth's
   crates unoptimised.
2. **Fix a cost in the test's own side:** the fixture, `large::served`, or the harness files under `Owns`. A harness fix changes no other test's
   assertions, goldens or trace, and its commit gives the before and after.
3. **Shrink per-target work.** Each target keeps at least two tasks: "several" in Steps, 1 means that. A target may render fewer templates.
4. **Shrink entries and documents, no longer together,** to at most these floors: **1,024 entries, of which 192 documents.** 192 is the 131 that 64
   hosts, fleet, environment and hearth need, and 61 task declarations and templates. Below that it's no longer the repository Steps, 1 describes.

Don't touch `hearth/crates/**`, `hearth/testing/**` or `skein/`: that's another writer's or a question. **If it's over 1.5 CPU s at the floors, or the
cause is in hearth's crates, skein or the fake git, stop and report `Question:`,** with the split from 1, the figures at the floors, and the fix you'd
make. The cost in a hearth crate may be `focused-budget`'s to fix, and a raise of 1.5 CPU s is the user's, never a run's. Keep the commit's measured
peak and ceiling true for whatever scenario you end on, and re-show the bite for it. `close` carries the final figures into the design, as
`notes/contracts.md`, 4 says.
