//! Literal cards drawn by the conversation (view.md, 4 and 9).
//! Task-page proposals remain beside the log unless that exact shared key
//! already has a card in a supplied turn, live call or decision.
use crate::contract::{ArchivedTurn, Block, LiveCall, ObjectKey, Turn, TurnRead};
use crate::thread::Thread;

impl Thread {
    /// Whether a shown conversation line or live call shows this object (view.md, 4.1).
    #[must_use]
    pub fn shows(&self, key: ObjectKey, live: &[crate::thread::LiveTurn]) -> bool {
        for line in &self.lines.rows {
            if let Some(ArchivedTurn { turn: TurnRead::Decoded(turn), .. }) = &line.decoded {
                if turn_shows(turn, key) {
                    return true;
                }
            }
        }
        for live in live {
            match &live.call {
                Some(LiveCall::Decoded(call)) => {
                    if call.objects.as_slice().contains(&key) {
                        return true;
                    }
                }
                Some(LiveCall::Raw(_)) | None => {}
            }
        }
        false
    }
}
fn turn_shows(turn: &Turn, key: ObjectKey) -> bool {
    for message in &turn.messages {
        for block in &message.blocks {
            match block {
                Block::Call(call) => {
                    if call.objects.as_slice().contains(&key) {
                        return true;
                    }
                }
                Block::Text(_) | Block::Result { .. } | Block::Opaque { .. } => {}
            }
        }
    }
    false
}
