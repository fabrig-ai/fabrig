//! Closes an unfinished `--ephemeral` directory's store, removes the directory,
//! and closes its parent (smith/docs/design/shell.md, 4.2).
//! It keeps the captured roots and the first kernel failure, not the run's outcome.
use skein_io::{file, file_layer, kernel};
use skein_lib::{Duration, Queue, Time, Token, Wall};
use skein_shell::Host;
use std::path::Path;

#[derive(Clone, Copy, PartialEq, Eq)]
enum State {
    StoreRoot,
    Directory,
    Closing,
    Closed,
}

/// Startup drives the captured store and parent through removal and close.
pub(super) struct EphemeralCleanup {
    io: file_layer::FileIo,
    directory: Option<Token>,
    name: Box<[u8]>,
    request: Option<file::Request>,
    state: State,
    failure: Option<kernel::Error>,
    events: Queue<file::Event>,
    completions: Queue<kernel::Complete>,
    submissions: Queue<kernel::Submit>,
}

impl EphemeralCleanup {
    pub(super) fn new(path: &Path, store: Option<kernel::Fd>, parent: Option<kernel::Fd>, timeout: Duration) -> Self {
        let mut io = file_layer::FileIo::with_whole_limit(2, 4096, 1, 1, timeout);
        let parent = match parent {
            Some(parent) => Ok(parent),
            None => skein_shell::open_root(path.parent().expect("private parent")).map_err(kernel::Error::Other),
        };
        let mut failure = None;
        let directory = match parent {
            Ok(parent) => Some(io.adopt_root(parent).expect("private parent slot")),
            Err(error) => {
                failure = Some(error);
                None
            }
        };
        let store = store.map(|root| io.adopt_root(root).expect("unfinished store slot"));
        let mut cleanup = Self {
            io,
            directory,
            name: path.file_name().expect("private directory name").as_encoded_bytes().into(),
            request: None,
            state: State::Closed,
            failure,
            events: Queue::with_capacity(4),
            completions: Queue::with_capacity(4),
            submissions: Queue::with_capacity(4),
        };
        match store {
            Some(file) => {
                cleanup.state = State::StoreRoot;
                cleanup.request = Some(file::Request::Close { owner: Token::new(1), file });
            }
            None if directory.is_some() => {
                cleanup.state = State::Directory;
                cleanup.request = Some(cleanup.remove_directory());
            }
            None => cleanup.state = State::Closed,
        }
        cleanup
    }

    fn remove_directory(&self) -> file::Request {
        file::Request::Remove {
            owner: Token::new(1),
            root: self
                .io
                .descriptor(self.directory.expect("private parent retained"))
                .expect("private parent descriptor"),
            name: self.name.clone(),
            directory: true,
        }
    }

    pub(super) fn failure(&mut self) -> Option<kernel::Error> {
        self.failure.take()
    }

    fn failed(&mut self, error: kernel::Error) {
        self.failure.get_or_insert(error);
        self.state = match self.state {
            State::StoreRoot | State::Directory if self.directory.is_some() => State::Closing,
            State::StoreRoot | State::Directory | State::Closing => State::Closed,
            State::Closed => unreachable!("only active cleanup requests fail"),
        };
    }
}

impl Host for EphemeralCleanup {
    fn iterate(&mut self, now: Time, _wall: Wall) {
        while let Some(complete) = self.completions.pop() {
            file_layer::up(&mut self.io, complete, &mut self.events, &mut self.submissions);
        }
        if self.io.is_due(now) {
            file_layer::expire(&mut self.io, now, &mut self.submissions);
        }
        while let Some(event) = self.events.pop() {
            match event {
                file::Event::Removed { .. } => self.state = State::Closing,
                file::Event::Closed { .. } if self.state == State::StoreRoot => {
                    if self.directory.is_some() {
                        self.state = State::Directory;
                        self.request = Some(self.remove_directory());
                    } else {
                        self.state = State::Closed;
                    }
                }
                file::Event::Closed { .. } => self.state = State::Closed,
                file::Event::Failed { error, .. } => self.failed(error),
                file::Event::Cancelled { .. } => self.failed(kernel::Error::Cancelled),
                file::Event::Stored { .. }
                | file::Event::Locked { .. }
                | file::Event::Busy { .. }
                | file::Event::Made { .. }
                | file::Event::Refused { .. }
                | file::Event::Opened { .. }
                | file::Event::Loaded { .. }
                | file::Event::Scanned { .. }
                | file::Event::Conflict { .. }
                | file::Event::TooLarge { .. }
                | file::Event::Stated { .. }
                | file::Event::Written { .. }
                | file::Event::Read { .. }
                | file::Event::Cut { .. }
                | file::Event::Synced { .. }
                | file::Event::Renamed { .. }
                | file::Event::Listed { .. } => {
                    unreachable!("ephemeral cleanup requests have removal and close terminals")
                }
            }
        }
        if self.io.takes() {
            let request = self.request.take().or_else(|| {
                (self.state == State::Closing).then(|| file::Request::Close {
                    owner: Token::new(2),
                    file: self.directory.expect("private parent retained"),
                })
            });
            if let Some(request) = request {
                file_layer::down(&mut self.io, now, request, &mut self.events, &mut self.submissions);
            }
        }
    }
    fn completions(&mut self) -> &mut Queue<kernel::Complete> {
        &mut self.completions
    }
    fn submissions(&mut self) -> &mut Queue<kernel::Submit> {
        &mut self.submissions
    }
    fn work_pending(&self, now: Time) -> bool {
        self.request.is_some()
            || !self.events.is_empty()
            || !self.completions.is_empty()
            || self.io.is_due(now)
            || (self.state == State::Closing && self.io.takes())
    }
    fn next_mechanism_deadline(&self) -> Option<Time> {
        self.io.next_deadline()
    }
    fn next_policy_deadline(&self) -> Option<Time> {
        None
    }
    fn is_empty(&self) -> bool {
        self.state == State::Closed
            && self.io.takes()
            && self.io.open_files() == 0
            && self.events.is_empty()
            && self.completions.is_empty()
            && self.submissions.is_empty()
    }
    fn operations(&self) -> u32 {
        4
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn complete(cleanup: &mut EphemeralCleanup, submit: kernel::Submit, result: Result<kernel::Done, kernel::Error>) {
        cleanup.completions.push(kernel::Complete { op: submit.op, kind: submit.kind, result });
        cleanup.iterate(Time::ZERO, Wall::from_nanos(0));
    }

    #[test]
    fn startup_cleanup_closes_a_captured_store_even_when_its_parent_cannot_open() {
        let path = Path::new("/dev/null/private");
        let mut cleanup = EphemeralCleanup::new(path, Some(kernel::Fd::new(41)), None, Duration::from_secs(1));
        cleanup.iterate(Time::ZERO, Wall::from_nanos(0));
        let closing = cleanup.submissions.pop().unwrap();
        assert!(matches!(closing.kind, kernel::Op::Close { fd } if fd == kernel::Fd::new(41)));
        complete(&mut cleanup, closing, Ok(kernel::Done::Nothing));
        assert!(cleanup.is_empty());
        assert!(matches!(cleanup.failure(), Some(kernel::Error::Other(20))));
    }

    #[test]
    fn startup_private_cleanup_settles_after_a_long_path_removal_fails() {
        let path = PathBuf::from(format!("{}private", "p/".repeat(2000)));
        let mut cleanup =
            EphemeralCleanup::new(&path, Some(kernel::Fd::new(41)), Some(kernel::Fd::new(42)), Duration::from_secs(1));
        cleanup.iterate(Time::ZERO, Wall::from_nanos(0));
        let closing = cleanup.submissions.pop().unwrap();
        complete(&mut cleanup, closing, Ok(kernel::Done::Nothing));
        let removing = cleanup.submissions.pop().unwrap();
        complete(&mut cleanup, removing, Err(kernel::Error::NotEmpty));
        let closing = cleanup.submissions.pop().unwrap();
        complete(&mut cleanup, closing, Ok(kernel::Done::Nothing));
        assert!(cleanup.is_empty());
    }
}
