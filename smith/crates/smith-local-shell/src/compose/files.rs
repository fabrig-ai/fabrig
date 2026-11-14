//! Exec's durable result file (shell/commands.md, sections 2–3).
//! The file owner holds teardown until its whole-file write and closes end.
//! It knows the output path and kernel terminals, not the run's outcome.
use super::{Refused, cleanup::EphemeralCleanup, host, root};
use crate::command::Usage;
use skein_io::file_layer::step;
use skein_io::{file, file_layer, kernel};
use skein_lib::{Duration, Env, Queue, Time, Token, Wall};
use skein_shell::{Clock, Host, Kernel};
use std::ffi::OsString;
use std::path::{Path, PathBuf};

const PATH_BYTES: u64 = 4096;

pub(crate) struct Files {
    pub(super) output: Option<Box<ResultFile>>,
    pub(super) temporary: Option<PathBuf>,
    created: bool,
    store_root: Option<kernel::Fd>,
    parent_root: Option<kernel::Fd>,
    target: Option<PathBuf>,
    maximum: u32,
    timeout: Duration,
}

impl Files {
    pub(crate) fn prepare(
        invocation: &crate::command::Exec,
        resolved: &crate::settings::resolve::Resolved,
        seed: u64,
    ) -> Result<Self, Refused> {
        let target = invocation.output.clone();
        if let Some(path) = &target {
            check_path(path)?;
            if path.file_name().is_none() {
                return Err(Refused::Usage(Usage("-o requires a file name".into())));
            }
        }
        let maximum = u32::try_from(resolved.limits.smith.run.outcome_bytes)
            .map_err(|_| Refused::Usage(Usage("the result file bound exceeds u32".into())))?
            .max(resolved.limits.show_bytes)
            .max(1);
        let temporary = if invocation.ephemeral {
            let base = ephemeral_base(std::env::var_os("XDG_RUNTIME_DIR"), std::env::var_os("TMPDIR"));
            let path = base.join(format!(".smith-exec-{}-{:016x}", std::process::id(), seed));
            check_path(&path)?;
            Some(path)
        } else {
            None
        };
        Ok(Self {
            target,
            temporary,
            created: false,
            store_root: None,
            parent_root: None,
            output: None,
            maximum,
            timeout: resolved.agent.service.limits.file_timeout,
        })
    }

    pub(crate) fn has_output(&self) -> bool {
        self.target.is_some()
    }

    pub(crate) fn teardown_bound(&self) -> Result<Duration, Refused> {
        if self.target.is_some() {
            // The whole-file store and the directory close each use FileIo's timeout.
            self.timeout
                .checked_add(self.timeout)
                .ok_or_else(|| Refused::Usage(Usage("the result file teardown bound overflows".into())))
        } else {
            Ok(Duration::ZERO)
        }
    }

    pub(super) fn prepare_output(&mut self, seed: u64) -> Result<(), Refused> {
        if let Some(path) = &self.target {
            self.output = Some(Box::new(ResultFile::open(path, self.maximum, self.timeout, seed)?));
        }
        Ok(())
    }

    pub(super) fn create_temporary(&mut self) -> Result<(), Refused> {
        use std::os::unix::fs::DirBuilderExt;
        if let Some(path) = &self.temporary {
            let base = path.parent().expect("private directory has a parent");
            std::fs::DirBuilder::new().recursive(true).mode(0o700).create(base).map_err(|why| host(&why))?;
            std::fs::DirBuilder::new().mode(0o700).create(path).map_err(|why| host(&why))?;
            self.created = true;
        }
        Ok(())
    }

    pub(super) fn store_root(&mut self, path: &Path) -> Result<kernel::Fd, Refused> {
        let descriptor = root(path)?;
        if self.created {
            self.store_root = Some(descriptor);
        }
        Ok(descriptor)
    }

    pub(super) fn parent_root(&mut self) -> Result<kernel::Fd, Refused> {
        let path = self.temporary.as_ref().expect("private directory configured");
        let descriptor = root(path.parent().expect("private directory has a parent"))?;
        self.parent_root = Some(descriptor);
        Ok(descriptor)
    }

    pub(super) fn attach_private(&mut self, local: &mut crate::Local) -> Result<(), Refused> {
        if let Some(path) = self.temporary.clone() {
            local.ephemeral(self.parent_root()?, &path).map_err(|_| {
                Refused::Local(crate::startup::Failure::Host { detail: "private parent cannot be adopted".into() })
            })?;
        }
        Ok(())
    }

    pub(super) fn composed(&mut self) {
        self.created = false;
        self.store_root = None;
        self.parent_root = None;
    }

    pub(super) fn close_startup(&mut self, kernel: &mut Kernel, clock: &Clock) {
        if let Some(mut output) = self.output.take() {
            output.close();
            let _exit = skein_shell::drive(kernel, clock, output.as_mut());
        }
        if self.created {
            let path = self.temporary.as_ref().expect("private directory created");
            let mut cleanup =
                EphemeralCleanup::new(path, self.store_root.take(), self.parent_root.take(), self.timeout);
            let _exit = skein_shell::drive(kernel, clock, &mut cleanup);
            if let Some(error) = cleanup.failure() {
                eprintln!("smith: warning: ephemeral directory {}: {error:?}", path.display());
            }
        }
    }
}

/// The base of `--ephemeral`'s private directory (shell/commands.md, section 2): `$XDG_RUNTIME_DIR` when set and non-empty, else `$TMPDIR`
/// when set and non-empty, else `/tmp`. Never the state directory, which
/// it is not given: a run's chat sits apart from the disk its builds fill.
fn ephemeral_base(runtime: Option<OsString>, temporary: Option<OsString>) -> PathBuf {
    runtime
        .filter(|value| !value.is_empty())
        .or_else(|| temporary.filter(|value| !value.is_empty()))
        .map_or_else(|| PathBuf::from("/tmp"), PathBuf::from)
}

pub(super) fn check_path(path: &Path) -> Result<(), Refused> {
    if path.as_os_str().len() >= usize::try_from(PATH_BYTES).expect("path bound fits") {
        return Err(Refused::Usage(Usage("a product file path exceeds 4095 bytes".into())));
    }
    Ok(())
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum State {
    Waiting,
    Saving,
    Closing,
    Closed,
}

/// The write or its later directory close failed, reported to product teardown.
pub(crate) enum Failure {
    /// The whole-file write or sync could not complete.
    Write(kernel::Error),
    /// The result has settled, but the directory's close failed.
    Close(kernel::Error),
}

/// The product's whole result write and the directory close it owes.
pub(crate) struct ResultFile {
    io: file_layer::FileIo,
    file_limits: file_layer::Limits,
    directory: Token,
    name: Box<[u8]>,
    path: PathBuf,
    request: Option<file::Request>,
    state: State,
    failure: Option<Failure>,
    events: Queue<file::Event>,
    completions: Queue<kernel::Complete>,
    submissions: Queue<kernel::Submit>,
}

impl ResultFile {
    fn new(io: file_layer::FileIo, file_limits: file_layer::Limits, directory: Token, path: &Path) -> Self {
        Self {
            io,
            file_limits,
            directory,
            name: path.file_name().expect("checked file name").as_encoded_bytes().into(),
            path: path.into(),
            request: None,
            state: State::Waiting,
            failure: None,
            events: Queue::with_capacity(4),
            completions: Queue::with_capacity(4),
            submissions: Queue::with_capacity(4),
        }
    }

    fn open(path: &Path, maximum: u32, timeout: Duration, seed: u64) -> Result<Self, Refused> {
        let parent = path.parent().filter(|parent| !parent.as_os_str().is_empty()).unwrap_or(Path::new("."));
        let file_limits = file_layer::Limits { files: 2, read: 4096, entries: 1, whole: maximum, timeout };
        let mut io = file_layer::FileIo::with_limits(&file_limits);
        let directory = io.adopt_root(root(parent)?).expect("one result directory");
        io.seed_randomness(seed);
        Ok(Self::new(io, file_limits, directory, path))
    }

    pub(crate) fn begin(&mut self, text: Box<[u8]>) {
        assert!(self.state == State::Waiting, "one result is offered to its file");
        self.state = State::Saving;
        self.request = Some(file::Request::Store {
            owner: Token::new(1),
            root: self.directory,
            path: self.name.clone(),
            bytes: text,
            expected: file::Expect::Any,
            no_follow: true,
        });
    }

    pub(crate) fn close(&mut self) {
        if self.state == State::Waiting {
            self.state = State::Closing;
        }
    }

    pub(crate) fn abort(&mut self) {
        match self.state {
            State::Saving => {
                if self.request.take().is_some() {
                    self.failure = Some(Failure::Write(kernel::Error::Cancelled));
                    self.state = State::Closing;
                } else {
                    file_layer::cancel(&mut self.io, Token::new(1), &mut self.submissions);
                }
            }
            State::Waiting | State::Closing | State::Closed => {}
        }
    }

    pub(crate) fn waiting(&self) -> bool {
        self.state == State::Waiting
    }
    pub(crate) fn failure(&mut self) -> Option<Failure> {
        self.failure.take()
    }
    pub(crate) fn path(&self) -> &Path {
        &self.path
    }

    fn failed(&mut self, error: kernel::Error) {
        match self.state {
            State::Saving => {
                self.failure = Some(Failure::Write(error));
                self.state = State::Closing;
            }
            State::Closing => {
                if self.failure.is_none() {
                    self.failure = Some(Failure::Close(error));
                }
                self.state = State::Closed;
            }
            State::Waiting | State::Closed => unreachable!("only active file requests fail"),
        }
    }
}

impl Host for ResultFile {
    fn iterate(&mut self, now: Time, wall: Wall) {
        let env = Env { now, wall, limits: self.file_limits };
        while let Some(complete) = self.completions.pop() {
            step::up(&mut self.io, &env, complete, &mut self.events, &mut self.submissions);
        }
        if self.io.is_due(now) {
            step::expire(&mut self.io, &env, &mut self.submissions);
        }
        while let Some(event) = self.events.pop() {
            match event {
                file::Event::Stored { .. } => self.state = State::Closing,
                file::Event::Closed { .. } => self.state = State::Closed,
                file::Event::Failed { error, .. } => self.failed(error),
                file::Event::Cancelled { .. } => self.failed(kernel::Error::Cancelled),
                file::Event::Removed { .. }
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
                | file::Event::Linked { .. }
                | file::Event::Renamed { .. }
                | file::Event::Listed { .. } => unreachable!("product file requests have whole-file terminals"),
            }
        }
        if self.io.takes() {
            let request = self.request.take().or_else(|| {
                (self.state == State::Closing)
                    .then(|| file::Request::Close { owner: Token::new(2), file: self.directory })
            });
            if let Some(request) = request {
                step::down(&mut self.io, &env, request, &mut self.events, &mut self.submissions);
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
        super::RESULT_FILE_OPERATIONS
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn file() -> ResultFile {
        let file_limits =
            file_layer::Limits { files: 2, read: 4096, entries: 1, whole: 4096, timeout: Duration::from_secs(1) };
        let mut io = file_layer::FileIo::with_limits(&file_limits);
        let directory = io.adopt_root(kernel::Fd::new(41)).unwrap();
        io.seed_randomness(1);
        ResultFile::new(io, file_limits, directory, Path::new("answer.txt"))
    }

    fn complete(file: &mut ResultFile, submit: kernel::Submit, result: Result<kernel::Done, kernel::Error>) {
        file.completions.push(kernel::Complete { op: submit.op, kind: submit.kind, result });
        file.iterate(Time::ZERO, Wall::from_nanos(0));
    }

    #[test]
    fn an_ephemeral_directory_is_made_under_the_runtime_then_the_temporary_directory() {
        // The state directory is not an input: no value of it moves the base.
        let cases = [
            (Some("/run/user/1000"), Some("/var/tmp"), "/run/user/1000"),
            (Some("/run/user/1000"), None, "/run/user/1000"),
            (None, Some("/var/tmp"), "/var/tmp"),
            (Some(""), Some("/var/tmp"), "/var/tmp"),
            (None, None, "/tmp"),
            (Some(""), None, "/tmp"),
            (None, Some(""), "/tmp"),
        ];
        for (runtime, temporary, base) in cases {
            assert_eq!(
                ephemeral_base(runtime.map(OsString::from), temporary.map(OsString::from)),
                Path::new(base),
                "runtime {runtime:?}, temporary {temporary:?}",
            );
        }
    }

    #[test]
    fn a_failed_result_write_closes_its_directory_before_it_is_empty() {
        let mut file = file();
        file.begin(b"answer".as_slice().into());
        file.iterate(Time::ZERO, Wall::from_nanos(0));
        let opening = file.submissions.pop().unwrap();
        complete(&mut file, opening, Err(kernel::Error::Permission));
        assert!(matches!(file.failure(), Some(Failure::Write(kernel::Error::Permission))));
        assert!(!file.is_empty());
        let closing = file.submissions.pop().unwrap();
        assert!(matches!(closing.kind, kernel::Op::Close { fd } if fd == kernel::Fd::new(41)));
        complete(&mut file, closing, Ok(kernel::Done::Nothing));
        assert!(file.is_empty());
    }

    #[test]
    fn an_interrupted_result_write_waits_for_the_cancel_and_completion() {
        let mut file = file();
        file.begin(b"answer".as_slice().into());
        file.iterate(Time::ZERO, Wall::from_nanos(0));
        let opening = file.submissions.pop().unwrap();
        file.abort();
        let cancelling = file.submissions.pop().unwrap();
        assert!(matches!(cancelling.kind, kernel::Op::Cancel { target } if target == opening.op));
        complete(&mut file, opening, Err(kernel::Error::Cancelled));
        assert!(!file.is_empty());
        complete(&mut file, cancelling, Ok(kernel::Done::Nothing));
        assert!(matches!(file.failure(), Some(Failure::Write(kernel::Error::Cancelled))));
        let closing = file.submissions.pop().unwrap();
        complete(&mut file, closing, Ok(kernel::Done::Nothing));
        assert!(file.is_empty());
    }

    #[test]
    fn an_unused_output_directory_is_closed_after_a_startup_refusal() {
        let mut file = file();
        file.close();
        file.iterate(Time::ZERO, Wall::from_nanos(0));
        let closing = file.submissions.pop().unwrap();
        assert!(matches!(closing.kind, kernel::Op::Close { fd } if fd == kernel::Fd::new(41)));
        complete(&mut file, closing, Ok(kernel::Done::Nothing));
        assert!(file.is_empty());
    }

    #[test]
    fn the_result_owner_keeps_four_kernel_operations() {
        let mut file = file();
        file.begin(vec![b'x'; 4096].into_boxed_slice());
        file.iterate(Time::ZERO, Wall::from_nanos(0));
        assert_eq!(file.operations(), 4);
    }
}
