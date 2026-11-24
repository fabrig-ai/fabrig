//! A fake's bounded ready line and early-exit diagnostic (temper/docs/design/testing/box/README.md, 3.8).
use std::{
    fmt::Debug,
    io::{self, Read},
    net::SocketAddr,
    process::{Child, Command, ExitStatus},
};

#[derive(Debug)]
pub struct Ready {
    pub address: SocketAddr,
    pub control: Option<SocketAddr>,
}

/// Leaves the child and its stdout available to the story after the ready line.
/// On failure, takes and reaps it so the diagnostic can name its completion.
pub fn child(slot: &mut Option<Child>, command: &Command) -> Result<Ready, String> {
    let mut stdout = slot.as_mut().expect("running fake").stdout.take().expect("stdout pipe");
    let ready = read_with(&mut stdout, command, |kind| {
        let mut child = slot.take().expect("running fake");
        if kind != io::ErrorKind::UnexpectedEof {
            child.kill().expect("stop fake after readiness failure");
        }
        let ended = child.wait_with_output().expect("reap fake after readiness failure");
        (ended.status, ended.stderr)
    })?;
    slot.as_mut().expect("running fake").stdout = Some(stdout);
    Ok(ready)
}

pub fn read_with(
    stdout: &mut impl Read,
    command: &impl Debug,
    ended: impl FnOnce(io::ErrorKind) -> (ExitStatus, Vec<u8>),
) -> Result<Ready, String> {
    let mut bytes = Vec::new();
    if let Err(why) = line(stdout, &mut bytes) {
        let (status, stderr) = ended(why.kind());
        return Err(format!(
            "ready line: {why:?}; status {status}; stderr {}; partial {:?}; command {command:?}",
            String::from_utf8_lossy(&stderr),
            String::from_utf8_lossy(&bytes)
        ));
    }
    let line = String::from_utf8(bytes).expect("ready line is UTF-8");
    let mut addresses = line.strip_prefix("ready ").unwrap_or_else(|| panic!("ready line: {line}")).split_whitespace();
    let address = addresses.next().expect("provider address").parse::<SocketAddr>().expect("provider address");
    let control = addresses.next().map(|word| word.parse::<SocketAddr>().expect("control address"));
    assert!(addresses.next().is_none(), "ready line has only its listeners");
    for listen in [Some(address), control].into_iter().flatten() {
        assert!(listen.ip().is_loopback());
        assert_ne!(listen.port(), 0);
    }
    Ok(Ready { address, control })
}

fn line(stdout: &mut impl Read, bytes: &mut Vec<u8>) -> io::Result<()> {
    for _ in 0..128 {
        let mut byte = [0];
        stdout.read_exact(&mut byte)?;
        bytes.push(byte[0]);
        if byte[0] == b'\n' {
            return Ok(());
        }
    }
    Ok(())
}
