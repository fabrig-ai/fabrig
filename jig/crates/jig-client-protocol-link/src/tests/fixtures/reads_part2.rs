use super::*;
pub fn read_wire_impact() -> w::Impact {
    let lim = w::CEILINGS;
    w::Impact::new(&lim, w::ImpactParts { tasks: read_wire_phase_counts(), live_runs: 7, proposals: 7 }).unwrap()
}

pub fn read_wire_phase_counts() -> w::ImpactCounts {
    let lim = w::CEILINGS;
    w::ImpactCounts::new(
        &lim,
        w::ImpactCountsParts {
            ended: 7,
            waiting: 7,
            running: 7,
            held: 7,
            closing: 7,
            done: 7,
            failed: 7,
            cancelled: 7,
        },
    )
    .unwrap()
}

pub fn read_wire_owner_escalation() -> w::Escalation {
    jig_client_test_records::read_wire_owner_escalation()
}

pub fn read_wire_question() -> w::Question {
    jig_client_test_records::read_wire_question()
}
