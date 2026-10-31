use super::*;

const CHILD: Token = Token::new(30);
const OWNER: Token = Token::new(14);
const INPUT: Token = Token::new(31);
const OUTPUT: Token = Token::new(32);
const ERRORS: Token = Token::new(33);

fn live() -> Process {
    let mut process = Process::new();
    process.spawn();
    process.spawned(CHILD, Pipes::from_spawn(&[INPUT, OUTPUT, ERRORS]));
    process
}

#[test]
fn the_last_pipe_releases_an_exited_childs_drain_grace() {
    let mut process = live();
    let mut below = Queue::uncapped();
    process.exited(kernel::Exit::Code(7), Time::from_nanos(10), &mut below);
    assert_eq!(below.len(), 2);
    process.closed(OWNER, OWNER);
    process.closed(INPUT, OWNER);
    process.closed(OUTPUT, OWNER);
    assert!(!process.settled());
    assert_eq!(process.next_deadline(), Some(Time::from_nanos(10)));
    process.closed(ERRORS, OWNER);
    assert!(process.settled());
    assert_eq!(process.exit(), Some(7));
    assert_eq!(process.next_deadline(), None);
    process.reset();
    assert_eq!(process.exit(), None);
    process.spawn();
    assert!(process.spawning());
}

#[test]
fn drain_expiry_aborts_the_remaining_pipes_once() {
    let mut process = live();
    let mut below = Queue::uncapped();
    process.exited(kernel::Exit::Signal(15), Time::from_nanos(10), &mut below);
    process.closed(OWNER, OWNER);
    process.closed(INPUT, OWNER);
    below.clear();
    process.drain(Time::from_nanos(9), &mut below);
    assert!(below.is_empty());
    process.drain(Time::from_nanos(10), &mut below);
    assert_eq!(below.pop(), Some(io::Request::Abort { entity: OUTPUT }));
    assert_eq!(below.pop(), Some(io::Request::Abort { entity: ERRORS }));
    assert!(below.is_empty());
    process.drain(Time::from_nanos(11), &mut below);
    assert!(below.is_empty());
    assert_eq!(process.exit(), Some(143));
    assert_eq!(process.next_deadline(), None);
    assert!(!process.settled());
    process.closed(OUTPUT, OWNER);
    process.closed(ERRORS, OWNER);
    assert!(process.settled());
}

#[test]
fn a_spawn_failure_releases_the_pending_invocation() {
    let mut process = Process::new();
    process.spawn();
    process.closed(OUTPUT, OWNER);
    assert!(process.spawning());
    process.failed();
    assert!(process.settled());
    assert!(!process.spawning());
    process.spawn();
    process.spawned(CHILD, Pipes::CLOSED);
    process.closed(OWNER, OWNER);
    assert!(process.settled());
    assert_eq!(process.exit(), None);
}
