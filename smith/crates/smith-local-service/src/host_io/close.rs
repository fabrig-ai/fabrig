//! The hub's permanent close, adopted roots and retained shutdown signal
//! (skein/docs/foundation/programming-model.md, 5.2; smith/docs/design/shell/README.md, 5).
//! It knows neither terminal demand nor child state. `begin`, root answers and
//! shutdown update the machine; `follow_close` submits roots in the down pass.

use super::{HostIo, file, file_layer::step, kernel, program_root};
use skein_lib::{Time, Token};

const ROOT_OWNER: Token = Token::new(u64::MAX);

/// How far the adopted roots have closed: one close in flight, last root first.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Roots {
    /// `left` roots wait for the file layer, and none is in flight.
    Next { left: u32 },
    /// One root's close awaits its terminal; `left` follow it.
    Waiting { left: u32 },
}

/// The hub's own permanent close.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Close {
    /// Nothing closes; a termination signal interrupts the chat.
    Open,
    /// Adopted roots close; `shutdown` holds a signal for the service.
    Closing { roots: Roots, shutdown: bool },
}

impl Close {
    pub(super) fn begin(&mut self, roots: u32) -> bool {
        match *self {
            Self::Open => {
                *self = Self::Closing { roots: Roots::Next { left: roots }, shutdown: false };
                true
            }
            Self::Closing { .. } => false,
        }
    }

    pub(super) const fn closing(&self) -> bool {
        match self {
            Self::Open => false,
            Self::Closing { .. } => true,
        }
    }

    pub(super) const fn roots_left(&self) -> u32 {
        match self {
            Self::Open => 0,
            Self::Closing { roots, .. } => match roots {
                Roots::Next { left } | Roots::Waiting { left } => *left,
            },
        }
    }

    pub(super) const fn root_due(&self) -> bool {
        match self {
            Self::Open => false,
            Self::Closing { roots, .. } => match roots {
                Roots::Next { left } => *left > 0,
                Roots::Waiting { .. } => false,
            },
        }
    }

    pub(super) fn next_root(&mut self) -> u32 {
        match self {
            Self::Open => unreachable!("roots close after the hub's close"),
            Self::Closing { roots, .. } => match *roots {
                Roots::Next { left } => {
                    let left = left.checked_sub(1).expect("positive roots to close");
                    *roots = Roots::Waiting { left };
                    left
                }
                Roots::Waiting { .. } => unreachable!("one root close at a time"),
            },
        }
    }

    pub(super) fn settle_root(&mut self, owner: Token) -> bool {
        match self {
            Self::Open => false,
            Self::Closing { roots, .. } => match *roots {
                Roots::Waiting { left } if owner == ROOT_OWNER => {
                    *roots = Roots::Next { left };
                    true
                }
                Roots::Next { .. } | Roots::Waiting { .. } => false,
            },
        }
    }

    pub(super) fn latch_shutdown(&mut self) -> bool {
        match self {
            Self::Open => false,
            Self::Closing { shutdown, .. } => {
                *shutdown = true;
                true
            }
        }
    }

    pub(super) fn take_shutdown(&mut self) -> bool {
        match self {
            Self::Open => false,
            Self::Closing { shutdown, .. } => core::mem::replace(shutdown, false),
        }
    }
}

impl HostIo {
    /// What closing implies for one down pass, under each layer's room conditions.
    pub(super) fn follow_close(&mut self, now: Time) {
        if self.close.root_due() && self.files.takes() {
            let position = self.close.next_root();
            let root =
                *self.file_roots.get(usize::try_from(position).expect("root position fits")).expect("adopted root");
            let env = self.file_env(now, skein_lib::Wall::EPOCH);
            step::down(
                &mut self.files,
                &env,
                file::Request::Close { owner: ROOT_OWNER, file: root },
                &mut self.file_events,
                &mut self.file_submissions,
            );
        }
        if self.close.closing() && self.edge_room() {
            let root = match &mut self.programs {
                Some(checks) => checks.closing_root(),
                None => None,
            };
            if let Some(fd) = root {
                self.forward(kernel::Submit { op: program_root(), kind: kernel::Op::Close { fd } });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_second_close_changes_nothing() {
        let mut close = Close::Open;
        assert!(close.begin(2));
        close.next_root();
        close.latch_shutdown();
        let held = close;
        assert!(!close.begin(9));
        assert_eq!(close, held);
    }

    #[test]
    fn roots_close_one_at_a_time_last_first() {
        let mut close = Close::Open;
        assert!(!close.root_due());
        assert_eq!(close.roots_left(), 0);
        close.begin(2);
        for position in [1, 0] {
            assert!(close.root_due());
            assert_eq!(close.next_root(), position);
            assert_eq!(close.roots_left(), position);
            assert!(!close.root_due());
            assert!(close.settle_root(ROOT_OWNER));
            assert!(!close.settle_root(ROOT_OWNER));
        }
        assert!(!close.root_due());
        assert!(close.closing());
    }

    #[test]
    fn a_foreign_file_terminal_answers_no_root() {
        let mut close = Close::Open;
        assert!(!close.settle_root(ROOT_OWNER));
        close.begin(1);
        close.next_root();
        let held = close;
        assert!(!close.settle_root(Token::new(1)));
        assert_eq!(close, held);
        assert!(close.settle_root(ROOT_OWNER));
    }

    #[test]
    fn a_shutdown_while_closing_waits_for_one_take() {
        let mut close = Close::Open;
        assert!(!close.latch_shutdown());
        assert!(!close.take_shutdown());
        close.begin(0);
        assert!(close.latch_shutdown());
        assert!(close.latch_shutdown());
        assert!(close.take_shutdown());
        assert!(!close.take_shutdown());
        assert!(close.closing());
    }
}
