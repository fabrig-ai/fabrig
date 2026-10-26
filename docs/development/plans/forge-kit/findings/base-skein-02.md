# skein 02: base checks and signature sketches

Base: temper `27995fb75`, skein `d120820`, 2026-10-19.
Base commands run in `/srv/data/git/runner/src/tmp/planning/temper-base`;
skein's destination crate does not exist there. Commands naming
`skein-forge-client` use the source package/path below as their base form.
These are measurements of the source, not a claim that new tests ran.

## Discovery

Used codebase-memory-mcp `search_graph` and `get_code_snippet` with
`srv-data-git-runner-src-temper`, and `search_graph` with
`srv-data-git-runner-src-skein`. No index-management call.
Every cited symbol was checked with rg in the base. Graph line numbers
lagged: effect_key and bind_deployment matched, but several tests and
bounds helpers moved relative to its results. The service-profile graph
query found nothing; rg found `limits::profile`, `CLIENT`, and
`the_small_profile_passes_each_domains_worst_case`.

Read development.md 760–1453, the whole forge-client design, README,
code-organization, decisions, all three assigned findings and both precedents.
The newly drafted skein 01 agrees that zero vocabulary tests leave the
client. Its unbounded counter lives in api.rs and private measure.rs;
02.2 names both shared paths.

## Base list and test run

```text
$ cargo nextest list -p temper-engine-domain-forge-client
directory: /srv/data/git/runner/src/tmp/planning/temper-base
Finished `test` profile [unoptimized + debuginfo] target(s) in 0.18s
temper-engine-domain-forge-client tests::effect_keys_are_stable_by_purpose_and_separate_deployments
temper-engine-domain-forge-client tests::projection_history_keys_keep_task_position_and_history_family
temper-engine-domain-forge-client tests::locate::a_locate_past_the_call_bound_is_refused_too_large
temper-engine-domain-forge-client tests::locate::a_located_repository_zero_is_an_invalid_answer
76 listed: root 31, keep 19, outbox 24, locate 2
```

The two purpose tests belong to temper 02.1. All other 74 tests move in
02.1: root 29, keep 19, outbox 24, locate 2. The locate tests drive a
Domain and observe its terminal; neither is a vocabulary-only unit.
02.2 and 02.3 move zero additional tests across repositories. All retained
base names follow (the two purpose names above stay with their owner;
other module paths are preserved by fixture extraction):

- `tests::a_fresh_read_starts_immediately_without_global_lifetime_silence`
- `tests::a_job_answers_step_name_counts_against_answer_bytes`
- `tests::a_rate_refusal_does_not_spend_a_fresh_reads_retry_attempt`
- `tests::a_zero_settle_window_is_refused`
- `tests::actual_cost_settles_failed_calls_and_priority_share`
- `tests::an_answer_for_another_pull_cannot_satisfy_a_fresh_read`
- `tests::an_answer_with_the_wrong_shape_is_typed_failure`
- `tests::arbitrary_item_since_tolerates_the_providers_coarse_inclusive_overlap`
- `tests::comparisons_must_return_the_requested_endpoints`
- `tests::confirmed_write_terminals_do_not_invent_update_commits_or_branch_success`
- `tests::deployment_config_and_decoded_key_frames_are_checked`
- `tests::diagnostics_can_overflow_without_changing_admission`
- `tests::effect_keys_are_stable_by_purpose_and_separate_deployments`
- `tests::entry_refusal_is_observable_even_without_diagnostics`
- `tests::file_reads_are_pinned_bounded_and_charged_to_the_fresh_lane`
- `tests::files_at_a_different_head_are_refused_and_deletions_survive`
- `tests::fresh_reads_retry_transient_failures_and_finish_once_at_the_bound`
- `tests::full_and_oversized_admission_answer_once_without_an_api_call`
- `tests::input_rows_are_checked_before_scanning_or_forwarding_them`
- `tests::job_logs_are_bounded_and_pinned_to_the_failing_attempt`
- `tests::keep::a_first_inbox_read_never_hides_an_edited_or_unknown_completed_own_echo`
- `tests::keep::a_full_batch_is_refused_before_any_partial_admission`
- `tests::keep::a_full_echo_ledger_blocks_new_writes_until_the_inbox_reads_past_it`
- `tests::keep::a_pending_review_submitted_later_is_delivered_despite_its_older_id`
- `tests::keep::a_rate_refused_terminal_after_departure_never_requeues_the_removed_resource`
- `tests::keep::a_repeated_listing_item_cannot_overflow_a_tiny_decisions_output_bound`
- `tests::keep::a_reserved_background_share_is_admitted_even_with_one_pending_slot_and_replenished_outbox`
- `tests::keep::a_restart_between_review_pages_never_repeats_delivered_rows_or_metadata`
- `tests::keep::comment_scan_progress_and_edited_old_versions_are_durable`
- `tests::keep::first_adoption_of_reviews_requires_namespace_and_authenticated_author`
- `tests::keep::first_adoption_suppresses_only_framed_historical_keys_from_our_writer`
- `tests::keep::inbox_overflow_never_partially_delivers_or_advances_a_page`
- `tests::keep::malformed_review_pages_are_rejected_before_any_delivery_or_cursor_mutation`
- `tests::keep::own_comment_echo_never_skips_an_intervening_unread_human_comment`
- `tests::keep::own_pushes_and_held_writer_hints_are_not_drift_but_a_later_external_tip_is`
- `tests::keep::owned_pull_keeps_ci_and_base_on_backoff_without_reading_forge_reviews`
- `tests::keep::time_paging_keeps_the_tie_replay_point_when_an_item_moves_during_the_pass`
- `tests::keep::watch_batches_are_atomic_and_leaving_an_outstanding_resource_consumes_its_terminal`
- `tests::keep::writer_moves_require_forward_ancestry_and_authenticated_identity`
- `tests::late_refund_cannot_credit_the_new_window_and_extra_cost_debits_it`
- `tests::limits_reject_incoherent_budgets_and_account_for_capacity`
- `tests::locate::a_locate_past_the_call_bound_is_refused_too_large`
- `tests::locate::a_located_repository_zero_is_an_invalid_answer`
- `tests::malformed_item_identity_order_or_kind_finishes_as_a_typed_failure`
- `tests::outbox::a_changed_monotonic_origin_uses_the_saved_absolute_expiry`
- `tests::outbox::a_closed_pull_found_after_an_uncertain_open_is_raced`
- `tests::outbox::a_copied_pending_key_cannot_establish_the_effects_outcome`
- `tests::outbox::a_created_branch_is_found_only_at_its_requested_commit`
- `tests::outbox::a_missing_comment_target_is_searched_until_its_deadline_then_held`
- `tests::outbox::a_rehanded_entry_with_a_started_attempt_finds_before_writing`
- `tests::outbox::a_retarget_between_precheck_and_merge_is_explicitly_a_raced_effect`
- `tests::outbox::a_retargeted_merge_fails_before_any_write`
- `tests::outbox::a_review_written_after_the_head_moves_is_classified_as_a_raced_effect`
- `tests::outbox::accepted_merge_requires_post_result_base_and_head_verification`
- `tests::outbox::accepted_update_cannot_be_reported_as_failed_after_read_faults`
- `tests::outbox::an_unavailable_write_retries_only_up_to_its_configured_bound`
- `tests::outbox::an_uncertain_comment_is_found_by_key_without_a_second_write`
- `tests::outbox::an_uncertain_set_waits_for_its_original_lifetime_without_neighbor_probes`
- `tests::outbox::comparison_overflow_leaves_update_uncertain_without_another_write`
- `tests::outbox::effect_admission_and_restoration_require_authenticated_writer_configuration`
- `tests::outbox::lane_order_and_cancellation_hold_while_a_capture_call_is_out`
- `tests::outbox::recovery_classes_follow_forgejo_guarantees`
- `tests::outbox::restart_keeps_the_original_deadline_and_holds_an_unrecoverable_write`
- `tests::outbox::review_recovery_rejects_revised_or_unknown_rows_before_classifying_the_head`
- `tests::outbox::review_recovery_requires_an_original_row_strictly_after_the_saved_boundary`
- `tests::outbox::revised_or_unknown_comments_do_not_settle_an_uncertain_creation`
- `tests::outbox::start_and_attempt_are_durable_before_the_first_write`
- `tests::outbox::update_recovery_proves_the_requested_base_is_an_ancestor`
- `tests::overlapping_rate_refusals_keep_the_later_reset`
- `tests::priority_and_parallel_limit_follow_the_budget_cases`
- `tests::projection_history_keys_keep_task_position_and_history_family`
- `tests::protection_nested_contexts_and_checks_provider_details_are_bounded`
- `tests::rate_refusal_stops_every_class_and_retries_with_a_distinct_call_token`
- `tests::reserve_is_available_to_fresh_reads_when_background_is_idle`
- `tests::reserve_keeps_a_share_for_keep_and_slow`
- `tests::zero_cost_refund_reopens_the_last_reservation`

```text
$ cargo nextest run -p temper-engine-domain-forge-client
directory: /srv/data/git/runner/src/tmp/planning/temper-base
Finished `test` profile [unoptimized + debuginfo] target(s) in 0.18s
Starting 76 tests across 1 binary
PASS [   0.005s] tests::keep::watch_batches_are_atomic_and_leaving_an_outstanding_resource_consumes_its_terminal
PASS [   0.002s] tests::locate::a_locate_past_the_call_bound_is_refused_too_large
PASS [   0.002s] tests::locate::a_located_repository_zero_is_an_invalid_answer
Summary [   0.027s] 76 tests run: 76 passed, 0 skipped
```

Each test: 0.002–0.005 s; all 76 per-test times sum to 0.196 s, the
74 client tests to 0.192 s. The survey's prior wall measurement was
0.051 s. No new test's time is claimed from this run. Planned totals:
02.1 = 74 + 7 = 81; 02.2 = 81 + 9 = 90; 02.3 = 90 + 20 = 110.
The 0.5 s per-package proof ceiling is not a slice of the whole-suite aim.

```text
$ cargo clippy -p temper-engine-domain-forge-client --all-targets -- -D warnings
directory: /srv/data/git/runner/src/tmp/planning/temper-base
Checking skein-lib v0.0.0 (https://github.com/forgetic/skein.git#8a51e10b)
Checking temper-engine-domain-forge-client v0.0.0
Finished `dev` profile [unoptimized + debuginfo] target(s) in 3.87s
```

## Signature sketches

```text
$ git worktree add --detach /srv/data/git/runner/src/tmp/planning/scratch-skein-02 27995fb75
directory: /srv/data/git/runner/src/temper
Preparing worktree (detached HEAD 27995fb75)
HEAD is now at 27995fb75 Point temper's forge documents to skein's forge kit
```

In the initial sketch, only scratch source changed: effect_key takes namespace/identity/max,
the purpose enum and its two tests are omitted there; the Domain has
restore_seen and bind_deployment's guard; Env is threaded through
keep::made, finish, finished, withdraw, branch_created, done,
merge_verified and their callers, including domain::step. The echo body
is a signature stub: these checks do not establish its timing behavior.
The three byte-counter signatures initially stayed unchanged. The later
answer 14 supersedes their names, as the follow-up below records.
No new cross-boundary record is introduced.

```text
$ cargo clippy -p temper-engine-domain-forge-client --all-targets -- -D warnings
directory: /srv/data/git/runner/src/tmp/planning/scratch-skein-02
error: non-binding `let` without a type annotation
 --> crates/temper-engine-domain-forge-client/src/keep.rs:1302:5
let _ = env;
error: could not compile temper-engine-domain-forge-client due to 1 previous error
```

Corrected the stub to `let _: &Env<Limits> = env;`; no lint exception.
The repeat is for that diagnosed error, not an additional unchanged check.

## Owner limits and scope

The moved unit fixture LIMITS has pending/calls 8/8, rate/reserve 4/2,
window 60s, op_bytes 256, answer_bytes 4096, rows 4, inbox 16,
resources/repositories 4/2, entries 4, read/write attempts 3/3,
backoff/backoff_max 1s/8s, lifetime/margin 10s/1s, poll/poll_max 5s/60s,
hinted 1s, slow 60s, settle 30s, facts 32.
The library has no shipped service profile. It cannot depend on the
owner's profile builder. Preserve these moved small fixtures, naming
boundary overrides (0, limit+1, pending saturation) beside their normal
value. No shipped limit changes in this session.

For provenance, the owner's builder is temper-service::limits::profile
(Profile::Small), with CLIENT: pending/calls 4/2, rate/reserve 100/0,
window 10s, op_bytes 512, answer_bytes 65536, rows 8, inbox 16,
resources/repositories 8/2, entries 8, attempts 3/3,
backoff/backoff_max 1s/4s, lifetime/margin 10s/1s, poll/poll_max 5s/30s,
hinted 1s, slow 40s, settle 30s, facts 16. The source profile admission
check below is ancillary evidence; it does not validate the future move.

No extra Needs at start. The initial shared vocabulary counter is supplied by
skein 01.1, already a prerequisite of temper 01.1; designer answer 14 adds
01.6 to 02.2's Needs, as recorded below. No marker dependency:
the client inspects framed keys and receives typed hints, not marker text.
The memory-saturation sum is first proved against the moved client in
skein 07.1, rather than asserted unchanged by this session.

## Completed follow-up commands

```text
$ cargo run -p temper-engine-domain-forge-client --example widths
directory: /srv/data/git/runner/src/tmp/planning/scratch-skein-02
Compiling skein-lib v0.0.0 (https://github.com/forgetic/skein.git#8a51e10b)
   Compiling temper-engine-domain-forge-client v0.0.0 (/srv/data/git/runner/src/tmp/planning/scratch-skein-02/crates/temper-engine-domain-forge-client)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.38s
     Running `target/debug/examples/widths`
Summary=96 Comment=72 Review=104 Status=128 Remark=56 File=48 Collaborator=16 Commit=32 Slice=16 Writer=16
```

This temporary diagnostic measured the base layouts, not the proposed fixed widths.
The designer subsequently chose the larger constants below. The example is not a
planned crate target and was removed after recording its output.

```text
$ cargo clippy -p temper-engine-domain-forge-client --all-targets -- -D warnings
directory: /srv/data/git/runner/src/tmp/planning/scratch-skein-02
error: use of a disallowed macro `std::println`
 --> crates/temper-engine-domain-forge-client/examples/widths.rs:3:5
error: use of `println!`
error: could not compile temper-engine-domain-forge-client (example "widths") due to 2 previous errors
```

The signature chain itself compiled; all-targets also checked the temporary
layout-reporting example. Removed that completed diagnostic target instead of
adding a lint exception. Answer 14 then changed the public counter paths.

```text
$ cargo clippy -p temper-engine-domain-forge-client --all-targets -- -D warnings
directory: /srv/data/git/runner/src/tmp/planning/scratch-skein-02
error: empty lines after doc comment
 --> crates/temper-engine-domain-forge-client/src/lib.rs:33:1
error: function `answer_bytes` is never used
 --> crates/temper-engine-domain-forge-client/src/bounds.rs:201:15
error: could not compile temper-engine-domain-forge-client due to 2 previous errors
```

Removed the obsolete wrapper's documentation and the now-unused private wrapper.
Final sketch adds public `measure::stored(&Stored) -> Option<u64>` and
`measure::effect(&Effect) -> Option<u64>`; their stub bodies delegate to base
accounting, so this is signature validation, not proof of fixed-width behavior.
Renamed the private flag to identity_used and sealed on Restore/Keep/Make/Remake,
not Restored, matching the designer's landed answer 17.

```text
$ cargo clippy -p temper-engine-domain-forge-client --all-targets -- -D warnings
directory: /srv/data/git/runner/src/tmp/planning/scratch-skein-02
Checking temper-engine-domain-forge-client v0.0.0 (/srv/data/git/runner/src/tmp/planning/scratch-skein-02/crates/temper-engine-domain-forge-client)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.51s
```

No final sketch warning or failure remains; only this crate was compiled.

```text
$ cargo nextest run -p temper-service -E 'test(the_small_profile_passes_each_domains_worst_case)'
directory: /srv/data/git/runner/src/tmp/planning/temper-base
Compiling temper-service v0.0.0 (/srv/data/git/runner/src/tmp/planning/temper-base/crates/temper-service)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 4.93s
info: NEXTEST_DOUBLE_SPAWN=0 set, disabling double-spawn for test processes
────────────
 Nextest run ID 6635186b-2ad0-4974-b063-886f9c345756 with nextest profile: default
    Starting 1 test across 1 binary (211 tests skipped)
        PASS [   0.005s] (1/1) temper-service tests::limits::the_small_profile_passes_each_domains_worst_case
────────────
     Summary [   0.006s] 1 test run: 1 passed, 211 skipped
```

The selected profile check passed in 0.005 s; 211 other service tests were not
selected. This is not filtering the client move's proof, which ran all 76 tests.

```text
$ sh -c 'wc -l crates/temper-engine-domain-forge-client/src/*.rs crates/temper-engine-domain-forge-client/src/tests/*.rs | awk '\''$1 > 800 && $2 != "total"'\'''
directory: /srv/data/git/runner/src/tmp/planning/temper-base
850 crates/temper-engine-domain-forge-client/src/api.rs
  1377 crates/temper-engine-domain-forge-client/src/keep.rs
  1079 crates/temper-engine-domain-forge-client/src/outbox.rs
```

Base form of the final file-size check: the future keep/outbox and fixture
subdirectories do not exist yet, so only the base source and test globs run.
api splits in 01.1; keep/outbox split in 02.2. These are expected base exceptions.

```text
$ rg -n -i 'temper|jig|EffectPurpose' crates/temper-engine-domain-forge-client
directory: /srv/data/git/runner/src/tmp/planning/temper-base
crates/temper-engine-domain-forge-client/Cargo.toml:2:name = "temper-engine-domain-forge-client"
crates/temper-engine-domain-forge-client/src/boundary.rs:1://! Values exchanged with the connector top (jig's domain/connectors.md, sections 2–4).
crates/temper-engine-domain-forge-client/src/boundary.rs:30:/// (jig's domain/connectors.md, section 4.3).
crates/temper-engine-domain-forge-client/src/tests.rs:40:    let purpose = crate::EffectPurpose::Call { task: 7, attempt: 2, completion: 3, position: 4 };
crates/temper-engine-domain-forge-client/src/tests.rs:48:            &crate::EffectPurpose::Call { task: 7, attempt: 2, completion: 3, position: 5 },
crates/temper-engine-domain-forge-client/src/tests.rs:673:    let purpose = crate::EffectPurpose::ProjectionHistory { goal: 1, task: 2, position: 3, family: 1 };
crates/temper-engine-domain-forge-client/src/tests.rs:677:        crate::EffectPurpose::ProjectionHistory { goal: 2, task: 2, position: 3, family: 1 },
crates/temper-engine-domain-forge-client/src/tests.rs:678:        crate::EffectPurpose::ProjectionHistory { goal: 1, task: 3, position: 3, family: 1 },
crates/temper-engine-domain-forge-client/src/tests.rs:679:        crate::EffectPurpose::ProjectionHistory { goal: 1, task: 2, position: 4, family: 1 },
crates/temper-engine-domain-forge-client/src/tests.rs:680:        crate::EffectPurpose::ProjectionHistory { goal: 1, task: 2, position: 3, family: 2 },
crates/temper-engine-domain-forge-client/src/tests.rs:681:        crate::EffectPurpose::Projection { goal: 1, part: 1, number: 3 },
crates/temper-engine-domain-forge-client/src/identity.rs:11:/// (jig's domain/connectors.md, section 4.4).
crates/temper-engine-domain-forge-client/src/identity.rs:13:pub enum EffectPurpose {
crates/temper-engine-domain-forge-client/src/identity.rs:29:pub fn effect_key(deployment: &[u8], purpose: &EffectPurpose, max_bytes: u32) -> Option<Box<[u8]>> {
crates/temper-engine-domain-forge-client/src/identity.rs:31:        EffectPurpose::ProjectionHistory { .. } => 26,
crates/temper-engine-domain-forge-client/src/identity.rs:32:        EffectPurpose::Call { .. } => 25,
crates/temper-engine-domain-forge-client/src/identity.rs:33:        EffectPurpose::Step { .. } => 11,
crates/temper-engine-domain-forge-client/src/identity.rs:34:        EffectPurpose::Projection { .. } => 18,
crates/temper-engine-domain-forge-client/src/identity.rs:35:        EffectPurpose::Release { resource, .. } => 11_usize.checked_add(resource.len())?,
crates/temper-engine-domain-forge-client/src/identity.rs:50:        EffectPurpose::ProjectionHistory { goal, task, position, family } => {
crates/temper-engine-domain-forge-client/src/identity.rs:57:        EffectPurpose::Call { task, attempt, completion, position } => {
crates/temper-engine-domain-forge-client/src/identity.rs:69:        EffectPurpose::Step { task, purpose } => {
crates/temper-engine-domain-forge-client/src/identity.rs:75:        EffectPurpose::Projection { goal, part, number } => {
crates/temper-engine-domain-forge-client/src/identity.rs:85:        EffectPurpose::Release { task, resource } => {
crates/temper-engine-domain-forge-client/src/outbox.rs:2://! section 6; jig's domain/connectors.md, section 4.3).
crates/temper-engine-domain-forge-client/src/lib.rs:2://! (domain/forge.md, sections 5–6; jig's domain/connectors.md, section 4).
crates/temper-engine-domain-forge-client/src/lib.rs:31:pub use identity::{Config, EffectPurpose, Writer, effect_key};
```

The five jig doc lines, owner-named Cargo package, purpose enum/re-export and
purpose tests are the expected base hits. The destination must print nothing.

## User searches

The first survey counts matching lines, including aliases imported without `::`.
Its 18 Cargo occurrences are the root entries, package declaration and 15
consumers, not 18 dependent crates. The second survey adds every private
function/type affected by splitting; generic names also match unrelated code.
The zero crate-name/path matches outside temper disambiguate those homonyms.

```text
$ python3 -c 'from pathlib import Path
import re, subprocess, json
root=Path("/srv/data/git/runner/src")
source=root/"tmp/planning/temper-base/crates/temper-engine-domain-forge-client/src"
names=set()
for f in source.glob("*.rs"):
    if f.name in ("api.rs", "locate.rs", "tests.rs"): continue
    names.update(re.findall(r"pub(?:\(crate\))? (?:const )?(?:fn|struct|enum) (\w+)", f.read_text()))
names.update("accepts Shape finish finished branch_created done merge_verified withdraw nudge_resource".split())
def scan(repo, pattern, glob):
    r=subprocess.run(["rg","-n",pattern,"--glob",glob,str(repo)],capture_output=True,text=True)
    if r.returncode not in (0,1): raise RuntimeError(r.stderr)
    return r.stdout.splitlines()
repos=[root/"tmp/planning/temper-base",root/"tmp/planning/skein-base",root/"jig",root/"smith",root/"hearth"]
for repo in repos:
    print(repo.name, "crate-path",len(scan(repo,r"temper_engine_domain_forge_client::","*.rs")),"crate-name",len(scan(repo,r"temper_engine_domain_forge_client","*.rs")),"Cargo-users",len(scan(repo,r"temper-engine-domain-forge-client","Cargo.toml")))
    counts={n:len(scan(repo,r"\b"+n+r"\b","*.rs")) for n in sorted(names)}
    print("symbol counts",json.dumps(counts,sort_keys=True))
for n in ("effect_key","bind_deployment","stored_bytes","effect_bytes","answer_bytes","keep::made"):
    lines=scan(repos[0],r"\b"+n+r"\b","*.rs")
    external=[x for x in lines if "/temper-engine-domain-forge-client/" not in x]
    print(n,"external",len(external),"; ".join(external))
'
directory: /srv/data/git/runner/src/tmp/planning/temper-base
temper-base crate-path 157 crate-name 193 Cargo-users 18
skein-base crate-path 0 crate-name 0 Cargo-users 0
jig crate-path 0 crate-name 0 Cargo-users 0
smith crate-path 0 crate-name 0 Cargo-users 0
hearth crate-path 0 crate-name 0 Cargo-users 0
```

Full symbol-name search: 204 names, including the private split helpers, each
searched with rg in temper, skein, jig, smith and hearth. All graph-derived names
used by the session are present at the base. New names `identity_used`,
`measure::stored` and `measure::effect` are explicitly new; their signatures were
compiled above. The base's earlier `stored_bytes` and `effect_bytes` users are
included in these counts. Output numbers are matching source lines, not callers.

```text
$ python3 -c 'from pathlib import Path
import re, subprocess, json
root=Path("/srv/data/git/runner/src")
source=root/"tmp/planning/temper-base/crates/temper-engine-domain-forge-client/src"
names=set()
for f in source.glob("*.rs"):
    if f.name in ("locate.rs","tests.rs"): continue
    text=f.read_text()
    if f.name=="api.rs": text="\n".join(text.splitlines()[417:])
    names.update(re.findall(r"(?:fn|struct|enum) (\w+)",text))
repos=[root/"tmp/planning/temper-base",root/"tmp/planning/skein-base",root/"jig",root/"smith",root/"hearth"]
for repo in repos:
    counts={}
    for name in sorted(names):
        p=subprocess.run(["rg","-n",r"\b"+name+r"\b","--glob","*.rs",str(repo)],capture_output=True,text=True)
        assert p.returncode in (0,1),p.stderr
        counts[name]=len(p.stdout.splitlines())
    print(repo.name,json.dumps(counts,sort_keys=True))
'
directory: /srv/data/git/runner/src/tmp/planning/temper-base
temper-base {"Alarm": 50, "Attempt": 43, "Budget": 54, "Cached": 69, "Call": 359, "Calls": 42, "Condition": 153, "Config": 290, "Delivery": 793, "Domain": 788, "Echo": 52, "Effect": 251, "EffectPurpose": 26, "Entry": 280, "Event": 3396, "Fact": 222, "Fresh": 33, "Inbox": 68, "Keep": 66, "Key": 708, "Lane": 4, "Limits": 3099, "Listing": 62, "Live": 289, "LiveRecord": 60, "Made": 230, "Outbox": 15, "Outcome": 590, "Owner": 190, "Phase": 730, "Position": 65, "Priority": 74, "Progress": 53, "Recovery": 88, "RecoveryClock": 43, "Repo": 113, "RepositoryRecord": 39, "Request": 4969, "Resource": 157, "Scan": 48, "Shape": 117, "State": 746, "Stored": 697, "Watch": 111, "What": 967, "Writer": 1322, "Writing": 16, "accepts": 42, "active_repository": 2, "active_resource": 2, "admission": 94, "affected": 16, "answer": 1802, "answer_bytes": 118, "answer_bytes_unbounded": 7, "answer_heap": 4, "answered": 406, "answered_repository": 2, "answered_resource": 2, "background_owed": 3, "backoff": 64, "bind_deployment": 4, "branch_answered": 2, "branch_compared": 2, "branch_created": 2, "branch_found": 2, "branch_observed": 3, "busy": 88, "bytes": 4014, "cached": 91, "cancel": 56, "cancel_entry": 2, "cancel_repository": 2, "cancel_resource": 2, "check_phase": 3, "checks_at_head": 2, "comments_found": 2, "compared": 10, "configured": 316, "connected": 33, "construct": 3, "copied_stamps": 3, "done": 346, "due": 48, "due_repository": 2, "due_resource": 2, "echo": 38, "echo_room": 2, "effect_bytes": 6, "effect_key": 17, "eligible": 9, "entry_resource": 2, "fact": 124, "facts_lost": 16, "failed": 489, "failed_resource": 8, "file_matches": 2, "files": 720, "find": 433, "finish": 1243, "finished": 189, "fire": 168, "first": 979, "fresh_done": 3, "has": 385, "has_repo": 4, "has_room": 3, "hint": 124, "historical": 23, "is_due": 42, "is_echo": 5, "is_full": 11, "is_ready": 50, "is_set": 2, "item_answered": 2, "item_matches": 2, "items_ordered": 2, "key": 2176, "key_echo": 3, "keys": 149, "known_id": 3, "lane": 9, "limited": 21, "listed": 88, "live": 414, "live_for_entry": 4, "made": 347, "make": 180, "max_out": 193, "merge_verified": 3, "new": 6624, "next": 901, "next_deadline": 178, "next_id": 6, "not_found": 15, "nudge_resource": 3, "observed_head": 2, "op": 604, "operation": 327, "optional": 42, "original": 93, "pause": 14, "pop_fact": 22, "position": 332, "prepare_inbox": 2, "priority": 122, "prune_comments": 2, "pull": 919, "pull_answer": 7, "pulled": 2, "pump": 37, "pushed": 328, "queue": 151, "queued": 69, "read": 1660, "reclaim": 156, "recovery": 61, "remaining": 109, "remember": 14, "remove_echo": 2, "remove_live": 2, "remove_repos": 2, "replace": 165, "repos": 171, "repository": 2975, "reset": 62, "rest_resource": 5, "restoration_failed": 1, "restore_live": 2, "restore_repository": 2, "restored": 89, "resume": 102, "retain_uncertain": 2, "retry": 112, "reviews_answered": 2, "reviews_found": 2, "reviews_ordered": 2, "rows": 1566, "same_key": 4, "scan_busy": 4, "seeded": 18, "send": 590, "sent": 320, "settle": 380, "settle_done": 7, "shape": 49, "spend": 111, "start_fresh": 2, "start_settle": 2, "step": 1021, "stored_bytes": 7, "summary": 110, "unseen": 3, "valid": 147, "valid_deliveries": 3, "valid_participation": 2, "valid_resource": 7, "wait": 113, "wait_read": 2, "window": 175, "withdraw": 10, "worst_case": 612, "write": 1088, "writer": 2726}
skein-base {"Alarm": 0, "Attempt": 0, "Budget": 0, "Cached": 0, "Call": 381, "Calls": 11, "Condition": 0, "Config": 526, "Delivery": 2, "Domain": 84, "Echo": 92, "Effect": 30, "EffectPurpose": 0, "Entry": 172, "Event": 4098, "Fact": 65, "Fresh": 47, "Inbox": 28, "Keep": 307, "Key": 246, "Lane": 0, "Limits": 2301, "Listing": 13, "Live": 42, "LiveRecord": 0, "Made": 114, "Outbox": 0, "Outcome": 330, "Owner": 306, "Phase": 961, "Position": 126, "Priority": 0, "Progress": 65, "Recovery": 17, "RecoveryClock": 0, "Repo": 0, "RepositoryRecord": 0, "Request": 2653, "Resource": 3, "Scan": 126, "Shape": 67, "State": 1338, "Stored": 27, "Watch": 16, "What": 431, "Writer": 334, "Writing": 89, "accepts": 103, "active_repository": 0, "active_resource": 0, "admission": 111, "affected": 3, "answer": 1119, "answer_bytes": 16, "answer_bytes_unbounded": 0, "answer_heap": 0, "answered": 284, "answered_repository": 0, "answered_resource": 0, "background_owed": 0, "backoff": 16, "bind_deployment": 0, "branch_answered": 0, "branch_compared": 0, "branch_created": 0, "branch_found": 0, "branch_observed": 0, "busy": 57, "bytes": 5036, "cached": 4, "cancel": 372, "cancel_entry": 0, "cancel_repository": 0, "cancel_resource": 0, "check_phase": 0, "checks_at_head": 0, "comments_found": 0, "compared": 19, "configured": 222, "connected": 154, "construct": 1, "copied_stamps": 0, "done": 432, "due": 236, "due_repository": 0, "due_resource": 0, "echo": 244, "echo_room": 2, "effect_bytes": 0, "effect_key": 0, "eligible": 26, "entry_resource": 0, "fact": 6, "facts_lost": 0, "failed": 687, "failed_resource": 0, "file_matches": 0, "files": 546, "find": 458, "finish": 651, "finished": 153, "fire": 179, "first": 1288, "fresh_done": 0, "has": 631, "has_repo": 0, "has_room": 3, "hint": 25, "historical": 9, "is_due": 56, "is_echo": 0, "is_full": 20, "is_ready": 72, "is_set": 0, "item_answered": 0, "item_matches": 0, "items_ordered": 0, "key": 1237, "key_echo": 0, "keys": 161, "known_id": 0, "lane": 0, "limited": 5, "listed": 106, "live": 209, "live_for_entry": 0, "made": 438, "make": 133, "max_out": 7, "merge_verified": 0, "new": 5903, "next": 1306, "next_deadline": 283, "next_id": 21, "not_found": 0, "nudge_resource": 0, "observed_head": 0, "op": 1094, "operation": 349, "optional": 63, "original": 99, "pause": 18, "pop_fact": 0, "position": 310, "prepare_inbox": 0, "priority": 0, "prune_comments": 0, "pull": 23, "pull_answer": 0, "pulled": 1, "pump": 21, "pushed": 22, "queue": 319, "queued": 157, "read": 2553, "reclaim": 151, "recovery": 15, "remaining": 119, "remember": 2, "remove_echo": 0, "remove_live": 0, "remove_repos": 0, "replace": 136, "repos": 2, "repository": 6, "reset": 177, "rest_resource": 0, "restoration_failed": 0, "restore_live": 0, "restore_repository": 0, "restored": 59, "resume": 94, "retain_uncertain": 0, "retry": 266, "reviews_answered": 0, "reviews_found": 0, "reviews_ordered": 0, "rows": 135, "same_key": 0, "scan_busy": 0, "seeded": 30, "send": 603, "sent": 695, "settle": 282, "settle_done": 0, "shape": 114, "spend": 4, "start_fresh": 0, "start_settle": 0, "step": 627, "stored_bytes": 0, "summary": 41, "unseen": 1, "valid": 253, "valid_deliveries": 0, "valid_participation": 0, "valid_resource": 0, "wait": 149, "wait_read": 0, "window": 153, "withdraw": 62, "worst_case": 672, "write": 730, "writer": 1215}
jig {"Alarm": 5, "Attempt": 290, "Budget": 814, "Cached": 10, "Call": 732, "Calls": 23, "Condition": 11, "Config": 233, "Delivery": 1236, "Domain": 1474, "Echo": 1, "Effect": 1240, "EffectPurpose": 22, "Entry": 387, "Event": 9003, "Fact": 628, "Fresh": 30, "Inbox": 834, "Keep": 160, "Key": 1768, "Lane": 9, "Limits": 19363, "Listing": 0, "Live": 634, "LiveRecord": 0, "Made": 573, "Outbox": 443, "Outcome": 1721, "Owner": 1247, "Phase": 2048, "Position": 134, "Priority": 113, "Progress": 132, "Recovery": 274, "RecoveryClock": 0, "Repo": 0, "RepositoryRecord": 0, "Request": 6979, "Resource": 813, "Scan": 23, "Shape": 65, "State": 939, "Stored": 1801, "Watch": 510, "What": 534, "Writer": 9915, "Writing": 106, "accepts": 31, "active_repository": 0, "active_resource": 0, "admission": 825, "affected": 4, "answer": 3156, "answer_bytes": 25, "answer_bytes_unbounded": 0, "answer_heap": 0, "answered": 571, "answered_repository": 0, "answered_resource": 0, "background_owed": 0, "backoff": 89, "bind_deployment": 0, "branch_answered": 0, "branch_compared": 0, "branch_created": 0, "branch_found": 0, "branch_observed": 0, "busy": 308, "bytes": 19300, "cached": 80, "cancel": 211, "cancel_entry": 0, "cancel_repository": 0, "cancel_resource": 0, "check_phase": 0, "checks_at_head": 0, "comments_found": 0, "compared": 4, "configured": 507, "connected": 94, "construct": 2, "copied_stamps": 0, "done": 392, "due": 313, "due_repository": 0, "due_resource": 0, "echo": 4, "echo_room": 0, "effect_bytes": 0, "effect_key": 0, "eligible": 146, "entry_resource": 0, "fact": 946, "facts_lost": 13, "failed": 653, "failed_resource": 0, "file_matches": 0, "files": 325, "find": 1077, "finish": 8266, "finished": 108, "fire": 266, "first": 2499, "fresh_done": 0, "has": 1042, "has_repo": 0, "has_room": 6, "hint": 34, "historical": 199, "is_due": 28, "is_echo": 0, "is_full": 7, "is_ready": 34, "is_set": 0, "item_answered": 0, "item_matches": 0, "items_ordered": 0, "key": 5700, "key_echo": 0, "keys": 495, "known_id": 0, "lane": 6, "limited": 135, "listed": 350, "live": 1862, "live_for_entry": 0, "made": 679, "make": 408, "max_out": 444, "merge_verified": 0, "new": 25717, "next": 1617, "next_deadline": 228, "next_id": 0, "not_found": 4, "nudge_resource": 0, "observed_head": 0, "op": 64, "operation": 394, "optional": 276, "original": 772, "pause": 32, "pop_fact": 30, "position": 1529, "prepare_inbox": 0, "priority": 541, "prune_comments": 0, "pull": 1, "pull_answer": 0, "pulled": 0, "pump": 51, "pushed": 0, "queue": 171, "queued": 84, "read": 3196, "reclaim": 172, "recovery": 281, "remaining": 403, "remember": 8, "remove_echo": 0, "remove_live": 0, "remove_repos": 0, "replace": 204, "repos": 0, "repository": 0, "reset": 25, "rest_resource": 0, "restoration_failed": 0, "restore_live": 0, "restore_repository": 0, "restored": 337, "resume": 106, "retain_uncertain": 0, "retry": 666, "reviews_answered": 0, "reviews_found": 0, "reviews_ordered": 0, "rows": 6028, "same_key": 0, "scan_busy": 0, "seeded": 43, "send": 1211, "sent": 423, "settle": 275, "settle_done": 0, "shape": 153, "spend": 1183, "start_fresh": 0, "start_settle": 0, "step": 2188, "stored_bytes": 5, "summary": 116, "unseen": 12, "valid": 296, "valid_deliveries": 0, "valid_participation": 0, "valid_resource": 0, "wait": 191, "wait_read": 0, "window": 1059, "withdraw": 40, "worst_case": 1552, "write": 453, "writer": 19043}
smith {"Alarm": 60, "Attempt": 37, "Budget": 865, "Cached": 17, "Call": 651, "Calls": 337, "Condition": 0, "Config": 191, "Delivery": 631, "Domain": 506, "Echo": 2, "Effect": 230, "EffectPurpose": 0, "Entry": 126, "Event": 5231, "Fact": 522, "Fresh": 16, "Inbox": 20, "Keep": 41, "Key": 52, "Lane": 20, "Limits": 3323, "Listing": 29, "Live": 102, "LiveRecord": 0, "Made": 15, "Outbox": 0, "Outcome": 1186, "Owner": 23, "Phase": 675, "Position": 8, "Priority": 0, "Progress": 36, "Recovery": 0, "RecoveryClock": 0, "Repo": 57, "RepositoryRecord": 0, "Request": 2625, "Resource": 1, "Scan": 31, "Shape": 114, "State": 634, "Stored": 48, "Watch": 12, "What": 198, "Writer": 1088, "Writing": 4, "accepts": 58, "active_repository": 0, "active_resource": 0, "admission": 267, "affected": 1, "answer": 2186, "answer_bytes": 114, "answer_bytes_unbounded": 0, "answer_heap": 0, "answered": 752, "answered_repository": 0, "answered_resource": 0, "background_owed": 0, "backoff": 37, "bind_deployment": 0, "branch_answered": 0, "branch_compared": 0, "branch_created": 0, "branch_found": 0, "branch_observed": 0, "busy": 76, "bytes": 4723, "cached": 175, "cancel": 247, "cancel_entry": 0, "cancel_repository": 0, "cancel_resource": 0, "check_phase": 0, "checks_at_head": 0, "comments_found": 0, "compared": 11, "configured": 496, "connected": 3, "construct": 5, "copied_stamps": 0, "done": 472, "due": 65, "due_repository": 0, "due_resource": 0, "echo": 9, "echo_room": 0, "effect_bytes": 0, "effect_key": 0, "eligible": 39, "entry_resource": 0, "fact": 674, "facts_lost": 11, "failed": 625, "failed_resource": 0, "file_matches": 0, "files": 597, "find": 217, "finish": 1612, "finished": 57, "fire": 216, "first": 1445, "fresh_done": 0, "has": 681, "has_repo": 0, "has_room": 18, "hint": 2, "historical": 45, "is_due": 79, "is_echo": 0, "is_full": 12, "is_ready": 75, "is_set": 0, "item_answered": 0, "item_matches": 0, "items_ordered": 0, "key": 957, "key_echo": 0, "keys": 39, "known_id": 0, "lane": 6, "limited": 4, "listed": 82, "live": 465, "live_for_entry": 0, "made": 129, "make": 44, "max_out": 204, "merge_verified": 0, "new": 8988, "next": 877, "next_deadline": 211, "next_id": 0, "not_found": 10, "nudge_resource": 0, "observed_head": 0, "op": 403, "operation": 450, "optional": 61, "original": 363, "pause": 18, "pop_fact": 100, "position": 507, "prepare_inbox": 0, "priority": 0, "prune_comments": 0, "pull": 2, "pull_answer": 0, "pulled": 0, "pump": 5, "pushed": 47, "queue": 227, "queued": 129, "read": 1959, "reclaim": 221, "recovery": 54, "remaining": 148, "remember": 5, "remove_echo": 0, "remove_live": 0, "remove_repos": 0, "replace": 252, "repos": 105, "repository": 325, "reset": 4, "rest_resource": 0, "restoration_failed": 0, "restore_live": 0, "restore_repository": 0, "restored": 71, "resume": 235, "retain_uncertain": 0, "retry": 187, "reviews_answered": 0, "reviews_found": 0, "reviews_ordered": 0, "rows": 80, "same_key": 0, "scan_busy": 0, "seeded": 21, "send": 144, "sent": 361, "settle": 479, "settle_done": 0, "shape": 52, "spend": 727, "start_fresh": 0, "start_settle": 0, "step": 1449, "stored_bytes": 0, "summary": 629, "unseen": 1, "valid": 246, "valid_deliveries": 0, "valid_participation": 0, "valid_resource": 0, "wait": 504, "wait_read": 0, "window": 1039, "withdraw": 64, "worst_case": 668, "write": 810, "writer": 2217}
hearth {"Alarm": 0, "Attempt": 0, "Budget": 49, "Cached": 41, "Call": 92, "Calls": 0, "Condition": 22, "Config": 144, "Delivery": 6, "Domain": 2, "Echo": 0, "Effect": 0, "EffectPurpose": 0, "Entry": 357, "Event": 1079, "Fact": 0, "Fresh": 0, "Inbox": 0, "Keep": 57, "Key": 129, "Lane": 0, "Limits": 690, "Listing": 59, "Live": 140, "LiveRecord": 0, "Made": 177, "Outbox": 0, "Outcome": 397, "Owner": 46, "Phase": 42, "Position": 580, "Priority": 64, "Progress": 66, "Recovery": 0, "RecoveryClock": 0, "Repo": 0, "RepositoryRecord": 0, "Request": 869, "Resource": 0, "Scan": 20, "Shape": 299, "State": 198, "Stored": 13, "Watch": 0, "What": 9, "Writer": 93, "Writing": 20, "accepts": 3, "active_repository": 0, "active_resource": 0, "admission": 4, "affected": 0, "answer": 261, "answer_bytes": 0, "answer_bytes_unbounded": 0, "answer_heap": 0, "answered": 5, "answered_repository": 0, "answered_resource": 0, "background_owed": 0, "backoff": 0, "bind_deployment": 0, "branch_answered": 0, "branch_compared": 0, "branch_created": 0, "branch_found": 0, "branch_observed": 0, "busy": 8, "bytes": 2231, "cached": 33, "cancel": 4, "cancel_entry": 0, "cancel_repository": 0, "cancel_resource": 0, "check_phase": 0, "checks_at_head": 0, "comments_found": 0, "compared": 5, "configured": 12, "connected": 0, "construct": 2, "copied_stamps": 0, "done": 51, "due": 0, "due_repository": 0, "due_resource": 0, "echo": 0, "echo_room": 0, "effect_bytes": 0, "effect_key": 0, "eligible": 2, "entry_resource": 0, "fact": 0, "facts_lost": 0, "failed": 155, "failed_resource": 0, "file_matches": 0, "files": 528, "find": 152, "finish": 181, "finished": 52, "fire": 21, "first": 558, "fresh_done": 0, "has": 133, "has_repo": 0, "has_room": 2, "hint": 2, "historical": 0, "is_due": 5, "is_echo": 0, "is_full": 0, "is_ready": 3, "is_set": 0, "item_answered": 0, "item_matches": 0, "items_ordered": 0, "key": 1927, "key_echo": 0, "keys": 195, "known_id": 0, "lane": 0, "limited": 28, "listed": 157, "live": 157, "live_for_entry": 0, "made": 277, "make": 172, "max_out": 0, "merge_verified": 0, "new": 2337, "next": 429, "next_deadline": 41, "next_id": 0, "not_found": 0, "nudge_resource": 0, "observed_head": 0, "op": 257, "operation": 57, "optional": 150, "original": 15, "pause": 0, "pop_fact": 0, "position": 213, "prepare_inbox": 0, "priority": 17, "prune_comments": 0, "pull": 1, "pull_answer": 0, "pulled": 0, "pump": 0, "pushed": 1, "queue": 37, "queued": 2, "read": 573, "reclaim": 5, "recovery": 0, "remaining": 35, "remember": 0, "remove_echo": 0, "remove_live": 0, "remove_repos": 0, "replace": 70, "repos": 10, "repository": 1050, "reset": 2, "rest_resource": 0, "restoration_failed": 0, "restore_live": 0, "restore_repository": 0, "restored": 0, "resume": 64, "retain_uncertain": 0, "retry": 12, "reviews_answered": 0, "reviews_found": 0, "reviews_ordered": 0, "rows": 144, "same_key": 0, "scan_busy": 0, "seeded": 8, "send": 15, "sent": 45, "settle": 33, "settle_done": 0, "shape": 138, "spend": 0, "start_fresh": 0, "start_settle": 0, "step": 212, "stored_bytes": 0, "summary": 0, "unseen": 0, "valid": 159, "valid_deliveries": 0, "valid_participation": 0, "valid_resource": 0, "wait": 15, "wait_read": 0, "window": 1, "withdraw": 6, "worst_case": 153, "write": 182, "writer": 207}
```

The actual changed public-counter and key calls, including aliases:

```text
$ rg -n '(client|forge_client)::(effect_key|effect_bytes|stored_bytes|answer_bytes)\(' crates tests
directory: /srv/data/git/runner/src/tmp/planning/temper-base
crates/temper-engine-domain-forge/src/lib.rs:150:    match client::effect_bytes(&effect) {
crates/temper-engine-domain-forge/src/lib.rs:370:            client::effect_bytes(&row.entry.effect)?.checked_add(resource)
crates/temper-engine-domain-forge/src/lib.rs:372:        Stored::Entry { entry, .. } => client::effect_bytes(&entry.effect),
crates/temper-engine-domain-forge/src/lib.rs:373:        Stored::Client(row) => client::stored_bytes(row),
crates/temper-engine-domain/src/route/forge_route.rs:326:    forge_client::effect_key(
crates/temper-engine-domain-forge/src/domain.rs:178:                Ok(value) => match client::answer_bytes(value, &limits.client) {
crates/temper-engine-domain-forge/src/domain.rs:2675:                return client::effect_key(
crates/temper-engine-domain-forge/src/domain.rs:2682:                return client::effect_key(
crates/temper-engine-domain-forge/src/domain.rs:2697:    client::effect_key(namespace, &client::EffectPurpose::Projection { goal, part, number }, l.client.op_bytes)
```

Production calls: effect_key 4 (domain-forge/domain.rs 3, engine-domain's
route/forge_route.rs 1); effect_bytes 3, stored_bytes 1, answer_bytes 1,
all five counter calls in domain-forge/lib.rs or domain.rs. bind_deployment
has one client call in domain-forge/domain.rs, plus the owner's wrapper and
its engine restart call. No external caller reaches keep::made or the private
outbox helpers; their callers are within the client and were compiled together.

```text
$ sh -c 'rg -l "^temper-engine-domain-forge-client[.]" --glob Cargo.toml crates testing tests tools | sort; rg -l "^temper-engine-domain-forge-client[.]" --glob Cargo.toml crates testing tests tools | wc -l'
directory: /srv/data/git/runner/src/tmp/planning/temper-base
crates/temper-engine-domain-forge/Cargo.toml
crates/temper-engine-domain/Cargo.toml
crates/temper-engine-protocol-forge/Cargo.toml
crates/temper-engine-smith/Cargo.toml
crates/temper-protocol-store/Cargo.toml
crates/temper-service/Cargo.toml
tests/client/Cargo.toml
tests/codecs/Cargo.toml
tests/conformance/Cargo.toml
tests/engine/domain/Cargo.toml
tests/engine/forge/Cargo.toml
tests/engine/forge/client/Cargo.toml
tests/engine/protocol-forge/Cargo.toml
tests/system/Cargo.toml
tools/forgejo-conformance/Cargo.toml
15
```

The 15 consumer manifests and their Rust path changes belong to temper 02,
not to this skein session. The base code contains 193 crate-name lines;
157 have the exact `temper_engine_domain_forge_client::` prefix requested.

## Landed answers and remaining coordination

During drafting, decisions.md recorded the designer's shape answers in skein
`d9f3e39` and temper `b4cfb999`. Read them with git show and read

Question 14 now defines these fixed list widths (strings additionally count
their bytes; a non-list value adds no width): Summary/Review 128, Status 160,
Comment 96, Remark/File 64, Commit 32, Collaborator/list string 16, list id 8.
Each is asserted >= its Rust size when built. Stored stamps/echoes count 16,
labels 16+length, reviewers 8. This supersedes freezing native layout widths.

02.2 now depends on **skein 01.6's measure::op/answer**, which the original
README row did not name. That step owns the vocabulary implementation;
02.2 owns client row admission and measure::stored/effect, removing the old
public wrappers. **temper 02's counter-path cutover must wait for 02.2**;
its README row still says only skein 02.1. The planner must align that row
and session; neither is edited here. This is a coordination correction from
answer 14, not a new design decision by this drafter.

Question 17 now seals binding on any Restore, Keep, Make or Remake taken,
including a refused one. Restored without those events is not a seal. The
final sketch and test shape follow that answer. Questions 15, 18, 20, 21,
22 remain marked pending; no new design question was decided here.

The base's valid_deliveries rejects duplicate ids and capacity overflow,
not decreasing ids alone. The restore test table asks for the former two;
it does not invent an ordering constraint on the saved stamp set.

```text
$ git worktree remove --force /srv/data/git/runner/src/tmp/planning/scratch-skein-02
directory: /srv/data/git/runner/src/temper
exit 0; no output
```

Only the requested session and this findings file persist from this task.
No commit or landing was made; both base worktrees remained clean.


## Revision after review: decision 24 (2026-10-19)

This appendix supersedes the earlier step numbers and the "remaining
coordination" paragraph above. There is no skein 01.6. Four increments:
02.1 moves the client; 02.2 owns both crates' measure; 02.3 splits the
large files and applies the identity/Busy rules; 02.4 adds restart coverage.
README alignment is already the planner's ruling, not deferred work here.
The next temper repin repairs the explicit "temper's side" calls.

Read the full assigned development section, README, code organization,
decisions, designer answers, session and findings; checked forge-client
2–10 and forge 2.1/3.7 against the amended draft. Discovery used
codebase-memory-mcp search_graph, get_file_outline and get_code_snippet
with project srv-data-git-runner-src-temper. The graph puts the unbounded
counter in lib/bounds; the base and skein 01.1 confirm its move to
skein_forge::api. Verified the symbols and callers with rg at temper-base;
no index-management call. Both bases retain their specified commits.

The public row check is now
`skein_forge_client::measure::answer_rows(&Answer, u32) -> bool`.
It counts no bytes and checks every nested list. `keep_named_answer`
combines that boolean with the shared answer measure and its byte cap.
Skein-forge's borrowed `measure::{write,summary,pull}` helpers expose the
existing vocabulary-part walkers with fixed widths and no Limits/row
argument. They let effect/cache/stored measurements share the implementation
without cloning a Write into Op or a Summary/Pull into Answer. No new
cross-boundary record or type alias is introduced.

02.1 explicitly retains the client's private bytes/optional/rows/summary/
files/answer_heap/pull copies until 02.2, when all byte walkers and four
old public counters retire. 02.2 also repairs the two protocol-test copies
if skein 03.1 has already landed, so that removing the vocabulary function
does not leave skein's own consumers broken.

Counts: 74 moved + 7 new = 81; the measure adds 3 client tests = 84;
02.3 adds 6 = 90 (the same 81 + 9); restart adds 20 = 110. Extending
02.1's successful bind test is not another test. Twelve additional pure
measure tests belong to skein-forge, not the client's 110. Compile-time
width assertions are not runtime tests. Questions 15, 18, 20, 21 and 22
remain pending the designer; answers 14 and 17 remain settled.

The stored fixture totals are cumulative: branch kit 3; comment and echo
32 more = 35; review stamp 16 more = 51; label kit 19 more = 70;
cached Pull's kit/kit and two reviewers 22 more = 92. Merge's condition
base kit adds 3 despite its write having no owned strings. The admission
fixtures are Items 131, Item/comment 100, Pull 22 and Job 5. Both sides of
each byte cap are checked independently from the nested row ceiling.

### Confirmed downstream callers

```text
$ rg -n 'effect_bytes|client::stored_bytes|client::answer_bytes|answer_bytes_unbounded|bind_deployment' crates/temper-engine-domain-forge/src/lib.rs crates/temper-engine-domain-forge/src/domain.rs crates/temper-engine-domain/src/restart.rs crates/temper-engine-protocol-forge/src/tests/reads.rs crates/temper-engine-protocol-forge/src/tests/contents.rs
directory: /srv/data/git/runner/src/tmp/planning/temper-base
crates/temper-engine-protocol-forge/src/tests/contents.rs:259:            temper_engine_domain_forge_client::answer_bytes_unbounded(&answer),
crates/temper-engine-protocol-forge/src/tests/reads.rs:489:            temper_engine_domain_forge_client::answer_bytes_unbounded(&answer),
crates/temper-engine-domain-forge/src/domain.rs:178:                Ok(value) => match client::answer_bytes(value, &limits.client) {
crates/temper-engine-domain-forge/src/domain.rs:263:    pub fn bind_deployment(&mut self, id: [u8; 16], limits: &Limits) -> bool {
crates/temper-engine-domain-forge/src/domain.rs:264:        if !self.client.bind_deployment(id, &limits.client) {
crates/temper-engine-domain/src/restart.rs:397:            if !domain.forge.bind_deployment(deployment.id, &env.limits.forge) {
crates/temper-engine-domain-forge/src/lib.rs:150:    match client::effect_bytes(&effect) {
crates/temper-engine-domain-forge/src/lib.rs:277:                Ok(answer) => client::answer_bytes_unbounded(answer),
crates/temper-engine-domain-forge/src/lib.rs:370:            client::effect_bytes(&row.entry.effect)?.checked_add(resource)
crates/temper-engine-domain-forge/src/lib.rs:372:        Stored::Entry { entry, .. } => client::effect_bytes(&entry.effect),
crates/temper-engine-domain-forge/src/lib.rs:373:        Stored::Client(row) => client::stored_bytes(row),
```

The restart arm calls restart_refuse on false. Its Deployment must still
arrive before any client Restore/Keep/Make/Remake; repinning cannot repair
this by ignoring the new false. `rg` also confirmed all named bounds
helpers and the keep/outbox functions that move or gain Env.

### New test command at the base

```text
$ cargo nextest run -p temper-engine-domain-forge-client -E 'test(measure_)'
directory: /srv/data/git/runner/src/tmp/planning/temper-base
Finished `test` profile [unoptimized + debuginfo] target(s) in 0.09s
Starting 0 tests across 1 binary (76 tests skipped)
Summary [   0.000s] 0 tests run: 0 passed, 76 skipped
error: no tests to run
```

This is the new skein-forge measure proof's base form. None of the twelve
new test names exists at the base; this is not a passing proof of them,
and no no-tests override is prescribed. The destination command must
select and pass twelve after 02.2 implements them (development.md 5.4's
code-not-yet-present rule). The existing unfiltered 76-test baseline,
list, owner-profile check, clippy, name and file-size commands above remain
the base evidence for the unchanged commands. Their future package names
are still skein-forge-client; the measure's new functions compile below.

### Signature sketch in a skein-base scratch worktree

```text
$ git worktree add --detach /srv/data/git/runner/src/tmp/planning/scratch-skein-02-revision d9f3e39
directory: /srv/data/git/runner/src/tmp/planning/skein-base
exit 0; detached worktree created at d9f3e39
```

Unlike the original temper scratch, this scratch is based on skein-base.
Copied the base vocabulary and client as scaffolding, split vocabulary
from client answer validation, and pointed client vocabulary imports at
skein-forge. Added only signature stubs for the new measure interfaces and
opaque key, build-time assertions against the copied types, the
identity_used field/guard wiring, and the changed Env call chain through
made/finish/finished/withdraw/branch_created/done/merge_verified. Copied
purpose tests were omitted, as the plan says. No behavior is proved by
these stubs: stored/effect delegate to base accounting; other measures
and the row predicate are placeholders for signature checking only.

The first clippy found scratch-scaffolding lint omissions, fixed without
lint exceptions. Compact error output:

```text
$ cargo clippy -p skein-forge-client --all-targets -- -D warnings
directory: /srv/data/git/runner/src/tmp/planning/scratch-skein-02-revision
error: this function could have a #[must_use] attribute (locate bytes/accepts)
error: usage of wildcard import (measure)
error: assert without any message (ten width assertions)
error: could not compile skein-forge (lib) due to 13 previous errors
```

Added must_use, explicit imports and assertion messages; moved temporary
imports below module documentation. The next check exposed the remaining
vocabulary split imports, then passed after their correction:

```text
$ cargo clippy -p skein-forge-client --all-targets -- -D warnings
directory: /srv/data/git/runner/src/tmp/planning/scratch-skein-02-revision
error: unused imports: File, Permission, Protection, Review, Settings, Status
error[E0433]: cannot find type Time in this scope
error[E0425]: cannot find function accepts in module api (six test calls)
```

```text
$ cargo clippy -p skein-forge-client --all-targets -- -D warnings
directory: /srv/data/git/runner/src/tmp/planning/scratch-skein-02-revision
Checking skein-forge-client v0.0.0
Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.24s
```

The final sketch additionally names the borrowed write/Summary/Pull
functions and the identity flag, then checks both crates' all-targets:

```text
$ cargo clippy -p skein-forge --all-targets -- -D warnings
directory: /srv/data/git/runner/src/tmp/planning/scratch-skein-02-revision
Checking skein-forge v0.0.0
Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.04s
```

```text
$ cargo clippy -p skein-forge-client --all-targets -- -D warnings
directory: /srv/data/git/runner/src/tmp/planning/scratch-skein-02-revision
Checking skein-forge v0.0.0
Checking skein-forge-client v0.0.0
Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.72s
```

Each vocabulary width's assertion compiled. The client's const assertion
block compiled these checks with explanatory assertion messages:
STAMP_BYTES=16 >= size_of::<Delivery>() and size_of::<Echo>();
LABEL_BYTES=16 >= size_of::<Box<[u8]>>(); REVIEWER_BYTES=8 >= 8 and
size_of::<u64>(). These are build assertions, not architecture-dependent
expected values in runtime tests.

### Fixture sizes after relocation

The first fixture-count attempt passed a heredoc to `python3 -`.
The queued process could not read that input and was terminated (exit 143,
59 s, no output). Re-ran the same count with the program as a quoted `-c`
argument; it completed. This is a planning count, not a prescribed proof.

```text
$ python3 -c 'from pathlib import Path
import re
p=Path("crates/temper-engine-domain-forge-client/src")
for name in ["tests.rs","tests/keep.rs","tests/outbox.rs"]:
    t=(p/name).read_text(); fixtures=0; purpose=0
    for m in re.finditer(r"^(?:fn \w+|struct \w+|impl \w+)",t,re.M):
        start=m.start(); op=t.index("{",m.end()); level=1; end=op+1
        while level:
            level+=(t[end]=="{")-(t[end]=="}"); end+=1
        lines=t[start:end].count("\n")+1
        if t[:start].rstrip().endswith("#[test]"):
            if "effect_keys_are_stable_by_purpose" in m.group() or "projection_history_keys" in m.group(): purpose+=lines+1
        else: fixtures+=lines
    if name=="tests.rs": fixtures+=28
    print(name, "total",len(t.splitlines()),"fixture",fixtures,"purpose",purpose,"remaining",len(t.splitlines())-fixtures-purpose)
'
directory: /srv/data/git/runner/src/tmp/planning/temper-base
tests.rs total 749 fixture 150 purpose 34 remaining 565
tests/keep.rs total 784 fixture 209 purpose 0 remaining 575
tests/outbox.rs total 719 fixture 201 purpose 0 remaining 518
```

The session rounds those remaining files to 570/580/520 with module imports;
locate remains 31. Fixtures, with imports, are about 160/220/210.
The new contracts file is estimated at 450 after 02.3 and below 800 after
02.4, with new tests below 40 lines. Production split estimates remain
keep 380/90/240/500/165 and outbox 380/250/160/290. These are planned
post-move sizes, not claims that the splits have been implemented.

### Final verification and cleanup

```text
$ git worktree remove --force /srv/data/git/runner/src/tmp/planning/scratch-skein-02-revision
directory: /srv/data/git/runner/src/tmp/planning/skein-base
exit 0; no output
```

A local document check counted 399 lines, four increments with every required
field, twelve named dialect measure tests, each Proof's layer prefix, no
01.6 reference in the revised session, the public answer_rows call and
restart.rs:397, and all five pending designer marks. No implementation
behavior test is claimed from the scratch compile. Sizes are 4,300/120,
220/280, 1,650/90 and 0/120 (moved/new production lines).

```text
$ git status --porcelain
directory: /srv/data/git/runner/src/tmp/planning/skein-base
exit 0; no output
```

```text
$ git status --porcelain
directory: /srv/data/git/runner/src/tmp/planning/temper-base
exit 0; no output
```

The scratch path no longer exists. Only skein/02-client.md and this
findings file persist from this revision. No commit or landing was made.

## Second-answer revision (2026-10-19; skein 25d2551, temper dd12b8e)

This appendix supersedes all earlier pending-answer statements and test totals.
Read development.md 760–1453, README, code organization, decisions, the prior
review, this session and its findings, and all three designer answer files.
The current forge-client.md 2–10 supplies answers 7 and 14–22; all are settled.
The prior measure ownership, public rows check, width assertions, temporary
byte walkers, fixture splits and explicit temper repin repairs are retained.

Discovery used search_graph, get_code_snippet and search_code with
project srv-data-git-runner-src-temper. Every existing name used in the
revision was verified with rg in temper-base; the graph's line positions
differed slightly from the base. No index-management call. The bases'
absolute roots and HEADs were checked: skein-base 25d2551ecc41751d30cfaf6a5a393db183a52f45,
temper-base dd12b8ed904edced9e65af9fa62cc769c5a09905.

### What the base does, and what changes

- Answer 15: keep::admission checks count, readiness/failure, each watch's
  size/participation/duplicate in input order, then capacity. outbox::make
  checks readiness, writer, validity, capacity. domain::step checks Read's
  size before full slab/background share. Preserve these; the three named
  pair tests are count TooLarge/restore Busy, unready Busy/missing-writer
  Forbidden, and read TooLarge/full-slab Busy. The Make fixture is otherwise
  valid, so it isolates its pair rather than three simultaneous faults.
- Answer 16: outbox::make asserts duplicate entry and, for Make but not
  Remake, non-increasing held number. keep::restore_live and
  restore_repository assert Restore before Restored and duplicate rows.
  calls::answered expects a live flight token, so unknown and repeated
  answers panic. These are the Preconditions bullet's base assertions.
  Their existing admission-before-assert order remains. Repeated Restored,
  PauseOutbox, ReadAfresh, SettleOutbox and internal invariant assertions
  are not new refusals or new preconditions in this revision; leave them.
  Invalid restored content remains restoration_failed, not an owner panic.
- Answer 18: there is no production deadline defect in dd12b8e.
  calls::answered requeues a rate refusal before dispatching to outbox;
  send calls outbox::sent each time, and Phase::Make unconditionally replaces
  Attempt using both current clocks. Start stays fixed and no failure is
  spent. A scratch regression passed against this base before any repair:
  send 100/deadline 111, resend 102/deadline and expires 113, failures 0,
  distinct token, saved progress before the release. 02.4 adds this test,
  rather than changing already conforming behavior.
- Answer 19: the base already distinguishes provenance: listed has none
  (Summary has no such field), comments_found and reviews_found require
  Original, and review recovery keeps its saved-id and head checks.
  However, all three use writer + same_key without identity::historical.
  A matching entry/summary key framed for another namespace incorrectly
  settles an issue. The scratch probe below reproduced it (two outcomes,
  Uncertain plus Made, where only Uncertain should exist). 02.4 adds the
  existing historical predicate to these three matching-key paths, keeping
  authenticated writer and exact entry-key equality. It does not change
  Make's admission or add provenance to Summary.
- The new belief checks break recovery fixtures whose keys were unframed:
  tests/outbox.rs's comment, recovering_review, recovered_review and their
  corresponding answer literals. Frame those keys for the fixture namespace;
  moved test names and timing budgets stay. In temper's client world,
  simulation.rs's issue/post and fuzzy_simulation.rs's CreateIssue use raw
  keys; world.rs configures namespace world. The next repin repairs those
  keys, or 02.4 repairs skein 07.1's copies if already moved. The simulation's
  raw marker round-trip test does not perform recovery and is unchanged.
  The memory world's arbitrary keys measure retention, not a recovery answer.
- Answer 20: calls::answered currently passes Err(Busy) through. 02.3
  normalizes it to InvalidAnswer before the failure fact and owner dispatch;
  separate read and write tests pin the terminal and uncertain recovery.
- Answer 21: withdraw returns immediately for missing/attempted entries.
  The regression drains earlier facts before checking zero new facts/output,
  then answers the outstanding attempted write to prove it still finishes.
- Answer 22: failed is initialized false once and only set true;
  keep::admission checks it after count, before any mutations.
  Corrected rows and the first Restored do not clear it. Tests use legal
  event sequences; they never try a second Restored or a late Restore to
  simulate recovery. Recovery uses a new Domain. The plan adds no reset.
  Exists on a sent write already has an attempt and enters a find; the new
  test preserves that deadline. Non-write phase errors are unchanged.
  The every-table-full story belongs solely to skein 07.1.
- Answer 7: calls::answered already maps an oversized successful write
  answer to InvalidAnswer and passes Timeout through; outbox::failed
  treats both as a lost answer. The test deliberately uses an oversized
  Branches answer to Post: all valid successful write answers are scalar,
  so an oversized answer necessarily has another shape too. It asserts
  the observable failure fact, failures=1, Uncertain and find, not a
  fabricated successful write answer with strings.
- The earlier echo alarm defect and Env signature chain remain in 02.4.
  No additional public type/signature is introduced by these answers.

### Base and scratch commands

All excerpts below are selected command output, at most 30 output lines
per block. The unfiltered base run remains the move's baseline; scratch
probes are audit evidence, not a claim that the 114 planned tests exist.

```text
$ cargo nextest run -p temper-engine-domain-forge-client
directory: /srv/data/git/runner/src/tmp/planning/temper-base
Finished `test` profile [unoptimized + debuginfo] target(s) in 0.09s
Starting 76 tests across 1 binary
Summary [   0.023s] 76 tests run: 76 passed, 0 skipped
```

```text
$ git worktree add --detach /srv/data/git/runner/src/tmp/planning/scratch-skein-02-answers2 25d2551
directory: /srv/data/git/runner/src/tmp/planning/skein-base
Preparing worktree (detached HEAD 25d2551)
HEAD is now at 25d2551 Have the forge component count a call's cost
```

Copied the base client into this scratch as skein-forge-client, using its
skein-lib dependency from skein-base. For this audit scaffold the vocabulary
and purpose API stayed in their base locations; the prior appendix already
compiled the extraction, opaque key and measure signatures. Added only
two behavioral probes first, leaving production byte-for-byte at the base.
The issue probe used version 1, namespace bad, identity a, matching the
entry's key and configured writer 1, against namespace test.

```text
$ cargo nextest run -p skein-forge-client -E 'test(rate_refused_writes_save_a_new_attempt) | test(issue_find_requires_a_believed_key_without_provenance)'
directory: /srv/data/git/runner/src/tmp/planning/scratch-skein-02-answers2
Finished `test` profile [unoptimized + debuginfo] target(s) in 1.77s
Starting 2 tests across 1 binary (76 tests skipped)
PASS [   0.002s] tests::outbox::rate_refused_writes_save_a_new_attempt
FAIL [   0.002s] tests::outbox::issue_find_requires_a_believed_key_without_provenance
assertion `left == right` failed: a foreign namespace cannot settle
  left: 2
 right: 1
Summary [   0.006s] 2 tests run: 1 passed, 1 failed, 76 skipped
```

Then applied the three historical-key checks and framed the existing
comment/review recovery fixtures for the scaffold's namespace test.
Threaded Env through made, finish, finished, withdraw, branch_created,
done, merge_verified and all callers. made consumes Env only as a
signature stub here: this run does not prove the echo alarm behavior.
No other admission, attempt, withdrawal or restore production code changed.

```text
$ cargo nextest run -p skein-forge-client
directory: /srv/data/git/runner/src/tmp/planning/scratch-skein-02-answers2
Finished `test` profile [unoptimized + debuginfo] target(s) in 1.83s
Starting 78 tests across 1 binary
PASS [   0.002s] tests::outbox::issue_find_requires_a_believed_key_without_provenance
PASS [   0.003s] tests::outbox::rate_refused_writes_save_a_new_attempt
Summary [   0.024s] 78 tests run: 78 passed, 0 skipped
```

The 78 are the base's 76 (including its two purpose tests), plus two audit
probes. This is not the session's 74 moved + 40 new count.

```text
$ cargo clippy -p skein-forge-client --all-targets -- -D warnings
directory: /srv/data/git/runner/src/tmp/planning/scratch-skein-02-answers2
Checking skein-lib v0.0.0
Checking skein-forge-client v0.0.0
Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.68s
```

Added the new recovery.rs reference test verbatim and compiled all targets
again, checking its placement and access to super::LIMITS:

```text
$ cargo clippy -p skein-forge-client --all-targets -- -D warnings
directory: /srv/data/git/runner/src/tmp/planning/scratch-skein-02-answers2
Checking skein-forge-client v0.0.0
Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.56s
```

### Name and consumer verification

```text
$ rg -n 'fn (sent|listed|comments_found|reviews_found|withdraw|admission)|identity::writer|same_key\(|historical\(|restored once|restore before restored|one stored row|commit order|handed off once|exactly one terminal|Err\(error\) => Err\(error\)' crates/temper-engine-domain-forge-client/src/outbox.rs crates/temper-engine-domain-forge-client/src/keep.rs crates/temper-engine-domain-forge-client/src/calls.rs crates/temper-engine-domain-forge-client/src/identity.rs
directory: /srv/data/git/runner/src/tmp/planning/temper-base
identity.rs:161:pub(crate) fn historical(d: &Domain, key: &[u8]) -> bool {
calls.rs:268:let id = d.calls.flights.remove(&token).expect("exactly one terminal per call token");
calls.rs:292:Err(error) => Err(error),
outbox.rs:173:assert!(!d.outbox.entries.contains_key(&entry.number), "an outbox entry is handed off once");
outbox.rs:175:assert!(*number < entry.number, "new entries arrive in commit order");
outbox.rs:224:pub(crate) fn withdraw(d: &mut Domain, number: u64, out: &mut Queue<Request>) {
outbox.rs:325:pub(crate) fn sent(d: &mut Domain, env: &Env<Limits>, number: u64, out: &mut Queue<Request>) {
outbox.rs:532:) && same_key(item.key.as_deref(), expected)
outbox.rs:576:&& same_key(comment.key.as_deref(), expected)
outbox.rs:605:&& same_key(review.key.as_deref(), expected)
keep.rs:220:fn admission(k: &Keep, l: &Limits, watches: &[Watch]) -> Result<(), Error> {
keep.rs:375:assert!(!d.keep.ready, "restore before restored");
keep.rs:399:"one stored row per resource"
keep.rs:403:assert!(!d.keep.ready, "restore before restored");
keep.rs:411:"one stored row per repository"
```

The above excerpt omits the repeated crates/temper-engine-domain-forge-client/src/
prefix and indentation for readability.

```text
$ rg -n 'key:|fn issue|fn post|namespace:' tests/engine/forge/client/tests/simulation.rs tests/engine/forge/client/tests/fuzzy_simulation.rs tests/engine/forge/client/src/world.rs
directory: /srv/data/git/runner/src/tmp/planning/temper-base
tests/engine/forge/client/src/world.rs:712:            namespace: Box::from(&b"world"[..]),
tests/engine/forge/client/tests/fuzzy_simulation.rs:18:                        key: Box::new([u8::try_from(entry).expect("world entry bound")]),
tests/engine/forge/client/tests/simulation.rs:3:fn issue(entry: u64) -> Entry {
tests/engine/forge/client/tests/simulation.rs:10:                key: Box::from(&b"deployment:goal"[..]),
tests/engine/forge/client/tests/simulation.rs:21:fn post(entry: u64, number: u64) -> Entry {
tests/engine/forge/client/tests/simulation.rs:29:                key: Box::new([u8::try_from(entry).expect("test entry bound")]),
```

### Counts, command coverage and cleanup

No new Proof or Done-when command is prescribed: the existing package run,
list, clippy, names and file-size commands are unchanged. Their base forms
and the measure-filter absence are recorded above; new behavior and its
future test counts are checked when the code lands. This pass refreshed
the unfiltered base run and compiled the Env sketch at the requested base.

The document audit initially searched the bare word pending and rejected
the legitimate phrase "eight pending calls"; corrected its pattern to
pending-designer marks. The final check below passed. Four increments:
4,300/120, 220/280, 1,650/90 and 0/160 moved/new production lines.
02.4's tests get their own recovery.rs (~600 lines), keeping contracts.rs
(~450) within 800. New reference test is max_out_covers_floor_and_batch;
it is counted once, not in addition to its table row.

```text
$ python3 -c 'from pathlib import Path
import re
p=Path("docs/development/plans/forge-kit/skein/02-client.md")
s=p.read_text()
assert len(s.splitlines()) <= 400
assert not re.search(r"pending the designer|pending marks|01\.6",s,re.I)
chunks=re.split(r"(?m)^### (02\.[1-4]) ",s)
counts=[]
for i in range(1,len(chunks),2):
    step,body=chunks[i:i+2]; body=body.split("\n## Done when")[0]
    for field in ["Scope","Needs","Files","Shape","Test shape","Proof","Don'\''t","Size","Removes"]:
        assert re.search("^"+re.escape(field)+":",body,re.M),(step,field)
    proof=[x for x in body.splitlines() if x.startswith("Proof:")]
    assert all(re.match(r"Proof: (unit|world|joint|process|live),",x) for x in proof)
    tests=body.split("Test shape:",1)[1].split("Proof:",1)[0]
    names=[a or b for a,b in re.findall(r"^\| `([a-z][a-z0-9_]+)` \||^- `([a-z][a-z0-9_]+)`:",tests,re.M)]
    counts.append(len(names))
    print(step,len(body.splitlines()),"lines;",len(names),"new tests;",re.search(r"^Size:.*",body,re.M).group())
assert counts==[7,15,6,24],counts
print("400 lines; four increments; client totals 81/84/90/114; 12 dialect tests.")
print("Required fields, proof layers and counts pass; no pending-designer marks.")
'
directory: /srv/data/git/runner/src/tmp/planning/temper-forge-kit
02.1 68 lines; 7 new tests; Size: about 4,300 lines moved, 120 new; production only.
02.2 110 lines; 15 new tests; Size: about 220 lines moved, 280 new across both crates; tests separate.
02.3 70 lines; 6 new tests; Size: about 1,650 lines moved, 90 new; production only.
02.4 87 lines; 24 new tests; Size: about 0 lines moved, 160 new; production only.
400 lines; four increments; client totals 81/84/90/114; 12 dialect tests.
Required fields, proof layers and counts pass; no pending-designer marks.
```

```text
$ git worktree remove --force /srv/data/git/runner/src/tmp/planning/scratch-skein-02-answers2
directory: /srv/data/git/runner/src/tmp/planning/skein-base
exit 0; no output
```

No design answer remains unsettled. No commit or landing was made.

Final verification: skein-base and temper-base each returned no output
from `git status --porcelain`; the scratch path is absent. The session is
400 lines. Only the assigned session and this appended findings file
persist from this pass; the plan worktree's pre-existing STATUS.md change
was not touched.

## Re-check revision: skein-02b (2026-10-19)

This appendix supersedes earlier claims that skein 01.1 moved the unbounded
counter or supplied a private measure. Read the re-check and planner's
rulings, the assigned planning rules and inputs, and the designer's answers.
Discovery used search_graph and get_code_snippet with project
srv-data-git-runner-src-temper; the store snippet was unavailable and the
accepts snippet's positions were stale, so the named base files were checked
with rg and targeted reads. No index-management call.

1. Added store::call_answer_bytes to 02.2's temper side. Its forge arm at
   store.rs:690 switches to skein_forge::measure::answer. Its callers at
   decision.rs:723, decision.rs:775 and store.rs:485 need no call-site change.
2. 02.1 retains all four public counters and their bounds implementations,
   including answer_bytes_unbounded, as at the base. 02.2 creates the
   vocabulary's measure.rs and pub mod measure in lib.rs, then removes the
   client's old counters and byte walkers. The existing 220 moved / 280 new
   estimate already covered the walkers and fixed-width implementation;
   the four Sizes remain 4300/120, 220/280, 1650/90 and 0/160.
3. The review correctly identifies that the old recovery test cannot isolate
   the byte guard. Its prescribed replacement does not exist: api::accepts
   permits only Created(u64), Commented(u64), Reviewed(u64), Merged(Commit),
   Branch(BranchCreation) and Done for writes. Commit is [u8;32], BranchCreation
   is a fieldless enum; none owns a byte string or list. All measure Some(0),
   at the base and under forge.md 3.3/3.7, even with answer_bytes=0.
   No valid write answer can exceed an unsigned byte cap. Charging scalar
   widths or adding a payload would contradict the settled measure.

   Kept the test's name and recovery assertions, but added independent
   bounds::answer checks: Branches [kit] costs STRING_BYTES+3 = 19; true
   at a cap of 19, false at 18, rows=4 in both. The two fresh Post cases
   then exercise that malformed answer and Timeout, preserving failures=1,
   Uncertain, one find and no repeat write. The session explicitly limits
   the claim to the byte predicate plus recovery, not isolation of the
   write dispatch's size branch. The correctly shaped read cases of 02.2
   still isolate byte admission through step. The literal item 3 ruling
   remains unsatisfiable with this vocabulary; it is not an open design
   question. No new answer variant, altered measure or signature is proposed.

### Read-only name and caller checks

These are discovery commands, not new prescribed proofs. Output excerpts
below have at most 30 lines. Every name added to the session was checked
in the base; the measure's destination path is in the landed design.

```text
$ rg -n 'call_answer_bytes|answer_bytes_unbounded' crates/temper-engine-domain/src/store.rs crates/temper-engine-domain/src/decision.rs crates/temper-engine-domain-forge-client/src/lib.rs crates/temper-engine-domain-forge-client/src/bounds.rs
directory: /srv/data/git/runner/src/tmp/planning/temper-base
crates/temper-engine-domain-forge-client/src/bounds.rs:205:pub(crate) fn answer_bytes_unbounded(value: &Answer) -> Option<u64> {
crates/temper-engine-domain-forge-client/src/lib.rs:50:pub fn answer_bytes_unbounded(answer: &api::Answer) -> Option<u64> {
crates/temper-engine-domain-forge-client/src/lib.rs:51:    bounds::answer_bytes_unbounded(answer)
crates/temper-engine-domain/src/decision.rs:723:                && bytes_within(crate::store::call_answer_bytes(answer), limits.transcript_bytes)
crates/temper-engine-domain/src/decision.rs:775:        if !bytes_within(crate::store::call_answer_bytes(&row.answer), limits.transcript_bytes)
crates/temper-engine-domain/src/store.rs:485:                Some(answer) => call_answer_bytes(&answer)?,
crates/temper-engine-domain/src/store.rs:687:pub(crate) fn call_answer_bytes(answer: &CallAnswer) -> Option<u64> {
crates/temper-engine-domain/src/store.rs:690:            Ok(answer) => temper_engine_domain_forge_client::answer_bytes_unbounded(answer),
exit 0
```

```text
$ rg -n 'pub fn (answer_bytes|answer_bytes_unbounded|stored_bytes|effect_bytes)|fn (bytes|optional|rows|summary|files|answer_heap|pull|answer)\b|pub enum BranchCreation|pub type Commit' crates/temper-engine-domain-forge-client/src/lib.rs crates/temper-engine-domain-forge-client/src/bounds.rs crates/temper-engine-domain-forge-client/src/api.rs
directory: /srv/data/git/runner/src/tmp/planning/temper-base
crates/temper-engine-domain-forge-client/src/api.rs:12:pub type Commit = [u8; 32];
crates/temper-engine-domain-forge-client/src/api.rs:282:pub enum BranchCreation {
crates/temper-engine-domain-forge-client/src/bounds.rs:10:fn bytes(value: &[u8]) -> Option<u64> {
crates/temper-engine-domain-forge-client/src/bounds.rs:13:fn optional(value: Option<&[u8]>) -> Option<u64> {
crates/temper-engine-domain-forge-client/src/bounds.rs:19:fn rows(count: usize, width: usize, limit: u32) -> Option<u64> {
crates/temper-engine-domain-forge-client/src/bounds.rs:92:fn summary(value: &Summary, limit: u32) -> Option<u64> {
crates/temper-engine-domain-forge-client/src/bounds.rs:101:fn files(values: &[File], limit: u32) -> Option<u64> {
crates/temper-engine-domain-forge-client/src/bounds.rs:111:pub(crate) fn answer(value: &Answer, l: &Limits) -> bool {
crates/temper-engine-domain-forge-client/src/bounds.rs:117:fn answer_heap(value: &Answer, limit: u32) -> Option<u64> {
crates/temper-engine-domain-forge-client/src/bounds.rs:209:fn pull(value: &crate::api::Pull, limit: u32) -> Option<u64> {
crates/temper-engine-domain-forge-client/src/lib.rs:35:pub fn stored_bytes(row: &Stored) -> Option<u64> {
crates/temper-engine-domain-forge-client/src/lib.rs:40:pub fn effect_bytes(effect: &Effect) -> Option<u64> {
crates/temper-engine-domain-forge-client/src/lib.rs:45:pub fn answer_bytes(answer: &api::Answer, limits: &Limits) -> Option<u64> {
crates/temper-engine-domain-forge-client/src/lib.rs:50:pub fn answer_bytes_unbounded(answer: &api::Answer) -> Option<u64> {
exit 0
```

No Proof or Done-when command was added or changed, and no signature sketch
changed. The recorded base runs and scratch clippy checks above remain
their evidence; no build, test, command or scratch worktree was run or
created in this pass. A read-only Python document audit passed: 400 lines,
four complete increments, layer-prefixed proofs, new test counts 7/15/6/24
(12 of 02.2's 15 belong to the dialect), client totals 81/84/90/114,
no stale private-measure/01.6/pending-designer claim, and the new caller and
explicit size-test limitation present. An initial patch failed to match;
it changed no file, and the corrected patch applied. Only the session file
and this appended findings section were written; no commit or landing.

