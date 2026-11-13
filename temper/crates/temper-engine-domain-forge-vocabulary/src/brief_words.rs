//! Fixed reasons and counts for section reads (temper/docs/design/domain/forge/change/repairs.md).
//! A forge away pauses preparation without spending a try (section 5.11).
use alloc::boxed::Box;
use skein_forge::api::{Answer, Error};

/// How a section's unread reason counts (temper/docs/design/domain/forge/change/repairs.md).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum BriefFate {
    Capacity,
    Transient,
    Permanent,
}

/// Words for a section that could not be read (temper/docs/design/domain/forge/change/repairs.md).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct BriefUnread {
    pub fate: BriefFate,
    pub words: Box<[u8]>,
}

pub fn room() -> BriefUnread {
    BriefUnread {
        fate: BriefFate::Capacity,
        words: Box::from(b"the connector has no room for another read".as_slice()),
    }
}
pub fn busy() -> BriefUnread {
    BriefUnread { fate: BriefFate::Capacity, words: Box::from(b"the forge client is busy".as_slice()) }
}
pub fn moved() -> BriefUnread {
    BriefUnread { fate: BriefFate::Capacity, words: Box::from(b"the change's head moved".as_slice()) }
}
pub fn moved_transient() -> BriefUnread {
    BriefUnread { fate: BriefFate::Transient, words: Box::from(b"the change's head moved".as_slice()) }
}
pub fn unnamed() -> BriefUnread {
    BriefUnread {
        fate: BriefFate::Permanent,
        words: Box::from(b"the change cannot be named in its section".as_slice()),
    }
}
pub fn checks() -> BriefUnread {
    BriefUnread {
        fate: BriefFate::Permanent,
        words: Box::from(b"the failed checks do not fit their section".as_slice()),
    }
}
pub fn bounds() -> BriefUnread {
    BriefUnread { fate: BriefFate::Permanent, words: Box::from(b"the connector cannot hold this section".as_slice()) }
}

/// Static reason words; a measured bound stays in the forge error, not the reason.
pub fn reason_of(result: &Result<Answer, Error>) -> &'static str {
    refusal(result).1
}

fn refusal(result: &Result<Answer, Error>) -> (BriefFate, &'static str) {
    use BriefFate::{Capacity, Permanent, Transient};
    let error = match result {
        Ok(Answer::Pull(_)) => return (Capacity, "the change's head moved"),
        Ok(
            Answer::Located { .. }
            | Answer::Items { .. }
            | Answer::Item { .. }
            | Answer::Reviews { .. }
            | Answer::Statuses { .. }
            | Answer::Remarks { .. }
            | Answer::Commit(_)
            | Answer::Branches(_)
            | Answer::PullFiles { .. }
            | Answer::Compare { .. }
            | Answer::Checks(_)
            | Answer::Job { .. }
            | Answer::File { .. }
            | Answer::Directory { .. }
            | Answer::Protection(_)
            | Answer::Settings(_)
            | Answer::Collaborators { .. }
            | Answer::Permission(_)
            | Answer::Created(_)
            | Answer::Commented(_)
            | Answer::Reviewed(_)
            | Answer::Merged(_)
            | Answer::Queued
            | Answer::Branch(_)
            | Answer::Runs { .. }
            | Answer::Run(_)
            | Answer::Tree { .. }
            | Answer::Commits { .. }
            | Answer::Done,
        ) => {
            return (Permanent, "the forge answered another read");
        }
        Err(error) => error,
    };
    match error {
        Error::Busy => (Capacity, "the forge client is busy"),
        Error::NotConnected => (Capacity, "the forge is not connected"),
        Error::Unavailable => (Capacity, "the forge could not be reached"),
        Error::Timeout => (Capacity, "the forge did not answer in time"),
        Error::RateLimited { .. } => (Transient, "the forge is limiting calls"),
        Error::InvalidAnswer => (Permanent, "the forge's answer did not match its read"),
        Error::Forbidden => (Permanent, "the forge forbids this read"),
        Error::NotInstalled => (Permanent, "the forge App is not installed"),
        Error::Missing | Error::MissingJob => (Permanent, "the forge has no such object"),
        Error::TooLarge(_) => (Permanent, "the answer is past"),
        Error::Empty
        | Error::Full
        | Error::Exists
        | Error::NothingToMerge
        | Error::Closed
        | Error::Stale
        | Error::Conflict
        | Error::Protected
        | Error::Refused => (Permanent, "the forge refused this read"),
    }
}

pub fn unread_of(result: &Result<Answer, Error>) -> BriefUnread {
    let (fate, reason) = refusal(result);
    let words = if let Err(Error::TooLarge(excess)) = result {
        let mut words = skein_lib::List::uncapped();
        for byte in reason.as_bytes().iter().chain(b" ").chain(crate::bound_words(*excess).iter()) {
            words.push(*byte).expect("short measured refusal");
        }
        words.into_boxed()
    } else {
        Box::from(reason.as_bytes())
    };
    BriefUnread { fate, words }
}
