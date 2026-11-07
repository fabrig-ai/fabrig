+++
id = "lib-counts"
part = "lib-memory"
kind = "feature"
needs = []
files = [
  "skein/tests/lib/tests/suite/memory.rs",
  "skein/docs/plan/skein-limits-heavy/figures/lib-counts.md",
]
effort = "medium"
minutes = 30
+++

# `skein-lib-tests`' `memory`: a light use need not last a million turns

## Goal

`skein-lib-tests::suite`'s `memory` is 28 tests and 1.56 CPU s, of which three metered loops of a million turns are 1.37. Each holds that a large container used lightly stays under 1024 bytes. Lower each count to what that behaviour needs, and show why by what the test asserts. Every assertion stays as strict as it was: each turn stays metered, the ceiling stays 1024.

Acceptance:

- `a_ring_used_lightly_holds_what_it_holds` and `a_queue_with_a_large_capacity_used_lightly_holds_what_it_holds` run 4096 turns.
- `an_intake_with_a_large_capacity_used_lightly_holds_what_it_holds` holds an intake of capacity `1 << 14` for `1 << 14` turns.
- A `const` beside each test names the count, with its reason in a comment and a compile-time check of that reason, as in step 2.
- Each smaller count bites, as the common note asks.
- `skein/docs/plan/skein-limits-heavy/figures/lib-counts.md` has the three, the row's median test and the row's next-costliest, with *Counts* lines.
- The next cause's gain is under about 0.05 CPU s.

## Inputs

- `skein/docs/plan/skein-limits-heavy/inputs/request.md`, item 3.
- `skein/docs/plan/skein-limits-heavy/notes/common.md`: how to measure, the figures' form and the rules.
- `skein/docs/plan/skein-limits-heavy/plan.md`, *Belonging*.
- `skein/docs/design/lib.md`, 10: what a container's price holds.
- `skein/docs/foundation/programming-model.md`, 6.3.

## Owns and may change

Owns the files in the header. Nothing else reads these tests.

Doesn't change: `skein_heap` (`Meter`, `Counting`: `skein-world-costs`), `skein-lib`'s product code, unless a profile puts a small cause there, and the other tests' capacities (`an_unused_*`, `a_list_clone_*`), which cost little because nothing is allocated.

Options, each decided:

- **The queue's capacity stays 15,000,000.** It allocates nothing up front, so it costs nothing, and the structure stays far past its ceiling.
- **A cost in `Metered` itself:** it's checked each turn, and that stays. A cost inside `skein_heap` goes under *Found and not fixed*.
- **Another test in the row is above the stop rule:** it falls under item 4, not 3. Fix a repeated setup in the test; leave a count unless you can show the same reason for it, and say so in *Counts*.

## Steps

1. Profile, with `-p skein-lib-tests --test suite` and the tests' full names (`memory::NAME`, or as `cargo nextest list` shows them): the three, `a_queue_filled_to_its_capacity_stays_within_its_price` and the median test `cargo nextest list` and the profile point at.
2. Change the counts. The reasons, which the comments give in the code's words:

   ```rust
   /// A use that stays under this ceiling.
   const LIGHT_CEILING: u64 = 1024;
   /// Turns of two pushes and two pops of a `u64`. A ring within the ceiling
   /// holds at most 128 of them, and this many pass through it, so it wraps
   /// at least 63 times; one that never reclaimed would pass the ceiling at
   /// its 65th turn.
   const LIGHT_TURNS: u64 = 4096;
   const _: () = assert!(LIGHT_TURNS * 2 * 8 >= 64 * LIGHT_CEILING);
   ```

   - **The ring and the queue** share `LIGHT_TURNS`. Their loops are otherwise as they are.
   - **The intake:** capacity and turns are one constant, `INTAKE_CAPACITY: u32 = 1 << 14`. That's 16 times the ceiling, so an intake that allocated its capacity fails its first step: a `const _` checks that. Two bytes a turn pass twice its capacity through it, as a million turns passed about twice a megabyte, so one that counted what has passed against its capacity is refused by `expect("room for two bytes")`: one constant for both keeps that so.
   - `Metered::new` names the capacity it's given, so the intake's label follows.
   - `Some(1024)` and `assert!(metered.most <= 1024)` read `LIGHT_CEILING`, and stay.
3. **Bite,** each once: in the ring and the queue, drop the second `pop` of a turn (and its comparison), and the meter's ceiling fails within `LIGHT_TURNS`. In the intake, take one byte of the two with `Read::Fill(1)` (its comparison following), so a byte stays behind each turn, and see the ceiling fail; then make the capacity 1024, and see the `const _` refuse it. Restore each. The *Counts* lines say what you saw.
4. Write the figures.
5. The implementer's text's end of a run: `cargo fmt`, old forms, `heavy cargo clippy -p skein-lib-tests --all-targets -- -D warnings`, and `heavy cargo nextest run -p skein-lib-tests`. Commit and push.

## Report

The before and after of each, what each bite showed, and what else the row's median pays for. Your implementation and check minutes apart.
