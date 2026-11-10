//! Pages use the owner's open directory as their cursor (runtime/io/files.md).
use super::{
    Complete, Done, Entry, Error, Event, Fd, FileIo, Limits, Op, Pending, Queue, Submit, Token, Vec, Work, issue, vec,
};
use alloc::boxed::Box;
use skein_lib::{Env, usize_of};

impl FileIo {
    fn list_at_most(&mut self, pending: Pending, fd: Fd, most: u32, subs: &mut Queue<Submit>) -> Work {
        issue(
            self,
            pending,
            Op::List {
                fd,
                entries: vec![crate::kernel::Entry::BLANK; usize_of(most.min(32))].into_boxed_slice(),
                names: vec![0_u8; 8192].into_boxed_slice(),
            },
            subs,
        )
    }
}

pub(super) fn page(
    io: &mut FileIo,
    env: &Env<Limits>,
    owner: Token,
    directory: Token,
    max: u32,
    events: &mut Queue<Event>,
    subs: &mut Queue<Submit>,
) -> Work {
    let Some(fd) = io.file(directory) else {
        return super::terminal_failure(events, owner, Error::NotFound);
    };
    if max == 0 || max > env.limits.entries {
        return super::terminal_failure(events, owner, Error::InvalidArgument);
    }
    io.list_at_most(Pending::Page { owner, fd, max, entries: Vec::with_capacity(usize_of(max)) }, fd, max, subs)
}

pub(super) fn answered(
    io: &mut FileIo,
    page: Pending,
    complete: Complete,
    stopped: Option<Error>,
    events: &mut Queue<Event>,
    subs: &mut Queue<Submit>,
) -> Work {
    let Pending::Page { owner, fd, max, mut entries } = page else { unreachable!("a page completion") };
    let count = match complete.result {
        Ok(Done::Count(count)) => count,
        Err(error) => {
            return super::terminal_failure(events, owner, stopped.unwrap_or(error));
        }
        Ok(_) => unreachable!("a list answers a count"),
    };
    let Op::List { entries: records, names, .. } = complete.kind else { unreachable!("a page lists") };
    for record in records.get(..usize_of(count)).expect("count within list") {
        let name = record.name(&names).expect("a listed name is within its buffer");
        entries.push(Entry { name: Box::from(name), kind: record.kind });
    }
    let held = u32::try_from(entries.len()).expect("a page is bounded by max");
    if count == 0 || held == max {
        events.push(Event::Listed { owner, entries: entries.into_boxed_slice() });
    } else if let Some(reason) = stopped {
        return super::terminal_failure(events, owner, reason);
    } else {
        return io.list_at_most(
            Pending::Page { owner, fd, max, entries },
            fd,
            max.checked_sub(held).expect("the list never takes past max"),
            subs,
        );
    }
    Work::Idle
}

/// A lock is synchronous and always retains its own answer, even after a stop.
pub(super) fn lock(
    io: &mut FileIo,
    owner: Token,
    root: Token,
    events: &mut Queue<Event>,
    subs: &mut Queue<Submit>,
) -> Work {
    let Some(fd) = io.file(root) else {
        return super::terminal_failure(events, owner, Error::NotFound);
    };
    issue(io, Pending::Lock(owner), Op::Lock { fd }, subs)
}

pub(super) fn locked(owner: Token, complete: Complete, events: &mut Queue<Event>) -> Work {
    match complete.result {
        Ok(Done::Nothing) => events.push(Event::Locked { owner }),
        Err(Error::Busy) => events.push(Event::Busy { owner }),
        Err(error) => return super::terminal_failure(events, owner, error),
        Ok(_) => unreachable!("a lock answers nothing"),
    }
    Work::Idle
}
