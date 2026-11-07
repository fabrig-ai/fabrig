# gallery-print: figures

| Test | Before (Ir) | After (Ir) | Cause | What changed |
|---|---:|---:|---|---|
| `temper-client-gallery::lib::tests::print_writes_every_page_and_the_bundle` | 31,170,701,091 | 8,292,374,426 | Two complete prints; repeated English phrasebook loading and bounds derivation | Prepare words and bounds once per print; print once onto an existing gallery, retaining every page, index and bundle assertion; hold whole-directory replacement separately |
| `temper-client-gallery::lib::tests::each_state_holds_what_its_name_says` | 12,608,794,546 | 5,534,470,586 | View construction is 93.28% before; words and bounds are rebuilt for each state | One pass shares prepared words and bounds and holds all four additional rules below |
| `temper-client-gallery::lib::tests::navigation_links_have_addresses_and_install_action_links_keep_their_binding` | 12,675,806,998 | 0 | View construction is 92.77% before | Row of `each_state_holds_what_its_name_says`; its helper costs 204,188,704 Ir, included in that holder |
| `temper-client-gallery::lib::tests::no_named_control_draws_a_text_child_in_any_state` | 12,470,818,651 | 0 | View construction is 94.30% before | Row of `each_state_holds_what_its_name_says`; its helper costs 5,264,990 Ir, included in that holder |
| `temper-client-gallery::lib::tests::no_state_reads_of_zero` | 12,573,446,064 | 0 | View construction is 93.93% before | Row of `each_state_holds_what_its_name_says`; its helper costs 104,150,536 Ir, included in that holder |
| `temper-client-gallery::lib::tests::each_chat_reads_its_allotment` | 1,966,208,085 | 0 | View construction is 93.73% before | Row of `each_state_holds_what_its_name_says`; its helper costs 1,096,040 Ir, included in that holder; its existing chat predicate and spend rows stay |
| `temper-client-gallery::lib::tests::the_browser_draws_each_model_name_once` | 54,346,700 | 54,341,265 | Single-sample floor: English loading is 25,355,847 Ir, bounds 6,787,821 Ir; samples array only 1,220,765 Ir | Stays: one sample needs its own view and page; public single-page construction moves its words without cloning them |
| `temper-client-gallery::lib::tests::replacing_a_gallery_removes_the_whole_previous_print` | unknown | 2,533,331 | New test, no before profile; the second complete rendering previously paid for directory replacement | Calls the same private replacement function as `print`, first for a missing directory, then over an index, stale page and nested old bundle; checks the replacement is an empty directory |

After zero means the separate test process went away, not that its assertions
went away. The helper counts are inclusive calls in the holder's after profile;
they must not be added to the holder again. Together, print, the five semantic
checks and the new replacement test fall from 83,465,775,435 to 13,829,378,343 Ir
(83.43%). The single-sample floor is unchanged within 0.02%.

## Measurement and causes

Unoptimized test builds, one x86-64 builder throughout, callgrind through nextest
with double spawn disabled. Each profile selects exactly one test. The command
for before, the intermediate shared-input stage, and after was:

```sh
NEXTEST_DOUBLE_SPAWN=0 \
CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER="valgrind --tool=callgrind --callgrind-out-file=$PWD/target/limits-heavy/STAGE-NAME.%p.cg" \
heavy cargo nextest run -p temper-client-gallery --lib \
  --profile fuzzy --ignore-default-filter -E 'test(=tests::NAME)'
callgrind_annotate --inclusive=yes --auto=no --threshold=100 target/limits-heavy/STAGE-NAME.PID.cg
```

The largest file is the selected test; smaller files are test discovery. The
fuzzy profile supplies callgrind's longer wall allowance, with the same explicit
selection before and after. Every selected profile passed. No slow tier ran.

The initial focused run passed 31 tests. Its four full-state semantic passes
were `each_state_holds_what_its_name_says`, navigation links, zero allotments and
named-control text. The chat-allotment pass visits its existing subset. Each
profile showed that building and dropping the fixture dominated its own check.
Before print, `Words::english` costs 14,606,021,112 Ir, `Limits::of` costs
3,909,786,048 Ir, `samples::domain` costs 2,346,472,997 Ir and view `build` costs
946,561,569 Ir. Page construction is 30,815,748,063 Ir inclusive, containing the
fixture; these overlapping costs are not summed. The bundler costs only
52,954,871 Ir and all three samples-array constructions together 3,667,028 Ir.

First, sharing prepared inputs reduced print to 16,332,645,643 Ir, the existing
state-holder test to 5,221,186,688 Ir and navigation to 5,291,184,326 Ir. In the
latter two, construction plus words cloning still cost 84.90% and 83.21%,
respectively, before dropping the view. The four cheap rules therefore became
helpers called by the state-holder test, whose loop still reads
`samples::samples()`, with no new list of names. Every original assertion stays;
failures identify the state and rule. The named helpers keep their original
names, including the named-control helper cited by the client design's status.

Then the second rendering was removed from the print test. `print` still calls
`replace_gallery` before writing anything. Its full-output test starts with an
existing owned gallery and a stale page, so the production call still exercises
replacement. The small replacement test holds its first and second preparation,
including the removal of the old index and nested bundle. Every first-print
assertion remains, across all current and future samples. There is no second
full render in the tests.

The new test's mutation check replaced `remove_dir_all` with a metadata read.
It failed at "whole previous print removed": three entries remained instead of
zero. Restoring removal made that test pass alone. The other rule checks only
moved, so needed no new mutation check.

The final focused package run passed 28 tests across two binaries. Clippy with
warnings denied passed for all gallery targets. Formatting and the tree's
old-forms check passed; no old-form baseline changed. No dependent package names
the gallery in Cargo metadata. Samples, worlds and the names file were untouched;
no merge with another draft was needed.

## Found and not fixed

- Owned phrasebook copying: `ViewInputs::clone` costs 1,913,349,338 Ir in print
  and 1,934,587,884 Ir in the state holder, almost entirely `Words`/`Phrasebook`
  cloning. Each independently constructed `View` owns its words. Avoiding those
  copies needs a change to phrasebook ownership or a view reset/reuse API that
  preserves each state's bindings, clock and sign-in configuration, beyond
  sharing the gallery's checked inputs. This is about 0.53 CPU s when priced
  against the request's original test figures, so it is explicitly left here.
  Files: `temper/tools/gallery/src/lib.rs`,
  `temper/crates/temper-client-view/src/root.rs`,
  `skein/crates/skein-ui/src/phrases.rs`.
- Repeated view validation: `Words::capacity` costs 989,777,800 Ir in each of
  print and the state holder, mostly through `View::with_sign_in`'s validation.
  It is about 0.27 CPU s on the same pricing. Avoiding it needs cached checked
  capacity or a constructor/validation change in client-view and its phrasebook,
  whose public book currently remains independently owned. The shared fixture
  floors are also being investigated by `limits-fixtures`. Files:
  `temper/crates/temper-client-view/src/limits.rs` and `src/words.rs`.
- Stateful world construction: `samples::domain` costs 1,046,177,354 Ir in print
  and 1,149,900,341 Ir in the holder (including navigation's existing inbox
  verification), about 0.31 CPU s together on that pricing. A world is stepped
  for its address and subsequent acts; this is not one identical root copied
  for every state. Sharing the mock-document/world construction would reorganize
  `temper/tools/gallery/src/worlds.rs` and its dispatch with `src/samples.rs`.
  Leave that work until after `look-round-13`, as the task specifies.
- HTML rendering remains the print check's own cost: the largest inclusive
  `Page::render` entry is 4,179,957,236 Ir in the one full print (about 0.56 CPU s
  on that pricing). Every page still needs rendering. The profile establishes
  this cost, not a smaller equivalent renderer; further work would need a
  renderer/batching profile and changes in `skein/crates/skein-ui-web/src/page.rs`,
  beyond the gallery and the permitted bundler fix. No state or output assertion
  was removed to avoid it.

The remaining local checks are below the stop rule: the four row helpers total
314,700,270 Ir including navigation's domain verification; the one-print bundler
is 26,485,195 Ir; and constructing the entire samples array costs 1,220,765 Ir in
the single-sample floor. None is near 0.2 CPU s at the request's prices.

## Tests that went

- `tests::navigation_links_have_addresses_and_install_action_links_keep_their_binding`:
  held by `tests::each_state_holds_what_its_name_says`, which passed at `bbf0fb6cd3`.
- `tests::no_named_control_draws_a_text_child_in_any_state`:
  held by `tests::each_state_holds_what_its_name_says`, which passed at `bbf0fb6cd3`.
- `tests::no_state_reads_of_zero`:
  held by `tests::each_state_holds_what_its_name_says`, which passed at `bbf0fb6cd3`.
- `tests::each_chat_reads_its_allotment`:
  held by `tests::each_state_holds_what_its_name_says`, which passed at `bbf0fb6cd3`.
