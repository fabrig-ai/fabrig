# The box's checks: the goal's contracts

2026-11-23. What the changes moving `temper-box::suite`'s admission, configuration, startup and support checks share while they run (`../../draft/design-economy.md`, 4.3): the shapes one change makes and another reads, each moved test's place, and their order. The lasting design is [README.md](README.md); this document goes with the goal's last change.

## 1. In one page

- **Thirteen parts,** A to M. Eight start at once (B to H, and A); four build on A's composed profiles (I, J, K, L); M, last, removes what the others leave unused and this document.
- **Every row moves whole:** a moved test keeps each assertion and each table row (`../../draft/slow-tests-down.md`, 3.2). Where a test keeps a native row above, the table says which.
- **A slow test is deleted** by the part that adds or names its holder below, in the same change, and its report names the holder of each assertion.

## 2. Shapes the parts share

### 2.1 Composed profiles (A; read by I, J, K, L)

- **The producers compose, without writing:** `temper_box_kit::owned::{bounded, bounded_records, bounded_template, bounded_reference, deployment_profile}`, the cluster's rendering of its description and of its engine's, `w1`'s and `w2`'s documents, and the playground story's, cluster-turn's, turn-landing's, stock-landing's and GitHub engine's documents. Each takes the root as a path written into the documents, never opened, and returns what it returns today (`Bounded`, a document's value or its bytes). They read only committed files of the tree, and take every other fact (addresses, ports, ready captures, a provider key's presence) as a fixed value.
- **One writer:** where a slow test still needs a profile on disk, a laying function of the kit writes what its producer returned; no other code writes a profile.
- **The agent:** `temper_agent_fixtures` composes an agent's configuration bytes without writing, and `temper_worker_shell::agent::prepare_bytes_with(bytes, log, private_directory)` becomes public, the programs directory's verdict its third argument (`README.md`, 3.7). Production's `prepare_bytes` is unchanged.
- **Admission over values:** `temper_box_kit`'s admission helper takes a composed profile's document, `Files` and agent, and reads nothing; its file-reading form stays only while a slow caller needs it.
- Parts I to L add producers of the same kind for the fixtures they move, under the same rule.

### 2.2 The named-file reader (B; read by K)

`temper_service_shell::read_files_with_asked_using(names, asked, user, read) -> Result<Files, String>`, where `read: impl FnMut(&Path, u32, bool) -> Result<skein_lib::Secret, skein_shell::Unread>` takes a file's path, its bound and whether it is private. `read_files_with_asked` passes today's dispatch (`README.md`, 3.1). K's handover check passes a reader that records each path it is asked for.

### 2.3 The setting copies (I; read by K)

`agents_setting` and `local_setting` in `temper-box-shell`'s serve each gain a companion over the copy's read result, `std::io::Result<Option<Box<[u8]>>>` (`Ok(None)`: open found nothing), with the same log writer; they are reachable from `temper_box_shell::testing::serve`. Production reads within the same bounds as today (`README.md`, 3.4).

### 2.4 Names and modules

- A test moved to `temper-box::domain` keeps its name and its module's name: `suite::startup::turn_bounds` becomes `domain::startup::turn_bounds`, `suite::configuration::creating::…` becomes `domain::configuration::creating::…`, `suite::support_tests::…` becomes `domain::support_tests::…`. Where the target module holds a test of that name already, the moved one takes a suffix naming what it adds, and the report says so.
- A test moved to a shell's step tests is named for what it checks there; the part's report names it beside the slow test it holds.
- Several parts append to `tests/domain/startup.rs` and `tests/domain/support_tests.rs`, and delete from `tests/suite/startup.rs` and `tests/suite/support_tests.rs`: those conflicts are the merging part's to settle, keeping both sides.

### 2.5 What a part leaves to M

A part deletes a helper, a kit function or a fixture only when nothing at its base still calls it. One that a sibling's moved tests still call at that base stays, even when the sibling will remove its last caller; M removes whatever is left unused once all have landed, with the suite's and kit's module lists and `tools/tiers/layers.toml`'s comments for `temper-box::suite` and `temper-box::domain`.

## 3. The placement

Paths are relative to `temper/bin/temper-box/tests/suite/`. *Domain* is `temper-box::domain`; a crate's name is its library's step tests; *stays* keeps the test, or the row named, in the suite.

### 3.1 Admission, configuration, creating, provider admission

| Test | Part | Goes to, and what holds it |
|---|---|---|
| `admission_edges.rs`: `transcript_padding_covers_all_resume_window_remainders`, `turn_and_resume_bounds_admit_the_edge_and_refuse_one_byte_short` | J | Domain, over a composed profile and the service's checked start: all four remainders, all seven rows |
| `admission_edges.rs`: `obsolete_owned_organization_members_refuse_startup` | J | Deleted: `application_machine_is_refused_with_its_moved_destination` (`temper-engine-service-startup`) holds every assertion |
| `configuration.rs`: its ten tests | J | Domain's `configuration`, over composed process, Forgejo, cluster and turn-story profiles |
| `creating.rs`: its three tests | J | Domain's `configuration::creating`, keeping the restore, frame, codec and client path |
| `configuration/turn_landing.rs`: its two tests | J | Domain's `configuration::turn_landing`, keeping the agent's preparation |
| `provider_admission.rs`: `missing_provider_key_refuses_before_serving`, `missing_client_secret_refuses_with_a_valid_signing_key` | B | `temper-service-shell`, through the reader (2.2) |
| `provider_admission.rs`: `rsa_2048_is_refused_even_when_the_client_is_valid` | B | `temper-service` (or the engine component's step tests), through `from_checked` with the committed 2048-bit fixture key and a valid client |

### 3.2 Startup

| Test | Part | Goes to, and what holds it |
|---|---|---|
| `a_stock_policy_past_its_limit_refuses_before_listening` | — | Stays: one serve refusal through `main` |
| `rung_b0s_live_configuration_is_admitted_without_an_account_or_backend` | I | Domain, over B0's composed recipe on the sizing path, with the copies' results (2.3) |
| `a_held_login_return_hook_is_idempotent_and_refuses_unsafe_metadata_without_reading_the_file`, `a_held_login_import_waits_for_the_rust_file_lock`, `a_held_login_import_refuses_after_returning_without_starting_ctl`, `a_failed_terminal_return_allows_recovery_without_reopening_import`, `a_lent_record_returns_through_the_box_still_up_and_once` | — | Stay (`README.md`, 4) |
| `a_configuration_file_past_its_bound_is_refused` | E | `temper-box-shell`, over `read_with` and serve's wording |
| `a_secret_file_readable_by_others_refuses_the_start_naming_it` | B | `temper-service-shell`, the reader's unsafe result, in serve's words |
| `a_worker_root_reached_through_a_link_is_refused` | C | `temper-worker-shell`, the private root's verdict over metadata |
| `problems_reach_stderr_one_line_each`, `a_document_charter_is_refused_naming_temper_ctl_settings`, `an_inline_seed_policy_is_refused_naming_its_path`, `a_setup_limit_past_its_check_refuses_temper_serve_naming_both_limits` | I | `temper-box-shell`, serve's wording over the configuration's bytes |
| `an_address_with_no_token_in_the_directory_is_not_signed_in` | B | `temper-box-shell`, the token check over a read result, both rows |
| `a_box_on_a_fresh_init_reaches_serves_link_with_forge_3_dormant`, `a_box_adopts_an_inherited_clients_listener_without_binding_again`, `a_box_refuses_an_unmatched_inherited_listener_naming_its_address` | — | Stay |
| `a_nonempty_box_configuration_without_a_document_is_refused_naming_it`, `a_box_configuration_missing_members_is_refused_one_line_each`, `a_setup_naming_no_forge_refuses_the_box_before_it_starts` | E | `temper-box-shell`, the box's configuration read and planning |
| `the_owned_example_starts_without_credential_files`, `serve_prepares_eight_slots_and_ten_workspaces_on_the_owned_example`, `serve_sizes_its_worker_from_eight_setting_places_not_document_slots`, `serve_prepares_once_on_a_bare_init`, `serve_refuses_an_explicit_call_bound_below_the_agents_call_input`, `serve_checks_the_owned_agents_admission_at_each_bound`, `the_shipped_chat_is_ended_by_its_requester`, `shipped_bounds_fit_the_configured_charters_and_worker` | K | Domain, over composed owned and init profiles; every row |
| `serve_prepares_once_on_the_owned_example` | K | Deleted once `the_owned_example_starts_without_credential_files` is in the domain: it holds its one assertion |
| `every_command_checks_setup_before_reading_files` | F | `temper-box-shell` and `temper-engine-shell`, all five rows over the commands' entries; the `forge prepare` row stays |
| `a_bundle_directory_is_read_with_its_one_digest_directory`, `embedded_and_directory_files_make_the_same_bundle`, `a_bundle_reads_each_form_onto_its_file`, `the_compressed_forms_count_against_the_bounds`, `a_compressed_form_with_no_file_refuses_the_start`, `a_gz_of_another_type_refuses_the_start`, `a_bundle_page_without_a_head_refuses_the_start_for_its_metadata`, `a_directory_that_is_no_bundle_refuses_the_start` | D | `temper-engine-shell`, the bundle over supplied entries, both readers where the test has two; the page and directory refusals in serve's mapped words |
| `a_bundle_with_a_forge_sign_in_entry_starts_before_any_forge_is_kept` | K | Domain and `temper-engine-shell`: the sign-in's creation over its effects, then a checked start with the headed bundle and no kept forge |
| `programs_directories_are_checked_at_the_name_before_startup_effects` | C | `smith-agent-document` and `temper-box-shell`, the verdict over metadata for all five rows; the `link` row stays |
| `program_login_refuses_invalid_input_before_creation`, `program_login_checks_both_directory_names_before_creating_anything` | C | `temper-box-shell` and `smith-agent-document`, program login over its read and lookups, every row, no creation or exec on a refusal |
| `program_login_prepares_private_directories_through_each_supported_parent` | C | The same, both rows; the missing-programs row stays, for the kernel's exec refusal |
| `init_prints_written_files_account_guidance_and_explicit_start_flags_on_stderr`, `box_initializes_only_absent_or_empty_configuration_directories` | E | `temper-box-shell`, init's transcript over its scripted startup, and the box's first configuration over its read, every row |
| `serve_box_refuses_closed_or_non_pipe_descriptors_before_starting` | F | `temper-box-shell`, all six rows over per-descriptor facts; the closed-first row without a listener and the non-pipe-first row with one stay |
| `serve_claims_its_clients_listener_and_box_pipes_follow_it` | — | Stays |
| `machine_startup_leaves_unconnected_forge_handover_files_unread` | K | Domain, all nine inventories, through the reader (2.2) recording no read of a handover or its siblings |
| `the_store_page_holds_the_templates_largest_turn_frame`, `the_live_github_configuration_passes_startup_admission`, `the_reference_profile_passes_startup_admission`, `checked_in_deployment_profiles_pass_startup_admission`, `turn_bounds` | A, except K for the live GitHub configuration | Domain, over composed profiles; every profile and row |
| `the_template_agent_refuses_jigs_small_hub_turn_bound_before_serving` | I | `temper-engine-shell`, held limits' text over init's bare document, in serve's words |
| `forge_run_refuses_a_listener_at_the_wrong_address_before_taking_the_lock`, `forge_run_keeps_fd_three_through_the_version_spawn_and_exec`, `parent_guard_observes_its_recorded_process_after_reaping`, `observing_parent_descendants_leaves_cleanup_with_their_world`, `forge_parent_listener_helper`, `forge_run_ignores_its_childs_term_relays_the_listener_and_dies_with_its_status`, `killing_forge_run_alone_kills_its_forgejo_child`, `listener_refusals_name_the_fd_before_serve_or_a_fake_opens_files`, `forge_child_refuses_a_wrong_parent_or_missing_program_without_reading_a_document`, `a_fresh_box_offers_chatgpt_and_claude_with_no_claude_on_path`, `a_fetched_program_runs_through_its_launcher` | — | Stay |
| `forge_run_refuses_other_listener_counts_and_any_for_the_runner_before_opening_files` | F | `temper-engine-shell`, all five rows over the environment's rows, no read before the refusal; the Forgejo `0` row stays |
| `program_login_places_both_launcher_links_only_for_a_fetched_program`, `serve_places_links_only_for_a_fetched_program` | H | `temper-worker-shell`, `temper-launcher` and `temper-fetch-shell`, both rows each, over the fake links and the cache's facts |
| `serve_listening_requires_the_clients_listener_in_this_runs_log`, `ctl_link_sign_in_waits_for_serve_before_starting_its_bound`, `native_sign_in_keeps_its_start_bound_until_serve_listens` | G | Domain's `support_tests`, over the log's text and a recording gate |

### 3.3 Support

| Test | Part | Goes to, and what holds it |
|---|---|---|
| `story_command_a_callback_command_captures_output_and_keeps_a_bad_exit`, `story_socket_bounded_reads_preserve_the_next_bytes_and_eof_is_an_error`, the four `patience_…` tests, `cluster_stop_bounds_read_the_printed_seconds_and_need_a_complete_line`, `cluster_a_member_wait_can_defer_only_to_a_longer_world`, the two `forge_land_rung_…` tests, `github_file_scan_excludes_only_jigs_generation_secrets_store` | G | Domain's `support_tests`, over supplied facts (`README.md`, 3.8) |
| `cluster_passed_accepts_only_a_successful_settled_verdict` | G | Deleted: domain's registration of the same helper holds every assertion |
| `the_sign_in_fixture_reads_the_initialized_listeners_without_an_llm_member`, `moving_listeners_preserves_the_retained_files_explicit_keys`, `rebinding_the_fake_model_keeps_the_split_deployments_issuer`, `the_elsewhere_deployment_keeps_the_whole_fixture_provider_in_its_data_directory`, `cluster_turn_setup_declares_the_paced_gate_and_prepare_job`, `cluster_turn_settings_leave_spend_for_preparation_and_keep_current_fields`, `the_scripted_batch_fits_its_budget_and_call`, `native_forgejo_settings_preserve_the_coding_producer`, `owned_settings_preserve_the_ordinary_producer_without_review`, `native_landing_rules_replace_the_projects_stock_review`, `turn_restart_configuration_passes_startup_admission`, `batch_landing_configuration_passes_startup_admission`, `turn_configuration_declares_the_prepare_job`, `elsewhere_settings_follow_the_connected_project`, `ctl_limit_settings_apply_both_kinds_in_one_revision` | L | Domain's `support_tests`, over composed fixtures, addresses as values and captured settings; the two elsewhere tests keep their Linux gate |
| `fixture_limits_are_applied_before_the_subprocess_restart`, `ctl_a_failed_ctl_check_reaches_the_world_after_cleanup`, `ctl_a_returned_visit_drains_without_another_process_or_a_poll` | — | Stay |

## 4. Order

- **At once:** A, B, C, D, E, F, G, H.
- **After A:** I and J; **after A, B and I:** K; **after J:** L, which shares the batch and turn fixtures J composes first.
- **After all:** M.
