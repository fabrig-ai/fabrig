//! Existing file opens and cuts (runtime/io/files.md).
use super::{FileIo, Pending, Work, issue, terminal_failure};
use crate::file::Event;
use crate::kernel::{Error, Fd, Op, OpenHow, Submit};
use alloc::boxed::Box;
use skein_lib::{Queue, Token};

pub(super) fn open_read(
    io: &mut FileIo,
    owner: Token,
    root: Fd,
    name: Box<[u8]>,
    how: OpenHow,
    events: &mut Queue<Event>,
    subs: &mut Queue<Submit>,
) -> Work {
    if name.contains(&0) {
        events.push(Event::failed(owner, Error::InvalidArgument));
        return Work::Idle;
    }
    if io.files.len() == io.files.capacity() {
        events.push(Event::failed(owner, Error::TooManyOpenFiles));
        return Work::Idle;
    }
    issue(io, Pending::OpenRead(owner), Op::Open { root, path: name, how }, subs)
}

pub(super) fn cut(
    io: &mut FileIo,
    owner: Token,
    file: Token,
    length: u64,
    events: &mut Queue<Event>,
    subs: &mut Queue<Submit>,
) -> Work {
    let Some(fd) = io.file(file) else {
        return terminal_failure(events, owner, Error::NotFound);
    };
    let op = Op::Truncate { fd, length };
    if !op.is_valid() {
        return terminal_failure(events, owner, Error::InvalidArgument);
    }
    issue(io, Pending::Cut(owner), op, subs)
}
