+++
id = "read-owners"
part = "owners"
kind = "feature"
needs = []
files = [
  "temper/crates/temper-engine-domain-forge/src/boundary.rs",
  "temper/crates/temper-engine-domain-forge/src/domain.rs",
  "temper/crates/temper-engine-domain-forge/src/landing.rs",
  "temper/crates/temper-engine-domain-forge/src/tests.rs",
  "temper/crates/temper-engine-domain/src/route/forge_route/reads.rs",
  "temper/tests/engine/domain/tests/suite/forge_routes/reviews.rs",
  "temper/docs/design/domain/forge.status.md",
]
effort = "high"
minutes = 120
+++

# Each forge answer reaches the read that asked

## Goal

The root and the forge connector name reads in one range of client owners from two counters, so a change step can take an agent's `Busy` and the agent its step's `Commits`: that is how live-temper's `temper serve` came to render a `read_file` answered `Commits` and panic (`temper/docs/plan/forge-read-answer/notes/contracts.md`, section 1). First a failing story in the engine domain's world shows it; then the root's reads get a range that is the root's alone, and the connector mints none of its own there.

This task departs from the implementer's text in two ways: its first commit is a test that fails (the common note's *failing test first*), and one status entry is yours to remove (acceptance 5).

Acceptance:

1. A story in `temper/tests/engine/domain/tests/suite/forge_routes/reviews.rs`, `a_runs_busy_reads_and_a_change_steps_read_each_get_their_own_answer`, is committed first and fails on the base for the cause above: an attempt's `Busy` never reaches the run, and the run's read is answered with the step's kind. The report names the attempt, the kind it got and the assertion that failed.
2. `temper_engine_domain_forge::root_read` and `is_root_read` exist as `notes/contracts.md`, section 2 sketches them; the root's `read_call` (`temper/crates/temper-engine-domain/src/route/forge_route/reads.rs`) mints by `root_read` alone, refusing "read correlation exhausted" where it gives `None`; the connector's router passes an unclaimed `Request::Read` to the root where `is_root_read`; a change step's and a landing's read owners are `sequence | 1 << 62`.
3. The story passes, and a table test in `temper/crates/temper-engine-domain-forge/src/tests.rs` pins the range: the rows below, and the connector's own pending step and landing reads never in it.
4. A change of meaning runs its readers' tests: `heavy cargo nextest run -p temper-engine-domain-forge --lib`, `-p temper-engine-domain --lib`, `-p temper-engine-forge-world`, `-p temper-engine-protocol-forge-world` and `-p temper-engine-domain-world` pass. Clippy is clean for every package changed.
5. The **Not yet** sub-entry for 5.3, *each answer reaches the read that asked*, is gone from `temper/docs/design/domain/forge.status.md`, the rest of the file unchanged.

## Inputs

- `temper/docs/plan/forge-read-answer/notes/common.md`.
- `temper/docs/plan/forge-read-answer/notes/contracts.md`, sections 1 and 2: they bind.
- `temper/docs/design/domain/forge.md`, section 5.3, from *An agent's fresh read refused `Busy`* to the end of *Each answer reaches the read that asked*.

## Owns and may change

**Owned:** the `files` above. `boundary.rs` is re-exported whole by `lib.rs` (`pub use boundary::*`): no export line is needed.

**Readers and neighbours, edits allowed:**

- `temper/tests/engine/protocol-forge/tests/suite/protocol.rs`, `commit_read`: its owner `1_u64 << 61` is `root_read(0)` and stays in range; you may write it as `root_read(0)`, nothing else there.
- Any test in the worlds of acceptance 4 that builds a step's or a landing's owner from `1 << 61`, or asserts on one: that owner's expression only, named in the report.

**Options, each with its default:**

- The story's held read is a change step's `Commits` (containment, forge.md 5.3, or the catch-up range). Default: wake the change while the reviewer is live (move its base, as `update_first.rs` or `catch_up.rs` do, or report its CI, as `ci_poll.rs`'s `status` does) and hold the first client `Answered` whose result is `Ok(Answer::Commits { .. })`, as `ci_poll.rs`'s `hold_reviews` holds `Reviews`. If no step reads `Commits` while a run is live, hold a landing's `Tree` or the step's `Pull`, `Checks` or `Reviews`, and say which: the assertions don't change.
- The sweep: the root's serial passes the connector's sequence by the run's reads, one serial per attempt. Default: measure, on the base, the attempt that crosses, and sweep at least twice that and at least 64 attempts, a comment naming the measured attempt. If the root's serial is already past the held read's sequence when the sweep starts, hold a read the connector makes later (wake the change again) and say so.
- Capacity: `review_read_budget(2, 0)` and `occupy_read_capacity` fill the client's slab, as `reviewer_reads_back_off_when_forge_admission_is_busy` does, so every sweep attempt is refused `Busy` at once. If the held step read and the slab need another shape, keep that test's helpers and say how.
- The router may keep its map lookups as they are, only the last branch changing. If a debug check that a root read's owner `is_root_read` where the root hands it to the connector would break the worlds' injected owners (`occupy_read_capacity`'s `10_000 + serial`), leave it out.

## Steps

1. The story, red, in `reviews.rs` beside `reviewer_reads_back_off_when_forge_admission_is_busy`:

   ```rust
   #[test]
   fn a_runs_busy_reads_and_a_change_steps_read_each_get_their_own_answer() {
       // forge.md 5.3, each answer reaches the read that asked: a step that took
       // a run's Busy handed it the step's Commits (live-temper, 11-06).
       let (mut world, reviewer, head) = review_read_budget(2, 0);
       let step = hold_step_read(&mut world);          // a change step's read in flight, its terminal held
       let held = occupy_read_capacity(&mut world);
       let file = Read::File { head, path: FILE.into(), max_bytes: 1024 };
       for attempt in 0..SWEEP {
           let busy = world.busy_calls;
           ask_read(&mut world, &reviewer, 10, forge_world::REPO, file.clone());
           // tick until the attempt is answered Busy, bounded
           assert_eq!(world.busy_calls, busy + 1, "attempt {attempt}: the run hears its own Busy");
       }
       // release `step` and `held`, tick
       // the run's read is admitted and gets its own kind
       assert!(matches!(read(&mut world, &reviewer, 10, forge_world::REPO, file), Ok(Answer::File { .. })));
       // no answer the run got since the sweep began is another read's kind
       // the change's step took its own answer: no InvalidAnswer refusal for the change
   }
   ```

   Commit it alone ("temper-engine-domain-world: a run's Busy read and a change step's read under one owner"); it fails on the base.
2. The range, in `temper-engine-domain-forge/src/boundary.rs`, beside `Request::Read`, with the contract's doc comments: `root_read(serial) -> Option<Token>` (`Some(Token::new(serial | 1 << 61))` for `serial < 1 << 59`), `is_root_read(owner) -> bool` (`owner.raw() >> 59 == 0b0_0100`). Checked arithmetic and shifts, as the strict subset has them.
3. The root mints by it (`reads.rs`, `read_call`), the router passes by it (`domain.rs`, the `skein_forge_client::Request::Read` arm's last branch before `lost_read`), and the connector's change step (`domain.rs`, the `PendingStep`'s owner) and landing (`landing.rs`, the `Pending`'s owner) take `sequence | 1 << 62`.
4. The table, in `temper-engine-domain-forge/src/tests.rs`:

   ```rust
   #[test]
   fn the_roots_read_range_is_its_own() {
       for (serial, owner) in [(0, Some(1 << 61)), (1, Some((1 << 61) | 1)), ((1 << 59) - 1, Some((1 << 61) | ((1 << 59) - 1))), (1 << 59, None)] {
           // root_read(serial) == owner, and is_root_read of it
       }
       for owner in [7 | (1 << 62), 7 | (1 << 61) | (1 << 59), 7 | (1 << 60), 7 | (1 << 63), 7 | (1 << 59), 10_000] {
           // !is_root_read(owner)
       }
   }
   ```

   and one more test, `the_connectors_own_reads_are_outside_the_roots_range`, with a change step's first read and a landing's first read pending (the fixtures `src/tests/change.rs` and `src/tests/landing.rs` use), asserting no key of `d.steps` or `d.landing.pending` `is_root_read`. Bite: put the step's owner back to `1 << 61` once and see this test and the story fail.
5. The status entry (acceptance 5).
6. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p temper-engine-domain-forge -p temper-engine-domain -p temper-engine-domain-world --all-targets -- -D warnings`; the runs of acceptance 4, the story's alone first (`heavy cargo nextest run -p temper-engine-domain-world --test suite -E 'test(=forge_routes::reviews::a_runs_busy_reads_and_a_change_steps_read_each_get_their_own_answer)'`), naming its CPU time. `heavy cargo check -p temper-service --tests` and `-p temper --tests` for the dependents. Commit ("temper: the root's forge reads take a range the connector never mints in, so each answer reaches its read"), push.

## Report

As the implementer's text says, and: whether the story's failure on the base is what the live stories met (`notes/contracts.md`, section 1: a `Busy` not noted, then the step's kind on the run's `read_file`), the measured crossing attempt and the sweep's bound, which read you held, and the story's CPU time.
