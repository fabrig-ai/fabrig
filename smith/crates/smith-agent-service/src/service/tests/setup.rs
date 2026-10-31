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
    with_admission(service(), Some(Token::new(run)))
}

pub(super) fn admit_pending(service: Service, run: u64) -> Service {
    with_admission(service, Some(Token::new(run)))
}

pub(super) fn with_input() -> Service {
    with_streams(service(), Some(Token::new(11)), None, None)
}

pub(super) fn with_channel_streams(signals: Option<Token>) -> Service {
    with_streams(service(), Some(Token::new(11)), Some(Token::new(12)), signals)
}

pub(super) fn without_channel() -> Service {
    end_channel(service())
}

pub(super) fn interrupted_before_admission(run: u64) -> Service {
    let mut service = service();
    route_io_event(
        &mut service.stages,
        Some(&mut service.frame),
        io::Event::Shutdown { signal: kernel::ServiceSignal::Interrupt, from: None },
    );
    assert!(service.stages.cancel_pending, "the opening run retains cancellation until admission");
    with_admission(service, Some(Token::new(run)))
}

pub(super) fn service() -> Service {
    Service::new(configuration(), 1).expect("bounded service")
}

pub(crate) fn account_service() -> Service {
    let mut agent = event_service_with(configuration(), true);
    agent
        .stages
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
    domain_request(&mut agent.stages, Some(&mut agent.frame), request);
    let records = smith_protocol_events::sinks::fixture::drain_ready(
        agent.stages.events.as_mut().unwrap().sinks(),
        Token::new(400),
    );
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
    let mut service = if framed { service } else { end_channel(service) };
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
    let mut service = with_admission(Service::new(config, 1).expect("small watermark service"), run);
    for number in 0..service.stages.limits.queue {
        service.stages.submissions.push(kernel::Submit {
            op: Token::new(u64::from(number)),
            kind: kernel::Op::Close { fd: kernel::Fd::new(90) },
        });
    }
    assert_eq!(backlog(&service.stages), 0, "kernel-edge submissions are outside admission");
    assert_eq!(service.stages.submissions.capacity(), service.stages.limits.queue);
    assert_eq!(service.stages.completions.capacity(), service.stages.limits.queue);
    for number in 0..service.stages.limits.queue {
        service.stages.io_submissions.push(kernel::Submit {
            op: Token::new(u64::from(number)),
            kind: kernel::Op::Close { fd: kernel::Fd::new(91) },
        });
    }
    assert_eq!(backlog(&service.stages), u64::from(service.stages.limits.queue));
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
    service.stages.events.as_mut().unwrap().starting(1, 0, &charter);
    let domain = &mut service.frame.domain;
    domain::step(
        domain,
        &service.stages.domain_env,
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
        &mut service.stages.domain_requests,
    );
    let run = match service.stages.domain_requests.pop() {
        Some(domain::Request::Admitted { run, .. }) => run,
        unexpected => panic!("the run waits in preparation: {unexpected:?}"),
    };
    // Preparation's effects and facts precede the two entrances under test.
    while service.stages.domain_requests.pop().is_some() {}
    while domain.pop_fact().is_some() {}
    (service, run)
}

fn with_admission(service: Service, admitted: Option<Token>) -> Service {
    let Service { stages, frame } = service;
    Service { stages: Stages { admitted, ..stages }, frame }
}

fn with_streams(service: Service, input: Option<Token>, output: Option<Token>, signals: Option<Token>) -> Service {
    let Service { stages, frame } = service;
    Service { stages: Stages { input, output, signals, ..stages }, frame }
}

fn end_channel(mut service: Service) -> Service {
    let mut above = Queue::uncapped();
    let mut below = Queue::uncapped();
    service.frame.channel.from_below(
        skein_channel::LowerEvent::Read(skein_lib::stream::Up::End),
        &mut above,
        &mut below,
    );
    assert!(service.frame.channel.ended(), "the detached fixture has a terminal channel");
    service
}

/// Typed entrances at the boundaries exercised by the service pass scenarios.
#[expect(clippy::large_enum_variant, reason = "test entrances are consumed immediately and never retained")]
pub(super) enum Input {
    Machine(machine::ToDomain),
    Provider(llm::ToDomain),
    Host(channel::OpenEvent),
    Domain(domain::Event),
    Io(io::Event),
    IoRequest(io::Request),
    KernelSubmission(kernel::Submit),
    IoSubmission(kernel::Submit),
    FileSubmission(kernel::Submit),
    PendingMessage(PendingMessage),
    OperationSubmitted { global: Token, original: Token },
    OperationCompleted { global: Token },
}

pub(super) fn enter(service: &mut Service, input: Input) {
    let stages = &mut service.stages;
    match input {
        Input::Machine(event) => stages.machine_events.push(event),
        Input::Provider(event) => stages.llm_events.push(event),
        Input::Host(event) => stages.channel_events.push(event),
        Input::Domain(event) => stages.domain_events.push(event),
        Input::Io(event) => stages.io_events.push(event),
        Input::IoRequest(request) => stages.io_requests.push(request),
        Input::KernelSubmission(submit) => stages.submissions.push(submit),
        Input::IoSubmission(submit) => stages.io_submissions.push(submit),
        Input::FileSubmission(submit) => stages.file_submissions.push(submit),
        Input::PendingMessage(message) => stages.pending_messages.push(message),
        Input::OperationSubmitted { global, original } => {
            stages.operations.insert(global, Operation::Io(original)).expect("one fake kernel operation");
        }
        Input::OperationCompleted { global } => {
            stages.operations.remove(&global);
        }
    }
}

pub(super) fn llm_route(owner: Token) -> (Service, Token) {
    let mut service = service();
    let route = service.stages.owners.insert(Owner::Llm(owner)).expect("one routed lower owner");
    (service, component_owner(route.token()))
}
