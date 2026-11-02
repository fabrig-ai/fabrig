use super::*;
use crate::Service;

#[test]
fn an_unbound_program_keeps_the_native_limits() {
    let mut configuration = super::super::tests::configuration();
    let limits = configuration.limits;
    configuration.program = Some(ProgramBackend {
        program: Programs { claude: b"/claude".as_slice().into(), directory: b"/programs".as_slice().into() },
        endpoints: Box::default(),
        relay: Box::default(),
        entropy: Secret::new(Box::default()),
        effective_user: 1000,
    });
    let service = Service::new(configuration, 1).expect("unbound declaration needs no backend settings");
    assert!(service.stages.program.is_none());
    assert_eq!(service.stages.limits.routes, limits.routes);
    assert_eq!(service.stages.limits.io, limits.io);
    assert_eq!(service.stages.limits.file_slots, limits.file_slots);
}

#[test]
fn program_close_reaches_io_with_pending_file_terminals() {
    let mut configuration = super::super::tests::configuration();
    configuration.program = Some(ProgramBackend {
        program: Programs { claude: b"/claude".as_slice().into(), directory: b"/programs".as_slice().into() },
        endpoints: Box::new([0]),
        relay: b"/agent".as_slice().into(),
        entropy: Secret::new(Box::new([7; 32])),
        effective_user: 1000,
    });
    let mut service = Service::new(configuration, 1).expect("bounded program service");
    let entity = Token::new(91);
    service
        .stages
        .program
        .as_mut()
        .expect("configured program")
        .below
        .push(backend::Below::Io(io::Request::Close { entity }));
    service.stages.file_events.push(file::Event::Closed { owner: Token::new(92) });
    assert_eq!(super::super::stages::backlog(&service.stages), 2, "program lower work counts toward admission");
    down(&mut service.stages);
    let io::Request::Close { entity: received } = service.stages.io_requests.pop().expect("program close progresses")
    else {
        unreachable!("only a close was queued")
    };
    assert_eq!(received, entity);
    assert!(service.stages.program.as_ref().expect("configured program").below.is_empty());
    assert_eq!(service.stages.file_events.len(), 1, "file terminals still await their consumer");
}

fn child_service() -> Service {
    let mut config = validity_tests::configuration();
    config.limits.domain.run.conversations = 2;
    let mut service = Service::new(config, 1).expect("bounded child service");
    service.adopt_program_root(kernel::Fd::new(19)).expect("programs root");
    let program = service.stages.program.as_mut().unwrap();
    let name = domain::GrantName { account: 0, generation: 1 };
    let lapses = skein_lib::Time::from_nanos(60_000_000_000);
    program.backend.grant(backend::Grant { name, source: b"source".as_slice().into(), lapses }).unwrap();
    program.grant = Some(backend::Grant { name, source: b"source".as_slice().into(), lapses });
    service
}

fn child_request(service: &Service, owner: Token, account: u32) -> llm::FromDomain {
    llm::FromDomain::Complete {
        owner,
        grant: Some(domain::GrantName { account, generation: 1 }),
        prompt: domain::llm::Prompt {
            purpose: domain::llm::Purpose::Ordinary { guard: None },
            asker: domain::llm::Asker::Child,
            endpoint: domain::llm::Endpoint(0),
            affinity: domain::llm::Affinity { key: [0; 16], thread: 1 },
            model: b"fake".as_slice().into(),
            effort: None,
            system: Box::default(),
            tools: domain::tools::Grants { inspect: true, modify: false, shell: false },
            namespace: domain::tools::Namespace::NoWorkspace,
            served: Box::default(),
            choice: domain::llm::ToolChoice::Auto,
            messages: Box::default(),
            max_tokens: 128,
        },
        deadlines: domain::session::Deadlines {
            head: Duration::from_secs(1),
            idle: Duration::from_secs(1),
            whole: skein_lib::Time::from_nanos(30_000_000_000),
        },
        bounds: service.stages.limits.llm.receiving,
        outcome: smith_protocol_llm::fixture::contract(),
        deliver: None,
        observation: None,
        deltas: llm::Deltas { text: false, input: false },
    }
}

#[test]
fn a_closing_child_holds_its_slot_until_the_directory_terminal() {
    let mut service = child_service();
    let owner = Token::new(101);
    let request = child_request(&service, owner, 0);
    complete(&mut service.stages, request);
    complete(&mut service.stages, llm::FromDomain::Yield { owner });
    assert!(service.stages.llm_events.is_empty(), "yield owes no terminal of its own");
    complete(&mut service.stages, llm::FromDomain::Cancel { owner });
    let Some(llm::ToDomain::Cancelled { owner: cancelled }) = service.stages.llm_events.pop() else {
        panic!("cancelled child")
    };
    assert_eq!(cancelled, owner);
    complete(&mut service.stages, llm::FromDomain::Cancel { owner });
    assert!(service.stages.llm_events.is_empty(), "a repeated cancel owes no second terminal");
    complete(&mut service.stages, llm::FromDomain::Close { owner });
    let next = Token::new(102);
    let request = child_request(&service, next, 0);
    complete(&mut service.stages, request);
    let llm::ToDomain::Failed { owner: failed_owner, failure, detail, .. } =
        service.stages.llm_events.pop().expect("full child table refuses")
    else {
        panic!("program failure")
    };
    assert_eq!(failed_owner, next);
    assert_eq!(failure, domain::llm::Failure::Program { exit: None });
    assert!(skein_lib::bytes::find(&detail, b"conversations - 1").is_some());
    let program = service.stages.program.as_ref().unwrap();
    let child = program.children.get(0).unwrap().as_ref().unwrap();
    let directory = child.directory_owner();
    assert_eq!(child.first.raw(), FIRST.raw() + STRIDE);
    assert!(file_event(&mut service.stages, file::Event::Cancelled { owner: directory }).is_none());
    fire(&mut service.stages);
    assert!(service.stages.program.as_ref().unwrap().children.get(0).unwrap().is_none());
    assert!(service.stages.llm_events.is_empty(), "the cancelled first completion has one terminal");
    let request = child_request(&service, next, 0);
    complete(&mut service.stages, request);
    assert_eq!(service.stages.program.as_ref().unwrap().children.get(0).unwrap().as_ref().unwrap().owner, next);
}

#[test]
fn a_child_on_another_account_keeps_the_direct_backend() {
    let mut service = child_service();
    let request = child_request(&service, Token::new(103), 1);
    complete(&mut service.stages, request);
    let program = service.stages.program.as_ref().unwrap();
    assert!(program.children.get(0).unwrap().is_none());
    assert!(program.below.is_empty());
    // The direct component has no configured endpoint in this fixture.
    let Some(llm::ToDomain::Failed { owner, failure, .. }) = service.stages.llm_events.pop() else {
        panic!("direct failure")
    };
    assert_eq!(owner, Token::new(103));
    assert_eq!(failure, domain::llm::Failure::Invalid);
}
