//! The scripted host's decision is separate from a signal sent to its agent.
use super::Parent;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Word {
    None,
    Cancel,
    Drain,
    HangUp,
    CancelFirst,
}

#[derive(Clone, Copy)]
pub(super) struct Plan {
    pub count: u32,
    pub before_admission: bool,
    pub word: Word,
    decided: bool,
}

impl Plan {
    pub fn new() -> Self {
        Self { count: 0, before_admission: false, word: Word::None, decided: false }
    }

    pub fn act(&mut self, parent: &mut Parent, calling: bool) {
        let told = parent.peer.observed().iter().any(|frame| frame.kind == smith_channel::kind::SIGNALLED);
        if self.word == Word::CancelFirst && calling && !self.decided {
            parent.cancel_pending = true;
            self.decided = true;
        }
        let ready = if self.word == Word::CancelFirst {
            parent.cancel_written && parent.status.is_none()
        } else if self.before_admission {
            parent.pidfd.is_some()
        } else {
            calling
        };
        if self.count > 0 && ready {
            parent.signal = parent.signal.max(if told { self.count } else { 1 });
        }
        if told && !self.decided {
            match self.word {
                Word::None | Word::CancelFirst => return,
                Word::Cancel => parent.cancel_pending = true,
                Word::Drain => parent.peer.play(
                    smith_channel::frame(smith_channel::Body::Drain(smith_channel::Drain { grace: None }))
                        .expect("drain frame"),
                ),
                Word::HangUp => {
                    if let Some(input) = parent.input.take() {
                        parent.close(input);
                    }
                }
            }
            self.decided = true;
        }
    }
}
