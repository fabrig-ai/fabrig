//! Ready's answered trial through the hosted Claude Code backend.

#[test]
fn a_welcome_trial_uses_one_stock_claude_turn_and_ends() {
    temper_one_process_world::held_program::welcome_trial();
}
