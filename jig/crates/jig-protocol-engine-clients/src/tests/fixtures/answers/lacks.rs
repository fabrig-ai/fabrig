use super::*;
pub fn lacks() -> [(p::Lack, w::Lacking); 21] {
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
        (p::Lack::Budget { needs: 11, left: 7 }, w::Lacking::Budget(w::LackingBudget { needed: 11, left: 7 })),
        (p::Lack::Grant { kind: 9 }, w::Lacking::GrantKind(w::LackingGrantKind { kind: 9 })),
        (p::Lack::TaskKind { executor: None }, w::Lacking::TaskKind(w::LackingTaskKind { executor: None })),
        (p::Lack::Depth { most: 3 }, w::Lacking::Depth(w::LackingDepth { needed: None, allowed: 3 })),
        (p::Lack::Deadline, w::Lacking::Deadline(w::LackingDeadline { asked: None, latest: None })),
        (p::Lack::Tasks { left: 4 }, w::Lacking::Tasks(w::LackingTasks { needed: None, left: 4 })),
        (p::Lack::Again { required: true }, w::Lacking::Again(w::LackingAgain { required: true })),
        (
            p::Lack::Required { connector: 9, kind: 10 },
            w::Lacking::Required(w::LackingRequired { connector: 9, kind: 10 }),
        ),
        (p::Lack::Failed { connector: 9, kind: 10 }, w::Lacking::Failed(w::LackingFailed { connector: 9, kind: 10 })),
        (
            p::Lack::Unguarded { connector: 9, kind: 10 },
            w::Lacking::Unguarded(w::LackingUnguarded { connector: 9, kind: 10 }),
        ),
    ]
}

pub fn ceiling_lacks() -> [(p::Lack, w::Lacking); 10] {
    [
        ceiling_case(
            (p::CeilingHolder::Rules, w::CeilingHolder::Rules),
            (p::CeilingDimension::PeriodSpend, w::CeilingDimension::PeriodSpend),
            Some(1000),
            Some(1001),
        ),
        ceiling_case(
            (p::CeilingHolder::Policy, w::CeilingHolder::Policy),
            (p::CeilingDimension::Tools, w::CeilingDimension::Tools),
            None,
            None,
        ),
        ceiling_case(
            (p::CeilingHolder::Role(0), w::CeilingHolder::Role(w::CeilingRole { role: 0 })),
            (p::CeilingDimension::PeriodSpend, w::CeilingDimension::PeriodSpend),
            Some(500),
            Some(499),
        ),
        ceiling_case(
            (p::CeilingHolder::Rules, w::CeilingHolder::Rules),
            (p::CeilingDimension::Spend, w::CeilingDimension::Spend),
            Some(500),
            Some(501),
        ),
        ceiling_case(
            (p::CeilingHolder::Rules, w::CeilingHolder::Rules),
            (p::CeilingDimension::Tasks, w::CeilingDimension::Tasks),
            Some(3),
            Some(4),
        ),
        ceiling_case(
            (p::CeilingHolder::Rules, w::CeilingHolder::Rules),
            (p::CeilingDimension::Depth, w::CeilingDimension::Depth),
            Some(2),
            Some(3),
        ),
        ceiling_case(
            (p::CeilingHolder::Rules, w::CeilingHolder::Rules),
            (p::CeilingDimension::Executors, w::CeilingDimension::Executors),
            None,
            None,
        ),
        ceiling_case(
            (p::CeilingHolder::Rules, w::CeilingHolder::Rules),
            (p::CeilingDimension::Grants, w::CeilingDimension::Grants),
            None,
            None,
        ),
        ceiling_case(
            (p::CeilingHolder::Rules, w::CeilingHolder::Rules),
            (p::CeilingDimension::Deadline, w::CeilingDimension::Deadline),
            Some(10),
            None,
        ),
        ceiling_case(
            (p::CeilingHolder::Rules, w::CeilingHolder::Rules),
            (p::CeilingDimension::Notes, w::CeilingDimension::Notes),
            None,
            None,
        ),
    ]
}
fn ceiling_case(
    holder: (p::CeilingHolder, w::CeilingHolder),
    dimension: (p::CeilingDimension, w::CeilingDimension),
    value: Option<u64>,
    asked: Option<u64>,
) -> (p::Lack, w::Lacking) {
    (
        p::Lack::Ceiling { holder: holder.0, dimension: dimension.0, value, asked },
        w::Lacking::Ceiling(w::LackingCeiling { holder: holder.1, dimension: dimension.1, value, asked }),
    )
}
