//! Named initial states and typed entrances for service boundary scenarios.

use super::*;

pub(super) fn limits() -> Limits {
    crate::fixture::small()
}

pub(crate) fn configuration() -> Config {
    let limits = limits();
    let channel_endpoints = channel::Endpoints::new(List::with_capacity(1));
    let llm_endpoints = llm::Endpoints::new(Box::new([]), 1).expect("empty endpoint table");
    Config {
        program: None,
        limits,
        domain: domain::Config { programs: Box::new([]), endpoints: Box::new([]), models: Box::new([]) },
        channel_endpoints,
        llm_endpoints,
        environment: machine::Environment::encode(&[], &limits.machine).expect("empty environment fits"),
        stream_mode: StreamMode::Two,
    }
}

pub(crate) fn charter(settings: &smith_agent_world::Settings) -> run::Charter {
    let mut charter = smith_agent_world::scripted_charter(settings);
    charter.llm.window = 2_097_152;
    charter.llm.output = 65_536;
    charter
}

pub(crate) fn domain_configuration(charter: &run::Charter) -> domain::Config {
    let model = &charter.llm;
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
            head: Duration::from_secs(60),
            idle: Duration::from_secs(30),
        }]),
    }
}

pub(super) fn admitted(run: u64) -> Service {
    Service { admitted: Some(Token::new(run)), ..service() }
}

pub(super) fn admit_pending(service: Service, run: u64) -> Service {
    Service { admitted: Some(Token::new(run)), ..service }
}

pub(super) fn with_input() -> Service {
    Service { input: Some(Token::new(11)), ..service() }
}

pub(super) fn with_channel_streams(signals: Option<Token>) -> Service {
    Service { input: Some(Token::new(11)), output: Some(Token::new(12)), signals, ..service() }
}

pub(super) fn without_channel() -> Service {
    Service { channel: None, ..service() }
}

pub(super) fn interrupted_before_admission(run: u64) -> Service {
    let mut service = service();
    route_io_event(&mut service, io::Event::Shutdown { signal: kernel::ServiceSignal::Interrupt, from: None });
    assert!(service.cancel_pending, "the opening run retains cancellation until admission");
    Service { admitted: Some(Token::new(run)), ..service }
}

pub(super) fn service() -> Service {
    Service::new(configuration(), 1).expect("bounded service")
}

pub(crate) fn account_service() -> Service {
    let mut agent = event_service_with(configuration(), true);
    agent
        .events
        .as_mut()
        .unwrap()
        .sinks()
        .add(
            smith_protocol_events::sinks::Destination::Trace,
            Token::new(400),
            smith_protocol_events::sinks::Delivery::Complete,
        )
        .unwrap();
    agent
}

pub(crate) fn account_request(agent: &mut Service, request: domain::Request) -> smith_events::Record {
    domain_request(agent, request);
    let records =
        smith_protocol_events::sinks::fixture::drain_ready(agent.events.as_mut().unwrap().sinks(), Token::new(400));
    assert_eq!(records.len(), 1);
    records.into_iter().next().unwrap()
}

pub(super) fn event_service() -> Service {
    event_service_with(configuration(), false)
}

pub(super) fn event_service_with(mut config: Config, framed: bool) -> Service {
    config.limits.events = Some(
        crate::event_limits(
            &config.limits,
            smith_protocol_events::sinks::Capture::Calls,
            1,
            0,
            &[],
            Duration::from_secs(1),
        )
        .expect("bounded event configuration"),
    );
    let service = Service::new(config, 1).expect("bounded producer service");
    let mut service = if framed { service } else { Service { channel: None, ..service } };
    service.start_session(
        Time::ZERO,
        smith_events::SessionStarted {
            chat: None,
            wall_ms: 0,
            agent: smith_events::Agent { name: b"test".as_slice().into(), version: Box::default() },
            mode: smith_events::Mode::Agent,
            containment: smith_events::Containment::Groups,
            pid: 1,
            profile: b"test".as_slice().into(),
            capture: smith_events::Capture::Calls,
            delivery: smith_events::Delivery::Complete,
            versions: smith_events::Versions { events: 4, channel: 5, charter: 4, transcript: 6, chat_store: None },
            limits: List::with_capacity(0),
        },
    );
    service
}

pub(super) fn watermark(run: Option<Token>) -> Service {
    let mut config = configuration();
    config.limits.queue = 8;
    let mut service = Service { admitted: run, ..Service::new(config, 1).expect("small watermark service") };
    for number in 0..service.limits.queue {
        service.submissions.push(kernel::Submit {
            op: Token::new(u64::from(number)),
            kind: kernel::Op::Close { fd: kernel::Fd::new(90) },
        });
    }
    assert_eq!(backlog(&service), 0, "kernel-edge submissions are outside admission");
    assert_eq!(service.submissions.capacity(), service.limits.queue);
    assert_eq!(service.completions.capacity(), service.limits.queue);
    for number in 0..service.limits.queue {
        service.io_submissions.push(kernel::Submit {
            op: Token::new(u64::from(number)),
            kind: kernel::Op::Close { fd: kernel::Fd::new(91) },
        });
    }
    assert_eq!(backlog(&service), u64::from(service.limits.queue));
    service
}

pub(super) fn at_watermark() -> Service {
    watermark(Some(Token::new(9)))
}

pub(super) fn host_message(name: u64) -> channel::OpenEvent {
    channel::OpenEvent::Message {
        finish_requested: false,
        name: Token::new(name),
        label: Box::from(b"host".as_slice()),
        text: Box::from(b"hello".as_slice()),
    }
}

pub(super) fn preparing_message_run() -> (Service, Token) {
    let mut charter = charter(&smith_agent_world::Settings::calm(1));
    charter.models = Box::default();
    charter.llm.account = None;
    let mut config = configuration();
    config.domain = domain_configuration(&charter);
    let mut service = event_service_with(config, false);
    service.events.as_mut().unwrap().starting(1, 0, &charter);
    let domain = service.domain.as_mut().unwrap();
    domain::step(
        domain,
        &service.domain_env,
        domain::Event::Start {
            reply_to: ReplyTo::new(Token::new(9)),
            host_run: Token::new(9),
            activation: 1,
            identity: None,
            window: domain::Window { turns: u32::MAX, bytes: u64::MAX },
            messages: Box::default(),
            charter,
            workspace: Some(run::Workspace {
                directories: Box::new([run::Directory {
                    name: Box::from(b"work".as_slice()),
                    host: Box::from(b"/work".as_slice()),
                    root: Token::new(900),
                    writable: true,
                    git: true,
                    conflicts: Box::default(),
                }]),
            }),
            grants: Box::default(),
            transcript: None,
            answered: Box::default(),
        },
        &mut service.domain_requests,
    );
    let run = match service.domain_requests.pop() {
        Some(domain::Request::Admitted { run, .. }) => run,
        unexpected => panic!("the run waits in preparation: {unexpected:?}"),
    };
    // Preparation's effects and facts precede the two entrances under test.
    while service.domain_requests.pop().is_some() {}
    while domain.pop_fact().is_some() {}
    (service, run)
}
