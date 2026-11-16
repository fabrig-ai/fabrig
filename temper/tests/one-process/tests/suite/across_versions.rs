//! Current engine and a worker at kept wire versions (temper/docs/design/protocol/wires.md, 11).
use temper_one_process_world::across_versions;

#[test]
fn a_goal_lands_across_wire_versions() {
    across_versions::goal();
}

#[test]
fn a_renewed_identity_reaches_the_worker_at_compatible_wire_versions() {
    across_versions::renewal();
}
