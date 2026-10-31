use super::*;

#[test]
fn one_content_drain_empties_the_root_beyond_the_service_fact_allowance() {
    use alloc::boxed::Box;
    use skein_lib::ReplyTo;

    let mut service = Service::new(super::super::tests::configuration(), 1).expect("bounded service");
    let settings =
        smith_agent_world::Settings { job: smith_agent_world::Job::HostTools, ..smith_agent_world::Settings::calm(1) };
    let mut charter = super::super::tests::charter(&settings);
    charter.conventions = None;
    charter.models = Box::default();
    charter.llm.account = None;
    charter.grants.agents = false;
    charter.grants.tools = run::charter::Tools { inspect: false, modify: false, shell: false };
    let model = &charter.llm;
    service.domain = Some(domain::Domain::new(
        &service.limits.domain,
        domain::Config {
            programs: Box::default(),
            endpoints: Box::new([model.endpoint]),
            models: Box::new([domain::ConfiguredModel {
                endpoint: model.endpoint,
                model: model.model.clone(),
                efforts: Box::default(),
                prices: model.prices,
                window: model.window,
                output: model.output,
                protocol_allowance: 0,
                head: skein_lib::Duration::from_secs(60),
                idle: skein_lib::Duration::from_secs(30),
            }]),
        },
        1,
    ));
    let root = service.domain.as_mut().expect("framed root");
    domain::step(
        root,
        &service.domain_env,
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
        &mut service.domain_requests,
    );
    let mut completion = None;
    while let Some(request) = service.domain_requests.pop() {
        if let domain::Request::Complete { owner, .. } = request {
            completion = Some(owner);
        }
    }
    domain::step(
        root,
        &service.domain_env,
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
        &mut service.domain_requests,
    );
    assert_eq!(root.pending_content(), 3, "two text records and usage");
    drain_content(&mut service);
    let root = service.domain.as_mut().expect("framed root");
    assert_eq!(root.pending_content(), 0);
    assert!(root.pop_content().is_none());
}
