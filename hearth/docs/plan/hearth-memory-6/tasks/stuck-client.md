+++
id = "stuck-client"
part = "orders"
kind = "feature"
needs = []
files = ["hearth/tests/builder/src/fixtures/worlds.rs", "hearth/tests/builder/src/simulated.rs", "hearth/tests/builder/tests/suite/bounds.rs", "hearth/tests/builder/tests/suite/memory.rs"]
effort = "high"
minutes = 45
+++

# A world test: a client that stops reading

## Goal

`answers.room()` is the one room check that does real work: it stops render outrunning a client that has stopped reading. `uncap` replaces it with a watermark. Before the check goes, the behaviour gets its world test, which passes on the code as it stands, on today's capped `answers`: a stuck client and many targets, the peak under a named ceiling, and every record printed once the output moves.

Acceptance:

- `simulated::checked_stuck` and `a_client_that_stops_reading_holds_render_under_its_ceiling` exist, in the shapes below, and the test passes.
- The test asserts that the output was held and released once; that at the release the client had nothing to do but wait (`Host::work_pending` was false), so that render had stopped; that every target prints, once and in order, after the release; and that the client exits 0.
- **It bites:** with `limits.queue` raised to 4,096 in the test, so that `answers` no longer holds render back, the world fails on the ceiling. The report gives both peaks.
- The test costs under 0.3 CPU s, or the report says why it must cost more, with the measurement.
- Every test in `hearth-builder-world`'s focused suite passes. No other world's behaviour changes: what isn't held answers as it does today.

## Inputs

- `hearth/docs/plan/hearth-memory-6/notes/common.md`
- `hearth/docs/design/domain/render.md`, 8.4: the watermark
- `hearth/docs/design/domain/testing.md`, 8.4 and 8.5: the row "a client that stops reading"
- `skein/docs/foundation/testing-strategy.md`, 6: a ceiling is a quarter above the peak, rounded up to two significant digits
- `skein/docs/foundation/programming-model.md`, 7.1
- `hearth/docs/development/debt/D001-room-checks-and-max-out.md`, "The service": the row on `answers.room()`

## Owns and may change

- **Owns:** the files under `files`.
- **No change to skein or to `hearth::Client`,** and none is needed: see Steps. If the hold can't be built this way, it's a `Question:`, with what stopped it.
- **Other readers:** `bounds.rs` is read by `memory.rs` and `orders.rs`. `QUEUE`, `TARGETS`, `limits()` and `queued()` stay as they are: `uncap` changes them. Add beside them.

## Steps

What a survey of the world found, read from the code and not run:

- The client's output is an append file: `startup_appends`, then `Shared::open_append` (`hearth/tests/builder/src/fixtures/worlds.rs`), and a write arrives in `Machine::step` as an `Ask::Append`. `Shared::step` answers it in the same turn, and is the place to hold it.
- `Machine::step` may simply not answer: the ticket stays open. It runs only on a new call, so it can't release by itself. `World::run_with_inputs` (`skein/testing/skein-world/src/world.rs`) lends `&mut Sim` between iterations, and `Sim::answer` (`skein/crates/skein-sim/src/sim/files.rs`) is public.
- **Release before `limits.io.close_timeout`** (5 s as shipped): that timer is the append's write timeout, and past it the client fails with 75. About 1 s of simulated time is right. The `lock` in `simulated.rs`'s `Options` and `Referee` shows the shape, including clearing it in `act` so that `next_deadline` moves on.
- **Backpressure reaches `answers` late.** io grants room for `limits.io.sends` queued writes (8 as shipped), so about ten records go down as text before `takes()` turns false. `limits.io.sends = 1` leaves about three.
- **The cost of a queued spec** was estimated at about 3 KB (±30%), by hand. Measure it. Render refuses past `limits.domain.render.targets`, and `limits.protocol.git.entries` must hold `6 + 2 × targets` files, with `paths_total` recomputed: `small_limits` gives 64 entries, which holds 29 targets.

1. **The hold.** In `Fake` and `Shared` (`fixtures/worlds.rs`): record the handle `open_append` gives for the path `b"output"`; when armed, keep the first `Ask::Append` on it with its ticket instead of answering it; add a method that takes the held call and answers it through the machine. Not armed, nothing changes.
2. **The world.** In `simulated.rs`, a release time in `Options` and `Referee`:

   ```rust
   /// Meter the client with its output held from its first write until `release`.
   #[must_use]
   pub fn checked_stuck(
       scenario: &Scenario<'_>,
       seed: u64,
       limits: hearth::Limits,
       ceiling: u64,
       release: Time,
   ) -> Outcome<Proc, Shared>
   ```

   It ends in `run_with_inputs`, whose closure answers the held write once the release has come. The referee's deadline holds the world until then, and it asserts at the release that the client's `work_pending` is false.
3. **The scenario.** In `bounds.rs`, beside what stands:

   ```rust
   pub const STUCK_TARGETS: u32 = 64; // Take fewer if the ceiling still bites.
   pub fn stuck_limits() -> hearth::Limits // `queue` 3, `io.sends` 1, and room for the targets
   pub fn stuck() -> Scenario<'static>     // `queued()`'s shape over `STUCK_TARGETS`
   ```

   Share `queued()`'s body rather than copy it, keeping what it gives at `TARGETS` unchanged. If the signal is too small, more files in a shared task grow every spec and the baseline only once. Say so if you do.
4. **The test,** in `memory.rs`:

   ```rust
   #[test]
   fn a_client_that_stops_reading_holds_render_under_its_ceiling() {
       const CEILING: u64 = ...; // A quarter above the peak measured here.
       ...
   }
   ```

   Print the peak once, with `outcome.peaks()`, to set the ceiling, and take the print out.
5. **Bite:** raise `limits.queue` to 4,096 in the test, see the world fail on the ceiling, and restore it. If it doesn't fail, the scenario is too small: grow it until it does.
6. `heavy cargo clippy -p hearth-builder-world --all-targets -- -D warnings`, then `heavy cargo nextest run -p hearth-builder-world --test suite`. Commit and push.

## Report

The peak with the hold and with the bite, the ceiling and how it was set, the targets you settled on, the rule broken and the failure seen, what ran, and the test's CPU seconds. The next task reuses all of this: name anything it should know.

## Answers

2026-11-06 (fabrig/hearth). Expand this task: you may change `Service::work_pending` in `hearth/crates/hearth-service-builder/src/lib.rs`, that function and nothing else in production, as its own commit, before the test's commit. No prerequisite task: the fix and the test that needs it belong together. This is the owner's call, and changes no decision of the user's.

This is a production fault the test found. With a client that has stopped reading, the builder reports work it can't do and spins on the CPU instead of waiting. The commit says so. It changes nothing a user reads: same output, same digests, no timeout changed.

The rule: `work_pending` is true only if a pass would make progress, by the same guards the stages use. `answers` nonempty counts only if `protocol.takes()` would take one; `domain.work_pending()`, and `calls` nonempty, count only if `domain_stage` would run them, that is, if `answers` has room; and so on for any other entrance your reading finds guarded. Share a predicate with the stage where that's clean, so the two can't drift. What's left waits for a completion or a deadline, as any blocked work does.

The risk is the opposite fault: a world that stalls because something runnable reports false. Every test in `hearth-builder-world`'s focused suite and every other hearth world must still pass, with `-p 'hearth*'`. A stall is a finding: fix the predicate, and don't add a wake-up that hides it.

Keep what you've built: the held append, the one-second release, no timeout shortened, no change to the adapter or to skein. Then finish as the spec says: measure the peaks, set the ceiling, do the queue-4096 bite and restore it, and obtain the CPU figure. Say in `## Left` what `uncap` must keep: the same rule, with its watermark as the guard.

2026-11-06 (fabrig/hearth). Yes, the first option: the production scope now includes shared runnable predicates and their stage call sites in `hearth/crates/hearth-service-builder/src/lib.rs`. No raised baseline. Name each predicate for what it says, have `work_pending` and the stage both call it, and change what the stages do in no other way. It's a refactor of the guards plus the fix you've made: amend nothing, add it as its own commit. `max_out` must be at or under its baseline, 39. `uncap` removes these constants next and will rewrite the predicates' bodies around the watermark, so keep them small and don't polish them further. Everything else in the last answer stands.
