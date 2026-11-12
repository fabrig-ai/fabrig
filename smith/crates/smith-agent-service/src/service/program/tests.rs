use super::*;
use crate::Service;

#[test]
fn an_unbound_program_keeps_the_native_limits() {
    let mut configuration = super::super::tests::configuration();
    let limits = configuration.limits;
    configuration.program = Some(ProgramBackend {
        keep_program_prompt: false,
        program: Programs { claude: b"/claude".as_slice().into(), directory: b"/programs".as_slice().into() },
        endpoints: Box::default(),
        relay: Box::default(),
        entropy: Secret::new(Box::default()),
        effective_user: 1000,
        parent: None,
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
        keep_program_prompt: false,
        program: Programs { claude: b"/claude".as_slice().into(), directory: b"/programs".as_slice().into() },
        endpoints: Box::new([0]),
        relay: b"/agent".as_slice().into(),
        entropy: Secret::new(Box::new([7; 32])),
        effective_user: 1000,
        parent: None,
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
    program_service(2)
}

fn program_service(conversations: u32) -> Service {
    let mut config = validity_tests::configuration();
    config.limits.domain.run.conversations = conversations;
    config.limits.domain.session.sessions = conversations;
    config.limits.domain.session.tools.kits = conversations;
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
        outcome: smith_protocol_llm_fixtures::contract(),
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

fn main_request(service: &Service, owner: Token) -> llm::FromDomain {
    let mut request = child_request(service, owner, 0);
    let llm::FromDomain::Complete { prompt, .. } = &mut request else { unreachable!("completion request") };
    prompt.asker = domain::llm::Asker::Main;
    request
}

fn check_pipe(ordinal: u32) -> Token {
    let slot = u64::from(ordinal).checked_mul(3).unwrap().checked_add(10).unwrap();
    Token::new((slot << 32) | 1)
}

fn spawn_check(service: &mut Service, ordinal: u32) {
    let program = service.stages.program.as_ref().unwrap();
    let backend = if ordinal == 0 {
        &program.backend
    } else {
        &program.children.get(ordinal.checked_sub(1).unwrap()).unwrap().as_ref().unwrap().backend
    };
    let mut owner = None;
    for request in &program.below {
        let backend::Below::Io(request) = request else { continue };
        let (io::Request::Spawn { owner: candidate, .. } | io::Request::SpawnAlone { owner: candidate, .. }) = request
        else {
            continue;
        };
        if backend.owns(*candidate) {
            assert!(owner.replace(*candidate).is_none(), "one check spawn per program");
        }
    }
    let owner = owner.expect("each program reaches its status-check spawn");
    let output = check_pipe(ordinal);
    io_event(
        &mut service.stages,
        io::Event::Spawned {
            owner,
            child: Token::new(output.raw().checked_add(2 * (1 << 32)).unwrap()),
            pipes: Box::new([output, Token::new(output.raw().checked_add(1 << 32).unwrap())]),
        },
    );
}

fn open_child_directory(service: &mut Service, index: u32) {
    let owner =
        service.stages.program.as_ref().unwrap().children.get(index).unwrap().as_ref().unwrap().directory_owner();
    let descriptor = kernel::Fd::new(i32::try_from(index).unwrap().checked_add(20).unwrap());
    let file = service.stages.files.adopt_root(descriptor).expect("child directory root");
    assert!(file_event(&mut service.stages, file::Event::Opened { owner, file, len: 0 }).is_none());
    spawn_check(service, index.checked_add(1).unwrap());
}

#[test]
fn a_tools_file_answer_never_reaches_a_program() {
    enum Answer {
        Closed,
        Stored,
        Opened,
        Failed,
        Removed,
        Loaded,
    }

    for (children, pipe_ordinal, answered, answer) in [
        (2_u32, 0_u32, 2_u32, Answer::Closed),
        (2, 1, 2, Answer::Stored),
        (3, 2, 3, Answer::Opened),
        (3, 0, 0, Answer::Failed),
        (5, 5, 5, Answer::Removed),
        (5, 1, 1, Answer::Closed),
        (5, 0, 3, Answer::Loaded),
    ] {
        let mut service = program_service(children.checked_add(1).unwrap());
        let request = main_request(&service, Token::new(100));
        complete(&mut service.stages, request);
        spawn_check(&mut service, 0);
        for index in 0..children {
            let request = child_request(&service, Token::new(u64::from(index).checked_add(101).unwrap()), 0);
            complete(&mut service.stages, request);
        }
        for index in 0..answered {
            open_child_directory(&mut service, index);
        }
        let owner = check_pipe(pipe_ordinal);
        let backend = if pipe_ordinal == 0 {
            &service.stages.program.as_ref().unwrap().backend
        } else {
            &service
                .stages
                .program
                .as_ref()
                .unwrap()
                .children
                .get(pipe_ordinal.checked_sub(1).unwrap())
                .unwrap()
                .as_ref()
                .unwrap()
                .backend
        };
        assert!(backend.owns(owner), "the tool owner collides with a live check pipe");
        let file = service.stages.files.adopt_root(kernel::Fd::new(30)).expect("tool root");
        let make_answer = || match answer {
            Answer::Closed => file::Event::Closed { owner },
            Answer::Stored => file::Event::Stored { owner, digest: io::digest::digest(b"tool answer") },
            Answer::Opened => file::Event::Opened { owner, file, len: 0 },
            Answer::Failed => {
                file::Event::Failed { owner, error: kernel::Error::NotFound, committed: false, residue: None }
            }
            Answer::Removed => file::Event::Removed { owner },
            Answer::Loaded => file::Event::Loaded { owner, bytes: b"tool answer".as_slice().into() },
        };
        let event = make_answer();
        let expected = make_answer();
        assert_eq!(file_event(&mut service.stages, event), Some(expected), "the machine keeps its file answer");
        for index in answered..children {
            open_child_directory(&mut service, index);
        }
        for event in &service.stages.llm_events {
            match event {
                llm::ToDomain::Failed { .. } => panic!("no program completion fails"),
                llm::ToDomain::BuiltinResult { .. }
                | llm::ToDomain::Completed { .. }
                | llm::ToDomain::Cancelled { .. }
                | llm::ToDomain::Text { .. }
                | llm::ToDomain::ReadVersion { .. } => {}
            }
        }
    }
}
