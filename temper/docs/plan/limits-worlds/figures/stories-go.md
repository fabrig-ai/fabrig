# stories-go: holders before removal

2026-11-07. Unchanged task base `fcf5f044b1dfce231a813a27e8adc795ef0b1b1c`.
Every holder below ran separately, by its full name, before any story or helper was removed:
`heavy cargo nextest run -p PACKAGE --profile PROFILE -E 'test(=NAME)'`.
Each run selected exactly one test and passed (exit 0).

| Package | Profile | Test | Result |
|---|---|---|---|
| `temper-client-tools` | `default` | `tests::jig::wait::both_requirements_must_hold_on_the_same_delivery` | 1 selected, 1 passed |
| `temper-real-loop-world` | `default` | `github_land::a_chats_change_lands_on_connected_github` | 1 selected, 1 passed |
| `temper-real-loop-world` | `fuzzy` | `fuzzy::landing::a_chat_keeps_its_live_delegate_until_the_change_lands` | 1 selected, 1 passed |
| `temper-engine-domain-world` | `default` | `root_routes::an_agent_changes_a_live_delegates_controls_and_replays_the_answer` | 1 selected, 1 passed |
| `temper-real-loop-world` | `fuzzy` | `fuzzy::case_4_1_4_3::the_opener_and_the_two_hour_check_run_on_three_workers` | 1 selected, 1 passed |
| `temper-engine-domain-world` | `default` | `root_routes::timer_cancellation::an_armed_timer_survives_a_fresh_director_run_and_its_helpers_cancellation` | 1 selected, 1 passed |
| `temper-engine-domain-world` | `default` | `root_routes::a_cancel_closes_three_levels_with_runs_live_deepest_first` | 1 selected, 1 passed |
| `temper-engine-domain-world` | `default` | `root_routes::a_note_written_corrected_by_a_person_and_recalled_is_durable` | 1 selected, 1 passed |
| `temper-real-loop-world` | `fuzzy` | `fuzzy::case_4_6::a_restarted_workstation_keeps_the_directors_question_and_answers_it_once` | 1 selected, 1 passed |
| `temper-real-loop-world` | `fuzzy` | `fuzzy::case_5_4::a_new_direction_cancels_x3_under_its_closed_pull_and_bs_landing_news_waits_for_its_release_with_an_overlap_wake_and_a_closing_comment` | 1 selected, 1 passed |
| `temper-real-loop-world` | `fuzzy` | `fuzzy::case_6_4::a_red_tip_pauses_kits_queue_for_one_repair_that_lands_before_a4_while_the_directors_interim_timer_waits_without_adopting_a4` | 1 selected, 1 passed |
| `temper-real-loop-world` | `fuzzy` | `fuzzy::case_5_1::a_held_director_keeps_the_owners_question_and_its_fresh_run_reads_the_new_rules` | 1 selected, 1 passed |
| `temper-real-loop-world` | `fuzzy` | `fuzzy::case_6_5::a_budget_question_climbs_two_owners_to_the_persons_inbox_and_a_raised_investigation_hands_its_report_to_an_advisor_without_the_director_asking_the_person` | 1 selected, 1 passed |
| `temper-engine-domain-world` | `default` | `root_routes::added_dependencies::one_batch_of_four_resolves_its_local_references_and_gates_only_the_fourth` | 1 selected, 1 passed |
| `temper-engine-domain-world` | `default` | `root_routes::added_dependencies::both_prerequisite_orders_and_completion_orders_gate_only_the_fourth_delegate` | 1 selected, 1 passed |
| `temper-real-loop-world` | `fuzzy` | `fuzzy::case_6_6::a_kit_change_lands_before_the_apps_early_repin_without_relay_without_the_a3_a4_epilogue` | 1 selected, 1 passed |
| `temper-engine-domain-world` | `default` | `root_routes::added_dependencies::a_live_delegate_is_kept_until_both_added_dependencies_are_met` | 1 selected, 1 passed |
| `jig-core-world` | `default` | `stories::dependencies::a_kept_delegate_is_woken_by_its_last_dependencys_met` | 1 selected, 1 passed |
| `temper-engine-domain-world` | `default` | `root_routes::added_dependencies::a_kept_delegate_resumes_with_its_committed_turns_and_its_journal_whole` | 1 selected, 1 passed |

The repeated references in the plan share these results: the kept landing holds
both landing and 6.2; 4.6 holds both cancellation's recall and resumed history;
6.5 holds both 6.1's hierarchy and 6.2's question path; 6.6 holds both the
batch-local path and complete journals. The two new lower-stories holders are
present in `temper/tests/engine/domain/tests/suite/root_routes/added_dependencies.rs`
and passed above.

The unchanged base's complete fuzzy run also passed: **22 selected, 22 passed**
(`heavy cargo nextest run -p temper-real-loop-world --profile fuzzy`).
No holder was missing or failing, so all four planned stories can go.

## After removal

- `heavy cargo nextest run -p temper-real-loop-world --lib`: 189 passed.
- `heavy cargo clippy -p temper-real-loop-world --all-targets -- -D warnings`: passed.
- `heavy cargo nextest run -p temper-real-loop-world`: 215 selected, 215 passed.
- `heavy cargo nextest run -p temper-real-loop-world --profile fuzzy`: **18 selected, 18 passed**, four fewer than the base's 22.
- `heavy cargo check -p temper --tests`: passed; the process-tier readers compile, without running that tier.
- `heavy cargo nextest run -p temper`: 491 selected, 491 passed (4 slow).
- The temper old-form recount lowered `comment:genuine` from 9 to 8; every other count stayed the same.
- `python3 -I -B tools/citations/citations.py --base origin/main`: zero new broken citations, zero total.

Shared readers keep `batch_member`, `batch_held`, `BATCH_WRITING`,
`batch_writing`, `batch_journal`, `batch_journal_pages`, `batch_done`,
`blocker_delivery` and the `batch_tests` message/journal builders in place.
Every `Observed` field still has surviving readers. `KeptTask` had no surviving
reader, so its response condition and its regression test went together.
The landing module keeps its live-delegate story, and both recorded-twin fixture
directories stay untouched. No test moved and no new test was added.
