//! Real-box terminal stories (temper/docs/design/protocol/setup/box.md, 2, and temper/docs/design/protocol/setup/testing.md, 4).
use crate::support::{self, parent::Guard};
use skein_io::kernel::{Complete, Done, Fd, Op, Submit};
use skein_lib::{Queue, Time, Token, Wall};
use skein_shell::{Delegate, Host};
use skein_world::{
    Referee,
    end_to_end::{Binary, Streams},
    real,
};
use std::{
    fs,
    io::Read,
    net::SocketAddr,
    os::{
        fd::AsRawFd,
        unix::fs::{DirBuilderExt, OpenOptionsExt},
    },
    path::Path,
    process::{Command, Stdio},
};

pub fn command(root: &Path, action: &str) -> Command {
    let mut command = Command::new(skein_scratch::paths::binary("temper-box"));
    if !action.is_empty() {
        command.arg(action);
    }
    for name in ["config", "data", "cache"] {
        command.arg(format!("--{name}")).arg(root.join(name));
    }
    isolate(&mut command, root);
    command
}

fn isolate(command: &mut Command, root: &Path) {
    let runtime = root.join("runtime");
    fs::DirBuilder::new().recursive(true).mode(0o700).create(&runtime).expect("private runtime");
    command.env("XDG_RUNTIME_DIR", runtime);
    let path = command
        .get_envs()
        .find(|(name, _)| *name == "PATH")
        .and_then(|(_, value)| value.map(ToOwned::to_owned))
        .unwrap_or_else(|| temper_test_env::value(&temper_shell::environment::PATH).unwrap_or_default());
    let binary = skein_scratch::paths::binary("temper-box");
    let paths =
        std::iter::once(binary.parent().expect("binary directory").to_path_buf()).chain(std::env::split_paths(&path));
    command.env("PATH", std::env::join_paths(paths).expect("fixture PATH"));
}

pub fn footer(link: &[u8]) -> Vec<u8> {
    [FOOTER, format!("In a terminal: temper-box http://{}\n", address(link)).as_bytes()].concat()
}

pub fn link_path(root: &Path) -> std::path::PathBuf {
    use std::os::unix::ffi::OsStrExt;
    temper_box_shell::link_file::path(
        root.join("data").as_os_str().as_bytes(),
        Some(root.join("runtime").as_os_str().as_bytes()),
        cfg!(target_os = "linux"),
    )
    .expect("fixture link path")
}

pub const PRELUDE: &[u8] = b"Open this link in a browser on this machine.\n\n";
pub const FOOTER: &[u8] = b"\nIt signs you in as this temper's owner, once, within 2 hours.\nOpened after that, it prints a fresh one here.\n";

pub struct Boxed {
    pub scratch: support::Scratch,
    pub guard: Guard,
    stdout: std::process::ChildStdout,
    printed: Vec<u8>,
}
impl Boxed {
    pub fn start(case: &str, path: Option<&str>) -> Self {
        let mut command = Command::new(skein_scratch::paths::binary("temper-box"));
        if let Some(path) = path {
            command.env("PATH", path);
        }
        Self::with_command(case, command)
    }
    pub fn with_command(case: &str, mut command: Command) -> Self {
        let scratch = support::Scratch::startup(case);
        isolate(&mut command, scratch.path());
        let stderr = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(scratch.path().join("stderr"))
            .expect("private stderr capture");
        let mut child = command
            .arg("--config")
            .arg(scratch.path().join("config"))
            .arg("--data")
            .arg(scratch.path().join("data"))
            .arg("--cache")
            .arg(scratch.path().join("cache"))
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(stderr)
            .spawn()
            .expect("real box");
        let stdout = child.stdout.take().expect("box stdout");
        Self { scratch, guard: Guard::new(child), stdout, printed: Vec::new() }
    }
    pub fn root(&self) -> &Path {
        self.scratch.path()
    }
    pub fn wait(&mut self, mut ready: impl FnMut(&mut Self) -> bool) {
        let mut patience = support::patience::Patience::arm("the real box story", support::WITHIN);
        patience.watch_process(self.guard.child.as_ref().expect("box child").id());
        while !ready(self) {
            assert!(
                self.guard.child.as_mut().expect("box child").try_wait().expect("box status").is_none(),
                "box stays up"
            );
            if let Some(figures) = patience.failed() {
                panic!("box story: {figures}");
            }
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
    }
    pub fn link(&mut self) -> Vec<u8> {
        let mut link = Vec::new();
        let mut patience = support::patience::Patience::arm("the box's link", support::WITHIN);
        patience.watch_process(self.guard.child.as_ref().expect("box child").id());
        loop {
            if let Some(figures) = patience.failed() {
                panic!("box link: {figures}");
            }
            if !skein_shell::keeper::wait_readable(
                Fd::new(self.stdout.as_raw_fd()),
                skein_lib::Duration::from_millis(10),
            )
            .expect("stdout readiness")
            {
                continue;
            }
            let mut byte = [0];
            assert_eq!(self.stdout.read(&mut byte).expect("link byte"), 1, "box prints a whole link");
            self.printed.push(byte[0]);
            if byte[0] == b'\n' {
                break;
            }
            link.push(byte[0]);
            assert!(link.len() <= 4096, "bounded link");
        }
        let _address = address(&link);
        link
    }
    pub fn stderr(&self) -> Vec<u8> {
        fs::read(self.root().join("stderr")).expect("captured stderr")
    }
    pub fn stop(mut self) -> (Vec<u8>, Vec<u8>, String) {
        support::parent::signal(self.guard.child.as_mut().expect("box child"), "TERM");
        let mut status = None;
        self.wait(|boxed| {
            status = boxed.guard.child.as_mut().expect("box child").try_wait().expect("box stopped");
            status.is_some()
        });
        assert!(status.expect("box exit").success(), "box drained successfully");
        self.stdout.read_to_end(&mut self.printed).expect("remaining stdout");
        let stderr = self.stderr();
        let log = support::boxed::box_log(&self.root().join("data"));
        assert!(!link_path(self.root()).exists(), "serve removes its link before exiting");
        self.scratch.remove();
        (self.printed, stderr, log)
    }
}

pub fn address(link: &[u8]) -> SocketAddr {
    let url = std::str::from_utf8(link).expect("link UTF-8");
    let authority = url.strip_prefix("http://").expect("unprefixed HTTP link").split('/').next().expect("authority");
    let address: SocketAddr = authority.parse().expect("literal loopback authority");
    assert!(address.ip().is_loopback(), "only the fixture's loopback listener");
    assert!(url.contains("/jig/v1/sign-in/link?secret="), "sign-in route");
    address
}

const READ: Token = Token::new(1);
const WRITE: Token = Token::new(2);
struct Terminal {
    input: Option<Fd>,
    output: Option<Fd>,
    reading: bool,
    writing: bool,
    closing: u32,
    next_close: u64,
    bytes: Vec<u8>,
    owner: bool,
    complete: Queue<Complete>,
    submit: Queue<Submit>,
}
impl Terminal {
    fn close(&mut self, fd: Fd) {
        self.closing += 1;
        self.next_close += 1;
        self.submit.push(Submit { op: Token::new(self.next_close), kind: Op::Close { fd } });
    }
}
impl Host for Terminal {
    fn iterate(&mut self, _now: Time, _wall: Wall) {
        while let Some(done) = self.complete.pop() {
            if done.op == READ {
                self.reading = false;
                let Op::PipeRead { buf, .. } = done.kind else { panic!("terminal read") };
                let Done::Count(count) = done.result.expect("terminal stdout") else { panic!("read count") };
                self.bytes.extend_from_slice(&buf[..usize::try_from(count).expect("bounded count")]);
                assert!(self.bytes.len() <= 262_144, "bounded terminal output");
                if count == 0 {
                    let fd = self.output.take().expect("terminal output");
                    self.close(fd);
                    if let Some(fd) = self.input.take() {
                        self.close(fd);
                    }
                }
            } else if done.op == WRITE {
                self.writing = false;
                done.result.expect("quit written");
                let fd = self.input.take().expect("terminal input");
                self.close(fd);
            } else {
                done.result.expect("terminal pipe closed");
                self.closing -= 1;
            }
        }
        if !self.owner
            && skein_lib::bytes::find(&self.bytes, b"Owner").is_some()
            && skein_lib::bytes::find(&self.bytes, b"Live").is_some()
        {
            self.owner = true;
            self.writing = true;
            self.submit.push(Submit {
                op: WRITE,
                kind: Op::PipeWrite { fd: self.input.expect("open stdin"), bytes: b"q\n".as_slice().into(), from: 0 },
            });
        }
        if !self.reading
            && let Some(fd) = self.output
        {
            self.reading = true;
            self.submit.push(Submit { op: READ, kind: Op::PipeRead { fd, buf: vec![0; 4096].into_boxed_slice() } });
        }
    }
    fn completions(&mut self) -> &mut Queue<Complete> {
        &mut self.complete
    }
    fn submissions(&mut self) -> &mut Queue<Submit> {
        &mut self.submit
    }
    fn work_pending(&self, _now: Time) -> bool {
        !self.complete.is_empty() || !self.submit.is_empty()
    }
    fn next_mechanism_deadline(&self) -> Option<Time> {
        None
    }
    fn next_policy_deadline(&self) -> Option<Time> {
        None
    }
    fn is_empty(&self) -> bool {
        self.input.is_none() && self.output.is_none() && self.closing == 0
    }
    fn operations(&self) -> u32 {
        4
    }
}
enum Process {
    Open(Box<Binary>),
    Terminal(Box<Terminal>),
}
impl Delegate for Process {
    fn host(&self) -> &dyn Host {
        match self {
            Self::Open(x) => x.as_ref(),
            Self::Terminal(x) => x.as_ref(),
        }
    }
    fn host_mut(&mut self) -> &mut dyn Host {
        match self {
            Self::Open(x) => x.as_mut(),
            Self::Terminal(x) => x.as_mut(),
        }
    }
}
struct Judge {
    patience: support::patience::Patience,
    settled: bool,
}
impl Referee<Process> for Judge {
    fn act(&mut self, _now: Time, _procs: &mut [Process]) {}
    fn observe(&mut self, _now: Time, procs: &[Process]) {
        self.settled = procs.iter().all(Host::is_empty);
        if !self.settled
            && let Some(figures) = self.patience.failed()
        {
            panic!("terminal sign-in: {figures}");
        }
    }
    fn next_deadline(&self) -> Option<Time> {
        if self.settled { None } else { self.patience.until() }
    }
    fn passed(&self) -> bool {
        self.settled
    }
}
/// Open the supplied link with the shipped terminal, without printing its bytes or credentials.
pub fn signs_in(root: &Path, private: &str, link: &[u8]) -> bool {
    let mut open = support::terminal::open(root, &root.join(private), link);
    let streams = open.take_streams().expect("terminal pipes");
    let Streams::Pipes { input, output } = streams else { panic!("piped terminal") };
    let mut world = real::World::new(Judge {
        patience: support::patience::Patience::watching("terminal sign-in", support::WITHIN, &open),
        settled: false,
    });
    world.spawn_with_fds(open.descriptors(), || Process::Open(Box::new(open)));
    world.spawn_with_fds(streams.descriptors(), || {
        Process::Terminal(Box::new(Terminal {
            input: Some(input),
            output: Some(output),
            reading: false,
            writing: false,
            closing: 0,
            next_close: 2,
            bytes: Vec::new(),
            owner: false,
            complete: Queue::with_capacity(4),
            submit: Queue::with_capacity(4),
        }))
    });
    let outcome = world.run(&skein_shell::Clock::new(), support::WITHIN);
    let Process::Terminal(terminal) = &outcome.procs[1] else { panic!("terminal output") };
    let Process::Open(open) = &outcome.procs[0] else { panic!("terminal process") };
    if terminal.owner {
        assert_eq!(open.exit_status(), Some(skein_io::kernel::Exit::Code(0)), "signed-in terminal quits cleanly");
    } else {
        assert_eq!(open.exit_status(), Some(skein_io::kernel::Exit::Code(1)), "refused terminal exits unsuccessfully");
        assert!(
            open.stderr() == b"temper open: the link was refused: use the fresh link temper serve printed\n",
            "the old link is refused, without a transport or filesystem failure"
        );
    }
    terminal.owner
}
