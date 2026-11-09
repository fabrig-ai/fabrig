//! Admission tests.
pub(in crate::tests) mod account_admission;
pub(in crate::tests) mod escalation_holder;
pub(in crate::tests) mod goal_of;
pub(in crate::tests) mod last_batch;
pub(in crate::tests) mod projects;
pub(in crate::tests) mod role_models;

mod deletion;
mod named_people;

mod visibility;

#[test]
fn asking_calls_require_the_asking_family() {
    use crate::tests::*;

    // The asking and read bits (jig/docs/design/domain/authority.md, 3).
    const ASK: u64 = 2048;
    const READ: u64 = 512;
    const TASK: u64 = 1;
    for (tools, expected) in [
        (ASK, None),
        (
            READ,
            Some(CallPart::ToolDenied {
                answer: authority::Answer::Propose,
                findings: Box::new([authority::Finding::Tool]),
            }),
        ),
    ] {
        let (mut core, limits) = core();
        let mut rules = core.authority.rules().clone();
        rules.ceiling.tools = authority::Tools(ASK | READ);
        let mut policy = core.authority.policy(1).unwrap().clone();
        policy.ceiling.tools = authority::Tools(ASK | READ);
        policy.roles[0].authority.tools = authority::Tools(tools);
        core.authority = authority::Domain::new(rules, limits.authority).unwrap();
        let mut out = Queue::uncapped();
        authority::step(&mut core.authority, authority::Event::Policy { project: 1, policy }, &mut out);
        make_task(&mut core, &limits, TASK);
        assert_eq!(
            core.authorize_tool(CallKey { task: TASK, attempt: 1, completion: 1, position: 0 }, ToolKind::Ask),
            expected,
            "task tools = {tools}"
        );
    }
}
