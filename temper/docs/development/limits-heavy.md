# Focused test costs: the repair's account

## The result

2026-11-07. Final measurements use `bdd65d61b3`, whose product code
matches merged integration `8822d7db70`. The earlier closing pass uses
`6755dae86e`: integration `30294e75e1`, including gallery
`436caabd1e`, combined with scale/restart `b925a75f81`.
The seven task figures are preserved in that earlier commit under
`temper/docs/plan/limits-heavy/figures/`. Each task's before/after pair
uses its own base, not one common baseline. Both closing passes use
16 CPUs, AMD EPYC-Genoa Processor, and unoptimized test builds.
The later integration merges newer main through `8822d7db70`, including
JSON string handling, bounded store-page reads and an additional UI phrase.
Trunk counts therefore include changes beyond this plan.

Each closing pass measures one exact named test per row through nextest
and Callgrind, with double spawn disabled. All 13 tests passed in each
pass: 26 successful profiles. Both snapshots remain in the table.
Discovery processes are excluded. The refused-port count sums four traced
processes; the launcher sums 74, excluding external readlink executables
on both sides of its comparison. The other counts are single processes;
fake-backend profiling also enabled child tracing but produced no children.
Gallery, retained words and the scale story use the existing fuzzy profile's
longer instrumentation allowance with an explicit selection; their tests
remain focused tests. No slow tier ran.

The selected test is the largest remaining measured candidate in its row's
figures. This is not a measured ranking of unprofiled tests. The listener
figures are thin: most individual counts are unknown, its refused-port
profile varies with RSA prime searches, and the fresh-box block uses a CPU
fallback because Callgrind cannot send its pidfd signal. The forge-landing
file also leaves most individual counts unknown. Only those rows' selected
candidates were measured again; their earlier work was not repeated.
Retained words is now the view row's largest, after small-line ordering
became cheaper.

Each CPU estimate is exactly the request's row CPU s × task-after Ir /
before Ir for its selected test. It assumes that test's ratio describes
the whole row; it is not a row CPU measurement. This assumption is
especially weak for the listener move, the gallery's consolidated rules,
and view limits with different remaining drawing costs.

| Request row / selected test | Request CPU s | Before Ir | Task after Ir | Early merged Ir | Merged trunk Ir | Estimated CPU s after |
|---|---:|---:|---:|---:|---:|---:|
| box_listeners: `temper::suite::box_listeners::a_later_box_refuses_a_taken_port_naming_the_port_and_document` | 17.6 | 975,595,748 | 5,413,110,171 | 4,924,326,388 | 3,146,142,721 | 97.654 |
| gallery: `temper-client-gallery-tool::lib::tests::print_writes_every_page_and_the_bundle` | 12.1 | 31,170,701,091 | 8,292,374,426 | 8,292,406,046 | 8,293,436,601 | 3.219 |
| fake_backends: `temper::lib::fake_backends::tests::retained_peer_edits_are_checked_without_replacement` | 11.5 | 19,892,711,440 | 2,481,957,851 | 2,287,284,908 | 1,637,101,265 | 1.435 |
| view limits: `temper-client-view::lib::tests::limits::kept_words_at_the_threads_bytes_build` | 9.1 | 12,148,031,421 | 11,991,007,201 | 11,910,812,477 | 11,910,975,872 | 8.982 |
| forge_land: `temper::suite::forge_land::the_launcher_gives_each_landing_story_its_own_store` | 4.0 | 79,217,172,205 | 211,172,655 | 200,883,688 | 203,251,483 | 0.011 |
| rate: `temper-engine-protocol-forge-world::suite::rate::moving_main_installation_requests_per_hour` | 3.5 | 8,321,536,270 | 8,008,904,224 | 8,008,707,539 | 7,237,418,084 | 3.369 |
| fabrig_scale: `temper-engine-domain-world::suite::fabrig_scale::a_goal_tree_takes_384_tasks` | 3.1 | 17,457,265,311 | 9,362,561,515 | 9,362,290,442 | 9,239,941,643 | 1.663 |
| restart: `temper-box::lib::tests::restart::early_reply_pipe_failure_and_busy_logs_allow_restarting` | 2.6 | 3,834,834,688 | 3,834,837,463 | 3,834,834,619 | 3,834,837,883 | 2.600 |
| github_land: `temper-engine-protocol-forge-world::suite::github_land::pending_ci_across_token_hour_completes_on_fresh_token` | 2.4 | 5,476,039,300 | 5,234,863,065 | 5,234,738,080 | 4,875,944,719 | 2.294 |
| turn: `temper-client-tools::lib::tests::jig::turn::a_document_sized_turn_prints_escaped_tool_names_with_the_shipped_limits` | 1.1 | 6,995,481,808 | 6,832,067,781 | 6,826,361,994 | 6,853,631,494 | 1.074 |
| document: `temper-client-tools::lib::tests::jig::reads::a_document_at_its_byte_bound_prints_its_maximal_json_escapes` | 0.6 | 5,180,449,245 | 3,363,194,720 | 3,362,385,613 | 1,587,522,677 | 0.390 |
| malformed setup: `temper-service::lib::setup::tests::lifecycle_tests::setup_malformed_queries_are_bad_requests` | 1.1 | 1,033,151,661 | 140,501,808 | 140,513,975 | 139,842,307 | 0.150 |
| expiry/paste race: `temper-service::lib::accounts::tests::expiry_and_native_paste_race_take_at_most_one_redirect` | 0.9 | 3,361,893,352 | 1,097,274,466 | 1,097,088,898 | 1,004,497,108 | 0.294 |
| **Total of listed rows and four single tests** | **69.7** | — | — | — | — | **123.133 (estimate)** |

The request's listed rows total **69.7 CPU s**, including the four single
tests, against its headline **about 89 CPU s**. The unexplained **19.3 CPU s**
is not assigned to an invented row or extrapolated. The literal estimate
is **123.133 CPU s**, 34.133 above that headline; it is dominated by the
listener ratio, not evidence that the repair made the suite slower.

The listener's 97.654 estimate applies random RSA variation to the entire
old 17.6 s, including two stories that moved. Removing their 15.9 s first
leaves 1.7 s; the same ratio then gives **9.432 s** for the remaining
listeners and **34.912 s** for all listed rows after the move, still an
estimate dominated by random key generation. The other rows contribute
**25.479 s** to either estimate. Neither extrapolation fills the missing
19.3 s or establishes a suite total. The gate's own CPU reading comes
after landing and is not this page's. CPU budgets and wall starvation
remain separate (`skein/docs/foundation/testing-strategy.md`, 8.1).

Three final trunk counts are above their task-after figures. The profiles
identify the intervening merge where possible; none is repaired here:

- Gallery print: 8,292,374,426 → 8,293,436,601 Ir, **+1,062,175**,
  about 0.013%. Main merge `8822d7db70` brings `ead6125140` through
  `31990916c8`: one additional box-capacity phrase. Input cloning rises
  1,913,349,338 → 1,936,794,027 Ir and capacity validation
  989,777,800 → 990,220,788; HTML rendering is
  4,179,957,236 → 4,179,894,898. These inclusive frames overlap and do
  not add to the net change. The gallery source itself is unchanged.
- Maximal turn: 6,832,067,781 → 6,853,631,494 Ir,
  **+21,563,713**, about 0.316%. The same main merge brings JSON changes
  through `586451b10a`, including `ae9260a7e2` and `bc0d8f4c4f`.
  Lossless JSON/duplicate-key validation rises
  5,214,465,400 → 5,368,214,692 Ir. The decoder's original sample was
  4,891,609,446; its final frame is 5,045,572,462. The new run path
  helps plain strings, but this test prints maximally escaped NULs.
  The profile locates the larger decoding work in the shared JSON
  changes, while writing also changes; it is not an isolated
  decode-only experiment. No assertion or byte bound changes.
- Faulted restart: 3,834,837,463 → 3,834,837,883 Ir, **+420**,
  about 0.000011%. The only later merge is the rebuilt tree above;
  no box, restart harness, I/O or fake-machine source changes.
  `probing_serve` is 797,602,253 Ir versus the figures' 797,602,028
  before sample, and all 720 orders pass. No source cause can be
  attributed for this tiny whole-process difference.

The earlier merged column also retains its increases: gallery
8,292,374,426 → 8,292,406,046 (**+31,620**) and malformed queries
140,501,808 → 140,513,975 (**+12,167**). At that snapshot the relevant
sources were unchanged; gallery input cloning and capacity matched their
task counts exactly, while service construction was 74,625,655 versus
74,625,022 and visits 45,525,924 versus 45,525,946. The final malformed
query count is lower, 139,842,307. The earlier counts are not substituted
for final ones.

Other main-merge changes matter too: maximal-document printing is now
1,587,522,677 Ir versus its task-after 3,363,194,720, and retained-peer
preparation is 1,637,101,265 versus 2,481,957,851. The merge includes
bulk JSON reading/writing and changed stock box configuration; these
whole-test comparisons do not isolate their individual contributions.
The scale story is 9,239,941,643 versus 9,362,561,515; the merge includes
`3d0fa8130d`'s restart-inventory reuse and bounded map-page loading.
These are later shared changes, not extra savings credited to this plan.

## What moved

`temper::suite::box_listeners::lifetime::a_hangup_stops_a_box_and_leaves_nothing_running`
and `a_killed_box_takes_its_steps_with_it` moved whole to
`temper::process::box_lifetime`, because running playground steps and
their deaths need real processes. They were compiled, not run, in the
new tier during this plan. The final merged process binary also compiled without executing a story.
Their assertions stayed. The original CPU
medians, **8.1 + 7.8 = 15.9 CPU s**, transfer to the process tier. Its
standing figures are 360 CPU s for the suite, 60 per test and a 1080 s
wall hard limit; its remaining margin and its runtime after this move
are unknown here.

The existing `a_story_box_and_its_steps_die_when_the_test_process_is_killed`
kills the owning test and watches its box and descendants without asking
for playground steps. The moved killed-box story signs in, starts the
playground, kills the box itself and observes every retained step exiting.
Both observe descendant death after owner death, but at different
ownership links and with different running descendants. Neither replaces
the other. These are moves, not removed tests.

Four gallery tests became rules in
`temper-client-gallery-tool::lib::tests::each_state_holds_what_its_name_says`,
which passed at `bbf0fb6cd3` before the separate processes went:

- `tests::navigation_links_have_addresses_and_install_action_links_keep_their_binding`.
- `tests::no_named_control_draws_a_text_child_in_any_state`.
- `tests::no_state_reads_of_zero`.
- `tests::each_chat_reads_its_allotment`.

Their original assertions and row names remain; the first three still
visit every state and the last its existing chat subset. Their helper
costs, already inside the holder, are 204,188,704; 5,264,990; 104,150,536;
and 1,096,040 Ir. No other task reports a test that went.

## Each row

These paragraphs account for each task's own changes. The result table
and merge findings above add the later trunk observations.

**Box listeners.** The move removes 15.9 of the request's 17.6 CPU s
from focused tests without claiming a faster process story. The six
remaining listeners' adjusted timing medians are 2.430 → 2.287 CPU s,
a different measurement from the request's residual 1.7 s. Their product
behavior is unchanged, and the difference includes key-generation noise.
Fresh-box block timing is 0.320 → 0.322 CPU s, with no instruction count;
its failed instrumentation is not treated as a valid sample.
The refused-port RSA generation alone was 704,660,593 → 5,142,084,149 Ir;
the early merged OpenSSL child is 4,689,274,084 and the final one
2,948,195,182. Init and box together were
253,648,112 → 253,716,953 in that task, versus 217,694,081 in the
early pass and 180,300,857 in the final pass.
The intervening startup reuse in `efde31e01a`, brought in by main merge
`cbdffcb951`, retains the checked machine rather than decoding it again.
The preparation task explicitly does not change ordinary box startup.
The whole refused-port totals, including the test parent, remain in
the result table, including their increase over baseline.

**Gallery.** Preparing words and derived bounds once per print, moving
single-page words rather than cloning them, and sharing one state pass
remove fixture repetition. The before print spent 14,606,021,112 Ir
loading English and 3,909,786,048 deriving bounds; the samples array was
only 3,667,028 across three constructions. Print falls
31,170,701,091 → 8,292,374,426 Ir. The state holder falls
12,608,794,546 → 5,534,470,586 while gaining the four cheap rules.
The four separate processes previously cost 12,675,806,998;
12,470,818,651; 12,573,446,064; and 1,966,208,085 Ir.
Print, those five semantic checks and the new replacement test together
fall **83,465,775,435 → 13,829,378,343 Ir (83.43%)**.
Only one complete print remains; whole-directory replacement has its own
2,533,331-Ir test and still runs in production print. The single-sample
model-name check is 54,346,700 → 54,341,265, effectively unchanged.
All pages, index links, stylesheets, bundle files and state checks stay.

**Fake backends.** The tests remain focused: they call preparation and
admission on scratch documents, without starting temper. One prepared
fixture replaces six preparations in each multi-case test, with every
edited or missing public document restored between cases. Optional
programs are explicit fixture facts; these tests no longer create an
unneeded provider key. Production still discovers its environment.
Retained-peer edits fall **19,892,711,440 → 2,481,957,851 Ir**, missing
documents **19,819,608,616 → 1,525,518,072**, and fresh preparation
**3,123,457,580 → 879,454,153**. Keeping admitted source bundle limits
avoids another encode/check: the fresh parent falls
953,772,289 → 880,194,066 before the final environment adjustment.
No refusal, reservation or retained-edit check goes; 47 original module
tests remain, with two new regressions. Their after counts are
172,875,213 and 497,843,702 Ir. The invalid-port floor slightly rises
6,219,759 → 6,221,247; it has no preparation fixture to remove.

**View limits.** Unique page positions use a bounded ordered map and
move whole decoded lines. Small lines fall
23,174,371,947 → 493,358,090 Ir, with ordering
22,711,430,720 → 30,407,393. Calls fall
6,084,229,642 → 963,736,342, with ordering
5,130,307,131 → 9,763,891. Markdown falls
6,288,570,002 → 5,592,630,919. Bulk byte-run construction removes
another approximately 212 million instructions across the six tests;
the inputs and both shells stay. Retained words fall only
12,148,031,421 → 11,991,007,201, long words
9,458,177,324 → 9,415,975,227, and request/composer
1,479,366,130 → 1,461,058,898: their drawing is still the check's cost.
The six total **58,632,746,466 → 29,917,766,677 Ir (49.0%)**.
The repeated-position path preserves its pairwise tie order and has a
separate regression; no bound or pattern shrinks.

**Forge landing launcher.** Process snapshots formerly started a shell
and external readlink per process. One Python working-directory scan per
snapshot replaces them; kept-store processes remain excluded from cleanup.
The comparable child trace falls **79,217,172,205 → 211,172,655 Ir**,
excluding external readlink on both sides. Its shell profiles fall
3,853 → 9 and 79,148,169,212 → 17,302,726 Ir. Process population affects
these totals. The task's 44-test adjusted CPU median falls
13.587 → 2.227 s, not the request's 4.0 s on another run.
Distinct README behavior stays separate: its startup-dominated count
slightly rises 12,156,740 → 12,159,810 Ir. No forge-landing test goes.

**Rate.** Caching readiness/deadline answers between unchanged-domain
transport fragments reduces iterating
8,256,333,753 → 7,943,865,123 Ir, moving main
8,321,536,270 → 8,008,904,224, and pending CI
1,603,071,588 → 1,552,089,088. Readiness/deadline queries in iterating
fall 235,865,053 / 129,554,951 → about 33.5 / 19.6 million.
All fragments, requests, invariants and the same 120/40-second pending
and 300/600-second busy windows stay. Observed hourly totals are unchanged:
pending 1890/3060/4230, iterating 2508/4296/6084 and moving main
2526/4386/6138 for N=1/2/3. The gain is about 3.8% in the busy tests,
not a shorter simulated hour.

**Scale.** Updating a tracked task in place and moving its neighbors
when appending removes repeated deep copies of a projection's whole
plan. All 384 tasks, capacity refusal and restart/read oracle stay.
The story falls **17,457,265,311 → 9,362,561,515 Ir (46.37%)**;
projection saving alone falls **8,299,019,885 → 211,326,253 (97.45%)**.
At 192 tasks the removed comparison measured
6,183,765,618 → 4,136,144,039 Ir. Before, doubling the tree multiplied
saving by 3.91; its final 192/384 counts are 78,842,002 / 211,326,253.
The whole story remains superlinear because whole-tree feeds grow too.
Eighty chats stay eighty: 2,922,851,330 → 2,923,111,661 Ir, including
the eighty-first refusal and restart oracle; that small increase remains
in the figures. The new allocation-retention regression costs 4,013,388 Ir.

**Restart.** All eight tests remain, including four permutation tests
with all 720 ending orders each. The basic profiled permutation test is
2,465,836,366 → 2,465,838,814 Ir; early reply-pipe failure with busy logs
is **3,834,834,688 → 3,834,837,463**. Per complete order, including setup
and teardown, these are 3,424,773 → 3,424,776 and
5,326,159 → 5,326,163 Ir. These slight increases are not hidden.
The profile found no small clonable startup snapshot; sharing it would
require support across the harness, box, I/O, simulator and fake machine.
Construction alone is only 47,079,679 Ir, 1.91% of the basic test.

**GitHub landing.** The same unchanged-domain cache reduces direct
landing **224,238,073 → 218,804,684 Ir** and pending CI across the token
hour **5,476,039,300 → 5,234,863,065**. The direct story's opening
setup is 85,601,148 Ir, including a 55,014,052 prepared prefix and
overlapping 45,029,427 App signing. The hour-long story still crosses
both token lifetimes and confirms one landing; it is not shortened.

**The four single tests.** Bulk allocation retains every NUL and its
six-byte JSON escape in the turn, reducing
6,995,481,808 → 6,832,067,781 Ir (2.3%). Filling exact hex byte pairs
and constructing the same NUL input in bulk reduces the maximal document
5,180,449,245 → 3,363,194,720 (35.1%); its shipped byte bound stays.
One service replaces sixteen constructions for malformed queries,
1,033,151,661 → 140,501,808 (86.4%), keeping every 400, page and header
assertion. The sign-in race uses its needed durable store/values directly
and settles only until quiet, reducing
3,361,893,352 → 1,097,274,466 (67.4%). All 54 schedules, independent
issuers, redirect/exchange/terminal checks and the 512-turn backstop stay.

## Found and not fixed

These are the row tasks' records at their after commits, including work
later affected by the main merge described above. They are not a claim
that every frame still has the same count on final trunk. Every finding
in the seven figures files follows, largest recorded
inclusive instruction count first. This ordering includes baseline
costs, required work and overlapping frames; the counts are not additive
savings. The box RSA line is supplemented with its figures' counts, and
the gallery's final paragraph on small checks is retained as an entry.
There are 31 original findings and that additional gallery entry.
The stopping rule was about 0.2 CPU s for the next cause, priced against
the request; several remaining causes exceed it and are recorded here
without a repair claim. Every item's further work remains outside this
closing task, which changes documents only.

- **fake-backends; largest recorded count 14,827,327,225 Ir.** **Production provider-key generation.** The traced fresh baseline spends 2,169,860,694 Ir in OpenSSL, of which 2,150,272,957 is inclusive RSA generation. Its work is random: the six children in the peer-edit baseline total 13,935,005,798 Ir, and those in missing documents total 14,827,327,225. A real environment that needs a new provider key still generates its fresh 3072-bit key. These preparation tests hold no rule about that provider key; its generation has its focused test in `init::tests::init_makes_a_private_3072_bit_provider_key_before_writing_its_document`. The GitHub App key is separate, seeded, and its existing test remains.

- **view-limits; largest recorded count 8,181,360,880 Ir.** Drawing at the byte bounds is linear in the required output. Pending words
  draw 65,536 and 1,048,576 newline bytes, in 1,024 and 8 chunks respectively,
  through both shells: 2,230,288 body paragraphs, including the final empty
  paragraph of each chunk. Their conversation costs 8,181,360,880 Ir after;
  freeing their trees costs 1,945,104,756 Ir. Long words retain seven
  131,072-byte lines for each of the two patterns, through both shells:
  conversation costs 6,832,484,982 Ir and tree destruction 1,591,132,006 Ir.
  The request and composer retain 131,072 bytes each through both shells;
  `draw_words` stays at 1,390,929,414 Ir. Reducing these counts by drawing
  fewer paragraphs, bytes or shells would weaken the bound tests.

- **small-rows; largest recorded count 5,214,465,400 Ir.** The turn's lossless JSON/duplicate-key validation costs 5,214,465,400 Ir
  on both sides, about 0.820 CPU s on the request's scale. Its underlying
  string decoder costs 4,891,609,446 Ir before. This assertion remains;
  removing it would weaken the test. A further repair belongs in the shared
  JSON reader/string decoder, outside this task's JSON-writing scope, and
  needs the decoder's own focused coverage.

- **box-lifetime; largest recorded count 5,142,084,149 Ir.** RSA-3072 key creation is required for each fresh sign-in provider. The
  refused-port profiles above show its variable cost. Sharing private
  keys across real boxes, shortening the key or changing the algorithm
  would change the product's behaviour; none is done. The RSA generation samples were 704,660,593 Ir before and 5,142,084,149 Ir after; the early merged OpenSSL child used 4,689,274,084 Ir and the final one 2,948,195,182 Ir.

- **view-limits; largest recorded count 4,679,394,088 Ir.** Markdown's full three-pattern window still costs 4,679,394,088 Ir in
  `markdown::read` after. The calls' conversation costs 851,754,746 Ir,
  and the small lines' conversation 285,657,448 Ir. These are the retained
  drawing work, including the markup and call structures the tests price.

- **gallery-print; largest recorded count 4,179,957,236 Ir.** HTML rendering remains the print check's own cost: the largest inclusive
  `Page::render` entry is 4,179,957,236 Ir in the one full print (about 0.56 CPU s
  on that pricing). Every page still needs rendering. The profile establishes
  this cost, not a smaller equivalent renderer; further work would need a
  renderer/batching profile and changes in `skein/crates/skein-ui-web/src/page.rs`,
  beyond the gallery and the permitted bundler fix. No state or output assertion
  was removed to avoid it.

- **scale-restart; largest recorded count 2,463,099,017 Ir.** The unchanged restart/read oracle costs 2,463,099,017 Ir in the final
  tree story and 761,158,338 Ir in the eighty-chat story. The tree's
  production store point reads cost 1,465,387,008 Ir, largely task joins
  and decoding; those are inside the oracle total, not additional to it.
  All retained rows and every task/result answer are still compared.
  Avoiding these reads would remove coverage; optimizing joins/codecs
  would need a separate store profile and a change in `jig-protocol-store`.

- **forge-rate; largest recorded count 2,310,404,711 Ir.** The shared product connection driver advances all admitted calls on
  transport events: `Component::drive_calls` is 2,310,404,711 inclusive Ir
  in iterating before, including exchange pumping and decoding. Its
  call-list snapshots and repeated routing are mixed with necessary work,
  not a measured removable total. Avoiding unrelated calls would require
  a ready-call mechanism with connection-to-call wake routing, admission
  ordering and timeout coverage in skein. This is shared by the real loop
  and the other worlds; it is not a small lookup substitution here.

- **scale-restart; largest recorded count 1,984,606,025 Ir.** The eighty-chat profile has 934,790,604 Ir in store requests and
  1,984,606,025 Ir in its admission helper, with store/engine routing
  nested within it. Its close oracle is included above. These totals
  describe required work, not one removable scan; even the whole store
  request total is about 0.13 CPU s against this row's original 0.4 CPU s.

- **gallery-print; largest recorded count 1,934,587,884 Ir.** Owned phrasebook copying: `ViewInputs::clone` costs 1,913,349,338 Ir in print
  and 1,934,587,884 Ir in the state holder, almost entirely `Words`/`Phrasebook`
  cloning. Each independently constructed `View` owns its words. Avoiding those
  copies needs a change to phrasebook ownership or a view reset/reuse API that
  preserves each state's bindings, clock and sign-in configuration, beyond
  sharing the gallery's checked inputs. This is about 0.53 CPU s when priced
  against the request's original test figures, so it is explicitly left here.
  Files: `temper/crates/temper-client-gallery-tool/src/lib.rs`,
  `temper/crates/temper-client-view/src/root.rs`,
  `skein/crates/skein-ui/src/phrases.rs`.

- **scale-restart; largest recorded count 1,903,543,827 Ir.** Whole-tree projection feeds: `finish_decision` costs 1,903,543,827 Ir in
  the final 384-task profile, including copying the complete plan required
  by `jig/docs/design/domain/connectors.md`, 8. The extra `ProjectionFeed`
  clone is 755,438,766 Ir and a boxed feed's drop is 425,775,339 Ir. Moving
  the owned feed to the final connector could avoid that copy and drop,
  about 0.18 CPU s against the original profile and request figure: below
  the stop rule. Eliminating whole-tree handoffs would change the feed
  contract. The remaining entry scan and append moves are within the
  211,326,253 Ir save total, about 0.033 CPU s by the same calculation.

- **fake-backends; largest recorded count 1,520,897,042 Ir.** **Repeated protocol admission and JSON conversion.** In the final fresh test, protocol parsing is 511,114,381 Ir (58.1%); `setup::check` is 397,821,765, `Source::read` 303,629,763, `Json::decode` 189,530,715, and the main recursive JSON writer 139,690,933. These inclusive figures overlap. In retained-peer edits, parsing is 1,520,897,042 and source admission 1,063,751,758; in missing documents, parsing is 971,376,362 and source admission 1,062,774,350. Every changed retained/source document still needs admission. Some consumers parse the same bytes again: sharing a checked protocol document and typed deployment through setup, step assembly and peer validation would address that remaining repetition while preserving refusal order. This is remaining work above the common note's approximate stopping threshold, recorded here rather than claimed cheap.

- **small-rows; largest recorded count 1,503,745,836 Ir.** JSON string measurement costs 1,503,745,836 Ir in the document and
  390,061,086 Ir in the turn; writing escaped strings costs 550,389,976 and
  348,028,667 Ir respectively. These counts include validation and writing
  the required bytes, including the digest's independently encoded subject.
  Eliminating all measurement would have a 0.235 CPU s ceiling across the
  two tests, but would lose validation and exact sizing. Further work would
  need bulk string classification or an encoder path for already validated
  hex, preserving refusal and byte-for-byte output. The remaining hex
  construction outside `Encoder::put_string` is 1,105,283,943 Ir in the
  document, a 0.128 CPU s ceiling on the original scale. These are inclusive
  costs and are not additive savings.

- **gallery-print; largest recorded count 1,149,900,341 Ir.** Stateful world construction: `samples::domain` costs 1,046,177,354 Ir in print
  and 1,149,900,341 Ir in the holder (including navigation's existing inbox
  verification), about 0.31 CPU s together on that pricing. A world is stepped
  for its address and subsequent acts; this is not one identical root copied
  for every state. Sharing the mock-document/world construction would reorganize
  `temper/crates/temper-client-gallery-tool/src/worlds.rs` and its dispatch with `src/samples.rs`.
  Leave that work until after `look-round-13`, as the task specifies.

- **gallery-print; largest recorded count 989,777,800 Ir.** Repeated view validation: `Words::capacity` costs 989,777,800 Ir in each of
  print and the state holder, mostly through `View::with_sign_in`'s validation.
  It is about 0.27 CPU s on the same pricing. Avoiding it needs cached checked
  capacity or a constructor/validation change in client-view and its phrasebook,
  whose public book currently remains independently owned. The shared fixture
  floors are also being investigated by `limits-fixtures`. Files:
  `temper/crates/temper-client-view/src/limits.rs` and `src/words.rs`.

- **small-rows; largest recorded count 878,343,149 Ir.** The unchanged occurrence scan costs 871,883,555 Ir, about 0.137 CPU s.
  An overlapping search using `bytes::find_from` was measured at
  878,343,149 Ir and reverted: constructing a search for each frequent
  match saves nothing here. It still counts overlapping occurrences of the
  same bytes. A faster search would need to retain a searcher across matches;
  even removing this scan entirely falls below the stopping threshold.

- **scale-restart; largest recorded count 797,602,028 Ir.** Restart setup repeats `probing_serve` 720 times: 797,266,182 Ir in the
  basic before profile and 797,602,028 Ir with both faults. In the basic
  profile shutdown through `run_to_end` adds 700,679,316 Ir. The harness,
  box service, `skein-io::Io`, simulator and fake machine do not provide a
  clonable snapshot. Sharing this prefix would require snapshot support
  across those owners, including the I/O entities, rather than one small
  derive. Construction through `with_stubborn` alone is only 47,079,679
  Ir (1.91%); its turn loop drives startup's real work. The probe's trace
  walk is small. The 720 six-ending deliveries, fault activity and final
  quiescence/descriptor checks stay in each of the four permutation tests.

- **forge-rate; largest recorded count 763,588,784 Ir.** HTTP/JSON processing remains the main protocol cost. The iterating
  profile charges 763,588,784 inclusive Ir to the JSON tokenizer's `up`
  and 585,014,221 to string decoding; those overlap. The same requests
  still cross fragmented streams and produce the same replies. Further
  work needs a profile of these skein parsers and their own focused tests;
  skipping bytes, endpoints or fragments would weaken these worlds.

- **small-rows; largest recorded count 610,479,172 Ir.** The race's remaining settling costs 610,479,172 Ir across all 54
  schedules, a 0.163 CPU s ceiling on the original scale; it now stops at
  quiescence. Its independent issuer's RSA signing costs 315,239,382 Ir
  (0.084 CPU s). Each fresh durable store costs part of the combined
  342,533,129 Ir in `marked_store` (0.092 CPU s). Avoiding the required
  fresh states or issuer exchanges would weaken the schedules. A smaller
  general store fixture is shared-fixture work; retain that owner's changes
  if a merge meets them.

- **view-limits; largest recorded count 598,135,002 Ir.** Pending admission costs 598,135,002 Ir in `Thread::sending`, including
  430,964,823 Ir rebuilding retained rows in `prune_read`. This is repeated
  work, but even removing the entire sending path would save only
  `598135002 / 12148031421 * 2.5 = 0.123` CPU s using the request's scale.
  A further repair would keep admission's retained rows/byte accounting
  without rebuilding them per chunk; no admission behavior is bypassed here.

- **view-limits; largest recorded count 495,559,774 Ir.** Remaining input construction, repeated weights and phrasebook work are
  below the stopping threshold. Bulk `repeated` costs 2,615,292 Ir across
  its three tests. Across the six original baseline profiles,
  `DecodedActivityLine::weight` costs 9,790,223 Ir,
  `Words::english` 495,559,774 Ir and `Words::capacity` 269,864,610 Ir.
  Conservative gain ceilings for weight, English construction and capacity
  are respectively 0.002, 0.198 and 0.096 CPU s: use the request's
  2.7/2.5/1.9 CPU s for its named tests,
  then assign all remaining 2.0 CPU s to the largest relevant inclusive
  share among the other three. These are separate ceilings, not additive
  savings. Reusing word lines or phrasebooks, or avoiding repeated
  capacity/weight queries remain possible; none is the next cause worth
  about 0.2 CPU s. Shared fixture construction stays
  with its existing owner.

- **forge-rate; largest recorded count 437,234,299 Ir.** Component deadline discovery is 437,234,299 Ir in iterating, unchanged
  by this fix. It covers connections, calls, installation state and logs.
  Removing those scans would require a maintained component deadline
  index, with updates for every scheduling transition and its own focused
  tests. This is shared product work, outside this harness cache.

- **forge-rate; largest recorded count 388,983,055 Ir.** The secret invariant's formatting and searches remain: 388,983,055 Ir
  after in iterating and 227,181,958 in the token-hour test. Only part is
  repeated formatting of the fixture token; the rest checks each value's
  Debug representation. A streaming checker would need tests of matches
  spanning formatter writes and of both textual and byte-valued secrets.
  No invariant is dropped to claim this cost as a saving.

- **forge-rate; largest recorded count 330,373,452 Ir.** The framing observer's 330,373,452 Ir in each iterating profile and
  258,553,795 in each token-hour profile includes byte-by-byte head and
  body accounting. A chunk-based observer could reduce it, with focused
  tests for split lines, content lengths, bodies and resets preserving
  first-byte and cut-in-head counts. The check remains in place.

- **gallery-print; largest recorded count 314,700,270 Ir.** The remaining local checks are below the stop rule: the four row helpers total
314,700,270 Ir including navigation's domain verification; the one-print bundler
is 26,485,195 Ir; and constructing the entire samples array costs 1,220,765 Ir in
the single-sample floor. None is near 0.2 CPU s at the request's prices.

- **forge-rate; largest recorded count 247,671,321 Ir.** App signing remains necessary: 236,401,606 Ir after in iterating,
  247,671,321 in the token-hour story, and about 45 million in direct
  landing. The short story pays it during setup; the long one also pays
  renewal. Reusing an authenticated world or shortening the hour would
  change what these independent scenarios check.

- **box-lifetime; largest recorded count 191,518,590 Ir.** Setup configuration parsing: 165,005,873 Ir in the refused box, with
  191,518,590 Ir in setup checking. A person's box pays this too. Reusing
  one parsed document across startup validation and setup would require
  restructuring that boundary; the whole parsing share is about 0.29 CPU s
  when conservatively projected over the request's six-test 1.7 s, and
  eliminating all parsing is impossible. It remains recorded rather than
  hidden behind a fixture or build setting.

- **box-lifetime; largest recorded count 165,089,256 Ir.** The launcher still starts Python for each snapshot (165,089,256 Ir across
  four Python executions including admission in the profiled story), plus
  its existing notice follower and evidence tools. A native snapshot helper
  could remove interpreter startup but would introduce another shipped
  launcher component. These remaining costs are recorded; no budget is
  raised to absorb them.

- **small-rows; largest recorded count 74,625,022 Ir.** Setup's remaining shared construction costs 74,625,022 Ir, a 0.079 CPU s
  ceiling on the original scale. Further general fixture work belongs to
  the shared-fixture task. No remaining test-local cause reaches about
  0.2 CPU s; the larger reader and writer paths are recorded above.

- **box-lifetime; largest recorded count 11,653,064 Ir.** Shared executable relocation: 11,653,064 Ir, 95.9% of the median README
  test. Distinct behaviours remain separate tests. Changing build settings
  is forbidden; separating the binary is outside this task's fixed layout.

- **fake-backends; largest recorded count 8,358,880 Ir.** **Small initialization and reservation costs stay.** In the initial parent-only fresh profile, source `init::write` is 3,986,792 Ir, sandbox `write_with_program` 8,358,880, and peer `reserve` 5,076. File publication and held listeners are required by the checks. There is no profile evidence for changing their algorithm or replacing fresh worker secrets.

- **view-limits; largest recorded count 0 Ir.** Repeated-position pages retain the pairwise ordering path, to preserve
  its order among equal positions. It has zero instructions in these six
  profiles. A broader replacement would need to preserve that tie order;
  the focused regression records it. Unique positions use O(n log n)
  bounded-map work without copying payloads.

## What stays heavy and why

The task profiles below price required work; current whole-test counts
and intervening changes are in the result table. Faster mechanics remain
possible without removing the required bytes, orders or assertions.

- **Bounds are the assertions.** Pending words still draw 65,536 and
  1,048,576 newline bytes, including 2,230,288 body paragraphs across
  both shells. Conversation and freeing cost 8,181,360,880 and
  1,945,104,756 Ir. Long words retain seven 131,072-byte lines per
  pattern through both shells: conversation 6,832,484,982 and freeing
  1,591,132,006. Request/composer drawing is 1,390,929,414.
  Fewer bytes, paragraphs, patterns or shells would weaken the rule.
- **A complete print still renders every page.** Its 4,179,957,236-Ir
  rendering frame is about 0.56 CPU s at the request's price.
  Rendering once establishes the cost; it does not establish that no
  better equivalent renderer exists. Phrasebook copying and repeated
  validation are separate avoidable-work candidates above, not asserted
  necessary merely because each page must render.
- **Every ending and cold restart stays.** The faulted restart test's final
  3,834,837,883 Ir pays for all 720 lifecycles. The 384-task story's
  2,463,099,017-Ir restart/read oracle compares retained state and every
  task/result answer. Removing orders or reads would remove coverage;
  a cross-owner snapshot or store optimization is different work.
- **The protocol tests still carry real bytes and token lifetimes.**
  Moving main is now about 7.237 billion Ir and the token-hour landing
  4.876 billion. App signing is 236,401,606 Ir in iterating and
  247,671,321 in the token-hour story. The retained requests, fragmented
  framing, leak checks and credential renewal are their checks; scans
  and parsing can still be investigated without skipping that behavior.
- **The maximal turn still validates its JSON.** Lossless parsing and
  duplicate-key validation cost 5,214,465,400 Ir in the task profile,
  about 0.820 CPU s at the request's price, and 5,368,214,692 Ir on
  final trunk, about 0.844 CPU s at that same price. Dropping the assertion is not a saving.
  A better shared decoder could be one.
- **Fresh provider keys remain a product obligation.** Real boxes still
  generate their own 3072-bit keys. The refused-port whole trace's
  975,595,748 → 5,413,110,171 → 4,924,326,388 → 3,146,142,721 Ir shows how much random
  prime search can change one observation. Preparation tests that do
  not assert provider-key generation now omit it through explicit facts;
  the dedicated key-generation test remains. Sharing private keys or
  weakening them would change behavior.

No CPU budget, timeout or build setting changes, and no budget proposal
is made. The remaining work and the measured suite's cost go to the user
under `skein/docs/foundation/testing-strategy.md`, 8.1.
