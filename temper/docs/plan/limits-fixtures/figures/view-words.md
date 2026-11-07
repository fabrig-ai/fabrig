# view-words: figures

Base `16c207bff23c4da176d17e59fd4f690c6e23bafa`, machine
`debian-16gb-hel1-8` (reported through the build wrapper), 2026-11-06 UTC.
The after figures describe the implementation committed with this file.

| Row | Test | Floor or median | Ir before | Ir after | What was built before the first assertion |
|---|---|---|---:|---:|---|
| `temper-client-view --lib`, outside `tests::limits` | `tests::frame::known_sign_in_ways_ignore_unknown_words_and_duplicates` | Cheapest (one of three tied by selection time) | 3,358,477 | 3,407,631 | Process/harness, parsed sign-in Ways and its copied origin. No Words, View, domain world or UI limits. |
| Same binary and selection | `tests::accounts::a_sign_in_in_an_omitted_row_stays_visible_without_duplicating_drawn_rows` | Median | 93,159,556 | 74,315,302 | One-account document; account-root domain state, watches and delivered facts; complete terminal English book and one deep clone; composed UI limits, a View and bindings; accounts semantic tree and extracted text. Its second assertion builds another root/View with zero visible accounts and a second clone. |
| Same binary, uses `fixtures::words()` | `tests::a_long_forge_path_shows_its_last_whole_segment` | Cheapest words fixture user | 45,443,546 | 38,117,793 | Complete terminal English book, one deep clone, three owned path segments, UI builder, resource tree and collected text. No View or `Limits::of`. |

Selection ran all 1,427 library tests outside `tests::limits` on the unchanged
base and sorted focused PASS durations. Durations only selected names; they
are not performance evidence. Earlier entries than the words row were
inspected: they parse values, inspect style names, decode bindings, or build
domain state without the words fixture. Each profile ran one exact name with
nextest's Callgrind tracer, `NEXTEST_DOUBLE_SPAWN=0`, default profile and
`TEMPER_BUNDLE=none`. Inclusive and exclusive annotations were read. Ir is
the whole process's `PROGRAM TOTALS`, including loading and teardown; the
setup column describes the inputs to its first assertion.

Words saves **7,325,753 Ir (16.1%)**, and the median saves **18,844,254 Ir
(20.2%)**. The floor grows 49,154 Ir (1.5%): added loader machinery enlarges
relocation work. The original binary floor was not a phrasebook cost.

## Causes

Inclusive figures overlap and must not be added. English contains
construction and load; load contains ordering and checks; maximum scans
contain ID construction/lookups; clones contain nested allocations.
W means the words row, M the median, F the floor.

| Cause | Where | Ir per test, on which test | Fixed in | Or what fixing it would take |
|---|---|---:|---|---|
| Relocations | `_dl_relocate_object` | F: 2,644,287 → 2,693,383 (78.7% → 79.0%); also paid by W/M | Retained | Binary/loading work, not fixture work; build/link changes are excluded. |
| Complete English startup | `Words::english` | W: 32,916,144 → 25,558,177; M: 32,918,433 → 25,559,488 | This implementation, through the product loader | Same book in product and tests. |
| Repeated ordering of the same ID sequence | `Phrasebook::load` | W load: 24,221,512 → 16,863,413; original two sorts: 18,448,343; after shape ordering: 9,725,414, language permutation: 1,172,579 | This implementation | Order shapes through bounded Map keys, retain source positions, and reuse them only after checking the language's original ID sequence. Independently ordered languages retain their independent sort. Further savings require already ordered vocabulary writers plus a checked ordered-input path. |
| Nested ID comparisons in lookup | `Phrasebook::capacity` / `say` | M, same intermediate scan count: `Words::capacity` 31,748,760 → 27,082,408; final scan total: 20,311,910 | This implementation | Private u64 keys preserve all 32 owner bits and 16 phrase bits. Public IDs and Segment are unchanged. |
| Repeated whole-vocabulary maximum scan | `Limits::of` | M: 23,828,048 → 13,557,736, including cheaper lookup | This implementation | Compute the phrase maximum once per invocation. |
| Remaining maximum/presence scans | `Words::capacity`, `limits::valid`, `View::new` | M: scans 20,311,910; `valid` 13,543,512; `View::new` 6,773,820 (overlapping) | Partly reduced | A cached vocabulary summary requires changing `Words::capacity` and its representation, beyond the owned English constructors. Publicly replaceable book/connector fields mean a cached maximum or presence boolean cannot simply be trusted after mutation. Removing validation changes refusals. |
| Nested fixture clone | `fixtures::words`, `Words::clone` | W: 6,760,940 → 6,740,696; M, two clones: 13,420,191 → 13,401,334 | Retained | Borrowing could save clones in readers that only borrow Words, but the required fixture returns owned Words and View consumes it. A broad caller/ownership change or flattened private loaded-book representation is needed. Rc/Arc in step code is excluded. Removing the cache rebuilds the 25.6 million Ir book for repeat calls. |
| Shapes, forms and literal allocations | `english_language` | W: 8,691,859 → 8,691,991; includes `hole_bytes` 1,038,507 and domain `extend_change` 947,865 | Retained | Substantial savings require a different construction representation across vocabulary writers, keeping every word and hole. The jig/domain builders are not the dominant independent cause. |
| Whole-book validation/capacity arithmetic | `check_forms`, `form_capacity` | W: 1,188,027 and 702,498, unchanged | Retained | Every hole, plural form and overflow remains checked. Fusing passes must preserve whole-load error precedence; occurrence counters need a bounded representation. |
| Book teardown | dropping owned Words | W: 1,945,186; M: 3,874,423 after | Retained | Flatten the many owned allocations, as for cloning. |
| Domain setup and semantic rendering | `fixtures::account_root`, `shown_at` | M after: roots 4,017,336; `shown_at` 70,387,990 includes the setup above | Retained | Domain facts, differently bounded Views and rendered text are the assertions' inputs. Removing them weakens this test; domain setup needs its own profile. |
| Harness and floor parsing | test list, argument parsing, thread setup, sign-in parsing | F after subtracting relocations: 714,190 → 714,248 | Retained | Harness/runner work is outside this task; sign-in parsing is the rule under test. |

## Stop rule and remaining work

The request's 22.7 CPU s row makes about 0.227 CPU s the 1% threshold.
Using that whole row as a conservative reach bound, the retained clone,
construction, ordering, maximum scans, checks and teardown each exceed the
threshold; each has its required repair above. These are estimates, not
measured CPU savings: profiles cannot establish call counts for every other
test. There is no claim that every expensive operation was eliminated.
The remaining local `Limits::of` arithmetic outside its scan is 14,224 Ir
in M; even assigning it the entire row gives about 0.0044 CPU s, below the
threshold. Further work is the ownership/representation or cross-vocabulary
work above, rather than another local limits tweak.

Neither `temper-docs` nor `temper-fixtures` execution appears as a setup
cause in these profiles. Both compile as dependencies; the accounts setup
constructs its domain data directly.

The contract touched is `skein/docs/design/ui.md`, section 9: whole-language
checks, owner-scoped IDs, sayings, refusals and longest-form capacities.
No public type or signature changes. `Segment::Text` remains `Box<[u8]>`.
`PHRASES = 1887`, `SEGMENTS = 3316`, and `ENGLISH_BYTES = 46193` stay unchanged.
Design/status documents and `src/tests/limits.rs` are untouched.

New focused tests are
`tests::phrases::duplicate_ids_are_refused_before_conflicting_holes_or_forms`
and `tests::phrases::shared_declaration_order_and_independent_order_keep_every_id_bit`.
Mutations proved they reject the first encountered duplicate instead of the
least duplicate ID, discarded owner bits, and a skipped language permutation.
Restoration passes all nine phrasebook tests.
