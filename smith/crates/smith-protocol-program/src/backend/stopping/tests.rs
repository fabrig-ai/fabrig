use super::*;
use crate::backend::Pipes;

const CHILD: skein_lib::Token = skein_lib::Token::new(30);
const INPUT: skein_lib::Token = skein_lib::Token::new(31);
const GRACE: Duration = Duration::from_nanos(10);

fn live() -> Process {
    let mut process = Process::new();
    process.spawn();
    process.spawned(CHILD, Pipes { input: Some(INPUT), ..Pipes::CLOSED });
    process
}

#[test]
fn a_result_releases_its_action_without_forgetting_escalation() {
    let mut stopping = Stopping::new();
    let mut process = live();
    let mut below = Queue::uncapped();
    stopping.begin(Action::Close, Some(Time::from_nanos(10)));
    stopping.terminate(Time::from_nanos(10), GRACE, &mut process, &mut below);
    assert_eq!(
        below.pop(),
        Some(io::Request::Signal { child: CHILD, to: kernel::Target::Group, signal: kernel::Signal::Terminate })
    );
    stopping.result();
    stopping.release_alarm();
    assert!(!stopping.waiting());
    assert_eq!(stopping.next_deadline(), None);
    stopping.begin(Action::Close, Some(Time::from_nanos(30)));
    stopping.terminate(Time::from_nanos(30), GRACE, &mut process, &mut below);
    assert_eq!(
        below.pop(),
        Some(io::Request::Signal { child: CHILD, to: kernel::Target::Group, signal: kernel::Signal::Kill })
    );
    assert_eq!(below.pop(), Some(io::Request::Abort { entity: INPUT }));
    assert_eq!(below.pop(), Some(io::Request::Close { entity: CHILD }));
    assert!(below.is_empty());
}

#[test]
fn repeated_pre_spawn_expiry_still_gives_the_late_child_one_terminate_grace() {
    let mut stopping = Stopping::new();
    let mut process = Process::new();
    let mut below = Queue::uncapped();
    process.spawn();
    stopping.begin(Action::Close, Some(Time::from_nanos(10)));
    stopping.terminate(Time::from_nanos(10), GRACE, &mut process, &mut below);
    stopping.terminate(Time::from_nanos(20), GRACE, &mut process, &mut below);
    assert!(below.is_empty());
    process.spawned(CHILD, Pipes::CLOSED);
    stopping.spawned(Time::from_nanos(30), GRACE, &mut process, &mut below);
    assert_eq!(
        below.pop(),
        Some(io::Request::Signal { child: CHILD, to: kernel::Target::Group, signal: kernel::Signal::Terminate })
    );
    assert!(below.is_empty());
    assert_eq!(stopping.next_deadline(), Some(Time::from_nanos(40)));
    assert_eq!(stopping.action(), Some(Action::Close));
}

#[test]
fn a_stop_without_a_child_has_no_grace_to_wait_for() {
    let mut stopping = Stopping::new();
    let mut process = Process::new();
    let mut below = Queue::uncapped();
    stopping.begin(Action::Close, None);
    stopping.result();
    assert!(!stopping.waiting());
    stopping.terminate(Time::ZERO, GRACE, &mut process, &mut below);
    assert_eq!(stopping.next_deadline(), None);
    assert!(below.is_empty());
}

#[test]
fn a_window_reset_retains_only_a_deferred_yield() {
    for (intent, retained) in [
        (Intent::Ordinary, Intent::Ordinary),
        (Intent::Finishing, Intent::Ordinary),
        (Intent::YieldRequested, Intent::YieldRequested),
    ] {
        let mut stopping = Stopping::new();
        stopping.intend(intent);
        stopping.begin(Action::Close, Some(Time::from_nanos(10)));
        stopping.reset();
        assert!(stopping.intent() == retained);
        assert!(!stopping.waiting());
        assert_eq!(stopping.next_deadline(), None);
    }
}
