//! A failed proc read waits for the retained identity's exit before it is judged.
use std::io;

pub(super) enum Read<T> {
    Complete(io::Result<Option<T>>),
    Wait(io::Error),
}

/// `waited` means the bounded exit wait following a read failure has finished.
pub(super) fn observe<T>(read: io::Result<T>, exited: bool, waited: bool) -> Read<T> {
    if exited {
        return Read::Complete(Ok(None));
    }
    match read {
        Ok(executable) => Read::Complete(Ok(Some(executable))),
        Err(error) if !waited => Read::Wait(error),
        Err(error) => Read::Complete(Err(error)),
    }
}
