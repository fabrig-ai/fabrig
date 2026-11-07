# forge-rate: figures

| Test | Before (Ir) | After (Ir) | Cause | What changed |
|---|---:|---:|---|---|
| `temper-engine-protocol-forge-world::suite::rate::iterating_changes_installation_requests_per_hour` | 8,256,333,753 | 7,943,865,123 | Readiness and deadline scans repeated between HTTP fragments although the domain had received no input | Cache the domain schedule between answers and domain steps; 3.8% fewer instructions |
| `temper-engine-protocol-forge-world::suite::rate::moving_main_installation_requests_per_hour` | 8,321,536,270 | 8,008,904,224 | The same unchanged-domain scans, including while independently moving main | The same schedule cache; 3.8% fewer instructions |
| `temper-engine-protocol-forge-world::suite::rate::pending_changes_installation_requests_per_hour` | 1,603,071,588 | 1,552,089,088 | The same scans during read-only pending CI | The same schedule cache; 3.2% fewer instructions |
| `temper-engine-protocol-forge-world::suite::github_land::direct_landing_finish_and_two_writes` | 224,238,073 | 218,804,684 | The same scans while opening and confirming the merge | The same schedule cache; 2.4% fewer instructions |
| `temper-engine-protocol-forge-world::suite::github_land::pending_ci_across_token_hour_completes_on_fresh_token` | 5,476,039,300 | 5,234,863,065 | The same scans across pending CI, credential renewal and landing | The same schedule cache; 4.4% fewer instructions |

## Measurement and cause

The base is `b1076994b8`. Profiles use unoptimized tests, one exact test
through nextest, on the same builder and build configuration, with
`NEXTEST_DOUBLE_SPAWN=0` and the target runner
`valgrind --tool=callgrind --callgrind-out-file=...`. Every run names
`-p temper-engine-protocol-forge-world --test suite -E 'test(=FULL_NAME)'`.
All ten profiles passed in the default profile. Counts are the largest
Callgrind file from each run, excluding the listing processes. Cryptographic
blinding can make small differences outside the changed code.

The rate windows already show whole cycles. Pending warms 120 seconds and
measures 40 seconds, scaled by ninety; its 5-second listings, 20-second
checks and 40-second slow reads fit. Each busy shape warms 300 seconds and
measures 600 seconds, scaled by six, covering staggered 300-second pushes,
120-second CI and, for moving main, its 600-second cycle. Windows, shapes,
per-kind baselines, hourly totals and phase allowances stay unchanged.
An additional before/after run of all three rate tests with `TEMPER_RATE=1`
produces identical endpoint histograms, sharing counts and procedure
timestamps. Pending totals remain 1890, 3060 and 4230 requests/hour;
iterating totals are 2508, 4296 and 6084; moving-main totals are 2526,
4386 and 6138, all at N=1, 2 and 3 respectively. The nominal baselines and
their allowances remain as written in the tests.

For iterating, `busy_run` costs 7,900,231,359 Ir before and 7,593,501,019
after across three 900-second shapes: approximately 2,926,012 versus
2,812,408 Ir per simulated second. These include the scheduler, domain,
protocol, fake and checks. The whole test costs 3,057,901 versus 2,942,172
Ir per simulated second including setup, final assertions and teardown.
The initial profile has 84,133 protocol iterations, about 31 per simulated
second; simulating one second is not one transport iteration.

Before, the joint's `tick` is 7,009,165,455 Ir (84.9% of the whole
iterating test). The domain's readiness and deadline queries are
235,865,053 and 129,554,951 Ir. After, the latter queries cost about
33.5 million and 19.6 million Ir: transport fragments leave their results
unchanged. The cache is local to `run_for`, refreshed after answers and
after every domain resume or timer fire. Advancing time still compares the
cached absolute deadline, and every transport iteration, reclamation and
invariant check stays. The scheduler's own task scans are not the large
cost: almost all its inclusive instructions enter `run_for`.

The secret-leak check costs 388,802,095 Ir before and 388,983,055 after,
about 144,000 Ir per simulated second. It still formats and examines every
event, request and answer. Counting and observing HTTP framing costs
330,373,452 Ir on both sides, about 122,361 per simulated second. These
checks are included in the joint and world costs, not additional totals.

The direct landing's shared `opened` setup costs 85,601,148 Ir (38.2%
of that test): adopt the repository, produce and enroll the branch, then
admit the opening. Its `prepared` prefix costs 55,014,052 Ir; the world's
initial App signing costs 45,029,427 Ir. These inclusive figures overlap.
The long story adds repeated HTTP reads, CI passage and both credentials'
renewal. Neither setup nor token lifecycle is bypassed.

No product code changes. The eleven focused scenarios and both ignored
GitHub scenarios remain separate and unchanged. The existing direct-landing
test detects a missing cache refresh: removing invalidation after answers
fails at `change step answered`; restoring it passes. The pending steady
rate test alone did not detect that mutation, so it is not claimed as the
cache-invalidation check. All three rate tests retain the traffic checks.

## Found and not fixed

- The shared product connection driver advances all admitted calls on
  transport events: `Component::drive_calls` is 2,310,404,711 inclusive Ir
  in iterating before, including exchange pumping and decoding. Its
  call-list snapshots and repeated routing are mixed with necessary work,
  not a measured removable total. Avoiding unrelated calls would require
  a ready-call mechanism with connection-to-call wake routing, admission
  ordering and timeout coverage in skein. This is shared by the real loop
  and the other worlds; it is not a small lookup substitution here.
- Component deadline discovery is 437,234,299 Ir in iterating, unchanged
  by this fix. It covers connections, calls, installation state and logs.
  Removing those scans would require a maintained component deadline
  index, with updates for every scheduling transition and its own focused
  tests. This is shared product work, outside this harness cache.
- HTTP/JSON processing remains the main protocol cost. The iterating
  profile charges 763,588,784 inclusive Ir to the JSON tokenizer's `up`
  and 585,014,221 to string decoding; those overlap. The same requests
  still cross fragmented streams and produce the same replies. Further
  work needs a profile of these skein parsers and their own focused tests;
  skipping bytes, endpoints or fragments would weaken these worlds.
- The framing observer's 330,373,452 Ir in each iterating profile and
  258,553,795 in each token-hour profile includes byte-by-byte head and
  body accounting. A chunk-based observer could reduce it, with focused
  tests for split lines, content lengths, bodies and resets preserving
  first-byte and cut-in-head counts. The check remains in place.
- The secret invariant's formatting and searches remain: 388,983,055 Ir
  after in iterating and 227,181,958 in the token-hour test. Only part is
  repeated formatting of the fixture token; the rest checks each value's
  Debug representation. A streaming checker would need tests of matches
  spanning formatter writes and of both textual and byte-valued secrets.
  No invariant is dropped to claim this cost as a saving.
- App signing remains necessary: 236,401,606 Ir after in iterating,
  247,671,321 in the token-hour story, and about 45 million in direct
  landing. The short story pays it during setup; the long one also pays
  renewal. Reusing an authenticated world or shortening the hour would
  change what these independent scenarios check.

These are retained findings, not claimed sub-0.2-CPU-second causes. In
particular, connection dispatch, deadline scans and decoding have enough
inclusive instructions to merit separate work. The request's CPU figures
are the scale for that judgment; instruction reductions are the measured
result here. An attempted CPU timing runner was unavailable, so no new CPU
second measurements are asserted.

## Tests that went

None.
