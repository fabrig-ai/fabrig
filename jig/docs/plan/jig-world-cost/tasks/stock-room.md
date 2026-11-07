+++
id = "stock-room"
part = "stock-room"
kind = "feature"
needs = ["focused-cuts"]
files = [
  "jig/tests/core/tests/suite/settings.rs",
  "jig/tests/core/tests/suite/fuzzy/settings.rs",
  "jig/tests/core/tests/settings/support.rs",
]
effort = "medium"
minutes = 25
+++

# The stock profiles' room: one world at Small's bound, Standard's drive in fuzzy

## Goal

`stock_profiles_hold_a_day_of_deciding_requests` (`jig/tests/core/tests/suite/settings.rs`) drives 10,000 requests through two profiles, 8,000 of them through `Standard`. What it checks is that each stock profile's room holds a day of a stated size.

Three things show that, each in its own place. Read at the cut:

- **The arithmetic on the stock figures is already a table test,** for both profiles: `profiles_retain_a_day_of_deciding_answers_with_fivefold_room` (`jig/crates/jig-protocol-config/src/tests/profile.rs`) holds `people.requests == people.people * 20 * 5 * 5` and a day's retention. Add no second one, and leave that file alone: another plan is changing that directory.
- **One world at `Small`'s bound and its restart** stays in focused: every request answered up to the bound, the last still answered, the first replayed after a cold restart.
- **What only `Standard`'s drive shows** goes to fuzzy: its whole room of retained answers stays under `REQUEST_HEAP_CEILING` (`jig/tests/core/tests/settings/support.rs`). The ceiling's 16,224,965-byte peak was recorded over both workloads together. `Small`'s world holds a quarter of `Standard`'s rows, so by reading the code, not by measuring, it wouldn't come near the ceiling in focused.

Acceptance:

1. `settings::a_small_profile_s_whole_room_of_answers_holds_and_restarts` passes in focused, and the old test's name is gone.
2. `fuzzy::settings::a_standard_profile_s_whole_room_of_answers_stays_under_the_heap_ceiling_and_restarts` passes in the fuzzy profile and isn't selected in focused.
3. Both run one shared driver, so `Small` and `Standard` make the same checks as today: each answer, the last request's `VisibilitySet`, the restart, the replay of the first request.
4. `REQUEST_HEAP_CEILING` keeps its value.
5. The report says which fuzzy test holds what, and why.

## Inputs

- `jig/docs/design/domain/testing.md`, 9: the focused suite runs a few cuts and seeds per scenario, the fuzzy one sweeps.
- `skein/docs/foundation/testing-strategy.md`, 8 and 8.1.
- `jig/docs/plan/jig-world-cost/notes/common.md`.
- `jig/docs/plan/jig-world-cost/plan.md`, Decisions.

## Owns and may change

- **Owns:** the spec's `files`.
- **Readers:** `RequestWorld` and `configuration` are read across `settings.rs` and `fuzzy/settings.rs`. Add to `support.rs`; change no existing helper there.
- **Departure from the implementer's text:** none.
- **Options:**
  - The bare `4` and `16` go: the driver takes `jig_protocol_config::profile(profile).core.people.requests`, which the table test holds to the stated size.
  - If you find something else only `Standard`'s drive shows, it stays in the fuzzy test, and you say so in the report.
  - `REQUEST_HEAP_CEILING`'s comment says "with both stock workloads". Make it say "with the stock workloads", since one process no longer runs both. Don't re-measure it.

## Steps

1. In `support.rs`, beside `RequestWorld`:

   ```rust
   /// A stock profile's whole room of deciding requests answered, a cold
   /// restart, and the first request replayed (the heap ceiling checks each step).
   pub(super) fn whole_room_of_answers(profile: jig_protocol_config::Profile) {
       // The body of today's loop, for one profile.
   }
   ```
2. In `settings.rs`, the test becomes:

   ```rust
   #[test]
   fn a_small_profile_s_whole_room_of_answers_holds_and_restarts() {
       support::whole_room_of_answers(jig_protocol_config::Profile::Small);
   }
   ```
3. In `fuzzy/settings.rs`, the same with `Profile::Standard`, under the name in acceptance 2, with a one-line comment saying what it adds.
4. A moved test needs no break-it-once.
5. Narrow runs of each test by name (fuzzy with `--profile fuzzy`), then the implementer's text's closing steps for `jig-core-world`. Commit and push.

## Report

The implementer's report, plus: the fuzzy test, what it holds that focused doesn't, and any other thing you found `Standard`'s drive to show.
