//! Reviewed program landing exceeds the focused guide (temper/docs/design/testing/tiers.md, 1.5).

#[test]
fn a_program_chat_resumes_after_its_reviewed_edit_lands() {
    temper_one_process_world::held_program::landing::reviewed_edit();
}
