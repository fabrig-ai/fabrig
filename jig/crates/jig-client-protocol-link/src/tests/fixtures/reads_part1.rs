use super::*;
use skein_lib::List;
pub fn read_wire_actor() -> w::Actor {
    w::Actor::Party(read_wire_actor_party())
}

pub fn read_wire_actor_party() -> w::ActorParty {
    let lim = w::CEILINGS;
    w::ActorParty::new(&lim, w::ActorPartyParts { value: 7 }).unwrap()
}

pub fn read_wire_recipient() -> w::Recipient {
    w::Recipient::Task(read_wire_recipient_task())
}

pub fn read_wire_recipient_task() -> w::RecipientTask {
    let lim = w::CEILINGS;
    w::RecipientTask::new(&lim, w::RecipientTaskParts { value: 7 }).unwrap()
}

pub fn read_wire_owner_proposal() -> w::OwnerProposal {
    jig_client_test_records::read_wire_owner_proposal()
}

pub fn read_wire_hold() -> w::Hold {
    let lim = w::CEILINGS;
    w::Hold::new(
        &lim,
        w::HoldParts {
            placed: None,
            revision: read_wire_revision(),
            reason: read_hold_reason_record(),
            before: read_wire_phase(),
            since: 7,
            result: None,
            spend: List::with_capacity(0),
        },
    )
    .unwrap()
}
