+++
id = "sweeps"
part = "sweeps"
kind = "feature"
needs = []
files = [
  "skein/crates/skein-ui/src/tests/date.rs",
  "skein/crates/skein-ui/src/tests/phrases.rs",
  "skein/tests/ui/Cargo.toml",
  "skein/tests/ui/tests/suite/main.rs",
  "skein/tests/ui/tests/suite/fuzzy/date.rs",
  "skein/tests/ui/tests/suite/fuzzy/phrases.rs",
  "skein/crates/skein-json/src/tests/text.rs",
  "skein/crates/skein-http/src/tests/gzip.rs",
  "skein/docs/plan/skein-limits-heavy/figures/sweeps.md",
]
effort = "medium"
minutes = 40
+++

# Four sweeps: two go to fuzzy, two run the cases they name

## Goal

The request names four sweeps under 8.1's first question, and `plan.md`, *Belonging*, has decided them. Two sweep every day there is, and belong in fuzzy. Two stay, at the count their behaviour needs. Every assertion stays as strict as it was.

Acceptance:

- `skein-ui`'s `tests::date::every_wall_day_round_trips_at_midnight` is `fuzzy::date::every_wall_day_round_trips_at_midnight` in `skein-ui-tests`, with every assertion.
- `skein-ui`'s `tests::phrases::every_representable_wall_day_matches_gregorian_rollover` is `fuzzy::phrases::every_representable_wall_day_matches_gregorian_rollover` in `skein-ui-tests`, with every assertion.
- Both run under `--profile fuzzy` and neither runs in the default profile: `cargo nextest list` shows it.
- `known_midnights_follow_gregorian_centuries` also holds that a day's last nanosecond is still that day, which the moved test alone held in focused.
- `skein-json`'s `three_and_four_byte_sequences_are_judged_at_every_boundary` runs its second byte over `edges` too, with its assertion as it is.
- `skein-http`'s `a_huffman_code_is_complete_canonical_and_as_short_as_any` runs four seeds, two for each `rarest`.
- Each smaller count bites, as the common note asks.
- `skein/docs/plan/skein-limits-heavy/figures/sweeps.md` has all four before and after, the moved tests as before in focused and 0 after, with what fuzzy gains; and *Counts* lines for the two that stay.

## Inputs

- `skein/docs/plan/skein-limits-heavy/inputs/request.md`, items 1 and 3, and its *Sweeps* line.
- `skein/docs/plan/skein-limits-heavy/notes/common.md`: how to measure, the figures' form and the rules.
- `skein/docs/plan/skein-limits-heavy/plan.md`, *Belonging*.
- `skein/docs/foundation/testing-strategy.md`, 8: the two suites, and "a scenario that needs randomness runs the few seeds that show its behaviour".

## Owns and may change

Owns the files in the header.

- **`skein/crates/skein-ui/src/tests/phrases.rs` is also `limits-fixtures/view-words`'.** Delete the one test and nothing else. `wall_book`, `wall_shape` and `wall_language` have other readers there and stay. The fuzzy file has its own short copies.
- **`skein-json`'s product code is `json-strings`'.** `skein/crates/skein-json/src/utf8.rs` is changed only for the bite, and restored.
- `skein/tests/ui/Cargo.toml`: its `description` may say it holds the day sweeps too. No dependency is needed: `skein-lib` and `skein-ui` are there, and `skein_ui::date` and `skein_ui::phrases` are public.

Options, each decided:

- **A name the fuzzy file needs isn't public** (`skein_ui::Owner`, say): it's reachable by another public path if the phrases module's readers use one. If it isn't, that test stays in `skein-ui` at its full count, and the figures say why. Add no public item for a test.
- **`skein-ui-tests`' counting allocator** makes the sweeps cost more in fuzzy than they did in focused: that's fine. Give the figure. It's under its guide, and neither `skein_heap` nor the allocator changes.
- **`text.rs` or `gzip.rs` has gained tests by the time you start:** leave them.

## Steps

1. Profile the four as they are: `-p skein-ui --lib` with `tests::date::every_wall_day_round_trips_at_midnight` and `tests::phrases::every_representable_wall_day_matches_gregorian_rollover`; `-p skein-json --lib` with `tests::text::three_and_four_byte_sequences_are_judged_at_every_boundary`; `-p skein-http --lib` with `tests::gzip::a_huffman_code_is_complete_canonical_and_as_short_as_any`.
2. **Move the two day sweeps.** `skein/tests/ui/tests/suite/main.rs` gains:

   ```rust
   mod fuzzy {
       mod date;
       mod phrases;
   }
   ```

   Each new file starts with a `//!` line saying what it sweeps (`testing-strategy.md`, 8), as `skein/tests/http/tests/suite/fuzzy/gzip.rs` does. Remove the tests and any import only they read from their old files.
3. **Hold the last nanosecond in focused:** in `known_midnights_follow_gregorian_centuries`, a day's last nanosecond is `civil` to that date, as the sweep asserts.
4. **Run the second byte over `edges` in `three_and_four_byte_sequences_are_judged_at_every_boundary`.** The loop goes from 1,802,240 cases to 70,400. Every range limit of a second byte in the well-formed table sits on `edges`: `E0`'s `A0`, `ED`'s `9F`, `F0`'s `90`, `F4`'s `8F`, the rest's `80` and `BF`, each with its neighbour past it. Say so in the test's comment, which now reads "each byte after the first". `every_one_and_two_byte_sequence_is_judged_as_the_standard_library_judges_it` stays as it is. `fuzzy::writer::text_is_written_exactly_when_it_is_utf8_and_reads_back_the_same`, in `skein/tests/json`, holds random bytes between the edges, through the public path: name it in the *Counts* line.
5. **Run four seeds in `a_huffman_code_is_complete_canonical_and_as_short_as_any`:** `(0..4)` in place of `(0..20)`. Each `rarest` gets two, because the `zip` cycles. `a_code_deeper_than_its_widest_is_cut_to_it` holds the cut, deterministically.
6. **Bite,** each once, and restore:
   - In `skein/crates/skein-json/src/utf8.rs`, move one second-byte limit by one (`E0`'s low limit, then `F4`'s high limit), and see the test fail at that byte.
   - In `skein-http`'s `huffman` (`skein/crates/skein-http/src/gzip.rs`), lengthen a symbol's width by one where no cut is needed, and see a seed of the four fail on `least`, or the helper's own checks.
7. Write the figures.
8. The implementer's text's end of a run: `cargo fmt`, old forms, `heavy cargo clippy -p skein-ui -p skein-ui-tests -p skein-json -p skein-http --all-targets -- -D warnings`. Then `heavy cargo nextest run -p skein-ui -p skein-ui-tests -p skein-json -p skein-http`, and `heavy cargo nextest run -p skein-ui-tests --profile fuzzy`. Commit and push.

## Report

The before and after of each, what each bite showed, what fuzzy gains, and any option you took. Your implementation and check minutes apart.
