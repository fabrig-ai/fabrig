//! A person's host-local login changes before the next activation; the worker checks the fake again.
use skein_fake_machine::How;
use skein_io::kernel;
use skein_lib::Queue;
use skein_world::Machine;
use std::{cell::Cell, rc::Rc};

pub(super) struct Files {
    pub(super) inner: crate::program_sources::Files,
    pub(super) login: Rc<Cell<Option<bool>>>,
}
impl Machine for Files {
    fn open_root(&mut self, path: &[u8]) -> Result<skein_sim::Handle, kernel::Error> {
        self.inner.open_root(path)
    }
    fn close_root(&mut self, root: skein_sim::Handle) {
        self.inner.close_root(root);
    }
    fn open_append(
        &mut self,
        root: skein_sim::Handle,
        path: &[u8],
        mode: u32,
    ) -> Result<skein_sim::Handle, kernel::Error> {
        self.inner.open_append(root, path, mode)
    }
    fn step(&mut self, call: skein_sim::Call, answers: &mut Queue<skein_sim::Answer>) {
        if let Some(signed_in) = self.login.take() {
            let directory = self
                .inner
                .files
                .open(self.inner.base, b"private/programs/alice", How::Directory)
                .expect("login directory");
            if signed_in {
                let file = self
                    .inner
                    .files
                    .open(directory, b"fake-claude.login", How::Create { mode: 0o600 })
                    .expect("sign in");
                self.inner.files.write(file, 0, b"claude.ai").expect("fake's login method");
                self.inner.files.close(file);
            } else {
                self.inner.files.remove(directory, b"fake-claude.login", false).expect("sign out on this host");
            }
            self.inner.files.close(directory);
        }
        self.inner.step(call, answers);
    }
}
