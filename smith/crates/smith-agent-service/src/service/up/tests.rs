use super::*;
use crate::Service;

#[test]
fn one_content_drain_empties_the_root_beyond_the_service_fact_allowance() {
    use alloc::boxed::Box;
    use skein_lib::ReplyTo;

    let settings =
        smith_agent_world::Settings { job: smith_agent_world::Job::HostTools, ..smith_agent_world::Settings::calm(1) };
    let mut charter = super::super::tests::charter(&settings);
    charter.conventions = None;
    charter.models = Box::default();
    charter.llm.account = None;
    charter.grants.agents = false;
    charter.grants.tools = run::charter::Tools { inspect: false, modify: false, shell: false };
    let mut configuration = super::super::tests::configuration();
    configuration.domain = super::super::tests::domain_configuration(&charter);
    let mut service = Service::new(configuration, 1).expect("configured service");
    let root = &mut service.frame.domain;
    domain::step(
        root,
        &service.stages.domain_env,
        domain::Event::Start {
            reply_to: ReplyTo::new(Token::new(1)),
            host_run: Token::new(1),
            activation: 1,
            identity: None,
            window: domain::Window { turns: u32::MAX, bytes: u64::MAX },
            messages: Box::default(),
            charter,
            workspace: None,
            grants: Box::default(),
            transcript: None,
            answered: Box::default(),
        },
        &mut service.stages.domain_requests,
    );
    let mut completion = None;
    while let Some(request) = service.stages.domain_requests.pop() {
        if let domain::Request::Complete { owner, .. } = request {
            completion = Some(owner);
        }
    }
    domain::step(
        root,
        &service.stages.domain_env,
        domain::Event::Completed {
            owner: completion.expect("provider request"),
            completion: domain::llm::Completion {
                reasoning_dropped: Box::default(),
                content: Box::new([
                    domain::llm::Said::Text { text: b"one".as_slice().into(), replay: None },
                    domain::llm::Said::Text { text: b"two".as_slice().into(), replay: None },
                ]),
                stop: domain::llm::Stop::EndTurn,
                usage: domain::llm::Usage::ZERO,
            },
        },
        &mut service.stages.domain_requests,
    );
    assert_eq!(root.pending_content(), 3, "two text records and usage");
    drain_content(&mut service.frame);
    let root = &mut service.frame.domain;
    assert_eq!(root.pending_content(), 0);
    assert!(root.pop_content().is_none());
}
