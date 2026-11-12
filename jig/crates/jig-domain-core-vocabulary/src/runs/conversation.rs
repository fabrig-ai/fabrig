//! Opaque settled answers (domain/engine/executors.md, 2–3;
//! domain/hosts/README.md, 2). Protocol decoding and rendering remain with the root.

use crate::{CallKey, Limits};
use alloc::boxed::Box;
use skein_lib::length::len_u64;

/// The original host tool and answer; its name is its key's (protocol/store/records.md, 1).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct SettledCall {
    /// Core-issued order of this settled answer; the protocol supplies zero.
    pub serial: u64,
    pub tool: Box<[u8]>,
    pub answer: SettledAnswer,
}

/// A settled answer and its original key, carried together on replay (protocol/store/records.md, 1).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Settled {
    pub key: CallKey,
    pub call: SettledCall,
}

/// An answer is opaque conversation data, including replay evidence for delivery.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum SettledAnswer {
    Host { error: bool, body: Box<[u8]> },
    Delivery { outcome: DeliveryOutcome, evidence: Box<[u8]> },
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum DeliveryOutcome {
    Delivered,
    Nothing,
    Stale,
    Refused,
    Invalid,
    Failed,
}

impl SettledCall {
    #[must_use]
    pub fn owned_bytes(&self) -> Option<u64> {
        let answer = match &self.answer {
            SettledAnswer::Host { body, .. } => body.len(),
            SettledAnswer::Delivery { evidence, .. } => evidence.len(),
        };
        len_u64(self.tool.len()).checked_add(len_u64(answer))
    }

    #[must_use]
    pub fn valid(&self, limits: &Limits) -> bool {
        !self.tool.is_empty() && self.excess(limits).is_none()
    }

    #[must_use]
    pub fn excess(&self, limits: &Limits) -> Option<(u64, u64)> {
        let held = self.owned_bytes().unwrap_or(u64::MAX);
        let limit = u64::from(limits.call_answer_bytes);
        if held > limit { Some((held, limit)) } else { None }
    }
}
