//! Field-for-field document values (client/domain/contract/documents.md and client/domain/contract/reads.md, 1, and client/domain/contract/watches.md, 1).
use super::{
    Limits, List, Reader, Wall, authority_document_fields, c, chip_fields, grant_piece, offered_piece,
    pool_ledger_fields, specification_document_fields, task_contract_document_fields, too_large, w,
};
use crate::generated::client;
use jig_client_protocol_wire::Record as _;

pub(crate) fn project_resource_piece(
    bytes: &[u8],
    limits: &Limits,
    bound: u32,
) -> Result<List<c::ProjectResource>, w::Problem> {
    let mut reader = Reader::new(bytes);
    let mut rows = List::with_capacity(bound);
    let mut sum = 0_u64;
    for _ in 0..=bound {
        let Some(item) = crate::translate::next_piece(&mut reader, &mut sum, limits)? else { return Ok(rows) };
        let row = w::ProjectResource::decode(limits.receiving(), &mut Reader::new(&item))?;
        let row = client::project_resource_to_domain(row);
        if rows.push(row).is_err() {
            return Err(too_large());
        }
    }
    Err(too_large())
}

pub(crate) fn project_policy_role_piece(
    bytes: &[u8],
    limits: &Limits,
    bound: u32,
) -> Result<List<c::ProjectPolicyRole>, w::Problem> {
    let mut reader = Reader::new(bytes);
    let mut rows = List::with_capacity(bound);
    let mut sum = 0_u64;
    for _ in 0..=bound {
        let Some(item) = crate::translate::next_piece(&mut reader, &mut sum, limits)? else { return Ok(rows) };
        let row = w::ProjectPolicyRole::decode(limits.receiving(), &mut Reader::new(&item))?;
        let row = client::project_policy_role_to_domain(row, limits)?;
        if rows.push(row).is_err() {
            return Err(too_large());
        }
    }
    Err(too_large())
}

pub(crate) fn policy_requirement_fields(value: &w::PolicyRequirement) -> c::PolicyRequirement {
    client::policy_requirement_to_domain(value.clone())
}

pub(crate) fn policy_requirement_piece(
    bytes: &[u8],
    limits: &Limits,
    bound: u32,
) -> Result<List<c::PolicyRequirement>, w::Problem> {
    let mut reader = Reader::new(bytes);
    let mut rows = List::with_capacity(bound);
    let mut sum = 0_u64;
    for _ in 0..=bound {
        let Some(item) = crate::translate::next_piece(&mut reader, &mut sum, limits)? else { return Ok(rows) };
        let row = w::PolicyRequirement::decode(limits.receiving(), &mut Reader::new(&item))?;
        let row = policy_requirement_fields(&row);
        if rows.push(row).is_err() {
            return Err(too_large());
        }
    }
    Err(too_large())
}

pub(crate) fn policy_permission_fields(value: &w::PolicyPermission) -> c::PolicyPermission {
    client::policy_permission_to_domain(value.clone())
}

pub(crate) fn project_policy_fields(value: &w::ProjectPolicy, limits: &Limits) -> Result<c::ProjectPolicy, w::Problem> {
    let opt0 = match &value.escalation_role {
        Some(item) => Some(*item),
        None => None,
    };
    let opt1 = match &value.permissions {
        Some(item) => {
            if (item).len() > limits.client().rows {
                return Err(too_large());
            }
            let mut list0 = List::with_capacity((item).len());
            for item in item {
                list0.push(policy_permission_fields(item)).expect("reserved supplied list");
            }
            Some(list0)
        }
        None => None,
    };
    Ok(c::ProjectPolicy {
        templates: super::me::overrides(&value.templates, limits)?,
        ceiling: authority_document_fields(&value.ceiling, limits)?,
        period_spend: value.period_spend,
        escalation_role: opt0,
        projections: grant_piece(&value.projections, limits, limits.client().authority)?,
        roles: project_policy_role_piece(&value.roles, limits, limits.client().rows)?,
        requirements: policy_requirement_piece(&value.requirements, limits, limits.client().rows)?,
        permissions: opt1,
        permissions_cut: value.permissions_cut,
        cut: value.cut,
    })
}

pub(crate) fn project_spend_piece(
    bytes: &[u8],
    limits: &Limits,
    bound: u32,
) -> Result<List<c::ProjectSpend>, w::Problem> {
    let mut reader = Reader::new(bytes);
    let mut rows = List::with_capacity(bound);
    let mut sum = 0_u64;
    for _ in 0..=bound {
        let Some(item) = crate::translate::next_piece(&mut reader, &mut sum, limits)? else { return Ok(rows) };
        let row = w::ProjectSpend::decode(limits.receiving(), &mut Reader::new(&item))?;
        let row = client::project_spend_to_domain(row, limits)?;
        if rows.push(row).is_err() {
            return Err(too_large());
        }
    }
    Err(too_large())
}

pub(crate) fn part_piece(bytes: &[u8], limits: &Limits, bound: u32) -> Result<List<c::Part>, w::Problem> {
    let mut reader = Reader::new(bytes);
    let mut rows = List::with_capacity(bound);
    let mut sum = 0_u64;
    for _ in 0..=bound {
        let Some(item) = crate::translate::next_piece(&mut reader, &mut sum, limits)? else { return Ok(rows) };
        let row = w::Part::decode(limits.receiving(), &mut Reader::new(&item))?;
        let row = crate::translate::facts::part(&row);
        if rows.push(row).is_err() {
            return Err(too_large());
        }
    }
    Err(too_large())
}

pub(crate) fn proposed_task_fields(value: &w::ProposedTask, limits: &Limits) -> Result<c::ProposedTask, w::Problem> {
    if value.dependencies.len() > limits.client().rows {
        return Err(too_large());
    }
    let mut list0 = List::with_capacity(value.dependencies.len());
    for item in &value.dependencies {
        list0.push(*item).expect("reserved supplied list");
    }
    let opt1 = match &value.priority {
        Some(item) => Some(*item),
        None => None,
    };
    Ok(c::ProposedTask {
        deadline: client::new_deadline_to_domain(value.deadline.clone()),
        number: value.number,
        spec: specification_document_fields(&value.spec, limits)?,
        executor: crate::translate::facts::executor(&value.executor),
        contract: task_contract_document_fields(&value.contract, limits)?,
        authority: authority_document_fields(&value.authority, limits)?,
        budget: value.budget,
        funder: crate::translate::facts::funder(&value.funder),
        dependencies: list0,
        resources: part_piece(&value.resources, limits, limits.client().rows)?,
        wake: crate::translate::facts::wake_policy(&value.wake),
        priority: opt1,
        cut: value.cut,
    })
}

pub(crate) fn proposed_task_piece(
    bytes: &[u8],
    limits: &Limits,
    bound: u32,
) -> Result<List<c::ProposedTask>, w::Problem> {
    let mut reader = Reader::new(bytes);
    let mut rows = List::with_capacity(bound);
    let mut sum = 0_u64;
    for _ in 0..=bound {
        let Some(item) = crate::translate::next_piece(&mut reader, &mut sum, limits)? else { return Ok(rows) };
        let row = w::ProposedTask::decode(limits.receiving(), &mut Reader::new(&item))?;
        let row = proposed_task_fields(&row, limits)?;
        if rows.push(row).is_err() {
            return Err(too_large());
        }
    }
    Err(too_large())
}

pub(crate) fn recurring_fields(value: &w::Recurring, limits: &Limits) -> Result<c::Recurring, w::Problem> {
    let opt0 = match &value.began {
        Some(item) => Some(Wall::from_nanos(*item)),
        None => None,
    };
    let opt1 = match &value.next {
        Some(item) => Some(Wall::from_nanos(*item)),
        None => None,
    };
    let opt2 = match &value.pending_period {
        Some(item) => Some(*item),
        None => None,
    };
    if value.last_batch.len() > limits.client().rows {
        return Err(too_large());
    }
    let mut list3 = List::with_capacity(value.last_batch.len());
    for item in &value.last_batch {
        list3.push(*item).expect("reserved supplied list");
    }
    Ok(c::Recurring {
        chip: chip_fields(&value.chip, limits)?,
        key: value.key,
        batch: proposed_task_piece(&value.batch, limits, limits.client().rows)?,
        budget: value.budget,
        overlap: client::recurring_overlap_to_domain(value.overlap.clone()),
        current_period: value.current_period,
        began: opt0,
        next: opt1,
        last_period: value.last_period,
        pending_period: opt2,
        last_batch: list3,
        cut: value.cut,
    })
}

pub(crate) fn recurring_piece(bytes: &[u8], limits: &Limits, bound: u32) -> Result<List<c::Recurring>, w::Problem> {
    let mut reader = Reader::new(bytes);
    let mut rows = List::with_capacity(bound);
    let mut sum = 0_u64;
    for _ in 0..=bound {
        let Some(item) = crate::translate::next_piece(&mut reader, &mut sum, limits)? else { return Ok(rows) };
        let row = w::Recurring::decode(limits.receiving(), &mut Reader::new(&item))?;
        let row = recurring_fields(&row, limits)?;
        if rows.push(row).is_err() {
            return Err(too_large());
        }
    }
    Err(too_large())
}

pub(crate) fn project_fields(value: &w::Project, limits: &Limits) -> Result<c::Project, w::Problem> {
    if value.members.len() > limits.client().rows {
        return Err(too_large());
    }
    let mut list0 = List::with_capacity(value.members.len());
    for item in &value.members {
        list0.push(client::project_member_to_domain(item.clone(), limits)?).expect("reserved supplied list");
    }
    let opt1 = match &value.current_period {
        Some(item) => Some(pool_ledger_fields(item)),
        None => None,
    };
    if value.pools.len() > limits.client().rows {
        return Err(too_large());
    }
    let mut list2 = List::with_capacity(value.pools.len());
    for item in &value.pools {
        list2.push(pool_ledger_fields(item)).expect("reserved supplied list");
    }
    if value.configured_pools.len() > limits.client().rows {
        return Err(too_large());
    }
    let mut list3 = List::with_capacity(value.configured_pools.len());
    for item in &value.configured_pools {
        list3.push(pool_ledger_fields(item)).expect("reserved supplied list");
    }
    Ok(c::Project {
        acts: offered_piece(&value.acts, limits, c::MAX_OFFERED_ACTS)?,
        number: value.number,
        name: crate::translate::facts::document_text(&value.name, limits)?,
        resources: project_resource_piece(&value.resources, limits, limits.client().rows)?,
        policy: project_policy_fields(&value.policy, limits)?,
        members: list0,
        current_period: opt1,
        pools: list2,
        configured_period: value.configured_period,
        configured_pools: list3,
        configured_pools_cut: value.configured_pools_cut,
        spend: project_spend_piece(&value.spend, limits, limits.client().rows)?,
        recurring: recurring_piece(&value.recurring, limits, limits.client().rows)?,
        cut: value.cut,
    })
}
