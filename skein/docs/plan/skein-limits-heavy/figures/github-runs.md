# github-runs: figures

A whole fake run pays 600,478,062 inclusive Ir in `write::check`,
421,858,593 in the fake's `answers::finish`, 245,264,993 in `Mask::body`,
203,058,404 in `Json::parse`, and 90,068,424 in RS256. `World::pass` costs
1,009,439,700 inclusive Ir, including the fake's padding work; these counts
overlap and must not be added. Total: **2,557,261,726 before,
1,670,813,970 after**, with **284 requests in the same order**.

The fixes are a standard-library text search after validating each stream
once, registering identical secret values once while retaining the first
kind, padding scratch writers in 8 KiB rather than 128-byte chunks, and
searching for known opaque values before copying a text body. Binary secret
searches and binary masking retain exact byte matching. Afterward the same
functions cost 241,639,290, 33,016,425, 106,315,035, 203,092,901 and
90,067,998 Ir respectively; `World::pass` falls to 620,610,602.

| Row | Test | Before (Ir) | After (Ir) | Cause | What changed |
|---|---|---:|---:|---|---|
| `github-conformance` `tests` | `tests::github_conformance_holds_every_fact_on_the_fake` | 2,557,261,726 | 1,670,813,970 | Secret scans, tiny padding chunks, and copies for absent opaque values | Fix those causes; keep the whole run and every assertion |
| `github-conformance` `tests` | `tests::github_conformance_names_a_token_expiring_within_30_days` | 90,195,918 | 69,910,384 | Setup pays the shared scan and padding costs | Same setup-only test; unchanged expiry and cleanup checks |
| `github-conformance` `tests` | `tests::github_conformance_queue_refusals_fail_one_fact` | 6,366,114,766 | 1,964,375,225 | Eleven fixtures and queue runs, plus the whole-run causes | Keep eleven refusal rows in six queue runs, with exact reasons and every cleanup check |
| `github-conformance` `tests` | `tests::github_conformance_masks_secrets_and_ids` | 2,769,528 | 2,589,454 | Body copied for each known opaque value even when absent | Search first; keep every masking category and embedded GraphQL node-id check |
| `github-conformance` `tests` | `tests::github_conformance_faults_fail_only_their_facts_and_the_run_continues` | unknown | 1,712,490,140 | Separate whole runs checked individual faults | New holder: one whole run, four faults, every later fact and cleanup held; no before profile |
| `github-conformance` `tests` | `tests::github_conformance_response_framing_records_the_exchange_and_continues` | unknown | 284,010,002 | Two whole runs checked a log exchange | Two log-group runs, six facts each; all outcome, transcript, masking, provenance and cleanup assertions stay; no before profile |
| `github-conformance` `tests` | `tests::github_conformance_epoch_refusals_and_non_json_fail_only_the_fact` | unknown | 406,786,398 | Three whole runs checked one epoch fact | Three epoch-only runs; all three status/body rows and cleanup assertions stay; no before profile |
| `github-conformance` `tests` | `tests::github_conformance_discovers_the_organization_before_minting` | unknown | 473,310,731 | Whole run checked organization discovery and queue rulesets | Queue alone still covers every merge-queue ruleset; discovery and before-minting checks stay; no before profile |
| `github-conformance` `tests` | `tests::github_conformance_missing_delivery_fails_only_its_fact` | unknown | unknown | Whole fixture for one absent input | Removed; holder below; no separate profile |
| `github-conformance` `tests` | `tests::github_conformance_null_merged_commit_fails_the_fact_and_continues` | unknown | unknown | Whole fixture for one malformed merged-pull answer | Removed; holder below; no separate profile |

Measured on 2026-11-07 on `debian-32gb-hel1-11`, in the same unoptimized
build, one exact library test per nextest invocation through `heavy`.
`NEXTEST_DOUBLE_SPAWN=0` and the x86-64 runner were set as in the common
note, using Callgrind and the test process's `summary` Ir. Listing processes
were excluded; none of these profiles starts child processes. The default
profile sufficed. Whole-run Ir falls 34.7%; the refusal test falls 69.1%.

Intermediate whole-run totals, fixing causes in profile order:

- Text search: 2,320,495,646 Ir.
- Batched padding: 1,931,660,905 Ir.
- Unique secret registration: 1,809,897,309 Ir.
- Avoid absent-value copies: 1,670,802,516 Ir, before the test restructuring.

Before grouping, the refusal test still cost 4,292,311,878 Ir after those
fixes, more than any remaining individual test. Six fixtures remove another
2,327,936,653 Ir. Each run carries at most one refusal per fact. The pending
approval refusal stays apart from merged-pull refusals because those facts
share a pull. The six dequeue rows require six runs, so there is no further
fixture to combine under that rule.

## Found and not fixed

- RS256: 90,067,998 Ir per final whole run; 22,519,347 in the setup-only
  test. The fake already caches a verified JWT and checks its claims on
  each use. Signing, renewal, wrong credentials and expiry remain real;
  changing ring or build settings is outside this repair.
- Required JSON collection and parsing: 203,092,901 Ir per whole run.
  `skein-json` product code stays closed. The next small avoidable cause is
  repeated parsing of the same response: the four redundant parses in
  `merge-repeat-idempotent` cost about 803,000 Ir per whole run. Even pricing
  all ten original runs at 325 ms gives about 0.001 CPU s. The repeated
  merged-queue parse costs 964,311 Ir across the entire final refusal test;
  at the request's 854 ms and its original 6,366,114,766 Ir that is about
  0.00013 CPU s. These are below 0.05 CPU s, so they stay.
- The remaining simulator passes drive requests, domain calls, delivery
  and CI clocks. No idle spin was found. `World::wait` is 22,710,261 Ir per
  whole run; every clock observation stays. Replacing this work needs a
  broader simulator or protocol change, rather than another padding fix.
- Transcripts and leak scans stay: tests read the files, and newly
  registered secrets must still be checked against retained bytes. Removing
  exchange, flush or output scans would weaken that rule. Remaining mask
  work checks the same categories and preserves first-appearance numbering.

## Tests that went

- `tests::github_conformance_missing_delivery_fails_only_its_fact`: held by
  `tests::github_conformance_faults_fail_only_their_facts_and_the_run_continues`,
  which passed at `726e06247ae00d3076bd20a1a6d2f126d76d1813`.
- `tests::github_conformance_null_merged_commit_fails_the_fact_and_continues`:
  held by the same test, which passed at
  `726e06247ae00d3076bd20a1a6d2f126d76d1813`.

The holder checks the exact four failure reasons, every override consumed,
all 57 facts recorded, the signature fact unchecked, every other fact
holding, the two named later merge facts present and holding, the failed
merge line in `facts.txt`, cleanup, empty journal/tokens/mints, output leak
scans and quiescent shutdown. Returning the framing error from `observation`
instead of recording its fact made it fail at completion with `invalid
response length`. Removing the empty-listing override made it fail because
`log-delivery-whole` held instead of failing with `input absent: delivery id`.
Both mutations were restored.

## Counts

- `tests::github_conformance_holds_every_fact_on_the_fake`: 57 facts and
  284 requests before and after. Temporary diagnostics also recorded the
  same ordered phase/method/target digest, `b596a850de5fd15a`, through the
  guard-replay test. The diagnostics were removed. Fact names and the order
  of facts and requests in `cases::run` stay unchanged.
- `tests::github_conformance_response_framing_records_the_exchange_and_continues`:
  two whole runs recording 57 facts each become two log-group runs recording
  six each. Every log fact holds alone in these injected rows except the
  selected invalid-length observation; no log assertion needed omission.
  Removing `record_failed` made the smaller test fail its request-transcript
  prefix assertion. Restored.
- `tests::github_conformance_epoch_refusals_and_non_json_fail_only_the_fact`:
  three whole runs recording 57 facts each become three epoch-only runs
  recording one each. Ignoring the epoch HTTP status made the 503/empty-array
  row hold, and its exact failure-outcome assertion failed. Restored.
- `tests::github_conformance_discovers_the_organization_before_minting`:
  one whole run becomes one queue run. Every queue ruleset still comes from
  this group, and all initial discovery and queue-origin assertions remain.
  Routing the queue to installation zero made the smaller test fail with
  `person ruleset creation refused`. Restored.
- `tests::github_conformance_queue_refusals_fail_one_fact`: eleven rows and
  all five queue facts stay; queue runs fall from eleven to six. Changing
  the group-read refusal from `Forbidden` to `InvalidAnswer` failed the exact
  reason assertion naming `queue-dequeued-delivered/group/0`. Restored.
- Across the focused GitHub library: 29 tests become 28; ten whole
  `cases::run` invocations become three (whole-facts, guard replay, faults).
  Explicit `cases::queue` invocations become seven: six refusal groups and
  organization discovery. Including the queue group inside the three whole
  runs gives ten queue-group executions, versus 21 before.
