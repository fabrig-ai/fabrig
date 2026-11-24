//! Coalesced socket and process wakes (temper/docs/design/testing/qa/README.md, 4).
use crate::{Refusal, faces::FilesFace, refused};
use std::{
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
pub fn bind<F: FilesFace>(home: &Path, files: &mut F) -> Result<F::Listener, Refusal> {
    files.bind_wake(&home.join("run.sock")).map_err(|why| refused(format!("run.sock: {why}")))
}

/// No bytes or answer: accepting a connection is the event.
pub fn accept<F: FilesFace>(listener: &F::Listener, waker: &Waker, files: &mut F) -> Result<(), Refusal> {
    if files.accept_wake(listener).map_err(|why| refused(format!("run.sock: {why}")))? {
        waker.poke();
    }
    Ok(())
}

pub fn poke(home: &Path, files: &mut impl FilesFace) -> Result<(), Refusal> {
    files.connect(&home.join("run.sock")).map(|_| ()).map_err(|why| refused(format!("run.sock: {why}")))
}
