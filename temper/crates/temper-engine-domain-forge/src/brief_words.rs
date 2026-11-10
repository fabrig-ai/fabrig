//! Fixed reasons and counts for section reads (temper/docs/design/domain/forge.md, 8.4).
use alloc::boxed::Box;
use skein_forge::api::{Answer, Error};

/// How a section's unread reason counts (temper/docs/design/domain/forge.md, 8.4).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum BriefFate {
    Capacity,
    Transient,
    Permanent,
}

/// Words for a section that could not be read (temper/docs/design/domain/forge.md, 8.4).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct BriefUnread {
    pub fate: BriefFate,
    pub words: Box<[u8]>,
}

pub(crate) fn room() -> BriefUnread {
    BriefUnread {
        fate: BriefFate::Capacity,
        words: Box::from(b"the connector has no room for another read".as_slice()),
    }
}
pub(crate) fn busy() -> BriefUnread {
    BriefUnread { fate: BriefFate::Capacity, words: Box::from(b"the forge client is busy".as_slice()) }
}
pub(crate) fn moved() -> BriefUnread {
    BriefUnread { fate: BriefFate::Transient, words: Box::from(b"the change's head moved".as_slice()) }
}
pub(crate) fn unnamed() -> BriefUnread {
    BriefUnread {
        fate: BriefFate::Permanent,
        words: Box::from(b"the change cannot be named in its section".as_slice()),
    }
}
pub(crate) fn checks() -> BriefUnread {
    BriefUnread {
        fate: BriefFate::Permanent,
        words: Box::from(b"the failed checks do not fit their section".as_slice()),
    }
}
pub(crate) fn bounds() -> BriefUnread {
    BriefUnread { fate: BriefFate::Permanent, words: Box::from(b"the connector cannot hold this section".as_slice()) }
}

pub(crate) fn unread_of(result: &Result<Answer, Error>) -> BriefUnread {
    let error = match result {
        Ok(Answer::Pull(_)) => return moved(),
        Ok(_) => {
            return BriefUnread {
                fate: BriefFate::Permanent,
                words: Box::from(b"the forge answered another read".as_slice()),
            };
        }
        Err(error) => error,
    };
    let (fate, words): (BriefFate, &'static [u8]) = match error {
        Error::Busy => return busy(),
        Error::NotConnected => (BriefFate::Capacity, b"the forge is not connected"),
        Error::Unavailable => (BriefFate::Transient, b"the forge could not be reached"),
        Error::Timeout => (BriefFate::Transient, b"the forge did not answer in time"),
        Error::RateLimited { .. } => (BriefFate::Transient, b"the forge is limiting calls"),
        Error::InvalidAnswer => (BriefFate::Permanent, b"the forge's answer did not match its read"),
        Error::Forbidden => (BriefFate::Permanent, b"the forge forbids this read"),
        Error::NotInstalled => (BriefFate::Permanent, b"the forge App is not installed"),
        Error::Missing | Error::MissingJob => (BriefFate::Permanent, b"the forge has no such object"),
        Error::TooLarge(excess) => {
            let mut words = skein_lib::List::uncapped();
            for byte in b"the answer is past ".iter().chain(crate::bound_words(*excess).iter()) {
                words.push(*byte).expect("short measured refusal");
            }
            return BriefUnread { fate: BriefFate::Permanent, words: words.into_boxed() };
        }
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
    BriefUnread { fate, words: Box::from(words) }
}
