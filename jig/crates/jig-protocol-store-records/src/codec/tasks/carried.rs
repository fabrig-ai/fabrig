//! Values carried by core rows in their own family (protocol/store/records.md, 1).
use super::Context;
use crate::generated::tasks as mapped;
use crate::tasks_v13::{self as wire, Record};
use alloc::boxed::Box;
use jig_domain_tasks as tasks;
use skein_lib::Reader;
type Limits = wire::Limits;

pub fn problem(value: &tasks::Problem, limits: &Limits) -> Result<Box<[u8]>, wire::Problem> {
    let value = mapped::task_problem_to_record(value)?;
    value.encode(limits)
}
pub fn read_problem(bytes: &[u8], limits: &Limits) -> Result<tasks::Problem, wire::Problem> {
    let value = wire::TaskProblem::decode(limits, &mut Reader::new(bytes))?;
    Ok(mapped::task_problem_to_domain(value))
}
pub fn end(value: &tasks::End, limits: &Limits) -> Result<Box<[u8]>, wire::Problem> {
    let value = wire::EndRecord { value: mapped::end_to_record(value, &Context::new(limits, None))? };
    value.encode(limits)
}
pub fn read_end(bytes: &[u8], limits: &Limits) -> Result<tasks::End, wire::Problem> {
    let value = wire::EndRecord::decode(limits, &mut Reader::new(bytes))?;
    mapped::end_to_domain(value.value, &Context::new(limits, None))
}
pub fn phase(value: &tasks::Phase, limits: &Limits) -> Result<Box<[u8]>, wire::Problem> {
    let value = wire::PhaseRecord { value: mapped::phase_to_record(value, &Context::new(limits, None))? };
    value.encode(limits)
}
pub fn read_phase(bytes: &[u8], limits: &Limits) -> Result<tasks::Phase, wire::Problem> {
    let value = wire::PhaseRecord::decode(limits, &mut Reader::new(bytes))?;
    mapped::phase_to_domain(value.value, &Context::new(limits, None))
}
pub fn change(value: &tasks::Change, limits: &Limits) -> Result<Box<[u8]>, wire::Problem> {
    let value = wire::ChangeRecord { value: mapped::change_to_record(value) };
    value.encode(limits)
}
pub fn read_change(bytes: &[u8], limits: &Limits) -> Result<tasks::Change, wire::Problem> {
    let value = wire::ChangeRecord::decode(limits, &mut Reader::new(bytes))?;
    Ok(mapped::change_to_domain(value.value))
}
