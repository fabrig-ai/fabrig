use super::*;
pub fn lacks() -> [(p::Lack, w::Lacking); 21] {
    let limits = wire();
    [
        (p::Lack::Approver, w::Lacking::Approver),
        (p::Lack::Funding, w::Lacking::Funding),
        (p::Lack::Tool, w::Lacking::Tool),
        (p::Lack::Scope, w::Lacking::Scope),
        (p::Lack::Models, w::Lacking::Models),
        (p::Lack::Reference, w::Lacking::Reference),
        (p::Lack::ResourceAccess, w::Lacking::ResourceAccess),
        (p::Lack::ResourceUnavailable, w::Lacking::ResourceUnavailable),
        (p::Lack::Shape, w::Lacking::Shape),
        (p::Lack::Account, w::Lacking::Account),
        (p::Lack::Writer, w::Lacking::Writer),
        (
            p::Lack::Budget { needs: 11, left: 7 },
            w::Lacking::Budget(
                w::LackingBudget::new(&limits, w::LackingBudgetParts { needed: 11, left: 7 }).expect("lack"),
            ),
        ),
        (
            p::Lack::Grant { kind: 9 },
            w::Lacking::GrantKind(
                w::LackingGrantKind::new(&limits, w::LackingGrantKindParts { kind: 9 }).expect("lack"),
            ),
        ),
        (
            p::Lack::TaskKind { executor: None },
            w::Lacking::TaskKind(
                w::LackingTaskKind::new(&limits, w::LackingTaskKindParts { executor: None }).expect("lack"),
            ),
        ),
        (
            p::Lack::Depth { most: 3 },
            w::Lacking::Depth(
                w::LackingDepth::new(&limits, w::LackingDepthParts { needed: None, allowed: 3 }).expect("lack"),
            ),
        ),
        (
            p::Lack::Deadline,
            w::Lacking::Deadline(
                w::LackingDeadline::new(&limits, w::LackingDeadlineParts { asked: None, latest: None }).expect("lack"),
            ),
        ),
        (
            p::Lack::Tasks { left: 4 },
            w::Lacking::Tasks(
                w::LackingTasks::new(&limits, w::LackingTasksParts { needed: None, left: 4 }).expect("lack"),
            ),
        ),
        (
            p::Lack::Again { required: true },
            w::Lacking::Again(w::LackingAgain::new(&limits, w::LackingAgainParts { required: true }).expect("lack")),
        ),
        (
            p::Lack::Required { connector: 9, kind: 10 },
            w::Lacking::Required(
                w::LackingRequired::new(&limits, w::LackingRequiredParts { connector: 9, kind: 10 }).expect("lack"),
            ),
        ),
        (
            p::Lack::Failed { connector: 9, kind: 10 },
            w::Lacking::Failed(
                w::LackingFailed::new(&limits, w::LackingFailedParts { connector: 9, kind: 10 }).expect("lack"),
            ),
        ),
        (
            p::Lack::Unguarded { connector: 9, kind: 10 },
            w::Lacking::Unguarded(
                w::LackingUnguarded::new(&limits, w::LackingUnguardedParts { connector: 9, kind: 10 }).expect("lack"),
            ),
        ),
    ]
}

pub fn ceiling_lacks() -> [(p::Lack, w::Lacking); 10] {
    let limits = wire();
    [
        ceiling_case(
            &limits,
            (p::CeilingHolder::Rules, w::CeilingHolder::Rules),
            (p::CeilingDimension::PeriodSpend, w::CeilingDimension::PeriodSpend),
            Some(1000),
            Some(1001),
        ),
        ceiling_case(
            &limits,
            (p::CeilingHolder::Policy, w::CeilingHolder::Policy),
            (p::CeilingDimension::Tools, w::CeilingDimension::Tools),
            None,
            None,
        ),
        ceiling_case(
            &limits,
            (
                p::CeilingHolder::Role(0),
                w::CeilingHolder::Role(w::CeilingRole::new(&limits, w::CeilingRoleParts { role: 0 }).expect("role")),
            ),
            (p::CeilingDimension::PeriodSpend, w::CeilingDimension::PeriodSpend),
            Some(500),
            Some(499),
        ),
        ceiling_case(
            &limits,
            (p::CeilingHolder::Rules, w::CeilingHolder::Rules),
            (p::CeilingDimension::Spend, w::CeilingDimension::Spend),
            Some(500),
            Some(501),
        ),
        ceiling_case(
            &limits,
            (p::CeilingHolder::Rules, w::CeilingHolder::Rules),
            (p::CeilingDimension::Tasks, w::CeilingDimension::Tasks),
            Some(3),
            Some(4),
        ),
        ceiling_case(
            &limits,
            (p::CeilingHolder::Rules, w::CeilingHolder::Rules),
            (p::CeilingDimension::Depth, w::CeilingDimension::Depth),
            Some(2),
            Some(3),
        ),
        ceiling_case(
            &limits,
            (p::CeilingHolder::Rules, w::CeilingHolder::Rules),
            (p::CeilingDimension::Executors, w::CeilingDimension::Executors),
            None,
            None,
        ),
        ceiling_case(
            &limits,
            (p::CeilingHolder::Rules, w::CeilingHolder::Rules),
            (p::CeilingDimension::Grants, w::CeilingDimension::Grants),
            None,
            None,
        ),
        ceiling_case(
            &limits,
            (p::CeilingHolder::Rules, w::CeilingHolder::Rules),
            (p::CeilingDimension::Deadline, w::CeilingDimension::Deadline),
            Some(10),
            None,
        ),
        ceiling_case(
            &limits,
            (p::CeilingHolder::Rules, w::CeilingHolder::Rules),
            (p::CeilingDimension::Notes, w::CeilingDimension::Notes),
            None,
            None,
        ),
    ]
}
fn ceiling_case(
    limits: &w::Limits,
    holder: (p::CeilingHolder, w::CeilingHolder),
    dimension: (p::CeilingDimension, w::CeilingDimension),
    value: Option<u64>,
    asked: Option<u64>,
) -> (p::Lack, w::Lacking) {
    (
        p::Lack::Ceiling { holder: holder.0, dimension: dimension.0, value, asked },
        w::Lacking::Ceiling(
            w::LackingCeiling::new(
                limits,
                w::LackingCeilingParts { holder: holder.1, dimension: dimension.1, value, asked },
            )
            .expect("ceiling"),
        ),
    )
}
