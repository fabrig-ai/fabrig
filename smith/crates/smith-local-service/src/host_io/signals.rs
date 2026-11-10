//! The adopted signal stream and its retained kernel read
//! (skein/docs/foundation/programming-model.md, 5.2; smith/docs/design/shell/README.md, 5).
//! Adoption, submission, completion and `Closed` update the stream's state.
//! `close` asks io to close it; the machine knows no shutdown policy or routes.

use super::io;
use skein_lib::{Queue, Token};

/// The adopted signal stream and io's retained read.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Signals {
    /// None adopted, or its `Closed` came up.
    Absent,
    /// Adopted, with no read in the kernel.
    Open { stream: Token },
    /// io's retained read is in the kernel under `read`.
    Reading { stream: Token, read: Token },
}

impl Signals {
    pub(super) fn adopted(&mut self, stream: Token) {
        match *self {
            Self::Absent => *self = Self::Open { stream },
            Self::Open { .. } | Self::Reading { .. } => unreachable!("signal stream is adopted once"),
        }
    }

    pub(super) fn reading(&mut self, read: Token) {
        match *self {
            Self::Absent => unreachable!("a signal read follows adoption"),
            Self::Open { stream } => *self = Self::Reading { stream, read },
            Self::Reading { .. } => unreachable!("one retained signal read"),
        }
    }

    pub(super) fn completed(&mut self, operation: Token) {
        match *self {
            Self::Reading { stream, read } if read == operation => *self = Self::Open { stream },
            Self::Absent | Self::Open { .. } | Self::Reading { .. } => {}
        }
    }

    pub(super) fn closed(&mut self) {
        *self = Self::Absent;
    }

    pub(super) fn close(&self, below: &mut Queue<io::Request>) {
        match *self {
            Self::Absent => {}
            Self::Open { stream } | Self::Reading { stream, .. } => {
                below.push(io::Request::Close { entity: stream });
            }
        }
    }

    pub(super) fn owns(&self, owner: Token) -> bool {
        match *self {
            Self::Absent => false,
            Self::Open { stream } | Self::Reading { stream, .. } => stream == owner,
        }
    }

    pub(super) const fn streams(&self) -> u32 {
        match self {
            Self::Absent => 0,
            Self::Open { .. } | Self::Reading { .. } => 1,
        }
    }

    pub(super) const fn reads(&self) -> u32 {
        match self {
            Self::Absent | Self::Open { .. } => 0,
            Self::Reading { .. } => 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const STREAM: Token = Token::new(7);
    const READ: Token = Token::new(8);

    #[test]
    fn only_the_retained_read_is_left_out_of_settling() {
        let config = crate::tests::config();
        let mut hub = super::super::HostIo::new(&config.limits.process, config.launch, config.endpoints, 1).unwrap();
        hub.close();
        assert!(hub.settled(0));
        hub.signals.adopted(STREAM);
        assert_eq!(hub.signals.reads(), 0);
        hub.signals.reading(READ);
        hub.in_flight = 1;
        assert_eq!(hub.signals.reads(), 1);
        assert!(hub.settled(0), "the retained signal read does not hold component closes");
        hub.signals.completed(Token::new(9));
        assert_eq!(hub.signals.reads(), 1);
        hub.in_flight += 1;
        assert!(!hub.settled(0), "every other operation still holds settling");
        hub.in_flight -= 1;
        hub.signals.completed(READ);
        assert_eq!(hub.signals.reads(), 0);
        assert!(!hub.settled(0), "a completed read must return its flight credit too");
        hub.in_flight -= 1;
        assert!(hub.settled(0));
        let mut below = Queue::uncapped();
        hub.signals.close(&mut below);
        assert_eq!(below.pop(), Some(io::Request::Close { entity: STREAM }));
        assert!(below.is_empty());
    }

    #[test]
    fn a_closed_stream_counts_no_more() {
        let mut signals = Signals::Absent;
        let mut below = Queue::uncapped();
        assert_eq!(signals.streams(), 0);
        assert_eq!(signals.reads(), 0);
        signals.completed(READ);
        signals.close(&mut below);
        assert!(below.is_empty());
        signals.adopted(STREAM);
        assert_eq!(signals.streams(), 1);
        assert!(signals.owns(STREAM));
        assert!(!signals.owns(READ));
        signals.reading(READ);
        signals.close(&mut below);
        assert_eq!(below.pop(), Some(io::Request::Close { entity: STREAM }));
        assert_eq!(signals.reads(), 1, "closing retains the read until its completion");
        signals.completed(READ);
        signals.closed();
        assert_eq!(signals.streams(), 0);
        assert_eq!(signals.reads(), 0);
        assert!(!signals.owns(STREAM));
        signals.close(&mut below);
        assert!(below.is_empty());
    }
}
