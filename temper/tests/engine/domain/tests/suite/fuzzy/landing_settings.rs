//! Restored landing settings and operator holds in the turn's engine world
//! (temper/docs/design/domain/landing-settings.md, 1.1 and 1.3).
#[test]
fn landing_values_wait_for_the_next_start_then_prove_the_turn() {
    crate::forge_routes::landing::go_live::a_project_with_its_own_roles_goes_live_by_turn_with_running_stays();
}
#[test]
fn the_operator_holds_queue_and_turn_until_restart() {
    crate::forge_routes::landing::the_operator_hold_waits_by_queue_and_turn_until_restart();
}
#[test]
fn the_operator_holds_the_goal_closers_land_step() {
    crate::forge_routes::goal_closer::the_operator_hold_stops_the_goal_closers_land_step();
}
