# skein 07: base checks

Source-code base: temper `27995fb75`, skein `d120820`, 2026-10-19.
At 00:47:15 UTC the shared base worktrees advanced to design-only commits
`b4cfb9999` and `d9f3e39`; subsequent commands ran there with identical Rust,
manifests and lockfiles. See the verified diff at the end. The scratch compile
remained at the original temper commit.
Commands run in the clean base; test builds unoptimized.
The future skein packages do not exist at the base. Each command below
is its source-package form, not a claim that the destination already runs.

The draft-era assignments and answer statuses below are historical, superseded
by **Review corrections** and **Re-check corrections** at the end. Their
command outputs remain evidence. The current session is 305 lines: 07.1 moves
the client world, 07.2 splits simulation.rs and adds recovery, 07.3 moves the
setup world with decode.rs, and 07.4 adds setup stories. All design questions
are answered; decisions 22 and 23 assign the component worlds to skein 08.

```text
$ cargo nextest run -p temper-engine-forge-client-world
directory: /srv/data/git/runner/src/tmp/planning/temper-base
        PASS [   0.011s] (24/29) temper-engine-forge-client-world::simulation a_seed_replays_and_facts_change_no_decision
        PASS [   0.011s] (29/29) temper-engine-forge-client-world::simulation creations_and_comments_survive_faults_late_landings_and_restart
        PASS [   0.008s] (20/29) temper-engine-forge-client-world::simulation active_comment_history_has_a_bounded_slow_repair_cost_and_catches_a_lost_edit_hint
     Summary [   0.020s] 29 tests run: 29 passed, 0 skipped
exit: 0
```

The list output was recovered from the completed test log after the recorder
failed to parse its trailing log text; cargo succeeded and was not rerun.

```text
$ cargo nextest list -p temper-engine-forge-client-world --message-format json
directory: /srv/data/git/runner/src/tmp/planning/temper-base
job_answer: 1 selected
locate: 2 selected
memory: 1 selected
simulation: 25 selected
exit: 0
```

Selected names at the base (names stay on a move):

```text
job_answer::a_job_with_no_failed_step_translates_to_its_whole_logs_end
locate::a_locate_answers_the_forges_id_for_a_repository_by_name
locate::a_locate_of_a_repository_the_forge_lacks_answers_missing
memory::full_live_caches_inbox_stamps_echoes_outbox_and_every_entry_point_fit
simulation::a_conditional_merge_lands_once_within_its_attempt_lifetime
simulation::a_deleted_uncertain_branch_is_held_after_its_deadline
simulation::a_human_edit_of_an_old_own_marker_is_news_even_when_the_original_author_stays
simulation::a_late_opening_closed_by_another_hand_is_reported_as_drift
simulation::a_late_set_reaches_its_final_state
simulation::a_pre_adoption_human_edit_is_news_despite_our_retained_author_and_marker
simulation::a_seed_replays_and_facts_change_no_decision
simulation::active_comment_history_has_a_bounded_slow_repair_cost_and_catches_a_lost_edit_hint
simulation::adoption_reads_history_and_preserves_foreign_or_copied_marker_news
simulation::an_uncertain_branch_is_found_without_retry_after_restart
simulation::an_uncertain_comment_found_by_key_is_made_without_retry
simulation::an_uncertain_comment_missing_after_its_deadline_holds_the_task
simulation::an_uncertain_open_pull_is_found_without_retry
simulation::an_unrecoverable_creation_is_found_if_it_lands_within_its_lifetime
simulation::another_writer_taking_a_branch_name_is_reported_without_overwriting_it
simulation::bounded_job_bytes_translate_to_a_truncated_attempt_log
simulation::comment_page_recovery_and_old_comment_edits_reach_the_subscriber_once
simulation::creations_and_comments_survive_faults_late_landings_and_restart
simulation::every_live_fresh_read_finishes_once_under_faults_and_rate_resets
simulation::idle_cost_is_independent_of_ten_times_the_history
simulation::inactive_history_does_not_change_a_participating_objects_idle_calls
simulation::markers_have_the_protocol_spelling_and_arbitrary_bytes_round_trip
simulation::review_page_recovery_keeps_old_pending_submissions_without_duplicate_news
simulation::slow_repair_finds_a_lost_review_hint_without_item_time_revision_evidence
simulation::the_groundwork_reads_keep_the_fake_api_facts_in_the_client_projection
```

```text
$ cargo nextest run -p temper-engine-forge-client-world --profile fuzzy
directory: /srv/data/git/runner/src/tmp/planning/temper-base
        PASS [   0.018s] (1/2) temper-engine-forge-client-world::fuzzy_simulation unrecoverable_branch_creation_survives_cuts_before_and_after_its_late_copy
        PASS [   0.169s] (2/2) temper-engine-forge-client-world::fuzzy_simulation seeded_faults_restarts_and_keyed_effects
     Summary [   0.169s] 2 tests run: 2 passed, 0 skipped
exit: 0
```

## Discovery and scope defects

The configured graph was queried with `search_graph` and `search_code`, project
`srv-data-git-runner-src-temper`. `search_code` found the client world's manifest;
`search_graph` returned no symbols for the protocol-world path. Source names
were checked directly with `rg` at the base. No index-management call was made.

### S07-A: the protocol remainder is not a kit-only move (superseded)

Historical scope analysis: decisions 22 and 23 settled this question. The
destinations and recommendation in this subsection are superseded by the
closing Review corrections; no claim or design answer waits on S07-A.

At the base, `tests/engine/protocol-forge/tests` partitions as follows:

| Binary | Focused | Fuzzy | Destination in the brief |
|---|---:|---:|---|
| machine | 14 | 0 | skein 03.4 |
| hooks | 19 | 0 | skein 03.4 |
| fuzzy_machine | 0 | 4 | skein 03.4 |
| fuzzy_hooks | 0 | 1 | skein 03.4 |
| fake_protocol | 5 | 0 | skein 07.3 |
| protocol | 9 | 0 | claimed by skein 07.3, blocked |
| fuzzy_protocol | 0 | 1 | claimed by skein 07.3, blocked |

Thus the claimed remainder is **14 focused + 1 fuzzy**, not 47 + 6;
only **5 focused** are a plainly movable component/fake suite. The four
long fuzzy-machine cases belong to 03.4, not to this session.

`src/protocol.rs::Protocol` owns `temper_engine_domain_forge::Domain`,
`Map<top::Key, top::Stored>` and a connector outbox. Its `event` takes
`top::Event`, `adopt_as` takes `top::Role`, and `seen` returns `top::Request`.
Its driver sends `jig_domain_people::Outcome`; the tests build
`top::ChangeRow`, `change::Status`, `issues::GoalView` and owner branch names.
Those are policy and engine behavior, not one downstream limit type which
can become plain numbers. Removing them is a new client-only driver and
new stories, not a copy/path rename. The connector's neutral top is explicitly
later.md work. Copying these types into skein violates forge-client.md 9 and
the plan's upstream-only/no-system rule.

The original recommendation was to keep those 9 + 1 owner tests in
temper; move the 5 `fake_protocol` tests; schedule a separate, sized client +
component joint-world increment before declaring forge-connection.md 10 met.
Decisions 22 and 23 now assign that kit work to skein 08 and leave the owner
tests in temper. Session 07 claims only the client and setup worlds.

The claimed remainder has **no transcript reader**. At the base,
`machine::case_bytes` reads `tools/forgejo-conformance/transcripts`, and
`a_recorded_forgejo_push_signed_by_the_world_is_hinted` reads a delivery;
both belong to 03.4. Its reader must use `tests/forge/conformance` once
06.2 lands. Do not manufacture a duplicate parser or claim these moved in 07.3.

### Shared support and extra dependencies

The draft's 03.4 and 07.2/07.3 assignments in the next paragraph are
superseded: machine support belongs to skein 08.1, the five movable protocol
tests to 08.3, and the setup move to 07.3. Re-check corrections below record
08.1's chosen package, public modules and 07.3's complete dependency list.

`hooks::a_domain_push_reaches_the_component_as_a_signed_delivery_and_a_hint`
uses `joint::Joint`; moving all hooks in 03.4 also needs this helper.
`Joint` itself uses `hooks::Peer`, `Pipe`, `Cut`, `fake_limits`, `fake_config`
and the shared translation. Therefore 03.4 must carry `Joint` as shared
support, and 07.3 should consume that single implementation from the machine
world. This adds **skein 03.4** to 07.3's Needs and requires its helper exports
to be checked at ready time. The same dependency applies to 07.2's `Pipe`,
`Cut` and `fake_limits`. Neither extra dependency is in the README row.
The concurrently drafted `skein/03-connection.md`, read at 00:49 UTC,
expressly says not to copy `joint` in 03.4, yet moves all 19 hooks tests.
That is an unresolved shared-helper contradiction, not an existing export
07.3 may assume. Its resolution belongs with S07-A before either move is ready.

`tests/engine/forge/src/lib.rs` already shares the client translation with
`#[path = "../client/src/translate.rs"]`. The following temper deletion must
retarget this path before deleting the client world. `fake_config` is also
used by the fake protocol tests, system world, engine-domain world and
conformance world. They stay temper's; 05.2 makes the fake's own fixtures.
`decode_answer` needs `Decoded` and `drive`, not only its function body;
07.3 copies that closed helper set into its own `src/decode.rs`
(the draft's 07.2 assignment is superseded).
The protocol-world crate is also named in `tests/one-process/Cargo.toml`.
These are deletion-step consumers, not permission to edit temper from skein 07.

### Moves and later test additions

The client source is 2,777 Rust lines at the base, setup 867 (about 940 with
the decoder support), and the complete protocol source 3,110. These totals
include tests and fixtures; no shipped production implementation is added.
07.1 keeps the 858-line `tests/simulation.rs` whole; 07.2 splits it when it
extracts the shared recovery fixtures and stories. It also splits the
746-line `src/world.rs` before extending recovery. New story work is after
the corresponding move, as code-organization.md 7 requires. The draft's
assignment of the split to 07.4 is superseded.

The existing memory test is `full_live_caches_inbox_stamps_echoes_outbox_and_every_entry_point_fit`.
It uses `skein_world::domain::heap::{Counting, Meter}` and calls
`client::worst_case` for both shipped world limits and a smaller profile.
The required floor is no overrun (measured retained/peak bytes <= bound),
not an invented positive margin. It stays a single allocator binary in 07.1.
forge-client.md 10 places it among step tests; the supplied brief expressly
places this existing test with the world. Location is not a new test claim.
```text
$ cargo nextest list -p temper-engine-forge-client-world --profile fuzzy --message-format json
directory: /srv/data/git/runner/src/tmp/planning/temper-base
fuzzy_simulation: 2 selected
exit: 0
```

Selected names at the base (names stay on a move):

```text
fuzzy_simulation::seeded_faults_restarts_and_keyed_effects
fuzzy_simulation::unrecoverable_branch_creation_survives_cuts_before_and_after_its_late_copy
```

## Interface sketch check

Scratch made with `git worktree add --detach /srv/data/git/runner/src/tmp/planning/scratch-skein-07 27995fb75` (exit 0).
Only the new client-world method signatures were added, with `todo!()` bodies,
using the source `Entry` and `Resource` types; 07.1 only renames their imports.
No behavior implementation is claimed by this compile.

```text
$ cargo clippy -p temper-engine-forge-client-world --all-targets -- -D warnings
directory: /srv/data/git/runner/src/tmp/planning/scratch-skein-07
    Checking temper-engine-forge-client-world v0.0.0 (/srv/data/git/runner/src/tmp/planning/scratch-skein-07/tests/engine/forge/client)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.40s
exit: 0
```

```text
$ wc -l tests/engine/forge/client/src/lib.rs tests/engine/forge/client/src/referee.rs tests/engine/forge/client/src/translate.rs tests/engine/forge/client/src/world.rs tests/engine/forge/client/tests/fuzzy_simulation.rs tests/engine/forge/client/tests/job_answer.rs tests/engine/forge/client/tests/locate.rs tests/engine/forge/client/tests/memory.rs tests/engine/forge/client/tests/simulation.rs tests/forge-setup/reconcile/src/exchange.rs tests/forge-setup/reconcile/src/fixtures.rs tests/forge-setup/reconcile/src/lib.rs tests/forge-setup/reconcile/src/world.rs tests/forge-setup/reconcile/tests/fuzzy_reconcile.rs tests/forge-setup/reconcile/tests/reconcile.rs tests/engine/protocol-forge/src/fixtures.rs tests/engine/protocol-forge/src/hooks.rs tests/engine/protocol-forge/src/joint.rs tests/engine/protocol-forge/src/lib.rs tests/engine/protocol-forge/src/machine.rs tests/engine/protocol-forge/src/peer.rs tests/engine/protocol-forge/src/pipe.rs tests/engine/protocol-forge/src/protocol.rs tests/engine/protocol-forge/tests/fake_protocol.rs tests/engine/protocol-forge/tests/fuzzy_hooks.rs tests/engine/protocol-forge/tests/fuzzy_machine.rs tests/engine/protocol-forge/tests/fuzzy_protocol.rs tests/engine/protocol-forge/tests/hooks.rs tests/engine/protocol-forge/tests/machine.rs tests/engine/protocol-forge/tests/protocol.rs
directory: /srv/data/git/runner/src/tmp/planning/temper-base
  6754 total
exit: 0
```

Source-file line inventory (test and fixture lines included):

- 6 tests/engine/forge/client/src/lib.rs
- 111 tests/engine/forge/client/src/referee.rs
- 567 tests/engine/forge/client/src/translate.rs
- 746 tests/engine/forge/client/src/world.rs
- 74 tests/engine/forge/client/tests/fuzzy_simulation.rs
- 34 tests/engine/forge/client/tests/job_answer.rs
- 21 tests/engine/forge/client/tests/locate.rs
- 360 tests/engine/forge/client/tests/memory.rs
- 858 tests/engine/forge/client/tests/simulation.rs
- 181 tests/forge-setup/reconcile/src/exchange.rs
- 226 tests/forge-setup/reconcile/src/fixtures.rs
- 5 tests/forge-setup/reconcile/src/lib.rs
- 159 tests/forge-setup/reconcile/src/world.rs
- 21 tests/forge-setup/reconcile/tests/fuzzy_reconcile.rs
- 275 tests/forge-setup/reconcile/tests/reconcile.rs
- 252 tests/engine/protocol-forge/src/fixtures.rs
- 243 tests/engine/protocol-forge/src/hooks.rs
- 463 tests/engine/protocol-forge/src/joint.rs
- 8 tests/engine/protocol-forge/src/lib.rs
- 320 tests/engine/protocol-forge/src/machine.rs
- 35 tests/engine/protocol-forge/src/peer.rs
- 130 tests/engine/protocol-forge/src/pipe.rs
- 535 tests/engine/protocol-forge/src/protocol.rs
- 113 tests/engine/protocol-forge/tests/fake_protocol.rs
- 19 tests/engine/protocol-forge/tests/fuzzy_hooks.rs
- 43 tests/engine/protocol-forge/tests/fuzzy_machine.rs
- 24 tests/engine/protocol-forge/tests/fuzzy_protocol.rs
- 215 tests/engine/protocol-forge/tests/hooks.rs
- 262 tests/engine/protocol-forge/tests/machine.rs
- 448 tests/engine/protocol-forge/tests/protocol.rs

```text
$ cargo nextest run -p temper-forge-setup-reconcile-world
directory: /srv/data/git/runner/src/tmp/planning/temper-base
        PASS [   0.068s] (17/21) temper-forge-setup-reconcile-world::reconcile a_deleted_bot_is_made_again
        PASS [   0.054s] (15/21) temper-forge-setup-reconcile-world::reconcile a_team_edited_by_a_person_is_repaired
        PASS [   0.050s] (11/21) temper-forge-setup-reconcile-world::reconcile a_deleted_hook_is_made_again
     Summary [   0.075s] 21 tests run: 21 passed, 0 skipped
exit: 0
```

```text
$ rg -n -i temper tests/engine/forge/client tests/forge-setup/reconcile tests/engine/protocol-forge
directory: /srv/data/git/runner/src/tmp/planning/temper-base
tests/engine/forge/client/tests/simulation.rs:511:                temper_fake_forge_domain::api::Step {
tests/engine/forge/client/tests/simulation.rs:512:                    number: temper_fake_forge_domain::api::WORKFLOW_STEP,
tests/engine/forge/client/tests/simulation.rs:514:                    status: temper_fake_forge_domain::api::StepStatus::Failure,
tests/engine/forge/client/tests/simulation.rs:517:                temper_fake_forge_domain::api::Step {
tests/engine/forge/client/tests/simulation.rs:520:                    status: temper_fake_forge_domain::api::StepStatus::Failure,
tests/engine/forge/client/tests/simulation.rs:525:        &temper_engine_forge_client_world::LIMITS,
tests/engine/forge/client/tests/simulation.rs:831:    let body = temper_engine_forge_client_world::translate::keyed(b"\x01\x00\x05worldold", b"a human revised this");
tests/engine/forge/client/tests/simulation.rs:846:        temper_engine_forge_client_world::translate::keyed(b"\x01\x00\x05worldold", b"human edit before adoption");
tests/engine/protocol-forge/tests/protocol.rs:2:use temper_engine_domain_forge as top;
tests/engine/protocol-forge/tests/protocol.rs:3:use temper_engine_domain_forge_change as change;
tests/engine/protocol-forge/tests/protocol.rs:4:use temper_engine_domain_forge_client as client;
tests/engine/protocol-forge/tests/protocol.rs:5:use temper_engine_domain_forge_issues as issues;
240 matching lines in the full log (the terminal tail showed 200)

exit: 0
```

```text
$ cargo nextest list -p temper-forge-setup-reconcile-world --message-format json
directory: /srv/data/git/runner/src/tmp/planning/temper-base
reconcile: 21 selected
exit: 0
```

Selected names at the base (names stay on a move):

```text
reconcile::a_deleted_bot_is_made_again
reconcile::a_deleted_hook_is_made_again
reconcile::a_listing_longer_than_a_page_is_read_to_its_end
reconcile::a_managed_forge_missing_an_org_is_refused_naming_it
reconcile::a_person_under_the_bots_login_is_refused
reconcile::a_removed_team_member_is_added_again
reconcile::a_revoked_token_is_made_again
reconcile::a_second_run_writes_only_the_hooks_patches
reconcile::a_team_edited_by_a_person_is_repaired
reconcile::a_team_named_for_the_bot_is_made_with_write_on_every_repository
reconcile::a_token_file_holding_a_secret_the_forge_never_made_is_made_again
reconcile::a_token_file_holding_another_users_token_is_made_again
reconcile::a_token_file_that_answers_as_the_bot_is_kept_untouched
reconcile::an_empty_forge_is_given_its_bot_and_both_tokens
reconcile::an_org_hook_is_made_with_the_secret_the_events_json_and_active
reconcile::an_owned_forge_makes_a_missing_org_for_its_person
reconcile::no_secret_is_in_any_failure_debug_or_trace
reconcile::the_tokens_hold_the_scopes_of_setup_section_4
reconcile::the_webhook_secret_is_drawn_when_missing
reconcile::the_webhook_secret_is_kept_when_present
reconcile::two_hooks_with_the_listeners_url_keep_the_lowest_patched_and_the_rest_deleted
```

```text
$ rg -n '"tests/(engine/forge/client|forge-setup/reconcile|engine/protocol-forge)"' Cargo.toml
directory: /srv/data/git/runner/src/tmp/planning/temper-base
6:    "tests/forge-setup/reconcile",
13:    "tests/engine/protocol-forge",
45:    "tests/engine/forge/client",
3 matching lines
exit: 0
```

```text
$ cargo nextest run -p temper-forge-setup-reconcile-world --profile fuzzy
directory: /srv/data/git/runner/src/tmp/planning/temper-base
        PASS [   0.744s] (1/1) temper-forge-setup-reconcile-world::fuzzy_reconcile random_initial_forges_converge_and_a_second_run_writes_only_hook_patches
     Summary [   0.744s] 1 test run: 1 passed, 0 skipped
exit: 0
```

```text
$ cargo nextest run -p temper-engine-protocol-forge-world --test fake_protocol
directory: /srv/data/git/runner/src/tmp/planning/temper-base
        PASS [   0.011s] (1/5) temper-engine-protocol-forge-world::fake_protocol the_component_starts_and_lists_items_through_the_fakes_bytes
        PASS [   0.011s] (2/5) temper-engine-protocol-forge-world::fake_protocol opening_with_nothing_to_merge_writes_no_pull_request
        PASS [   0.017s] (3/5) temper-engine-protocol-forge-world::fake_protocol a_failed_jobs_log_end_comes_through_a_suffix_range
        PASS [   0.018s] (4/5) temper-engine-protocol-forge-world::fake_protocol a_failed_job_read_names_its_failed_step_and_reads_its_log
        PASS [   0.019s] (5/5) temper-engine-protocol-forge-world::fake_protocol pull_for_answers_the_open_pull_when_the_lookup_names_a_closed_one
     Summary [   0.019s] 5 tests run: 5 passed, 0 skipped
exit: 0
```

```text
$ cargo nextest list -p temper-forge-setup-reconcile-world --profile fuzzy --message-format json
directory: /srv/data/git/runner/src/tmp/planning/temper-base
fuzzy_reconcile: 1 selected
exit: 0
```

Selected names at the base (names stay on a move):

```text
fuzzy_reconcile::random_initial_forges_converge_and_a_second_run_writes_only_hook_patches
```

```text
$ cargo nextest run -p temper-engine-protocol-forge-world
directory: /srv/data/git/runner/src/tmp/planning/temper-base
        PASS [   0.131s] (46/47) temper-engine-protocol-forge-world::protocol a_failed_ci_head_is_repaired_then_checked_again
        PASS [   0.122s] (45/47) temper-engine-protocol-forge-world::protocol a_change_produced_opened_checked_queued_and_landed
        PASS [   0.102s] (47/47) temper-engine-protocol-forge-world::protocol idle_forge_call_cost_does_not_follow_old_issue_history
     Summary [   0.235s] 47 tests run: 47 passed, 0 skipped
exit: 0
```

```text
$ cargo nextest list -p temper-engine-protocol-forge-world --message-format json
directory: /srv/data/git/runner/src/tmp/planning/temper-base
fake_protocol: 5 selected
hooks: 19 selected
machine: 14 selected
protocol: 9 selected
exit: 0
```

Selected names at the base (names stay on a move):

```text
fake_protocol::a_failed_job_read_names_its_failed_step_and_reads_its_log
fake_protocol::a_failed_jobs_log_end_comes_through_a_suffix_range
fake_protocol::opening_with_nothing_to_merge_writes_no_pull_request
fake_protocol::pull_for_answers_the_open_pull_when_the_lookup_names_a_closed_one
fake_protocol::the_component_starts_and_lists_items_through_the_fakes_bytes
hooks::a_body_past_its_cap_is_answered_413_and_hints_nothing
hooks::a_connection_past_hooks_waits_until_one_closes
hooks::a_domain_push_reaches_the_component_as_a_signed_delivery_and_a_hint
hooks::a_failed_listener_is_told_to_the_root_as_lost
hooks::a_get_to_a_forges_path_is_answered_405
hooks::a_held_hint_stops_the_next_body_until_it_is_offered
hooks::a_hint_the_root_refused_is_held_and_offered_again
hooks::a_malformed_head_is_answered_4xx_and_closed
hooks::a_missing_signature_is_answered_403_without_reading_a_body
hooks::a_path_without_a_forge_is_answered_404
hooks::a_path_without_a_secret_is_answered_404
hooks::a_recorded_forgejo_push_signed_by_the_world_is_hinted
hooks::a_request_unfinished_at_its_deadline_is_closed_with_no_answer
hooks::a_signed_push_is_answered_204_before_its_hint_goes_up
hooks::a_verified_body_the_decoder_calls_malformed_is_answered_400
hooks::a_verified_event_of_no_row_is_answered_204_and_hints_nothing
hooks::a_wrong_signature_is_answered_403_and_hints_nothing
hooks::the_secret_appears_in_no_event_or_debug_text
hooks::two_deliveries_on_one_connection_are_both_answered_and_hinted
machine::a_body_past_body_bytes_is_carried_empty_without_its_key
machine::a_call_silent_past_its_deadline_is_a_timeout_and_aborts
machine::a_call_waiting_past_its_deadline_is_unavailable_at_no_cost
machine::a_connection_marked_close_finishes_its_call_and_the_next_connects
machine::a_path_dropped_on_404_is_resolved_again_by_id
machine::a_redirect_drops_the_path_and_answers_unavailable
machine::a_reset_after_a_written_request_is_timeout
machine::a_well_framed_malformed_document_is_invalid_answer
machine::a_write_body_is_read_before_its_scripted_response
machine::an_idle_connection_closes_and_the_next_call_connects
machine::an_unreachable_forge_is_checked_again_after_its_backoff
machine::another_user_stops_the_start_once_naming_the_forge
machine::settings_answered_404_stops_the_start_with_its_status
machine::the_start_reaches_ready_from_the_recorded_answers
protocol::a_busy_pass_lists_at_most_changed_items_over_the_page_plus_one
protocol::a_change_produced_opened_checked_queued_and_landed
protocol::a_created_issue_with_its_answer_lost_is_found_after_restart
protocol::a_failed_ci_head_is_repaired_then_checked_again
protocol::a_merge_with_its_answer_lost_is_found_after_restart_without_merging_twice
protocol::an_external_move_of_a_held_branch_is_reported_as_drift
protocol::honest_costs_equal_the_requests_the_fake_served
protocol::idle_forge_call_cost_does_not_follow_old_issue_history
protocol::the_api_token_reaches_no_record_request_or_fact
```

## Scratch cleanup

```text
$ git worktree remove --force /srv/data/git/runner/src/tmp/planning/scratch-skein-07
directory: /srv/data/git/runner/src/temper
exit: 0
```

Only this scratch worktree had code edits, solely to compile signatures.
The shared queue temporarily stalled on another session's Python stdin;
it later cleared. Queued time is not test runtime.
```text
$ cargo nextest run -p temper-engine-protocol-forge-world --profile fuzzy
directory: /srv/data/git/runner/src/tmp/planning/temper-base
        PASS [   0.313s] (1/6) temper-engine-protocol-forge-world::fuzzy_protocol stories_through_cut_streams_answer_each_call_once
        PASS [   0.667s] (2/6) temper-engine-protocol-forge-world::fuzzy_hooks a_delivery_split_at_every_point_is_answered_alike
        PASS [   3.791s] (3/6) temper-engine-protocol-forge-world::fuzzy_machine a_response_cut_at_each_byte_ends_its_call_once_seed_1
        PASS [   4.038s] (4/6) temper-engine-protocol-forge-world::fuzzy_machine a_response_cut_at_each_byte_ends_its_call_once
        PASS [   4.214s] (5/6) temper-engine-protocol-forge-world::fuzzy_machine a_response_cut_at_each_byte_ends_its_call_once_seed_3
        PASS [   4.892s] (6/6) temper-engine-protocol-forge-world::fuzzy_machine a_response_cut_at_each_byte_ends_its_call_once_seed_2
     Summary [   4.892s] 6 tests run: 6 passed, 0 skipped
exit: 0
```

```text
$ cargo nextest list -p temper-engine-protocol-forge-world --profile fuzzy --message-format json
directory: /srv/data/git/runner/src/tmp/planning/temper-base
fuzzy_hooks: 1 selected
fuzzy_machine: 4 selected
fuzzy_protocol: 1 selected
exit: 0
```

Selected names at the base (names stay on a move):

```text
fuzzy_hooks::a_delivery_split_at_every_point_is_answered_alike
fuzzy_machine::a_response_cut_at_each_byte_ends_its_call_once
fuzzy_machine::a_response_cut_at_each_byte_ends_its_call_once_seed_1
fuzzy_machine::a_response_cut_at_each_byte_ends_its_call_once_seed_2
fuzzy_machine::a_response_cut_at_each_byte_ends_its_call_once_seed_3
fuzzy_protocol::stories_through_cut_streams_answer_each_call_once
```


## Measured protocol partition

The measured counts remain evidence; this section's draft assignment to
03.4 and its recommended remainder are superseded by decisions 22 and 23
and the closing Review corrections.

The completed whole-source focused run (`20261019T005137-817.log`) gives
these sums of per-test PASS times, not wall-time claims for new suites:

| Binary | Tests | Sum of test seconds |
|---|---:|---:|
| fake_protocol | 5 | 0.109 |
| hooks | 19 | 0.256 |
| machine | 14 | 0.464 |
| protocol | 9 | 0.673 |

The claimed remainder (fake_protocol + protocol) has 14 focused tests;
its one fuzzy_protocol test took 0.313 s. The separate recommended five-test
subset took 0.019 s wall time. The other five fuzzy tests belong to 03.4.

All six required whole-source run commands and all six list commands exited 0.
The focused/fuzzy times in this repeat are client 0.020/0.169 s, setup
0.075/0.744 s, protocol 0.235/4.892 s. Earlier base-runs.md times are retained
as prior measurements, not replaced by narrower budgets.
## Users of moved and changed symbols

The batched `rg` search below includes every declaration in the three source
worlds, codec support, and the connector fixture module. It also searches
the proposed client-world method names. Counts include definitions; generic
names can include unrelated declarations, so import users are audited below.

```text
$ python3 -c 'from pathlib import Path
import re, subprocess
root=Path("/srv/data/git/runner/src")
base=root/"tmp/planning/temper-base"
paths=[p for d in ["tests/engine/forge/client", "tests/forge-setup/reconcile", "tests/engine/protocol-forge"] for p in (base/d).rglob("*.rs")]
paths += [base/"tests/codecs/src/forgejo.rs",base/"tests/engine/forge/src/world.rs"]
names=sorted(set(re.findall(r"\b(?:fn|struct|enum|type|const)\s+([A-Za-z_][A-Za-z_0-9]*)", "\n".join(p.read_text() for p in paths))) | {"temper_engine_forge_world", "temper_engine_forge_client_world", "temper_engine_protocol_forge_world", "temper_codecs_world", "writer", "remake", "recovery_completions"})
counts={name:[] for name in names}
pattern=r"\b(?:"+"|".join(names)+r")\b"
for repo in ["temper","skein","jig","smith","hearth"]:
 command=["rg","-n","--glob","*.rs","--glob","Cargo.toml","--glob","!target/**","--glob","!docs/**","--",pattern,str(root/repo)]
 result=subprocess.run(command,text=True,capture_output=True)
 assert result.returncode in (0,1),result.stderr
 lines=result.stdout.splitlines()
 for name in names: counts[name].append(sum(bool(re.search(r"\b"+re.escape(name)+r"\b",line.split(":",2)[-1])) for line in lines))
print("symbol | temper | skein | jig | smith | hearth")
for name in names: print(name+" | "+" | ".join(map(str,counts[name])))
print("searched",len(names),"names; counts are matching lines, including definitions and same-spelled symbols")
'
directory: /srv/data/git/runner/src/tmp/planning/temper-base
exit: 1
```

| Symbol | temper | skein | jig | smith | hearth |
|---|---:|---:|---:|---:|---:|

## User-count search, one pass per source line

The earlier counting script rescanned each line for every symbol; it was
stopped after 171 s (`pueue kill 785`, exit 0; queued script exit 1), then
replaced by the equivalent one-pass counter below. This was an analysis
script failure, not a Rust test/build failure or a hidden test-time change.

```text
$ python3 -c 'from pathlib import Path
from collections import Counter
import re, subprocess
root=Path("/srv/data/git/runner/src"); base=root/"tmp/planning/temper-base"
paths=[p for d in ["tests/engine/forge/client","tests/forge-setup/reconcile","tests/engine/protocol-forge"] for p in (base/d).rglob("*.rs")]
paths += [base/"tests/codecs/src/forgejo.rs",base/"tests/engine/forge/src/world.rs"]
names=set(re.findall(r"\b(?:fn|struct|enum|type|const)\s+([A-Za-z_][A-Za-z_0-9]*)", "\n".join(p.read_text() for p in paths))) | {"temper_engine_forge_world","temper_engine_forge_client_world","temper_engine_protocol_forge_world","temper_codecs_world","writer","remake","recovery_completions"}
pattern=r"\b(?:"+"|".join(sorted(names))+r")\b"; counts={name:[] for name in names}
for repo in ["temper","skein","jig","smith","hearth"]:
 result=subprocess.run(["rg","-n","--glob","*.rs","--glob","Cargo.toml","--glob","!target/**","--glob","!docs/**","--",pattern,str(root/repo)],text=True,capture_output=True)
 assert result.returncode in (0,1),result.stderr
 count=Counter()
 for line in result.stdout.splitlines(): count.update(set(re.findall(r"\b[A-Za-z_][A-Za-z_0-9]*\b",line.split(":",2)[-1])) & names)
 for name in names: counts[name].append(count[name])
print("symbol | temper | skein | jig | smith | hearth")
for name in sorted(names): print(name+" | "+" | ".join(map(str,counts[name])))
print("searched",len(names),"names; matching source lines, including definitions and same-spelled symbols")
'
directory: /srv/data/git/runner/src/tmp/planning/temper-base
searched 419 names; matching source lines, including definitions and same-spelled symbols
exit: 0
```

| Symbol | temper | skein | jig | smith | hearth |
|---|---:|---:|---:|---:|---:|
| ADMIN | 49 | 0 | 0 | 0 | 0 |
| API | 59 | 8 | 23 | 5 | 0 |
| Answer | 3021 | 146 | 2147 | 978 | 347 |
| Asked | 165 | 75 | 73 | 220 | 223 |
| BUILD_STEP_LOG | 2 | 0 | 0 | 0 | 0 |
| BUILD_STEP_NAME | 3 | 0 | 0 | 0 | 0 |
| CLIENT | 21 | 18 | 439 | 0 | 6 |
| Call | 359 | 381 | 732 | 651 | 92 |
| Connect | 43 | 234 | 12 | 49 | 3 |
| Cut | 86 | 111 | 293 | 116 | 39 |
| Decoded | 33 | 63 | 176 | 332 | 185 |
| Delivery | 793 | 2 | 1236 | 631 | 6 |
| DeliveryRequest | 3 | 0 | 0 | 0 | 0 |
| END | 6 | 128 | 3 | 14 | 4 |
| End | 293 | 356 | 848 | 917 | 111 |
| Ended | 244 | 158 | 1471 | 346 | 180 |
| Exchange | 74 | 241 | 155 | 22 | 0 |
| FORGE | 193 | 0 | 0 | 0 | 0 |
| GIT | 14 | 5 | 0 | 8 | 6 |
| HOOK_SECRET | 8 | 0 | 0 | 0 | 0 |
| Joint | 46 | 1 | 75 | 1 | 0 |
| KEY | 13 | 17 | 88 | 4 | 33 |
| Kind | 768 | 823 | 921 | 416 | 1114 |
| LIMITS | 377 | 1201 | 910 | 685 | 96 |
| Listener | 24 | 162 | 4 | 0 | 0 |
| MANAGED | 2 | 0 | 0 | 0 | 0 |
| Machine | 47 | 414 | 44 | 124 | 99 |
| Measured | 16 | 53 | 39 | 25 | 0 |
| News | 59 | 0 | 194 | 12 | 0 |
| OTHER_SECRET | 3 | 0 | 0 | 0 | 0 |
| OTHER_URL | 3 | 0 | 0 | 0 | 0 |
| OWNED | 2 | 0 | 0 | 0 | 0 |
| OtherHooks | 2 | 0 | 0 | 0 | 0 |
| PING | 11 | 0 | 0 | 0 | 0 |
| Peer | 77 | 130 | 106 | 141 | 0 |
| Pending | 239 | 400 | 476 | 57 | 148 |
| Pipe | 35 | 125 | 4 | 23 | 69 |
| Protocol | 50 | 97 | 28 | 68 | 21 |
| REPO | 402 | 0 | 0 | 1 | 0 |
| Referee | 56 | 141 | 145 | 91 | 14 |
| Response | 32 | 153 | 14 | 191 | 0 |
| SECOND | 25 | 73 | 54 | 1 | 2 |
| SETUP_TEAM | 5 | 0 | 0 | 0 | 0 |
| STEP_BOUND | 3 | 0 | 0 | 0 | 0 |
| Settings | 518 | 204 | 394 | 630 | 125 |
| Stats | 16 | 0 | 24 | 19 | 0 |
| TOKEN | 30 | 2 | 21 | 2 | 0 |
| Transcript | 52 | 28 | 94 | 201 | 0 |
| URL | 42 | 12 | 9 | 4 | 10 |
| WEBHOOK | 8 | 0 | 0 | 0 | 0 |
| WRITER | 7 | 32 | 0 | 0 | 0 |
| World | 403 | 653 | 1314 | 1374 | 11 |
| a_body_past_body_bytes_is_carried_empty_without_its_key | 1 | 0 | 0 | 0 | 0 |
| a_body_past_its_cap_is_answered_413_and_hints_nothing | 1 | 0 | 0 | 0 | 0 |
| a_busy_pass_lists_at_most_changed_items_over_the_page_plus_one | 1 | 0 | 0 | 0 | 0 |
| a_call_silent_past_its_deadline_is_a_timeout_and_aborts | 1 | 0 | 0 | 0 | 0 |
| a_call_waiting_past_its_deadline_is_unavailable_at_no_cost | 1 | 0 | 0 | 0 | 0 |
| a_change_produced_opened_checked_queued_and_landed | 2 | 0 | 0 | 0 | 0 |
| a_conditional_merge_lands_once_within_its_attempt_lifetime | 1 | 0 | 0 | 0 | 0 |
| a_connection_marked_close_finishes_its_call_and_the_next_connects | 1 | 0 | 0 | 0 | 0 |
| a_connection_past_hooks_waits_until_one_closes | 1 | 0 | 0 | 0 | 0 |
| a_created_issue_with_its_answer_lost_is_found_after_restart | 2 | 0 | 0 | 0 | 0 |
| a_deleted_bot_is_made_again | 1 | 0 | 0 | 0 | 0 |
| a_deleted_hook_is_made_again | 1 | 0 | 0 | 0 | 0 |
| a_deleted_uncertain_branch_is_held_after_its_deadline | 1 | 0 | 0 | 0 | 0 |
| a_delivery_split_at_every_point_is_answered_alike | 1 | 0 | 0 | 0 | 0 |
| a_domain_push_reaches_the_component_as_a_signed_delivery_and_a_hint | 1 | 0 | 0 | 0 | 0 |
| a_failed_ci_head_is_repaired_then_checked_again | 2 | 0 | 0 | 0 | 0 |
| a_failed_job_read_names_its_failed_step_and_reads_its_log | 1 | 0 | 0 | 0 | 0 |
| a_failed_jobs_log_end_comes_through_a_suffix_range | 1 | 0 | 0 | 0 | 0 |
| a_failed_listener_is_told_to_the_root_as_lost | 1 | 0 | 0 | 0 | 0 |
| a_get_to_a_forges_path_is_answered_405 | 1 | 0 | 0 | 0 | 0 |
| a_held_hint_stops_the_next_body_until_it_is_offered | 1 | 0 | 0 | 0 | 0 |
| a_hint_the_root_refused_is_held_and_offered_again | 1 | 0 | 0 | 0 | 0 |
| a_human_edit_of_an_old_own_marker_is_news_even_when_the_original_author_stays | 1 | 0 | 0 | 0 | 0 |
| a_job_with_no_failed_step_translates_to_its_whole_logs_end | 1 | 0 | 0 | 0 | 0 |
| a_late_opening_closed_by_another_hand_is_reported_as_drift | 1 | 0 | 0 | 0 | 0 |
| a_late_set_reaches_its_final_state | 1 | 0 | 0 | 0 | 0 |
| a_listing_longer_than_a_page_is_read_to_its_end | 1 | 0 | 0 | 0 | 0 |
| a_locate_answers_the_forges_id_for_a_repository_by_name | 1 | 0 | 0 | 0 | 0 |
| a_locate_of_a_repository_the_forge_lacks_answers_missing | 1 | 0 | 0 | 0 | 0 |
| a_malformed_head_is_answered_4xx_and_closed | 1 | 0 | 0 | 0 | 0 |
| a_managed_forge_missing_an_org_is_refused_naming_it | 1 | 0 | 0 | 0 | 0 |
| a_merge_with_its_answer_lost_is_found_after_restart_without_merging_twice | 2 | 0 | 0 | 0 | 0 |
| a_missing_signature_is_answered_403_without_reading_a_body | 1 | 0 | 0 | 0 | 0 |
| a_path_dropped_on_404_is_resolved_again_by_id | 1 | 0 | 0 | 0 | 0 |
| a_path_without_a_forge_is_answered_404 | 1 | 0 | 0 | 0 | 0 |
| a_path_without_a_secret_is_answered_404 | 1 | 0 | 0 | 0 | 0 |
| a_person_under_the_bots_login_is_refused | 1 | 0 | 0 | 0 | 0 |
| a_pre_adoption_human_edit_is_news_despite_our_retained_author_and_marker | 1 | 0 | 0 | 0 | 0 |
| a_recorded_forgejo_push_signed_by_the_world_is_hinted | 1 | 0 | 0 | 0 | 0 |
| a_redirect_drops_the_path_and_answers_unavailable | 1 | 0 | 0 | 0 | 0 |
| a_removed_team_member_is_added_again | 1 | 0 | 0 | 0 | 0 |
| a_request_unfinished_at_its_deadline_is_closed_with_no_answer | 1 | 0 | 0 | 0 | 0 |
| a_reset_after_a_written_request_is_timeout | 1 | 0 | 0 | 0 | 0 |
| a_response_cut_at_each_byte_ends_its_call_once | 1 | 0 | 0 | 0 | 0 |
| a_response_cut_at_each_byte_ends_its_call_once_seed_1 | 1 | 0 | 0 | 0 | 0 |
| a_response_cut_at_each_byte_ends_its_call_once_seed_2 | 1 | 0 | 0 | 0 | 0 |
| a_response_cut_at_each_byte_ends_its_call_once_seed_3 | 1 | 0 | 0 | 0 | 0 |
| a_revoked_token_is_made_again | 1 | 0 | 0 | 0 | 0 |
| a_second_run_writes_only_the_hooks_patches | 1 | 0 | 0 | 0 | 0 |
| a_seed_replays_and_facts_change_no_decision | 1 | 0 | 0 | 0 | 0 |
| a_signed_push_is_answered_204_before_its_hint_goes_up | 1 | 0 | 0 | 0 | 0 |
| a_team_edited_by_a_person_is_repaired | 1 | 0 | 0 | 0 | 0 |
| a_team_named_for_the_bot_is_made_with_write_on_every_repository | 1 | 0 | 0 | 0 | 0 |
| a_token_file_holding_a_secret_the_forge_never_made_is_made_again | 1 | 0 | 0 | 0 | 0 |
| a_token_file_holding_another_users_token_is_made_again | 1 | 0 | 0 | 0 | 0 |
| a_token_file_that_answers_as_the_bot_is_kept_untouched | 1 | 0 | 0 | 0 | 0 |
| a_verified_body_the_decoder_calls_malformed_is_answered_400 | 1 | 0 | 0 | 0 | 0 |
| a_verified_event_of_no_row_is_answered_204_and_hints_nothing | 1 | 0 | 0 | 0 | 0 |
| a_well_framed_malformed_document_is_invalid_answer | 1 | 0 | 0 | 0 | 0 |
| a_write_body_is_read_before_its_scripted_response | 1 | 0 | 0 | 0 | 0 |
| a_wrong_signature_is_answered_403_and_hints_nothing | 1 | 0 | 0 | 0 | 0 |
| aborts | 14 | 20 | 6 | 7 | 0 |
| accept | 50 | 307 | 97 | 10 | 3 |
| accept_hook | 2 | 0 | 0 | 0 | 0 |
| active_comment_history_has_a_bounded_slow_repair_cost_and_catches_a_lost_edit_hint | 1 | 0 | 0 | 0 | 0 |
| adopt | 123 | 16 | 43 | 20 | 0 |
| adopt_as | 5 | 0 | 0 | 0 | 0 |
| adoption_reads_history_and_preserves_foreign_or_copied_marker_news | 1 | 0 | 0 | 0 | 0 |
| an_empty_forge_is_given_its_bot_and_both_tokens | 1 | 0 | 0 | 0 | 0 |
| an_external_move_of_a_held_branch_is_reported_as_drift | 2 | 0 | 0 | 0 | 0 |
| an_idle_connection_closes_and_the_next_call_connects | 1 | 0 | 0 | 0 | 0 |
| an_org_hook_is_made_with_the_secret_the_events_json_and_active | 1 | 0 | 0 | 0 | 0 |
| an_owned_forge_makes_a_missing_org_for_its_person | 1 | 0 | 0 | 0 | 0 |
| an_uncertain_branch_is_found_without_retry_after_restart | 1 | 0 | 0 | 0 | 0 |
| an_uncertain_comment_found_by_key_is_made_without_retry | 1 | 0 | 0 | 0 | 0 |
| an_uncertain_comment_missing_after_its_deadline_holds_the_task | 1 | 0 | 0 | 0 | 0 |
| an_uncertain_open_pull_is_found_without_retry | 1 | 0 | 0 | 0 | 0 |
| an_unreachable_forge_is_checked_again_after_its_backoff | 1 | 0 | 0 | 0 | 0 |
| an_unrecoverable_creation_is_found_if_it_lands_within_its_lifetime | 1 | 0 | 0 | 0 | 0 |
| another_user_stops_the_start_once_naming_the_forge | 1 | 0 | 0 | 0 | 0 |
| another_writer_taking_a_branch_name_is_reported_without_overwriting_it | 1 | 0 | 0 | 0 | 0 |
| answer | 1802 | 1125 | 3157 | 2187 | 261 |
| answered | 406 | 284 | 571 | 752 | 5 |
| asked | 231 | 271 | 518 | 332 | 178 |
| assert_bot | 2 | 0 | 0 | 0 | 0 |
| assert_org | 2 | 0 | 0 | 0 | 0 |
| assert_other_hooks | 3 | 0 | 0 | 0 | 0 |
| body | 1961 | 1803 | 1320 | 522 | 355 |
| bot_tokens | 5 | 0 | 0 | 0 | 0 |
| bounded_job_bytes_translate_to_a_truncated_attempt_log | 1 | 0 | 0 | 0 | 0 |
| branch | 1855 | 36 | 0 | 70 | 39 |
| branches_answer | 2 | 0 | 0 | 0 | 0 |
| busy | 88 | 57 | 308 | 76 | 8 |
| byte_answer | 2 | 0 | 0 | 0 | 0 |
| bytes | 4015 | 5038 | 19300 | 4723 | 2231 |
| call | 1256 | 1482 | 2290 | 3087 | 529 |
| calls | 426 | 443 | 443 | 1315 | 47 |
| calm | 117 | 372 | 98 | 490 | 39 |
| carried | 20 | 20 | 101 | 63 | 0 |
| case | 501 | 277 | 155 | 128 | 342 |
| case_bytes | 5 | 0 | 0 | 0 | 0 |
| change_row | 39 | 0 | 0 | 0 | 0 |
| change_step | 40 | 0 | 0 | 0 | 0 |
| change_step_task | 8 | 0 | 0 | 0 | 0 |
| change_step_with_release | 6 | 0 | 0 | 0 | 0 |
| check | 692 | 493 | 4162 | 1610 | 222 |
| check_out | 18 | 10 | 0 | 3 | 0 |
| check_pull | 2 | 0 | 0 | 0 | 0 |
| checks | 372 | 193 | 194 | 662 | 23 |
| checks_answer | 2 | 0 | 0 | 0 | 0 |
| client | 2988 | 3518 | 6879 | 1074 | 149 |
| client_down | 3 | 13 | 0 | 0 | 0 |
| client_up | 4 | 5 | 0 | 0 | 0 |
| close | 405 | 1664 | 601 | 536 | 146 |
| collaborators_answer | 2 | 0 | 0 | 0 | 0 |
| combined | 38 | 1 | 37 | 21 | 4 |
| comment_ids | 9 | 0 | 0 | 0 | 0 |
| comment_page_recovery_and_old_comment_edits_reach_the_subscriber_once | 1 | 0 | 0 | 0 | 0 |
| commit | 2110 | 191 | 818 | 329 | 166 |
| config | 1243 | 663 | 832 | 556 | 993 |
| configured | 316 | 222 | 507 | 496 | 12 |
| connections_opened | 16 | 0 | 0 | 0 | 0 |
| connects | 7 | 58 | 0 | 1 | 0 |
| converged | 8 | 0 | 0 | 0 | 0 |
| costs | 14 | 8 | 7 | 39 | 37 |
| creations_and_comments_survive_faults_late_landings_and_restart | 1 | 0 | 0 | 0 | 0 |
| cut_script | 2 | 0 | 0 | 0 | 0 |
| cutting | 29 | 7 | 5 | 1 | 1 |
| debug | 9 | 8 | 13 | 7 | 4 |
| decode | 1478 | 374 | 10429 | 1185 | 94 |
| decode_answer | 8 | 0 | 10 | 7 | 0 |
| decode_body | 3 | 0 | 0 | 0 | 0 |
| decode_delivery | 6 | 0 | 0 | 9 | 0 |
| decode_error | 9 | 20 | 0 | 0 | 16 |
| decode_refusal | 5 | 0 | 12 | 0 | 0 |
| decode_route | 5 | 0 | 0 | 0 | 0 |
| delivered | 205 | 169 | 370 | 154 | 5 |
| delivery | 481 | 161 | 513 | 690 | 17 |
| delivery_cases | 4 | 0 | 2 | 0 | 0 |
| digest | 103 | 150 | 259 | 201 | 444 |
| direct | 41 | 2 | 69 | 14 | 5 |
| down | 428 | 1638 | 683 | 267 | 157 |
| drain | 161 | 192 | 270 | 112 | 20 |
| drive | 193 | 122 | 1326 | 105 | 10 |
| edit_comment | 3 | 0 | 0 | 0 | 0 |
| end | 629 | 1509 | 1482 | 1652 | 382 |
| enqueue_merge | 2 | 0 | 0 | 0 | 0 |
| entry | 1032 | 685 | 2744 | 710 | 1590 |
| error | 364 | 1121 | 286 | 1104 | 927 |
| event | 1600 | 1745 | 3079 | 1997 | 321 |
| events | 521 | 2066 | 819 | 852 | 124 |
| every_live_fresh_read_finishes_once_under_faults_and_rate_resets | 1 | 0 | 0 | 0 | 0 |
| external | 107 | 58 | 64 | 41 | 2 |
| external_at | 13 | 0 | 0 | 0 | 0 |
| failing | 53 | 30 | 3 | 25 | 8 |
| fake_config | 25 | 0 | 0 | 0 | 0 |
| fake_limits | 8 | 9 | 0 | 3 | 0 |
| file | 666 | 1394 | 664 | 1792 | 1127 |
| file_writes | 5 | 0 | 0 | 0 | 0 |
| files_in | 3 | 0 | 0 | 0 | 0 |
| finish | 1243 | 651 | 8266 | 1612 | 181 |
| fmt | 63 | 98 | 30 | 38 | 23 |
| fn | 10698 | 10021 | 42564 | 11617 | 5814 |
| full_live_caches_inbox_stamps_echoes_outbox_and_every_entry_point_fit | 1 | 0 | 0 | 0 | 0 |
| goal_view | 3 | 0 | 0 | 0 | 0 |
| has_work | 7 | 73 | 4 | 15 | 0 |
| hear | 4 | 2 | 7 | 4 | 0 |
| held_branch | 2 | 0 | 0 | 0 | 0 |
| hex | 64 | 65 | 5 | 35 | 136 |
| hint | 124 | 25 | 34 | 2 | 2 |
| hinted_after_answer | 3 | 0 | 0 | 0 | 0 |
| hints | 41 | 0 | 17 | 0 | 13 |
| historical_comment | 10 | 0 | 0 | 0 | 0 |
| historical_issue | 9 | 0 | 0 | 0 | 0 |
| historical_pull | 5 | 0 | 0 | 0 | 0 |
| historical_review | 3 | 0 | 0 | 0 | 0 |
| historical_topic | 4 | 0 | 0 | 0 | 0 |
| honest_costs_equal_the_requests_the_fake_served | 1 | 0 | 0 | 0 | 0 |
| hook_limits | 7 | 0 | 0 | 0 | 0 |
| hook_settings | 7 | 0 | 0 | 0 | 0 |
| hooked | 37 | 0 | 0 | 0 | 0 |
| hooked_protocol_config | 3 | 0 | 0 | 0 | 0 |
| idle_cost_is_independent_of_ten_times_the_history | 1 | 0 | 0 | 0 | 0 |
| idle_forge_call_cost_does_not_follow_old_issue_history | 2 | 0 | 0 | 0 | 0 |
| ids | 90 | 49 | 106 | 49 | 116 |
| inactive_history_does_not_change_a_participating_objects_idle_calls | 1 | 0 | 0 | 0 | 0 |
| inbox_count | 2 | 0 | 0 | 0 | 0 |
| io | 1183 | 2849 | 614 | 1321 | 286 |
| issue | 393 | 79 | 7 | 17 | 64 |
| issues_made | 3 | 0 | 0 | 0 | 0 |
| iterate | 133 | 122 | 93 | 97 | 15 |
| job_answer | 4 | 0 | 0 | 4 | 0 |
| json_cases | 2 | 0 | 0 | 0 | 0 |
| keep | 230 | 229 | 164 | 112 | 48 |
| key | 2176 | 1238 | 5700 | 957 | 1927 |
| key_of | 7 | 0 | 10 | 0 | 0 |
| keyed | 85 | 17 | 307 | 1 | 5 |
| land_change | 2 | 0 | 0 | 0 | 0 |
| land_writes_late | 12 | 0 | 0 | 0 | 0 |
| limits | 9583 | 5874 | 56714 | 10141 | 2224 |
| listener | 255 | 702 | 111 | 5 | 6 |
| listening | 20 | 121 | 20 | 2 | 0 |
| listing_passes | 3 | 0 | 0 | 0 | 0 |
| locate_name | 7 | 0 | 0 | 0 | 0 |
| lose | 3 | 2 | 21 | 2 | 0 |
| main_name | 4 | 0 | 0 | 0 | 0 |
| make | 185 | 150 | 418 | 50 | 176 |
| managed | 46 | 0 | 1 | 0 | 0 |
| markers_have_the_protocol_spelling_and_arbitrary_bytes_round_trip | 1 | 0 | 0 | 0 | 0 |
| missed_edit_comment | 3 | 0 | 0 | 0 | 0 |
| missed_review_submission | 2 | 0 | 0 | 0 | 0 |
| missing_org | 2 | 0 | 0 | 0 | 0 |
| new | 6629 | 5919 | 25731 | 8994 | 2348 |
| news | 87 | 0 | 352 | 3 | 0 |
| no_secret_is_in_any_failure_debug_or_trace | 1 | 0 | 0 | 0 | 0 |
| now | 1276 | 1577 | 3028 | 1752 | 174 |
| number | 3530 | 586 | 8914 | 882 | 92 |
| observe | 83 | 128 | 293 | 286 | 2 |
| op | 604 | 1094 | 64 | 403 | 257 |
| open_pull | 22 | 0 | 0 | 0 | 0 |
| opened_change | 2 | 0 | 0 | 0 | 0 |
| opening_with_nothing_to_merge_writes_no_pull_request | 1 | 0 | 0 | 0 | 0 |
| outcomes | 84 | 40 | 52 | 52 | 16 |
| outside_branch | 2 | 0 | 0 | 0 | 0 |
| outside_close_pull | 2 | 0 | 0 | 0 | 0 |
| outside_delete_branch | 2 | 0 | 0 | 0 | 0 |
| own_branch | 2 | 0 | 0 | 0 | 0 |
| owned | 588 | 347 | 443 | 327 | 64 |
| page | 902 | 890 | 2232 | 6 | 76 |
| page_range | 4 | 0 | 0 | 0 | 0 |
| peer | 269 | 879 | 703 | 1056 | 15 |
| permission | 259 | 22 | 170 | 7 | 6 |
| post | 98 | 32 | 19 | 20 | 0 |
| preload_history | 6 | 0 | 12 | 0 | 0 |
| produce | 35 | 0 | 7 | 14 | 1 |
| produce_file | 6 | 0 | 0 | 0 | 0 |
| profile_limits | 9 | 0 | 10 | 0 | 0 |
| protocol_config | 5 | 0 | 0 | 0 | 0 |
| protocol_shared | 3 | 0 | 0 | 0 | 0 |
| pull | 919 | 23 | 1 | 2 | 1 |
| pull_answer | 7 | 0 | 0 | 0 | 0 |
| pull_for_answers_the_open_pull_when_the_lookup_names_a_closed_one | 1 | 0 | 0 | 0 | 0 |
| push | 2677 | 2058 | 6136 | 2248 | 1316 |
| push_as | 5 | 0 | 0 | 0 | 0 |
| push_file | 9 | 0 | 0 | 0 | 0 |
| push_hook | 2 | 0 | 0 | 0 | 0 |
| put | 1167 | 574 | 6451 | 1218 | 879 |
| random | 102 | 128 | 96 | 76 | 3 |
| random_initial_forges_converge_and_a_second_run_writes_only_hook_patches | 1 | 0 | 0 | 0 | 0 |
| read | 1660 | 2554 | 3196 | 1960 | 573 |
| read_op | 2 | 0 | 0 | 0 | 0 |
| read_result | 11 | 0 | 30 | 36 | 0 |
| ready | 287 | 343 | 619 | 175 | 82 |
| recent_review | 3 | 0 | 0 | 0 | 0 |
| reconcile | 136 | 0 | 4 | 37 | 0 |
| recovery_completions | 0 | 0 | 0 | 0 | 0 |
| refuse | 29 | 51 | 163 | 92 | 8 |
| remake | 4 | 0 | 0 | 0 | 0 |
| repair_cost | 3 | 0 | 0 | 0 | 0 |
| replace | 165 | 136 | 204 | 252 | 70 |
| request | 2109 | 2032 | 3874 | 1643 | 737 |
| requests | 339 | 1077 | 1231 | 535 | 138 |
| resolve | 20 | 26 | 12 | 88 | 265 |
| respond | 13 | 60 | 83 | 0 | 0 |
| response | 272 | 1283 | 139 | 732 | 5 |
| restart | 164 | 72 | 582 | 5 | 26 |
| restore_participants | 2 | 0 | 0 | 0 | 0 |
| resume | 102 | 94 | 106 | 235 | 64 |
| review_count | 4 | 0 | 0 | 0 | 0 |
| review_page_recovery_keeps_old_pending_submissions_without_duplicate_news | 1 | 0 | 0 | 0 | 0 |
| route | 350 | 93 | 572 | 107 | 0 |
| run | 1577 | 1763 | 3754 | 8888 | 598 |
| run_for | 215 | 0 | 0 | 0 | 0 |
| run_to | 6 | 0 | 0 | 0 | 0 |
| run_until_inbox | 4 | 0 | 0 | 0 | 0 |
| script_done | 8 | 0 | 0 | 0 | 0 |
| secret | 380 | 117 | 331 | 32 | 69 |
| secrets_absent | 8 | 0 | 0 | 0 | 0 |
| seed_org | 2 | 0 | 0 | 0 | 0 |
| seed_token | 2 | 0 | 0 | 0 | 0 |
| seeded_faults_restarts_and_keyed_effects | 1 | 0 | 0 | 0 | 0 |
| seen | 191 | 876 | 665 | 855 | 28 |
| send | 590 | 603 | 1211 | 144 | 15 |
| server_down | 3 | 20 | 0 | 0 | 0 |
| server_events | 2 | 9 | 0 | 0 | 0 |
| server_up | 3 | 0 | 0 | 0 | 0 |
| settings | 1055 | 565 | 599 | 1613 | 576 |
| settings_answered_404_stops_the_start_with_its_status | 1 | 0 | 0 | 0 | 0 |
| settle | 380 | 282 | 275 | 479 | 33 |
| setup_cases | 5 | 0 | 0 | 0 | 0 |
| setup_hook_cases | 2 | 0 | 0 | 0 | 0 |
| setup_hook_events | 2 | 0 | 0 | 0 | 0 |
| setup_hook_spec | 3 | 0 | 0 | 0 | 0 |
| setup_org_cases | 2 | 0 | 0 | 0 | 0 |
| setup_team_spec | 3 | 0 | 0 | 0 | 0 |
| sha | 104 | 5 | 0 | 0 | 18 |
| shape | 49 | 114 | 153 | 52 | 138 |
| shared | 262 | 174 | 178 | 602 | 138 |
| signed | 54 | 74 | 129 | 22 | 15 |
| slow_calls | 10 | 0 | 0 | 0 | 0 |
| slow_repair_finds_a_lost_review_hint_without_item_time_revision_evidence | 1 | 0 | 0 | 0 | 0 |
| specimen | 202 | 0 | 0 | 0 | 0 |
| stale_team | 3 | 0 | 0 | 0 | 0 |
| stamps | 29 | 0 | 0 | 0 | 0 |
| start_reads | 2 | 0 | 0 | 0 | 3 |
| state | 1287 | 1323 | 3132 | 880 | 149 |
| stats | 173 | 9 | 173 | 257 | 0 |
| status | 1002 | 416 | 525 | 360 | 23 |
| statuses | 297 | 4 | 6 | 0 | 0 |
| statuses_answer | 2 | 0 | 0 | 0 | 0 |
| step_change | 8 | 0 | 0 | 0 | 0 |
| stopped | 111 | 145 | 172 | 111 | 0 |
| store | 1397 | 714 | 4006 | 415 | 911 |
| store_mut | 24 | 0 | 0 | 0 | 0 |
| stored | 157 | 47 | 197 | 64 | 19 |
| stories_through_cut_streams_answer_each_call_once | 1 | 0 | 0 | 0 | 0 |
| submit | 149 | 541 | 145 | 158 | 13 |
| submit_review | 2 | 0 | 0 | 0 | 0 |
| summary | 110 | 41 | 116 | 629 | 0 |
| sweep_seed | 5 | 0 | 0 | 0 | 0 |
| take_answers | 3 | 0 | 0 | 0 | 0 |
| take_client | 4 | 0 | 0 | 0 | 0 |
| take_forge | 6 | 0 | 0 | 0 | 0 |
| take_seen | 50 | 0 | 0 | 0 | 0 |
| take_top | 8 | 0 | 0 | 0 | 0 |
| tally | 136 | 35 | 46 | 85 | 27 |
| team | 184 | 0 | 1 | 0 | 0 |
| temper_codecs_world | 19 | 0 | 0 | 0 | 0 |
| temper_engine_forge_client_world | 20 | 0 | 0 | 0 | 0 |
| temper_engine_forge_world | 57 | 0 | 0 | 0 | 0 |
| temper_engine_protocol_forge_world | 12 | 0 | 0 | 0 | 0 |
| the_api_token_reaches_no_record_request_or_fact | 1 | 0 | 0 | 0 | 0 |
| the_component_starts_and_lists_items_through_the_fakes_bytes | 1 | 0 | 0 | 0 | 0 |
| the_groundwork_reads_keep_the_fake_api_facts_in_the_client_projection | 1 | 0 | 0 | 0 | 0 |
| the_secret_appears_in_no_event_or_debug_text | 1 | 0 | 0 | 0 | 0 |
| the_start_reaches_ready_from_the_recorded_answers | 1 | 0 | 0 | 0 | 0 |
| the_tokens_hold_the_scopes_of_setup_section_4 | 1 | 0 | 0 | 0 | 0 |
| the_webhook_secret_is_drawn_when_missing | 1 | 0 | 0 | 0 | 0 |
| the_webhook_secret_is_kept_when_present | 1 | 0 | 0 | 0 | 0 |
| tick | 96 | 102 | 138 | 28 | 6 |
| time | 243 | 407 | 331 | 757 | 32 |
| token_absent | 7 | 0 | 0 | 0 | 0 |
| token_kept | 3 | 0 | 0 | 0 | 0 |
| top | 864 | 97 | 36 | 24 | 33 |
| trace | 103 | 362 | 272 | 505 | 22 |
| transcript | 314 | 168 | 397 | 728 | 0 |
| transcript_cases | 3 | 0 | 0 | 0 | 0 |
| two_deliveries_on_one_connection_are_both_answered_and_hinted | 1 | 0 | 0 | 0 | 0 |
| two_hooks_with_the_listeners_url_keep_the_lowest_patched_and_the_rest_deleted | 1 | 0 | 0 | 0 | 0 |
| unrecoverable_branch_creation_survives_cuts_before_and_after_its_late_copy | 1 | 0 | 0 | 0 | 0 |
| up | 554 | 1660 | 887 | 404 | 169 |
| user | 1019 | 239 | 25 | 181 | 209 |
| verdict | 268 | 17 | 492 | 477 | 22 |
| verdict_out | 2 | 0 | 0 | 0 | 0 |
| walk | 82 | 38 | 18 | 18 | 69 |
| wall | 373 | 598 | 1489 | 536 | 89 |
| with | 747 | 1168 | 2451 | 1092 | 126 |
| with_facts | 6 | 0 | 0 | 0 | 0 |
| world | 4037 | 4491 | 11006 | 5820 | 214 |
| write | 1088 | 730 | 453 | 810 | 182 |
| write_cases | 6 | 0 | 0 | 0 | 0 |
| write_op | 2 | 0 | 0 | 0 | 0 |
| write_other_cases | 2 | 0 | 0 | 0 | 0 |
| write_routes | 6 | 0 | 0 | 0 | 0 |
| writer | 2726 | 1221 | 19043 | 2217 | 207 |
| writer_settings | 3 | 0 | 0 | 0 | 0 |
| writes | 524 | 305 | 656 | 132 | 20 |

## Owner literals and derived frames

The client move also changes the independent marker fixture and golden bytes,
and the namespace/old-key frame together. The memory fixture continues to
fill its namespace to the declared bound. Other owners keep their bytes.

```text
$ rg -n -F -e worldold -e '<!-- temper:key ' -e 'b"world"' /srv/data/git/runner/src/temper /srv/data/git/runner/src/skein /srv/data/git/runner/src/jig /srv/data/git/runner/src/smith /srv/data/git/runner/src/hearth --glob '*.rs' --glob '!target/**'
directory: /srv/data/git/runner/src/tmp/planning/temper-base
/srv/data/git/runner/src/smith/tests/agent-process/src/tracing.rs:137:                        version: b"world".as_slice().into(),
/srv/data/git/runner/src/jig/tests/core/tests/stories/watches.rs:77:    for (offset, text) in [(0, b"hello ".as_slice()), (6, b"world".as_slice())] {
/srv/data/git/runner/src/jig/tests/core/tests/stories/watches.rs:88:    assert_eq!((streams[1].from, streams[1].to, streams[1].text.as_ref()), (6, 11, b"world".as_slice()));
/srv/data/git/runner/src/temper/tools/forgejo-conformance/src/cases.rs:105:        Some("{\"title\":\"fixture issue\",\"body\":\"<!-- temper:key 01000001 -->\\nissue text\"}"),
/srv/data/git/runner/src/temper/tools/forgejo-conformance/src/cases.rs:115:        Some("{\"body\":\"<!-- temper:key 01000002 -->\\ncomment text\"}"),
/srv/data/git/runner/src/temper/tools/forgejo-conformance/src/cases.rs:136:            Some(&format!("{{\"event\":\"{event}\",\"body\":\"<!-- temper:key 010000{key} -->\\nreview text\"}}")),
/srv/data/git/runner/src/temper/tools/forgejo-conformance/src/deliveries.rs:305:    s.act("POST", "/issues", &json!({"title":"issue","body":"<!-- temper:key 01 -->\n"}), false, &["issues"])?;
/srv/data/git/runner/src/temper/tools/forgejo-conformance/src/deliveries.rs:312:    s.act("POST", "/issues/1/comments", &json!({"body":"<!-- temper:key 02 -->\n"}), true, &["issue_comment"])?;
/srv/data/git/runner/src/temper/tools/forgejo-conformance/src/deliveries.rs:323:    s.act("POST", "/issues/2/comments", &json!({"body":"<!-- temper:key 03 -->\n"}), true, &["pull_request_comment"])?;
/srv/data/git/runner/src/temper/tools/forgejo-conformance/src/deliveries.rs:335:            &json!({"event":event,"body":"<!-- temper:key 04 -->\n"}),
/srv/data/git/runner/src/temper/tools/forgejo-conformance/src/component/writes.rs:30:            && got["body"] == "<!-- temper:key 01000001 -->\ncomponent body",
/srv/data/git/runner/src/temper/tools/forgejo-conformance/src/component/writes.rs:57:    facts.push(super::fact("component-write-post", matches!(post, Ok(Answer::Commented(id)) if comments.as_array().is_some_and(|rows| rows.iter().any(|r| r["id"] == id && r["body"] == "<!-- temper:key 01000002 -->\npost body"))), &post));
/srv/data/git/runner/src/temper/tools/forgejo-conformance/src/component/writes.rs:69:    facts.push(super::fact("component-write-review", matches!(review, Ok(Answer::Reviewed(id)) if reviews.as_array().is_some_and(|rows| rows.iter().any(|r| r["id"] == id && r["body"] == "<!-- temper:key 01000003 -->\nkeyed review" && r["state"] == "COMMENT"))), &review));
/srv/data/git/runner/src/temper/tools/forgejo-conformance/src/component/writes.rs:157:    facts.push(super::fact("component-keys-found-on-read-back", matches!(&item, Ok(Answer::Item { item, comments, .. }) if item.key == Some(Box::new([1, 0, 0, 1])) && comments.iter().any(|c| c.key == Some(Box::new([1, 0, 0, 2])) && c.body.as_ref() == b"<!-- temper:key 01000002 -->\npost body")) && matches!(&reviews, Ok(Answer::Reviews { reviews, .. }) if reviews.iter().any(|r| r.key == Some(Box::new([1, 0, 0, 3])) && r.body.as_ref() == b"<!-- temper:key 01000003 -->\nkeyed review")), &(item, reviews)));
/srv/data/git/runner/src/temper/tools/forgejo-conformance/src/facts.rs:364:        comment.body["body"] == "<!-- temper:key 01000002 -->\ncomment text",
/srv/data/git/runner/src/temper/tests/codecs/tests/forgejo_writes.rs:46:                body: Body::Kept(Box::from(*b"<!-- temper:key 01000001 -->\nissue text")),
/srv/data/git/runner/src/temper/tests/codecs/tests/forgejo_writes.rs:75:        [("post-comment", 4, "<!-- temper:key 01000002 -->\ncomment text"), ("post-comment-plain", 5, "plain comment")]
/srv/data/git/runner/src/temper/tests/codecs/tests/forgejo_writes.rs:89:        ("post-review-approved", 1, ReviewState::Approved, true, "<!-- temper:key 01000003 -->\nreview text"),
/srv/data/git/runner/src/temper/tests/codecs/tests/forgejo_writes.rs:95:            "<!-- temper:key 01000004 -->\nreview text",
59 matching lines in the full log
exit: 0
```

- temper: 54 matching lines.
- skein: 2 matching lines.
- jig: 2 matching lines.
- smith: 1 matching lines.
- hearth: 0 matching lines.

## Shared base advance and received design answers

Reflogs show both base worktrees checked out the designer’s answer commits
at 00:47:15 UTC. The client runs preceded this; setup/protocol, counts and
searches followed it. The original scratch compile stayed at 27995fb75.
The complete changed-path lists below establish that no Rust, manifests,
lockfiles, fixtures or test settings changed; no code rerun was necessary.
Answers 14 and 23 were read from decisions.md and incorporated at that time.
That draft's unresolved status for 25 and S07-A is superseded: answer 25
requires the mark whole on both sides; decisions 22 and 23 settle S07-A.

```text
$ git diff --name-only 27995fb75 HEAD
directory: /srv/data/git/runner/src/tmp/planning/temper-base
docs/design/protocol/setup.md
exit: 0
```

```text
$ git diff --name-only d120820 HEAD
directory: /srv/data/git/runner/src/tmp/planning/skein-base
docs/design/fake-forge.md
docs/design/forge-client.md
docs/design/forge-connection.md
docs/design/forge-setup.md
docs/design/forge.md
exit: 0
```


## Final member check

Anchor to member rows so a workspace dependency path does not count twice.

```text
$ rg -n '^    "tests/(engine/forge/client|forge-setup/reconcile|engine/protocol-forge)",$' Cargo.toml
directory: /srv/data/git/runner/src/tmp/planning/temper-base
6:    "tests/forge-setup/reconcile",
13:    "tests/engine/protocol-forge",
45:    "tests/engine/forge/client",
exit: 0
```

## Review corrections, 2026-10-19

This section supersedes the earlier S07-A recommendation, protocol partition
ownership and increment numbers. Decisions 22 and 23 settle that scope;
there is no new design question about it. Session 07 now has four increments:

| Increment | Scope | Size |
|---|---|---|
| 07.1 | Client world moved | about 2,800 lines moved, 80 new |
| 07.2 | Scripted parent's recovery | about 180 lines moved, 180 new |
| 07.3 | Setup world moved | about 950 lines moved, 120 new |
| 07.4 | Paging and independent-owner setup stories | about 0 lines moved, 220 new |

- Review 1, 8, 9: removed S07-A and the protocol increment and closing
  checks from the session. Its member check now names client and setup only.
- Review 2: `src/joint.rs`, its signed-delivery story and a world-side
  `fake_config` belong to skein 08.1; no Joint size or machine test count
  is claimed by session 07. This revision does not edit session 08.
- Review 3: the five movable `fake_protocol` tests belong to skein 08.3;
  their dependency on 07.1's translation belongs there too. The nine
  focused policy tests and one fuzzy policy test stay temper's (decision 22).
- Review 4 and 6: 07.3 needs temper 04.1 and skein 08.1's shared `Pipe`,
  `Cut`, `fake_limits`; 07.1 explicitly needs skein 02.2's byte measure
  and remeasured `client::worst_case`. No skein 03.4 Need remains.
- Review 5: renumbered all local references in file order, 07.1–07.4;
  the setup move is 07.3, matching the README and code organization.
- Review 7: `World::trace` is included in the setup move's keep-list.
- Review 10 and (b)/(c): forge-connection 10's client/component stories
  and invariants belong to skein 08.4–08.5, with its machine/listener
  work in 08.1–08.2; session 07 claims only its two worlds.

Design questions 14, 22, 23 and 25 are answered. The earlier review's status
for 25 is superseded by answers-2: the mark matches whole on both sides,
as 07.4 now says.
The four retained nextest commands are unchanged; their source-package
runs above remain the evidence (client 29/2, setup 21/1). This revision
adds no tests to those bases and claims no run of the future 31/27 counts.
Destination packages and new stories are checked against landed code at
the ready check, as development.md 5.4 requires.

### Graph and source checks

Queried the supplied `srv-data-git-runner-src-temper` graph with
`search_graph`, `get_code_snippet` and `search_code`. The graph locates
the client `World`; setup and protocol-helper searches returned no
symbols, so direct `rg` at temper-base verified their names and consumers.
No project/index-management command was used. Existing names in the
session were checked in the source; recovery methods and new story names
are additions, not claims of existing definitions. Setup's given-values
type and `Item` remain skein 04.1's interface, with no second sketch here.

```text
$ rg -n 'Pipe|Cut|fake_limits|pub fn trace' tests/forge-setup/reconcile/src
directory: /srv/data/git/runner/src/tmp/planning/temper-base
tests/forge-setup/reconcile/src/world.rs:37:        let limits = temper_engine_protocol_forge_world::fixtures::fake_limits();
tests/forge-setup/reconcile/src/world.rs:91:    pub fn trace(&self) -> &[String] {
tests/forge-setup/reconcile/src/fixtures.rs:79:    let limits = temper_engine_protocol_forge_world::fixtures::fake_limits();
tests/forge-setup/reconcile/src/exchange.rs:7:use temper_engine_protocol_forge_world::pipe::{Cut, Pipe};
tests/forge-setup/reconcile/src/exchange.rs:23:    pipe: Pipe,
tests/forge-setup/reconcile/src/exchange.rs:47:        pipe: Pipe::new(seed, Cut::Never),
exit: 0
```

### The two narrowed closing commands, in source form

The destination paths do not exist at skein-base. These are the base
source-path forms of the new two-world member and neutral-name checks.
The source has two member rows and still names temper; the destination
must have two member rows and no such names (rg exit 1).

```text
$ rg -n '^    "tests/(engine/forge/client|forge-setup/reconcile)",$' Cargo.toml
directory: /srv/data/git/runner/src/tmp/planning/temper-base
6:    "tests/forge-setup/reconcile",
45:    "tests/engine/forge/client",
exit: 0
```

```text
$ rg -n -i 'temper' tests/engine/forge/client tests/forge-setup/reconcile
directory: /srv/data/git/runner/src/tmp/planning/temper-base
tests/forge-setup/reconcile/tests/reconcile.rs:1:use temper_fake_forge_protocol::Route;
tests/forge-setup/reconcile/tests/reconcile.rs:2:use temper_forge_forgejo::request::Request;
tests/forge-setup/reconcile/tests/reconcile.rs:3:use temper_forge_setup::place::File;
tests/forge-setup/reconcile/tests/reconcile.rs:4:use temper_forge_setup_reconcile_world::{Ended, World, fixtures};
144 matching source lines in the full test log; excerpt above.
exit: 0
```

### Recovery signatures compiled on skein-base

The earlier scratch was based on temper. For this review a detached
worktree was made from skein-base `d9f3e39`. Copied the client world,
client and fake-domain source crates from temper-base `b4cfb999`, using
skein-base's local dependencies. These temporary source package names are
only compile scaffolding; no extraction implementation or future API is
claimed. Added `world/recovery.rs` with the three sketched methods on the
source `World`, using its `Entry` and `Resource` and `todo!()` bodies.
Unchanged signatures were removed from the session's interface sketches.

```text
$ git worktree add --detach /srv/data/git/runner/src/tmp/planning/scratch-skein-07-review d9f3e39
directory: /srv/data/git/runner/src/tmp/planning/skein-base
Preparing worktree (detached HEAD d9f3e39)
HEAD is now at d9f3e39 Answer the forge kit planner's shape questions
exit: 0
```

The first compile caught `clippy::doc_markdown` in the sketch's method
documentation. Added backticks around the two event names in the session
and scratch; kept `#[must_use]` on the read-only counter method. No lint
exception, dependency optimization or implementation change was needed.

```text
$ cargo clippy -p temper-engine-forge-client-world --all-targets -- -D warnings
directory: /srv/data/git/runner/src/tmp/planning/scratch-skein-07-review
error: item in documentation is missing backticks
  --> tests/forge/client/src/world/recovery.rs:13:31
error: item in documentation is missing backticks
  --> tests/forge/client/src/world/recovery.rs:13:50
error: could not compile `temper-engine-forge-client-world` (lib) due to 2 previous errors
exit: 101
```

```text
$ cargo clippy -p temper-engine-forge-client-world --all-targets -- -D warnings
directory: /srv/data/git/runner/src/tmp/planning/scratch-skein-07-review
    Checking temper-engine-forge-client-world v0.0.0 (/srv/data/git/runner/src/tmp/planning/scratch-skein-07-review/tests/forge/client)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.63s
exit: 0
```

```text
$ git worktree remove --force /srv/data/git/runner/src/tmp/planning/scratch-skein-07-review
directory: /srv/data/git/runner/src/tmp/planning/skein-base
exit: 0
```

Historical structural check (superseded by Re-check corrections): the review
recorded 279 lines, but the file it left was 280, with four increments of
44/69/42/54 lines, each with all nine labels and no obsolete protocol
increment, S07-A, 03.4 or 07.5 reference. All new-line Sizes are at most
220. Only `skein/07-worlds.md` and this findings file were revised;
the scratch was removed, and no commit or landing was made.

## Re-check corrections, 2026-10-19

Applied all six items as ruled in `tmp/planning/reviews/skein-07b.txt`,
preserving the earlier review fixes and four increments. This section and
the notices above supersede the draft's assignments and answer statuses;
none of its recorded command output has been changed.

1. 07.3 names every owner-value substitution in world.rs, fixtures.rs and
   the tests: word/login `kit`, admin `kit-admin`, token names `kit-api` and
   `kit-git`, email `kit@example.invalid`, seeded/expected mark `kit:bot`.
   The names are handed literally through 04.1's Given fields, not derived.
2. 07.3 names the base-to-04.1 adaptations: `entry: Entry` and the separate
   listener become Given; listening and missing_org update its fields;
   Reconcile::new takes it by reference and its Result is checked before
   Begin; Item replaces File in the store and Read/Write arms. The fixture's
   owned selector supplies Some(ada) or None as org_owner, with no make flag.
   These use skein 04.1's interface already reached by temper 04.1; they
   introduce no second type or new interface sketch in this revision.
3. `World::missing_org` and `World::tally` join the keep-list; trace stays.
4. 08.1's package is `skein-forge-machine-world`; `pipe::{Pipe,Cut}` and
   `fixtures::fake_limits` are public in `skein_forge_machine_world`.
   07.3 now lists all seven dependencies: skein-lib, skein-http, skein-json,
   skein-forge, skein-forge-setup, skein-fake-forge-protocol and the machine
   world. The private decode module uses skein-json's tokenizer, skein-lib
   and skein-forge; exchange calls crate::decode::decode_answer. No helper
   export decision is deferred, and no Pipe/Cut implementation is copied.
5. 07.1 keeps world/referee private and Referee pub(crate); tests reach
   LIMITS at the client-world crate root. No visibility is widened.
6. Stale findings are marked where they occur: recovery/simulation split
   is 07.2, setup/decode copy is 07.3, and the old 279-line claim was wrong.
   The revised session is **305 lines**, with Sizes unchanged: 07.1 about
   2,800 moved/80 new; 07.2 about 180/180; 07.3 about 950/120; 07.4 about
   0/220. All four increments retain Scope, Needs, Files, Shape, Test shape,
   Proof, Don't, Size and Removes, and all existing closing commands.

Source checks used the supplied temper graph: search_graph returned no
matches for the scoped world/helper query; search_code found the client
Referee, with no setup helper coverage. Direct rg checks in temper-base
confirmed lib.rs's private modules/root exports, Referee's visibility,
World's fields and calls, entry's owned selector, all owner literals,
Pipe/Cut/fake_limits, the manifest edges, and Decoded/decode_answer/drive.
The copied driver directly uses skein_json::tokenizer, which needs a direct
skein-json dependency after leaving the codecs crate. rg in skein-base
confirmed the existing lib/http/json package names and design's kit crate
names. Given's Rust field names do not exist at these bases yet; checked
them against their owner, skein/04-setup.md 04.1. The future machine package
and module paths match skein/08-component-worlds.md 08.1.

No new Proof or Done when command, test, limit, or interface sketch was
prescribed, so no command or scratch compile was rerun. The four
source-package nextest runs, two narrowed closing searches and corrected
skein-base recovery-signature compile recorded above remain the evidence.
Destination types/packages are still checked against landed code at readiness
(development.md 5.4); that is not an unsettled design question. Only the
session and this findings file were written; no commit or landing was made.
