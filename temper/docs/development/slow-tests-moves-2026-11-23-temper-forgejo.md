# temper's real-Forgejo stories moved below

2026-11-23. The record that `temper/docs/design/draft/slow-tests-down.md`, 3.2 asks of its goal D2 (5.1, "temper's real Forgejo paths" and "temper's repeated command paths"): each story of `temper-box::process` and `temper-box::proof_live` that the goal deletes, shortens or folds, and beside each of its assertions the lower test that holds it. It was read again at `b574a13436`, from the survey `slow-tests-survey-2026-11-23-temper-1.md`, groups 1 and 6, and four read-only surveys of that commit. *Added* marks a holder the goal's own change adds; every other holder already ran at that commit. Binaries are named as `tools/tiers/layers.toml` names them; paths are under `temper/` unless they name their tree.

What stays above is `temper/docs/design/testing/process-tier.md`'s: one real owned-Forgejo landing (the first owned repository's CSV story), a real Forgejo's signed deliveries, the owned Forgejo's preparation, runner, dump and restart witnesses, the sign-in stories on a real Forgejo, the shortened stories below, and B0's twin.

## 1. A person's change landed and found after a crash

`a_persons_change_lands_on_the_forge_and_is_found_after_a_restart`: **gone**, with its `landing` helper.

| Assertion | Holder |
|---|---|
| A repository adopted through the native client, the forge's row connected as temper | `temper-client-world::suite` `a_repository_adopted_from_the_navigation_link_is_held_by_the_engine`; `temper-engine-domain-world::suite` `an_adoption_on_a_configured_forge_is_adopted_with_what_temper_may_do` |
| Signed in by the printed link, the deployment's key added and its account usable, a first chat answered | `temper-client-world::suite` `adding_a_key_reaches_ready`; `temper-one-process-world::suite` `a_person_signs_in_by_the_printed_link_and_a_chat_is_answered` |
| The change pushed by git, its pull merged, `main`'s README ending in the agent's edit | `temper-one-process-world::suite` `an_adopted_repository_gets_its_change_pushed_through_git` and `story_a_is_walked_from_the_welcome_to_the_csv_change_landed`; `temper-system-world::suite` `a_stock_chats_delegate_reports_the_landed_pull_and_merge_without_invented_checks` (exact README at `main`) |
| CI passed at the merged pull's head, read through Forgejo's API; real git, runner, merge and `main` | `temper-box::process` `a_csv_change_passes_ci_and_a_blocking_agent_review_on_the_owned_forge_then_lands_and_survives_restart`, which reads Forgejo's merged pull, its head's status and `main`'s file apart from temper (*added*) |
| With the keep's polls and the change's stall past the budget and the job outlasting `settle`, the change lands only because the job's end is delivered | `temper-engine-protocol-forge-world::suite` `a_slow_job_lands_its_change_only_by_its_delivered_end` (*added*: lands with the delivery, not by the cutoff without it); `temper-engine-forge-world::suite` `a_job_end_hint_makes_pending_ci_wait_a_hinted_read` |
| The owned Forgejo sends that job's end to temper's hook | `temper-box::process` `a_push_and_its_jobs_end_are_delivered_signed_with_the_secret_reconcile_drew` (*added* to the signed-push witness) |
| The whole serve tree killed, nothing of it left, started again on the same directory | `temper-box::process` `a_killed_serve_keeps_its_port_and_prints_a_fresh_link`; `a_killed_box_takes_its_steps_with_it` |
| The work found after that crash: the chat, the goal and its plan, the same merged pull | Story A's end, `temper-one-process-world::suite` (*added*: every process crashed after the landing, the person reads the landed task, its pull and merge as before, and the fake forge's ledger has no further push or merge); `temper-engine-domain-world::suite` `a_cleared_result_reaches_its_requester_and_the_ctl_document_after_restart`; `temper-engine-forge-world::suite` `a_merge_with_its_answer_lost_is_found_after_restart_without_merging_twice` |

## 2. The plan face's landing and restart

`a_persons_plan_change_lands_on_the_owned_forge_and_is_found_after_a_restart`: **shortened** to `a_persons_plan_is_connected_and_answers_a_chat_with_its_bearer_on_every_request`, which keeps the face: printed-link sign-in, `me`, `account connect 1` through the fake issuer, `account list --until settled` with the plan usable, a chat's reply on gpt-6-luna, the plan's bearer on every model request, catalogue reads included, and the box's exit, link and tree.

| Assertion | Holder |
|---|---|
| `temper-box init --box`, the owned forge set up from the client, `work/hello` made and adopted, cache, version, listener and handover checks in both sittings | the CSV story (the same setup checks, both sittings) |
| The plan usable in a fresh client after a restart, with no reconnect | `temper-engine-domain-world::suite` `a_connected_plan_stays_usable_after_a_restart_without_reconnecting` (*added*) |
| The chat delegates the change; the coder edits on the plan's grant; the starter workflow passes; the change lands with no review | `temper-engine-domain-world::suite` `a_landed_chats_reply_is_committed_once_and_nothing_reruns_after_a_restart` (*added*, the producer's runs on the plan's grant); `temper-system-world::suite` `a_stock_chats_delegate_reports_the_landed_pull_and_merge_without_invented_checks`; the CSV story's `assert_task`, real read, edit and workflow |
| The landing's reads in both sittings alike: task, panel, pull 1, the whole merge commit, CI at its head | the CSV story (same task, head and merge in its second box); `temper-client-world::suite` `task_show_reads_a_cleared_change_before_and_after_restart` and `late_open_landed_change_keeps_its_final_panel` |
| The landing reply committed once in both sittings, the restart's reply once; the same ended delegates and no task beyond the chat, its change and its producer; no producer or reviewer model call after the restart | `a_landed_chats_reply_is_committed_once_and_nothing_reruns_after_a_restart` (*added*) |
| Close ends the chat, no live root is left, an unknown task is refused | the same test (*added*); `temper-client-world::suite` `a_new_chat_lands_its_delegated_change_and_closes_in_your_chats` |
| Paging the chat's activity backwards across pages | `temper-client-tools` lib `a_closed_chat_fills_its_conversation_across_activity_pages` |
| The peer's bearer on every request, the coder's included | the shortened story (the chat's requests and catalogue reads); the producer's account, the added domain test above |

## 3. A playground's copy read by the worker

`playground_setup_adopts_widget_and_the_worker_reads_its_source_on_the_owned_forge`: **gone**, with `assert_playground_setup` and the kit's playground-read helpers.

| Assertion | Holder |
|---|---|
| `forge setup --playground` on a real box, exit and tree | `temper-box::process` `a_hangup_stops_a_box_and_leaves_nothing_running` and `a_killed_box_takes_its_steps_with_it` (the same command, its cache unchanged, the box's exit and tree) |
| ctl's answer: `project: 1 · widget`, adopted owned, the project's home; the forge's row connected, made, `playground: true` | `temper-client-tools` lib `playground_setup_prints_its_project_home_and_a_playground_row` (*added*), beside `forge_setup_without_a_number_sends_a_new_owned_forge` and `made_sends_the_home_adoption_with_its_stable_key_and_prints_its_answer` |
| The copy's README, workflow and source bytes | `temper-engine-service-component` lib `playground_create_copies_the_fake_source_beside_the_starter_and_keeps_forgejos_readme` |
| A real Forgejo takes that copy, `src/main.rs` readable on `main` | `temper-box::proof_live` `rung_b0s_walk_passes_offline_on_its_twin` (its setup adopts `playground/widget`) |
| The worker reads the adopted repository's files whole | `temper-system-world::suite` `an_adopted_repository_is_prepared_by_the_worker_and_mounted_in_smith`; the CSV story's real checkout and reads |
| The pull's address on the owned forge, landing branch `main` | `temper-client-tools` lib `checking_text_and_json_name_pull_head_gates_bounds_and_same_digest`; `temper-engine-service-vocabulary` lib `a_panels_pull_address_uses_its_configured_forge_kind_and_base` |
| `project show 1 --json`: one owned home at its forge and repository path | `temper-client-tools` lib `project_show_names_an_owned_home_by_its_forge_and_repository_path` (*added*); `temper-engine-domain-world::suite` `an_adopted_repository_is_the_projects_home_after_restart` |

## 4. A Forgejo elsewhere landing a chat's change

`a_forgejo_elsewhere_connected_as_its_admin_makes_the_first_repository_lands_a_chats_change_and_is_found_after_a_restart`: **shortened** to `a_forgejo_elsewhere_connected_as_its_admin_makes_its_first_repository_and_is_found_after_a_restart`, which keeps the real admin token's private paste, `forge connect ... --first`, the managed, connected and made row, the first repository's hook at the listener, the provider's settings applied, one reopening with no paste that reads the same row and repository, the provider's whole value and its startup copy, the hook made again after its removal by hand, `main` unchanged and no pull, the admin still authenticating, and the scans of every output and file.

| Assertion | Holder |
|---|---|
| A chat's change landed: pull 1, the whole merge, CI at its head, the README edit on `main` | `temper-system-world::suite` `a_stock_chats_delegate_reports_the_landed_pull_and_merge_without_invented_checks`; the CSV story |
| After the restart: the same merged pull, one repository and one pull, `main` unchanged | `temper-engine-forge-world::suite` `a_merge_with_its_answer_lost_is_found_after_restart_without_merging_twice`; the shortened story (`main` unchanged, no pull) |
| The ended result read after the restart | `temper-engine-domain-world::suite` `a_cleared_result_reaches_its_requester_and_the_ctl_document_after_restart` |
| The project's landing rule (`enforce`, `ci`, `up_to_date`, no gate) and producer charter read back as changed, in both lives | `temper-engine-domain-world::suite` `a_projects_changed_rule_and_charter_are_read_after_a_restart` (*added*) |
| `other/open` and `other/closed` adopted with `--no-home`, both able to push; their collaborators and hooks removed by hand and made again on reopening | `temper-engine-service-component` lib `a_managed_forge_reopened_from_its_record_keeps_its_first_repository_provider_and_access` (*added*); `a_managed_adoption_on_this_host_grants_write_and_makes_one_hook_at_the_listener`; skein's `skein-forge-setup-world::suite` `a_collaborator_removed_is_made_again` and `a_deleted_hook_is_made_again`; the shortened story's one real hook made again |
| The kept provider handed to the shell again on reopening | the same added service test; the shortened story (the whole value read back, and its startup copy) |

## 5. Repeated command paths

| Story | What happens | Holders |
|---|---|---|
| `an_older_recorded_runner_has_its_files_prepared_again_without_a_dump` | **gone** | `temper-forge-setup` lib `the_upgrade_prepares_an_older_runner_again_without_a_dump` and `the_upgrade_registers_an_older_runner_missing_its_configuration_without_a_dump` (*added*: the configuration written again byte for byte from its registration, the pinned state, no dump or backup); `temper-box::process` `a_prepared_forge_runs_forgejo_and_its_runner_once_each` and `an_older_recorded_forgejo_is_dumped_then_upgraded` for the real runner and dump |
| `ctl_from_the_deployments_client_signs_in_to_an_engine_and_reads_me` | **folded** with the checkpoint wrapper | `ctl_from_the_deployments_client_signs_in_reads_me_and_takes_a_checkpoint_while_the_deployment_serves`: the sibling `temper` present, three calls each exiting 0, `me`'s numeric party, and every checkpoint assertion |
| `an_owned_forgejo_signs_its_member_in_through_temper` and `an_owned_forgejo_accepts_a_public_source_once_and_exchanges_its_code` | **folded** into one Forgejo, its serve started again for each source's configuration | `an_owned_forgejo_signs_its_people_in_through_temper`: the public source's legs, then the confidential source's: `forge sign-in`'s secret on no output, the printed command adding the source, the member `sam` made with `3:sam@example.com`, the links-only restart (linked member signs in, no registration, both signup posts 403, an unlinked account linked by its password), the spent link refused and its replacement taken, and the admin's connect giving forge 1 managed and the page's meta with no restart |
| `rung_b0s_setup_reaches_account_import_with_the_fake_model_and_no_account` | **folded** into the twin | `rung_b0s_walk_passes_offline_on_its_twin`, whose offline face checks at the import's boundary: no party account, no model request, the project and roles funded at least at the rung's figures; and the box's tree empty at its exit |

## 6. What the goal adds below, by layer

- **step:** the client tools' two playground and project cases; the forge setup's older-runner case; the service's managed reopening.
- **domain:** the engine domain's plan restart, landed chat restart and changed rule and charter cases.
- **protocol:** the protocol-forge world's delivered job end.
- **simulated:** Story A's crash and readback.
- **end-to-end** (compiled, run after landing): the signed job end, the CSV story's Forgejo reads, and the shortened and folded stories.

The goal's close measures each layer against main (`tools/tiers/budgets.md`, 8): simulated has no accepted level, so a growth past its 2% is brought to the user with the gate's figures.
