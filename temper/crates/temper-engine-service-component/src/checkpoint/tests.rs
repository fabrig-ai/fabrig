use super::*;
use skein_io::kernel::{Fd, Pipe, Way};

fn holder() -> State {
    State::new(
        Some(CheckpointName { store: 12, suffix: 255 }),
        Some((
            b"cmd".as_slice().into(),
            Ok(Spawn {
                program: b"/bin/cmd".as_slice().into(),
                args: Box::new([b"cmd".as_slice().into(), b"arg".as_slice().into()]),
                env: Box::new([b"TEMPER_CHECKPOINT=/deployment/checkpoints/name".as_slice().into()]),
                root: Fd::new(5),
                dir: b".".as_slice().into(),
                pipes: Box::new([
                    Pipe { child: 0, way: Way::Inherit, parent: None },
                    Pipe { child: 1, way: Way::Inherit, parent: None },
                    Pipe { child: 2, way: Way::Inherit, parent: None },
                ]),
            }),
        )),
    )
}
fn held(state: &mut State) {
    state.store(Event::CheckpointHeld { owner: Token::new(1) });
}
fn child(state: &mut State) -> Token {
    held(state);
    let Some(io::Request::Spawn { owner, spawn }) = state.io.pop() else { panic!("hold spawns once") };
    assert_eq!(owner, CHILD);
    assert_eq!(spawn.program.as_ref(), b"/bin/cmd");
    assert_eq!(spawn.args.as_ref(), &[Box::from(b"cmd".as_slice()), Box::from(b"arg".as_slice())]);
    assert_eq!(spawn.dir.as_ref(), b".");
    assert_eq!(spawn.root, Fd::new(5));
    assert_eq!(spawn.env.as_ref(), &[Box::from(b"TEMPER_CHECKPOINT=/deployment/checkpoints/name".as_slice())]);
    assert_eq!(spawn.pipes.len(), 3);
    for (number, pipe) in spawn.pipes.iter().enumerate() {
        assert_eq!(pipe.child, u32::try_from(number).unwrap());
        assert_eq!(pipe.way, Way::Inherit);
        assert_eq!(pipe.parent, None);
    }
    assert!(state.io.is_empty());
    let child = Token::new(99);
    state.up(io::Event::Spawned { owner, child, pipes: Box::new([]) });
    child
}

#[test]
fn held_child_is_reaped_before_the_lock_is_released_and_exits_are_kept() {
    for exit in [Exit::Code(3), Exit::Signal(15)] {
        let mut state = holder();
        assert!(state.io.is_empty());
        let child = child(&mut state);
        assert!(state.store.is_empty());
        state.up(io::Event::Exited { owner: CHILD, exit });
        let Some(io::Request::Close { entity }) = state.io.pop() else { panic!("child close after exit") };
        assert_eq!(entity, child);
        assert!(state.store.is_empty());
        assert!(!state.done());
        state.up(io::Event::Closed { owner: CHILD });
        let Some(Request::Close { owner }) = state.store.pop() else { panic!("holder close after child close") };
        assert_eq!(owner, Token::new(1));
        assert!(!state.done());
        state.store(Event::Closed { owner: Token::new(1), removed: true });
        assert!(state.done());
        let Some(Outcome::Child(observed)) = state.outcome else { panic!("child outcome retained") };
        assert_eq!(observed, exit);
    }
}

#[test]
fn failed_spawn_and_missing_program_release_the_holder_before_finishing() {
    for error in [None, Some(io::Error::Busy), Some(io::Error::Other)] {
        let mut state = holder();
        if error.is_none() {
            state.spawn = Some(Err(()));
        }
        held(&mut state);
        if let Some(error) = error {
            let Some(io::Request::Spawn { .. }) = state.io.pop() else { panic!("held spawns") };
            state.up(io::Event::Failed { owner: CHILD, error });
            assert!(state.store.is_empty(), "wait for failed spawn's Closed");
            assert!(!state.done());
            state.up(io::Event::Closed { owner: CHILD });
        }
        let Some(Request::Close { .. }) = state.store.pop() else { panic!("release after failed command") };
        assert!(!state.done());
        state.store(Event::Closed { owner: Token::new(1), removed: true });
        assert!(state.done());
        let Some(Outcome::CantRun { why, .. }) = state.outcome else { panic!("command failure retained") };
        assert_eq!(why, error);
    }
}

#[test]
fn termination_signals_go_to_the_childs_group_even_during_spawn() {
    for early in [true, false] {
        let mut state = holder();
        if early {
            state.shutdown(ServiceSignal::Interrupt);
        }
        let child = child(&mut state);
        if !early {
            state.shutdown(ServiceSignal::Interrupt);
        }
        let Some(io::Request::Signal { child: observed, signal, to, reason }) = state.io.pop() else {
            panic!("forward interrupt")
        };
        assert_eq!(observed, child);
        assert_eq!(signal, Signal::Interrupt);
        assert_eq!(to, Target::Group);
        assert_eq!(reason, Reason::Asked);
        state.shutdown(ServiceSignal::Terminate);
        let Some(io::Request::Signal { signal, to, .. }) = state.io.pop() else { panic!("forward termination") };
        assert_eq!(signal, Signal::Terminate);
        assert_eq!(to, Target::Group);
        assert!(state.store.is_empty());
        assert!(!state.done());
    }
}
