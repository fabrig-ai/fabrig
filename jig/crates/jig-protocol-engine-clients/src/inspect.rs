//! Frozen reads for the clients' world (jig/docs/design/domain/testing-fakes.md, 1.1).
use super::{Component, Time, Token, Waiting, Which};
use skein_lib::length::usize_of;

impl Component {
    /// For its worlds' inspection adapter (skein/docs/design/testing/inspection.md, 2).
    #[must_use]
    pub const fn inspection_exchange_capacity(&self) -> u32 {
        self.exchanges_per_client
    }

    /// For its worlds' inspection adapter (skein/docs/design/testing/inspection.md, 2).
    #[must_use]
    pub fn inspection_exchange_count(&self) -> u32 {
        self.pending.exchanges.len()
    }

    /// For its worlds' inspection adapter (skein/docs/design/testing/inspection.md, 2).
    #[must_use]
    pub fn inspection_exchange(&self, index: u32) -> Option<(Token, Option<u64>)> {
        let (exchange, waiting) = self.pending.exchanges.iter().nth(usize_of(index))?;
        let sign_in = match waiting {
            Waiting::LinkAskWhose { sign_in, .. }
            | Waiting::Authorize { sign_in, .. }
            | Waiting::LinkGet { sign_in, .. }
            | Waiting::SignOut { sign_in, .. }
            | Waiting::Watch { sign_in, .. }
            | Waiting::Keep { sign_in, .. }
            | Waiting::Made { sign_in, .. }
            | Waiting::Moving { sign_in, .. }
            | Waiting::Read { sign_in, .. }
            | Waiting::ApplicationRead { sign_in, .. }
            | Waiting::Checked { sign_in, .. } => Some(*sign_in),
            Waiting::Ask { sign_in, .. } | Waiting::Carry { sign_in, .. } => {
                self.keyed_caller(*exchange, Some(*sign_in))
            }
            Waiting::Get { owner, .. } => match owner {
                super::services::GetOwner::SignIn(sign_in) => Some(*sign_in),
                super::services::GetOwner::Service(_) | super::services::GetOwner::LinkAskService(_) => None,
            },
            Waiting::Core { held } => *held,
            Waiting::Reissue { sign_in, .. } => self.keyed_caller(*exchange, *sign_in),
            Waiting::AskedWhose { .. } | Waiting::ApprovalWhose { .. } | Waiting::ApprovalCore { .. } => None,
        };
        Some((*exchange, sign_in))
    }

    /// For its worlds' inspection adapter (skein/docs/design/testing/inspection.md, 2).
    #[must_use]
    pub fn inspection_exchange_owner(&self, exchange: Token) -> Token {
        match self.members.get(&exchange) {
            Some(member) => member.batch,
            None => exchange,
        }
    }

    /// For its worlds' inspection adapter (skein/docs/design/testing/inspection.md, 2).
    #[must_use]
    pub fn inspection_link_deadline(&self, local: bool) -> Option<Time> {
        let which = if local { Which::Local } else { Which::Printed };
        Some(self.slot(which).live.as_ref()?.until)
    }

    /// For its worlds' inspection adapter (skein/docs/design/testing/inspection.md, 2).
    #[must_use]
    pub fn inspection_stream_count(&self) -> u32 {
        self.streams.len()
    }

    /// For its worlds' inspection adapter (skein/docs/design/testing/inspection.md, 2).
    #[must_use]
    pub fn inspection_stream(&self, index: u32) -> Option<(Token, Option<Time>)> {
        let (owner, stream) = self.streams.iter().nth(usize_of(index))?;
        let deadline = if !stream.closing() && stream.opened && !stream.waiting { Some(stream.deadline) } else { None };
        Some((*owner, deadline))
    }
}
