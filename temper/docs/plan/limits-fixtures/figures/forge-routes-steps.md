# forge-routes-steps: figures

Base `ed1e8e03f4749b719abe98924a12b6605307ca98`, machine
`debian-32gb-hel1-14` (through `heavy hostname`), 2026-11-07 UTC.
Before profiles precede every edit. Final profiles use the completed source
changes, the same unoptimized `temper-engine-domain-world --test suite`,
default nextest profile, Callgrind tracer and disabled double spawn.
`callgrind_annotate` supplies both inclusive and self annotations; the raw
profiles and logs stay under ignored `target/lf/`.

The pristine `forge_routes::` run passed 425 tests. Sorting its durations,
with names breaking ties, selects the first and 213th tests below. The
fixture task's hint floor was second in this run; both of its forge samples
are retained as comparison anchors. The resolution story calls
`change_world`, through `change_world_funded`.

| Row | Test | Floor or median | Ir before | Ir after | What was built before the story's checks |
|---|---|---|---:|---:|---|
| forge_routes | `forge_routes::hints::both_fresh_base_facts_remain_dependencies_when_the_pull_and_branch_disagree` | floor | 14,653,106 | 14,883,008 | Forge-only hint world: fake repository, two changes and subscriptions; a delayed base read and an external branch advance. |
| forge_routes | `forge_routes::b2_tree::directory::an_advisor_lists_the_root_and_finds_src_main_rs` | median | 343,153,206 | 248,034,702 | Init model and effective limits, stock defaults, real store and first start, cold root, sign-in/adoption, worker hello and chat; delegates work and its advisor, then reads root, directory and source through a scripted model. |
| change_world | `forge_routes::delegates::a_resolution_delegate_carries_its_base_as_hexadecimal_text` | story | 135,827,581 | 134,885,818 | Configured world, real first start/store, adoption, hello, chat, change and assigned producer; then branch/base advances and the producer's answer lead to a resolution delegate. |
| domain-fixture anchor | `forge_routes::hints::a_named_issue_still_gets_its_hint_reads_and_wakes_its_change` | previous floor | 14,078,917 | 14,308,922 | Same forge-only fixture; creates and names an issue, then checks two identical hints. |
| domain-fixture anchor | `forge_routes::questions::a_restart_keeps_an_open_question_where_it_stood` | previous median | 209,740,343 | 177,192,064 | Real first start/store and cold root, adoption, worker/chat/requester, change and producer; asks a question and restarts, retaining its questions and exactly one requester reply. |

The current median saves 95,118,504 Ir (27.72%); the restart/question
anchor saves 32,548,279 (15.52%). The change story saves 941,763 (0.69%).
Both floors increase almost entirely in the loader: `_dl_start` grows by
229,670 Ir, including relocations from 8,570,281 to 8,799,951. No build
setting changed. The current floor's remaining increase is 232 Ir; the
previous floor's is 335 Ir.

Fixture helpers assert their setup too. The construction column describes
the walk into the story, rather than claiming its helpers contain no assertions.

## Beyond construction

The following median partitions are disjoint. The residual includes chat
and delegate setup, the advisor's steps, harness work and teardown. The
model entry is the init model getter, not the scripted LLM's completions.

| Median partition | Ir before | Ir after |
|---|---:|---:|
| Loader, `_dl_start` | 8,678,627 | 8,908,297 |
| Effective limits, `box_limits` (two calls) | 134,169,194 | 67,274,150 |
| Init model, `stock::box_model` (five calls before, one after) | 32,302,792 | 6,610,150 |
| `World::configured_settings`, real first start and cold root | 29,349,822 | 28,063,945 |
| Sign-in and adoption, `World::adopt` | 19,772,895 | 18,456,920 |
| Own steps, remaining setup, harness and teardown | 118,879,876 | 118,721,240 |

The median has no change procedure: its work/advisor delegation and
three path reads are its own story. Its scripted `Model::next` costs
685,106 / 686,051 Ir, and `change_tool::named_read` costs
2,642,024 / 2,643,168. Those checks and steps stay intact.

The change story's disjoint partitions show its repeated walk explicitly.
Sign-in alone, nested within adoption, costs 10,007,910 / 9,475,054 Ir.

| Change story partition | Ir before | Ir after |
|---|---:|---:|
| Loader | 8,678,627 | 8,908,297 |
| Configured world | 16,339,009 | 15,805,761 |
| Sign-in and adoption | 12,204,941 | 11,672,089 |
| Rest of `change_world_funded`: hello, chat, change and producer | 37,190,493 | 37,084,233 |
| Own branch/base advances, resolution, assertions, harness and teardown | 61,414,511 | 61,415,438 |

Its own `next_delegate` still costs 55,090,434 / 55,091,028 Ir. This is
the real producer-answer-to-resolution walk, not eliminated setup.

The question anchor's boundaries overlap: `ask` includes one restart,
and `plan_world` includes construction and adoption. They must not be summed.

| Question anchor boundary | Ir before | Ir after |
|---|---:|---:|
| `plan_world`, through the requester | 60,935,504 | 59,762,552 |
| Sign-in and adoption, within `plan_world` | 12,222,770 | 11,689,878 |
| Change/producer walk, `up_world` minus `plan_world` | 28,070,137 | 28,070,125 |
| Its own question, `ask` | 55,195,455 | 39,324,724 |
| Both `restart_with_charter` calls, including temporary setup/drop before | 39,577,677 | 9,568,400 |
| Full configured worlds across the story | 48,721,829 (three) | 15,805,726 (one) |

Both hint samples have no engine sign-in, adoption or change walk. Their
forge-only fixture includes the connector's adoption. For the current
floor, `hint_world` costs 3,405,799 / 3,405,765 Ir and the own-step/harness/
teardown residual after subtracting that fixture and loader is
2,568,644 / 2,568,910. The previous floor's corresponding figures are
3,405,741 / 3,405,705 and 1,994,549 / 1,994,920.

## Each turn

These are inclusive boundaries, including construction's turns where the
same function is used. They overlap: store application, poll and collect
can be inside tick or flush. Reclaim is the engine root's total over all
callers, not just tick's calls.

| Sample | Boundary | Ir before | Ir after |
|---|---|---:|---:|
| median | `tick_with_reply_cut` | 122,795,435 | 121,337,220 |
| median | `flush` | 47,069,806 | 45,711,236 |
| median | `collect` | 13,234,788 | 11,923,153 |
| median | `Store::apply` | 35,011,621 | 34,217,307 |
| median | `poll` | 29,990,919 | 30,000,899 |
| median | root `domain::step` | 66,536,027 | 66,496,091 |
| median | root `Domain::reclaim` | 421,611 | 421,557 |
| question | `tick_with_reply_cut` | 129,960,926 | 127,727,267 |
| question | `flush` | 92,905,639 | 90,773,750 |
| question | `Store::apply` | 34,914,318 | 28,390,623 |
| question | root `domain::step` | 63,966,768 | 60,784,038 |
| question | root `Domain::reclaim` | 384,443 | 345,881 |
| change | `tick_with_reply_cut` | 100,131,025 | 99,492,507 |
| change | `flush` | 56,206,757 | 55,673,903 |
| change | `Store::apply` | 31,205,751 | 30,833,650 |
| change | root `domain::step` | 59,051,913 | 59,051,897 |
| change | root `Domain::reclaim` | 361,376 | 361,376 |

Reclaim is under 0.3% of each whole test. Poll's inclusive cost mostly
drives real work; it is not evidence of an expensive idle reclaim. Neither
tick's 100 ms clock advance nor any settling bound or story wait changes.

## Causes

The stop reference is 1% of the request's 52.9 binary CPU s, about 0.529 s.
Estimates use its 31.2 module CPU s as an upper bound for broadly shared
paths; they are not new CPU measurements or claims that every hint-only
test reaches an engine/store path. Inclusive rows below overlap.

| Cause | Where | Ir per test, on which test | Fixed in | Or what fixing it would take |
|---|---|---:|---|---|
| Reparse identical init limits for defaults and root, also for store joins in other stories | `box_profiles` through `box_limits`/`box_joins` | median `box_limits`: 134,169,194 to 67,274,150 | `forge_routes.rs`: retain the real parser's immutable, Copy profiles | One process still parses once; its roughly 67m Ir remains. Removing that first read would need a separately held constant projection checked against the shipped document, or work on configuration/stream parsing. The profile shows `Slice::answer` allocation/copy under JSON feeding; that generic mechanism is in skein-lib, outside this task's small product changes. |
| Parse identical init model separately for defaults, root and worker-bound fields | `b2_tree::model` | median getter: 32,302,792 to 6,610,150 | `b2_tree.rs`: one real parsed model, copied for subsequent calls in this story's process | The cache accessor including copies is 6,621,723 Ir after; copying its small model is about 11.6k Ir, far below the removed parses. A test using the getter once gains no parse saving. |
| Construct and discard a whole started world just to take a restart configuration | `restart_with_charter` | question: 39,577,677 to 9,568,400 over two calls | `forge_routes.rs`: extract the unchanged paired configuration builder and use its runtime configuration directly | The retained world's real store, cold restoration, sign-in/adoption and change walk still run. No rows are seeded or story walk skipped. The paired builder still constructs a bootstrap configuration too; eliminating that remaining half requires factoring its per-configuration body. |
| Recompute fixed piece framing, including the whole family's bounds validation, for each authority/pattern conversion | `codec::people::pieces::{shared_sum,permission_sum}` | question shared sum: 3,752,738 to 54,322; permission sum: 86,340 to 984 | `jig-protocol-store`: fixed 17-byte grant framing and 12-byte permission framing | The generated bounds, including the u32 piece length, are held by `codec::people::pieces::tests::fixed_piece_framing_matches_the_generated_bounds`. Encodings, allowances and refusals keep the same values. Shared sum's before share estimates up to 0.558 CPU s of the module; its repeated family validation is gone. |
| Loader relocations | dynamic loader | current floor: 8,570,281 to 8,799,951 | left | Needs binary/linker/build work; build settings are closed. A fixture cannot remove it. |
| Real row encoding/decoding and temporary generated values | jig store codecs and temper store routing | question after: encode 9,516,131, decode 10,654,467 | fixed framing overhead removed | A larger borrowed translation/writer interface would avoid cloning domain payloads into generated owned values. The real codecs and all malformed/bound refusals must remain; bypassing them with the typed mirror would weaken the world. |
| Write history, typed mirror and consuming store request require separate owning rows | world's `Store::tick`/`event` | question after `Write::clone`: 3,209,023; change: 3,292,190 | left | Removing the inflight copy needs a borrowed commit-encoding seam through service routing and the component, or a new durable-fork representation. The mirror and replayable history still need their own retained values; a simple move cannot replace both. |
| One-row map requests, exclusive cursors and key copies during restore | `pages::reader::walk_map` | question after: 12,359,983, including `load_map` 5,000,247 | left | Batch map reads while preserving encoded/decoded bounds, the lookahead cursor, first-row refusal and corruption/diagnostic order. That requires a separate paging algorithm change with its boundary tests, rather than omitting real pages. |
| Real first start, cold root and policy/admission snapshots | fixtures, root and jig core | question after: first start 9,650,477, root constructors 4,681,963; median first start 18,942,538 | discarded restart worlds removed | Remaining construction is the live world's real start/restore. Direct seeding needs a configuration-dependent independent writer checked against the real script. Snapshot ownership changes must preserve admission and journal lifetimes. |
| Story decisions and their durable walks | root/core step, producer resolution, question handling | change root step: 59,051,913 to 59,051,897; own `next_delegate`: about 55.09m both | kept | These are what the stories check. A general reduction needs a profile of the individual admission/snapshot operations; skipping their inputs, commits or assertions is not a fixture repair. |
| Remaining local copies and idle bookkeeping | piece splitting, restart config copy, reclaim | question after: piece split 1,613,137, config copy 1,579,880, reclaim 345,881 | below local stop rule | Even removing each entire boundary estimates only 0.284, 0.278 and 0.061 module CPU s respectively, each below 0.529. Borrowed piece views need a codec borrowing interface; config copies protect consumed startup inputs. |

The larger remaining boundaries are recorded with the changes they need.
The next isolated local copy/bookkeeping savings are below the stop rule.
Both caches help repeated calls inside one test; neither claims sharing
between nextest processes. Public fixture names, signatures, limits, rows
and bytes remain the same.

Before the shared product edit, `git fetch origin` and
`git log origin/limits-worlds/integration -- jig/crates/jig-protocol-store/src/codec/people/pieces.rs`
showed no competing fixed-framing change. The fetched integration was
`fcf5f044b1dfce231a813a27e8adc795ef0b1b1c`; the last file commit was
`a5d8dbe2a2f713b72befc5050c6a100121777552`. No second fix was taken
from that plan.

## Validation

Final runs passed: engine domain 895 focused tests (893 suite and two lib),
26 fuzzy tests, client 396, conformance 31 and system 42. The focused
`walking::walking_story_replays_exactly_with_store_and_page_latency`
retains seed 62 and compares trace and durable rows; fuzzy walks also
compare every frozen state during replay. The engine's ten existing
ignored stories remain unobserved.

The product and its direct readers passed another 4,731 focused tests,
including all 729 `jig-protocol-store` tests. All eight temper packages
directly reading that jig crate compiled with `--tests`. Changed-package
clippy passed with `--all-targets -- -D warnings`; both trees' old-form
checks passed with unchanged baselines.

The new framing test was checked by changing grant framing from 17 to 18:
it failed with `Some(18)` versus the generated `Some(17)` for the grant's
empty pattern and piece length. Restoring 17 passed. Its final test module
also passes clippy. Browser, process and live tiers were not run.
