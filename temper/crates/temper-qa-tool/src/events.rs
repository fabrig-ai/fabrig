//! Coalesced socket and process wakes (temper/docs/design/testing/qa/README.md, 4).
use crate::{Refusal, faces::FilesFace, refused};
use std::{
    fs,
    os::unix::{fs::PermissionsExt, net::UnixListener},
    path::Path,
    sync::mpsc::{self, Receiver, SyncSender},
    time::Duration,
};

#[derive(Clone)]
pub struct Waker(SyncSender<()>);
impl Waker {
    /// One pending wake covers every event until the next tick.
    pub fn poke(&self) {
        let _ = self.0.try_send(());
    }
}

pub struct Events {
    sender: Waker,
    receiver: Receiver<()>,
}
impl Default for Events {
    fn default() -> Self {
        let (sender, receiver) = mpsc::sync_channel(1);
        Self { sender: Waker(sender), receiver }
    }
}
impl Events {
    #[must_use]
    pub fn waker(&self) -> Waker {
        self.sender.clone()
    }
    #[must_use]
    pub fn take(&self) -> bool {
        self.receiver.try_recv().is_ok()
    }
    #[must_use]
    pub fn wait(&self, timeout: Option<u64>) -> bool {
        if let Some(seconds) = timeout {
            self.receiver.recv_timeout(Duration::from_secs(seconds)).is_ok()
        } else {
            self.receiver.recv().is_ok()
        }
    }
}

/// The caller holds run.lock before replacing this home's socket.
pub fn bind(home: &Path) -> Result<UnixListener, Refusal> {
    let path = home.join("run.sock");
    match fs::remove_file(&path) {
        Ok(()) => (),
        Err(why) if why.kind() == std::io::ErrorKind::NotFound => (),
        Err(why) => return Err(refused(format!("run.sock: {why}"))),
    }
    let listener = UnixListener::bind(&path).map_err(|why| refused(format!("run.sock: {why}")))?;
    fs::set_permissions(path, fs::Permissions::from_mode(0o600)).map_err(|why| refused(format!("run.sock: {why}")))?;
    Ok(listener)
}

/// No bytes or answer: accepting the connection is the event.
pub fn accept(listener: &UnixListener, waker: &Waker) -> Result<(), Refusal> {
    listener.accept().map_err(|why| refused(format!("run.sock: {why}")))?;
    waker.poke();
    Ok(())
}

pub fn poke(home: &Path, files: &mut impl FilesFace) -> Result<(), Refusal> {
    files.connect(&home.join("run.sock")).map(|_| ()).map_err(|why| refused(format!("run.sock: {why}")))
}
