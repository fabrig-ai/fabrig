//! Terminal input demand and the independent terminal-size read
//! (smith/docs/design/protocol/host.md, 5.1).
//! Input retains its adopted stream until `Closed`; window answers may outlive it.
//! Neither machine knows terminal parsing or local policy. Adoption, reading,
//! bytes and close drive input; asks, answers and `follow` drive window reads.

use super::{io, local};
use skein_lib::{Queue, Token, stream};

/// The terminal's input stream and its current demand.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum TerminalInput {
    /// None adopted, or its `Closed` came up.
    Absent,
    /// Open and the person may type: each read asks for the next.
    Reading { stream: Token },
    /// Open and held by the local domain: no demand.
    Held { stream: Token },
    /// Closing still forwards bytes, and asks for no more.
    Closing { stream: Token },
}

impl TerminalInput {
    pub(super) fn adopted(&mut self, stream: Token, read_input: bool, below: &mut Queue<io::Request>) {
        match *self {
            Self::Absent => *self = Self::Reading { stream },
            Self::Reading { .. } | Self::Held { .. } | Self::Closing { .. } => {
                unreachable!("terminal input is adopted once")
            }
        }
        if read_input {
            self.follow(below);
        }
    }

    pub(super) fn reading(&mut self, reading: local::Reading, below: &mut Queue<io::Request>) {
        match *self {
            Self::Absent | Self::Closing { .. } => return,
            Self::Reading { stream } => match reading {
                local::Reading::Open => return,
                local::Reading::Held => *self = Self::Held { stream },
            },
            Self::Held { stream } => match reading {
                local::Reading::Open => *self = Self::Reading { stream },
                local::Reading::Held => return,
            },
        }
        self.follow(below);
    }

    pub(super) fn bytes(&self, below: &mut Queue<io::Request>) {
        match self {
            Self::Reading { .. } => self.follow(below),
            Self::Absent | Self::Held { .. } | Self::Closing { .. } => {}
        }
    }

    pub(super) fn close(&mut self, below: &mut Queue<io::Request>) {
        match *self {
            Self::Absent | Self::Closing { .. } => return,
            Self::Reading { stream } | Self::Held { stream } => *self = Self::Closing { stream },
        }
        self.follow(below);
    }

    pub(super) fn abort(&self, below: &mut Queue<io::Request>) {
        match *self {
            Self::Absent => {}
            Self::Reading { stream } | Self::Held { stream } | Self::Closing { stream } => {
                below.push(io::Request::Abort { entity: stream });
            }
        }
    }

    pub(super) fn closed(&mut self) {
        *self = Self::Absent;
    }

    pub(super) fn owns(&self, owner: Token) -> bool {
        match *self {
            Self::Absent => false,
            Self::Reading { stream } | Self::Held { stream } | Self::Closing { stream } => stream == owner,
        }
    }

    pub(super) const fn open(&self) -> Option<Token> {
        match *self {
            Self::Absent | Self::Closing { .. } => None,
            Self::Reading { stream } | Self::Held { stream } => Some(stream),
        }
    }

    /// What entering a state asks of the stream.
    fn follow(&self, below: &mut Queue<io::Request>) {
        match *self {
            Self::Absent => {}
            Self::Reading { stream } => below.push(io::Request::Stream {
                stream,
                down: stream::Down::Demand { read: stream::Read::Fill(1), room: 0 },
            }),
            Self::Held { stream } => below.push(io::Request::Stream {
                stream,
                down: stream::Down::Demand { read: stream::Read::Nothing, room: 0 },
            }),
            Self::Closing { stream } => below.push(io::Request::Close { entity: stream }),
        }
    }
}

/// One terminal-size read, whose answer may follow the input stream's `Closed`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum WindowRead {
    /// No read requested or awaiting its answer.
    Idle,
    /// One read waits for the down pass.
    Requested,
    /// One submitted read awaits its answer.
    Pending,
    /// One submitted read is followed by another request.
    Again,
}

impl WindowRead {
    pub(super) fn asked(&mut self) {
        *self = match *self {
            Self::Idle | Self::Requested => Self::Requested,
            Self::Pending | Self::Again => Self::Again,
        };
    }

    pub(super) fn answered(&mut self) {
        *self = match *self {
            Self::Again => Self::Requested,
            Self::Idle | Self::Requested | Self::Pending => Self::Idle,
        };
    }

    #[expect(clippy::trivially_copy_pass_by_ref, reason = "state readers borrow the machine consistently")]
    pub(super) const fn requested(&self) -> bool {
        match self {
            Self::Requested => true,
            Self::Idle | Self::Pending | Self::Again => false,
        }
    }

    pub(super) fn follow(&mut self, input: Option<Token>, below: &mut Queue<io::Request>) {
        match *self {
            Self::Requested => match input {
                Some(stream) => {
                    *self = Self::Pending;
                    below.push(io::Request::Window { owner: stream, stream });
                }
                None => *self = Self::Idle,
            },
            Self::Idle | Self::Pending | Self::Again => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const INPUT: Token = Token::new(7);

    fn demand(read: stream::Read) -> io::Request {
        io::Request::Stream { stream: INPUT, down: stream::Down::Demand { read, room: 0 } }
    }

    #[test]
    fn a_stream_demands_as_its_reading_changes() {
        let mut input = TerminalInput::Absent;
        let mut below = Queue::uncapped();
        input.reading(local::Reading::Held, &mut below);
        assert!(below.is_empty());
        input.adopted(INPUT, true, &mut below);
        assert_eq!(below.pop(), Some(demand(stream::Read::Fill(1))));
        input.reading(local::Reading::Open, &mut below);
        assert!(below.is_empty());
        input.reading(local::Reading::Held, &mut below);
        assert_eq!(below.pop(), Some(demand(stream::Read::Nothing)));
        input.reading(local::Reading::Held, &mut below);
        assert!(below.is_empty());
        assert_eq!(input.open(), Some(INPUT));
        input.reading(local::Reading::Open, &mut below);
        assert_eq!(below.pop(), Some(demand(stream::Read::Fill(1))));
        assert!(below.is_empty());
    }

    #[test]
    fn bytes_ask_for_more_only_while_reading() {
        let mut input = TerminalInput::Absent;
        let mut below = Queue::uncapped();
        input.bytes(&mut below);
        assert!(below.is_empty());
        input.adopted(INPUT, false, &mut below);
        assert!(below.is_empty());
        input.bytes(&mut below);
        assert_eq!(below.pop(), Some(demand(stream::Read::Fill(1))));
        input.reading(local::Reading::Held, &mut below);
        assert_eq!(below.pop(), Some(demand(stream::Read::Nothing)));
        input.bytes(&mut below);
        assert!(below.is_empty());
        input.close(&mut below);
        assert_eq!(below.pop(), Some(io::Request::Close { entity: INPUT }));
        input.bytes(&mut below);
        assert!(below.is_empty());
    }

    #[test]
    fn a_closing_stream_takes_no_reading_and_still_aborts() {
        let mut input = TerminalInput::Absent;
        let mut below = Queue::uncapped();
        input.close(&mut below);
        input.abort(&mut below);
        assert!(below.is_empty());
        input.adopted(INPUT, false, &mut below);
        input.close(&mut below);
        assert_eq!(below.pop(), Some(io::Request::Close { entity: INPUT }));
        assert!(input.owns(INPUT));
        assert!(!input.owns(Token::new(8)));
        assert_eq!(input.open(), None);
        input.close(&mut below);
        input.reading(local::Reading::Held, &mut below);
        input.reading(local::Reading::Open, &mut below);
        assert!(below.is_empty());
        input.abort(&mut below);
        assert_eq!(below.pop(), Some(io::Request::Abort { entity: INPUT }));
        input.closed();
        assert!(!input.owns(INPUT));
        input.abort(&mut below);
        assert!(below.is_empty());
    }

    #[test]
    fn a_requested_window_settles_unread_once_the_input_closes() {
        let mut input = TerminalInput::Absent;
        let mut window = WindowRead::Idle;
        let mut below = Queue::uncapped();
        input.adopted(INPUT, false, &mut below);
        window.asked();
        window.asked();
        window.follow(input.open(), &mut below);
        assert_eq!(below.pop(), Some(io::Request::Window { owner: INPUT, stream: INPUT }));
        assert_eq!(window, WindowRead::Pending);
        window.asked();
        window.asked();
        assert_eq!(window, WindowRead::Again);
        input.close(&mut below);
        assert_eq!(below.pop(), Some(io::Request::Close { entity: INPUT }));
        input.closed();
        window.answered();
        assert!(window.requested());
        window.follow(input.open(), &mut below);
        assert_eq!(window, WindowRead::Idle);
        assert!(!window.requested());
        window.follow(input.open(), &mut below);
        assert!(below.is_empty());
    }
}
