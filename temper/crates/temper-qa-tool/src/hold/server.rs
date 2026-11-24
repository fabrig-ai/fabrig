//! The holder owns the lock, the socket and a separate drive process group (qa/places.md, 5).
use super::{Drive, REQUEST_LIMIT, Reply, START, STOP, TICK, io_refusal, tail, wait_for_exit};
use crate::{
    Exit, Refusal,
    faces::{ClockFace, EnvironmentEntry, FilesFace, Processes, RemoveMode, Signal},
    refused,
};
use std::{
    collections::VecDeque,
    io::{self, Read, Write},
    time::Duration,
};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Answer {
    Start,
    Line,
    End,
}

pub struct Holder<'a, F: FilesFace, P: Processes, C: ClockFace> {
    drive: Drive,
    files: &'a mut F,
    processes: &'a mut P,
    clock: &'a mut C,
    lock: F::Lock,
    listener: F::Listener,
    process: u64,
    input: P::Connection,
    output: P::Connection,
    pid: u32,
    pending: Vec<u8>,
    sent: usize,
    partial: Vec<u8>,
    lines: VecDeque<String>,
    exit: Option<u8>,
    started: Duration,
    watch_period: Duration,
    next_watch: Duration,
    stop_at: Option<Duration>,
    starting: bool,
}
impl<'a, F: FilesFace, P: Processes, C: ClockFace> Holder<'a, F, P, C> {
    /// Production uses sixty seconds; focused tests can shorten the ledger watch.
    pub fn start(
        drive: Drive,
        environment: &[EnvironmentEntry],
        watch_period: Duration,
        pid: u32,
        files: &'a mut F,
        processes: &'a mut P,
        clock: &'a mut C,
    ) -> Result<Self, Refusal> {
        drive.place.ensure(files)?;
        let lock = drive.take_lock(files)?;
        let scripts = drive.scripts(files)?;
        let socket = drive.place.directory.join("drive.sock");
        let listener = files.bind(&socket).map_err(io_refusal)?;
        let program = drive.place.bin.join("temper-client-journeys");
        let session =
            match processes.drive(program.as_os_str(), &drive.session_arguments(scripts.as_ref()), environment) {
                Ok(session) => session,
                Err(why) => {
                    let _ = files.remove(&socket, RemoveMode::File);
                    return Err(why);
                }
            };
        let started = clock.monotonic();
        Ok(Self {
            drive,
            files,
            processes,
            clock,
            lock,
            listener,
            process: session.process,
            input: session.input,
            output: session.output,
            pid,
            pending: Vec::new(),
            sent: 0,
            partial: Vec::new(),
            lines: VecDeque::new(),
            exit: None,
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
        let removed =
            self.files.remove(&self.drive.place.directory.join("drive.sock"), RemoveMode::File).map_err(io_refusal);
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
            match self.files.accept(&self.listener) {
                Ok(Some(stream)) => {
                    if self.connection(stream)? {
                        return Ok(());
                    }
                }
                Ok(None) => {}
                Err(why) => return Err(io_refusal(why)),
            }
            self.poll_exit()?;
            // Preserve a failed start until open's first request can collect its lines/status.
            if !self.starting && self.exit.is_some() {
                return Ok(());
            }
            self.clock.sleep(TICK);
        }
    }
    fn watch(&mut self) -> Result<bool, Refusal> {
        let time = self.clock.monotonic();
        if time >= self.next_watch {
            self.next_watch = time.saturating_add(self.watch_period);
            return self.drive.running(self.files, self.clock);
        }
        Ok(true)
    }
    fn connection(&mut self, mut stream: F::Connection) -> Result<bool, Refusal> {
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
                Err(why) if why.kind() == io::ErrorKind::WouldBlock => self.clock.sleep(TICK),
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
            if pid == self.pid.to_string() {
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
                Err(why) if why.kind() == io::ErrorKind::WouldBlock => {}
                Err(_) => break,
            }
            if sent < packet.len() {
                // Backpressure from a client must not prevent the ledger watch.
                if !self.watch()? {
                    self.end()?;
                    return Ok(true);
                }
                self.clock.sleep(TICK);
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
            self.stop_at = Some(self.clock.monotonic().saturating_add(STOP));
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
            if kind == Answer::Start && self.clock.monotonic().saturating_sub(self.started) >= START {
                start_expired = true;
                self.begin_end();
            }
            if self.stop_at.is_some_and(|deadline| self.clock.monotonic() >= deadline) {
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
            self.clock.sleep(TICK);
        }
    }
    fn pump(&mut self) -> Result<(), Refusal> {
        if self.sent < self.pending.len() && self.exit.is_none() {
            match self.input.write(self.pending.get(self.sent..).expect("pending suffix")) {
                Ok(count) => self.sent += count,
                Err(why) if why.kind() == io::ErrorKind::WouldBlock || why.kind() == io::ErrorKind::BrokenPipe => {}
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
                Err(why) if why.kind() == io::ErrorKind::WouldBlock => break,
                Err(why) => return Err(io_refusal(why)),
            }
        }
        Ok(())
    }
    fn poll_exit(&mut self) -> Result<(), Refusal> {
        if self.exit.is_none()
            && let Some(status) = self.processes.poll(self.process)?
        {
            self.exit = Some(match status {
                Exit::Code(0) => 0,
                Exit::Code(3) => 3,
                Exit::Code(4) => 4,
                _ => 1,
            });
            // Drain the final bytes after observing exit, not only those readable before it.
            self.pump()?;
        }
        Ok(())
    }
    fn kill(&mut self) -> Result<(), Refusal> {
        // A ledger watch during the final reply can ask to end again after reaping.
        if self.exit.is_none() {
            self.processes.signal(self.process, Signal::Kill)?;
            wait_for_exit(self.process, self.processes, self.clock)?;
        }
        self.exit = Some(4);
        Ok(())
    }
}
