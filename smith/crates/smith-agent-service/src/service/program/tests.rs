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
