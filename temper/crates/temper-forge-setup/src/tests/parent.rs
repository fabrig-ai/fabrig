use crate::parent::{Act, Event, Parent, Status, Stop};

#[test]
fn forgejos_shutdown_is_ignored_every_time() {
    let mut parent = Parent::running();
    for stop in [Stop::Terminate, Stop::Terminate, Stop::Interrupt] {
        assert_eq!(parent.step(Event::Shutdown { stop, from_forgejo: true }), Act::Nothing);
    }
}
#[test]
fn every_external_shutdown_is_forwarded_with_the_same_signal() {
    let mut parent = Parent::running();
    for stop in [Stop::Terminate, Stop::Terminate, Stop::Interrupt] {
        assert_eq!(parent.step(Event::Shutdown { stop, from_forgejo: false }), Act::Forward(stop));
    }
}
#[test]
fn the_childs_exit_ends_the_parent_and_later_signals_do_nothing() {
    for (status, code) in [(Status::Code(0), 0), (Status::Code(3), 3), (Status::Signal(15), 143)] {
        let mut parent = Parent::running();
        assert_eq!(parent.step(Event::Exited(status)), Act::End(code));
        for from_forgejo in [false, true] {
            assert_eq!(parent.step(Event::Shutdown { stop: Stop::Terminate, from_forgejo }), Act::Nothing);
        }
        assert_eq!(parent.step(Event::Exited(Status::Code(0))), Act::Nothing);
    }
}
