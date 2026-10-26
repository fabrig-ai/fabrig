# Base checks for skein 08: the component's worlds

Checked 2026-10-19. Source: temper `b4cfb9999` (code of `27995fb75`) at
`/srv/data/git/runner/src/tmp/planning/temper-base`; skein `d9f3e39` (code of `d120820`) at
`/srv/data/git/runner/src/tmp/planning/skein-base`.
The moves copy origin/main at execution, not these planning checkouts.
No base code was changed. The temporary detached scratch was removed.

## Discovery and disposition

Read development.md 5 in full, README/code-organization/decisions/later,
the designer's forge-kit-answers-1, the two precedents, source drafts 03.4–03.5
and S07-A, their reviews, surveys/traces and base-skein-03/07, and the current
03/05/06/07 session files. Names here were checked against source at the base.
The supplied code graphs were queried first with the exact supplied projects.
They lack useful protocol-world coverage; the fake_config snippet is stale.
Source reads and rg establish the results below; empty graph results were not
treated as proof of absence. No project/index management calls were made.

- 08.1 moves **33 focused + 5 fuzzy**, including all 19 hook tests and Joint's
  signed-delivery story. Earlier drafts' 32 focused/37 total omit that story.
- 08.3 moves **5 focused**, not the policy's nine focused plus one fuzzy.
  Those ten names and their fixture ownership are explicit in the session.
- Joint is 463 lines and calls the engine world's fake_config. Copy its
  approximately 48-line value fixture into machine/src/fixtures.rs; do not
  invent an export in the fake library. This removes the engine-world
  dependency without importing its connector policy.
- Temper's retained src/protocol.rs uses Joint's component/env/above/below/
  fake_env/answers fields, busy/time/replace and joint::checks. Their visibility
  must become public at the new crate boundary; no behavioral rewrite.
  busy/checks need #[must_use], confirmed by the scratch clippy failure/fix.
- Setup's world/exchange use Pipe/Cut; setup fixtures and one-process's forge
  use fake_limits. skein 07.3 depends on 08.1 for these exports.
- **Additional dependency:** 08.3 needs skein 07.1's translate::commit for its
  two job tests. 08.4 also reuses its LIMITS. README's row does not yet list
  this; the file's Needs does. There is no cycle: 07.1 does not need 08.1.
- 08.1 retains the existing raw/document/HTTP fixture limits. Its Shared
  flattens according to 03.1; hook limits become plain values 4, 8192,
  1048576, 10 seconds, 128 instead of importing the service's small profile.
  Neutral marker lengths change body room from 1045 to 1042:
  2*512+3+15, following `kit`, not a new budget.
- Recorded case_bytes reads the sibling conformance/transcripts; the hook
  recording is conformance/deliveries/push/request. Both require skein 06.2.
- 08.4 uses 07.1's client LIMITS with its calls/rows/answer_bytes/op_bytes/
  lifetime mirrored into Shared, component/shared repositories=1. The
  constructor admission probe passes. Fake profile/page cap2 stays as in
  the moved fixture; late/reordered/lost delivery work remains in later.md.
- TLS uses skein-tls-world's server::Server and pki helpers, including VALID
  as the certificate clock; pki::EPOCH would be wrong. Counted allocation
  reuses skein-heap Counting/Meter, excluding owner/peer allocations.
  03.1 computes the declared worst case; 08.2 first independently measures
  the full component holders. No unmeasured margin is claimed here.

## Open design question S08-cost (new, decisions number not assigned)

forge-connection 4.1 counts a request when its first byte is sent; section 10
asks for equality with requests the fake served while streams are cut.
The fake's tally advances on complete parsed request heads. These differ
when a request is cut partway through its head. This is not just startup
traffic: the scratch probe subtracts User/ApiSettings and still finds
seed 26, cut chance 300/1000: **reported cost 2, served 1, starts 2, Timeout**.

Recommendation, pending the designer: preserve literal served=reported
for complete requests with random response cuts, and for arbitrary cuts
assert reported=served+unserved, with the latter counted independently at
the first-byte/complete-head boundaries. This preserves the stated first-byte
cost rule. The session keeps both cases visible and names the pending ruling;
it must not be treated as ready to land that rule before the designer answers.
Only the session and this finding were writable, so decisions.md is unchanged.

## Timing and inventory

The retained baseline from the review and base-skein-03 is 47 focused at
0.404 s, six fuzzy at 7.709 s. The four fuzzy_machine tests, unsuffixed then
seed_1/seed_2/seed_3, cost **7.635/7.544/7.709/7.481 s**. Each sweep recreates
the component and start reads, then resolves the repository for each response
cut. The inherited work is understood structurally, not attributed to a
profiled hot function. Decision 11 retains those sweeps and their budgets.
Every faster execution below is a **lighter-load rerun**, not a replacement
baseline or evidence that the inherited cost disappeared.

| Binary | Focused | Fuzzy | Destination |
|---|---:|---:|---|
| machine | 14 | 0 | 08.1 machine |
| hooks | 19 | 0 | 08.1 machine, including Joint story |
| fuzzy_machine | 0 | 4 | 08.1 machine |
| fuzzy_hooks | 0 | 1 | 08.1 machine |
| fake_protocol | 5 | 0 | 08.3 protocol |
| protocol | 9 | 0 | stays temper |
| fuzzy_protocol | 0 | 1 | stays temper |

The list commands supply names; timings are from each individual moved
binary's rerun below, or the full run for retained policy tests.

| Binary | Test | Rerun seconds | Disposition |
|---|---|---:|---|
| fake_protocol | a_failed_job_read_names_its_failed_step_and_reads_its_log | 0.007 | moves |
| fake_protocol | a_failed_jobs_log_end_comes_through_a_suffix_range | 0.007 | moves |
| fake_protocol | opening_with_nothing_to_merge_writes_no_pull_request | 0.007 | moves |
| fake_protocol | pull_for_answers_the_open_pull_when_the_lookup_names_a_closed_one | 0.010 | moves |
| fake_protocol | the_component_starts_and_lists_items_through_the_fakes_bytes | 0.009 | moves |
| hooks | a_body_past_its_cap_is_answered_413_and_hints_nothing | 0.003 | moves |
| hooks | a_connection_past_hooks_waits_until_one_closes | 0.004 | moves |
| hooks | a_domain_push_reaches_the_component_as_a_signed_delivery_and_a_hint | 0.006 | moves |
| hooks | a_failed_listener_is_told_to_the_root_as_lost | 0.003 | moves |
| hooks | a_get_to_a_forges_path_is_answered_405 | 0.003 | moves |
| hooks | a_held_hint_stops_the_next_body_until_it_is_offered | 0.006 | moves |
| hooks | a_hint_the_root_refused_is_held_and_offered_again | 0.004 | moves |
| hooks | a_malformed_head_is_answered_4xx_and_closed | 0.003 | moves |
| hooks | a_missing_signature_is_answered_403_without_reading_a_body | 0.003 | moves |
| hooks | a_path_without_a_forge_is_answered_404 | 0.004 | moves |
| hooks | a_path_without_a_secret_is_answered_404 | 0.003 | moves |
| hooks | a_recorded_forgejo_push_signed_by_the_world_is_hinted | 0.020 | moves |
| hooks | a_request_unfinished_at_its_deadline_is_closed_with_no_answer | 0.002 | moves |
| hooks | a_signed_push_is_answered_204_before_its_hint_goes_up | 0.004 | moves |
| hooks | a_verified_body_the_decoder_calls_malformed_is_answered_400 | 0.003 | moves |
| hooks | a_verified_event_of_no_row_is_answered_204_and_hints_nothing | 0.003 | moves |
| hooks | a_wrong_signature_is_answered_403_and_hints_nothing | 0.003 | moves |
| hooks | the_secret_appears_in_no_event_or_debug_text | 0.003 | moves |
| hooks | two_deliveries_on_one_connection_are_both_answered_and_hinted | 0.004 | moves |
| machine | a_body_past_body_bytes_is_carried_empty_without_its_key | 0.018 | moves |
| machine | a_call_silent_past_its_deadline_is_a_timeout_and_aborts | 0.017 | moves |
| machine | a_call_waiting_past_its_deadline_is_unavailable_at_no_cost | 0.005 | moves |
| machine | a_connection_marked_close_finishes_its_call_and_the_next_connects | 0.024 | moves |
| machine | a_path_dropped_on_404_is_resolved_again_by_id | 0.028 | moves |
| machine | a_redirect_drops_the_path_and_answers_unavailable | 0.029 | moves |
| machine | a_reset_after_a_written_request_is_timeout | 0.012 | moves |
| machine | a_well_framed_malformed_document_is_invalid_answer | 0.017 | moves |
| machine | a_write_body_is_read_before_its_scripted_response | 0.020 | moves |
| machine | an_idle_connection_closes_and_the_next_call_connects | 0.021 | moves |
| machine | an_unreachable_forge_is_checked_again_after_its_backoff | 0.006 | moves |
| machine | another_user_stops_the_start_once_naming_the_forge | 0.002 | moves |
| machine | settings_answered_404_stops_the_start_with_its_status | 0.005 | moves |
| machine | the_start_reaches_ready_from_the_recorded_answers | 0.005 | moves |
| protocol | a_busy_pass_lists_at_most_changed_items_over_the_page_plus_one | 0.036 | temper |
| protocol | a_change_produced_opened_checked_queued_and_landed | 0.069 | temper |
| protocol | a_created_issue_with_its_answer_lost_is_found_after_restart | 0.016 | temper |
| protocol | a_failed_ci_head_is_repaired_then_checked_again | 0.071 | temper |
| protocol | a_merge_with_its_answer_lost_is_found_after_restart_without_merging_twice | 0.026 | temper |
| protocol | an_external_move_of_a_held_branch_is_reported_as_drift | 0.043 | temper |
| protocol | honest_costs_equal_the_requests_the_fake_served | 0.022 | temper |
| protocol | idle_forge_call_cost_does_not_follow_old_issue_history | 0.046 | temper |
| protocol | the_api_token_reaches_no_record_request_or_fact | 0.014 | temper |
| fuzzy_hooks | a_delivery_split_at_every_point_is_answered_alike | 0.189 | moves |
| fuzzy_machine | a_response_cut_at_each_byte_ends_its_call_once | 1.746 | moves |
| fuzzy_machine | a_response_cut_at_each_byte_ends_its_call_once_seed_1 | 1.680 | moves |
| fuzzy_machine | a_response_cut_at_each_byte_ends_its_call_once_seed_2 | 1.712 | moves |
| fuzzy_machine | a_response_cut_at_each_byte_ends_its_call_once_seed_3 | 1.689 | moves |
| fuzzy_protocol | stories_through_cut_streams_answer_each_call_once | 0.322 | temper |

## Commands on the untouched base

The destination crates do not exist at the base. Machine-world commands
map to this source package and its four moved binaries; protocol focused
commands map to --test fake_protocol. Its new client-only fuzzy tests have
no base equivalent; the source package's fuzzy run checks source viability
and does not claim those tests exist. The final two-package clippy maps to
this single source package. The checks of future 37/7 and 8/3 totals belong
to their increments; no base count is presented as proof of new behavior.

### Base command 1

```text
$ cargo nextest run -p temper-engine-protocol-forge-world
directory: /srv/data/git/runner/src/tmp/planning/temper-base
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.11s
    Starting 47 tests across 5 binaries (3 binaries skipped via profile.default.default-filter)
     Summary [   0.117s] 47 tests run: 47 passed, 0 skipped
exit: 0
```

### Base command 2

```text
$ cargo nextest run -p temper-engine-protocol-forge-world --profile fuzzy
directory: /srv/data/git/runner/src/tmp/planning/temper-base
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.11s
    Starting 6 tests across 3 binaries (5 binaries skipped via profile.fuzzy.default-filter)
        PASS [   0.322s] (1/6) temper-engine-protocol-forge-world::fuzzy_protocol stories_through_cut_streams_answer_each_call_once
        PASS [   0.366s] (2/6) temper-engine-protocol-forge-world::fuzzy_hooks a_delivery_split_at_every_point_is_answered_alike
        PASS [   1.749s] (3/6) temper-engine-protocol-forge-world::fuzzy_machine a_response_cut_at_each_byte_ends_its_call_once_seed_1
        PASS [   1.753s] (4/6) temper-engine-protocol-forge-world::fuzzy_machine a_response_cut_at_each_byte_ends_its_call_once_seed_3
        PASS [   1.878s] (5/6) temper-engine-protocol-forge-world::fuzzy_machine a_response_cut_at_each_byte_ends_its_call_once
        PASS [   1.895s] (6/6) temper-engine-protocol-forge-world::fuzzy_machine a_response_cut_at_each_byte_ends_its_call_once_seed_2
     Summary [   1.895s] 6 tests run: 6 passed, 0 skipped
exit: 0
```

### Base command 3

```text
$ cargo nextest list -p temper-engine-protocol-forge-world
directory: /srv/data/git/runner/src/tmp/planning/temper-base
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.11s
exit: 0
```
Parsed output counts: fake_protocol=5, hooks=19, machine=14, protocol=9; total47. Full names are in the inventory above.

### Base command 4

```text
$ cargo nextest list -p temper-engine-protocol-forge-world --profile fuzzy
directory: /srv/data/git/runner/src/tmp/planning/temper-base
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.12s
temper-engine-protocol-forge-world::fuzzy_hooks a_delivery_split_at_every_point_is_answered_alike
temper-engine-protocol-forge-world::fuzzy_machine a_response_cut_at_each_byte_ends_its_call_once
temper-engine-protocol-forge-world::fuzzy_machine a_response_cut_at_each_byte_ends_its_call_once_seed_1
temper-engine-protocol-forge-world::fuzzy_machine a_response_cut_at_each_byte_ends_its_call_once_seed_2
temper-engine-protocol-forge-world::fuzzy_machine a_response_cut_at_each_byte_ends_its_call_once_seed_3
temper-engine-protocol-forge-world::fuzzy_protocol stories_through_cut_streams_answer_each_call_once
exit: 0
```

### Base command 5

```text
$ cargo nextest run -p temper-engine-protocol-forge-world --test machine
directory: /srv/data/git/runner/src/tmp/planning/temper-base
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.11s
    Starting 14 tests across 1 binary
        PASS [   0.005s] ( 1/14) temper-engine-protocol-forge-world::machine a_call_waiting_past_its_deadline_is_unavailable_at_no_cost
        PASS [   0.012s] ( 2/14) temper-engine-protocol-forge-world::machine a_reset_after_a_written_request_is_timeout
        PASS [   0.017s] ( 3/14) temper-engine-protocol-forge-world::machine a_call_silent_past_its_deadline_is_a_timeout_and_aborts
        PASS [   0.017s] ( 4/14) temper-engine-protocol-forge-world::machine a_well_framed_malformed_document_is_invalid_answer
        PASS [   0.018s] ( 5/14) temper-engine-protocol-forge-world::machine a_body_past_body_bytes_is_carried_empty_without_its_key
        PASS [   0.002s] ( 6/14) temper-engine-protocol-forge-world::machine another_user_stops_the_start_once_naming_the_forge
        PASS [   0.006s] ( 7/14) temper-engine-protocol-forge-world::machine an_unreachable_forge_is_checked_again_after_its_backoff
        PASS [   0.005s] ( 8/14) temper-engine-protocol-forge-world::machine settings_answered_404_stops_the_start_with_its_status
        PASS [   0.005s] ( 9/14) temper-engine-protocol-forge-world::machine the_start_reaches_ready_from_the_recorded_answers
        PASS [   0.024s] (10/14) temper-engine-protocol-forge-world::machine a_connection_marked_close_finishes_its_call_and_the_next_connects
        PASS [   0.020s] (11/14) temper-engine-protocol-forge-world::machine a_write_body_is_read_before_its_scripted_response
        PASS [   0.028s] (12/14) temper-engine-protocol-forge-world::machine a_path_dropped_on_404_is_resolved_again_by_id
        PASS [   0.029s] (13/14) temper-engine-protocol-forge-world::machine a_redirect_drops_the_path_and_answers_unavailable
        PASS [   0.021s] (14/14) temper-engine-protocol-forge-world::machine an_idle_connection_closes_and_the_next_call_connects
     Summary [   0.034s] 14 tests run: 14 passed, 0 skipped
exit: 0
```

### Base command 6

```text
$ cargo nextest run -p temper-engine-protocol-forge-world --test hooks
directory: /srv/data/git/runner/src/tmp/planning/temper-base
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.12s
    Starting 19 tests across 1 binary
        PASS [   0.003s] ( 1/19) temper-engine-protocol-forge-world::hooks a_malformed_head_is_answered_4xx_and_closed
        PASS [   0.003s] ( 2/19) temper-engine-protocol-forge-world::hooks a_body_past_its_cap_is_answered_413_and_hints_nothing
        PASS [   0.003s] ( 3/19) temper-engine-protocol-forge-world::hooks a_failed_listener_is_told_to_the_root_as_lost
        PASS [   0.003s] ( 4/19) temper-engine-protocol-forge-world::hooks a_get_to_a_forges_path_is_answered_405
        PASS [   0.004s] ( 5/19) temper-engine-protocol-forge-world::hooks a_hint_the_root_refused_is_held_and_offered_again
        PASS [   0.004s] ( 6/19) temper-engine-protocol-forge-world::hooks a_connection_past_hooks_waits_until_one_closes
        PASS [   0.003s] ( 7/19) temper-engine-protocol-forge-world::hooks a_missing_signature_is_answered_403_without_reading_a_body
        PASS [   0.006s] ( 8/19) temper-engine-protocol-forge-world::hooks a_held_hint_stops_the_next_body_until_it_is_offered
        PASS [   0.003s] ( 9/19) temper-engine-protocol-forge-world::hooks a_path_without_a_secret_is_answered_404
        PASS [   0.006s] (10/19) temper-engine-protocol-forge-world::hooks a_domain_push_reaches_the_component_as_a_signed_delivery_and_a_hint
        PASS [   0.004s] (11/19) temper-engine-protocol-forge-world::hooks a_path_without_a_forge_is_answered_404
        PASS [   0.002s] (12/19) temper-engine-protocol-forge-world::hooks a_request_unfinished_at_its_deadline_is_closed_with_no_answer
        PASS [   0.003s] (13/19) temper-engine-protocol-forge-world::hooks a_verified_body_the_decoder_calls_malformed_is_answered_400
        PASS [   0.003s] (14/19) temper-engine-protocol-forge-world::hooks a_verified_event_of_no_row_is_answered_204_and_hints_nothing
        PASS [   0.004s] (15/19) temper-engine-protocol-forge-world::hooks a_signed_push_is_answered_204_before_its_hint_goes_up
        PASS [   0.003s] (16/19) temper-engine-protocol-forge-world::hooks a_wrong_signature_is_answered_403_and_hints_nothing
        PASS [   0.003s] (17/19) temper-engine-protocol-forge-world::hooks the_secret_appears_in_no_event_or_debug_text
        PASS [   0.004s] (18/19) temper-engine-protocol-forge-world::hooks two_deliveries_on_one_connection_are_both_answered_and_hinted
        PASS [   0.020s] (19/19) temper-engine-protocol-forge-world::hooks a_recorded_forgejo_push_signed_by_the_world_is_hinted
     Summary [   0.024s] 19 tests run: 19 passed, 0 skipped
exit: 0
```

### Base command 7

```text
$ cargo nextest run -p temper-engine-protocol-forge-world --profile fuzzy --test fuzzy_machine
directory: /srv/data/git/runner/src/tmp/planning/temper-base
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.11s
    Starting 4 tests across 1 binary
        PASS [   1.680s] (1/4) temper-engine-protocol-forge-world::fuzzy_machine a_response_cut_at_each_byte_ends_its_call_once_seed_1
        PASS [   1.689s] (2/4) temper-engine-protocol-forge-world::fuzzy_machine a_response_cut_at_each_byte_ends_its_call_once_seed_3
        PASS [   1.712s] (3/4) temper-engine-protocol-forge-world::fuzzy_machine a_response_cut_at_each_byte_ends_its_call_once_seed_2
        PASS [   1.746s] (4/4) temper-engine-protocol-forge-world::fuzzy_machine a_response_cut_at_each_byte_ends_its_call_once
     Summary [   1.746s] 4 tests run: 4 passed, 0 skipped
exit: 0
```

### Base command 8

```text
$ cargo nextest run -p temper-engine-protocol-forge-world --profile fuzzy --test fuzzy_hooks
directory: /srv/data/git/runner/src/tmp/planning/temper-base
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.11s
    Starting 1 test across 1 binary
        PASS [   0.189s] (1/1) temper-engine-protocol-forge-world::fuzzy_hooks a_delivery_split_at_every_point_is_answered_alike
     Summary [   0.189s] 1 test run: 1 passed, 0 skipped
exit: 0
```

### Base command 9

```text
$ cargo nextest run -p temper-engine-protocol-forge-world --test fake_protocol
directory: /srv/data/git/runner/src/tmp/planning/temper-base
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.12s
    Starting 5 tests across 1 binary
        PASS [   0.007s] (1/5) temper-engine-protocol-forge-world::fake_protocol a_failed_job_read_names_its_failed_step_and_reads_its_log
        PASS [   0.007s] (2/5) temper-engine-protocol-forge-world::fake_protocol a_failed_jobs_log_end_comes_through_a_suffix_range
        PASS [   0.007s] (3/5) temper-engine-protocol-forge-world::fake_protocol opening_with_nothing_to_merge_writes_no_pull_request
        PASS [   0.009s] (4/5) temper-engine-protocol-forge-world::fake_protocol the_component_starts_and_lists_items_through_the_fakes_bytes
        PASS [   0.010s] (5/5) temper-engine-protocol-forge-world::fake_protocol pull_for_answers_the_open_pull_when_the_lookup_names_a_closed_one
     Summary [   0.010s] 5 tests run: 5 passed, 0 skipped
exit: 0
```

### Base command 10

```text
$ cargo clippy -p temper-engine-protocol-forge-world --all-targets -- -D warnings
directory: /srv/data/git/runner/src/tmp/planning/temper-base
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.15s
exit: 0
```

### Member and neutral-name checks, in their base forms

The final member check must find two explicit skein members; the source
form finds one. The final no-temper check must have no matches (rg exit1);
its base form deliberately finds the old names (96 matching lines, exit0).

```text
$ rg -n '^    "tests/engine/protocol-forge",$' Cargo.toml
directory: /srv/data/git/runner/src/tmp/planning/temper-base
13:    "tests/engine/protocol-forge",
exit: 0
```

```text
$ rg -n -i temper tests/engine/protocol-forge
directory: /srv/data/git/runner/src/tmp/planning/temper-base
tests/engine/protocol-forge/tests/fuzzy_machine.rs:2:use temper_engine_domain_forge_client::api::Error;
tests/engine/protocol-forge/tests/fuzzy_machine.rs:3:use temper_engine_protocol_forge_world::{
tests/engine/protocol-forge/tests/protocol.rs:2:use temper_engine_domain_forge as top;
tests/engine/protocol-forge/tests/protocol.rs:3:use temper_engine_domain_forge_change as change;
tests/engine/protocol-forge/tests/protocol.rs:4:use temper_engine_domain_forge_client as client;
tests/engine/protocol-forge/tests/protocol.rs:5:use temper_engine_domain_forge_issues as issues;
tests/engine/protocol-forge/tests/protocol.rs:6:use temper_engine_protocol_forge_world::{
tests/engine/protocol-forge/tests/protocol.rs:10:use temper_fake_forge_protocol::Route;
exit: 0
```

## Users, copies and derived values

The following command searched all defined functions/types/constants in the
moved files plus the changed/new helpers, using rg in all five repositories.
Counts are exact identifier occurrences, including definitions and unrelated
same-spelled symbols (new, call, World, etc.); they are not semantic caller
counts. Qualified import searches below distinguish actual consumers. The
retired temper.old tree is excluded. The extra mixed_calls search follows.

```text
$ python3 -c 'from pathlib import Path
from collections import Counter
import re, subprocess, json
root=Path("/srv/data/git/runner/src"); base=root/"tmp/planning/temper-base"; world=base/"tests/engine/protocol-forge"
files=[world/"src"/(n+".rs") for n in ["machine","hooks","peer","pipe","joint","fixtures"]]
files += [world/"tests"/(n+".rs") for n in ["machine","hooks","fuzzy_machine","fuzzy_hooks","fake_protocol"]]
names=set(re.findall(r"\b(?:fn|struct|enum|const)\s+(\w+)", "\n".join(p.read_text() for p in files)))
names.update(["fake_config","commit","worst_case","full_heap","tls_cuts","listener_faults","World","event","run_for","restart","finish","creations","made","accounting","calls_and_answers","secrets_absent","seed","lost_creation","lose_next_creation_answer"])
pattern=r"\b(?:"+"|".join(sorted(names))+r")\b"
repos=[base,root/"tmp/planning/skein-base",root/"jig",root/"smith",root/"hearth"]
for repo in repos:
 r=subprocess.run(["rg","-o","--no-filename","--glob","*.rs","--glob","!**/temper.old/**",pattern,str(repo)],capture_output=True,text=True)
 assert r.returncode in (0,1),r.stderr
 c=Counter(r.stdout.splitlines()); print(json.dumps({"repo":repo.name,"counts":{n:c[n] for n in sorted(names)}}))
print(json.dumps({"base_lines":{str(p.relative_to(base)):len(p.read_text().splitlines()) for p in files}}))
patterns=[r"temper[-_]engine[-_]protocol[-_]forge[-_]world|temper[-_]engine[-_]forge[-_]world",r"\bfake_config\b|\bfake_limits\b|pipe::|joint::|peer::",r"Intake::with_capacity\(65536\)|latency_max: Duration::from_millis\(50\)|max_response_items: 2"]
for pat in patterns:
 for repo in repos:
  r=subprocess.run(["rg","-n","--glob","*.rs","--glob","Cargo.toml","--glob","!**/temper.old/**",pat,str(repo)],capture_output=True,text=True)
  assert r.returncode in (0,1),r.stderr
  rows=r.stdout.splitlines(); counts=Counter(str(Path(x.split(":")[0]).relative_to(repo)) for x in rows)
  print(json.dumps({"pattern":pat,"repo":repo.name,"lines":len(rows),"files":dict(counts)}))
'
directory: /srv/data/git/runner/src/tmp/planning/temper-base
exit: 0
```

| Identifier | temper | skein | jig | smith | hearth |
|---|---:|---:|---:|---:|---:|
| Answer | 3169 | 148 | 2304 | 1063 | 446 |
| BUILD_STEP_LOG | 2 | 0 | 0 | 0 | 0 |
| BUILD_STEP_NAME | 3 | 0 | 0 | 0 | 0 |
| Connect | 43 | 243 | 12 | 57 | 3 |
| Cut | 91 | 118 | 312 | 118 | 39 |
| End | 303 | 362 | 940 | 976 | 111 |
| FORGE | 208 | 0 | 0 | 0 | 0 |
| HOOK_SECRET | 8 | 0 | 0 | 0 | 0 |
| Joint | 46 | 1 | 76 | 1 | 0 |
| Listener | 24 | 182 | 4 | 0 | 0 |
| Machine | 49 | 418 | 50 | 132 | 99 |
| PING | 11 | 0 | 0 | 0 | 0 |
| Peer | 79 | 137 | 109 | 145 | 0 |
| Pipe | 38 | 130 | 4 | 23 | 73 |
| REPO | 403 | 0 | 0 | 1 | 0 |
| Response | 32 | 155 | 14 | 226 | 0 |
| SECOND | 25 | 73 | 54 | 1 | 2 |
| TOKEN | 33 | 2 | 21 | 2 | 0 |
| WRITER | 7 | 32 | 0 | 0 | 0 |
| World | 405 | 655 | 1316 | 1376 | 11 |
| a_body_past_body_bytes_is_carried_empty_without_its_key | 1 | 0 | 0 | 0 | 0 |
| a_body_past_its_cap_is_answered_413_and_hints_nothing | 1 | 0 | 0 | 0 | 0 |
| a_call_silent_past_its_deadline_is_a_timeout_and_aborts | 1 | 0 | 0 | 0 | 0 |
| a_call_waiting_past_its_deadline_is_unavailable_at_no_cost | 1 | 0 | 0 | 0 | 0 |
| a_connection_marked_close_finishes_its_call_and_the_next_connects | 1 | 0 | 0 | 0 | 0 |
| a_connection_past_hooks_waits_until_one_closes | 1 | 0 | 0 | 0 | 0 |
| a_delivery_split_at_every_point_is_answered_alike | 1 | 0 | 0 | 0 | 0 |
| a_domain_push_reaches_the_component_as_a_signed_delivery_and_a_hint | 1 | 0 | 0 | 0 | 0 |
| a_failed_job_read_names_its_failed_step_and_reads_its_log | 1 | 0 | 0 | 0 | 0 |
| a_failed_jobs_log_end_comes_through_a_suffix_range | 1 | 0 | 0 | 0 | 0 |
| a_failed_listener_is_told_to_the_root_as_lost | 1 | 0 | 0 | 0 | 0 |
| a_get_to_a_forges_path_is_answered_405 | 1 | 0 | 0 | 0 | 0 |
| a_held_hint_stops_the_next_body_until_it_is_offered | 1 | 0 | 0 | 0 | 0 |
| a_hint_the_root_refused_is_held_and_offered_again | 1 | 0 | 0 | 0 | 0 |
| a_malformed_head_is_answered_4xx_and_closed | 1 | 0 | 0 | 0 | 0 |
| a_missing_signature_is_answered_403_without_reading_a_body | 1 | 0 | 0 | 0 | 0 |
| a_path_dropped_on_404_is_resolved_again_by_id | 1 | 0 | 0 | 0 | 0 |
| a_path_without_a_forge_is_answered_404 | 1 | 0 | 0 | 0 | 0 |
| a_path_without_a_secret_is_answered_404 | 1 | 0 | 0 | 0 | 0 |
| a_recorded_forgejo_push_signed_by_the_world_is_hinted | 1 | 0 | 0 | 0 | 0 |
| a_redirect_drops_the_path_and_answers_unavailable | 1 | 0 | 0 | 0 | 0 |
| a_request_unfinished_at_its_deadline_is_closed_with_no_answer | 1 | 0 | 0 | 0 | 0 |
| a_reset_after_a_written_request_is_timeout | 1 | 0 | 0 | 0 | 0 |
| a_response_cut_at_each_byte_ends_its_call_once | 1 | 0 | 0 | 0 | 0 |
| a_response_cut_at_each_byte_ends_its_call_once_seed_1 | 1 | 0 | 0 | 0 | 0 |
| a_response_cut_at_each_byte_ends_its_call_once_seed_2 | 1 | 0 | 0 | 0 | 0 |
| a_response_cut_at_each_byte_ends_its_call_once_seed_3 | 1 | 0 | 0 | 0 | 0 |
| a_signed_push_is_answered_204_before_its_hint_goes_up | 1 | 0 | 0 | 0 | 0 |
| a_verified_body_the_decoder_calls_malformed_is_answered_400 | 1 | 0 | 0 | 0 | 0 |
| a_verified_event_of_no_row_is_answered_204_and_hints_nothing | 1 | 0 | 0 | 0 | 0 |
| a_well_framed_malformed_document_is_invalid_answer | 1 | 0 | 0 | 0 | 0 |
| a_write_body_is_read_before_its_scripted_response | 1 | 0 | 0 | 0 | 0 |
| a_wrong_signature_is_answered_403_and_hints_nothing | 1 | 0 | 0 | 0 | 0 |
| aborts | 14 | 20 | 6 | 7 | 0 |
| accept | 51 | 314 | 97 | 10 | 3 |
| accept_hook | 2 | 0 | 0 | 0 | 0 |
| accounting | 7 | 12 | 54 | 36 | 0 |
| an_idle_connection_closes_and_the_next_call_connects | 1 | 0 | 0 | 0 | 0 |
| an_unreachable_forge_is_checked_again_after_its_backoff | 1 | 0 | 0 | 0 | 0 |
| another_user_stops_the_start_once_naming_the_forge | 1 | 0 | 0 | 0 | 0 |
| answered | 496 | 294 | 594 | 783 | 5 |
| branch | 2209 | 41 | 0 | 75 | 40 |
| busy | 90 | 62 | 311 | 79 | 8 |
| call | 1386 | 1591 | 2593 | 3482 | 591 |
| calls_and_answers | 0 | 0 | 0 | 0 | 0 |
| carried | 20 | 20 | 101 | 67 | 0 |
| case | 526 | 287 | 160 | 139 | 441 |
| case_bytes | 5 | 0 | 0 | 0 | 0 |
| checks | 406 | 178 | 197 | 687 | 22 |
| client_down | 3 | 13 | 0 | 0 | 0 |
| client_up | 5 | 5 | 0 | 0 | 0 |
| close | 407 | 1690 | 606 | 548 | 158 |
| commit | 2438 | 214 | 902 | 379 | 222 |
| connections_opened | 16 | 0 | 0 | 0 | 0 |
| connects | 9 | 60 | 0 | 1 | 0 |
| creations | 12 | 0 | 15 | 0 | 0 |
| cut_script | 2 | 0 | 0 | 0 | 0 |
| cutting | 29 | 7 | 5 | 1 | 1 |
| debug | 10 | 6 | 13 | 3 | 4 |
| direct | 42 | 2 | 69 | 14 | 5 |
| down | 464 | 1738 | 716 | 270 | 159 |
| end | 661 | 1604 | 1617 | 1770 | 433 |
| event | 1746 | 2002 | 3313 | 2130 | 346 |
| events | 545 | 2128 | 836 | 943 | 128 |
| failing | 55 | 28 | 3 | 25 | 8 |
| fake_config | 29 | 0 | 0 | 0 | 0 |
| fake_limits | 9 | 9 | 0 | 3 | 0 |
| finish | 1243 | 654 | 8271 | 1637 | 181 |
| fmt | 92 | 156 | 41 | 58 | 28 |
| full_heap | 0 | 0 | 0 | 0 | 0 |
| goal_view | 3 | 0 | 0 | 0 | 0 |
| has_work | 9 | 74 | 4 | 15 | 0 |
| hinted_after_answer | 3 | 0 | 0 | 0 | 0 |
| hints | 41 | 0 | 17 | 0 | 13 |
| hook_limits | 7 | 0 | 0 | 0 | 0 |
| hook_settings | 7 | 0 | 0 | 0 | 0 |
| hooked | 37 | 0 | 0 | 0 | 0 |
| hooked_protocol_config | 3 | 0 | 0 | 0 | 0 |
| io | 1310 | 3024 | 694 | 1419 | 297 |
| issue | 419 | 79 | 7 | 18 | 66 |
| limits | 10767 | 6650 | 61117 | 11092 | 2508 |
| listener_faults | 0 | 0 | 0 | 0 | 0 |
| lose | 3 | 2 | 21 | 2 | 0 |
| lose_next_creation_answer | 0 | 0 | 0 | 0 | 0 |
| lost_creation | 0 | 0 | 0 | 0 | 0 |
| made | 385 | 456 | 734 | 134 | 315 |
| new | 8769 | 6432 | 48114 | 10784 | 2462 |
| now | 1462 | 1765 | 3380 | 1826 | 206 |
| opening_with_nothing_to_merge_writes_no_pull_request | 1 | 0 | 0 | 0 | 0 |
| peer | 322 | 1030 | 763 | 1103 | 23 |
| protocol_config | 5 | 0 | 0 | 0 | 0 |
| protocol_shared | 3 | 0 | 0 | 0 | 0 |
| pull | 1106 | 23 | 1 | 2 | 1 |
| pull_for_answers_the_open_pull_when_the_lookup_names_a_closed_one | 1 | 0 | 0 | 0 | 0 |
| push | 2768 | 2059 | 7401 | 2295 | 1316 |
| push_hook | 2 | 0 | 0 | 0 | 0 |
| ready | 296 | 358 | 631 | 175 | 93 |
| refuse | 29 | 51 | 163 | 92 | 8 |
| replace | 168 | 142 | 209 | 253 | 73 |
| request | 2313 | 2132 | 4244 | 1782 | 835 |
| requests | 348 | 1108 | 1312 | 548 | 145 |
| respond | 13 | 61 | 83 | 0 | 0 |
| response | 301 | 1397 | 144 | 827 | 7 |
| restart | 165 | 82 | 597 | 5 | 26 |
| run_for | 215 | 0 | 0 | 0 | 0 |
| script_done | 8 | 0 | 0 | 0 | 0 |
| secrets_absent | 8 | 0 | 0 | 0 | 0 |
| seed | 619 | 1761 | 1487 | 1083 | 849 |
| send | 599 | 632 | 1220 | 153 | 15 |
| server_down | 3 | 20 | 0 | 0 | 0 |
| server_events | 2 | 9 | 0 | 0 | 0 |
| server_up | 3 | 0 | 0 | 0 | 0 |
| settings | 1152 | 585 | 637 | 1724 | 645 |
| settings_answered_404_stops_the_start_with_its_status | 1 | 0 | 0 | 0 | 0 |
| settle | 382 | 282 | 275 | 482 | 33 |
| shared | 276 | 186 | 177 | 627 | 143 |
| signed | 55 | 78 | 130 | 23 | 15 |
| status | 1124 | 473 | 572 | 405 | 23 |
| statuses | 341 | 4 | 6 | 0 | 0 |
| stopped | 114 | 151 | 175 | 119 | 0 |
| sweep_seed | 5 | 0 | 0 | 0 | 0 |
| tally | 169 | 37 | 46 | 86 | 32 |
| the_component_starts_and_lists_items_through_the_fakes_bytes | 1 | 0 | 0 | 0 | 0 |
| the_secret_appears_in_no_event_or_debug_text | 1 | 0 | 0 | 0 | 0 |
| the_start_reaches_ready_from_the_recorded_answers | 1 | 0 | 0 | 0 | 0 |
| tick | 96 | 102 | 141 | 28 | 6 |
| time | 253 | 417 | 354 | 836 | 33 |
| tls_cuts | 0 | 0 | 0 | 0 | 0 |
| token_absent | 7 | 0 | 0 | 0 | 0 |
| two_deliveries_on_one_connection_are_both_answered_and_hinted | 1 | 0 | 0 | 0 | 0 |
| up | 599 | 1774 | 955 | 420 | 195 |
| with | 751 | 1177 | 2445 | 1108 | 124 |
| world | 4176 | 4665 | 11452 | 5975 | 204 |
| worst_case | 620 | 686 | 1580 | 672 | 155 |
| write | 1210 | 773 | 494 | 835 | 190 |
| writer_settings | 3 | 0 | 0 | 0 | 0 |

```text
$ rg -n '\bmixed_calls\b' /srv/data/git/runner/src/tmp/planning/temper-base /srv/data/git/runner/src/tmp/planning/skein-base /srv/data/git/runner/src/jig /srv/data/git/runner/src/smith /srv/data/git/runner/src/hearth --glob '*.rs' --glob '!**/temper.old/**'
directory: /srv/data/git/runner/src/tmp/planning/temper-base
exit: 1
```

No occurrences of mixed_calls in any of the five base repositories.

### Qualified-import users and copied-fixture values

The script above also ran each of these literal rg patterns with *.rs and
Cargo.toml globs. Counts below are matching lines, not occurrences.

1. `temper[-_]engine[-_]protocol[-_]forge[-_]world|temper[-_]engine[-_]forge[-_]world`
2. `\bfake_config\b|\bfake_limits\b|pipe::|joint::|peer::`
3. `Intake::with_capacity\(65536\)|latency_max: Duration::from_millis\(50\)|max_response_items: 2`

| Pattern | temper | skein | jig | smith | hearth |
|---|---:|---:|---:|---:|---:|
| 1 | 78 | 0 | 0 | 0 | 0 |
| 2 | 56 | 27 | 51 | 15 | 0 |
| 3 | 12 | 0 | 0 | 0 | 0 |

Actual temper import users (pattern1), including definitions/manifests:

| Base file | Matching lines |
|---|---:|
| testing/temper-fake-forge-protocol/Cargo.toml | 1 |
| testing/temper-fake-forge-protocol/src/tests/fixtures.rs | 1 |
| tests/conformance/Cargo.toml | 1 |
| tests/conformance/src/lib.rs | 1 |
| tests/conformance/src/link.rs | 1 |
| tests/conformance/src/scenarios.rs | 4 |
| tests/conformance/src/stories.rs | 3 |
| tests/conformance/src/systems.rs | 3 |
| tests/engine/domain/Cargo.toml | 1 |
| tests/engine/domain/src/forge_stand.rs | 2 |
| tests/engine/domain/src/walking.rs | 2 |
| tests/engine/domain/tests/forge_routes.rs | 1 |
| tests/engine/domain/tests/referee.rs | 1 |
| tests/engine/forge/Cargo.toml | 1 |
| tests/engine/forge/tests/fuzzy_simulation.rs | 1 |
| tests/engine/forge/tests/hooks.rs | 1 |
| tests/engine/forge/tests/memory.rs | 1 |
| tests/engine/forge/tests/replay.rs | 1 |
| tests/engine/forge/tests/settle.rs | 2 |
| tests/engine/forge/tests/simulation.rs | 18 |
| tests/engine/protocol-forge/Cargo.toml | 2 |
| tests/engine/protocol-forge/src/fixtures.rs | 1 |
| tests/engine/protocol-forge/src/joint.rs | 1 |
| tests/engine/protocol-forge/src/protocol.rs | 1 |
| tests/engine/protocol-forge/tests/fake_protocol.rs | 3 |
| tests/engine/protocol-forge/tests/fuzzy_hooks.rs | 1 |
| tests/engine/protocol-forge/tests/fuzzy_machine.rs | 1 |
| tests/engine/protocol-forge/tests/fuzzy_protocol.rs | 1 |
| tests/engine/protocol-forge/tests/hooks.rs | 2 |
| tests/engine/protocol-forge/tests/machine.rs | 1 |
| tests/engine/protocol-forge/tests/protocol.rs | 3 |
| tests/forge-setup/reconcile/Cargo.toml | 1 |
| tests/forge-setup/reconcile/src/exchange.rs | 1 |
| tests/forge-setup/reconcile/src/fixtures.rs | 1 |
| tests/forge-setup/reconcile/src/world.rs | 1 |
| tests/one-process/Cargo.toml | 1 |
| tests/one-process/src/forge.rs | 1 |
| tests/system/Cargo.toml | 1 |
| tests/system/src/composed.rs | 5 |
| tests/system/src/composed/link.rs | 1 |
| tests/system/src/forge.rs | 1 |

Pattern2's temper fixture/pipe/joint/peer users:

| Base file | Matching lines |
|---|---:|
| crates/temper-service/src/tests/clients.rs | 8 |
| testing/temper-fake-forge-protocol/src/tests/fixtures.rs | 4 |
| tests/client/src/front.rs | 5 |
| tests/client/src/translate.rs | 1 |
| tests/client/src/translate/forge.rs | 1 |
| tests/conformance/src/systems.rs | 1 |
| tests/engine/domain/src/forge_stand.rs | 1 |
| tests/engine/domain/tests/forge_routes.rs | 5 |
| tests/engine/domain/tests/referee.rs | 2 |
| tests/engine/forge/src/lib.rs | 1 |
| tests/engine/forge/src/world.rs | 2 |
| tests/engine/protocol-forge/src/fixtures.rs | 1 |
| tests/engine/protocol-forge/src/hooks.rs | 1 |
| tests/engine/protocol-forge/src/joint.rs | 8 |
| tests/engine/protocol-forge/src/machine.rs | 1 |
| tests/engine/protocol-forge/src/protocol.rs | 3 |
| tests/engine/protocol-forge/tests/fake_protocol.rs | 1 |
| tests/engine/protocol-forge/tests/hooks.rs | 1 |
| tests/forge-setup/reconcile/src/exchange.rs | 1 |
| tests/forge-setup/reconcile/src/fixtures.rs | 1 |
| tests/forge-setup/reconcile/src/world.rs | 1 |
| tests/one-process/src/forge.rs | 1 |
| tests/system/src/forge.rs | 5 |
Pattern3's copied-value sites:

| Base file | Matching lines |
|---|---:|
| crates/temper-engine-protocol-forge/src/tests.rs | 1 |
| crates/temper-forge-setup/src/tests/fixtures.rs | 1 |
| crates/temper-service/src/tests/worker.rs | 2 |
| testing/temper-fake-forge-protocol/src/tests/fixtures.rs | 2 |
| tests/codecs/tests/forgejo_start.rs | 1 |
| tests/engine/forge/client/src/world.rs | 1 |
| tests/engine/forge/src/world.rs | 1 |
| tests/engine/protocol-forge/src/fixtures.rs | 1 |
| tests/engine/protocol-forge/src/pipe.rs | 1 |
| tests/forge-setup/reconcile/src/fixtures.rs | 1 |

No qualified world-crate users occur in skein, jig, smith or hearth.
Same-spelled pipe/joint/peer/fake_limits matches there are unrelated worlds.
Temper's policy retains protocol.rs and writer_settings/protocol_shared/
goal_view and its owner-valued settings/shared/token_absent. Other engine,
system and conformance users keep their engine-world LIMITS/fake_config;
temper07 removes only the copies that moved and repairs surviving imports.
The literal/copy matches do not authorize changing unrelated worlds.

### Line inventory at the base

Counted in the same search command (source bytes unchanged):

| Source file | Lines |
|---|---:|
| tests/engine/protocol-forge/src/machine.rs | 320 |
| tests/engine/protocol-forge/src/hooks.rs | 243 |
| tests/engine/protocol-forge/src/peer.rs | 35 |
| tests/engine/protocol-forge/src/pipe.rs | 130 |
| tests/engine/protocol-forge/src/joint.rs | 463 |
| tests/engine/protocol-forge/src/fixtures.rs | 252 |
| tests/engine/protocol-forge/tests/machine.rs | 262 |
| tests/engine/protocol-forge/tests/hooks.rs | 215 |
| tests/engine/protocol-forge/tests/fuzzy_machine.rs | 43 |
| tests/engine/protocol-forge/tests/fuzzy_hooks.rs | 19 |
| tests/engine/protocol-forge/tests/fake_protocol.rs | 113 |

08.1 is about 2,000 moved lines: machine/hooks/peer/pipe=728, Joint=463,
539 test lines, the used subset of 252 fixture lines, the approximately
48-line fake_config value fixture and module/manifest glue. 08.3 is the
113-line fake_protocol test file. Tests/support are distinguished from
shipped production; no newly written increment exceeds about 300 planned lines.

## Temporary signature sketches and probes

```text
$ git worktree add --detach /srv/data/git/runner/src/tmp/planning/scratch-skein-08 b4cfb999
directory: /srv/data/git/runner/src/tmp/planning/temper-base
Preparing worktree (detached HEAD b4cfb9999)
exit: 0
```

The scratch added declaration-only bodies (todo!()) for full_heap,
tls_cuts, listener_faults, World::{new,event,run_for,restart,finish,creations,
made,accounting,calls_and_answers,secrets_absent}, lost_creation,
lose_next_creation_answer and mixed_calls. Existing client Event/Duration/
Joint types checked the interfaces; this does not claim an implemented
world or measured memory bound. It also compiled the required Joint public
fields/methods and the two probe tests. Pure functions keep #[must_use].
A scratch-only dev-dependency supplied temper-engine-forge-client-world::LIMITS.

### Initial scratch clippy and correction

```text
$ cargo clippy -p temper-engine-protocol-forge-world --all-targets -- -D warnings
directory: /srv/data/git/runner/src/tmp/planning/scratch-skein-08
error: this method could have a `#[must_use]` attribute
   --> tests/engine/protocol-forge/src/joint.rs:139:12
139 |     pub fn busy(&self) -> bool {
139 ~     #[must_use]
error: this function could have a `#[must_use]` attribute
   --> tests/engine/protocol-forge/src/joint.rs:453:8
453 | pub fn checks(passes: u32) -> raw::Checks {
453 + #[must_use]
454 | pub fn checks(passes: u32) -> raw::Checks {
error: could not compile `temper-engine-protocol-forge-world` (lib) due to 2 previous errors
exit: 101
```

The failure identified the two home-forced must_use attributes on newly
public Joint::busy and joint::checks. Added those attributes; no lint exception.

```text
$ cargo clippy -p temper-engine-protocol-forge-world --all-targets -- -D warnings
directory: /srv/data/git/runner/src/tmp/planning/scratch-skein-08
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.28s
exit: 0
```

After adding the final mixed_calls declaration, compiled the same crate again:

```text
$ cargo clippy -p temper-engine-protocol-forge-world --all-targets -- -D warnings
directory: /srv/data/git/runner/src/tmp/planning/scratch-skein-08
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.87s
exit: 0
```

### Profile admission probe

The probe combined the existing client-world LIMITS with the machine fixture,
changing only component/shared repositories to client.repositories=1, and
asserted Component::new(writer_settings, limits, shared).is_ok(). It also
asserted the existing hook_settings/hook_limits/shared combination is admitted.
The fake retains its already exercised profile; no shipped application profile
is added. The future flattened Shared is owned by 03.1 and checked again at ready.

```text
$ cargo nextest run -p temper-engine-protocol-forge-world --test profile_probe
directory: /srv/data/git/runner/src/tmp/planning/scratch-skein-08
    Finished `test` profile [unoptimized + debuginfo] target(s) in 2.98s
    Starting 1 test across 1 binary
        PASS [   0.002s] (1/1) temper-engine-protocol-forge-world::profile_probe the_client_world_limits_admit_the_protocol_fixture
     Summary [   0.002s] 1 test run: 1 passed, 0 skipped
exit: 0
```

### Partial-request cost probe

Scratch-only test: for seeds0..64, Joint::cutting(seed,300,Faults with all
zero rates), run20 virtual seconds; call1, REPO, Read::Items sinceTime::ZERO,
page1, kindNone; run11 seconds. Subtract tally.routes[User]+[ApiSettings]
from tally.total and compare to answered(1).cost. Return on the first
discrepancy; fail if none. The first discrepancy is seed26, as printed.
No component/fake production behavior was changed for this probe.

```text
$ cargo nextest run -p temper-engine-protocol-forge-world --test cost_probe --success-output immediate
directory: /srv/data/git/runner/src/tmp/planning/scratch-skein-08
    Finished `test` profile [unoptimized + debuginfo] target(s) in 4.49s
    Starting 1 test across 1 binary
        PASS [   0.086s] (1/1) temper-engine-protocol-forge-world::cost_probe a_partial_request_can_cost_more_than_the_fake_served
    seed=26 cost=2 served=1 starts=2 result=Err(Timeout)
     Summary [   0.086s] 1 test run: 1 passed, 0 skipped
exit: 0
```

### Scratch removal

```text
$ git worktree remove --force /srv/data/git/runner/src/tmp/planning/scratch-skein-08
directory: /srv/data/git/runner/src/tmp/planning/temper-base
(no output)
exit: 0
```

## Final document checks

These are checks of the authored documents, not prescribed implementation
proofs. The document command was rerun after formatting; its latest output
is shown. The source base status check is empty; no base code was edited. Since the plan is still
untracked, diff --check alone cannot inspect its new files; the document
check verifies their structure, all 53 names and the command-block bound.

```text
$ python3 - <<'PY'
from pathlib import Path
import re
p=Path('docs/development/plans/forge-kit')
s=(p/'skein/08-component-worlds.md').read_text(); f=(p/'findings/base-skein-08.md').read_text()
assert len(s.splitlines())<=400
for part in re.split(r'^### 08\.[1-5] ',s,flags=re.M)[1:]:
 for label in ['Scope:', 'Needs:', 'Files:', 'Shape:', 'Test shape:', 'Proof:', "Don\x27t:", 'Size:', 'Removes:']:
  assert label in part,label
for block in re.findall(r'```text\n(.*?)\n```',f,re.S):
 if block.startswith('$ '): assert len(block.splitlines())<=30
names=[]
for match in re.finditer(r'^\| (machine|hooks|fuzzy_machine|fuzzy_hooks|fake_protocol|protocol|fuzzy_protocol) \| ([a-z_][a-z_0-9]+) \| [0-9.]+ \|',f,re.M):
 names.append(match.group(2)); assert match.group(2) in s,match.group(2)
assert len(names)==53,len(names)
print(f'session: {len(s.splitlines())} lines; findings: {len(f.splitlines())} lines; all 5 increments complete; all 53 base names present; command blocks <=30 lines')
print('scratch exists:',Path('/srv/data/git/runner/src/tmp/planning/scratch-skein-08').exists())
PY
directory: /srv/data/git/runner/src/tmp/planning/temper-forge-kit
session: 388 lines; findings: 854 lines; all 5 increments complete; all 53 base names present; command blocks <=30 lines
scratch exists: False
exit: 0
```

```text
$ git diff --check -- docs/development/plans/forge-kit/skein/08-component-worlds.md docs/development/plans/forge-kit/findings/base-skein-08.md
directory: /srv/data/git/runner/src/tmp/planning/temper-forge-kit
(no output)
exit: 0
```

```text
$ git status --short -- tests/engine/protocol-forge Cargo.toml Cargo.lock
directory: /srv/data/git/runner/src/tmp/planning/temper-base
(no output)
exit: 0
```


## Review revision: sessions 08 and 09 (2026-10-19)

Read raw-08.txt and the planner's skein-08.txt rulings, development.md 5,
README, code-organization, decisions, the original session and these findings,
and the designer's answers 1–4 against the relevant base design sections.
The skein base is now **064c55f**, temper **dd12b8e**; documentation changed,
source code is still the recorded base's. Codebase-memory-mcp is unavailable
in this revision session; targeted rg checks against both bases were used.
No index-management commands ran. No commit or landing was made.

This appendix supersedes the earlier pending S08-cost wording: question36
is answered by forge-kit-answers-4.md and forge-connection.md 10. The two
invariants hold in every run, partial-head cuts included; the original
seed26 probe is retained as its own calibrated subcase, since a new owner
may consume randoms differently. Nothing is pending the designer.

### Rulings applied

1. Split into 08-machine-world (five increments) and 09-protocol-world
   (four). All Sizes include tests, with their estimates broken down.
   Machine stories split further into API stories, Close/listener, TLS,
   and memory so each is at most about300 new lines. Recount below.
2. Move names Settings.word=b"kit", user_agent=b"kit/0.0.0", drops Forge.host,
   and uses Transport::Plaintext { name: None }.
3. Named plaintext_request_cuts_end_once covers request and response-head
   byte cuts; inherited response-body sweeps stay unchanged.
4. Literal phase/deadline/error/cost and Close tables; short table-driven
   references; fuzzy count assertions; memory named bound, floor0 and
   observed saturation [2,2,2,2,8,4].
5. TLS flight length comes from each trial's captured flight, cut in that
   same trial; entropy and varying ECDSA lengths are explicit, assertions
   use outcomes and counts rather than a ciphertext length.
6. New world tests for doubling backoff, another forge taking an idle
   connection, held-answer reoffer and no returned-answer reoffer.
7. Joint/Pipe observation getters have signatures and retained temper
   callers; additive interfaces break no existing calls at the next repin.
8. Protocol scenarios name skein_forge_machine_world::joint::REPO, {1,2}.
9. Both machine package Proofs start world; protocol joint Proof has one
   command line; moved sweeps explicitly at most7.8 s each; 5/0.010 s is
   the recorded fake_protocol rerun, not the unrecorded0.019 s.
10. token_absent stays temper's; fake_limits' provenance comment is
    neutralized along with hook_limits'.
11. Manifests name path dependency skein-tls-world (../../tls), workspace
    skein-heap, and protocol's fake domain/protocol (direct/Route), both
    world dependencies and client/component dependencies.
12. Each Done when sums its new focused/fuzzy tests separately and reports
    against decision18 with the other session. Planned new bounds2 s/10 s
    exceed whole-plan1.5 s/6 s by0.5 s/4 s even before moves. The inherited
    four fuzzy times sum30.369 s, max7.709 s (sums are not suite wall time).
    README's whole-suite measurement still decides the aim; no budget rises.

### Recount, including tests

```text
$ wc -l tests/engine/protocol-forge/src/{machine,hooks,peer,pipe,joint,fixtures}.rs tests/engine/protocol-forge/tests/{machine,hooks,fuzzy_machine,fuzzy_hooks,fake_protocol}.rs tests/engine/forge/client/src/world.rs tests/engine/forge/client/tests/memory.rs tests/engine/domain/tests/memory.rs
directory: /srv/data/git/runner/src/tmp/planning/temper-base
   320 tests/engine/protocol-forge/src/machine.rs
   243 tests/engine/protocol-forge/src/hooks.rs
    35 tests/engine/protocol-forge/src/peer.rs
   130 tests/engine/protocol-forge/src/pipe.rs
   463 tests/engine/protocol-forge/src/joint.rs
   252 tests/engine/protocol-forge/src/fixtures.rs
   262 tests/engine/protocol-forge/tests/machine.rs
   215 tests/engine/protocol-forge/tests/hooks.rs
    43 tests/engine/protocol-forge/tests/fuzzy_machine.rs
    19 tests/engine/protocol-forge/tests/fuzzy_hooks.rs
   113 tests/engine/protocol-forge/tests/fake_protocol.rs
   746 tests/engine/forge/client/src/world.rs
   360 tests/engine/forge/client/tests/memory.rs
   325 tests/engine/domain/tests/memory.rs
  3526 total
```

08.1 retains about2,000 moved lines from the earlier per-file inventory;
09.1 moves113. New lines are estimates, not claims of an implemented count:

| Increment | Moved | New, including tests | Estimate breakdown |
|---|---:|---:|---|
| 08.1 | 2000 | 120 | fixtures/paths60, manifests/modules40, visibility/comments20 |
| 08.2 | 0 | 300 | driver100, focused cases100, request/head sweep100 |
| 08.3 | 0 | 280 | observation90, tables/tests90, listener sweep100 |
| 08.4 | 0 | 300 | TLS peer140, sweep100, tests60 |
| 08.5 | 0 | 300 | metering120, filling110, test/fixture45, formula/glue25 |
| 09.1 | 113 | 40 | manifest/module30, imports10 |
| 09.2 | 0 | 300 | owner150, routing70, fixture/hook30, tests50 |
| 09.3 | 0 | 270 | support140, fixtures40, tests90 |
| 09.4 | 0 | 300 | stream observation110, referee100, tests90 |

The746-line client-world driver is a comparison, not a file to copy: the
new protocol world reuses Joint's463 lines and the existing translation.
Memory reuses those drivers and skein-heap's meter, not either360/325-line
memory suite. A landing still reports its measured count by file, per
code-organization6. Final counts: machine42 focused/8 fuzzy (33+9,5+3),
protocol10 focused/3 fuzzy (5+5,0+3); all existing names preserved.

### Names and downstream users

Targeted rg searches verified Settings/Transport, fixtures/shared/limits,
Joint::REPO={1,2}, client-world REPO={0,0}, machine fixtures::REPO={1,1},
Joint/Pipe methods, direct, Route, pki::{VALID,name,client,Server},
server::Server and Counting/Meter. Searches for the added test/helper names
found no base implementations; they are planned additions, not claimed
existing exports. Existing request_lines and deliveries matches elsewhere
are unrelated (component Harness and fake domain, respectively).

Retained tests/engine/protocol-forge/src/protocol.rs calls Joint::replace,
tick,busy,time,call,tally and its public component/queue fields. Setup's
reconcile/src/exchange.rs calls Pipe::new, client_down, server_down,
server_up, client_up. 09.2–09.4 preserve those signatures and RNG scheduling;
new observers are additive. The component bound's downstream aggregate is
crates/temper-service/src/system.rs's limits::worst_case call; 08.5 names it.

### Changed signatures: scratch clippy

```text
$ git worktree add --detach /srv/data/git/runner/src/tmp/planning/scratch-skein-08-revision 064c55f
directory: /srv/data/git/runner/src/tmp/planning/skein-base
Preparing worktree (detached HEAD 064c55f)
HEAD is now at 064c55f Count the protocol world's costs at its streams
```

Only declaration bodies (todo!()) for changed/new helpers were compiled in
scratch tests/tls/src/forge_plan_sketch.rs, exported from its lib.rs:
full_heap()->(u64,u64,[u64;6]); listener_faults(u64)->Vec<[u64;6]>;
tls_cuts and plaintext_request_cuts(u64)->Vec<(u64,u64,u64,u64)>;
Pipe/Joint request_counts()->(u64,u64), request_lines()->&[Vec<u8>];
Joint deliveries()->&[Vec<u8>], delivery_counts()->(u64,u64,bool);
World owner_counts()->(u64,u64). Getters have #[must_use]. Receivers are
elided in the sketch because the forge worlds do not yet exist in skein.
This checks the declaration types/lints, not their behavior or the future
crate wiring; the ready check compiles against the landed dependencies.
Unchanged World and loss-hook signatures reuse the earlier scratch evidence.

```text
$ cargo clippy -p skein-tls-world --all-targets -- -D warnings
directory: /srv/data/git/runner/src/tmp/planning/scratch-skein-08-revision
    Checking skein-tls-world v0.0.0 (/srv/data/git/runner/src/tmp/planning/scratch-skein-08-revision/tests/tls)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.99s
```

```text
$ git worktree remove --force /srv/data/git/runner/src/tmp/planning/scratch-skein-08-revision
directory: /srv/data/git/runner/src/tmp/planning/skein-base
(no output; exit0)
```

No suite was rerun: the unchanged base-form nextest, clippy, member and
neutral-name commands above supply both sessions' command evidence. The
separate destination member/name checks map to the same already recorded
source check. New world tests cannot run at this base; their counts/times
are execution requirements, not measured results. The memory measurement
first runs in08.5; no measured byte margin is invented.

### Document validation

A local read-only Python check found: 08-machine-world371 lines, increments
87/62/59/59/41 lines; 09-protocol-world308 lines, increments44/76/49/77.
All nine have every required field, Proof lines start with their layer and
command, and new Sizes are at most300 including tests. All53 names
from this finding's moved/retained inventory and all original reference
signatures remain in the replacement files. The scratch is absent. The
original08-component-worlds.md is deleted; README/decisions/code-organization
are left to the planner, as requested.
