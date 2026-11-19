use super::*;
pub fn hold_refusal(value: w::HoldCategory) -> w::Refusal {
    let reason = Some(w::CurrentHold::Reason(
        w::CurrentHoldReason { value },
    ));
    let what = w::Moved::Hold(w::MovedHold { reason });
    w::Refusal::Moved(w::RefusalMoved { now: revision(31), what })
}
pub fn holds() -> [(p::HoldReason, w::HoldCategory); 22] {
    let limits = wire();
    [
        (p::HoldReason::Stalled, w::HoldCategory::Stalled),
        (p::HoldReason::Stopped, w::HoldCategory::Stopped),
        (p::HoldReason::Drift, w::HoldCategory::Drift),
        (p::HoldReason::Effects, w::HoldCategory::Effects),
        (p::HoldReason::EffectFailed, w::HoldCategory::EffectFailed),
        (p::HoldReason::Budget, w::HoldCategory::Budget),
        (p::HoldReason::Deadline, w::HoldCategory::Deadline),
        (p::HoldReason::HoldsWaited, w::HoldCategory::HoldsWaited),
        (p::HoldReason::ResourceUnavailable, w::HoldCategory::ResourceUnavailable),
        (p::HoldReason::ResourceAccess, w::HoldCategory::ResourceAccess),
        (p::HoldReason::Procedure, w::HoldCategory::Procedure),
        (
            p::HoldReason::Dependency { task: 9 },
            w::HoldCategory::Dependency(
                w::HoldCategoryDependency { task: 9 },
            ),
        ),
        (
            p::HoldReason::Placed { hold: 9 },
            w::HoldCategory::Placed(
                w::HoldCategoryPlaced { hold: 9 },
            ),
        ),
        (
            p::HoldReason::Uncertain { entry: 9 },
            w::HoldCategory::Uncertain(
                w::HoldCategoryUncertain { entry: 9 },
            ),
        ),
        (
            p::HoldReason::PoolLost { pool: 9 },
            w::HoldCategory::PoolLost(
                w::HoldCategoryPoolLost { pool: 9 },
            ),
        ),
        (
            p::HoldReason::Failures { class: p::FailureClass::Transient },
            w::HoldCategory::Failures(
                w::HoldCategoryFailures { class: w::HoldFailure::Transient },
            ),
        ),
        (
            p::HoldReason::Failures { class: p::FailureClass::Permanent },
            w::HoldCategory::Failures(
                w::HoldCategoryFailures { class: w::HoldFailure::Permanent },
            ),
        ),
        (
            p::HoldReason::Failures { class: p::FailureClass::Run },
            w::HoldCategory::Failures(
                w::HoldCategoryFailures { class: w::HoldFailure::Run },
            ),
        ),
        (
            p::HoldReason::Failures { class: p::FailureClass::Agent },
            w::HoldCategory::Failures(
                w::HoldCategoryFailures { class: w::HoldFailure::Agent },
            ),
        ),
        (
            p::HoldReason::Failures { class: p::FailureClass::Lost },
            w::HoldCategory::Failures(
                w::HoldCategoryFailures { class: w::HoldFailure::Lost },
            ),
        ),
        (
            p::HoldReason::Failures { class: p::FailureClass::Invalid },
            w::HoldCategory::Failures(
                w::HoldCategoryFailures { class: w::HoldFailure::Invalid },
            ),
        ),
        (
            p::HoldReason::Failures { class: p::FailureClass::Infrastructure },
            w::HoldCategory::Failures(
                w::HoldCategoryFailures { class: w::HoldFailure::Infrastructure },
            ),
        ),
    ]
}
pub fn refusals() -> [(p::Refusal, w::Refusal); 9] {
    let limits = wire();
    [
        (p::Refusal::NotWaiting, w::Refusal::NotWaiting),
        (p::Refusal::NoFurther, w::Refusal::NoFurther),
        (p::Refusal::Unknown, w::Refusal::Unknown),
        (p::Refusal::NotOffered, w::Refusal::NotOffered),
        (p::Refusal::KeyConflict, w::Refusal::KeyConflict),
        (p::Refusal::NoEscalationHolder, w::Refusal::NoEscalationHolder),
        (
            p::Refusal::Taken { by: 7 },
            w::Refusal::Taken(w::RefusalTaken { by: 7 }),
        ),
        (
            p::Refusal::Role { held: None },
            w::Refusal::Role(w::RefusalRole { held: None }),
        ),
        (
            p::Refusal::Role { held: Some(p::Role::Policy { role: 99 }) },
            w::Refusal::Role(
                w::RefusalRole {
                        held: Some(w::HeldRole::Policy(
                            w::HeldRolePolicy { role: 99 },
                        )),
                    },
            ),
        ),
    ]
}
pub fn ending_refusal(value: w::EndingFields) -> w::Refusal {
    let limits = wire();
    let row =
        w::Fact::Ending(w::EndingRow { parent: None, value });
    let mut out = Writer::new(skein_lib::length::usize_of(row.measure().checked_add(4).expect("length")));
    out.put(&row.measure().to_be_bytes()).expect("row length");
    row.encode(&mut out).expect("row");
    let how = w::Ending { rows: out.finish() };
    w::Refusal::Ended(w::RefusalEnded { how })
}
pub fn endings() -> [(p::Ending, w::Refusal); 7] {
    let limits = wire();
    [
        (p::Ending::Done, ending_refusal(w::EndingFields::Done)),
        (
            p::Ending::Failed,
            ending_refusal(w::EndingFields::Failed(
                w::EndingFailedFields { reason: None },
            )),
        ),
        (p::Ending::Deleted, ending_refusal(w::EndingFields::Deleted)),
        (
            p::Ending::Decided { by: 7, at: Wall::from_nanos(13) },
            ending_refusal(w::EndingFields::Decided(
                w::EndingDecidedFields { by: 7, party: None, how: None, at: 13 },
            )),
        ),
        (
            p::Ending::Cancelled { by: p::Party::Person(7), at: Wall::from_nanos(13) },
            ending_refusal(w::EndingFields::Cancelled(
                w::EndingCancelledFields {
                        by: w::Actor::Party(
                            w::ActorParty { value: 7 },
                        ),
                        reason: None,
                        at: Some(13),
                    },
            )),
        ),
        (
            p::Ending::Cancelled { by: p::Party::Task(8), at: Wall::from_nanos(13) },
            ending_refusal(w::EndingFields::Cancelled(
                w::EndingCancelledFields {
                        by: w::Actor::Task(w::ActorTask { value: 8 }),
                        reason: None,
                        at: Some(13),
                    },
            )),
        ),
        (
            p::Ending::Cancelled { by: p::Party::Deployment { project: 3 }, at: Wall::from_nanos(13) },
            ending_refusal(w::EndingFields::Cancelled(
                w::EndingCancelledFields {
                        by: w::Actor::ProjectDeployment(
                            w::ActorProjectDeployment { project: 3 },
                        ),
                        reason: None,
                        at: Some(13),
                    },
            )),
        ),
    ]
}
pub fn extra_refusals() -> [(p::Refusal, w::Refusal); 7] {
    let limits = wire();
    [
        (
            p::Refusal::Role { held: Some(p::Role::Owner) },
            w::Refusal::Role(
                w::RefusalRole { held: Some(w::HeldRole::Owner) },
            ),
        ),
        (
            p::Refusal::Role { held: Some(p::Role::Maintainer) },
            w::Refusal::Role(
                w::RefusalRole { held: Some(w::HeldRole::Maintainer) },
            ),
        ),
        (
            p::Refusal::Role { held: Some(p::Role::Member) },
            w::Refusal::Role(
                w::RefusalRole { held: Some(w::HeldRole::Member) },
            ),
        ),
        (
            p::Refusal::Role { held: Some(p::Role::Observer) },
            w::Refusal::Role(
                w::RefusalRole { held: Some(w::HeldRole::Observer) },
            ),
        ),
        (
            p::Refusal::Moved { now: p::Moved::Spec { revision: 31 } },
            w::Refusal::Moved(
                w::RefusalMoved { now: revision(31), what: w::Moved::Spec },
            ),
        ),
        (
            p::Refusal::Moved { now: p::Moved::Person { revision: 31 } },
            w::Refusal::Moved(
                w::RefusalMoved { now: revision(31), what: w::Moved::PersonTask },
            ),
        ),
        (
            p::Refusal::Moved { now: p::Moved::Note { revision: 31 } },
            w::Refusal::Moved(
                w::RefusalMoved { now: revision(31), what: w::Moved::Note },
            ),
        ),
    ]
}
