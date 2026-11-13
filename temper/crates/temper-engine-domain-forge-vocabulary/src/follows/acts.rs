//! Owner acts retained until the follow's next decision (follows.md, 5).
use super::FollowRefusal;
use alloc::boxed::Box;
use skein_forge::api::Commit;

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum FollowAct {
    Start { after: Commit },
    Hold { after: Commit, reason: Box<[u8]> },
    Skip { after: Commit, reason: Box<[u8]> },
    Merge { afters: Box<[Commit]> },
}
impl FollowAct {
    #[must_use]
    pub fn afters(&self) -> &[Commit] {
        match self {
            Self::Start { after } | Self::Hold { after, .. } | Self::Skip { after, .. } => core::slice::from_ref(after),
            Self::Merge { afters } => afters,
        }
    }
}
impl FollowRefusal {
    #[must_use]
    pub const fn words(self) -> &'static [u8] {
        match self {
            Self::UnknownFollow => b"no such follow",
            Self::Stopped => b"the follow is stopped",
            Self::Skipped => b"this entry was skipped",
            Self::UnknownEntry => b"no queued entry has this commit; it may already be followed or skipped",
            Self::AlreadyStarted => b"the change has started producing",
            Self::NotConsecutive => b"merge needs consecutive entries in queue order",
            Self::MergeBound => b"merge needs two to forge.follow_merge entries",
            Self::ReasonBound => b"the reason exceeds forge.follow_reason_bytes",
            Self::NoWord => b"code starts this entry; it has no owner's hold to lift",
            Self::Pending => b"the follow has an owner act awaiting its step",
            Self::NotOwner => b"only the follow's owner or an owner of its project may act",
        }
    }
}
