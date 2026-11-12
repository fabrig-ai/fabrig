//! A pass's drive session and its one-shot clients (qa/places.md, 5).
mod server;

use crate::{Home, Refusal, faces::Environment, place::PlacePaths, refused};
pub use server::Holder;
use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    os::unix::{fs::OpenOptionsExt, net::UnixStream, process::CommandExt},
    path::PathBuf,
    process::{Command, Stdio},
    time::Duration,
};

pub const WATCH: Duration = Duration::from_secs(5);
const START: Duration = Duration::from_mins(5);
const STOP: Duration = Duration::from_secs(70);
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
}
impl Drive {
    pub fn for_session(home: &Home, session: u64) -> Result<Self, Refusal> {
        home.read(|ledger| {
            let number = ledger.pass_command_session(session)?.pass.expect("pass session");
            let pass = ledger.passes.iter().find(|pass| pass.number == number).expect("session's pass");
            let place = home.configuration.place_directory(&home.directory, &pass.place)?;
            let mut paths = PlacePaths::new(&std::path::absolute(place).map_err(io_refusal)?);
            paths.ports = home.configuration.ports.get(&pass.place).cloned();
            Ok(Self {
                home: home.clone(),
                session,
                place: paths,
                setups: std::path::absolute(home.directory.join("passes").join(number.to_string()).join("setups"))
                    .map_err(io_refusal)?,
            })
        })
    }

    fn lock(&self) -> Result<File, Refusal> {
        OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .mode(0o600)
            .open(&self.place.drive_lock)
            .map_err(|why| refused(format!("drive.lock: {why}")))
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
        self.home.read(|ledger| Ok(ledger.pass_command_session(self.session).is_ok()))
    }

    pub fn open(&self, environment: &Environment) -> Result<Reply, Refusal> {
        // Check before starting a background process, so a refusal belongs to this command.
        self.scripts()?;
        let lock = self.lock()?;
        lock.try_lock().map_err(|_| self.busy())?;
        let log = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .mode(0o600)
            .open(self.place.directory.join("drive.log"))
            .map_err(|why| refused(format!("drive.log: {why}")))?;
        let mut command = Command::new(std::env::current_exe().map_err(|why| refused(format!("temper-qa: {why}")))?);
        crate::faces::captured(&mut command, environment);
        // Release the probe before spawn: the child may take its lock before spawn returns.
        drop(lock);
        let mut child = command
            .args(["pass", "hold"])
            .stdin(Stdio::null())
            .stderr(log.try_clone().map_err(|why| refused(format!("drive.log: {why}")))?)
            .stdout(log)
            .process_group(0)
            .spawn()
            .map_err(|why| refused(format!("pass hold: {why}")))?;
        let clock = skein_shell::Clock::new();
        let started = now(&clock);
        let request = format!("start {}", child.id());
        loop {
            if let Some(status) = child.try_wait().map_err(|why| refused(format!("pass hold: {why}")))? {
                return Ok(Reply {
                    status: status.code().and_then(|code| u8::try_from(code).ok()).unwrap_or(1),
                    text: tail(&fs::read_to_string(self.place.directory.join("drive.log")).unwrap_or_default()),
                });
            }
            if let Ok(stream) = UnixStream::connect(self.place.directory.join("drive.sock")) {
                let reply = exchange(stream, &request, Some((started, START)), &clock);
                if let Ok(reply) = reply {
                    if reply.status != 0 {
                        let _ = child.wait();
                    }
                    return Ok(reply);
                }
                // A stale socket is removed by the new holder; keep waiting for its start.
            }
            if now(&clock).saturating_sub(started) >= START {
                // The holder bounds its start too and drains its drive group before exiting.
                let _ = self.request("end", None);
                let _ = child.wait();
                return Ok(Reply { status: 4, text: "box failed: drive session start took past 300 seconds\n".into() });
            }
            std::thread::sleep(TICK);
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
        // The response precedes socket removal and lock release. Do not report a free place early.
        let lock = self.lock()?;
        lock.lock().map_err(|why| refused(format!("drive.lock: {why}")))?;
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
        let lock = self.lock().map_err(|_| self.absent())?;
        if lock.try_lock().is_ok() {
            return Err(self.absent());
        }
        let stream = UnixStream::connect(self.place.directory.join("drive.sock")).map_err(|_| self.absent())?;
        exchange(stream, request, bound, &skein_shell::Clock::new())
    }

    fn scripts(&self) -> Result<Vec<PathBuf>, Refusal> {
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
            return Ok(Vec::new());
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
        let mut name = (*name).to_owned();
        let mut visited = Vec::new();
        let mut scripts = Vec::new();
        loop {
            let fail = |why| refused(format!("recipe {name}: {why}"));
            visited.push(name.clone());
            let directory = self.setups.join(&name);
            let recipe = fs::read_to_string(directory.join("recipe")).map_err(|why| fail(why.to_string()))?;
            if recipe.len() > 64 * 1024 || recipe.lines().count() > 1024 {
                return Err(fail("recipe exceeds its size or line bound".into()));
            }
            let mut lines = recipe.lines();
            if lines.next() != Some("recipe 1") {
                return Err(fail("expected recipe 1".into()));
            }
            let script = directory.join("scripts.json");
            match fs::metadata(&script) {
                Ok(metadata) if metadata.is_file() => {
                    File::open(&script).map_err(|why| fail(why.to_string()))?;
                    scripts.push(script);
                }
                Ok(_) => return Err(fail("scripts.json is not a file".into())),
                Err(why) if why.kind() == std::io::ErrorKind::NotFound => {}
                Err(why) => return Err(fail(why.to_string())),
            }
            let Some(second) = lines.next() else {
                return Err(fail("no closing check".into()));
            };
            if second == "after" || second.starts_with("after\t") {
                return Err(fail("invalid after record".into()));
            }
            let Some(after) = second.strip_prefix("after ") else {
                break;
            };
            let parent = after.strip_prefix('"').and_then(|value| value.strip_suffix('"')).unwrap_or(after);
            if !recipe_name(parent) {
                return Err(fail("invalid after recipe name".into()));
            }
            if visited.iter().any(|name| name == parent) {
                return Err(fail("cycle in after chain".into()));
            }
            if visited.len() == 9 {
                return Err(fail("more than eight parents".into()));
            }
            name = parent.into();
        }
        scripts.reverse();
        Ok(scripts)
    }
}
fn recipe_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 64
        && name.as_bytes().first().is_some_and(u8::is_ascii_lowercase)
        && name.bytes().all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}
fn now(clock: &skein_shell::Clock) -> Duration {
    Duration::from_nanos(clock.now().now.as_nanos())
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
    clock: &skein_shell::Clock,
) -> Result<Reply, Refusal> {
    stream.set_nonblocking(true).map_err(io_refusal)?;
    let packet = format!("{request}\n");
    let mut sent = 0;
    let mut bytes = Vec::new();
    loop {
        if bound.is_some_and(|(start, limit)| now(clock).saturating_sub(start) >= limit) {
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
            Err(why) if why.kind() == std::io::ErrorKind::WouldBlock => std::thread::sleep(TICK),
            Err(why) => return Err(io_refusal(why)),
        }
    }
    let text = String::from_utf8(bytes).map_err(|why| refused(format!("drive socket: {why}")))?;
    let (status, text) = text.split_once('\n').ok_or_else(|| refused("drive socket: missing status"))?;
    Ok(Reply { status: status.parse().map_err(|_| refused("drive socket: invalid status"))?, text: text.into() })
}
#[expect(clippy::needless_pass_by_value, reason = "map_err consumes the IO error at this boundary")]
fn io_refusal(why: std::io::Error) -> Refusal {
    refused(format!("drive session: {why}"))
}
