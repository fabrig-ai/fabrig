//! One person-story inventory on every front (jig/docs/design/client/domain/testing.md, 4–5).
/// Run the person stories, with each front's explicitly named differences.
#[macro_export]
macro_rules! person_story_suite {
    (@test $make:path, $name:ident, $module:ident, $facts:expr, [$($attribute:meta)*]) => {
        #[test]
        $(#[$attribute])*
        fn $name() {
            let mut world = $make($facts);
            $crate::stories::shared::$module::$name(&mut world);
        }
    };
    ($make:path; release: [$($release:meta)*], compound_release: [$($compound_release:meta)*], inbox_chat: [$($inbox_chat:meta)*], close: [$($close:meta)*], family: [$($family:meta)*] $(,)?) => {
        $crate::person_story_suite!(@test $make, a_draft_is_kept_across_a_reload, b_conversations, $crate::stories::shared::b_seed::Facts::Goals, []);
        $crate::person_story_suite!(@test $make, a_reload_with_requests_pending_sends_them_again_under_their_keys, b_conversations, $crate::stories::shared::b_seed::Facts::Goals, []);
        $crate::person_story_suite!(@test $make, a_dropped_watch_shows_the_page_behind_until_it_reopens, b_conversations, $crate::stories::shared::b_seed::Facts::Goals, []);
        $crate::person_story_suite!(@test $make, a_page_left_with_reads_in_flight_drops_their_answers, b_conversations, $crate::stories::shared::b_seed::Facts::Goals, []);
        $crate::person_story_suite!(@test $make, the_conversations_section_lists_live_chats_by_last_activity, b_conversations, $crate::stories::shared::b_seed::Facts::Goals, []);
        $crate::person_story_suite!(@test $make, a_person_s_chats_list_live_then_ended_by_last_activity, b_conversations, $crate::stories::shared::b_seed::Facts::Goals, []);
        $crate::person_story_suite!(@test $make, an_engine_outside_the_client_s_versions_shows_out_of_date, b_conversations, $crate::stories::shared::b_seed::Facts::Goals, []);
        $crate::person_story_suite!(@test $make, starting_a_chat_at_the_text_shell, b_conversations, $crate::stories::shared::b_seed::Facts::Goals, []);
        $crate::person_story_suite!(@test $make, moving_a_goal_up_reorders_the_board, b_goals, $crate::stories::shared::b_seed::Facts::Goals, []);
        $crate::person_story_suite!(@test $make, the_project_overview_shows_the_board_s_first_goals, b_goals, $crate::stories::shared::b_seed::Facts::Goals, []);
        $crate::person_story_suite!(@test $make, the_frame_shows_no_conversations_entry_without_a_chat_template, b_goals, $crate::stories::shared::b_seed::Facts::NoChatTemplate, []);
        $crate::person_story_suite!(@test $make, a_goal_beyond_the_allotment_is_proposed_as_the_confirmation_said, b_goals, $crate::stories::shared::b_seed::Facts::GoalOffer { procedure: false }, []);
        $crate::person_story_suite!(@test $make, a_procedure_only_goal_form_confirms_the_supplied_executor, b_goals, $crate::stories::shared::b_seed::Facts::GoalOffer { procedure: true }, []);
        $crate::person_story_suite!(@test $make, a_held_chat_is_released, b_effects, $crate::stories::shared::b_seed::Facts::Goals, [$($release)*]);
        $crate::person_story_suite!(@test $make, a_held_chat_is_released_at_the_text_shell, b_effects, $crate::stories::shared::b_seed::Facts::Goals, [$($release)*]);
        $crate::person_story_suite!(@test $make, a_person_s_words_are_shown_as_sent, b_effects, $crate::stories::shared::b_seed::Facts::Goals, []);
        $crate::person_story_suite!(@test $make, words_to_a_stopped_task_release_it_together, b_effects, $crate::stories::shared::b_seed::Facts::Goals, [$($compound_release)*]);
        $crate::person_story_suite!(@test $make, the_engine_restarting_under_a_pending_request_answers_it_once, b_effects, $crate::stories::shared::b_seed::Facts::Goals, []);
        $crate::person_story_suite!(@test $make, a_proposal_accepted_in_the_inbox_leaves_its_chat_too, b_proposals, $crate::stories::shared::b_seed::Facts::ProposedGoal, [$($inbox_chat)*]);
        $crate::person_story_suite!(@test $make, two_tabs_deciding_one_thing_are_told_who_decided_first, b_proposals, $crate::stories::shared::b_seed::Facts::ProposedGoal, []);
        $crate::person_story_suite!(@test $make, closing_a_chat_cancels_one_delegate_and_moves_the_other_to_the_person, b_proposals, $crate::stories::shared::b_seed::Facts::Delegates, [$($close)*]);
        $crate::person_story_suite!(@test $make, a_person_s_part_request_is_answered_from_the_connector_s_outcome, b_family, $crate::stories::shared::b_seed::Facts::Goals, [$($family)*]);
    };
}
