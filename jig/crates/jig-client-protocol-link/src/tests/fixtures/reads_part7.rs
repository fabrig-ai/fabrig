use super::*;
use skein_lib::List;
pub fn read_value_owner_person_task() -> c::PersonTask {
    c::PersonTask {
        task: chip(),
        revision: read_value_revision(),
        requester: read_value_owner_requester(),
        spec: read_value_specification_document(),
        address: read_value_person_address(),
        taken_by: None,
        since: None,
        stall: None,
        deadline: None,
        contract: read_value_task_contract_document(),
        results: List::with_capacity(0),
        cut: false,
        acts: List::with_capacity(0),
    }
}

pub fn read_value_owner_requester() -> c::Requester {
    c::Requester::Task(7)
}

pub fn read_value_person_address() -> c::PersonAddress {
    c::PersonAddress::Party(7)
}

pub fn read_value_task_contract_document() -> c::TaskContractDocument {
    c::TaskContractDocument::Approval { state: Box::from([]), part: read_value_owner_part() }
}

pub fn read_value_owner_part() -> c::Part {
    c::Part { connector: 7, kind: read_value_owner_part_kind(), key: read_value_owner_part_key() }
}

pub fn read_value_owner_part_kind() -> c::PartKind {
    c::PartKind::Resource
}

pub fn read_value_owner_part_key() -> c::PartKey {
    c::PartKey::Path(List::with_capacity(0))
}

pub fn read_value_result() -> c::Result {
    c::Result {
        task: chip(),
        ending: c::Ending::Done,
        value: None,
        at: None,
        details: None,
        acts: List::with_capacity(0),
    }
}

pub fn read_value_owner_ended() -> c::Ended {
    c::Ended { key: read_value_object_key(), value: read_value_owner_ended_value(), at: None, by: None }
}

pub fn read_value_object_key() -> c::ObjectKey {
    c::ObjectKey::Task(7)
}

pub fn read_value_owner_ended_value() -> c::EndedValue {
    c::EndedValue::Task { document: Box::new(task()), result: Box::new(read_value_result()) }
}
