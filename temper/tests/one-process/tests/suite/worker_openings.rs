//! Current engine and a worker at its layouts (temper/docs/design/protocol/wires.md, 11).
use temper_one_process_world::worker_openings;

#[test]
fn a_goal_lands_over_the_worker_link() {
    worker_openings::goal();
}

#[test]
fn a_renewed_identity_reaches_the_worker_over_its_link() {
    worker_openings::renewal();
}
