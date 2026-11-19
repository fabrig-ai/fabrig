//! Executor and dependency pieces of a party batch.
use super::super::policy::{decode_person_address, encode_person_address};
use super::super::{Box, List, Reader, count, domain, problem, wire};
use wire::Record;
pub(crate) fn encode_executor(value: domain::TaskExecutor, limits: &wire::Limits) -> wire::TaskExecutor {
    match value {
        domain::TaskExecutor::Agent { charter } => wire::TaskExecutor::Agent(wire::TaskExecutorAgent { charter }),
        domain::TaskExecutor::Procedure { connector, code } => {
            wire::TaskExecutor::Procedure(wire::TaskExecutorProcedure { connector, code })
        }
        domain::TaskExecutor::Person(address) => {
            wire::TaskExecutor::Person(wire::TaskExecutorPerson { address: encode_person_address(&address, limits) })
        }
    }
}
pub(crate) fn decode_executor(value: wire::TaskExecutor) -> domain::TaskExecutor {
    match value {
        wire::TaskExecutor::Agent(value) => domain::TaskExecutor::Agent { charter: value.charter },
        wire::TaskExecutor::Procedure(value) => {
            let wire::TaskExecutorProcedure { connector, code } = value;
            domain::TaskExecutor::Procedure { connector, code }
        }
        wire::TaskExecutor::Person(value) => domain::TaskExecutor::Person(decode_person_address(value.address)),
    }
}
pub(super) fn encode_dependencies(
    values: &[domain::Dependency],
    limits: &wire::Limits,
) -> Result<Box<[u8]>, wire::Problem> {
    let mut items = List::with_capacity(count(values.len())?);
    for value in values {
        let value = match *value {
            domain::Dependency::Batch(at) => wire::Dependency::Batch(wire::DependencyBatch { at }),
            domain::Dependency::Existing(task) => wire::Dependency::Existing(wire::DependencyExisting { task }),
        };
        let writer = value.encode(limits)?;
        items.push(writer).expect("dependency count");
    }
    super::joined(items, limits.member_dependencies)
}
pub(super) fn decode_dependencies(
    bytes: Box<[u8]>,
    limits: &wire::Limits,
) -> Result<Box<[domain::Dependency]>, wire::Problem> {
    let mut reader = Reader::new(&bytes);
    let mut values = List::with_capacity(count(bytes.len())?);
    for _ in 0..bytes.len() {
        let Some(bytes) = super::super::pieces::split(&mut reader)? else { break };
        let value = match wire::Dependency::decode(limits, &mut Reader::new(&bytes))? {
            wire::Dependency::Batch(value) => domain::Dependency::Batch(value.at),
            wire::Dependency::Existing(value) => domain::Dependency::Existing(value.task),
        };
        values.push(value).expect("framed dependency count");
    }
    if u64::from(values.len())
        .checked_mul(u64::try_from(size_of::<domain::Dependency>()).expect("dependency size"))
        .ok_or(problem())?
        > u64::from(super::super::pieces::shared_sum(limits))
    {
        return Err(problem());
    }
    Ok(values.into_boxed())
}
