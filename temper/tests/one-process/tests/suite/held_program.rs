//! Orderly program stop (temper/docs/design/testing/tiers.md, 1.5).
#[test]
fn a_service_stops_its_held_claude_turn_before_ending() {
    temper_one_process_world::held_program::stop();
}
