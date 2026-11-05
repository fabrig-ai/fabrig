//! Fixed reasons and counts for section reads (temper/docs/design/domain/forge.md, 8.4).
use skein_forge::api::{Answer, Error};

/// How a section's unread reason counts (temper/docs/design/domain/forge.md, 8.4).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum BriefFate {
    Capacity,
    Transient,
    Permanent,
}

/// Fixed words for a section that could not be read (temper/docs/design/domain/forge.md, 8.4).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct BriefUnread {
    pub fate: BriefFate,
    pub words: &'static [u8],
}

pub(crate) const ROOM: BriefUnread =
    BriefUnread { fate: BriefFate::Capacity, words: b"the connector has no room for another read" };
pub(crate) const BUSY: BriefUnread = BriefUnread { fate: BriefFate::Capacity, words: b"the forge client is busy" };
pub(crate) const MOVED: BriefUnread = BriefUnread { fate: BriefFate::Transient, words: b"the change's head moved" };
pub(crate) const UNNAMED: BriefUnread =
    BriefUnread { fate: BriefFate::Permanent, words: b"the change cannot be named in its section" };
pub(crate) const CHECKS: BriefUnread =
    BriefUnread { fate: BriefFate::Permanent, words: b"the failed checks do not fit their section" };
pub(crate) const BOUNDS: BriefUnread =
    BriefUnread { fate: BriefFate::Permanent, words: b"the connector cannot hold this section" };

pub(crate) fn unread_of(result: &Result<Answer, Error>) -> BriefUnread {
    let error = match result {
        Ok(Answer::Pull(_)) => return MOVED,
        Ok(_) => return BriefUnread { fate: BriefFate::Permanent, words: b"the forge answered another read" },
        Err(error) => error,
    };
    let (fate, words): (BriefFate, &'static [u8]) = match error {
        Error::Busy => return BUSY,
        Error::NotConnected => (BriefFate::Capacity, b"the forge is not connected"),
        Error::Unavailable => (BriefFate::Transient, b"the forge could not be reached"),
        Error::Timeout => (BriefFate::Transient, b"the forge did not answer in time"),
        Error::RateLimited { .. } => (BriefFate::Transient, b"the forge is limiting calls"),
        Error::InvalidAnswer => (BriefFate::Permanent, b"the forge's answer did not match its read"),
        Error::Forbidden => (BriefFate::Permanent, b"the forge forbids this read"),
        Error::NotInstalled => (BriefFate::Permanent, b"the forge App is not installed"),
        Error::Missing | Error::MissingJob => (BriefFate::Permanent, b"the forge has no such object"),
        Error::TooLarge => (BriefFate::Permanent, b"the answer is past the client's limit"),
        Error::Empty
        | Error::Full
        | Error::Exists
        | Error::NothingToMerge
        | Error::Closed
        | Error::Stale
        | Error::Conflict
        | Error::Protected
        | Error::Refused => (BriefFate::Permanent, b"the forge refused this read"),
    };
    BriefUnread { fate, words }
}
