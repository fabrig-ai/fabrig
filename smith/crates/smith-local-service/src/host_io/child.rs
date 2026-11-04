//! The agent child and the host's single channel read
//! (smith/docs/design/protocol/hosts.md, sections 2 and 3).
//! `spawned`, `hold` and `answered` keep the child and read together; the hub's
//! `drain_process` releases read terminals. This machine knows neither the
//! process's internal progress nor its IO routes; `Process` owns those.

use super::protocol;
use alloc::boxed::Box;

/// The host's one read of the child's channel.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Read {
    /// No host read awaits a terminal.
    Idle,
    /// One host read awaits a record, live piece or channel end.
    Held,
}

/// The latest agent child, retained through its reap.
#[derive(Debug)]
pub(super) enum Child {
    /// Runs are inline, or no agent has spawned yet.
    Absent,
    /// `Process` owns the latest spawn's progress and channel.
    Spawned { process: Box<protocol::Process>, read: Read },
}

impl Child {
    pub(super) fn spawned(&mut self, process: protocol::Process) {
        let state = core::mem::replace(self, Self::Absent);
        *self = match state {
            Self::Absent => Self::Spawned { process: Box::new(process), read: Read::Idle },
            Self::Spawned { read, .. } => match read {
                Read::Idle => Self::Spawned { process: Box::new(process), read: Read::Idle },
                Read::Held => unreachable!("the previous child's channel end answers its held read before Spawn"),
            },
        };
    }

    pub(super) fn hold(&mut self) {
        match self {
            Self::Absent => unreachable!("Spawn precedes Read"),
            Self::Spawned { read, .. } => {
                assert_eq!(*read, Read::Idle, "one host read at a time");
                *read = Read::Held;
            }
        }
    }

    pub(super) const fn read_held(&self) -> bool {
        match self {
            Self::Absent => false,
            Self::Spawned { read, .. } => match read {
                Read::Idle => false,
                Read::Held => true,
            },
        }
    }

    pub(super) fn answered(&mut self) {
        match self {
            Self::Absent => {}
            Self::Spawned { read, .. } => *read = Read::Idle,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use skein_lib::Token;

    #[test]
    fn a_new_spawn_holds_no_read() {
        let config = crate::tests::config();
        let mut hub = super::super::HostIo::new(&config.limits.process, config.launch, config.endpoints, 1).unwrap();
        assert!(!hub.read_held());
        hub.child.answered();
        hub.spawned(protocol::Process::new(
            Token::new(7), &hub.limits.channel, hub.limits.detail_bytes, hub.endpoints.clone(),
        ).unwrap());
        assert!(!hub.read_held());
        hub.hold_read();
        assert!(hub.read_held());
        hub.child.answered();
        assert!(!hub.read_held());
        hub.child.answered();
        hub.spawned(protocol::Process::new(
            Token::new(8), &hub.limits.channel, hub.limits.detail_bytes, hub.endpoints.clone(),
        ).unwrap());
        assert!(!hub.read_held());
    }
}
