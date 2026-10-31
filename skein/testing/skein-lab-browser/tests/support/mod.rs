//! One-thread pipe reader with deadlines and stderr evidence.
use crate::json::{self, Value};
use skein_io::{
    self as io, Io,
    kernel::{Complete, Fd, Submit},
};
use skein_lib::{Duration, Env, Queue, Time, Token, stream};
use skein_shell::{Clock, Wait};
use skein_world::{Deadline, Expired, ring::Ring};
use std::{
    collections::VecDeque,
    os::fd::IntoRawFd,
    path::{Path, PathBuf},
    process::{Child as Process, ChildStdout},
};
pub fn field<'a>(value: &'a Value, key: &str) -> &'a Value {
    let Value::Object(fields) = value else { panic!("object: {value:?}") };
    fields.get(key).unwrap_or_else(|| panic!("missing {key}: {value:?}"))
}
pub fn integer(value: &Value) -> i64 {
    let Value::Number(n) = value else { panic!("integer: {value:?}") };
    *n
}
pub fn text(value: &Value) -> &str {
    let Value::String(s) = value else { panic!("string: {value:?}") };
    s
}
pub struct Child(pub Process);
impl Drop for Child {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}
pub fn chromium() -> PathBuf {
    if let Some(path) = std::env::var_os("SKEIN_TEST_CHROMIUM") {
        let path = PathBuf::from(path);
        assert!(path.is_file(), "SKEIN_TEST_CHROMIUM: {}", path.display());
        return path.canonicalize().expect("resolve Chromium path");
    }
    let path = std::env::var_os("PATH").expect("PATH locates chromium");
    for dir in std::env::split_paths(&path) {
        let path = dir.join("chromium");
        if path.is_file() {
            return path.canonicalize().expect("resolve Chromium path");
        }
    }
    panic!("Chromium is required: set SKEIN_TEST_CHROMIUM or put chromium on PATH");
}
pub fn fonts() -> PathBuf {
    let configured = std::env::var_os("SKEIN_TEST_FONTS").map(PathBuf::from);
    let fallback = Path::new("/usr/share/fonts/truetype/dejavu");
    if let Some(dir) = &configured
        && dir.join("DejaVuSans.ttf").is_file()
    {
        return dir.canonicalize().expect("resolve fonts directory");
    }
    if fallback.join("DejaVuSans.ttf").is_file() {
        return fallback.to_path_buf();
    }
    panic!(
        "DejaVuSans.ttf missing from SKEIN_TEST_FONTS ({configured:?}) and {}; install Debian package fonts-dejavu-core",
        fallback.display()
    );
}
pub struct Reader {
    io: Io,
    ring: Ring,
    env: Env<io::Limits>,
    clock: Clock,
    pipe: Token,
    complete: Queue<Complete>,
    submits: Queue<Submit>,
    events: Queue<io::Event>,
    received: VecDeque<stream::Up>,
    stderr: PathBuf,
    children: Vec<u32>,
}
impl Reader {
    pub fn new(stdout: ChildStdout, stderr: PathBuf, children: Vec<u32>) -> Self {
        let limits = io::Limits {
            sockets: 1,
            names: 0,
            refusals: 1,
            intake: 32 << 20,
            receive: 65536,
            output: 1,
            sends: 1,
            accepts: 1,
            backlog: 1,
            close_timeout: Duration::from_secs(1),
            retry: Duration::from_millis(10),
        };
        let mut io = Io::new(&limits);
        let pipe = io.adopt_read_pipe(Fd::new(stdout.into_raw_fd())).expect("adopt child stdout");
        let clock = Clock::new();
        let now = clock.now();
        Self {
            io,
            ring: Ring::new(
                io::operations(&limits).expect("reader operation bound"),
                vec![io::operations(&limits).expect("reader operation bound")],
            )
            .expect("reader ring"),
            env: Env { now: now.now, wall: now.wall, limits },
            clock,
            pipe,
            complete: Queue::with_capacity(32),
            submits: Queue::with_capacity(32),
            events: Queue::with_capacity(32),
            received: VecDeque::new(),
            stderr,
            children,
        }
    }
    pub fn deadline(&self, what: &'static str, patience: Duration) -> Waiting {
        Waiting::new(what, patience, &self.children).evidence(&self.stderr)
    }
    pub fn now(&self) -> Time {
        self.clock.now().now
    }
    pub fn stderr(&self) -> String {
        std::fs::read_to_string(&self.stderr).expect("program stderr")
    }
    pub fn answer(&mut self) -> Value {
        let line = self.line();
        let value = json::parse(&line).unwrap_or_else(|e| panic!("answer JSON: {e}; {}", self.stderr()));
        assert_eq!(json::encode(&value), line, "answer must be canonical");
        value
    }
    pub fn line(&mut self) -> Vec<u8> {
        let deadline = self.deadline("the lab program stdout answer", Duration::from_secs(10));
        io::down(
            &mut self.io,
            &self.env,
            io::Request::Stream {
                stream: self.pipe,
                down: stream::Down::Demand {
                    read: stream::Read::Scan {
                        until: stream::Delimiter::new(b"\n").expect("LF delimiter"),
                        max: 32 << 20,
                    },
                    room: 0,
                },
            },
            &mut self.submits,
        );
        loop {
            self.pause();
            if let Some(up) = self.received.pop_front() {
                let bytes = match up {
                    stream::Up::Bytes(bytes) => bytes,
                    stream::Up::End => deadline.terminal(&format!("program stdout ended; {}", self.stderr())),
                    stream::Up::Failed(error) => panic!("program stdout: {error:?}; {}", self.stderr()),
                    stream::Up::Room => panic!("unexpected stdout room"),
                };
                assert!(bytes.ends_with(b"\n"), "answer missing newline: {}", self.stderr());
                return bytes[..bytes.len() - 1].to_vec();
            }
            deadline.check(&format!("answer deadline passed: {}", self.stderr()));
        }
    }
    pub fn pause(&mut self) {
        let now = self.clock.now();
        self.env.now = now.now;
        self.env.wall = now.wall;
        while let Some(complete) = self.ring.reap().pop() {
            let (host, complete) = self.ring.complete(complete);
            self.ring.returned(host, complete);
        }
        self.ring.deliver(0, &mut self.complete);
        io::stage::up(&mut self.io, &self.env, &mut self.complete, &mut self.events, &mut self.submits, now.now);
        while let Some(event) = self.events.pop() {
            match event {
                io::Event::Stream { up, .. } => self.received.push_back(up),
                other @ (io::Event::Listening { .. }
                | io::Event::Accepted { .. }
                | io::Event::Connecting { .. }
                | io::Event::Connected { .. }
                | io::Event::Output { .. }
                | io::Event::Spawned { .. }
                | io::Event::Exited { .. }
                | io::Event::Usage { .. }
                | io::Event::Resized
                | io::Event::Window { .. }
                | io::Event::Shutdown { .. }
                | io::Event::Failed { .. }
                | io::Event::Closed { .. }) => panic!("stdout io: {other:?}"),
            }
        }
        self.io.reclaim();
        while let Some(submit) = self.submits.pop() {
            let submit = self.ring.register(0, submit);
            self.ring.queue(submit);
        }
        self.ring.enter(Wait::Until(now.now.saturating_add(Duration::from_millis(10))));
    }
}

/// Owned processes and the unanswered wait they must answer (testing-strategy.md, 2.10).
pub struct Waiting {
    deadlines: Vec<Deadline>,
    children: Vec<(u32, Fd)>,
    stderr: Option<PathBuf>,
}
impl Waiting {
    pub fn new(what: &'static str, patience: Duration, pids: &[u32]) -> Self {
        let mut deadlines = Vec::new();
        let mut children = Vec::new();
        for pid in pids {
            let mut deadline = Deadline::arm(what, patience);
            deadline.watch_process(*pid);
            deadlines.push(deadline);
            if *pid != std::process::id() {
                let keeper =
                    skein_shell::open_child_pidfd(*pid, std::process::id()).expect("retain owned child while waiting");
                children.push((*pid, keeper));
                // The lab waits on Chromium in a separate child group. Measure
                // that owned child too when it exists at the start of the wait.
                if let Ok(owned) = std::fs::read_to_string(format!("/proc/{pid}/task/{pid}/children")) {
                    for child in owned.split_whitespace() {
                        let child = child.parse().expect("owned browser PID");
                        if let Ok(keeper) = skein_shell::open_child_pidfd(child, *pid) {
                            let mut deadline = Deadline::arm(what, patience);
                            deadline.watch_process(child);
                            deadlines.push(deadline);
                            children.push((child, keeper));
                        }
                    }
                }
            }
        }
        if deadlines.is_empty() {
            deadlines.push(Deadline::arm(what, patience));
        }
        Self { deadlines, children, stderr: None }
    }
    pub fn evidence(mut self, stderr: &Path) -> Self {
        self.stderr = Some(stderr.to_path_buf());
        self
    }
    // A lab process is itself a harness. Forward its verdict only after a
    // successful exit; an assertion, refusal or bad exit still fails here.
    pub fn verdict(&self, success: bool) {
        if !success {
            return;
        }
        let Some(stderr) = &self.stderr else { return };
        let stderr = std::fs::read_to_string(stderr).expect("child verdict evidence");
        if let Some(line) = stderr.lines().find(|line| line.starts_with("skein-world: starved: ")) {
            assert!(std::env::var_os("NEXTEST").is_some(), "{line}");
            self.end();
            eprintln!("{line}");
            std::process::exit(0);
        }
    }
    fn terminal(&self, why: &str) -> ! {
        // EOF can precede the child's final stderr write. Wait for its exit
        // within this same patience before deciding whether it passed starved.
        if let Some((_, child)) = self.children.first() {
            loop {
                match skein_shell::poll_child(*child, false) {
                    Ok(Some(exit)) => {
                        self.verdict(exit == io::kernel::Exit::Code(0));
                        break;
                    }
                    Ok(None) => {}
                    Err(error) => panic!("{why}: child exit: {error:?}"),
                }
                if self.deadlines.iter().all(Deadline::passed) {
                    break;
                }
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
        }
        panic!("{why}")
    }
    pub fn check(&self, why: &str) {
        self.check_socket(why, None);
    }
    fn check_socket(&self, why: &str, socket: Option<&std::net::TcpStream>) {
        if !self.deadlines.iter().all(Deadline::passed) {
            return;
        }
        // A completed child is an answer to an exit wait, or a broken
        // conversation. Neither can be reclassified from scheduler load.
        for (_, child) in &self.children {
            if let Ok(Some(exit)) = skein_shell::poll_child(*child, false) {
                self.verdict(exit == io::kernel::Exit::Code(0));
                panic!("{why}: child exited: {exit:?}");
            }
        }
        let mut failed = Vec::new();
        for deadline in &self.deadlines {
            match deadline.expired() {
                Expired::Failed(figures) => failed.push(figures.to_string()),
                Expired::Starved(starved) => {
                    if let Some(socket) = socket {
                        socket.shutdown(std::net::Shutdown::Both).expect("end the unanswered socket");
                    }
                    self.end();
                    starved.pass();
                }
            }
        }
        panic!("{why}; {}", failed.join("; "));
    }
    fn end(&self) {
        // Chromium leads its own group below the lab process. Bind its identity
        // while the unreaped parent still owns it, then end that group too.
        for (pid, keeper) in &self.children {
            if let Ok(children) = std::fs::read_to_string(format!("/proc/{pid}/task/{pid}/children")) {
                for child in children.split_whitespace() {
                    let child = child.parse().expect("owned child PID");
                    if let Ok(browser) = skein_shell::open_child_pidfd(child, *pid) {
                        let _ = skein_shell::signal_kept_child(
                            browser,
                            io::kernel::Signal::Kill,
                            io::kernel::Target::Group,
                        );
                        skein_shell::close_keeper_fd(browser);
                    }
                }
            }
            skein_shell::abandon_binary(Some(*keeper), &[]);
        }
    }
    pub fn read_line(&self, reader: &mut std::io::BufReader<std::net::TcpStream>, line: &mut Vec<u8>, why: &str) {
        use std::io::BufRead;
        loop {
            self.set_timeout(reader.get_ref());
            let bytes = match reader.fill_buf() {
                Ok(bytes) => bytes,
                Err(error) if matches!(error.kind(), std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut) => {
                    self.check_socket(why, Some(reader.get_ref()));
                    continue;
                }
                Err(error) => panic!("{why}: {error}"),
            };
            if bytes.is_empty() {
                if line.is_empty() && self.stderr.is_some() {
                    self.terminal(why);
                }
                return;
            }
            let end = bytes.iter().position(|byte| *byte == b'\n');
            let count = end.map_or(bytes.len(), |at| at + 1);
            line.extend_from_slice(&bytes[..count]);
            reader.consume(count);
            if end.is_some() {
                return;
            }
            self.check_socket(why, Some(reader.get_ref()));
        }
    }
    pub fn read_end(
        &self,
        reader: &mut std::io::BufReader<std::net::TcpStream>,
        bytes: &mut Vec<u8>,
        why: &str,
        reset: bool,
    ) {
        use std::io::Read;
        let mut buffer = [0; 4096];
        loop {
            self.set_timeout(reader.get_ref());
            match reader.read(&mut buffer) {
                Ok(0) => return,
                Err(error) if reset && error.kind() == std::io::ErrorKind::ConnectionReset => return,
                Ok(count) => {
                    bytes.extend_from_slice(&buffer[..count]);
                    assert!(bytes.is_empty(), "{why}: unexpected bytes: {bytes:?}");
                }
                Err(error) if matches!(error.kind(), std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut) => {
                    self.check_socket(why, Some(reader.get_ref()));
                }
                Err(error) => panic!("{why}: {error}"),
            }
        }
    }
    pub fn write(&self, stream: &mut std::net::TcpStream, mut bytes: &[u8], why: &str) {
        use std::io::Write;
        while !bytes.is_empty() {
            let remaining = self.deadlines.iter().map(Deadline::remaining).min().expect("a wait has a deadline");
            stream
                .set_write_timeout(Some(std::time::Duration::from_nanos(remaining.as_nanos().max(1))))
                .expect("socket write deadline");
            match stream.write(bytes) {
                Ok(0) => panic!("{why}: socket write ended"),
                Ok(count) => bytes = &bytes[count..],
                Err(error) if matches!(error.kind(), std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut) => {
                    self.check_socket(why, Some(stream));
                }
                Err(error) => panic!("{why}: {error}"),
            }
            if !bytes.is_empty() {
                self.check_socket(why, Some(stream));
            }
        }
    }
    fn set_timeout(&self, stream: &std::net::TcpStream) {
        let remaining = self.deadlines.iter().map(Deadline::remaining).min().expect("a wait has a deadline");
        stream
            .set_read_timeout(Some(std::time::Duration::from_nanos(remaining.as_nanos().max(1))))
            .expect("socket wait deadline");
    }
}
impl Drop for Waiting {
    fn drop(&mut self) {
        for (_, keeper) in &self.children {
            skein_shell::close_keeper_fd(*keeper);
        }
    }
}
