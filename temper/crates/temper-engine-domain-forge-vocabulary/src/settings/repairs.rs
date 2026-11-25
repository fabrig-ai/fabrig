//! Repair settings kept by name and resolved for one start (domain/red-repairs.md, 7–8).
use alloc::boxed::Box;

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct RepairSettings {
    pub goals: u8,
    pub attempts: u8,
    pub deadline_ms: u64,
    pub goal: RepairGoalSettings,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct RepairGoalSettings {
    pub charter: u32,
    pub llm: Option<NamedLlm>,
    pub priority: u32,
    pub budget: u64,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct NamedLlm {
    pub account: Option<u32>,
    pub main: NamedModel,
    pub subagents: Box<[NamedModel]>,
    pub later: Option<NamedLater>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct NamedLater {
    pub main: NamedModel,
    pub subagents: Box<[NamedModel]>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct NamedModel {
    pub model: Box<[u8]>,
    pub effort: Option<Box<[u8]>>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Repairs {
    pub goals: u8,
    pub attempts: u8,
    pub deadline_ms: u64,
    pub goal: RepairGoal,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct RepairGoal {
    pub charter: u32,
    pub llm: Option<jig_domain_tasks::LlmChoice>,
    pub priority: u32,
    pub budget: u64,
}

pub fn repairs_heap(v: &RepairSettings) -> Option<u64> {
    let Some(llm) = &v.goal.llm else { return Some(0) };
    let mut held = model_heap(&llm.main)?.checked_add(models_heap(&llm.subagents)?)?;
    if let Some(later) = &llm.later {
        held = held.checked_add(model_heap(&later.main)?)?.checked_add(models_heap(&later.subagents)?)?;
    }
    Some(held)
}
fn model_heap(v: &NamedModel) -> Option<u64> {
    let mut held = skein_lib::len_u64(v.model.len());
    if let Some(effort) = &v.effort {
        held = held.checked_add(skein_lib::len_u64(effort.len()))?;
    }
    Some(held)
}
fn models_heap(v: &[NamedModel]) -> Option<u64> {
    let mut held = skein_lib::len_u64(v.len()).checked_mul(u64::try_from(size_of::<NamedModel>()).ok()?)?;
    for v in v {
        held = held.checked_add(model_heap(v)?)?;
    }
    Some(held)
}
