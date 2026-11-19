//! Whole change panels at their receiving bounds.
use super::limits;
use skein_lib::List;
use temper_client_domain::forge;
use temper_client_protocol_wire::{self as wire, v1 as w};
/// Every heap field at its receiving bound, with optional facts populated.
#[must_use]
pub fn panel(task: u64, step: w::Step) -> w::ChangePanel {
    let limits = forge::wire_limits(&limits().forge).expect("wire limits");
    let mut gates = List::with_capacity(wire::GATES);
    for number in 1..=wire::GATES {
        gates
            .push(
                w::Gate {
                        number: u64::from(number),
                        kind: w::GateKind::Review,
                        blocks: true,
                        verdict: w::Verdict::Approved,
                    },
            )
            .expect("bounded gates");
    }
    w::ChangePanel {
            wait: None,
            source: None,
            way: w::Way::Queue,
            in_landing: None,
            task,
            forge: 13,
            owner: field(b'o', limits.change_panel_owner),
            name: field(b'n', limits.change_panel_name),
            branch: field(b'b', limits.change_panel_branch),
            landing: field(b'l', limits.change_panel_landing),
            head: Some([0x39; 32]),
            pull: Some(
                w::Pull { number: 23, address: field(b'a', limits.pull_address) },
            ),
            step,
            hold: Some(w::Hold::Moved),
            runs: Some(29),
            ci_required: true,
            ci: w::Ci::Passed,
            approvals: List::with_capacity(0),
            unmapped_reviews: List::with_capacity(0),
            gates,
            repairs: w::Used { used: 1, limit: 2 },
            resolutions: w::Used { used: 3, limit: 4 },
            updates: w::Used { used: 5, limit: 6 },
            questions: w::Questions {
                    route: w::Route::None,
                    taken: w::Used { used: 0, limit: 4 },
                    open: List::with_capacity(0),
                },
        }
}
#[must_use]
pub fn queued() -> w::Step {

    let mut ahead = List::with_capacity(wire::TASKS_AHEAD);
    for task in 1..=wire::TASKS_AHEAD {
        ahead.push(u64::from(task)).expect("tasks ahead");
    }
    w::Step::Queued(w::Queued { place: 65, ahead })
}

#[must_use]
pub fn field(byte: u8, count: u32) -> Box<[u8]> {
    let mut bytes = skein_lib::bytes::zeroed(usize::try_from(count).unwrap());
    bytes.fill(byte);
    bytes
}
