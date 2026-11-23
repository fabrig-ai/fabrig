//! A pass's drive session and its one-shot clients (qa/places.md, 5).
mod server;

use crate::{
    Home, Refusal,
    faces::{ClockFace, Environment, Processes, RealClock, RealFiles, RealProcesses, WriteMode},
    place::PlacePaths,
    private, refused,
};
pub use server::Holder;
use std::{
    ffi::OsString,
    fs::{self, File},
    io::{Read, Write},
    os::unix::net::UnixStream,
    path::PathBuf,
    time::Duration,
};

pub const WATCH: Duration = Duration::from_secs(60);
const START: Duration = Duration::from_mins(5);
pub(crate) const STOP: Duration = Duration::from_secs(70);
const TICK: Duration = Duration::from_millis(10);
const REQUEST_LIMIT: usize = 64 * 1024;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Reply {
    pub status: u8,
    pub text: String,
}
impl Reply {
    fn packet(&self) -> Vec<u8> {
        format!("{}\n{}", self.status, self.text).into_bytes()
    }
}

/// The session is checked under the ledger lock, before any place is touched.
#[derive(Clone, Debug)]
pub struct Drive {
    home: Home,
    session: u64,
    place: PlacePaths,
    setups: PathBuf,
    files: PathBuf,
    ctl: bool,
}
impl Drive {
    pub fn for_session(home: &Home, session: u64) -> Result<Self, Refusal> {
        home.read(&mut RealFiles, |ledger| {
            let number = ledger.pass_command_session(session)?.pass.expect("pass session");
            let pass = ledger.passes.iter().find(|pass| pass.number == number).expect("session's pass");
            let place = home.configuration.place_directory(&home.directory, &pass.place)?;
            let mut paths = PlacePaths::new(
                &std::path::absolute(place).map_err(io_refusal)?,
                home.configuration.place_kind(&pass.place),
            );
            paths.ports = home.configuration.ports.get(&pass.place).cloned();
            Ok(Self {
                ctl: false,
                home: home.clone(),
                session,
                place: paths,
                setups: std::path::absolute(crate::pass_files::setups(home, number)).map_err(io_refusal)?,
                files: std::path::absolute(crate::pass_files::files(home, number)).map_err(io_refusal)?,
            })
        })
    }

    #[must_use]
    pub fn with_ctl(mut self, ctl: bool) -> Self {
        self.ctl = ctl;
        self
    }

    fn session_arguments(&self, chain: Option<&crate::kind::Chain>) -> Vec<OsString> {
        let mut arguments = vec!["drive".into()];
        arguments.extend(self.place.kind.drive_arguments(chain.map(|chain| chain.root.as_path())));
        arguments.extend(["--bin".into(), self.place.bin.as_os_str().into()]);
        arguments.extend(["--bundle".into(), self.place.bundle.as_os_str().into()]);
        arguments.extend(["--scratch".into(), self.place.scratch.as_os_str().into()]);
        if let Some(chain) = chain {
            for script in chain.members.iter().filter_map(|member| member.scripts.as_ref()) {
                arguments.extend(["--scripts".into(), script.as_os_str().into()]);
            }
        }
        arguments.extend(["--strict".into(), "--files".into(), self.files.as_os_str().into()]);
        if let Some(ports) = &self.place.ports {
            arguments.extend(["--ports".into(), ports.into()]);
        }
        if self.ctl {
            arguments.push("--ctl".into());
        }
        arguments
    }

    fn lock(&self) -> Result<File, Refusal> {
        private::file(&self.place.drive_lock, &mut RealFiles, WriteMode::OpenOrCreate, true)
    }
    fn busy(&self) -> Refusal {
        refused(format!(
            "the drive session is open at {}; close it first: temper-qa pass close",
            self.place.directory.display()
        ))
    }
    fn absent(&self) -> Refusal {
        refused(format!("no drive session is open at {}: temper-qa pass open first", self.place.directory.display()))
    }
    fn running(&self) -> Result<bool, Refusal> {
        self.home.read(&mut RealFiles, |ledger| Ok(ledger.pass_command_session(self.session).is_ok()))
    }

    pub fn open(&self, environment: &Environment) -> Result<Reply, Refusal> {
        // Check before starting a background process, so a refusal belongs to this command.
        self.place.ensure(&mut RealFiles)?;
        self.scripts()?;
        let lock = self.lock()?;
        lock.try_lock().map_err(|_| self.busy())?;
        let mut processes = RealProcesses::default();
        let process =
            processes.holder(&self.place.directory.join("drive.log"), self.ctl, environment, || drop(lock))?;
        let mut child = processes.take_child(process);
        let mut clock = RealClock::default();
        let started = clock.monotonic();
        let request = format!("start {}", child.id());
        loop {
            if let Some(status) = child.try_wait().map_err(|why| refused(format!("pass hold: {why}")))? {
                return Ok(Reply {
                    status: status.code().and_then(|code| u8::try_from(code).ok()).unwrap_or(1),
                    text: tail(&fs::read_to_string(self.place.directory.join("drive.log")).unwrap_or_default()),
                });
            }
            if let Ok(stream) = UnixStream::connect(self.place.directory.join("drive.sock")) {
                let reply = exchange(stream, &request, Some((started, START)), &mut clock);
                if let Ok(reply) = reply {
                    if reply.status != 0 {
                        let _ = child.wait();
                    }
                    return Ok(reply);
                }
                // A stale socket is removed by the new holder; keep waiting for its start.
            }
            if clock.monotonic().saturating_sub(started) >= START {
                // The holder bounds its start too and drains its drive group before exiting.
                let _ = self.request("end", None);
                let _ = child.wait();
                return Ok(Reply { status: 4, text: "box failed: drive session start took past 300 seconds\n".into() });
            }
            clock.sleep(TICK);
        }
    }

    pub fn line(&self, line: &str) -> Result<Reply, Refusal> {
        if line.contains(['\r', '\n']) || line.len() > REQUEST_LIMIT.saturating_sub(6) {
            return Err(refused("a drive line must be one line of at most 65530 bytes"));
        }
        if line == "end" {
            return self.request("end", None);
        }
        self.request(&format!("line {line}"), None)
    }
    pub fn close(&self) -> Result<Reply, Refusal> {
        let mut reply = self.request("end", None)?;
        // The response precedes socket removal and lock release. Do not report a free place early, and
        // hold the lock no longer than it takes to see it released, so the place frees at once.
        let lock = self.lock()?;
        lock.lock().map_err(|why| refused(format!("drive.lock: {why}")))?;
        drop(lock);
        let lines = match fs::read_to_string(self.place.scratch.join("session/lines")) {
            Ok(lines) => lines,
            Err(why) if why.kind() == std::io::ErrorKind::NotFound => String::new(),
            Err(why) => return Err(refused(format!("session/lines: {why}"))),
        };
        reply.text.push_str("kept lines:\n");
        for line in lines.lines() {
            reply.text.push_str(line);
            reply.text.push('\n');
        }
        Ok(reply)
    }
    fn request(&self, request: &str, bound: Option<(Duration, Duration)>) -> Result<Reply, Refusal> {
        self.place.ensure(&mut RealFiles)?;
        let lock = self.lock().map_err(|_| self.absent())?;
        if lock.try_lock().is_ok() {
            return Err(self.absent());
        }
        let stream = UnixStream::connect(self.place.directory.join("drive.sock")).map_err(|_| self.absent())?;
        exchange(stream, request, bound, &mut RealClock::default())
    }

    fn scripts(&self) -> Result<Option<crate::kind::Chain>, Refusal> {
        let setup = fs::read_to_string(&self.place.setup).map_err(|why| {
            if why.kind() == std::io::ErrorKind::NotFound {
                refused(format!(
                    "no setup is restored at {}: temper-qa pass setup NAME first",
                    self.place.directory.display()
                ))
            } else {
                refused(format!("setup: {why}"))
            }
        })?;
        let words: Vec<_> = setup.split_whitespace().collect();
        if words == ["fresh"] {
            self.place.kind.check_fresh()?;
            return Ok(None);
        }
        let [name, hash] = words.as_slice() else {
            return Err(refused("setup: expected NAME HASH or fresh"));
        };
        if !recipe_name(name)
            || hash.len() != 64
            || !hash.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        {
            return Err(refused("setup: expected NAME HASH or fresh"));
        }
        crate::kind::read_chain(&self.setups, name, crate::kind::ChainUse::Holder).map(Some)
    }
}
fn recipe_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 64
        && name.as_bytes().first().is_some_and(u8::is_ascii_lowercase)
        && name.bytes().all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}
fn tail(text: &str) -> String {
    let lines: Vec<_> = text.lines().collect();
    let first = lines.len().saturating_sub(40);
    let mut text = lines.iter().skip(first).copied().collect::<Vec<_>>().join("\n");
    if !text.is_empty() {
        text.push('\n');
    }
    text
}
fn exchange(
    mut stream: UnixStream,
    request: &str,
    bound: Option<(Duration, Duration)>,
    clock: &mut impl ClockFace,
) -> Result<Reply, Refusal> {
    stream.set_nonblocking(true).map_err(io_refusal)?;
    let packet = format!("{request}\n");
    let mut sent = 0;
    let mut bytes = Vec::new();
    loop {
        if bound.is_some_and(|(start, limit)| clock.monotonic().saturating_sub(start) >= limit) {
            return Err(refused("drive session start timed out"));
        }
        if sent < packet.len() {
            match stream.write(packet.as_bytes().get(sent..).expect("packet suffix")) {
                Ok(0) => return Err(refused("drive socket closed writing request")),
                Ok(count) => sent += count,
                Err(why) if why.kind() == std::io::ErrorKind::WouldBlock => {}
                Err(why) => return Err(io_refusal(why)),
            }
        }
        let mut buffer = [0; 8192];
        match stream.read(&mut buffer) {
            Ok(0) => break,
            Ok(count) => bytes.extend_from_slice(buffer.get(..count).expect("read length")),
            Err(why) if why.kind() == std::io::ErrorKind::WouldBlock => clock.sleep(TICK),
            Err(why) => return Err(io_refusal(why)),
        }
    }
    let text = String::from_utf8(bytes).map_err(|why| refused(format!("drive socket: {why}")))?;
    let (status, text) = text.split_once('\n').ok_or_else(|| refused("drive socket: missing status"))?;
    Ok(Reply { status: status.parse().map_err(|_| refused("drive socket: invalid status"))?, text: text.into() })
}
#[expect(clippy::needless_pass_by_value, reason = "map_err consumes the IO error at this boundary")]
pub(crate) fn io_refusal(why: std::io::Error) -> Refusal {
    refused(format!("drive session: {why}"))
}
