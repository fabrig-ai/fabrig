//! Executor and dependency pieces of a party batch.

use super::super::{Box, List, Reader, count, domain, problem, wire};
use crate::generated::people as mapped;
use wire::Record;

pub(super) fn encode_dependencies(
    values: &[domain::Dependency],
    limits: &wire::Limits,
) -> Result<Box<[u8]>, wire::Problem> {
    let mut items = List::with_capacity(count(values.len())?);
    for value in values {
        let value = mapped::dependency_to_record(value, limits)?;
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
        let value = mapped::dependency_to_domain(wire::Dependency::decode(limits, &mut Reader::new(&bytes))?, limits)?;
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
