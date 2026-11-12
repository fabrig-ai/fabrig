//! The client fetches from a fake HTTPS site, survives an interrupted fetch, and execs
//! (temper/docs/design/client/box-binary.md, 7). Process tier only.
use crate::support;
use skein_lib::{Duration, sha256::Sha256};
use skein_world::{Deadline, Expired};
use std::{
    fs,
    io::{self, Read, Write},
    net::TcpListener,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    thread,
};

fn digest(bytes: &[u8]) -> String {
    let mut hash = Sha256::new();
    hash.update(bytes);
    String::from_utf8(temper_fetch::fetch::hex(&hash.finish()).into()).expect("hex")
}
fn platform() -> &'static str {
    match (std::env::consts::OS, std::env::consts::ARCH) {
        ("linux", "x86_64") => "Linux-x86_64",
        ("linux", "aarch64") => "Linux-aarch64",
        ("macos", "aarch64") => "Darwin-arm64",
        ("macos", "x86_64") => "Darwin-x86_64",
        _ => panic!("the process story needs a release platform"),
    }
}
fn check(deadline: &Deadline) {
    if deadline.passed() {
        match deadline.expired() {
            Expired::Starved(starved) => starved.pass(),
            Expired::Failed(figures) => panic!("box binary story exceeded its bound: {figures}"),
        }
    }
}
fn pause(deadline: &Deadline) {
    check(deadline);
    thread::sleep(std::time::Duration::from_millis(5));
}

#[expect(clippy::disallowed_methods, reason = "the fake HTTPS site is a real socket peer on its own test thread")]
fn site(
    listener: TcpListener,
    prefix: String,
    sums: Vec<u8>,
    compressed: Vec<u8>,
    resume: PathBuf,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        listener.set_nonblocking(true).expect("nonblocking site");
        let mut deadline = Deadline::arm("the fake box binary site's four requests", support::WITHIN);
        deadline.watch_process(std::process::id());
        for request in 0..4 {
            let socket = loop {
                match listener.accept() {
                    Ok((socket, _)) => break socket,
                    Err(why) if why.kind() == io::ErrorKind::WouldBlock => pause(&deadline),
                    Err(why) => panic!("accept: {why}"),
                }
            };
            socket.set_read_timeout(Some(std::time::Duration::from_millis(100))).expect("site read timeout");
            socket.set_write_timeout(Some(std::time::Duration::from_millis(100))).expect("site write timeout");
            let mut tls = rustls_std::StreamOwned::new(
                rustls_std::ServerConnection::new(skein_tls_kit::pki::Server::plain().config()).expect("site TLS"),
                socket,
            );
            let mut head = Vec::new();
            while !head.ends_with(b"\r\n\r\n") {
                check(&deadline);
                let mut byte = [0];
                match tls.read(&mut byte) {
                    Ok(1) => head.push(byte[0]),
                    Err(why)
                        if matches!(
                            why.kind(),
                            io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut | io::ErrorKind::Interrupted
                        ) => {}
                    Ok(_) | Err(_) => panic!("client ended before its request: {head:?}"),
                }
                assert!(head.len() <= 4096, "bounded request");
            }
            let name = if request % 2 == 0 { "SHA256SUMS".into() } else { format!("{}/temper-box", platform()) };
            assert!(
                head.starts_with(format!("GET {prefix}/{name} HTTP/1.1\r\n").as_bytes()),
                "{}",
                String::from_utf8_lossy(&head)
            );
            let body = if request % 2 == 0 { &sums } else { &compressed };
            write!(tls, "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", body.len())
                .expect("site head");
            if request == 1 {
                tls.write_all(&body[..body.len() / 2]).expect("partial download");
                tls.flush().expect("partial flush");
                while !resume.exists() {
                    pause(&deadline);
                }
                // The client has been killed; this response never publishes a complete file.
            } else {
                tls.write_all(body).expect("site body");
                tls.flush().expect("site flush");
            }
        }
    })
}

fn command(client: &Path, root: &Path, origin: &str) -> Command {
    let mut command = Command::new(client);
    command
        .args(["box", "serve", "--echo", "steps", "a b", ""])
        .env_clear()
        .env("TEMPER_CONFIG", root.join("config"))
        .env("TEMPER_DATA", root.join("data"))
        .env("TEMPER_CACHE", root.join("cache"))
        .env("TEMPER_INTERNAL_SITE", origin)
        .env("SSL_CERT_FILE", root.join("root.pem"))
        .current_dir(root)
        .stdin(Stdio::null());
    command
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "one process story observes the interrupted staging, lock, publication and exec in order"
)]
fn temper_box_fetches_from_its_site_and_execs_after_an_interrupted_download() {
    let scratch = support::Scratch::new("client_box_binary");
    let root = scratch.path();
    let client = root.join("temper");
    fs::copy(skein_scratch::paths::binary("temper-box").with_file_name("temper"), &client)
        .expect("copy the shipped client away from its sibling box");
    let identity =
        support::story_command::standalone(Command::new(&client).arg("--version"), "client identity", support::WITHIN);
    assert!(identity.status.success());
    let line = String::from_utf8(identity.stdout).expect("identity");
    let (version, commit) =
        line.trim_end().strip_prefix("temper ").expect("client name").split_once(" (").expect("version and commit");
    let commit = commit.strip_suffix(')').expect("commit");
    if !temper_fetch::box_binary::clean(commit.as_bytes()) {
        let output = support::story_command::standalone(
            &mut command(&client, root, "https://127.0.0.1:8443"),
            "checkout box refusal",
            support::WITHIN,
        );
        assert_eq!(output.status.code(), Some(1));
        assert_eq!((output.stdout).len(), 0);
        assert_eq!(
            String::from_utf8(output.stderr).expect("refusal"),
            format!(
                "temper: no temper-box of this build ({version} ({commit})) beside {}/temper; a build from a checkout downloads none: build it with cargo build -p temper-box\n",
                root.display()
            )
        );
        assert!(!root.join("cache").exists(), "a checkout reads no cache and downloads none");
        scratch.remove();
        return;
    }
    let script = format!(
        "#!/bin/sh\nif [ \"$1\" = --version ]; then printf 'temper-box {version} ({commit})\\n'; exit 0; fi\nprintf 'pid=%s\\n' \"$$\"\nprintf '<%s>\\n' \"$@\"\nexit 23\n"
    );
    let mut gzip = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::fast());
    gzip.write_all(script.as_bytes()).expect("gzip fixture");
    let compressed = gzip.finish().expect("gzip end");
    let sums = format!("{}  {}/temper-box\n", digest(&compressed), platform()).into_bytes();
    support::fetch::roots(&root.join("root.pem"));
    let listener = TcpListener::bind("127.0.0.1:0").expect("loopback site");
    let origin = format!("https://{}", listener.local_addr().expect("site address"));
    let prefix = format!("/temper/v{}/{commit}", version.split('.').next().expect("major"));
    let resume = root.join("resume-site");
    let server = site(listener, prefix, sums, compressed, resume.clone());
    let directory = root.join(format!("cache/bin/{commit}/{}", platform()));
    let stage = directory.join("temper-box.gz.part");
    let file = directory.join("temper-box");
    let mut first = support::parent::Guard::new(
        command(&client, root, &origin)
            .stdout(Stdio::null())
            .stderr(fs::File::create(root.join("first.stderr")).expect("fetch stderr"))
            .spawn()
            .expect("first fetch"),
    );
    let child = first.child.as_mut().expect("fetch child");
    let mut deadline = Deadline::arm("the interrupted box download's staging", support::WITHIN);
    deadline.watch_process(child.id());
    while !fs::metadata(&stage).is_ok_and(|meta| meta.len() != 0) {
        assert!(
            child.try_wait().expect("fetch status").is_none(),
            "{}",
            fs::read_to_string(root.join("first.stderr")).expect("fetch stderr")
        );
        pause(&deadline);
    }
    let lock = fs::OpenOptions::new().write(true).open(directory.join("temper-box.lock")).expect("lock file");
    assert!(lock.try_lock().is_err(), "the in-flight fetch holds the cache lock");
    assert!(!file.exists());
    assert!(!directory.join("temper-box.sha256").exists());
    assert_eq!(fs::metadata(&stage).expect("stage").permissions().mode() & 0o777, 0o600);
    child.kill().expect("interrupt fetch");
    child.wait().expect("reap interrupted fetch");
    drop(first.child.take());
    lock.try_lock().expect("interruption releases lock");
    lock.unlock().expect("release inspection lock");
    assert!(stage.exists(), "the interrupted download leaves staging, not a published file");
    fs::write(directory.join("temper-box.part"), b"stale interrupted unpack").expect("stale unpack");
    fs::write(directory.join("temper-box.sha256.part"), b"stale interrupted sum").expect("stale sum");
    fs::write(resume, b"resume").expect("release the fake site's interrupted response");
    let mut second = support::parent::Guard::new(
        command(&client, root, &origin).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().expect("retry fetch"),
    );
    let child = second.child.as_mut().expect("retry child");
    let pid = child.id();
    let mut deadline = Deadline::arm("the downloaded box exec", support::WITHIN);
    deadline.watch_process(pid);
    while child.try_wait().expect("exec status").is_none() {
        pause(&deadline);
    }
    let output = second.child.take().expect("reaped box").wait_with_output().expect("box output");
    assert_eq!(output.status.code(), Some(23));
    assert_eq!(
        output.stdout,
        format!("pid={pid}\n<serve>\n<--echo>\n<steps>\n<a b>\n<>\n").as_bytes(),
        "exec keeps the PID and arguments"
    );
    let errors = String::from_utf8(output.stderr).expect("progress");
    assert!(errors.starts_with("fetching temper-box\nfetched temper-box 0 of "), "{errors}");
    assert_eq!(errors.matches("fetching temper-box\n").count(), 1);
    assert!(errors.lines().skip(1).all(|line| line.starts_with("fetched temper-box ")), "{errors}");
    server.join().expect("site's request script completed");
    assert_eq!(fs::read(&file).expect("published binary"), script.as_bytes());
    assert_eq!(
        fs::read_to_string(directory.join("temper-box.sha256")).expect("published sum"),
        format!("{}\n", digest(script.as_bytes()))
    );
    assert_eq!(fs::metadata(&file).expect("executable").permissions().mode() & 0o777, 0o755);
    assert_eq!(fs::metadata(directory.join("temper-box.lock")).expect("lock").permissions().mode() & 0o777, 0o600);
    for directory in [root.join("cache/bin"), root.join(format!("cache/bin/{commit}")), directory.clone()] {
        assert_eq!(fs::metadata(directory).expect("cache directory").permissions().mode() & 0o777, 0o700);
    }
    for name in ["temper-box.gz.part", "temper-box.part", "temper-box.sha256.part"] {
        assert!(!directory.join(name).exists(), "{name} removed");
    }
    let reused = support::story_command::standalone(
        &mut command(&client, root, &origin),
        "cached box exec",
        Duration::from_secs(10),
    );
    assert_eq!(reused.status.code(), Some(23));
    assert!(reused.stderr.is_empty(), "the cached file needs no site, and says nothing");
    scratch.remove();
}
