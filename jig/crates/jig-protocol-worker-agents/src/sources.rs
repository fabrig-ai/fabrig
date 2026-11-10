//! The real smith host half for source commands (jig/docs/design/domain/hosts/README.md, 5.1, and jig/docs/design/domain/hosts/hub.md, 8).
//! Paths and a whole environment are injected below the worker root; no login
//! contents or process environment are read here. Commands settle before reuse.
use alloc::boxed::Box;
use jig_domain_host::sources;
use jig_domain_link::{LoginMethod, LoginState, SignOutResult, SourceFailure, SourceName};
use skein_io::{self as io, file, kernel::Fd};
use skein_lib::{Queue, Time, Token};
use smith_host_domain as smith;
use smith_host_protocol::{ListLaunch, ListRead, ProgramSource, SourceAnswer, SourceAsk, SourceInspection};
use smith_protocol_program::Programs;

mod files;
pub use files::FileStage;

const ENVIRONMENT_ENTRIES: u32 = 64;
const ENVIRONMENT_BYTES: u64 = 4096;

/// Shell-provided paths, environment, effective user and programs parent.
pub struct Resources {
    programs: Option<Programs>,
    environment: Box<[Box<[u8]>]>,
    user: u32,
    parent: Option<Fd>,
    launch: Option<Box<ListLaunch>>,
}
// Command entries may contain credentials. Debug exposes only retained counts.
#[expect(
    clippy::missing_fields_in_debug,
    reason = "source resources expose counts alone, including after binding a user and parent"
)]
impl core::fmt::Debug for Resources {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Resources")
            .field("programs", &usize::from(self.programs.is_some()))
            .field("environment_entries", &self.environment.len())
            .finish()
    }
}
impl Resources {
    /// Bind the reference root's agent program and configuration to its list launch.
    pub fn list_launch(&mut self, launch: ListLaunch) {
        self.launch = Some(Box::new(launch));
    }
    /// The program's paths, each at most `path_bytes`, and its explicit environment.
    #[expect(clippy::manual_map, reason = "step code matches optional fields without callbacks")]
    #[must_use]
    pub fn new(
        programs: Programs,
        environment: Box<[Box<[u8]>]>,
        path_bytes: usize,
        user: u32,
        parent: Option<Fd>,
        launch: Option<ListLaunch>,
    ) -> Option<Self> {
        if programs.directory.len() > path_bytes
            || programs.claude.is_empty()
            || programs.claude.len() > path_bytes
            || programs.claude.contains(&0)
        {
            return None;
        }
        let configuration = smith_protocol_program::configuration_directory(&programs, &[b'x'; 32]).ok()?;
        // Validate both the retained base and the filtered child environment.
        if environment.len() > usize::try_from(ENVIRONMENT_ENTRIES).ok()? {
            return None;
        }
        let mut bytes = 0_usize;
        for entry in &environment {
            bytes = bytes.checked_add(entry.len())?;
        }
        if u64::try_from(bytes).ok()? > ENVIRONMENT_BYTES {
            return None;
        }
        smith_protocol_program::environment(&environment, &configuration).ok()?;
        let launch = match launch {
            Some(launch) => Some(Box::new(launch)),
            None => None,
        };
        Some(Self { programs: Some(programs), environment, user, parent, launch })
    }
}
#[derive(Debug)]
struct Command {
    owner: Token,
    io_owner: Token,
    source: smith::SourceName,
    ask: SourceAsk,
    root: Fd,
    deadline: Time,
    inspection: Option<SourceInspection>,
    child: Option<ProgramSource>,
}
#[derive(Debug)]
struct Reading {
    read: u64,
    child: ListRead,
}
/// One serialized source command or list read, settled before reuse.
#[derive(Debug)]
pub struct Component {
    resources: Option<Resources>,
    command: Option<Command>,
    reading: Option<Reading>,
    detail: u32,
}
impl Component {
    /// Keep the host bound on redacted failure detail.
    #[must_use]
    pub fn new(detail: u32) -> Self {
        Self { resources: None, command: None, reading: None, detail }
    }
    /// Adopt shell resources before any source command starts.
    pub fn configure(&mut self, resources: Resources) -> Result<(), Resources> {
        if !self.settled() || self.resources.is_some() {
            return Err(resources);
        }
        self.resources = Some(resources);
        Ok(())
    }
    /// Bind a list launch independently of optional program-login resources.
    #[expect(clippy::manual_map, reason = "step code matches optional fields without callbacks")]
    pub fn list_launch(&mut self, launch: Option<ListLaunch>) {
        if self.resources.is_none() && launch.is_some() {
            self.resources =
                Some(Resources { programs: None, environment: Box::default(), user: 0, parent: None, launch: None });
        }
        if let Some(resources) = &mut self.resources {
            resources.launch = match launch {
                Some(launch) => Some(Box::new(launch)),
                None => None,
            };
        }
    }
    /// Whether an I/O or file owner belongs to this source operation.
    #[must_use]
    pub fn owns(&self, owner: Token) -> bool {
        if let Some(reading) = &self.reading {
            return reading.child.owns(owner);
        }
        match &self.command {
            Some(command) => match &command.child {
                Some(child) => child.owns(owner),
                None => command.io_owner == owner,
            },
            None => false,
        }
    }
    /// Whether no source command or inspection remains active.
    #[must_use]
    pub fn settled(&self) -> bool {
        self.command.is_none() && self.reading.is_none()
    }
    /// Inspect a check's directories before spawning; sign-out runs directly.
    #[expect(clippy::too_many_arguments, reason = "the host's source binding and lower queues travel together")]
    pub fn start(
        &mut self,
        owner: Token,
        io_owner: Token,
        name: SourceName,
        operation: sources::Operation,
        root: Fd,
        deadline: Time,
        below: &mut Queue<io::Request>,
        files: &mut Queue<file::Request>,
    ) -> Option<(Token, sources::Answer)> {
        assert!(self.settled(), "host serializes source commands");
        let Some(resources) = &self.resources else {
            return Some(failed(owner, operation, SourceFailure::Unconfigured, b"no program configured", self.detail));
        };
        let Some(programs) = &resources.programs else {
            return Some(failed(owner, operation, SourceFailure::Unconfigured, b"no program configured", self.detail));
        };
        let Some(source) = smith::SourceName::new(name.as_bytes()) else {
            return Some(failed(owner, operation, SourceFailure::Unconfigured, b"invalid source name", self.detail));
        };
        let ask = match operation {
            sources::Operation::Check => SourceAsk::Check,
            sources::Operation::SignOut => SourceAsk::SignOut,
        };
        let inspection = match ask {
            SourceAsk::Check => {
                let Some(parent) = resources.parent else {
                    return Some(failed(
                        owner,
                        operation,
                        SourceFailure::Unconfigured,
                        b"programs parent directory is unavailable",
                        self.detail,
                    ));
                };
                let name = directory_name(&programs.directory);
                Some(SourceInspection::new(io_owner, parent, Box::from(name), source, resources.user, deadline, files))
            }
            SourceAsk::SignOut => None,
        };
        self.command = Some(Command { owner, io_owner, source, ask, root, deadline, inspection, child: None });
        self.advance(below)
    }
    /// Deliver one owned child event and take a settled answer.
    pub fn from_io(&mut self, event: io::Event, below: &mut Queue<io::Request>) -> Option<(Token, sources::Answer)> {
        if let Some(reading) = &mut self.reading {
            reading.child.from_io(event, below);
            return self.read_answer();
        }
        self.command
            .as_mut()
            .expect("owned source event")
            .child
            .as_mut()
            .expect("spawned source")
            .from_io(event, below);
        self.advance(below)
    }
    /// Deliver one owned inspection result and launch only after private readiness.
    pub fn from_file(
        &mut self,
        event: file::Event,
        below: &mut Queue<io::Request>,
        files: &mut Queue<file::Request>,
    ) -> Option<(Token, sources::Answer)> {
        self.command
            .as_mut()
            .expect("owned inspection")
            .inspection
            .as_mut()
            .expect("inspection active")
            .from_file(event, files);
        self.advance(below)
    }
    /// Enforce the check's deadline across inspection and the child command.
    pub fn fire(&mut self, now: Time, below: &mut Queue<io::Request>) -> Option<(Token, sources::Answer)> {
        if let Some(reading) = &mut self.reading {
            reading.child.fire(now, below);
            return self.read_answer();
        }
        if let Some(command) = &mut self.command {
            if let Some(inspection) = &mut command.inspection {
                inspection.fire(now);
            }
            if let Some(child) = &mut command.child {
                child.fire(now, below);
            }
        }
        self.advance(below)
    }
    /// The active inspection or child's next deadline.
    #[must_use]
    pub fn next_deadline(&self) -> Option<Time> {
        if let Some(reading) = &self.reading {
            return reading.child.next_deadline();
        }
        let command = self.command.as_ref()?;
        match &command.inspection {
            Some(inspection) => inspection.next_deadline(),
            None => command.child.as_ref()?.next_deadline(),
        }
    }
    /// Start one list read with a value taken from the worker's table at this call.
    pub fn read(
        &mut self,
        read: u64,
        io_owner: Token,
        endpoint: u32,
        credential: Option<smith_channel::CredentialValue>,
        deadline: Time,
        below: &mut Queue<io::Request>,
    ) -> Option<(Token, sources::Answer)> {
        assert!(self.settled(), "host serializes list reads and source commands");
        let failed = Some((
            Token::new(read),
            sources::Answer::Read(jig_domain_link::ListOutcome::Failed(jig_domain_link::ListFailure::Agent)),
        ));
        let Some(resources) = &self.resources else { return failed };
        let Some(launch) = &resources.launch else { return failed };
        let Some(credential) = credential else { return failed };
        match ListRead::new(io_owner, launch.as_ref().clone(), endpoint, credential, deadline, below) {
            Ok(child) => self.reading = Some(Reading { read, child }),
            Err(_) => return failed,
        }
        self.read_answer()
    }
    /// Cancel the child while retaining ownership until all its resources settle.
    pub fn cancel_read(&mut self, read: u64, below: &mut Queue<io::Request>) -> Option<(Token, sources::Answer)> {
        if let Some(reading) = &mut self.reading
            && reading.read == read
        {
            reading.child.cancel(below);
        }
        self.read_answer()
    }
    fn read_answer(&mut self) -> Option<(Token, sources::Answer)> {
        let reading = self.reading.as_mut()?;
        if !reading.child.settled() {
            return None;
        }
        // A cancelled child's terminal only releases the host's queue.
        let outcome = match reading.child.answer() {
            Some(answer) => crate::route::list_outcome(answer),
            None => jig_domain_link::ListOutcome::Failed(jig_domain_link::ListFailure::Agent),
        };
        let read = reading.read;
        self.reading = None;
        Some((Token::new(read), sources::Answer::Read(outcome)))
    }
    /// Transfer the shell-opened parent for closure after the worker settles.
    pub fn take_parent(&mut self) -> Option<Fd> {
        assert!(self.settled(), "source resources settle before closure");
        self.resources.as_mut()?.parent.take()
    }
    fn advance(&mut self, below: &mut Queue<io::Request>) -> Option<(Token, sources::Answer)> {
        let command = self.command.as_mut()?;
        if let Some(inspection) = &mut command.inspection {
            match inspection.answer()? {
                Ok(()) => command.inspection = None,
                Err(check) => return Some(self.finish(SourceAnswer::Check(check))),
            }
        }
        if command.child.is_none() {
            let resources = self.resources.as_ref().expect("configured command");
            match ProgramSource::new(
                command.io_owner,
                command.ask,
                resources.programs.as_ref().expect("configured source program"),
                command.source,
                &resources.environment,
                command.root,
                Box::from(b".".as_slice()),
                command.deadline,
                self.detail,
                below,
            ) {
                Ok(child) => command.child = Some(child),
                Err(error) => {
                    let operation = match command.ask {
                        SourceAsk::Check => sources::Operation::Check,
                        SourceAsk::SignOut => sources::Operation::SignOut,
                    };
                    let answer = failed(
                        command.owner,
                        operation,
                        SourceFailure::Unconfigured,
                        binding_error(error),
                        self.detail,
                    );
                    self.command = None;
                    return Some(answer);
                }
            }
        }
        let answer = command.child.as_mut()?.answer()?;
        Some(self.finish(answer))
    }
    fn finish(&mut self, answer: SourceAnswer) -> (Token, sources::Answer) {
        let owner = self.command.take().expect("one active command").owner;
        (owner, translate(answer, self.detail))
    }
}
fn binding_error(error: smith_protocol_program::Error) -> &'static [u8] {
    match error {
        smith_protocol_program::Error::Name => b"invalid source name",
        smith_protocol_program::Error::Directory => b"invalid programs directory",
        smith_protocol_program::Error::Environment => b"invalid program environment",
    }
}
pub(crate) fn translate(answer: SourceAnswer, bound: u32) -> sources::Answer {
    match answer {
        SourceAnswer::Check(check) => {
            let (state, email) = match check {
                smith::ProgramCheck::Usable { email } => (LoginState::Usable, email),
                smith::ProgramCheck::NotSignedIn => (LoginState::NotSignedIn, None),
                smith::ProgramCheck::SignedInOtherwise(method) => (
                    LoginState::SignedInOtherwise(
                        LoginMethod::new(method.as_bytes()).expect("same printable method grammar"),
                    ),
                    None,
                ),
                smith::ProgramCheck::Failed(failure) => (
                    LoginState::Failed { reason: reason(failure.reason), detail: bounded(failure.detail, bound) },
                    None,
                ),
            };
            sources::Answer::Checked { state, email }
        }
        SourceAnswer::SignOut(sign_out) => sources::Answer::SignedOut(match sign_out {
            smith::ProgramSignOut::SignedOut => SignOutResult::SignedOut,
            smith::ProgramSignOut::Failed(failure) => {
                SignOutResult::Failed { reason: reason(failure.reason), detail: bounded(failure.detail, bound) }
            }
        }),
    }
}

fn reason(reason: smith::FailureReason) -> SourceFailure {
    match reason {
        smith::FailureReason::Missing => SourceFailure::Missing,
        smith::FailureReason::Deadline => SourceFailure::Deadline,
        smith::FailureReason::Unreadable => SourceFailure::Unreadable,
        smith::FailureReason::Exited => SourceFailure::Exited,
    }
}
fn failed(
    owner: Token,
    operation: sources::Operation,
    reason: SourceFailure,
    detail: &[u8],
    bound: u32,
) -> (Token, sources::Answer) {
    let detail = Box::from(detail.get(..detail.len().min(skein_lib::length::usize_of(bound))).expect("bounded detail"));
    let answer = match operation {
        sources::Operation::Check => {
            sources::Answer::Checked { state: LoginState::Failed { reason, detail }, email: None }
        }
        sources::Operation::SignOut => sources::Answer::SignedOut(SignOutResult::Failed { reason, detail }),
    };
    (owner, answer)
}

fn directory_name(path: &[u8]) -> &[u8] {
    let mut end = path.len();
    for byte in path.iter().rev() {
        if *byte != b'/' {
            break;
        }
        end = end.checked_sub(1).expect("path length");
    }
    if end == 0 {
        return b".";
    }
    let path = path.get(..end).expect("trimmed path");
    let mut start = 0;
    for (index, byte) in path.iter().enumerate() {
        if *byte == b'/' {
            start = index.checked_add(1).expect("path bound");
        }
    }
    path.get(start..).expect("last path component")
}

fn bounded(bytes: Box<[u8]>, bound: u32) -> Box<[u8]> {
    let bound = skein_lib::length::usize_of(bound);
    if bytes.len() <= bound { bytes } else { Box::from(bytes.get(..bound).expect("bounded failure detail")) }
}
