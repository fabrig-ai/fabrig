//! Field-for-field document values (client/domain/contract.md, 5–7).
use super::{
    Limits, List, Reader, Wall, authority_document_fields, c, chip_fields, grant_piece, piece_problem,
    project_resource_fields, specification_document_fields, task_contract_document_fields, too_large, w,
};
pub(crate) fn project_resource_piece(
    bytes: &[u8],
    limits: &Limits,
    bound: u32,
) -> Result<List<c::ProjectResource>, w::Problem> {
    let mut reader = Reader::new(bytes);
    let mut rows = List::with_capacity(bound);
    let mut sum = 0_u64;
    for _ in 0..=bound {
        let item = match skein_codec::piece::split(&mut reader) {
            Ok(value) => value,
            Err(reason) => return Err(piece_problem(reason)),
        };
        let Some(item) = item else {
            return Ok(rows);
        };
        sum = sum
            .checked_add(u64::try_from(item.len()).expect("piece fits u64"))
            .ok_or(too_large())?
            .checked_add(4)
            .ok_or(too_large())?;
        if sum > limits.client().document_bytes {
            return Err(too_large());
        }
        let row = w::ProjectResource::decode(limits.receiving(), &mut Reader::new(&item))?;
        let row = project_resource_fields(&row);
        if rows.push(row).is_err() {
            return Err(too_large());
        }
    }
    Err(too_large())
}

pub(crate) fn project_policy_role_fields(
    value: &w::ProjectPolicyRole,
    limits: &Limits,
) -> Result<c::ProjectPolicyRole, w::Problem> {
    Ok(c::ProjectPolicyRole {
        number: value.number(),
        authority: authority_document_fields(value.authority(), limits)?,
        period_spend: value.period_spend(),
        requests: value.requests(),
        decides: value.decides(),
    })
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
        let item = match skein_codec::piece::split(&mut reader) {
            Ok(value) => value,
            Err(reason) => return Err(piece_problem(reason)),
        };
        let Some(item) = item else {
            return Ok(rows);
        };
        sum = sum
            .checked_add(u64::try_from(item.len()).expect("piece fits u64"))
            .ok_or(too_large())?
            .checked_add(4)
            .ok_or(too_large())?;
        if sum > limits.client().document_bytes {
            return Err(too_large());
        }
        let row = w::ProjectPolicyRole::decode(limits.receiving(), &mut Reader::new(&item))?;
        let row = project_policy_role_fields(&row, limits)?;
        if rows.push(row).is_err() {
            return Err(too_large());
        }
    }
    Err(too_large())
}

pub(crate) fn policy_judge_fields(value: &w::PolicyJudge) -> c::PolicyJudge {
    c::PolicyJudge { connector: value.connector(), requirement: value.requirement(), parameters: value.parameters() }
}

pub(crate) fn policy_guard_fields(value: &w::PolicyGuard) -> c::PolicyGuard {
    match value {
        w::PolicyGuard::Guarded => c::PolicyGuard::Guarded,
        w::PolicyGuard::Observed(row) => c::PolicyGuard::Observed { freshness: row.freshness() },
    }
}

pub(crate) fn policy_requirement_fields(value: &w::PolicyRequirement) -> c::PolicyRequirement {
    c::PolicyRequirement {
        connector: value.connector(),
        kind: value.kind(),
        pattern: crate::translate::answers::pattern(value.pattern()),
        judge: policy_judge_fields(value.judge()),
        guard: policy_guard_fields(value.guard()),
        must_be_guarded: value.must_be_guarded(),
    }
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
        let item = match skein_codec::piece::split(&mut reader) {
            Ok(value) => value,
            Err(reason) => return Err(piece_problem(reason)),
        };
        let Some(item) = item else {
            return Ok(rows);
        };
        sum = sum
            .checked_add(u64::try_from(item.len()).expect("piece fits u64"))
            .ok_or(too_large())?
            .checked_add(4)
            .ok_or(too_large())?;
        if sum > limits.client().document_bytes {
            return Err(too_large());
        }
        let row = w::PolicyRequirement::decode(limits.receiving(), &mut Reader::new(&item))?;
        let row = policy_requirement_fields(&row);
        if rows.push(row).is_err() {
            return Err(too_large());
        }
    }
    Err(too_large())
}

pub(crate) fn policy_permission_fields(value: &w::PolicyPermission) -> c::PolicyPermission {
    c::PolicyPermission { connector: value.connector(), permission: value.permission(), role: value.role() }
}

pub(crate) fn project_policy_fields(value: &w::ProjectPolicy, limits: &Limits) -> Result<c::ProjectPolicy, w::Problem> {
    let opt0 = match value.escalation_role() {
        Some(item) => Some(*item),
        None => None,
    };
    let opt1 = match value.permissions() {
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
        templates: super::me::overrides(value.templates(), limits)?,
        ceiling: authority_document_fields(value.ceiling(), limits)?,
        period_spend: value.period_spend(),
        escalation_role: opt0,
        projections: grant_piece(value.projections(), limits, limits.client().authority)?,
        roles: project_policy_role_piece(value.roles(), limits, limits.client().rows)?,
        requirements: policy_requirement_piece(value.requirements(), limits, limits.client().rows)?,
        permissions: opt1,
        permissions_cut: value.permissions_cut(),
        cut: value.cut(),
    })
}

pub(crate) fn project_member_fields(value: &w::ProjectMember, limits: &Limits) -> Result<c::ProjectMember, w::Problem> {
    Ok(c::ProjectMember {
        party: value.party(),
        kind: crate::translate::answers::party_kind(value.kind()),
        role: crate::translate::answers::held_role(value.role()),
        login: crate::translate::facts::document_text(value.login(), limits)?,
        name: crate::translate::facts::document_text(value.name(), limits)?,
    })
}

pub(crate) fn project_spend_fields(value: &w::ProjectSpend, limits: &Limits) -> Result<c::ProjectSpend, w::Problem> {
    Ok(c::ProjectSpend {
        chip: chip_fields(value.chip(), limits)?,
        funder: crate::translate::facts::funder(value.funder()),
        numbers: crate::translate::facts::numbers(value.numbers()),
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
        let item = match skein_codec::piece::split(&mut reader) {
            Ok(value) => value,
            Err(reason) => return Err(piece_problem(reason)),
        };
        let Some(item) = item else {
            return Ok(rows);
        };
        sum = sum
            .checked_add(u64::try_from(item.len()).expect("piece fits u64"))
            .ok_or(too_large())?
            .checked_add(4)
            .ok_or(too_large())?;
        if sum > limits.client().document_bytes {
            return Err(too_large());
        }
        let row = w::ProjectSpend::decode(limits.receiving(), &mut Reader::new(&item))?;
        let row = project_spend_fields(&row, limits)?;
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
        let item = match skein_codec::piece::split(&mut reader) {
            Ok(value) => value,
            Err(reason) => return Err(piece_problem(reason)),
        };
        let Some(item) = item else {
            return Ok(rows);
        };
        sum = sum
            .checked_add(u64::try_from(item.len()).expect("piece fits u64"))
            .ok_or(too_large())?
            .checked_add(4)
            .ok_or(too_large())?;
        if sum > limits.client().document_bytes {
            return Err(too_large());
        }
        let row = w::Part::decode(limits.receiving(), &mut Reader::new(&item))?;
        let row = crate::translate::facts::part(&row);
        if rows.push(row).is_err() {
            return Err(too_large());
        }
    }
    Err(too_large())
}

pub(crate) fn proposed_task_fields(value: &w::ProposedTask, limits: &Limits) -> Result<c::ProposedTask, w::Problem> {
    if value.dependencies().len() > limits.client().rows {
        return Err(too_large());
    }
    let mut list0 = List::with_capacity(value.dependencies().len());
    for item in value.dependencies() {
        list0.push(*item).expect("reserved supplied list");
    }
    let opt1 = match value.priority() {
        Some(item) => Some(*item),
        None => None,
    };
    Ok(c::ProposedTask {
        deadline: match value.deadline() {
            w::NewDeadline::Named => c::NewDeadline::Named,
            w::NewDeadline::Span(value) => c::NewDeadline::Span(value.value()),
        },
        number: value.number(),
        spec: specification_document_fields(value.spec(), limits)?,
        executor: crate::translate::facts::executor(value.executor()),
        contract: task_contract_document_fields(value.contract(), limits)?,
        authority: authority_document_fields(value.authority(), limits)?,
        budget: value.budget(),
        funder: crate::translate::facts::funder(value.funder()),
        dependencies: list0,
        resources: part_piece(value.resources(), limits, limits.client().rows)?,
        wake: crate::translate::facts::wake_policy(value.wake()),
        priority: opt1,
        cut: value.cut(),
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
        let item = match skein_codec::piece::split(&mut reader) {
            Ok(value) => value,
            Err(reason) => return Err(piece_problem(reason)),
        };
        let Some(item) = item else {
            return Ok(rows);
        };
        sum = sum
            .checked_add(u64::try_from(item.len()).expect("piece fits u64"))
            .ok_or(too_large())?
            .checked_add(4)
            .ok_or(too_large())?;
        if sum > limits.client().document_bytes {
            return Err(too_large());
        }
        let row = w::ProposedTask::decode(limits.receiving(), &mut Reader::new(&item))?;
        let row = proposed_task_fields(&row, limits)?;
        if rows.push(row).is_err() {
            return Err(too_large());
        }
    }
    Err(too_large())
}

pub(crate) fn recurring_overlap_fields(value: &w::RecurringOverlap) -> c::RecurringOverlap {
    match value {
        w::RecurringOverlap::Skip => c::RecurringOverlap::Skip,
        w::RecurringOverlap::Wait => c::RecurringOverlap::Wait,
    }
}

pub(crate) fn recurring_fields(value: &w::Recurring, limits: &Limits) -> Result<c::Recurring, w::Problem> {
    let opt0 = match value.began() {
        Some(item) => Some(Wall::from_nanos(*item)),
        None => None,
    };
    let opt1 = match value.next() {
        Some(item) => Some(Wall::from_nanos(*item)),
        None => None,
    };
    let opt2 = match value.pending_period() {
        Some(item) => Some(*item),
        None => None,
    };
    if value.last_batch().len() > limits.client().rows {
        return Err(too_large());
    }
    let mut list3 = List::with_capacity(value.last_batch().len());
    for item in value.last_batch() {
        list3.push(*item).expect("reserved supplied list");
    }
    Ok(c::Recurring {
        chip: chip_fields(value.chip(), limits)?,
        key: value.key(),
        batch: proposed_task_piece(value.batch(), limits, limits.client().rows)?,
        budget: value.budget(),
        overlap: recurring_overlap_fields(value.overlap()),
        current_period: value.current_period(),
        began: opt0,
        next: opt1,
        last_period: value.last_period(),
        pending_period: opt2,
        last_batch: list3,
        cut: value.cut(),
    })
}

pub(crate) fn recurring_piece(bytes: &[u8], limits: &Limits, bound: u32) -> Result<List<c::Recurring>, w::Problem> {
    let mut reader = Reader::new(bytes);
    let mut rows = List::with_capacity(bound);
    let mut sum = 0_u64;
    for _ in 0..=bound {
        let item = match skein_codec::piece::split(&mut reader) {
            Ok(value) => value,
            Err(reason) => return Err(piece_problem(reason)),
        };
        let Some(item) = item else {
            return Ok(rows);
        };
        sum = sum
            .checked_add(u64::try_from(item.len()).expect("piece fits u64"))
            .ok_or(too_large())?
            .checked_add(4)
            .ok_or(too_large())?;
        if sum > limits.client().document_bytes {
            return Err(too_large());
        }
        let row = w::Recurring::decode(limits.receiving(), &mut Reader::new(&item))?;
        let row = recurring_fields(&row, limits)?;
        if rows.push(row).is_err() {
            return Err(too_large());
        }
    }
    Err(too_large())
}
