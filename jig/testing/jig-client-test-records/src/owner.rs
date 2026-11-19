use jig_client_protocol_wire::Record as _;
use super::{List, chip_from_rows, chip_root_fact, frame_record, task_record, w, wire_owner_chip_details_row};
use alloc::boxed::Box;
extern crate alloc;

#[must_use]
pub fn frame_two_projects_record() -> w::Me {
    let mut parts = frame_record();
    let mut identity = parts.party;
    identity.login = w::DocumentText { bytes: Box::from("é".as_bytes()), cut: true };
    parts.party = identity;
    let mut projects = List::with_capacity(2);
    for (project, role) in [(3, w::HeldRole::Owner), (5, w::HeldRole::Observer)] {
        projects
            .push(
                w::MeProject {
                        left_out: 0,
                        defaults: List::with_capacity(0),
                        public: None,
                        templates: Box::from([]),
                        creating: None,
                        project,
                        name: w::DocumentText { bytes: Box::from(*b"Tally"), cut: false },
                        role,
                        pool: None,
                        models: List::with_capacity(0),
                    },
            )
            .expect("literal fixture fits");
    }
    let mut size = 0_u32;
    for project in &projects {
        size = size.checked_add(project.measure(&crate::limits()).expect("bounded fixture")).expect("fixture sizes").checked_add(4).expect("piece header");
    }
    let mut out = skein_lib::Writer::new(skein_lib::length::usize_of(size));
    for project in &projects {
        let mut encoded = skein_lib::Writer::new(skein_lib::length::usize_of(project.measure(&crate::limits()).expect("bounded fixture")));
        project.write(&crate::limits(), &mut encoded).expect("fixture encode");
        skein_codec::piece::write(&mut out, &encoded.finish()).expect("fixture piece");
    }
    parts.projects = out.finish();
    parts
}
#[must_use]
pub fn owner_phase_records() -> [w::ChipPhase; 6] {
    [
        w::ChipPhase::Waiting(
            w::ChipPhaseWaiting { dependencies: List::with_capacity(0), more: false, holds: false },
        ),
        w::ChipPhase::Active(
            w::ChipPhaseActive { value: w::RunStage::Idle },
        ),
        w::ChipPhase::Person(
            w::ChipPhasePerson {
                    address: w::PersonAddress::Role(
                        w::PersonAddressRole { value: 17 },
                    ),
                    taken_by: Some(19),
                    since: Some(23),
                },
        ),
        w::ChipPhase::Closing(
            w::ChipPhaseClosing { value: w::ClosingStage::Effects },
        ),
        w::ChipPhase::Held(
            w::ChipPhaseHeld {
                    reason: w::HoldCategory::Uncertain(
                        w::HoldCategoryUncertain { entry: 29 },
                    ),
                    revision: 31,
                },
        ),
        w::ChipPhase::Ended,
    ]
}
#[must_use]
pub fn owner_chip_record(phase: w::ChipPhase, compact: w::Phase) -> w::Chip {
    let mut root = match chip_root_fact() {
        w::Fact::Chip(value) => value,
        w::Fact::ChipPhase(_)
        | w::Fact::Closing(_)
        | w::Fact::Ending(_)
        | w::Fact::HeldRuntime(_)
        | w::Fact::HoldReason(_)
        | w::Fact::ChipDetails(_)
        | w::Fact::ChipEnding(_)
        | w::Fact::ResultValue(_) => unreachable!("literal chip root"),
    };
    let mut fields = root.value;
    fields.phase = compact;
    root.value = fields;
    chip_from_rows(&[
        w::Fact::Chip(root),
        w::Fact::ChipPhase(
            w::ChipPhaseRow {
                    parent: Some(0),
                    value: w::ChipPhaseOwnerFields { phase, details: 2 },
                },
        ),
        w::Fact::ChipDetails(Box::new(wire_owner_chip_details_row())),
    ])
}
#[must_use]
pub fn owner_waiting_chip_record() -> w::Chip {
    owner_chip_record(owner_phase_records().into_iter().next().expect("literal fixture fits"), w::Phase::Waiting)
}
#[must_use]
pub fn task_with_hold_record() -> w::Task {
    let mut parts = task_record();
    parts.chip = owner_waiting_chip_record();
    parts.hold = Some(hold_record());
    parts
}
#[must_use]
pub fn hold_record() -> w::ChipHold {
    w::ChipHold {
            words: None,
            placed: None,
            circle: None,
            account: None,
            source: None,
            reason: w::HoldCategory::Uncertain(
                w::HoldCategoryUncertain { entry: 29 },
            ),
            revision: 31,
            before: w::PriorPhase::Waiting,
            escalation: None,
            tries: w::FailureTries {
                    transient: 2,
                    permanent: 3,
                    run: 5,
                    agent: 7,
                    lost: 11,
                    invalid: 13,
                    infrastructure: 17,
                },
            hold_wait_since: Some(17),
        }
}

#[must_use]
pub fn owner_chip_records() -> [w::Chip; 6] {
    let mut phases = owner_phase_records().into_iter();
    [w::Phase::Waiting, w::Phase::Idle, w::Phase::Active, w::Phase::Closing, w::Phase::Held, w::Phase::Ended]
        .map(|compact| owner_chip_record(phases.next().expect("six literal phases"), compact))
}
