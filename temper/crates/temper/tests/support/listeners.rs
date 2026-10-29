//! Real inherited-fd handoffs without unsafe pre-exec hooks.
use std::{
    io::{self, BufRead, BufReader, Read, Write},
    net::{SocketAddr, TcpListener},
    os::fd::{AsRawFd, OwnedFd},
    process::{Command, ExitStatus, Stdio},
};

#[must_use]
pub fn command(listener: TcpListener, script: &str) -> Command {
    listener.set_nonblocking(true).expect("nonblocking listener");
    let mut command = Command::new("/bin/sh");
    // stdin carries the socket only to the shell; dup to 3 clears CLOEXEC,
    // exactly as the box's Given does, before any temper startup opens.
    command
        .args([
            "-c",
            script,
            "inherited-listener",
            skein_scratch::paths::binary("temper").to_str().expect("a UTF-8 path"),
        ])
        .env_clear()
        .env("LISTEN_FDS", "1")
        .stdin(Stdio::from(OwnedFd::from(listener)))
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    command
}

pub struct Probed {
    pub ready: Option<String>,
    pub reply: io::Result<Vec<u8>>,
    pub status: ExitStatus,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}

#[must_use]
pub fn probe(mut command: Command, address: SocketAddr, marker: &str) -> Probed {
    let mut child = command.spawn().expect("listener claimant");
    let mut deadline = skein_world::Deadline::arm("the inherited listener's printed marker", crate::support::WITHIN);
    deadline.watch_process(child.id());
    let mut stdout = Vec::new();
    let mut ready = None;
    loop {
        let pipe = child.stdout.as_mut().expect("stdout pipe");
        let left = deadline.remaining();
        if !skein_shell::wait_readable(skein_io::kernel::Fd::new(pipe.as_raw_fd()), left).expect("stdout readiness") {
            if child.try_wait().expect("claimant status").is_some() {
                break;
            }
            if marker_expired(&deadline, &mut child) {
                break;
            }
            continue;
        }
        let mut bytes = [0; 1024];
        let count = pipe.read(&mut bytes).expect("stdout read");
        if count == 0 {
            break;
        }
        stdout.extend_from_slice(&bytes[..count]);
        let text = String::from_utf8_lossy(&stdout);
        if text.contains(marker) {
            ready = Some(text.into_owned());
            break;
        }
        if stdout.len() > 65_536 || deadline.passed() && marker_expired(&deadline, &mut child) {
            break;
        }
    }
    let reply = if ready.is_some() { request(address) } else { Err(io::ErrorKind::TimedOut.into()) };
    crate::support::parent::signal(&mut child, "TERM");
    let output = child.wait_with_output().expect("claimant reaped");
    stdout.extend_from_slice(&output.stdout);
    Probed { ready, reply, status: output.status, stdout, stderr: output.stderr }
}

fn marker_expired(deadline: &skein_world::Deadline, child: &mut std::process::Child) -> bool {
    if !deadline.passed() {
        return false;
    }
    match deadline.expired() {
        skein_world::Expired::Starved(starved) => {
            crate::support::parent::end_child(child);
            starved.pass();
        }
        skein_world::Expired::Failed(figures) => {
            eprintln!("listener marker did not arrive: {figures}");
            true
        }
    }
}

pub fn request(address: SocketAddr) -> io::Result<Vec<u8>> {
    // Callers retain the claimant or its world; this probe cannot end it.
    let mut socket = crate::support::socket::Socket::connect(
        address,
        "the inherited listener's connection",
        crate::support::WITHIN,
        ["the inherited listener's HTTP reply", "the inherited listener's HTTP request"],
        true,
        None,
    )?;
    socket.write_all(b"GET / HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n")?;
    let mut bytes = Vec::new();
    BufReader::new(socket.take(32)).read_until(b'\n', &mut bytes)?;
    Ok(bytes)
}
