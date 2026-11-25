//! Whole single-read owner documents (protocol/client/wire.md, 1, and protocol/client/README.md, 4).
use super::super::{chips, owner, task, w};
use crate::generated::client as mapped;
use alloc::boxed::Box;
use jig_client_protocol_wire::Record as _;
use jig_domain_core_vocabulary::documents as d;
use skein_lib::{List, Writer};
pub fn project(models: &[crate::Model], value: &d::Project, limits: &w::Limits) -> Result<w::Project, w::Problem> {
    let mut resources = List::with_capacity(value.resources.len());
    for row in &value.resources {
        resources.push(mapped::project_resource_to_record(row)).expect("supplied rows fit");
    }
    let resources = project_resource_pieces(&resources, limits.project_resources, w::paths::PROJECT_RESOURCES, limits)?;
    let mut members = List::with_capacity(value.members.len());
    for row in &value.members {
        members.push(mapped::project_member_to_record(row)).expect("supplied rows fit");
    }
    let mut pools = List::with_capacity(value.pools.len());
    for row in &value.pools {
        pools.push(owner::ledger(row)).expect("supplied rows fit");
    }
    let mut configured_pools = List::with_capacity(value.configured_pools.len());
    for row in &value.configured_pools {
        configured_pools.push(owner::ledger(row)).expect("supplied rows fit");
    }
    let mut spend = List::with_capacity(value.spend.len());
    for row in &value.spend {
        spend.push(mapped::project_spend_to_record(row, &(&[], limits))?).expect("supplied rows fit");
    }
    let spend = project_spend_pieces(&spend, limits.project_spend, w::paths::PROJECT_SPEND, limits)?;
    let mut recurring = List::with_capacity(value.recurring.len());
    for row in &value.recurring {
        recurring.push(recurrence(models, row, limits)?).expect("supplied rows fit");
    }
    let recurring = recurring_pieces(&recurring, limits.project_recurring, w::paths::PROJECT_RECURRING, limits)?;

    Ok(w::Project {
        acts: task::offered(models, &value.acts, limits, limits.project_acts, w::paths::PROJECT_ACTS)?,
        number: value.number,
        name: owner::text(&value.name),
        resources,
        policy: policy(models, &value.policy, limits)?,
        members,
        current_period: match &value.current_period {
            Some(item) => Some(owner::ledger(item)),
            None => None,
        },
        pools,
        configured_period: value.configured_period,
        configured_pools,
        configured_pools_cut: value.configured_pools_cut,
        spend,
        recurring,
        cut: value.cut,
    })
}
pub fn policy(models: &[crate::Model], value: &d::Policy, limits: &w::Limits) -> Result<w::ProjectPolicy, w::Problem> {
    let mut projections = List::with_capacity(value.projections.len());
    for row in &value.projections {
        projections.push(owner::grant(row)).expect("supplied rows fit");
    }
    let projections =
        grant_pieces(&projections, limits.project_policy_projections, w::paths::PROJECT_POLICY_PROJECTIONS, limits)?;
    let mut roles = List::with_capacity(value.roles.len());
    for row in &value.roles {
        roles.push(mapped::project_policy_role_to_record(row, &(models, limits))?).expect("supplied rows fit");
    }
    let roles =
        project_policy_role_pieces(&roles, limits.project_policy_roles, w::paths::PROJECT_POLICY_ROLES, limits)?;
    let mut requirements = List::with_capacity(value.requirements.len());
    for row in &value.requirements {
        requirements
            .push(w::PolicyRequirement {
                connector: row.grant.connector,
                kind: row.grant.kind,
                pattern: owner::pattern(&row.grant.segments, &row.grant.last),
                judge: mapped::policy_judge_to_record(&row.judge),
                guard: mapped::policy_guard_to_record(&row.guard),
                must_be_guarded: row.must_be_guarded,
            })
            .expect("supplied rows fit");
    }
    let requirements = policy_requirement_pieces(
        &requirements,
        limits.project_policy_requirements,
        w::paths::PROJECT_POLICY_REQUIREMENTS,
        limits,
    )?;
    let permissions = match &value.permissions {
        Some(rows) => {
            let mut copied = List::with_capacity(rows.len());
            for row in rows {
                copied.push(mapped::policy_permission_to_record(row)).expect("supplied permissions fit");
            }
            Some(copied)
        }
        None => None,
    };
    Ok(w::ProjectPolicy {
        templates: super::super::me::overrides(&value.templates),
        ceiling: owner::authority(models, &value.ceiling, limits)?,
        period_spend: value.period_spend,
        escalation_role: value.escalation_role,
        projections,
        roles,
        requirements,
        permissions,
        permissions_cut: value.permissions_cut,
        cut: value.cut,
    })
}
pub fn recurrence(
    models: &[crate::Model],
    value: &d::Recurring,
    limits: &w::Limits,
) -> Result<w::Recurring, w::Problem> {
    let mut batch = List::with_capacity(value.batch.len());
    for row in &value.batch {
        batch.push(super::proposals::proposed(models, row, limits)?).expect("supplied rows fit");
    }
    let batch =
        super::proposals::proposed_task_pieces(&batch, limits.recurring_batch, w::paths::RECURRING_BATCH, limits)?;
    let mut last_batch = List::with_capacity(value.last_batch.len());
    for row in &value.last_batch {
        last_batch.push(*row).expect("supplied rows fit");
    }

    Ok(w::Recurring {
        chip: chips::chip(&value.chip, limits)?,
        key: value.key,
        batch,
        budget: value.budget,
        overlap: mapped::recurring_overlap_to_record(&value.overlap),
        current_period: value.current_period,
        began: match value.began {
            Some(item) => Some(item.as_nanos()),
            None => None,
        },
        next: match value.next {
            Some(item) => Some(item.as_nanos()),
            None => None,
        },
        last_period: value.last_period,
        pending_period: value.pending_period,
        last_batch,
        cut: value.cut,
    })
}
pub(super) fn project_resource_pieces(
    rows: &List<w::ProjectResource>,
    bound: u32,
    path: w::Path,
    limits: &w::Limits,
) -> Result<Box<[u8]>, w::Problem> {
    let mut encoded = List::with_capacity(rows.len());
    for row in rows {
        let mut out = Writer::new(skein_lib::length::usize_of(row.measure(limits)?));
        match row.write(limits, &mut out) {
            Ok(()) => (),
            Err(_) => return Err(w::Problem { path, reason: skein_codec::Reason::Bound }),
        }
        encoded.push(out.finish()).expect("supplied rows fit");
    }
    super::super::pieces(&encoded, bound, path)
}
pub(super) fn project_spend_pieces(
    rows: &List<w::ProjectSpend>,
    bound: u32,
    path: w::Path,
    limits: &w::Limits,
) -> Result<Box<[u8]>, w::Problem> {
    let mut encoded = List::with_capacity(rows.len());
    for row in rows {
        let mut out = Writer::new(skein_lib::length::usize_of(row.measure(limits)?));
        match row.write(limits, &mut out) {
            Ok(()) => (),
            Err(_) => return Err(w::Problem { path, reason: skein_codec::Reason::Bound }),
        }
        encoded.push(out.finish()).expect("supplied rows fit");
    }
    super::super::pieces(&encoded, bound, path)
}
pub(super) fn recurring_pieces(
    rows: &List<w::Recurring>,
    bound: u32,
    path: w::Path,
    limits: &w::Limits,
) -> Result<Box<[u8]>, w::Problem> {
    let mut encoded = List::with_capacity(rows.len());
    for row in rows {
        let mut out = Writer::new(skein_lib::length::usize_of(row.measure(limits)?));
        match row.write(limits, &mut out) {
            Ok(()) => (),
            Err(_) => return Err(w::Problem { path, reason: skein_codec::Reason::Bound }),
        }
        encoded.push(out.finish()).expect("supplied rows fit");
    }
    super::super::pieces(&encoded, bound, path)
}
pub(super) fn grant_pieces(
    rows: &List<w::Grant>,
    bound: u32,
    path: w::Path,
    limits: &w::Limits,
) -> Result<Box<[u8]>, w::Problem> {
    let mut encoded = List::with_capacity(rows.len());
    for row in rows {
        let mut out = Writer::new(skein_lib::length::usize_of(row.measure(limits)?));
        match row.write(limits, &mut out) {
            Ok(()) => (),
            Err(_) => return Err(w::Problem { path, reason: skein_codec::Reason::Bound }),
        }
        encoded.push(out.finish()).expect("supplied rows fit");
    }
    super::super::pieces(&encoded, bound, path)
}
pub(super) fn project_policy_role_pieces(
    rows: &List<w::ProjectPolicyRole>,
    bound: u32,
    path: w::Path,
    limits: &w::Limits,
) -> Result<Box<[u8]>, w::Problem> {
    let mut encoded = List::with_capacity(rows.len());
    for row in rows {
        let mut out = Writer::new(skein_lib::length::usize_of(row.measure(limits)?));
        match row.write(limits, &mut out) {
            Ok(()) => (),
            Err(_) => return Err(w::Problem { path, reason: skein_codec::Reason::Bound }),
        }
        encoded.push(out.finish()).expect("supplied rows fit");
    }
    super::super::pieces(&encoded, bound, path)
}
pub(super) fn policy_requirement_pieces(
    rows: &List<w::PolicyRequirement>,
    bound: u32,
    path: w::Path,
    limits: &w::Limits,
) -> Result<Box<[u8]>, w::Problem> {
    let mut encoded = List::with_capacity(rows.len());
    for row in rows {
        let mut out = Writer::new(skein_lib::length::usize_of(row.measure(limits)?));
        match row.write(limits, &mut out) {
            Ok(()) => (),
            Err(_) => return Err(w::Problem { path, reason: skein_codec::Reason::Bound }),
        }
        encoded.push(out.finish()).expect("supplied rows fit");
    }
    super::super::pieces(&encoded, bound, path)
}
