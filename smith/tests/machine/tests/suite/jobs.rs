//! Commands held under their original token, on the real kernel (protocol/agent.md, 2).

use skein_lib::{Duration, Time, Token};
use smith_domain::tools;
use smith_machine_world::World;
use smith_protocol_machine::{FromDomain, ToDomain};

pub(super) fn spawn(root: Token, command: &[u8], head: u32, tail: u32, deadline: Time) -> FromDomain {
    FromDomain::Op {
        owner: Token::new(10),
        op: tools::Op::Spawn {
            cwd: tools::Place { root, path: Box::default() },
            command: command.into(),
            head,
            tail,
            after: Some(Duration::from_millis(30)),
        },
        deadline,
    }
}

pub(super) fn read(head: u32, tail: u32, wait: Duration) -> FromDomain {
    FromDomain::Job { owner: Token::new(10), act: tools::JobAct::Read { head, tail, wait } }
}

pub(super) fn done(terminal: &ToDomain) -> tools::Done {
    match terminal {
        ToDomain::Done { owner, done } => {
            assert_eq!(*owner, Token::new(10));
            done.clone()
        }
        ToDomain::Read { .. } | ToDomain::Probed { .. } | ToDomain::Checked { .. } | ToDomain::Aborted { .. } => {
            panic!("job owes Done")
        }
    }
}

pub(super) fn far() -> Time {
    Time::from_nanos(5_000_000_000)
}

#[test]
fn a_job_held_at_the_machines_close_is_closed_within_one_close() {
    let mut world = World::new(75, &[]);
    let replies = world.jobs(vec![spawn(world.root(), b"sleep 5", 8, 8, far())]);
    assert_eq!(done(&replies[0]), tools::Done::Held { head: Box::default(), tail: Box::default(), dropped: 0 });
    assert!(world.now() < Time::from_nanos(1_030_000_000), "yield plus one io close");
}
