//! Finding this build's temper-box (temper/docs/design/client/box-binary.md, 2, 3, 6).
use crate::fetch::{self, Connection};
use skein_lib::{Queue, sha256::Sha256};
use std::os::unix::{
    ffi::OsStrExt,
    fs::{DirBuilderExt, OpenOptionsExt, PermissionsExt},
    net::UnixStream,
};
use std::{
    fs,
    io::{self, Read, Write},
    path::{Path, PathBuf},
};
use temper_fetch::{
    Origin,
    box_binary::{self as machine, Build, Candidate, Event, Request, Unpacked},
};

/// Inputs already captured and resolved by the client.
pub struct Find<'a> {
    pub beside: &'a Path,
    pub cache: &'a Path,
    pub site: &'a Origin,
    pub trust: &'a skein_shell::Trust,
    pub version: &'a str,
    pub commit: &'a str,
}
/// Words without the caller's `temper: ` prefix.
#[derive(Debug)]
pub struct Failed {
    words: String,
}
impl Failed {
    #[must_use]
    pub fn words(&self) -> String {
        self.words.clone()
    }
}
fn file_error(path: &Path, why: &io::Error) -> Failed {
    Failed { words: format!("{}: {why}", path.display()) }
}
fn path(bytes: &[u8]) -> &Path {
    Path::new(std::ffi::OsStr::from_bytes(bytes))
}
fn suffix(file: &Path, suffix: &str) -> PathBuf {
    let mut name = file.as_os_str().to_os_string();
    name.push(suffix);
    name.into()
}
fn platform() -> String {
    match (std::env::consts::OS, std::env::consts::ARCH) {
        ("linux", "x86_64") => "Linux-x86_64".into(),
        ("linux", "aarch64") => "Linux-aarch64".into(),
        ("macos", "aarch64") => "Darwin-arm64".into(),
        ("macos", "x86_64") => "Darwin-x86_64".into(),
        (os, arch) => format!("{os}-{arch}"),
    }
}
fn executable(file: &Path) -> bool {
    match fs::metadata(file) {
        Ok(meta) => meta.is_file() && meta.permissions().mode() & 0o111 != 0,
        Err(_) => false,
    }
}
fn absent() -> Candidate {
    Candidate { line: Box::new([]), success: false }
}
/// A nonblocking stdout socket bounds both output and time without a reader thread.
fn identity(file: &Path) -> Candidate {
    use std::process::{Command, Stdio};
    use std::time::Duration;
    if !executable(file) {
        return absent();
    }
    let Ok((mut read, write)) = UnixStream::pair() else { return absent() };
    if read.set_nonblocking(true).is_err() {
        return absent();
    }
    let stdout: std::os::fd::OwnedFd = write.into();
    let clock = skein_shell::Clock::new();
    let started = clock.now().now.as_nanos();
    let Ok(mut child) = Command::new(file)
        .arg("--version")
        .env_clear()
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .stdout(Stdio::from(stdout))
        .spawn()
    else {
        return absent();
    };
    let mut line = Vec::new();
    let mut eof = false;
    let mut status = None;
    let success = loop {
        let mut bytes = [0; 257];
        match read.read(&mut bytes[..257 - line.len()]) {
            Ok(0) => eof = true,
            Ok(n) => line.extend_from_slice(&bytes[..n]),
            Err(why) if why.kind() == io::ErrorKind::WouldBlock => {}
            Err(_) => break false,
        }
        if line.len() > 256 {
            break false;
        }
        match child.try_wait() {
            Ok(Some(exit)) => status = Some(exit.success()),
            Ok(None) => {}
            Err(_) => break false,
        }
        if clock.now().now.as_nanos().saturating_sub(started) >= 10_000_000_000 {
            break false;
        }
        if eof && let Some(status) = status {
            break status;
        }
        std::thread::sleep(Duration::from_millis(2));
    };
    if status.is_none() {
        match child.kill() {
            Ok(()) | Err(_) => {}
        }
    }
    match child.wait() {
        Ok(_) | Err(_) => {}
    }
    line.truncate(256);
    Candidate { line: line.into_boxed_slice(), success }
}
fn remove(file: &Path) -> Result<(), Failed> {
    match fs::remove_file(file) {
        Ok(()) => Ok(()),
        Err(why) if why.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(why) => Err(file_error(file, &why)),
    }
}
fn clean(directory: &Path) -> Result<(), Failed> {
    for name in ["temper-box.gz.part", "temper-box.part", "temper-box.sha256.part"] {
        remove(&directory.join(name))?;
    }
    Ok(())
}
fn lock(directory: &Path) -> Result<fs::File, Failed> {
    fs::DirBuilder::new().recursive(true).mode(0o700).create(directory).map_err(|why| file_error(directory, &why))?;
    let name = directory.join("temper-box.lock");
    let file = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(false)
        .mode(0o600)
        .open(&name)
        .map_err(|why| file_error(&name, &why))?;
    file.set_permissions(fs::Permissions::from_mode(0o600)).map_err(|why| file_error(&name, &why))?;
    file.lock().map_err(|why| file_error(&name, &why))?;
    Ok(file)
}
fn open(file: &Path) -> Result<fs::File, Failed> {
    // Staging names have been removed while holding the lock.
    fs::OpenOptions::new().write(true).create_new(true).mode(0o600).open(file).map_err(|why| file_error(file, &why))
}
fn hash(file: &Path) -> Result<Option<[u8; 32]>, Failed> {
    let mut input = match fs::File::open(file) {
        Ok(input) => input,
        Err(why) if why.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(why) => return Err(file_error(file, &why)),
    };
    if !input.metadata().map_err(|why| file_error(file, &why))?.is_file() {
        return Ok(None);
    }
    let mut hash = Sha256::new();
    let mut buf = [0; 16_384];
    loop {
        let n = input.read(&mut buf).map_err(|why| file_error(file, &why))?;
        if n == 0 {
            return Ok(Some(hash.finish()));
        }
        hash.update(&buf[..n]);
    }
}
fn cached(file: &Path) -> Result<Event, Failed> {
    let digest = hash(file)?;
    let sum_file = suffix(file, ".sha256");
    let sum = match fs::File::open(&sum_file) {
        Ok(input) => {
            let mut bytes = Vec::new();
            input.take(66).read_to_end(&mut bytes).map_err(|why| file_error(&sum_file, &why))?;
            // Parse precisely the stored 64 lowercase digits and line feed.
            if bytes.len() == 65 && bytes.last() == Some(&b'\n') {
                bytes.pop();
                bytes.extend_from_slice(b"  temper-box\n");
                machine::sum(&bytes, b"temper-box").ok()
            } else {
                None
            }
        }
        Err(why) if why.kind() == io::ErrorKind::NotFound => None,
        Err(why) => return Err(file_error(&sum_file, &why)),
    };
    Ok(Event::Cached { file: digest, sum })
}
fn unpack(archive: &Path, part: &Path, bound: u64) -> Result<Unpacked, Failed> {
    let input = fs::File::open(archive).map_err(|why| file_error(archive, &why))?;
    let mut decoder = flate2::bufread::GzDecoder::new(io::BufReader::new(input));
    let mut output = open(part)?;
    let mut hash = Sha256::new();
    let mut total = 0_u64;
    let mut buffer = [0; 16_384];
    loop {
        let Ok(n) = decoder.read(&mut buffer) else { return Ok(Unpacked::Invalid) };
        if n == 0 {
            break;
        }
        let n64 = u64::try_from(n).expect("chunk fits u64");
        if n64 > bound.saturating_sub(total) {
            return Ok(Unpacked::TooLong);
        }
        total += n64;
        output.write_all(&buffer[..n]).map_err(|why| file_error(part, &why))?;
        hash.update(&buffer[..n]);
    }
    // A single member is allowed. Zero padding is allowed; another member is not.
    let mut rest = decoder.into_inner();
    loop {
        let n = rest.read(&mut buffer).map_err(|why| file_error(archive, &why))?;
        if n == 0 {
            break;
        }
        if buffer[..n].iter().any(|b| *b != 0) {
            return Ok(Unpacked::Invalid);
        }
    }
    output.set_permissions(fs::Permissions::from_mode(0o755)).map_err(|why| file_error(part, &why))?;
    output.sync_all().map_err(|why| file_error(part, &why))?;
    Ok(Unpacked::File { digest: hash.finish(), bytes: total })
}
fn write_sum(file: &Path, digest: &[u8; 32]) -> Result<(), Failed> {
    let part = suffix(file, ".sha256.part");
    let mut output = open(&part)?;
    output
        .write_all(&temper_fetch::fetch::hex(digest))
        .and_then(|()| output.write_all(b"\n"))
        .and_then(|()| output.sync_all())
        .map_err(|why| file_error(&part, &why))
}
fn rename_sum(file: &Path) -> Result<(), Failed> {
    fs::rename(suffix(file, ".sha256.part"), suffix(file, ".sha256")).map_err(|why| file_error(file, &why))
}
fn publish(file: &Path) -> Result<(), Failed> {
    fs::rename(suffix(file, ".part"), file).map_err(|why| file_error(file, &why))?;
    let parent = file.parent().expect("cache file has a parent");
    fs::File::open(parent).and_then(|dir| dir.sync_all()).map_err(|why| file_error(parent, &why))?;
    remove(&suffix(file, ".gz.part"))
}
fn transport(event: temper_fetch::fetch::Event) -> Event {
    match event {
        temper_fetch::fetch::Event::Answered { status, length, location } => {
            Event::Answered { status, length, location }
        }
        temper_fetch::fetch::Event::Chunk(bytes) => Event::Chunk(bytes),
        temper_fetch::fetch::Event::Ended => Event::Ended,
        temper_fetch::fetch::Event::Begin
        | temper_fetch::fetch::Event::Done
        | temper_fetch::fetch::Event::Hashed(_)
        | temper_fetch::fetch::Event::Extracted(_) => unreachable!("connection returns HTTP events"),
    }
}
fn tls_config(trust: &skein_shell::Trust) -> Result<skein_tls::Config, Failed> {
    let roots = skein_shell::read_trust_roots(
        trust,
        &skein_shell::RootBounds { bundle_bytes: 1 << 20, certificates: 1024, certificate_bytes: 65_536 },
    )
    .map_err(|why| Failed { words: fetch::Failed::Roots(why).words() })?;
    let (config, _) = skein_tls::Config::from_der(&roots.certificates, &[b"http/1.1"])
        .map_err(|why| Failed { words: fetch::Failed::Trust(why).words() })?;
    Ok(config)
}
fn quote(path: &Path) -> String {
    format!("'{}'", path.to_string_lossy().replace('\'', "'\\''"))
}
fn hand(find: &Find<'_>) -> String {
    let major = find.version.split('.').next().expect("version component");
    let platform = platform();
    let url = temper_fetch::fetch::url(
        find.site,
        format!("/temper/v{major}/{}/{platform}/temper-box", find.commit).as_bytes(),
    );
    let destination = quote(&find.beside.join("temper-box"));
    format!(
        "Fetch it by hand: curl -fL {} | gunzip > {destination} && chmod +x {destination}",
        String::from_utf8_lossy(&url)
    )
}
/// The file effects used by the production driver and scratch-directory tests.
#[derive(Default)]
struct Files {
    held: Option<fs::File>,
    directory: Option<PathBuf>,
    output: Option<fs::File>,
}
impl Files {
    fn answer(&mut self, request: Request) -> Result<Event, Failed> {
        match request {
            Request::Identity { file } => Ok(Event::Identity(identity(path(&file)))),
            Request::Lock { directory } => {
                self.held = Some(lock(path(&directory))?);
                self.directory = Some(path(&directory).to_path_buf());
                Ok(Event::Done)
            }
            Request::Clean { directory } => {
                self.output = None;
                clean(path(&directory)).map(|()| Event::Done)
            }
            Request::Cached { file } => {
                assert!(self.held.is_some(), "cache read holds lock");
                cached(path(&file))
            }
            Request::RemoveCached { file } => {
                remove(path(&file)).and_then(|()| remove(&suffix(path(&file), ".sha256"))).map(|()| Event::Done)
            }
            Request::Open { file } => {
                self.output = Some(open(path(&file))?);
                Ok(Event::Done)
            }
            Request::Write(bytes) => self
                .output
                .as_mut()
                .expect("open before write")
                .write_all(&bytes)
                .map(|()| Event::Done)
                .map_err(|why| Failed { words: format!("writing temper-box: {why}") }),
            Request::Unpack { archive, part, bound } => {
                self.output = None;
                unpack(path(&archive), path(&part), bound).map(Event::Unpacked)
            }
            Request::WriteSum { file, digest } => write_sum(path(&file), &digest).map(|()| Event::Done),
            Request::RenameSum { file } => rename_sum(path(&file)).map(|()| Event::Done),
            Request::Publish { file } => publish(path(&file)).map(|()| Event::Done),
            Request::Get { .. } | Request::Read | Request::Say(_) | Request::End(_) => unreachable!("file effect"),
        }
    }
    fn cleanup(&mut self) -> Result<(), Failed> {
        self.output = None;
        match &self.directory {
            Some(directory) => clean(directory),
            None => Ok(()),
        }
    }
}
/// Find beside the executable, then under the cache lock, downloading only a clean build.
#[expect(clippy::too_many_lines, reason = "the driver keeps its exhaustive requests and locked cleanup in one scope")]
pub fn find(limits: &skein_io::Limits, find: &Find<'_>, out: &mut dyn Write) -> Result<PathBuf, Failed> {
    let build = Build {
        beside: find.beside.as_os_str().as_bytes().into(),
        cache: find.cache.as_os_str().as_bytes().into(),
        site: find.site.clone(),
        version: find.version.into(),
        commit: find.commit.into(),
        platform: platform().into_bytes().into_boxed_slice(),
        beside_exists: executable(&find.beside.join("temper-box")),
    };
    let mut machine = machine::Find::new(build);
    let mut event = Event::Begin;
    let mut requests = Queue::uncapped();
    let mut files = Files::default();
    let mut connection: Option<Connection> = None;
    let mut config = None;
    let mut downloading = None;
    let result = loop {
        machine::step(&mut machine, event, &mut requests);
        assert_eq!(requests.len(), 1, "one request per event");
        let request = requests.pop().expect("one request");
        let answer = match request {
            request @ (Request::Identity { .. }
            | Request::Lock { .. }
            | Request::Clean { .. }
            | Request::Cached { .. }
            | Request::RemoveCached { .. }
            | Request::Open { .. }
            | Request::Write(_)
            | Request::Unpack { .. }
            | Request::WriteSum { .. }
            | Request::RenameSum { .. }
            | Request::Publish { .. }) => files.answer(request),
            Request::Get { origin, path } => {
                downloading = Some(temper_fetch::fetch::url(&origin, &path));
                if let Some(mut old) = connection.take() {
                    old.close();
                }
                let next = if let Some(config) = &config {
                    Ok(config)
                } else {
                    match tls_config(find.trust) {
                        Ok(new) => {
                            config = Some(new);
                            Ok(config.as_ref().expect("new config"))
                        }
                        Err(why) => Err(why),
                    }
                };
                match next {
                    Ok(config) => match Connection::new(limits, config, &origin, &path, None) {
                        Ok(new) => {
                            connection = Some(new);
                            connection
                                .as_mut()
                                .expect("new connection")
                                .answer()
                                .map(transport)
                                .map_err(|why| Failed { words: why.words() })
                        }
                        Err(why) => Err(Failed { words: why.words() }),
                    },
                    Err(why) => Err(why),
                }
            }
            Request::Read => {
                let connection = connection.as_mut().expect("read follows get");
                connection.demand();
                connection.answer().map(transport).map_err(|why| Failed { words: why.words() })
            }
            Request::Say(progress) => {
                match out.write_all(&progress.line()) {
                    Ok(()) | Err(_) => {}
                }
                Ok(Event::Done)
            }
            Request::End(result) => {
                break result
                    .map(|file| path(&file).to_path_buf())
                    .map_err(|why| Failed { words: String::from_utf8_lossy(&why.words()).into_owned() });
            }
        };
        match answer {
            Ok(next) => event = next,
            Err(why) => break Err(why),
        }
    };
    if let Some(mut connection) = connection {
        connection.close();
    }
    let result = if let Err(mut why) = result {
        if let Err(cleanup) = files.cleanup() {
            why.words.push_str("; cleanup: ");
            why.words.push_str(&cleanup.words());
        }
        if let Some(url) = downloading {
            let url = String::from_utf8_lossy(&url);
            if !why.words.contains(url.as_ref()) {
                why.words = format!("temper-box: {url}: {}", why.words);
            }
            why.words.push('\n');
            why.words.push_str(&hand(find));
        }
        Err(why)
    } else {
        result
    };
    drop(files);
    result
}

#[cfg(test)]
mod tests;
