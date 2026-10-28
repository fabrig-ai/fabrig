//! forge run as Forgejo's parent (protocol/setup.md, 6.4).
/// A request to stop the supervised process.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stop {
    Terminate,
    Interrupt,
}
/// The child's final status.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    Code(u8),
    Signal(u32),
}
/// A signal's origin is resolved by io, outside this machine.
#[derive(Debug)]
pub enum Event {
    Shutdown { stop: Stop, from_forgejo: bool },
    Exited(Status),
}
/// The shell carries out each event's one act.
#[derive(Debug, PartialEq, Eq)]
pub enum Act {
    Nothing,
    Forward(Stop),
    End(u8),
}
/// Once ended, no later event changes the result or forwards a signal.
#[derive(Debug)]
pub struct Parent {
    ended: bool,
}
impl Parent {
    #[must_use]
    pub const fn running() -> Self {
        Self { ended: false }
    }
    #[must_use]
    pub fn step(&mut self, event: Event) -> Act {
        if self.ended {
            return Act::Nothing;
        }
        match event {
            Event::Shutdown { stop, from_forgejo } => {
                if from_forgejo {
                    Act::Nothing
                } else {
                    Act::Forward(stop)
                }
            }
            Event::Exited(status) => {
                self.ended = true;
                let code = match status {
                    Status::Code(code) => code,
                    Status::Signal(signal) => {
                        let code = signal.checked_add(128).expect("Unix signal plus 128 fits");
                        u8::try_from(code).expect("Unix signal exit fits one byte")
                    }
                };
                Act::End(code)
            }
        }
    }
}
