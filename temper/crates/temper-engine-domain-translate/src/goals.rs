//! Pure goal shape and repair-row translations (domain/agent/tools-goal.md, 1; domain/red-repairs.md, 4.3–4.4).
use alloc::boxed::Box;
use forge::FindingRefusal;
use jig_domain_tasks as tasks;
use skein_lib::List;
use temper_engine_domain_forge as forge;

/// Narrow a goal's grants and kinds; its making owns the one-level depth reduction.
#[must_use]
pub fn goal_authority(
    forge_connector: u16,
    context: &tasks::DelegationContext,
    advisor: bool,
    spend: u64,
) -> tasks::Authority {
    let mut authority = context.authority.clone();
    let mut grants = List::with_capacity(u32::try_from(authority.grants.len()).expect("bounded grants"));
    for grant in &authority.grants {
        if grant.connector == forge_connector
            && (forge::READ_KINDS.contains(&grant.kind) || (!advisor && forge::CHANGE_KINDS.contains(&grant.kind)))
        {
            grants.push(grant.clone()).expect("narrowed grants fit");
        }
    }
    authority.grants = grants.into_boxed();
    let mut kinds = List::with_capacity(u32::try_from(authority.delegation.kinds.len()).expect("bounded kinds"));
    for kind in &authority.delegation.kinds {
        match *kind {
            tasks::AuthorityExecutor::Role(_) => {}
            tasks::AuthorityExecutor::Job(_)
            | tasks::AuthorityExecutor::Charter(_)
            | tasks::AuthorityExecutor::Procedure(_) => kinds.push(*kind).expect("narrowed kinds fit"),
        }
    }
    authority.delegation.kinds = kinds.into_boxed();
    authority.delegation.tasks = context.tasks_left.saturating_sub(1);
    authority.budget.spend = spend;
    authority.notes = tasks::Scopes(authority.notes.0 & tasks::Scopes::GOAL.0);
    authority.note_resources = Box::new([]);
    authority
}

pub fn repair_goal_template(
    core: &jig_domain_core::Core,
    forge_connector: u16,
) -> Result<(u32, tasks::Flags), FindingRefusal> {
    let template = core.current.template(2).ok_or(FindingRefusal::NoGoal)?;
    let mut flags = template.flags;
    let fallback = tasks::Closer { connector: forge_connector, code: crate::GOAL_CLOSER_CODE };
    let closer = match core.current.template(4) {
        Some(template) => match template.flags.ended_by {
            tasks::EndedBy::Closer(closer) => closer,
            tasks::EndedBy::Result | tasks::EndedBy::Requester | tasks::EndedBy::Standing => fallback,
        },
        None => fallback,
    };
    flags.ended_by = tasks::EndedBy::Closer(closer);
    Ok((template.label, flags))
}

pub fn repair_goal_spec(
    domain: &forge::Domain,
    bound: u32,
    writes: &[forge::FindingWrite],
    words: &[u8],
    attempts: u8,
) -> Result<tasks::Spec, FindingRefusal> {
    let first = &writes.first().expect("repair findings").row;
    let repository = domain
        .repository(first.signature.as_ref().expect("landed red signature").repository)
        .ok_or(FindingRefusal::NoGoal)?;
    let repository = skein_lib::bytes::concat(&[&repository.owner, b"/", &repository.name]);
    let Some(words) = forge::findings::repair_words(&repository, writes, words, attempts, bound) else {
        return Err(FindingRefusal::Words(tasks::Refusal::Spec.words()));
    };
    Ok(tasks::Spec { job: None, asks: false, words, parameters: Box::new([]), inputs: Box::new([]) })
}

pub fn stamp_repair(writes: &mut [forge::FindingWrite], goal: u64, wall: skein_lib::Wall, deadline_ms: u64) {
    let until = skein_lib::Wall::from_nanos(
        wall.as_nanos().saturating_add(skein_lib::Duration::from_millis(deadline_ms).as_nanos()),
    );
    for write in writes {
        let row = &mut write.row;
        let (attempts, failing) = match &row.repair {
            Some(repair) => (repair.attempts.saturating_add(1), repair.failing.clone()),
            None => (1, Box::default()),
        };
        row.state = forge::FindingState::Routed;
        row.goal = Some(goal);
        row.repair = Some(forge::FindingRepair {
            goal,
            made: true,
            attempts,
            cancelled: false,
            failing,
            until: Some(until),
            judging: None,
        });
    }
}
