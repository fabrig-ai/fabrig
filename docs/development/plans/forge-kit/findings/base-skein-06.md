# skein 06: base commands and findings

Base: temper `27995fb752f4df586eeb92443ff17a9986f9de61`, skein
`d120820ec3d71686e38555378202c1dbc43efac2`; 2026-10-19.
Only this findings file and `skein/06-conformance.md` are plan outputs.
Scratch: `/srv/data/git/runner/src/tmp/planning/scratch-skein-06`, a detached
worktree of temper's base, removed after the commands.

## Discovery and corrections to the inputs

The named graph projects were used. `search_graph` found no current throwaway
symbols and found a vanished `tools/forgejo-conformance/run.py`; `search_code`
likewise missed the Rust throwaway. `get_code_snippet` found `skein_shell::drive`,
but its source bounds lagged. All names below were checked in the base with
`rg`; direct source reads supplied the graph's missing coverage. No index was
created or queried for status.

- The corpus has **151 case directories plus `binary.txt`**, i.e. 152 entries
  directly under `transcripts`, **303 files** recursively, and 24 delivery
  directories (48 files). The input's "152 transcripts" is not 152 cases.
- The dialect tests are not serde-free: `src/forgejo.rs` and five `forgejo_*`
  test files use serde for shape inspection or mutation. The move must replace
  those helpers as well as the tool's 17 serde-bearing source files.
- `profile_limits()` and the delivery fuzzy test reach the downstream service's
  shipped Small profile. Move the resulting plain `Limits` values, preserving
  both that profile and the separate generic codec fixture; no service edge in
  skein. Values and their admission check are recorded below.
- `limits::check(&Limits)` is a free function, not `Limits::check`.
- Taking data byte for byte and landing neutral fixture words requires a
  recording: 06.2 copies and checks the original bytes, then the neutralized
  recorder replaces them before landing. 06.4 separately repairs masking and
  records again. Never edit the corpus by hand or replace words while loading it.
- Designer answers 30/31 arrived during final checks (skein `d9f3e39`, temper
  `b4cfb999`, recorded in decisions.md). The final draft keeps attach's base
  signature and reads login/id from GET /api/v1/user; the flags are confirmed.
  The earlier explicit-login argument sketch below is superseded, not approval.

## Base test runs

```
$ cargo nextest run -p temper-throwaway-forgejo
# directory: /srv/data/git/runner/src/tmp/planning/temper-base
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.10s
    Starting 13 tests across 2 binaries (1 test skipped)
     Summary [   0.010s] 13 tests run: 13 passed, 1 skipped
```

```
$ cargo nextest run -p forgejo-conformance
# directory: /srv/data/git/runner/src/tmp/planning/temper-base
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.10s
    Starting 25 tests across 2 binaries
        PASS [   0.109s] (25/25) forgejo-conformance tests::the_listener_holds_its_answer_until_the_probe_replies
     Summary [   0.114s] 25 tests run: 25 passed, 0 skipped
```

```
$ cargo nextest run -p temper-codecs-world
# directory: /srv/data/git/runner/src/tmp/planning/temper-base
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.13s
    Starting 60 tests across 12 binaries (4 binaries skipped via profile.default.default-filter)
     Summary [   0.169s] 60 tests run: 60 passed, 0 skipped
```

```
$ cargo nextest run -p temper-codecs-world --profile fuzzy
# directory: /srv/data/git/runner/src/tmp/planning/temper-base
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.16s
    Starting 6 tests across 4 binaries (12 binaries skipped via profile.fuzzy.default-filter)
        PASS [   2.372s] (4/6) temper-codecs-world::fuzzy_forgejo forgejo_delivery_decoders_answer_every_mutated_delivery_without_panicking
        PASS [   2.745s] (5/6) temper-codecs-world::fuzzy_forgejo forgejo_decoders_answer_every_mutated_transcript_without_panicking
        PASS [   3.243s] (6/6) temper-codecs-world::fuzzy_forgejo forgejo_server_routes_answer_every_mutated_request_without_panicking
     Summary [   3.244s] 6 tests run: 6 passed, 0 skipped
```

The focused conformance listener test costs 0.109 s: it binds a loopback
listener and deliberately withholds its answer until the probe finishes. It is
a focused world test, not a <20 ms unit test. Codec delivery cases traverse the
recorded events repeatedly; the moved fuzzy workloads keep their 32 response,
200 route and 16 delivery mutations per case and their original seeds.
`base-runs.md` measured those three at 3.079, 3.702 and 4.347 s; this run measured
2.372, 2.745 and 3.243 s. These are unoptimized builds, not new budgets.

## Moved-test inventory from nextest list

```
$ cargo nextest list -p temper-throwaway-forgejo -p forgejo-conformance -p temper-codecs-world --message-format json
# directory: /srv/data/git/runner/src/tmp/planning/temper-base
forgejo-conformance: 25 listed; 0 ignored
temper-codecs-world::client_wire: 1 listed; 0 ignored
temper-codecs-world::drift: 3 listed; 0 ignored
temper-codecs-world::forgejo_deliveries: 4 listed; 0 ignored
temper-codecs-world::forgejo_files: 6 listed; 0 ignored
temper-codecs-world::forgejo_items: 5 listed; 0 ignored
temper-codecs-world::forgejo_reviews: 6 listed; 0 ignored
temper-codecs-world::forgejo_setup: 22 listed; 0 ignored
temper-codecs-world::forgejo_start: 4 listed; 0 ignored
temper-codecs-world::forgejo_writes: 8 listed; 0 ignored
temper-codecs-world::worker_wire: 1 listed; 0 ignored
temper-throwaway-forgejo: 8 listed; 0 ignored
temper-throwaway-forgejo::throwaway: 6 listed; 1 ignored
exit 0
```

```
$ cargo nextest list -p temper-codecs-world --profile fuzzy --message-format json
# directory: /srv/data/git/runner/src/tmp/planning/temper-base
temper-codecs-world::fuzzy_client_wire: 1 listed; 0 ignored
temper-codecs-world::fuzzy_forgejo: 3 listed; 0 ignored
temper-codecs-world::fuzzy_store: 1 listed; 0 ignored
temper-codecs-world::fuzzy_worker_wire: 1 listed; 0 ignored
exit 0
```

Names below are the baseline contract; a move keeps every name.

### forgejo-conformance

- `tests::a_chunked_response_is_written_unframed_with_its_masked_length`
- `tests::a_fact_line_is_holds_fails_or_unchecked_with_its_reason`
- `tests::a_fact_read_from_a_case_compares_its_status_and_message`
- `tests::a_fresh_password_in_a_body_leaves_its_transcript_masked`
- `tests::a_listed_runner_is_found_by_its_uuid`
- `tests::a_made_tokens_value_and_tail_leave_its_transcript_masked`
- `tests::a_masked_delivery_keeps_a_content_length_that_matches_its_body`
- `tests::a_password_is_read_between_its_first_and_last_quote`
- `tests::a_provisioning_failure_keeps_at_most_two_hundred_body_bytes`
- `tests::a_raw_token_is_forty_lowercase_digits_and_a_line_feed`
- `tests::a_secret_in_a_case_stops_the_run_naming_the_case`
- `tests::a_sent_request_is_counted_by_its_request_line`
- `tests::a_signature_holds_only_for_the_body_it_signs`
- `tests::a_truncated_git_packet_keeps_its_length_attached_to_the_hex_run`
- `tests::git_packet_ids_share_the_plain_ids_first_appearance_order`
- `tests::masking_also_rewrites_the_listeners_port_and_delivery_ids`
- `tests::masking_rewrites_ports_ids_times_and_dates_by_first_appearance`
- `tests::masking_sorts_the_deprecated_units_set_in_every_team`
- `tests::targets_encode_every_byte_but_unreserved_ones`
- `tests::the_clone_workflow_runs_steps_only_and_names_the_jobs_token`
- `tests::the_component_block_is_replaced_and_other_lines_kept`
- `tests::the_listener_holds_its_answer_until_the_probe_replies`
- `tests::the_pull_pair_records_any_checking_probe_that_settles`
- `tests::the_roll_up_ignores_the_losers_of_the_pairs`
- `tests::the_runner_configuration_names_one_connection_by_its_token_file`

### temper-codecs-world::forgejo_deliveries

- `a_recorded_deletion_ref_names_its_branch`
- `every_subscribed_event_has_a_recorded_delivery_the_decoder_knows`
- `recorded_deliveries_decode_to_the_fields_their_hints_read`
- `the_encoders_bodies_decode_unchanged_with_the_recordings_shape`

### temper-codecs-world::forgejo_files

- `a_failing_actions_status_links_its_run_and_job`
- `file_answers_the_server_encodes_decode_unchanged_with_the_transcripts_shape`
- `file_requests_encode_as_the_tool_sent_them`
- `file_requests_route_on_the_servers_side_to_the_same_request`
- `file_transcript_mutations_are_malformed_naming_the_field`
- `file_transcripts_decode_to_their_documents_and_rows`

### temper-codecs-world::forgejo_items

- `item_answers_the_server_encodes_decode_unchanged_with_the_transcripts_shape`
- `item_requests_encode_as_the_tool_sent_them`
- `item_requests_route_on_the_servers_side_to_the_same_request`
- `item_transcript_mutations_are_malformed_naming_the_field`
- `item_transcripts_decode_to_their_documents_and_rows`

### temper-codecs-world::forgejo_reviews

- `open_pulls_rows_are_the_pull_transcripts_documents`
- `review_answers_the_server_encodes_decode_unchanged_with_the_transcripts_shape`
- `review_requests_encode_as_the_tool_sent_them`
- `review_requests_route_on_the_servers_side_to_the_same_request`
- `review_transcript_mutations_are_malformed_naming_the_field`
- `review_transcripts_decode_to_their_documents_and_rows`

### temper-codecs-world::forgejo_setup

- `a_deleted_tokens_refusal_is_read_with_its_masked_value`
- `a_git_token_without_read_user_reads_the_scope_refusal`
- `a_missing_full_name_is_refused_naming_its_field`
- `a_password_past_string_bytes_is_refused_naming_its_field`
- `a_scope_list_past_the_profiles_inner_is_refused_naming_scopes`
- `a_scope_list_that_is_text_is_refused_naming_scopes`
- `a_text_must_change_password_is_refused_naming_its_field`
- `a_token_for_a_person_who_must_change_password_reads_the_refusal`
- `an_admin_token_without_a_user_scope_reads_the_refusal`
- `an_admin_token_without_write_admin_reads_the_scope_refusal`
- `an_api_token_without_write_issue_reads_the_scope_refusal`
- `basic_auth_for_a_person_who_must_change_password_reads_the_refusal`
- `creating_a_user_as_a_writer_reads_the_admin_refusal`
- `creating_an_existing_user_reads_its_refusal`
- `deleting_a_missing_hook_reads_its_refusal`
- `deleting_a_missing_token_reads_the_refusal`
- `making_a_token_of_an_existing_name_reads_the_refusal`
- `patching_a_missing_hook_reads_its_refusal`
- `setup_answers_the_server_encodes_decode_unchanged_with_the_transcripts_shape`
- `setup_requests_encode_as_the_tool_sent_them`
- `setup_requests_route_on_the_servers_side_to_the_same_request`
- `setup_transcripts_decode_to_their_documents`

### temper-codecs-world::forgejo_start

- `start_answers_the_server_encodes_decode_unchanged_with_the_transcripts_shape`
- `start_requests_encode_as_the_tool_sent_them`
- `start_requests_route_on_the_servers_side_to_the_same_request`
- `start_transcripts_decode_to_their_documents`

### temper-codecs-world::forgejo_writes

- `error_answers_meet_the_client_with_the_transcripts_shape`
- `failure_transcripts_name_their_refusal_by_status_and_message`
- `write_answers_the_server_encodes_decode_unchanged_with_the_transcripts_shape`
- `write_bodies_enforce_fields_types_and_bounds`
- `write_requests_encode_as_the_tool_sent_them`
- `write_requests_route_on_the_servers_side_to_the_same_request`
- `write_transcript_mutations_are_malformed_naming_the_field`
- `write_transcripts_decode_to_their_documents_and_rows`

### temper-throwaway-forgejo

- `tests::a_basic_request_records_its_scheme_and_holds_no_credential`
- `tests::a_chunked_answer_is_read_whole_and_unframed`
- `tests::a_fresh_secret_is_64_hexadecimal_digits_and_never_repeats`
- `tests::a_range_is_sent_as_a_head_or_a_suffix`
- `tests::a_recorded_request_holds_no_token`
- `tests::an_output_shows_its_lengths_and_no_text`
- `tests::the_app_ini_allows_the_webhook_host_it_is_given`
- `tests::the_app_ini_names_loopback_sqlite_and_the_page_cap`

### temper-throwaway-forgejo::throwaway

- `a_binary_printing_another_version_is_refused_naming_it`
- `a_failed_work_path_command_keeps_its_exit_code_and_both_streams`
- `a_scratch_directory_that_is_not_empty_is_refused`
- `a_throwaway_forgejo_starts_serves_its_admin_and_is_removed` (ignored)
- `a_work_path_is_refused_when_its_scratch_is_not_empty`
- `a_work_path_runs_commands_in_the_scratch_with_its_environment`

### temper-codecs-world::fuzzy_forgejo

- `forgejo_decoders_answer_every_mutated_transcript_without_panicking`
- `forgejo_delivery_decoders_answer_every_mutated_delivery_without_panicking`
- `forgejo_server_routes_answer_every_mutated_request_without_panicking`

## Selected dialect base forms of 06.3's commands

These select all seven moved focused binaries and the whole moved fuzzy binary;
no moved test is filtered to change its cost.

```
$ cargo nextest run -p temper-codecs-world --test 'forgejo_*'
# directory: /srv/data/git/runner/src/tmp/planning/temper-base
    Finished `test` profile [unoptimized + debuginfo] target(s) in 5.23s
    Starting 55 tests across 7 binaries
        PASS [   0.007s] (44/55) temper-codecs-world::forgejo_writes error_answers_meet_the_client_with_the_transcripts_shape
     Summary [   0.134s] 55 tests run: 55 passed, 0 skipped
```

```
$ cargo nextest run -p temper-codecs-world --test fuzzy_forgejo --profile fuzzy
# directory: /srv/data/git/runner/src/tmp/planning/temper-base
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.18s
    Starting 3 tests across 1 binary
     Summary [   2.524s] 3 tests run: 3 passed, 0 skipped
```

## Pinned binary proofs

The new script and packages do not exist at the base. The first command below
is 06.1's ignored proof with the old environment variable and package. The next
two commands together are the base form of 06.2/06.3/06.4's script: ordinary
recording first, then component recording to the same output. No base source
or committed recording was changed. Runtime output was kept only in scratch.

```
$ TEMPER_FORGEJO=$HOME/src/tmp/temper/forgejo/v17.0.0/forgejo-17.0.0-linux-amd64 cargo nextest run -p temper-throwaway-forgejo --run-ignored only
# directory: /srv/data/git/runner/src/tmp/planning/temper-base
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.10s
    Starting 1 test across 2 binaries (13 tests skipped)
     Summary [   7.898s] 1 test run: 1 passed (1 slow), 13 skipped
```

```
$ TEMPER_FORGEJO=$HOME/src/tmp/temper/forgejo/v17.0.0/forgejo-17.0.0-linux-amd64 TEMPER_FORGEJO_RUNNER=$HOME/src/tmp/temper/forgejo/runner-v13.2.0/forgejo-runner-13.2.0-linux-amd64 cargo run -p forgejo-conformance -- --scratch /srv/data/git/runner/src/tmp/planning/scratch-skein-06/target/conformance-instance --out /srv/data/git/runner/src/tmp/planning/scratch-skein-06/target/conformance-base
# directory: /srv/data/git/runner/src/tmp/planning/temper-base
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 5.04s
```

```
$ TEMPER_FORGEJO=$HOME/src/tmp/temper/forgejo/v17.0.0/forgejo-17.0.0-linux-amd64 TEMPER_FORGEJO_RUNNER=$HOME/src/tmp/temper/forgejo/runner-v13.2.0/forgejo-runner-13.2.0-linux-amd64 cargo run -p forgejo-conformance -- --component --scratch /srv/data/git/runner/src/tmp/planning/scratch-skein-06/target/conformance-component --out /srv/data/git/runner/src/tmp/planning/scratch-skein-06/target/conformance-base
# directory: /srv/data/git/runner/src/tmp/planning/temper-base
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.21s
```

The ordinary run used 106 s alone (108 s including build/driver). The component
run used 22 s alone; reported 409 s because its exclusive wrapper waited
for a previous queued build to drain before invoking cargo. The 5-minute Proof
bound is execution, excluding queue/lock waits. Both processes exited 0.
The opt-in throwaway test was SLOW at 7.898 s because it starts/provisions a
process; it stays ignored in the focused suite and is never hidden by tuning.

## Facts and data at the base

```
$ python3 -c 'exec("from pathlib import Path\nfrom collections import Counter\nimport re,hashlib\nroot=Path(\"/srv/data/git/runner/src\")\nbase=root/\"tmp/planning/temper-base/tools/forgejo-conformance\"\nfresh=root/\"tmp/planning/scratch-skein-06/target/conformance-base\"\nfor root in [base,fresh]:\n lines=(root/\"facts.txt\").read_text().splitlines()\n facts=[x for x in lines if x and not x.startswith(\"#\")]\n print(root, dict(Counter(x.split()[0] for x in facts)),len(facts))\n print(\"component facts\",sum(x.split()[1].startswith(\"component-\") for x in facts))\n print(\"data:\",len(list((root/\"transcripts\").iterdir())),\"transcript entries;\",len(list((root/\"transcripts\").rglob(\"request\"))),\"cases;\",len(list((root/\"deliveries\").iterdir())),\"delivery directories\")\n if root==fresh:\n  print(\"\\n\".join(x for x in facts if not x.startswith(\"holds\")))\ndoc=Path(\"/srv/data/git/runner/src/tmp/planning/skein-base/docs/design/forge-setup.md\").read_text().split(\"## 8.\")[1].split(\"## 9.\")[0]\nnames=re.findall(r\"\\x60((?:setup-|delivery-loopback-|api-default-)[^\\x60]+)\\x60\",doc)\ntext=(fresh/\"facts.txt\").read_text()\nmissing=[n for n in names if not re.search(r\"^(holds|fails|unchecked) \"+re.escape(n)+r\"(?::|$)\",text,re.M)]\nprint(\"forge-setup 8:\",len(names),\"named;\",len(missing),\"missing;\",sum(\"holds \"+n+\"\\n\" in text for n in names),\"hold\")\npat=re.compile(rb\"[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}\")\nfor root in [base,fresh]:\n uuids=[v for p in (root/\"transcripts\").rglob(\"*\") if p.is_file() for v in pat.findall(p.read_bytes()) if not v.startswith(b\"00000000-0000-0000-0000-\")]\n print(root.name,\"unmasked UUID occurrences:\",len(uuids))\n")'
# directory: /srv/data/git/runner/src/tmp/planning/temper-base
/srv/data/git/runner/src/tmp/planning/temper-base/tools/forgejo-conformance {'holds': 211, 'fails': 7} 218
component facts 40
data: 152 transcript entries; 151 cases; 24 delivery directories
/srv/data/git/runner/src/tmp/planning/scratch-skein-06/target/conformance-base {'holds': 208, 'fails': 10} 218
component facts 40
data: 152 transcript entries; 151 cases; 24 delivery directories
fails merge-old-head-is-409-head-out-of-date: HTTP 405: "Please try again later"
fails pull-for-names-the-newest-of-several: 12
fails open-pull-no-changes-is-422: HTTP 201: null
fails delete-missing-branch-is-404: HTTP 500: ""
fails effective-protection-name-for-a-writer: effective_branch_protection_name=""
fails delivery-branch-deletion-sends-no-push: job statuses=[true, true], pull mergeability=[[true], [true]]
fails delivery-pull-reads-not-mergeable-while-checked: job statuses=[true, true], pull mergeability=[[true], [true]]
fails delivery-job-end-precedes-its-commit-status: job statuses=[true, true], pull mergeability=[[true], [true]]
fails delivery-pull-is-checking-when-its-push-is-delivered: job statuses=[true, true], pull mergeability=[[true], [true]]
fails webhook-deliveries: delivery facts failed
forge-setup 8: 36 named; 0 missing; 36 hold
forgejo-conformance unmasked UUID occurrences: 4
conformance-base unmasked UUID occurrences: 0
```

```
$ sh -c 'awk "/^(holds|fails|unchecked) / {sub(/:.*/, \"\", \$2); print \$1, \$2}" /srv/data/git/runner/src/tmp/planning/scratch-skein-06/target/conformance-base/facts.txt | LC_ALL=C sort | sha256sum'
# directory: /srv/data/git/runner/src/tmp/planning/temper-base
e8996bbcf191f5f11e955823c061e6ad3bf80c663bf2c0b322563036fcf974ec  -
```

The SHA-256 compares sorted **outcome and fact name**, ignoring provenance and
volatile reason text. This is the freshly measured baseline requested by README
Done when 7: **218 facts: 208 holds, 10 fails, 0 unchecked; all 40 component facts
hold; all 36 forge-setup section 8 facts hold.** Exit 0 alone would miss failures.

The committed baseline had 211 holds and 7 fails. Five individual outcomes
changed in this run (timing-sensitive observations are not stable merely because
the binaries are pinned):

| Fact | Committed | This run |
|---|---|---|
| merge-old-head-is-409-head-out-of-date | holds | fails |
| delivery-pull-reads-not-mergeable-while-checked | holds | fails |
| delivery-pull-is-settled-when-its-push-is-delivered | fails | holds |
| delivery-pull-is-checking-when-its-push-is-delivered | holds | fails |
| webhook-deliveries | holds | fails |

Do not weaken or silently replace these observations to obtain a passing hash.
If a future proof changes an outcome, report it with its recorded reason for the
planner/designer; this draft does not decide whether timing-dependent facts
should acquire a different final comparison policy.

The recorded binary provenance:

- forgejo forgejo version 17.0.0+gitea-1.22.0 (release name 17.0.0) built with GNU Make 4.4.1, go1.26.9 : bindata, timetzdata, sqlite, sqlite_unlock_notify sha256 5a3b07e124ed70b4e46ee3939515f003bd3cc85352257a9a4607a527251042ca
- runner forgejo-runner version v13.2.0

All captured named outcomes, retained here before removing scratch:

| Fact | Outcome |
|---|---|
| api-default-page-cap-50-and-30 | holds |
| pull-for-accepts-encoded-slash | holds |
| update-conflict-is-409-merge-failed | holds |
| merge-old-head-is-409-head-out-of-date | fails |
| merge-answers-200-with-no-body | holds |
| pull-files-page-with-x-hasmore | holds |
| compare-ignores-page-and-limit | holds |
| create-branch-at-full-commit-id | holds |
| writer-protection-reads-are-403 | holds |
| leastupdate-orders-least-recently-updated-first | holds |
| leastupdate-ties-ordered-by-number | holds |
| since-is-inclusive-at-the-second | holds |
| since-in-2554-answers-an-empty-page-with-date | holds |
| updated-moves-on-comment | holds |
| updated-stays-on-comment-edit | holds |
| updated-stays-on-comment-delete | holds |
| updated-moves-on-review | holds |
| updated-moves-on-head-push | holds |
| updated-stays-on-head-status | holds |
| updated-stays-on-base-push | holds |
| updated-moves-on-close | holds |
| updated-moves-on-reopen | holds |
| unedited-comment-updated-equals-created | holds |
| unedited-review-updated-equals-submitted | holds |
| edit-moves-comment-updated | holds |
| marker-line-feed-kept | holds |
| body-of-65536-bytes-kept-whole | holds |
| body-of-1048576-bytes-kept-whole | holds |
| patch-issue-edits-a-pull-request | holds |
| pull-for-names-the-newest-of-several | fails |
| branch-path-takes-encoded-slash | holds |
| base-sha-is-the-base-tip | holds |
| combined-status-without-status-counts-0 | holds |
| skipped-and-success-combine-as-passed | holds |
| comments-come-oldest-first | holds |
| reviews-come-oldest-first | holds |
| dismissed-review-keeps-its-body | holds |
| remark-position-is-its-line | holds |
| raw-range-answers-206 | holds |
| merge-closed-is-405 | holds |
| merge-conflict-is-405 | holds |
| merge-protected-is-405 | holds |
| merge-style-is-405 | holds |
| open-pull-exists-is-409 | holds |
| open-pull-no-changes-is-422 | fails |
| empty-title-is-422 | holds |
| moved-repository-is-a-redirect | holds |
| archived-write-is-423 | holds |
| delete-protected-branch-is-403 | holds |
| delete-missing-branch-is-404 | fails |
| create-branch-exists-is-409 | holds |
| unreadable-repository-by-path-is-404 | holds |
| unreadable-repository-by-id-is-404 | holds |
| effective-protection-name-for-a-writer | fails |
| effective-protection-name-for-an-admin | holds |
| no-rate-limit-headers | holds |
| runner-register-secret-file-prints-the-uuid | holds |
| runner-register-refuses-a-secret-of-another-length | holds |
| runner-register-refuses-a-secret-that-is-not-hexadecimal | holds |
| runner-register-again-keeps-one-global-runner | holds |
| runner-create-runner-file-takes-its-secret-only-as-an-argument | holds |
| runner-uuid-is-the-secrets-first-sixteen-characters-in-hex | holds |
| runner-token-is-the-secret | holds |
| runner-connection-with-a-token-file-comes-online | holds |
| runner-connection-alone-takes-the-jobs | holds |
| actions-report-each-job-as-a-status-on-its-head | holds |
| actions-status-link-is-runs-n-jobs-i | holds |
| actions-status-link-names-no-attempt | holds |
| run-number-is-index-in-repo | holds |
| run-jobs-come-in-index-order | holds |
| job-log-range-answers-206 | holds |
| job-steps-number-from-0-set-up-first-complete-last | holds |
| failed-workflow-step-and-completion-step-are-failure | holds |
| step-log-holds-that-steps-lines-alone | holds |
| step-log-suffix-range-answers-206-with-its-last-bytes | holds |
| step-log-suffix-content-range-names-its-first-byte | holds |
| empty-log-suffix-range-answers-206-empty | holds |
| step-past-the-last-is-404 | holds |
| runner-a-jobs-token-clones-its-private-repository | holds |
| compare-files-false-lists-no-commits-files | holds |
| compare-files-false-keeps-its-own-files-empty | holds |
| compare-verification-false-changes-nothing-kept | holds |
| compare-files-false-keeps-total-commits-and-shas | holds |
| setup-create-user-answers-201-with-login-email-and-full-name | holds |
| setup-created-user-is-no-site-admin-and-keeps-its-full-name | holds |
| setup-create-user-again-is-refused-with-a-message | holds |
| setup-user-of-a-login-reads-admin-flag-and-full-name | holds |
| setup-user-of-a-missing-login-is-404 | holds |
| setup-a-non-admin-token-is-refused-the-admin-routes | holds |
| setup-the-admin-token-reads-user-as-a-site-admin | holds |
| setup-a-user-made-to-change-its-password-is-refused-the-api-under-basic-auth | holds |
| setup-a-user-made-to-change-its-password-is-refused-under-its-token-until-the-admin-clears-the-flag | holds |
| setup-every-recorded-user-carries-is-admin-and-full-name | holds |
| setup-admin-route-makes-a-token-answering-201-with-name-scopes-and-value | holds |
| setup-token-of-an-existing-name-is-refused | holds |
| setup-token-list-shows-names-and-scopes-without-values | holds |
| setup-token-list-pages-by-page-and-limit | holds |
| setup-a-made-token-answers-user-as-the-bot | holds |
| setup-a-git-token-of-write-repository-alone-is-refused-user | holds |
| setup-a-git-token-of-write-repository-and-read-user-answers-user-as-the-bot | holds |
| setup-token-delete-answers-204-and-ends-the-token | holds |
| setup-token-delete-of-a-missing-id-is-404 | holds |
| setup-admin-token-without-write-admin-is-refused-the-token-routes | holds |
| setup-admin-token-without-a-user-scope-is-refused-user | holds |
| setup-admin-token-with-read-user-in-place-of-write-user-reads-user | holds |
| setup-api-token-without-write-issue-is-refused-an-issue-write | holds |
| setup-git-token-with-write-repository-opens-the-push-discovery | holds |
| setup-git-token-without-it-is-refused | holds |
| setup-org-made-for-a-user-by-an-admin-is-201-and-private | holds |
| setup-org-made-again-is-422 | holds |
| setup-org-read-by-name-names-its-id | holds |
| setup-org-missing-is-404 | holds |
| setup-org-route-orgs-needs-write-organization | holds |
| setup-org-made-for-a-user-takes-write-admin-alone | holds |
| setup-org-starts-with-only-the-owners-team | holds |
| setup-team-made-with-write-all-repositories-and-four-units | holds |
| setup-team-listing-reads-back-what-was-made | holds |
| setup-team-made-again-is-422 | holds |
| setup-team-edit-without-a-name-keeps-the-name | holds |
| setup-team-edit-replaces-units-and-flags | holds |
| setup-team-member-put-is-204 | holds |
| setup-team-members-list-the-added-user | holds |
| setup-team-member-put-of-a-missing-user-is-404 | holds |
| setup-team-member-writes-in-a-repository-made-later | holds |
| setup-team-non-member-cannot-see-the-private-repository | holds |
| setup-team-listing-pages-by-limit | holds |
| setup-hook-for-a-loopback-address-is-made | holds |
| setup-hook-lists-the-events-it-was-made-with | holds |
| setup-hook-answers-its-target-in-config-url | holds |
| setup-hook-is-json-and-active | holds |
| setup-hook-never-shows-its-secret | holds |
| setup-hook-patch-narrows-events-but-keeps-the-workflow-job-ones | holds |
| setup-hook-patch-leaves-the-orgs-other-hook | holds |
| setup-hook-patch-of-a-missing-id-is-404 | holds |
| setup-hook-listing-pages-by-limit | holds |
| setup-hook-of-one-url-can-be-made-twice | holds |
| setup-hook-delete-is-204-and-leaves-the-orgs-other-hooks | holds |
| setup-hook-delete-of-a-missing-id-is-404 | holds |
| cli-migrate-runs-on-a-fresh-work-path | holds |
| cli-migrate-runs-again-on-a-migrated-work-path | holds |
| cli-user-create-prints-the-random-password-first | holds |
| cli-user-create-random-password-length-is-asked | holds |
| cli-user-create-again-is-refused-naming-the-user | holds |
| cli-user-list-prints-a-header-and-a-row-per-user | holds |
| cli-user-create-random-password-signs-in | holds |
| cli-token-raw-prints-the-token-alone | holds |
| cli-token-scopes-limit-what-the-token-reads | holds |
| cli-token-name-used-twice-is-refused | holds |
| cli-token-admin-scopes-comma-joined-are-taken-under-two-suffixes | holds |
| cli-user-made-to-change-its-password-is-refused-the-api-until-cleared | holds |
| cli-dump-needs-the-repositories-directory | holds |
| cli-dump-writes-the-database-and-app-ini | holds |
| cli-dump-appends-zip-to-a-file-name-without-it | holds |
| cli-dump-needs-its-tempdir-to-exist | holds |
| cli-dump-over-an-existing-file-overwrites-it | holds |
| delivery-issues-row | holds |
| delivery-pull-requests-row | holds |
| delivery-comments-row | holds |
| delivery-reviews-row | holds |
| delivery-push-row | holds |
| delivery-cancelled-job-end-is-delivered | holds |
| delivery-create-delete-row | holds |
| delivery-job-ends-row | holds |
| delivery-carries-the-repository-id | holds |
| delivery-subscribed-events-read-back | holds |
| delivery-signature-is-hmac-sha256-hex | holds |
| delivery-refs-are-full-and-tags-are-told-apart | holds |
| delivery-branch-deletion-sends-no-push | fails |
| delivery-echoes-the-actors-own-writes | holds |
| delivery-loopback-is-refused-by-default | holds |
| delivery-loopback-is-allowed-when-named | holds |
| delivery-failed-delivery-is-not-sent-again | holds |
| delivery-pull-reads-not-mergeable-while-checked | fails |
| delivery-job-end-follows-its-commit-status | holds |
| delivery-job-end-precedes-its-commit-status | fails |
| delivery-pull-is-settled-when-its-push-is-delivered | holds |
| delivery-pull-is-checking-when-its-push-is-delivered | fails |
| webhook-deliveries | fails |
| component-start-ready | holds |
| component-start-other-user-stops | holds |
| component-start-wrong-token-stops | holds |
| component-read-items | holds |
| component-read-item | holds |
| component-read-pull | holds |
| component-read-pull-for | holds |
| component-read-reviews | holds |
| component-read-statuses | holds |
| component-read-remarks | holds |
| component-read-branch | holds |
| component-read-branches | holds |
| component-read-pull-files | holds |
| component-read-compare | holds |
| component-read-checks | holds |
| component-read-job | holds |
| component-read-job-answers-the-failed-steps-end | holds |
| component-read-file | holds |
| component-read-protection | holds |
| component-read-settings | holds |
| component-read-collaborators | holds |
| component-read-permission | holds |
| component-read-locate | holds |
| component-write-create-issue | holds |
| component-write-open-pull | holds |
| component-write-post | holds |
| component-write-review | holds |
| component-write-edit | holds |
| component-write-set-reviewers | holds |
| component-write-close | holds |
| component-write-reopen | holds |
| component-write-merge | holds |
| component-write-update | holds |
| component-write-status | holds |
| component-write-create-branch | holds |
| component-write-delete-branch | holds |
| component-keys-found-on-read-back | holds |
| component-costs-equal-requests-sent | holds |
| component-read-pull-for-open-after-closed | holds |
| component-write-open-pull-no-changes | holds |

## Source counts, exact names and users

The initial broad classifier below counted some top-level test files as source;
its corpus and rg counts are sound. The corrected production/test partition is
in the following sizing command: **728 + 168** throwaway lines; **4,161 + 324**
conformance lines; **2,405 + 30** dialect test/helper lines. Counts include blank
and documentation lines, as the move's diff count does; test helpers are excluded
from production Size. The 06.1 HTTP replacement and 06.2 JSON replacement are
new-line estimates, not measured implementations.

Multiline Python arguments are rendered below with Bash ANSI quoting so the
command fits on one physical line while passing exactly the original argument.

```
$ python3 -c $'import pathlib,subprocess,re,json\nroot=pathlib.Path("/srv/data/git/runner/src")\nbase=root/"tmp/planning/temper-base"\nrepos=[base,root/"tmp/planning/skein-base",root/"jig",root/"smith",root/"hearth"]\nfor group,rel in [("throwaway","testing/temper-throwaway-forgejo"),("conformance","tools/forgejo-conformance"),("codecs","tests/codecs")]:\n files=[p for p in (base/rel).rglob("*.rs")]\n prod=[]; test=[]\n for p in files:\n  s=p.read_text()\n  if "/tests/" in str(p.relative_to(base/rel)) or p.name=="tests.rs": test.append((p,len(s.splitlines())))\n  else:\n   if "#[cfg(test)]" in s: s=s.split("#[cfg(test)]")[0]\n   prod.append((p,len(s.splitlines())))\n print(group,"source lines",sum(n for p,n in prod),"test lines",sum(n for p,n in test),"max",max((n,p.name) for p,n in prod+test))\nprint("serde files",subprocess.run(["rg","-l","serde",str(base/"tools/forgejo-conformance/src")],text=True,capture_output=True).stdout.count("\\n"))\np=base/"tools/forgejo-conformance"\nprint("data:",len(list((p/"transcripts").rglob("*"))),"entries;",len([x for x in (p/"transcripts").iterdir() if x.is_dir()]),"transcript directories;",len([x for x in (p/"transcripts").rglob("*") if x.is_file()]),"files;",len([x for x in (p/"deliveries").iterdir() if x.is_dir()]),"deliveries")\nfor pat in [r"temper_throwaway_forgejo::",r"Instance::attach",r"Forgejo::from_env",r"tools/forgejo-conformance/(transcripts|deliveries)",r"temper_codecs_world::forgejo",r"forgejo::profile_limits",r"forgejo_conformance::",r"TEMPER_FORGEJO(_RUNNER)?"]:\n row=[]\n for repo in repos:\n  result=subprocess.run(["rg","--glob","*.rs","-n",pat,str(repo)],text=True,capture_output=True)\n  row.append(str(len(result.stdout.splitlines())))\n print(pat,",".join(row),"[temper,skein,jig,smith,hearth]")\nprint("data readers:",subprocess.run(["rg","-l",r"tools/forgejo-conformance",str(base/"tests"),"--glob","*.rs"],text=True,capture_output=True).stdout.replace(str(base)+"/","").strip().replace("\\n",", "))\nprint("downstream profile literals:")\np=base/"crates/temper-service/src/limits.rs"\ns=p.read_text(); print("\\n".join(s.splitlines()[51:86]))\n'
# directory: /srv/data/git/runner/src/tmp/planning/temper-base
throwaway source lines 820 test lines 67 max (268, 'instance.rs')
conformance source lines 4159 test lines 324 max (568, 'facts.rs')
codecs source lines 2906 test lines 0 max (725, 'forgejo.rs')
serde files 17
data: 454 entries; 151 transcript directories; 303 files; 24 deliveries
Instance::attach 1,0,0,0,0 [temper,skein,jig,smith,hearth]
Forgejo::from_env 4,0,0,0,0 [temper,skein,jig,smith,hearth]
tools/forgejo-conformance/(transcripts|deliveries) 5,0,0,0,0 [temper,skein,jig,smith,hearth]
TEMPER_FORGEJO(_RUNNER)? 15,0,0,0,0 [temper,skein,jig,smith,hearth]
data readers: tests/engine/protocol-forge/tests/hooks.rs, tests/engine/protocol-forge/src/machine.rs, tests/codecs/src/forgejo.rs
```

```
$ python3 -c 'exec("from pathlib import Path\nimport re\nr=Path(\".\")\nthrow=r/\"testing/temper-throwaway-forgejo\"\ntool=r/\"tools/forgejo-conformance\"\ncodecs=r/\"tests/codecs\"\nfor root in [throw,tool]:\n prod=[p for p in (root/\"src\").rglob(\"*.rs\") if p.name!=\"tests.rs\"]\n tests=[p for p in root.rglob(\"*.rs\") if p.name==\"tests.rs\" or \"tests\" in p.relative_to(root).parts]\n print(root, \"production\",sum(len(p.read_text().splitlines()) for p in prod),\"test\",sum(len(p.read_text().splitlines()) for p in tests))\nfiles=[codecs/\"src/forgejo.rs\",*(codecs/\"tests\").glob(\"forgejo_*.rs\"),codecs/\"tests/fuzzy_forgejo.rs\"]\nprint(\"dialect helper and tests\",sum(len(p.read_text().splitlines()) for p in files),\"plus 30 lines of mutation helpers\")\npat=re.compile(rb\"20\\d\\d-\\d\\d-\\d\\dT\\d\\d:\\d\\d:\\d\\d\\.\\d{7}(?:Z|[+-]\\d\\d:\\d\\d)\")\nfor root in [tool,Path(\"/srv/data/git/runner/src/tmp/planning/scratch-skein-06/target/conformance-base\")]:\n rows=[(p.name,m.group()) for p in (root/\"transcripts\").rglob(\"response\") if p.parent.name in (\"actions-job-log\",\"actions-job-log-step\") for m in pat.finditer(p.read_bytes())]\n print(root.name,\"seven-fraction log times\",len(rows))\n")'
# directory: /srv/data/git/runner/src/tmp/planning/temper-base
testing/temper-throwaway-forgejo production 728 test 168
tools/forgejo-conformance production 4161 test 324
dialect helper and tests 2405 plus 30 lines of mutation helpers
forgejo-conformance seven-fraction log times 9
conformance-base seven-fraction log times 9
```

The source corpus has four unmasked UUID occurrences and nine unquoted log
timestamps with seven fractional digits. Re-recording with the base code removes
all UUID occurrences already (its UUID rule landed after the old recordings),
but leaves all nine log timestamps. Thus 06.4 needs a time-mask fix and a
re-recording, not a second UUID mechanism.

The exhaustive lexical search below takes **every function, struct and enum
name** in the moved production files and dialect helpers, then runs
`rg --glob '*.rs' -l '\\bNAME\\b' REPO` for each name in all five repositories.
Each tuple is matching-file counts in temper/skein/jig/smith/hearth order.
Generic spellings such as `run`, `Limits` and `Token` have unrelated matches;
these are candidates, not claimed cross-repository API dependencies. Qualified
crate searches above identify the users that must change. Each function/type
name from the stale graph was checked again this way in the base.

```
$ python3 -c 'exec("from pathlib import Path\nimport re,subprocess\nroot=Path(\"/srv/data/git/runner/src\")\nbase=root/\"tmp/planning/temper-base\"\nrepos=[base,root/\"tmp/planning/skein-base\",root/\"jig\",root/\"smith\",root/\"hearth\"]\nsources=sorted((base/\"testing/temper-throwaway-forgejo/src\").glob(\"*.rs\"))+sorted((base/\"tools/forgejo-conformance/src\").rglob(\"*.rs\"))+[base/\"tests/codecs/src/forgejo.rs\",base/\"tests/codecs/src/lib.rs\"]\nfor source in sources:\n if source.name==\"tests.rs\":continue\n names=sorted(set(re.findall(r\"\\b(?:fn|struct|enum)\\s+([A-Za-z_][A-Za-z_0-9]*)\",source.read_text())))\n rows=[]\n for name in names:\n  counts=[]\n  for repo in repos:\n   result=subprocess.run([\"rg\",\"--glob\",\"*.rs\",\"-l\",r\"\\b\"+name+r\"\\b\",str(repo)],text=True,capture_output=True)\n   if result.returncode not in (0,1):raise RuntimeError(result.stderr)\n   counts.append(len(result.stdout.splitlines()))\n  rows.append(name+\"=\"+\"/\".join(map(str,counts)))\n print(str(source.relative_to(base))+\": \"+\", \".join(rows))\n")'
# directory: /srv/data/git/runner/src/tmp/planning/temper-base
exit 0; 26 source-file groups searched; detailed counts below.
```

| Source | Symbol = matching files (temper/skein/jig/smith/hearth) |
|---|---|
| testing/temper-throwaway-forgejo/src/binary.rs | Forgejo=97/0/0/0/0, Refusal=95/89/326/168/28, binary=37/15/9/33/43, environment=33/13/71/85/85, from_env=5/0/0/0/0, from_path=2/0/1/0/0, sha256=20/11/2/29/97, start=208/178/242/232/73, version=92/90/170/137/88 |
| testing/temper-throwaway-forgejo/src/cli.rs | Output=43/77/48/78/85, WorkPath=4/0/0/0/0, checked=126/185/267/176/67, command=55/37/9/172/40, fmt=31/49/20/19/16, prepare=52/21/36/42/9, random=24/60/35/28/3, run=253/210/490/456/103, run_user=1/0/0/0/0, scratch=38/34/26/49/41, work_path=8/0/0/0/0 |
| testing/temper-throwaway-forgejo/src/http.rs | basic=8/3/0/0/0, exchange=43/94/82/13/1, parse=37/55/36/51/38, request=297/239/598/253/126 |
| testing/temper-throwaway-forgejo/src/instance.rs | Instance=8/2/0/0/0, add_collaborator=3/0/0/0/0, admin=59/0/0/0/1, attach=4/2/0/6/0, base=146/28/37/40/88, cleanup=5/17/11/39/5, cli=10/1/0/3/0, create_repository=4/0/0/0/0, create_user=3/0/0/0/0, drop=48/87/85/76/65, exchange=43/94/82/13/1, exchange_basic=3/0/0/0/0, expect=519/533/1243/478/479, fmt=31/49/20/19/16, ini=17/0/0/0/0, log_tail=3/0/0/0/0, mask=13/3/6/2/4, port=30/30/11/4/31, ready=90/84/180/42/30, scratch=38/34/26/49/41, secrets=64/11/40/10/66, start=208/178/242/232/73, stop=52/55/75/138/8, token=218/212/373/154/27, value=285/237/950/305/310 |
| testing/temper-throwaway-forgejo/src/lib.rs | Account=43/13/96/21/0, Exchange=27/48/54/5/0, Failure=81/57/103/125/34, Init=5/0/0/0/0, Made=34/31/165/11/46, Outgoing=11/8/1/1/0, Range=76/15/53/0/52, Settings=136/34/106/82/37, Token=267/239/630/266/62, fmt=31/49/20/19/16, fresh_secret=3/0/0/0/0, from=450/484/960/459/270, get=241/217/563/206/127, json=91/70/27/90/180, new=545/580/1365/528/421, small=92/86/114/72/32, value=285/237/950/305/310 |
| tools/forgejo-conformance/src/cases.rs | Auth=3/0/0/0/0, Recorder=6/0/0/0/0, Written=8/12/23/44/21, failures=31/29/68/35/6, flags=19/16/153/13/1, made_token=2/0/0/0/0, reads=109/144/282/95/60, record=137/178/500/247/168, send=88/146/237/45/8, send_new=3/0/0/0/0, start=208/178/242/232/73, w=45/4/237/5/9, writes=102/94/213/71/13 |
| tools/forgejo-conformance/src/cli.rs | create=65/19/41/27/6, created=65/16/51/19/2, dump=13/0/0/0/2, dumps=1/0/1/0/0, facts=129/21/486/145/11, get=241/217/563/206/127, password_of=2/0/0/0/0, raw_token=2/0/0/0/0, remember=3/1/5/4/0, seen=38/93/179/97/7, token=218/212/373/154/27, tokens=85/130/63/144/7, zip=41/60/61/44/49 |
| tools/forgejo-conformance/src/component/hosted.rs | Hosted=7/3/22/1/6, call=160/201/314/301/82, completions=27/84/24/87/8, finish=168/181/354/221/101, is_empty=265/345/537/302/134, iterate=56/53/34/28/8, limits=395/339/1033/361/292, new=545/580/1365/528/421, next_deadline=67/95/98/67/16, next_policy_deadline=18/38/13/20/6, operations=69/101/143/87/57, pump=20/10/11/1/0, sent_requests=2/0/0/0/0, started=51/25/99/119/10, submissions=26/67/19/40/9, work_pending=31/48/24/27/24, worst_case=120/173/313/140/39 |
| tools/forgejo-conformance/src/component/reads.rs | actions=56/3/42/3/0, bytes=381/448/836/384/392, commit=237/21/202/52/57, repository=267/2/0/71/121, run=253/210/490/456/103 |
| tools/forgejo-conformance/src/component/writes.rs | effects=25/14/168/74/1, empty_pull=1/0/0/0/0, open_after_closed=1/0/0/0/0, run=253/210/490/456/103 |
| tools/forgejo-conformance/src/component.rs | fact=41/5/195/92/0, observe=38/52/69/59/2, operation=75/73/120/117/23, record=137/178/500/247/168, run=253/210/490/456/103 |
| tools/forgejo-conformance/src/deliveries.rs | Capture=7/1/14/66/0, Listener=14/16/2/0/0, Received=11/5/9/14/0, Reply=83/40/245/38/31, Scenario=15/14/19/14/21, act=55/64/183/54/3, answer=272/190/561/313/81, evaluate=3/3/0/5/94, header=51/89/100/44/20, next=185/255/368/198/108, provision=2/5/8/4/0, pull_pair=2/0/0/0/0, receive=21/88/34/33/1, record=137/178/500/247/168, reread=3/1/7/1/0, response=49/125/43/121/2, roll_up=2/0/0/0/0, scenario=39/82/54/34/30, signed=26/26/77/10/7, start=208/178/242/232/73, take=133/179/228/105/74 |
| tools/forgejo-conformance/src/facts.rs | Case=11/13/30/3/14, Fact=66/6/168/85/0, Outcome=95/64/245/98/65, add=31/23/26/34/13, bodies=26/20/25/42/3, case=61/71/58/51/42, errors=9/23/28/20/226, evaluate=3/3/0/5/94, flags=19/16/153/13/1, initial=16/17/55/43/8, line=95/140/192/208/177, listing=34/9/11/34/69, matches=105/145/331/226/40, orders=7/2/8/1/3, pause=6/4/10/7/0, pulls=54/2/0/0/0, replace_block=3/0/0/0/0, updated=29/2/4/5/0, utc=1/0/0/1/0, write=223/180/187/212/66 |
| tools/forgejo-conformance/src/fixture.rs | Fixture=24/2/66/19/0, api=231/36/8/67/0, api_answer=2/0/0/0/0, bare=2/1/1/1/3, base64=18/13/17/7/0, body_preview=1/0/0/0/0, branch=216/3/0/19/16, encode=101/44/193/40/109, exchange=43/94/82/13/1, exchange_basic=3/0/0/0/0, get=241/217/563/206/127, new=545/580/1365/528/421, pull=106/7/1/2/1, put=83/54/101/44/77, ready=90/84/180/42/30 |
| tools/forgejo-conformance/src/hooks.rs | body=238/176/296/113/65, captures_hide_secrets=1/0/0/0/0, event_set=1/0/0/0/0, facts=129/21/486/145/11, no_secret=1/0/0/0/0, record=137/178/500/247/168 |
| tools/forgejo-conformance/src/job_clone.rs | fact=41/5/195/92/0, workflow=8/0/0/3/0 |
| tools/forgejo-conformance/src/lib.rs | failure=120/155/164/241/43, json=91/70/27/90/180, run=253/210/490/456/103 |
| tools/forgejo-conformance/src/main.rs | main=113/32/22/138/67 |
| tools/forgejo-conformance/src/mask.rs | Mask=10/0/0/0/5, bytes=381/448/836/384/392, git_packets=1/0/0/0/0, listener=57/77/23/4/3, literal=12/30/68/28/29, new=545/580/1365/528/421, plain=21/57/19/38/17, request=297/239/598/253/126, response=49/125/43/121/2, sorted_units=1/0/0/0/0 |
| tools/forgejo-conformance/src/orgs.rs | facts=129/21/486/145/11, record=137/178/500/247/168, write_team=1/0/0/0/0 |
| tools/forgejo-conformance/src/registration.rs | capture=9/10/10/92/5, configuration=82/46/133/128/28, facts=129/21/486/145/11, held=147/169/462/104/50, listed=36/24/118/36/71, offline=6/3/32/19/0, online=2/0/6/0/0, private=68/50/38/70/14, register=5/2/3/0/0 |
| tools/forgejo-conformance/src/runner.rs | Runner=12/0/0/0/1, capture=9/10/10/92/5, command=55/37/9/172/40, component=84/42/97/70/40, connection_facts=1/0/0/0/0, contains=124/140/274/182/82, content_range=7/0/0/0/0, drop=48/87/85/76/65, exchange=43/94/82/13/1, job_steps=1/0/0/0/0, jobs_in_index_order=1/0/0/0/0, record=137/178/500/247/168, rows=200/30/695/18/27, run=253/210/490/456/103, run_with=1/0/0/1/7, start=208/178/242/232/73, valid_link=1/0/0/0/0, version=92/90/170/137/88 |
| tools/forgejo-conformance/src/setup.rs | facts=129/21/486/145/11, record=137/178/500/247/168, scoped=11/3/43/17/2 |
| tools/forgejo-conformance/src/tokens.rs | bot=30/0/0/0/0, facts=129/21/486/145/11, make=37/54/114/29/47, record=137/178/500/247/168 |
| tools/forgejo-conformance/src/users.rs | facts=129/21/486/145/11, record=137/178/500/247/168 |
| tools/forgejo-conformance/src/write.rs | Leak=2/0/0/0/0, SecretKind=9/0/0/0/0, Secrets=21/0/30/0/57, add=31/23/26/34/13, case=61/71/58/51/42, check=158/147/175/240/82, delivery=112/59/136/152/9, fmt=31/49/20/19/16, from_instance=4/0/0/0/0, of=248/369/495/261/152 |
| tests/codecs/src/forgejo.rs | Decoded=18/15/46/70/46, DeliveryRequest=1/0/0/0/0, Kind=101/66/175/87/183, Transcript=23/7/41/77/0, body=238/176/296/113/65, decode=126/54/238/47/38, decode_answer=3/0/6/3/0, decode_body=1/0/0/0/0, decode_delivery=3/0/0/4/0, decode_error=5/11/0/0/11, decode_refusal=4/0/7/0/0, decode_route=4/0/0/0/0, delivery=112/59/136/152/9, delivery_cases=3/0/2/0/0, drive=22/32/99/24/4, json_cases=2/0/0/0/0, limits=395/339/1033/361/292, page=173/90/343/4/16, profile_limits=2/0/9/0/0, route=117/35/135/23/0, setup_cases=2/0/0/0/0, setup_hook_cases=1/0/0/0/0, setup_hook_events=1/0/0/0/0, setup_hook_spec=1/0/0/0/0, setup_org_cases=1/0/0/0/0, setup_team_spec=1/0/0/0/0, sha=29/1/0/0/5, shape=28/38/65/24/35, specimen=23/0/0/0/0, transcript=68/17/132/157/0, transcript_cases=2/0/0/0/0, user=168/58/7/56/62, walk=26/5/12/8/22, write_cases=2/0/0/0/0, write_other_cases=1/0/0/0/0 |
| tests/codecs/src/lib.rs | drawn=28/39/21/19/12, goldens=5/6/5/3/6, mutated=6/10/2/3/5, mutated_into=2/0/0/0/0 |

Qualified users and ownership:

- `temper_throwaway_forgejo::`: 42 Rust lines, all in temper: recorder,
  tools/git-transcripts, crates/temper process tests/support and its ignored
  test. `Instance::attach` has one caller, support/owned.rs; temper 06.1 keeps
  that call's arguments and repoints its import. The kit reads its admin login
  from the API, per designer answer 30. The two conformance calls are main.rs's.
- `Forgejo::from_env`: four callers (recorder normal/component, git-transcripts,
  ignored throwaway test). The copied recorder/test change here; temper 06.1
  changes git-transcripts and keeps environment reading in temper's owner.
- Corpus paths: five Rust lines in three files: codecs/src/forgejo.rs,
  protocol-forge/src/machine.rs, protocol-forge/tests/hooks.rs. This session
  moves the first; skein 03.4/07.3 move the latter readers; temper 07 removes
  their old paths and data after those capabilities land.
- No qualified throwaway or corpus user was found in skein, jig, smith or
  hearth. Generic lexical collisions above are not additional Needs.

## Scratch signature and fixture checks

```
$ git worktree add --detach /srv/data/git/runner/src/tmp/planning/scratch-skein-06 27995fb75
# directory: /srv/data/git/runner/src/temper
Preparing worktree (detached HEAD 27995fb75)
HEAD is now at 27995fb75 Point temper's forge documents to skein's forge kit
```

The first clippy compiled the then-recommended attachment method in place and the path
arguments through conformance main, lib, component and runner, including all
callers in those packages. The second added the `skein_json::Document` return
sketches with divergent placeholder bodies and the full unquoted-time reference
test. It checks types/lints only; it does not claim the HTTP/JSON implementations
or new mask behavior exist. Existing type names substitute for their future
skein paths; the ready check must compile the renamed sketch against landed kit
crates. Answer 30 supersedes the extra login argument; the final attachment
signature is identical to the base's, already exercised by its process tier.
No new production type was introduced by a sketch.

```
$ cargo clippy -p temper-throwaway-forgejo -p forgejo-conformance --all-targets -- -D warnings
# directory: /srv/data/git/runner/src/tmp/planning/scratch-skein-06
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 6.77s
```

```
$ cargo clippy -p temper-throwaway-forgejo -p forgejo-conformance --all-targets -- -D warnings
# directory: /srv/data/git/runner/src/tmp/planning/scratch-skein-06
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 3.34s
```

```
$ cargo clippy -p temper-codecs-world --all-targets -- -D warnings
# directory: /srv/data/git/runner/src/tmp/planning/scratch-skein-06
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 6.92s
```

The codecs clippy also compiled the full limits reference test shown in 06.3.
The fixture probe below calls the shipped profile and asserts
`temper_forge_forgejo::limits::check(&limits) == Ok(())`, then prints its values:

```
$ cargo run -p temper-codecs-world --example forge_profile
# directory: /srv/data/git/runner/src/tmp/planning/scratch-skein-06
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 17.51s
Limits { json: Limits { depth: 16, string: 16384, number: 32, chunk: 4096, length: 16777216 }, writer: Limits { depth: 16, length: 16777216 }, string_bytes: 512, body_bytes: 16384, inner: 32, target_bytes: 4096 }
```

## Base forms of the remaining prescribed checks

The future neutral/source scans print zero. Their base forms deliberately expose
307 system/serde lines and four throwaway environment/raw-TCP/sleep lines.
Those positive counts are planned removals, not missing paths or failed cargo
commands. The diff compares data against the untouched data in the scratch copy,
before any recording; the new-home path is the only substitution.

```
$ sh -c 'rg -n -i "temper|serde" testing/temper-throwaway-forgejo tools/forgejo-conformance tests/codecs/src/forgejo.rs tests/codecs/tests/forgejo_*.rs tests/codecs/tests/fuzzy_forgejo.rs | wc -l'
# directory: /srv/data/git/runner/src/tmp/planning/temper-base
307
```

```
$ sh -c 'rg -n "from_env|TcpStream|thread::sleep" testing/temper-throwaway-forgejo/src | wc -l'
# directory: /srv/data/git/runner/src/tmp/planning/temper-base
4
```

```
$ sh -c 'for p in transcripts deliveries facts.txt; do diff -qr "$1/$p" "$2/$p" || exit; done' sh /srv/data/git/runner/src/tmp/planning/temper-base/tools/forgejo-conformance /srv/data/git/runner/src/tmp/planning/scratch-skein-06/tools/forgejo-conformance
# directory: /srv/data/git/runner/src/tmp/planning/temper-base
```

## Unsettled items

- Designer answers 30 and 31 have been incorporated; neither remains pending.
- README Done when 7 can be checked against the fresh 218-fact baseline here,
  but five outcomes differ from the committed baseline. A changed result at
  execution must be reported; accepting a range of timing outcomes would be a
  new designer decision, not a quiet change to this draft.
- No new cross-session capability dependency was needed. The serde/profile
  corrections are home-forced changes, already scoped to 06.3.
- All prescribed commands ran successfully in base form. Future names/counts
  (61 focused dialect, 4 fuzzy, 27 conformance) require the planned new tests;
  no base run is claimed to have exercised those not-yet-written behaviors.

## Cleanup and document checks

```
$ git worktree remove --force /srv/data/git/runner/src/tmp/planning/scratch-skein-06
# directory: /srv/data/git/runner/src/temper
exit 0; scratch worktree and its generated recordings removed
```

The session's final review incorporated designer answers 30/31 from decisions.md
and `git show d9f3e39:docs/design/fake-forge.md`. Base code/tests did not change.
The attachment signature now matches the base, so the extra-login sketch is
historical evidence only. The supplied `base-runs.md` process-tier run covers
the existing attachment caller; its import is repinned in temper 06.

The session is below 400 lines, has four increments and every required part.
Every command-output fence is at most 30 lines. Both clean base worktrees'
`git status --short` print nothing. No commit or landing was made.



## Revision after the 2026-10-19 review

The planner's rulings supersede the earlier whole-218-fact digest and the earlier
Unsettled items paragraph. No new Forgejo recording ran during this revision.
The two original recording commands remain the base forms for 06.2 and 06.4;
06.3 now has only document-test proofs. README 7, decisions 16 and later.md
remain the planner's edits, outside this task's two permitted files.

Graph discovery used search_graph on srv-data-git-runner-src-temper and
srv-data-git-runner-src-skein, plus get_code_snippet for the HTTP Response.
The former still missed Instance::attach; direct base reads/rg verified every
existing symbol used here. No index/list/status call was made.

### The comparison and the five races

The new command validates 218 unique named outcomes, each holds or fails.
It prints all five timing outcomes, including their recorded reason where present;
the other 213 must have the base digest (207 holds, 6 fails). It rejects unchecked,
missing, duplicate and additional facts; a stable name/outcome change fails the hash.
Provenance and volatile reason text are retained in facts.txt but excluded from hashing.

```
$ bash -o pipefail -c 'set -eu; digest=$(awk '\''BEGIN { split("merge-old-head-is-409-head-out-of-date delivery-pull-reads-not-mergeable-while-checked delivery-pull-is-settled-when-its-push-is-delivered delivery-pull-is-checking-when-its-push-is-delivered webhook-deliveries", a); for (i in a) timing[a[i]]=1 } /^#/ || NF==0 { next } { name=$2; sub(/:.*/, "", name); if (seen[name]++ || $1 !~ /^(holds|fails)$/) exit 1; n++;   if (name in timing) { t++; print "timing " $0 > "/dev/stderr" }   else { s++; h+=($1=="holds"); f+=($1=="fails"); print $1, name } } END { if (n!=218 || s!=213 || h!=207 || f!=6 || t!=5) exit 1;   print "facts=218 stable=213 holds=207 fails=6 timing=5 unchecked=0" > "/dev/stderr" }'\'' "$1" | LC_ALL=C sort | sha256sum); printf "%s\n" "$digest"; test "$digest" = "d1f7ba2bb323b8944e05d2e05bc1c83396fb7bfffadbcefea4faad64ec18347d  -"' sh tools/forgejo-conformance/facts.txt
# directory: /srv/data/git/runner/src/tmp/planning/temper-base
timing holds merge-old-head-is-409-head-out-of-date
timing holds delivery-pull-reads-not-mergeable-while-checked
timing fails delivery-pull-is-settled-when-its-push-is-delivered: job statuses=[true, true], pull mergeability=[[false, true], [false, true]]
timing holds delivery-pull-is-checking-when-its-push-is-delivered
timing holds webhook-deliveries
facts=218 stable=213 holds=207 fails=6 timing=5 unchecked=0
d1f7ba2bb323b8944e05d2e05bc1c83396fb7bfffadbcefea4faad64ec18347d  -
```
The fresh file below was reconstructed from this findings file's earlier 218-row
table, with names/outcomes only. It is evidence about the previously recorded run,
not a new process run; its reason text was not reconstructed. It gives the same hash.

```
$ bash -o pipefail -c 'set -eu; digest=$(awk '\''BEGIN { split("merge-old-head-is-409-head-out-of-date delivery-pull-reads-not-mergeable-while-checked delivery-pull-is-settled-when-its-push-is-delivered delivery-pull-is-checking-when-its-push-is-delivered webhook-deliveries", a); for (i in a) timing[a[i]]=1 } /^#/ || NF==0 { next } { name=$2; sub(/:.*/, "", name); if (seen[name]++ || $1 !~ /^(holds|fails)$/) exit 1; n++;   if (name in timing) { t++; print "timing " $0 > "/dev/stderr" }   else { s++; h+=($1=="holds"); f+=($1=="fails"); print $1, name } } END { if (n!=218 || s!=213 || h!=207 || f!=6 || t!=5) exit 1;   print "facts=218 stable=213 holds=207 fails=6 timing=5 unchecked=0" > "/dev/stderr" }'\'' "$1" | LC_ALL=C sort | sha256sum); printf "%s\n" "$digest"; test "$digest" = "d1f7ba2bb323b8944e05d2e05bc1c83396fb7bfffadbcefea4faad64ec18347d  -"' sh /srv/data/git/runner/src/tmp/planning/scratch-skein-06-revision/earlier-fresh-facts.txt
# directory: /srv/data/git/runner/src/tmp/planning/temper-base
timing fails merge-old-head-is-409-head-out-of-date
timing fails delivery-pull-reads-not-mergeable-while-checked
timing holds delivery-pull-is-settled-when-its-push-is-delivered
timing fails delivery-pull-is-checking-when-its-push-is-delivered
timing fails webhook-deliveries
facts=218 stable=213 holds=207 fails=6 timing=5 unchecked=0
d1f7ba2bb323b8944e05d2e05bc1c83396fb7bfffadbcefea4faad64ec18347d  -
```
The cause trace is in cases::failures -> Fixture::put/ready and deliveries::
Scenario::{take,reread}, scenario, evaluate, pull_pair and roll_up (rg checked).
Fixture::ready sees mergeable=true and the new head, then the merge request can
still race asynchronous mergeability work: the recorded 405 says Please try again
later rather than the expected stale-head 409. The delivery checking observation
depends on catching mergeable=false during create or after the matching push.
The settled/checking pair tests the first reads at the two matching pushes;
checking also needs reread to settle within **30 seconds**, as the base does.
roll_up inherits the checking observation and exactly one successful pair member;
it is not an independent fifth server race. Stabilizing them is later.md work.

### Paths, environment, pin users and recording bytes

```
$ sh -c 'rg -n "env::var|TEMPER_FORGEJO" tools/forgejo-conformance/src testing/temper-throwaway-forgejo/src | wc -l'
# directory: /srv/data/git/runner/src/tmp/planning/temper-base
8
```
The eight lines are five runner variable reads (runner::run/run_with,
deliveries::record, component::reads::run/actions), two runner-variable reason
strings and binary::from_env's Forgejo read. Both conformance Forgejo constructor
calls (lib::run and component::record) are also replaced. Main alone reads flags;
the optional runner is threaded through every one of those paths, including
delivery CI. Kit test errors explicitly name SKEIN_FORGEJO or SKEIN_FORGEJO_RUNNER.

```
$ sh -c 'rg -n "Forgejo::from_env|Instance::attach|TEMPER_FORGEJO|RUNNER_VERSION|::VERSION|names_runner" tools/git-transcripts/src/lib.rs crates/temper/tests/{process.rs,support/owned.rs,support/forgejo.rs} testing/temper-throwaway-forgejo tools/forgejo-conformance/src'
# directory: /srv/data/git/runner/src/tmp/planning/temper-base
rg: crates/temper/tests/{process.rs,support/owned.rs,support/forgejo.rs}: No such file or directory
exit 2: sh did not expand Bash braces; corrected below.
```
```
$ bash -c 'rg -n "Forgejo::from_env|Instance::attach|TEMPER_FORGEJO|RUNNER_VERSION|::VERSION|names_runner" tools/git-transcripts/src/lib.rs crates/temper/tests/{process.rs,support/owned.rs,support/forgejo.rs} testing/temper-throwaway-forgejo tools/forgejo-conformance/src'
# directory: /srv/data/git/runner/src/tmp/planning/temper-base
28 matching lines; exit 0; full log 20261019T011836-734.log
git-transcripts/src/lib.rs:76: Forgejo::from_env
support/owned.rs:10,97,144: runner pin import/uses
support/owned.rs:402: Instance::attach
process.rs:197,230: throwaway::VERSION assertions
conformance/lib.rs:27 and component.rs:17: Forgejo::from_env
conformance/runner.rs:49,63,69,256: variable reads/reason, names_runner
conformance/deliveries.rs:264: runner variable
conformance/component/reads.rs:57,93,96: variable reads/reason
conformance/registration.rs:16,170 and tests.rs:156: RUNNER_VERSION
```
The designed runner re-export stays: RUNNER_VERSION and names_runner.
VERSION's test import moves to skein_forge::forgejo::pin (throwaway.rs:2).
Temper's VERSION import/uses are owned.rs:10,96,143,362 (scripted, finished,
dump) and process.rs:197,230 (the two named tests in Shape). Its next repin
repairs them, git-transcripts::run and the support/forgejo.rs -> owned::attach /
binaries path. owned.rs already reads its own variables; support/forgejo.rs
does not directly call from_env at this base. Attachment already read the id;
the behavior change removes its hard-coded login. Three planned tests exercise
login/id, child reaping with scratch retained, and cli refusal using a Rust
scripted child, without requiring a Forgejo installation. Count: 16 + 1 ignored.

The byte construction cannot use client::Response to recreate a reason phrase.
The Host captures the raw head (through the final header CRLF), while skein-http
alone decodes framing. Sent is captured from client sends and credentials are
masked before retention. Call's reserved Content-Length/Connection are written
by skein-http; Range would precede Content-Length if both occur. The base corpus
has no such request. The existing request test grows a head/sent golden using
transcripts/user: head **254 bytes**, sent **99 bytes**, HTTP/1.1 200 OK.

```
$ python3 -c 'from pathlib import Path; import re; b=Path("tools/forgejo-conformance"); ps=[p for d in ("transcripts","deliveries") for p in (b/d).rglob("*") if p.is_file()]; both=[str(p) for p in (b/"transcripts").rglob("request") if b"Range:" in p.read_bytes() and b"Content-Length:" in p.read_bytes()]; print("requests with Range and Content-Length:",len(both)); print("source marker occurrences:",sum(p.read_text().count("temper:key") for p in (b/"src").rglob("*.rs"))); print("corpus files naming temper:",sum(b"temper" in p.read_bytes().lower() for p in ps)); head=(b/"transcripts/user/response").read_bytes().split(b"\r\n\r\n")[0]+b"\r\n"; print("user head:",len(head),"bytes; status:",head.split(b"\r\n")[0]); print("user sent:",len((b/"transcripts/user/request").read_bytes()),"bytes"); root=Path("/srv/data/git/runner/src/tmp/planning/skein-base"); print("corrected corpus path:",(root/"crates/skein-forge/../../tests/forge/conformance").resolve()==root/"tests/forge/conformance")'
# directory: /srv/data/git/runner/src/tmp/planning/temper-base
requests with Range and Content-Length: 0
source marker occurrences: 13
corpus files naming temper: 49
user head: 254 bytes; status: b'HTTP/1.1 200 OK'
user sent: 99 bytes
corrected corpus path: True
```
```
$ sh -c 'rg -o "temper:key" tools/forgejo-conformance/transcripts tools/forgejo-conformance/deliveries | wc -l'
# directory: /srv/data/git/runner/src/tmp/planning/temper-base
29
```
The corrected marker inventory is 13 source occurrences and 29 in the corpus,
not 29 source occurrences. All 49 files containing the old owner's word are
re-recorded. The complete old-to-new list is now in 06.2. The corpus path is
two parents above crates/skein-forge, not three.

### Revision sizing

The following runs code-organization 6's git diff --no-index --numstat for each
source pair. The size-copy tree applies only crate path and owner-word renames;
it is a measurement of that stage, not an implemented HTTP/JSON replacement.
The five throwaway source files total 728; the 21 recorder source files total
4,161. tests.rs and all test/helper/data files are excluded.

```
$ python3 /srv/data/git/runner/src/tmp/planning/scratch-skein-06-revision/measure_moves.py
# directory: /srv/data/git/runner/src/tmp/planning/temper-base
testing/temper-throwaway-forgejo moved 723 new 5 removed 5
| src/binary.rs | 70 | 2 | 2 |
| src/cli.rs | 132 | 0 | 0 |
| src/http.rs | 121 | 0 | 0 |
| src/instance.rs | 266 | 2 | 2 |
| src/lib.rs | 134 | 1 | 1 |
tools/forgejo-conformance moved 4071 new 90 removed 90
| src/cases.rs | 343 | 6 | 6 |
| src/cli.rs | 265 | 8 | 8 |
| src/component/hosted.rs | 297 | 3 | 3 |
| src/component/reads.rs | 121 | 5 | 5 |
| src/component/writes.rs | 189 | 5 | 5 |
| src/component.rs | 70 | 16 | 16 |
| src/deliveries.rs | 470 | 8 | 8 |
| src/facts.rs | 565 | 3 | 3 |
| src/fixture.rs | 193 | 3 | 3 |
| src/hooks.rs | 193 | 1 | 1 |
| src/job_clone.rs | 36 | 1 | 1 |
| src/lib.rs | 56 | 2 | 2 |
| src/main.rs | 26 | 0 | 0 |
| src/mask.rs | 184 | 1 | 1 |
| src/orgs.rs | 155 | 1 | 1 |
| src/registration.rs | 175 | 2 | 2 |
| src/runner.rs | 345 | 11 | 11 |
| src/setup.rs | 36 | 1 | 1 |
| src/tokens.rs | 145 | 7 | 7 |
| src/users.rs | 123 | 5 | 5 |
| src/write.rs | 84 | 1 | 1 |
```
Columns in those per-file rows are path, unchanged/moved, added/new, deleted.
The scratch measurement script was:

```python
from pathlib import Path
import subprocess
base = Path('/srv/data/git/runner/src/tmp/planning/temper-base')
scratch = Path(__file__).parent
for rel in ['testing/temper-throwaway-forgejo', 'tools/forgejo-conformance']:
    for src in sorted((base / rel / 'src').rglob('*.rs')):
        if src.name == 'tests.rs': continue
        dst = scratch / 'size-copy' / src.relative_to(base)
        result = subprocess.run(['git', 'diff', '--no-index', '--numstat',
                                 str(src), str(dst)], capture_output=True, text=True)
        assert result.returncode in (0, 1)
        added, deleted = map(int, result.stdout.split()[:2]) if result.stdout else (0, 0)
        # Report these three numbers per pair and sum them by source group.
        moved = len(src.read_text().splitlines()) - deleted
```

Renames: temper_throwaway_forgejo -> skein_throwaway_forgejo;
temper_forge_forgejo:: -> skein_forge::forgejo::;
temper_engine_domain_forge_client::api -> skein_forge::api;
remaining client/component crate prefixes -> skein_forge_client /
skein_forge_connection; owner words temper/TEMPER -> kit/KIT.

06.1's revised estimate deducts http.rs's 121 replaced lines and further
instance/binary edits from the 723 unchanged lines: about 570 moved, 300 new.
06.2 similarly deducts the JSON/flag edits from 4,071 unchanged: about 3,750
moved, 300 new. Neither estimate adds replacement code on top of the entire
source count. **The full replacement numstat remains unsettled:** only signatures
were sketched, as requested; 300 new lines is a target, not a measured ceiling.
The recorder's JSON conversion may exceed it. The planner must resolve that
size/split before readiness; this revision does not claim a measured complete
implementation. This is the remaining part of review defect 9.

### Signature check in skein's detached base

```
$ git worktree add --detach /srv/data/git/runner/src/tmp/planning/scratch-skein-06-revision d9f3e39
# directory: /srv/data/git/runner/src/tmp/planning/skein-base
Preparing worktree (detached HEAD d9f3e39)
HEAD is now at d9f3e39 Answer the forge kit planner's shape questions
```
A temporary crate, forge-session-sketch, depends on this worktree's skein-json.
It compiles Instance::attach and value, conformance run/json, component record/run,
runner run/component/run_with, deliveries record and component::reads run/actions.
All added Option<&Path> positions and Document returns are checked. The future
kit types are opaque placeholders; no implementation or new test run is claimed.
Private functions are exposed only in the scratch crate to avoid dead code;
the ready check still compiles against the landed prerequisite crates.

```
$ cargo clippy -p forge-session-sketch --all-targets -- -D warnings
# directory: /srv/data/git/runner/src/tmp/planning/scratch-skein-06-revision
exit 101: scratch placeholder/lint errors (std instead of core,
unimplemented!, missing Debug, wildcard imports, uninhabited error type).
Log: 20261019T011835-733.log; no production source changed.
```
The scratch was corrected to explicit imports/core paths, Debug types,
an inhabited Failure placeholder and unreachable signature-only bodies;
no lint suppression was added. The same command then passed:

```
$ cargo clippy -p forge-session-sketch --all-targets -- -D warnings
# directory: /srv/data/git/runner/src/tmp/planning/scratch-skein-06-revision
Checking forge-session-sketch v0.0.0
Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.13s
```
### Mask split and remaining coordination

06.2's base mask already masks UUIDs; its neutral recording clears four stale
occurrences. 06.4 alone adds the unquoted-time recognizer and clears the nine
unquoted log times. The reference test now exercises both quote forms, distinct
timestamps, repeat identity, fractional widths and Z/+05:30 offsets. Quoted
times still drop fractions; unquoted times zero the fraction preserving its
width. Their shared map identifies the original timestamp without quote bytes,
so the same input gets the same ordinal but different rendered bytes. The
independent whole-corpus scan requires bad UUID/time counts 0/0.

README Done when 7 needs the exact checked stable-fact command; decisions 16
needs the 06.2 UUID / 06.4 unquoted-time split; later.md needs the five-race
determinism item. These are left to the planner, as instructed. Answers 30/31
are settled; none of the other pending design answers was silently decided.

### Revision final checks and cleanup

```text
$ python3 /srv/data/git/runner/src/tmp/planning/scratch-skein-06-revision/audit_plan.py
# directory: /srv/data/git/runner/src
Session: 391 lines; 4 increments; all nine fields present
Recorder proofs: 2; 06.3 process proofs: 0; comparison commands: 2
Correct corpus path; obsolete digest absent; balanced fences
Revision findings command output blocks: at most 30 lines
```

```
$ git worktree remove --force /srv/data/git/runner/src/tmp/planning/scratch-skein-06-revision
# directory: /srv/data/git/runner/src/tmp/planning/skein-base
exit 0; scratch and its signature/count fixtures removed
```

Both base worktrees' git status --short were empty. No commit or landing was
made. Persistent writes by this task are only skein/06-conformance.md and this
findings file. The full HTTP/JSON replacement size remains unmeasured, as above.


## Re-check revision, 2026-10-19 (skein-06b)

Read the whole planning section 5, README, code organization, decisions,
three designer answer files, both reviews, the session and these findings.
The rulings in reviews/skein-06b.txt govern this revision. No design answer
is pending in this session. Persistent writes are limited to this file and
skein/06-conformance.md; no commit or landing was made.

Graph discovery used search_graph and get_code_snippet on the supplied
skein/temper projects, and get_file_outline for skein-json's Document.
The graph missed the current throwaway and conformance; direct base reads
and rg supplied that coverage. No list/index/status tool was called.

### Items fixed

- Item 1: keep deliveries::record's existing refusal without a runner,
  at its existing lookup. Only the diagnostic becomes `--runner required
  for delivery CI`. The existing unchecked runner/component groups remain;
  delivery CI is never skipped. This does not add a new behavior or test.
- Item 3: attach's login lookup is explicitly forced by answer 30 and
  the HTTP work by skein-http. Delete the Range/Content-Length ordering
  sentence; the raw-head/sent golden and its existing test remain.
- Item 4: tests/support/mod.rs with `mod support;` and support::attached,
  matching skein-fake-peers' convention. No standalone tests/fixtures.rs.
- Item 5: delete the planner note after the stable-fact check.
- Smaller items: both 06.2 and 06.4 name temper's side. The recorder flags
  have no external temper caller; git-transcripts belongs to 06.1.
  Name the digest call in binary.rs::Forgejo::from_path (line 42 in this
  base, not the review's 177). Decision 3 replaces the io wrapper with
  skein_lib::sha256::digest and byte-wise `{byte:02x}` rendering. The old
  wrapper packs the same digest into big-endian u64 words, so the old
  `{word:016x}` and new rendering agree, including leading zero bytes.
- Earlier fixes retained: both stable-fact commands are byte-identical;
  the five timing facts still print separately; 06.2 records with the base
  mask and clears UUIDs; 06.4 alone repairs unquoted times and tests the
  corpus. There are still only two recordings. The runner pin re-exports,
  attach tests, byte capture, neutralization list, environment sites and
  two-parent corpus path remain. No increment is renumbered; README 7
  still refers to 06.4.

### Re-check sizing — item 2 remains unresolved

The scratch below contains physical copies of every production source file
at the proposed new paths. Beyond the earlier rename-only copy, it removes
from_env/Unset and root VERSION; reads attach's login; changes the digest
call/rendering; and threads explicit binary/runner paths through main, lib,
component, runner, deliveries and component/reads. All old service values
and crate paths are renamed. Tests, manifests and data are excluded.

**This is still a partial measurement.** Neither the HTTP/JSON replacements
nor the recording script was implemented for counting. Thus the numbers
below are not full increment Sizes, and do not resolve the ruling. The
session now says this explicitly instead of describing 300 new lines as a
ceiling or target. I did not establish a complete split within five
increments. This session is not ready until item 2 is completed; no
unmeasured work is claimed to fit, and no new dependency or renumbering is
invented to conceal the missing count. The former 570/300 and 3,750/300
estimates are superseded, not validated by this partial copy.

```
$ git worktree add --detach /srv/data/git/runner/src/tmp/planning/scratch-skein-06b 25d2551
# directory: /srv/data/git/runner/src/tmp/planning/skein-base
Preparing worktree (detached HEAD 25d2551)
HEAD is now at 25d2551 Have the forge component count a call's cost
```

Each source pair uses `git diff --no-index --numstat`, as code-organization
6 requires; moved = source lines minus deleted, new = added. The counting
script (the same algorithm as Revision sizing, now over the edited copies):

```python
from pathlib import Path
import subprocess, sys
base = Path('/srv/data/git/runner/src/tmp/planning/temper-base')
scratch = Path(__file__).parent / 'sizing'
pairs = {'throwaway': ('testing/temper-throwaway-forgejo', 'testing/skein-throwaway-forgejo'), 'conformance': ('tools/forgejo-conformance', 'tests/forge/conformance')}
old, new = pairs[sys.argv[1]]
rows = []
for source in sorted((base / old / 'src').rglob('*.rs')):
    if source.name == 'tests.rs': continue
    relative = source.relative_to(base / old)
    target = scratch / new / relative
    diff = subprocess.run(['git', 'diff', '--no-index', '--numstat', str(source), str(target)], capture_output=True, text=True)
    assert diff.returncode in (0, 1), diff.stderr
    added, deleted = map(int, diff.stdout.split()[:2]) if diff.stdout else (0, 0)
    moved = len(source.read_text().splitlines()) - deleted
    rows.append((str(relative), moved, added, deleted))
print('path | moved | new | deleted')
for path, moved, added, deleted in rows: print(f'{path} | {moved} | {added} | {deleted}')
print('TOTAL', *(sum(row[i] for row in rows) for i in (1, 2, 3)))
print('Partial: HTTP/JSON replacements and recording script not included; no full Size established.')
```

```
$ python3 /srv/data/git/runner/src/tmp/planning/scratch-skein-06b/measure_moves.py throwaway
# directory: /srv/data/git/runner/src/tmp/planning/temper-base
path | moved | new | deleted
src/binary.rs | 62 | 3 | 10
src/cli.rs | 132 | 0 | 0
src/http.rs | 121 | 0 | 0
src/instance.rs | 266 | 3 | 2
src/lib.rs | 134 | 1 | 1
TOTAL 715 7 13
Partial: HTTP/JSON replacements and recording script not included; no full Size established.
```

```
$ python3 /srv/data/git/runner/src/tmp/planning/scratch-skein-06b/measure_moves.py conformance
# directory: /srv/data/git/runner/src/tmp/planning/temper-base
path | moved | new | deleted
src/cases.rs | 343 | 6 | 6
src/cli.rs | 265 | 8 | 8
src/component/hosted.rs | 297 | 3 | 3
src/component/reads.rs | 118 | 8 | 8
src/component/writes.rs | 189 | 5 | 5
src/component.rs | 64 | 22 | 22
src/deliveries.rs | 469 | 9 | 9
src/facts.rs | 565 | 3 | 3
src/fixture.rs | 193 | 3 | 3
src/hooks.rs | 193 | 1 | 1
src/job_clone.rs | 36 | 1 | 1
src/lib.rs | 52 | 6 | 6
src/main.rs | 22 | 9 | 4
src/mask.rs | 184 | 1 | 1
src/orgs.rs | 155 | 1 | 1
src/registration.rs | 175 | 2 | 2
src/runner.rs | 339 | 17 | 17
src/setup.rs | 36 | 1 | 1
src/tokens.rs | 145 | 7 | 7
src/users.rs | 123 | 5 | 5
src/write.rs | 84 | 1 | 1
TOTAL 4047 119 114
Partial: HTTP/JSON replacements and recording script not included; no full Size established.
```

### Names and signature check

```
$ bash -c 'rg -n "pub fn digest|pub fn (attach|from_path)|digest::digest\(&bytes\)|TEMPER_FORGEJO_RUNNER missing" /srv/data/git/runner/src/tmp/planning/skein-base/crates/skein-lib/src/sha256.rs /srv/data/git/runner/src/tmp/planning/skein-base/crates/skein-io/src/digest.rs /srv/data/git/runner/src/tmp/planning/temper-base/testing/temper-throwaway-forgejo/src /srv/data/git/runner/src/tmp/planning/temper-base/tools/forgejo-conformance/src/deliveries.rs; rg --files /srv/data/git/runner/src/tmp/planning/skein-base/tests /srv/data/git/runner/src/tmp/planning/skein-base/testing | rg "/tests/support/mod.rs$"'
# directory: /srv/data/git/runner/src/tmp/planning/temper-base
7 matches; full log 20261019T014027-733.log
skein-lib/src/sha256.rs:90: pub fn digest(message: &[u8]) -> [u8; 32]
skein-io/src/digest.rs:22: pub fn digest(bytes: &[u8]) -> Digest
throwaway/src/instance.rs:85: pub fn attach(child, port, scratch, admin, settings)
throwaway/src/binary.rs:30: pub fn from_path(binary: &Path)
throwaway/src/binary.rs:42: for word in skein_io::digest::digest(&bytes).0
conformance/src/deliveries.rs:264: TEMPER_FORGEJO_RUNNER missing
skein-base/testing/skein-fake-peers/tests/support/mod.rs
```

Signature-only traits in scratch's forge-session-sketch check attach/value,
recorder run/json, component record/run, runner run/component/run_with,
deliveries record and component reads run/actions. Opaque prerequisite
structs stand for the future forge types; Document is this base's actual
skein-json type. This is not a compiled replacement implementation.

```
$ cargo clippy -p forge-session-sketch --all-targets -- -D warnings
# directory: /srv/data/git/runner/src/tmp/planning/scratch-skein-06b
exit 101: 8 disallowed-type errors (Vec/String)
The scratch crate inherited the step-code clippy.toml, not the host/test rules.
```

Copied skein-base's tests/clippy.toml into the temporary signature crate,
which models ordinary Rust tools, not a step crate. No lint attribute or
exception was added, and no production/test type signature was changed.

```
$ cargo clippy -p forge-session-sketch --all-targets -- -D warnings
# directory: /srv/data/git/runner/src/tmp/planning/scratch-skein-06b
Checking forge-session-sketch v0.0.0
Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.12s
```

### Document checks

```
$ python3 /srv/data/git/runner/src/tmp/planning/scratch-skein-06b/audit_plan.py
# directory: /srv/data/git/runner/src/tmp/planning/temper-base
exit 1: the audit's substring tests/fixtures.rs also matched the legitimate
src/forgejo/tests/fixtures.rs reference in 06.3.
```

The check was corrected to the exact backtick-delimited `tests/fixtures.rs`;
06.3's fixture reference remains unchanged. No new Proof or Done-when
command was added; their earlier base runs still apply. No new recording
or test run is claimed, and the five stable-fact comparisons were not rerun.


```
$ python3 /srv/data/git/runner/src/tmp/planning/scratch-skein-06b/audit_plan.py
# directory: /srv/data/git/runner/src/tmp/planning/temper-base
396 lines; four increments; all nine fields present
Stable-fact commands byte-identical; 06.4 unchanged; two recording proofs
Review items 1, 3, 4, 5 and smaller items represented
Review item 2 remains unresolved: both full move Sizes unmeasured
No new proof/count command added to the session; earlier base runs retained
```

For clarity, there are two stable-fact command occurrences in the session,
each printing five timing outcomes; neither command was rerun in this revision.

```
$ git worktree remove --force /srv/data/git/runner/src/tmp/planning/scratch-skein-06b
# directory: /srv/data/git/runner/src/tmp/planning/skein-base
exit 0; scratch source copies and signature crate removed
```

Both base worktrees' git status --short print nothing. The revision is
partial: item 2's full sizing and any required split are unfinished. No
commit or landing was made. Only the session and this findings file persist.
