//! Repair settings kept by name and resolved for one start (domain/red-repairs.md, 7–8).
use alloc::boxed::Box;
use skein_lib::List;
use temper_client_protocol_wire::v1 as w;

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

#[expect(clippy::manual_map, reason = "optional records are translated without closures")]
pub(super) fn record(value: Option<&RepairSettings>) -> Option<w::RepairSettings> {
    match value {
        Some(v) => Some(w::RepairSettings {
            goals: v.goals,
            attempts: v.attempts,
            deadline_ms: v.deadline_ms,
            goal: w::RepairGoalSettings {
                charter: v.goal.charter,
                priority: v.goal.priority,
                budget: v.goal.budget,
                llm: match &v.goal.llm {
                    Some(v) => Some(llm_record(v)),
                    None => None,
                },
            },
        }),
        None => None,
    }
}
fn model_record(v: &NamedModel) -> w::NamedModel {
    w::NamedModel { model: v.model.clone(), effort: v.effort.clone() }
}
fn models_record(v: &[NamedModel]) -> List<w::NamedModel> {
    let mut out = List::with_capacity(u32::try_from(v.len()).expect("bounded models"));
    for v in v {
        out.push(model_record(v)).expect("counted models");
    }
    out
}
#[expect(clippy::manual_map, reason = "optional records are translated without closures")]
fn llm_record(v: &NamedLlm) -> w::NamedLlm {
    w::NamedLlm {
        account: v.account,
        main: model_record(&v.main),
        subagents: models_record(&v.subagents),
        later: match &v.later {
            Some(v) => Some(w::NamedLater { main: model_record(&v.main), subagents: models_record(&v.subagents) }),
            None => None,
        },
    }
}
#[expect(clippy::manual_map, reason = "optional records are translated without closures")]
pub(super) fn from_record(v: Option<w::RepairSettings>) -> Option<RepairSettings> {
    let v = v?;
    let g = v.goal;
    Some(RepairSettings {
        goals: v.goals,
        attempts: v.attempts,
        deadline_ms: v.deadline_ms,
        goal: RepairGoalSettings {
            charter: g.charter,
            priority: g.priority,
            budget: g.budget,
            llm: match g.llm {
                Some(v) => Some(llm_from(v)),
                None => None,
            },
        },
    })
}
fn model_from(v: w::NamedModel) -> NamedModel {
    NamedModel { model: v.model, effort: v.effort }
}
fn models_from(v: List<w::NamedModel>) -> Box<[NamedModel]> {
    let mut out = List::with_capacity(v.len());
    for v in v.into_boxed() {
        out.push(model_from(v)).expect("counted models");
    }
    out.into_boxed()
}
fn llm_from(v: w::NamedLlm) -> NamedLlm {
    NamedLlm {
        account: v.account,
        main: model_from(v.main),
        subagents: models_from(v.subagents),
        later: match v.later {
            Some(v) => Some(NamedLater { main: model_from(v.main), subagents: models_from(v.subagents) }),
            None => None,
        },
    }
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
