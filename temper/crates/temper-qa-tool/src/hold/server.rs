//! The holder owns the lock, the socket and a separate drive process group (qa/places.md, 5).
use super::{Drive, REQUEST_LIMIT, Reply, START, STOP, TICK, io_refusal, now, tail};
use crate::{Refusal, faces::Environment, refused};
use std::{
    collections::VecDeque,
    fs::{self, File},
    io::{Read, Write},
    os::{
        fd::OwnedFd,
        unix::{
            fs::PermissionsExt,
            net::{UnixListener, UnixStream},
            process::CommandExt,
        },
    },
    process::{Child, Command, Stdio},
    time::Duration,
};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Answer {
    Start,
    Line,
    End,
}

pub struct Holder {
    drive: Drive,
    lock: File,
    listener: UnixListener,
    child: Child,
    input: UnixStream,
    output: UnixStream,
    pending: Vec<u8>,
    sent: usize,
    partial: Vec<u8>,
    lines: VecDeque<String>,
    exit: Option<u8>,
    environment: Environment,
    clock: skein_shell::Clock,
    started: Duration,
    watch_period: Duration,
    next_watch: Duration,
    stop_at: Option<Duration>,
    starting: bool,
}
impl Holder {
    /// Production uses five seconds; focused tests can shorten the ledger watch.
    pub fn start(drive: Drive, environment: Environment, watch_period: Duration) -> Result<Self, Refusal> {
        let lock = drive.lock()?;
        lock.try_lock().map_err(|_| drive.busy())?;
        let scripts = drive.scripts()?;
        let socket = drive.place.directory.join("drive.sock");
        match fs::remove_file(&socket) {
            Ok(()) => {}
            Err(why) if why.kind() == std::io::ErrorKind::NotFound => {}
            Err(why) => return Err(io_refusal(why)),
        }
        let listener = UnixListener::bind(&socket).map_err(io_refusal)?;
        let start = || {
            fs::set_permissions(&socket, fs::Permissions::from_mode(0o600)).map_err(io_refusal)?;
            listener.set_nonblocking(true).map_err(io_refusal)?;
            // Socket pairs provide piped stdin/stdout with safe std nonblocking handles.
            // Separate pairs let ending stdin remain independent of draining stdout.
            let (input, child_input) = UnixStream::pair().map_err(io_refusal)?;
            let (output, child_output) = UnixStream::pair().map_err(io_refusal)?;
            input.set_nonblocking(true).map_err(io_refusal)?;
            output.set_nonblocking(true).map_err(io_refusal)?;
            let mut command = Command::new(drive.place.bin.join("temper-client-journeys"));
            crate::faces::captured(&mut command, &environment);
            command
                .arg("drive")
                .arg("--bin")
                .arg(&drive.place.bin)
                .arg("--bundle")
                .arg(&drive.place.bundle)
                .arg("--scratch")
                .arg(&drive.place.scratch);
            for script in scripts {
                command.arg("--scripts").arg(script);
            }
            if let Some(ports) = &drive.place.ports {
                command.args(["--ports", ports]);
            }
            let child = command
                .arg("--strict")
                .stdin(Stdio::from(OwnedFd::from(child_input)))
                .stdout(Stdio::from(OwnedFd::from(child_output)))
                .stderr(Stdio::inherit())
                .process_group(0)
                .spawn()
                .map_err(io_refusal)?;
            Ok((input, output, child))
        };
        let (input, output, child) = match start() {
            Ok(parts) => parts,
            Err(why) => {
                let _ = fs::remove_file(&socket);
                return Err(why);
            }
        };
        let clock = skein_shell::Clock::new();
        let started = now(&clock);
        Ok(Self {
            drive,
            lock,
            listener,
            child,
            input,
            output,
            pending: Vec::new(),
            sent: 0,
            partial: Vec::new(),
            lines: VecDeque::new(),
            exit: None,
            environment,
            clock,
            started,
            watch_period,
            next_watch: started.saturating_add(watch_period),
            stop_at: None,
            starting: true,
        })
    }

    pub fn serve(mut self) -> Result<(), Refusal> {
        let result = self.listen();
        // An IO/ledger failure still ends the drive group before releasing the place.
        if self.exit.is_none() && self.end().is_err() {
            self.kill()?;
        }
        let removed = fs::remove_file(self.drive.place.directory.join("drive.sock")).map_err(io_refusal);
        drop(self.listener);
        drop(self.lock);
        result.and(removed)
    }
    fn listen(&mut self) -> Result<(), Refusal> {
        loop {
            if !self.watch()? {
                self.end()?;
                return Ok(());
            }
            match self.listener.accept() {
                Ok((stream, _)) => {
                    if self.connection(stream)? {
                        return Ok(());
                    }
                }
                Err(why) if why.kind() == std::io::ErrorKind::WouldBlock => {}
                Err(why) => return Err(io_refusal(why)),
            }
            self.poll_exit()?;
            // Preserve a failed start until open's first request can collect its lines/status.
            if !self.starting && self.exit.is_some() {
                return Ok(());
            }
            std::thread::sleep(TICK);
        }
    }
    fn watch(&mut self) -> Result<bool, Refusal> {
        let time = now(&self.clock);
        if time >= self.next_watch {
            self.next_watch = time.saturating_add(self.watch_period);
            return self.drive.running();
        }
        Ok(true)
    }
    fn connection(&mut self, mut stream: UnixStream) -> Result<bool, Refusal> {
        stream.set_nonblocking(true).map_err(io_refusal)?;
        let mut bytes = Vec::new();
        let request = loop {
            if !self.watch()? {
                self.end()?;
                return Ok(true);
            }
            let mut buffer = [0; 4096];
            match stream.read(&mut buffer) {
                Ok(0) => return Ok(false),
                Ok(count) => bytes.extend_from_slice(buffer.get(..count).expect("read length")),
                Err(why) if why.kind() == std::io::ErrorKind::WouldBlock => std::thread::sleep(TICK),
                Err(_) => return Ok(false), // A disconnected client does not own the session.
            }
            if bytes.len() > REQUEST_LIMIT {
                return Ok(false);
            }
            if bytes.last() == Some(&b'\n') {
                bytes.pop();
                break String::from_utf8(bytes).map_err(|why| refused(format!("drive request: {why}")))?;
            }
        };
        let (reply, end) = if let Some(pid) = request.strip_prefix("start ") {
            if pid == std::process::id().to_string() {
                let reply = self.answer(Answer::Start)?;
                self.starting = false;
                let end = reply.status != 0;
                (reply, end)
            } else {
                (Reply { status: 1, text: "another drive holder owns the socket\n".into() }, false)
            }
        } else if request == "end" {
            (self.end()?, true)
        } else if let Some(line) = request.strip_prefix("line ") {
            if line.contains(['\n', '\r']) {
                (Reply { status: 1, text: "a drive request must be one line\n".into() }, false)
            } else {
                self.queue(line);
                let reply = self.answer(Answer::Line)?;
                let end = self.exit.is_some();
                (reply, end)
            }
        } else {
            (Reply { status: 1, text: "unknown drive request\n".into() }, false)
        };
        let packet = reply.packet();
        let mut sent = 0;
        while sent < packet.len() {
            match stream.write(packet.get(sent..).expect("packet suffix")) {
                Ok(0) => break,
                Ok(count) => sent += count,
                Err(why) if why.kind() == std::io::ErrorKind::WouldBlock => {}
                Err(_) => break,
            }
            if sent < packet.len() {
                // Backpressure from a client must not prevent the ledger watch.
                if !self.watch()? {
                    self.end()?;
                    return Ok(true);
                }
                std::thread::sleep(TICK);
            }
        }
        Ok(end)
    }
    fn queue(&mut self, line: &str) {
        self.pending.extend_from_slice(line.as_bytes());
        self.pending.push(b'\n');
    }
    fn begin_end(&mut self) {
        if self.stop_at.is_none() && self.exit.is_none() {
            self.queue("end");
            self.stop_at = Some(now(&self.clock).saturating_add(STOP));
        }
    }
    fn end(&mut self) -> Result<Reply, Refusal> {
        self.begin_end();
        self.answer(Answer::End)
    }
    fn answer(&mut self, kind: Answer) -> Result<Reply, Refusal> {
        let mut text = String::new();
        let mut start_expired = false;
        loop {
            if !self.watch()? {
                self.begin_end();
            }
            if kind == Answer::Start && now(&self.clock).saturating_sub(self.started) >= START {
                start_expired = true;
                self.begin_end();
            }
            if self.stop_at.is_some_and(|deadline| now(&self.clock) >= deadline) {
                self.kill()?;
                text.push_str("box failed: drive session did not stop within 70 seconds\n");
                return Ok(Reply { status: 4, text: tail(&text) });
            }
            self.pump()?;
            self.poll_exit()?;
            while let Some(line) = self.lines.pop_front() {
                text.push_str(&line);
                text.push('\n');
                if self.stop_at.is_none() && self.exit.is_none() {
                    if kind == Answer::Start && line == "ready" {
                        return Ok(Reply { status: 0, text });
                    }
                    if kind == Answer::Line && (line == "ok" || line.starts_with("failed: ")) {
                        return Ok(Reply { status: if line == "ok" { 0 } else { 5 }, text });
                    }
                }
            }
            if let Some(status) = self.exit {
                if !self.partial.is_empty() {
                    text.push_str(&String::from_utf8_lossy(&self.partial));
                    text.push('\n');
                    self.partial.clear();
                }
                let status = if start_expired {
                    text.push_str("box failed: drive session start took past 300 seconds\n");
                    4
                } else if kind != Answer::Start && (status == 3 || (status == 0 && kind == Answer::Line)) {
                    4
                } else {
                    status
                };
                return Ok(Reply { status, text: if kind == Answer::Start { tail(&text) } else { text } });
            }
            std::thread::sleep(TICK);
        }
    }
    fn pump(&mut self) -> Result<(), Refusal> {
        if self.sent < self.pending.len() && self.exit.is_none() {
            match self.input.write(self.pending.get(self.sent..).expect("pending suffix")) {
                Ok(count) => self.sent += count,
                Err(why)
                    if why.kind() == std::io::ErrorKind::WouldBlock || why.kind() == std::io::ErrorKind::BrokenPipe => {
                }
                Err(why) => return Err(io_refusal(why)),
            }
        }
        if self.sent == self.pending.len() {
            self.pending.clear();
            self.sent = 0;
        }
        loop {
            let mut buffer = [0; 8192];
            match self.output.read(&mut buffer) {
                Ok(0) => break,
                Ok(count) => {
                    self.partial.extend_from_slice(buffer.get(..count).expect("read length"));
                    while let Some(end) = self.partial.iter().position(|byte| *byte == b'\n') {
                        let line: Vec<_> = self.partial.drain(..=end).collect();
                        self.lines
                            .push_back(String::from_utf8_lossy(line.strip_suffix(b"\n").expect("newline")).into());
                    }
                }
                Err(why) if why.kind() == std::io::ErrorKind::WouldBlock => break,
                Err(why) => return Err(io_refusal(why)),
            }
        }
        Ok(())
    }
    fn poll_exit(&mut self) -> Result<(), Refusal> {
        if self.exit.is_none()
            && let Some(status) = self.child.try_wait().map_err(io_refusal)?
        {
            self.exit = Some(match status.code() {
                Some(0) => 0,
                Some(3) => 3,
                Some(4) => 4,
                _ => 1,
            });
            // Drain the final bytes after observing exit, not only those readable before it.
            self.pump()?;
        }
        Ok(())
    }
    fn kill(&mut self) -> Result<(), Refusal> {
        let mut command = Command::new("kill");
        crate::faces::captured(&mut command, &self.environment);
        command
            .args(["-s", "KILL", "--", &format!("-{}", self.child.id())])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map_err(io_refusal)?;
        self.child.wait().map_err(io_refusal)?;
        self.exit = Some(4);
        Ok(())
    }
}
