//! Wire projections of owner part facts (client/domain/testing.md, 4).
use super::super::{bytes, c, list, w};
pub(in crate::codec) fn wire_owner_part(value: &c::OwnerPart, limits: &w::Limits) -> Result<w::OwnerPart, w::Problem> {
    w::OwnerPart::new(
        limits,
        w::OwnerPartParts {
            connector: value.connector,
            kind: wire_owner_part_kind(&value.kind, limits)?,
            key: wire_owner_part_key(&value.key, limits)?,
        },
    )
}

pub(in crate::codec) fn wire_owner_part_kind(
    value: &c::OwnerPartKind,
    _limits: &w::Limits,
) -> Result<w::OwnerPartKind, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::OwnerPartKind::Resource => w::OwnerPartKind::Resource,
        c::OwnerPartKind::Pool => w::OwnerPartKind::Pool,
        c::OwnerPartKind::Result => w::OwnerPartKind::Result,
        c::OwnerPartKind::Procedure => w::OwnerPartKind::Procedure,
        c::OwnerPartKind::Effect => w::OwnerPartKind::Effect,
        c::OwnerPartKind::Outbox => w::OwnerPartKind::Outbox,
        c::OwnerPartKind::Hold => w::OwnerPartKind::Hold,
    })
}

pub(in crate::codec) fn wire_owner_part_key(
    value: &c::OwnerPartKey,
    limits: &w::Limits,
) -> Result<w::OwnerPartKey, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::OwnerPartKey::Path(value) => w::OwnerPartKey::Path(w::OwnerPartKeyPath::new(
            limits,
            w::OwnerPartKeyPathParts {
                value: list((value).iter().map(|item| Ok::<_, w::Problem>(bytes::copy_of(item))))?,
            },
        )?),
        c::OwnerPartKey::Number(value) => {
            w::OwnerPartKey::Number(w::OwnerPartKeyNumber::new(limits, w::OwnerPartKeyNumberParts { value: *(value) })?)
        }
    })
}

pub(in crate::codec) fn wire_owner_executor(
    value: &c::OwnerExecutor,
    limits: &w::Limits,
) -> Result<w::OwnerExecutor, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::OwnerExecutor::Agent { charter } => w::OwnerExecutor::Agent(w::OwnerExecutorAgent::new(
            limits,
            w::OwnerExecutorAgentParts { charter: *(charter) },
        )?),
        c::OwnerExecutor::Procedure { connector, code } => w::OwnerExecutor::Procedure(w::OwnerExecutorProcedure::new(
            limits,
            w::OwnerExecutorProcedureParts { connector: *(connector), code: *(code) },
        )?),
        c::OwnerExecutor::Person(value) => w::OwnerExecutor::Person(w::OwnerExecutorPerson::new(
            limits,
            w::OwnerExecutorPersonParts { value: wire_person_address(value, limits)? },
        )?),
    })
}

pub(in crate::codec) fn wire_person_address(
    value: &c::PersonAddress,
    limits: &w::Limits,
) -> Result<w::PersonAddress, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::PersonAddress::Party(value) => {
            w::PersonAddress::Party(w::PersonAddressParty::new(limits, w::PersonAddressPartyParts { value: *(value) })?)
        }
        c::PersonAddress::Role(value) => {
            w::PersonAddress::Role(w::PersonAddressRole::new(limits, w::PersonAddressRoleParts { value: *(value) })?)
        }
    })
}
