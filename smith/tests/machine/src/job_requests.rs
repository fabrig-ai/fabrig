//! Job requests shared by simulated and real stories (smith/docs/design/testing.md, 7).

use skein_lib::{Duration, Time, Token};
use smith_domain::tools;
use smith_protocol_machine::{FromDomain, ToDomain};

/// Request one yielding shell command.
#[must_use]
pub fn spawn(root: Token, command: &[u8], head: u32, tail: u32, deadline: Time) -> FromDomain {
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

/// Read a held job under its original token.
#[must_use]
pub fn read(head: u32, tail: u32, wait: Duration) -> FromDomain {
    FromDomain::Job { owner: Token::new(10), act: tools::JobAct::Read { head, tail, wait } }
}

/// Copy the one job terminal, checking its owner.
#[must_use]
pub fn done(terminal: &ToDomain) -> tools::Done {
    match terminal {
        ToDomain::Done { owner, done } => {
            assert_eq!(*owner, Token::new(10));
            done.clone()
        }
        ToDomain::Read { .. }
        | ToDomain::Probed { .. }
        | ToDomain::HooksProbed { .. }
        | ToDomain::Checked { .. }
        | ToDomain::Aborted { .. } => {
            panic!("job owes Done")
        }
    }
}

/// The five-second job deadline.
#[must_use]
pub fn far() -> Time {
    Time::from_nanos(5_000_000_000)
}
