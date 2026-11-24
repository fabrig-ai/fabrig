//! Wire projections of owner part facts (client/domain/testing.md, 4).
use super::super::{c, list, w};
pub(in crate::codec) fn wire_part(value: &c::Part) -> w::Part {
    w::Part { connector: value.connector, kind: wire_part_kind(&value.kind), key: wire_part_key(&value.key) }
}

pub(in crate::codec) fn wire_part_kind(value: &c::PartKind) -> w::PartKind {
    match value {
        c::PartKind::Resource => w::PartKind::Resource,
        c::PartKind::Pool => w::PartKind::Pool,
        c::PartKind::Result => w::PartKind::Result,
        c::PartKind::Procedure => w::PartKind::Procedure,
        c::PartKind::Effect => w::PartKind::Effect,
        c::PartKind::Outbox => w::PartKind::Outbox,
        c::PartKind::Hold => w::PartKind::Hold,
    }
}

pub(in crate::codec) fn wire_part_key(value: &c::PartKey) -> w::PartKey {
    match value {
        c::PartKey::Path(value) => w::PartKey::Path(w::PartKeyPath { value: list((value).iter().cloned()) }),
        c::PartKey::Number(value) => w::PartKey::Number(w::PartKeyNumber { value: *(value) }),
    }
}

pub(in crate::codec) fn wire_executor(value: &c::Executor) -> w::Executor {
    match value {
        c::Executor::Job { kind } => w::Executor::Job(w::ExecutorJob { kind: *kind }),
        c::Executor::Agent { charter } => w::Executor::Agent(w::ExecutorAgent { charter: *(charter) }),
        c::Executor::Procedure { connector, code } => {
            w::Executor::Procedure(w::ExecutorProcedure { connector: *(connector), code: *(code) })
        }
        c::Executor::Person(value) => w::Executor::Person(w::ExecutorPerson { value: wire_person_address(value) }),
    }
}

pub(in crate::codec) fn wire_person_address(value: &c::PersonAddress) -> w::PersonAddress {
    match value {
        c::PersonAddress::Party(value) => w::PersonAddress::Party(w::PersonAddressParty { value: *(value) }),
        c::PersonAddress::Role(value) => w::PersonAddress::Role(w::PersonAddressRole { value: *(value) }),
    }
}
