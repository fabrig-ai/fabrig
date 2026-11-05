+++
id = "large-render"
part = "large"
kind = "feature"
needs = []
files = ["hearth/tests/builder/src/fixtures/large.rs", "hearth/tests/builder/src/fixtures.rs", "hearth/tests/builder/tests/suite/main.rs", "hearth/tests/builder/tests/suite/memory.rs", "hearth/tests/builder/tests/suite/fuzzy/memory.rs"]
effort = "high"
minutes = 60
+++

# A large render, under its own ceiling

## Goal

The 1 GiB figure stood for one real guarantee: the defaults fit a modest machine. That claim is kept as a measurement: a fixed, realistic large
render through the builder at the shipped limits, its peak under a ceiling set from what it measures.

Acceptance:

- `a_large_render_holds_no_more_than_its_ceiling` is in `hearth/tests/builder/tests/suite/fuzzy/memory.rs`, and passes in the fuzzy suite.
- It runs `fixtures::large::scenario()` of `notes/contracts.md`, 4 at `hearth::limits()`.
- Its test asserts what makes the scenario large: the targets printed, and the entries and the documents the fake git served.
- Its ceiling follows `notes/common.md`, "A ceiling", and bites.
- Its CPU seconds, by nextest's time, are in the report.
- **The commit says what "large" means here:** entries, documents and targets, with the peak and the ceiling.

## Inputs

- `hearth/docs/plan/hearth-memory-1/notes/common.md`
- `hearth/docs/plan/hearth-memory-1/notes/contracts.md`, 4
- `hearth/docs/design/domain/render.md`, 8.4
- `hearth/docs/design/domain/testing.md`, 8.4
- `skein/docs/foundation/testing-strategy.md`, 6

## Owns and may change

- **Owns:** the files under `files`.
- **`hearth/tests/builder/src/fixtures.rs`:** only the `pub mod large;` line.
- **`hearth/tests/builder/tests/suite/main.rs`:** only `mod memory;` in `mod fuzzy`.
- **`hearth/tests/builder/tests/suite/memory.rs`:** only taking out the new test. The four scenarios there keep their ceilings.
- **Options, each pre-approved:**
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
4. `cargo fmt`, clippy on `hearth-builder-world`, focused tests of `hearth-builder-world`, and the new test under `--profile fuzzy`.
5. Commit, with what "large" means, and push.

## Report

The first line, the scenario's figures, its peak and ceiling, what bit, its CPU seconds, and which of the options you took.

## Answers

2026-11-05. **Asked:** at 2,048 entries, 256 documents and 64 targets the test costs 6.3 CPU s against 1.5, and another exact halving leaves too few
documents for 64 hosts: may entries fall further with a document floor, or may the work look outside the three files?

2026-11-05 (fabrig/hearth, the plan's owner, through agents/loop). **Don't shrink the scenario.** At 2,048 entries it stopped being large, and at 4,096 it costs 15 CPU s. Keep the plan's 4,096 entries, 512 documents and 64 targets; drop the 1.5 s cap; run it as a test under the suite's `fuzzy` module, so the fuzzy profile selects it: hearth's fuzzy suite is at about 68 of 120 CPU s, and the focused budget doesn't pay for it. It stays one fixed scenario under its ceiling, and the counting allocator is the same suite's.

**What that means here.** The owner's answer is the task's, in place of any shrinking and of the 1.5 CPU s. Continue from `737159af7e` on `hearth-memory-1/large-render`:
bring its fixture back to the plan's 4,096 entries, 512 documents and 64 targets, and move its test to `fuzzy/memory.rs`, a place this spec sets over
`notes/contracts.md`, 4's. Keep the commit's measured peak and ceiling true for that scenario, and re-show the bite for it. Don't touch
`hearth/crates/**`, `hearth/testing/**` or `skein/`. No run has isolated the cost: the run's table had it following the work per target, not the listing.
