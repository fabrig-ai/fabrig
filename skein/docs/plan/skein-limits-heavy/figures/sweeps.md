# sweeps: figures

| Row | Test | Before (Ir) | After (Ir) | Cause | What changed |
|---|---|---:|---:|---|---|
| `skein-ui` `tests::date` | `tests::date::every_wall_day_round_trips_at_midnight` | 896,193,271 | 0 | Parsing every day's text costs 442,777,692 inclusive instructions, formatting 320,866,908 | Moves whole to fuzzy: a sweep over every representable day, rather than a named scenario |
| `skein-ui` `tests::phrases` | `tests::phrases::every_representable_wall_day_matches_gregorian_rollover` | 1,090,220,691 | 0 | Repeated `Phrasebook::say` costs 767,847,787 inclusive instructions | Moves whole to fuzzy: comparison against a naive Gregorian counter |
| `skein-ui-tests::suite` `fuzzy::date` | `fuzzy::date::every_wall_day_round_trips_at_midnight` | 0 | 902,606,331 | The complete calendar and text round trips | Fuzzy gains every assertion over all 213,504 representable days, including the partial final day |
| `skein-ui-tests::suite` `fuzzy::phrases` | `fuzzy::phrases::every_representable_wall_day_matches_gregorian_rollover` | 0 | 1,900,707,510 | Saying and building expected words for every day, now under the world's counting allocator | Fuzzy gains the complete Gregorian comparison over the same 213,504 days |
| `skein-ui` `tests::date` | `tests::date::known_midnights_follow_gregorian_centuries` | unknown | unknown | Not profiled: five named dates | Also checks that each day's last nanosecond still maps to that date |
| `skein-ui` `tests::phrases` | `tests::phrases::dates_and_moments_use_utc_calendar_words_in_every_wall_format` | unknown | unknown | Not profiled: unchanged focused holder | Stays: named leap days, century rollover, year end, and the largest wall instant |
| `skein-json` `tests::text` | `tests::text::three_and_four_byte_sequences_are_judged_at_every_boundary` | 939,051,364 | 43,028,377 | Repeated `checked` costs 579,302,752 inclusive instructions before; the second byte enumerates interiors as well as limits | Second byte uses the same ten edges as subsequent bytes; every equality assertion stays |
| `skein-http` `tests::gzip` | `tests::gzip::a_huffman_code_is_complete_canonical_and_as_short_as_any` | 768,988,106 | 152,422,834 | Repeated sorting in the plain `least` tree builder costs 542,600,173 inclusive instructions before | Four seeds, two for each `rarest`, retaining all code and weight checks |
| `skein-http` `tests::gzip` | `tests::gzip::a_code_deeper_than_its_widest_is_cut_to_it` | unknown | unknown | Not profiled: unchanged deterministic cut holder | Stays: Fibonacci counts force both width cuts |

Measured alone through `heavy cargo nextest run`, with
`NEXTEST_DOUBLE_SPAWN=0` and the x86-64 runner
`valgrind --tool=callgrind`, in unoptimized test builds on the same
builder. Each figure is the test process's Callgrind instruction total,
not its enumeration processes. Before and after use the same named-test
command for JSON and HTTP; the UI after commands necessarily select the
new package, suite binary and fuzzy profile. Zero means no execution in
that suite, rather than a measured empty process. No settings changed.
The UI fuzzy gains total 2,803,313,841 instructions. Inclusive costs in
the table overlap and must not be added.

`heavy cargo nextest list -p skein-ui -p skein-ui-tests
--message-format json` marks both new names as `mismatch` with reason
`default-filter`; adding `--profile fuzzy` marks both as `matches`.
Neither old library name remains.

## Found and not fixed

- The fuzzy phrases sweep pays 440,549,378 inclusive instructions in
  `skein_heap::Counting::alloc` and 384,535,557 in its `dealloc`.
  The allocator belongs to `skein-world-costs`; changing it is outside
  this task. Its presence accounts for the increased fuzzy figure, and
  its guide remains unchanged.
- The calendar checks and phrase rendering remain in full in fuzzy:
  their exhaustive comparison is the behaviour the sweeps hold.
- The remaining Huffman reference builder still sorts repeatedly.
  Even eliminating its entire remaining test would save only about
  0.012 CPU s by the request's 0.058 s baseline and the instruction
  ratio, below the common note's 0.05 s stop rule.
- Test-process startup remains included in the figures and unchanged;
  the request assigns that cost to other plans.

## Tests that went

- `skein-ui::tests::date::every_wall_day_round_trips_at_midnight`:
  held whole by `skein-ui-tests::suite::fuzzy::date::every_wall_day_round_trips_at_midnight`,
  which passed at `4327ac3b36`. The focused named-day holder
  `tests::date::known_midnights_follow_gregorian_centuries` also passed
  with the last-nanosecond assertion at that commit.
- `skein-ui::tests::phrases::every_representable_wall_day_matches_gregorian_rollover`:
  held whole by `skein-ui-tests::suite::fuzzy::phrases::every_representable_wall_day_matches_gregorian_rollover`,
  which passed at `4327ac3b36`. The unchanged focused holder
  `tests::phrases::dates_and_moments_use_utc_calendar_words_in_every_wall_format`
  also passed at that commit.

## Counts

- `tests::text::three_and_four_byte_sequences_are_judged_at_every_boundary`:
  1,802,240 comparisons become 70,400:
  `64 * 10 * 10 * (1 + 10)` instead of
  `64 * 256 * 10 * (1 + 10)`. Each byte after the first visits the
  continuation limits and their outside neighbours: E0's A0/9F,
  ED's 9F/A0, F0's 90/8F, F4's 8F/90, and 80/7F and BF/C0.
  Each complete sequence is still compared with the standard library.
  The exhaustive one- and two-byte test is unchanged;
  `fuzzy::writer::text_is_written_exactly_when_it_is_utf8_and_reads_back_the_same`
  in `skein-json-world` retains random text checking through the public
  writer path. Bite: E0's lower limit A0 changed to A1 failed on
  `E0 A0 80` (false versus true); restored, then F4's upper limit 8F
  changed to 90 failed on `F4 90 80 80` (true versus false).
  Both production mutations were restored, and the reduced test passed.
- `tests::gzip::a_huffman_code_is_complete_canonical_and_as_short_as_any`:
  20 seeds become four, seeds 0 and 2 at `rarest = 1000`, seeds 1 and 3
  at `rarest = 1`. Every seed still checks completeness, canonical
  decoding, width limits, frequency ordering and weight against `least`;
  the deterministic `a_code_deeper_than_its_widest_is_cut_to_it` keeps
  both cut cases. Bite: lengthening the first used symbol by one bit
  when no depth needed cutting made seed 0 fail the helper's complete
  code assertion: 32,736 slots filled versus 32,768. The production
  mutation was restored; both the four-seed test and cut holder passed.
