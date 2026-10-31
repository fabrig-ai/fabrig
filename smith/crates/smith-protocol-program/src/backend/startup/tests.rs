#![expect(clippy::disallowed_methods, reason = "ordinary Rust step fixtures, programming-model.md 10.2")]
use super::*;
use crate::{Ended, Launch};
use skein_io::kernel;

#[test]
fn a_window_keeps_its_identity_but_releases_its_invocation_alarms() {
    let mut startup = Startup::new();
    let mut below = Queue::uncapped();
    let id = [7; 16];
    startup.prepare(&mut below);
    startup.window(Window::Resume(id), &mut below);
    startup.launch(Time::from_nanos(10), &mut below);
    assert_eq!(startup.startup_deadline(), Some(Time::from_nanos(10)));
    startup.initialized(&mut below);
    startup.remember(&mut below);
    assert_eq!(startup.startup_deadline(), None);
    assert_eq!(startup.session(), super::super::session::uuid(id));
    startup.relay_ended(Some(Time::from_nanos(20)), &mut below);
    startup.relay_ended(Some(Time::from_nanos(30)), &mut below);
    assert_eq!(startup.eof_deadline(), Some(Time::from_nanos(20)));
    startup.switch(&mut below);
    assert_eq!(startup.eof_deadline(), None);
    startup.reset(&mut below);
    assert_eq!(startup.phase(), Phase::Idle);
    assert_eq!(startup.running_session(), None);
    assert!(!startup.resuming());
    assert_eq!(startup.recorded(), Some(id));
    assert!(below.is_empty());
}

#[test]
fn closing_a_check_keeps_its_late_spawn_until_every_resource_settles() {
    let mut startup = Startup::new();
    let mut below = Queue::uncapped();
    let owner = Token::new(10);
    let child = Token::new(20);
    let output = Token::new(21);
    let errors = Token::new(22);
    let command = Command::new(
        owner,
        Launch {
            program: b"/claude".as_slice().into(),
            arguments: Box::default(),
            environment: Box::default(),
            root: kernel::Fd::new(30),
            directory: Box::default(),
        },
        Time::from_nanos(10),
        128,
        &mut below,
    );
    startup.checking(command, &mut below);
    below.clear();
    startup.close(&mut below);
    assert_eq!(startup.phase(), Phase::Ending);
    assert!(startup.owns(owner));
    assert_eq!(startup.check().expect("closing check").next_deadline(), None);
    assert!(below.is_empty());
    assert_eq!(
        startup.io_event(
            Time::from_nanos(1),
            io::Event::Spawned { owner, child, pipes: Box::new([output, errors]) },
            &mut below
        ),
        None
    );
    assert!(
        below.iter().any(|request| *request
            == io::Request::Signal { child, to: kernel::Target::Child, signal: kernel::Signal::Kill })
    );
    assert_eq!(
        startup.io_event(Time::from_nanos(1), io::Event::Exited { owner, exit: kernel::Exit::Signal(9) }, &mut below),
        None
    );
    for closed in [owner, output, errors] {
        assert_eq!(startup.io_event(Time::from_nanos(1), io::Event::Closed { owner: closed }, &mut below), None);
        if closed != errors {
            assert!(startup.check().expect("retained check").ended().is_none());
        }
    }
    assert_eq!(startup.check().expect("settled check").ended(), Some(Ended::Deadline { detail: &[] }));
    startup.checked(&mut below);
    assert!(!startup.owns(owner));
    startup.ended(&mut below);
    assert_eq!(startup.phase(), Phase::Ended);
}
