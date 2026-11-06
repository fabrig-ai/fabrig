//! Adopted roots close in reverse order, with one report outstanding at a time.
//! The state knows root tokens, never their paths or directory contracts.
//! `adopt` retains a root, `take_report` settles its close, and `follow` requests the next.
//! Contract: smith/docs/design/protocol/agent.md, section 6.

use super::Stages;
use core::mem;
use skein_io::{file, file_layer::step};
use skein_lib::{List, Token};

/// Adopted roots and their ordered close reports.
#[derive(Debug)]
pub(super) enum Roots {
    /// No root is held: none adopted, or every close reported.
    Empty,
    /// Roots in adoption order, with no close in flight.
    Held { roots: List<Token> },
    /// The root after `roots` awaits its close report.
    Closing { roots: List<Token> },
}

impl Roots {
    pub(super) fn adopt(&mut self, root: Token, capacity: u32) {
        let mut roots = match mem::replace(self, Self::Empty) {
            Self::Empty => List::with_capacity(capacity),
            Self::Held { roots } => roots,
            Self::Closing { .. } => unreachable!("roots attach before their close"),
        };
        roots.push(root).expect("admitted root count");
        *self = Self::Held { roots };
    }

    pub(super) fn first(&self) -> Option<Token> {
        match self {
            Self::Empty => None,
            Self::Held { roots } | Self::Closing { roots } => roots.get(0).copied(),
        }
    }

    pub(super) const fn held(&self) -> bool {
        match self {
            Self::Held { .. } => true,
            Self::Empty | Self::Closing { .. } => false,
        }
    }

    pub(super) const fn closed(&self) -> bool {
        match self {
            Self::Empty => true,
            Self::Held { .. } | Self::Closing { .. } => false,
        }
    }

    pub(super) fn take_report(&mut self) -> bool {
        let (state, reported) = match mem::replace(self, Self::Empty) {
            Self::Closing { roots } => (Self::Held { roots }, true),
            state @ (Self::Empty | Self::Held { .. }) => (state, false),
        };
        *self = state;
        reported
    }

    fn close_next(&mut self) -> Option<Token> {
        let (state, next) = match mem::replace(self, Self::Empty) {
            Self::Held { mut roots } => match roots.last().copied() {
                Some(root) => {
                    let remaining = roots.len().checked_sub(1).expect("positive root count");
                    roots.truncate(remaining);
                    (Self::Closing { roots }, Some(root))
                }
                None => (Self::Empty, None),
            },
            state @ (Self::Empty | Self::Closing { .. }) => (state, None),
        };
        *self = state;
        next
    }
}

/// Roots may close after the machine and all its routes have settled.
pub(super) fn closable(service: &Stages) -> bool {
    service.machine.closed() && service.owners.is_empty() && service.pipe_routes.is_empty() && service.files.takes()
}

/// Close the machine, then request at most one root close when its children permit it.
pub(super) fn follow(service: &mut Stages) {
    service.machine.close(&mut service.machine_below);
    if closable(service)
        && let Some(root) = service.roots.close_next()
    {
        step::down(
            &mut service.files,
            &service.file_env,
            file::Request::Close { owner: super::ROOT_CLOSE_OWNER, file: root },
            &mut service.file_events,
            &mut service.file_submissions,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::Roots;
    use skein_lib::Token;

    #[test]
    fn roots_close_last_first_and_one_at_a_time() {
        let mut roots = Roots::Empty;
        assert!(roots.closed());
        assert!(!roots.held());
        assert!(!roots.take_report());
        assert_eq!(roots.close_next(), None);
        for value in [11, 12, 13] {
            roots.adopt(Token::new(value), 3);
        }
        assert_eq!(roots.first(), Some(Token::new(11)));
        for value in [13, 12, 11] {
            assert!(roots.held());
            assert!(!roots.closed());
            assert!(!roots.take_report());
            assert_eq!(roots.close_next(), Some(Token::new(value)));
            assert!(!roots.held());
            assert!(!roots.closed());
            assert_eq!(roots.close_next(), None);
            assert!(roots.take_report());
        }
        assert!(roots.held(), "the last report retains an empty Held until the next pass");
        assert!(!roots.closed());
        assert!(!roots.take_report());
        assert_eq!(roots.first(), None);
        assert_eq!(roots.close_next(), None);
        assert!(roots.closed());
        assert!(!roots.held());
        assert!(!roots.take_report());
    }
}
