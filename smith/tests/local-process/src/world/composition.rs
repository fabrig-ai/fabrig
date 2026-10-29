//! Named plain roots and their observed files in the simulated composition
//! (smith/docs/design/protocol/hosts.md, 5.5).
use super::{Files, Placement, World};
use crate::process::PlainDirectory;
use skein_fake_machine::How;

impl World {
    /// Snapshot the private store's names, metadata and bytes outside every process.
    pub fn store_snapshot(&mut self) -> Vec<(Box<[u8]>, skein_fake_machine::Facts, Vec<u8>)> {
        let machine = match &mut self.machine {
            Some(machine) => machine,
            None => &mut self.outcome.as_mut().expect("settled composition").machine,
        };
        let root = machine.files.open(machine.master, b"chat", How::Directory).expect("store root");
        let mut snapshot = Vec::new();
        // The store's production scan and record bounds.
        for (_, name) in machine.files.list(root, 16_384, 16 << 20).expect("store names") {
            let file = machine.files.open(root, &name, How::ReadNoFollow).expect("store record");
            snapshot.push((name, machine.files.stat(file), machine.files.read(file, 0, 64 << 20).expect("store bytes")));
            machine.files.close(file);
        }
        machine.files.close(root);
        snapshot.sort_by(|left, right| left.0.cmp(&right.0));
        snapshot
    }

    /// Start a report with one named plain directory and its selected write authority.
    #[must_use]
    pub fn plain(seed: u64, files: &Files, placement: Placement, name: &[u8], writable: bool) -> Self {
        let mut world = Self::changed(seed, files, placement);
        world.scenario.launch.plain = Some(PlainDirectory { name: name.into(), writable });
        world.scenario.launch.read_report = true;
        world.scenario.ending = crate::referee::Ending::Report(b"Wrote it".to_vec());
        world.scenario.commands = vec![
            crate::terminal::Action::Send(b"Note it\n".as_slice().into()),
            crate::terminal::Action::Wait(b"Wrote it".as_slice().into()),
            crate::terminal::Action::Eof,
        ];
        *world.checkout.0.borrow_mut() = None;
        let machine = world.machine.as_mut().expect("fixture machine");
        machine.files.make_directory(machine.master, b"extra", 0o700).expect("named plain root");
        world
    }

    /// Lay scenario bytes on the fake machine before any process starts.
    pub fn fixture_file(&mut self, path: &[u8], bytes: &[u8]) {
        let machine = self.machine.as_mut().expect("before startup");
        let file = machine.files.open(machine.master, path, How::Create { mode: 0o600 }).expect("fixture file");
        machine.files.write(file, 0, bytes).expect("fixture bytes");
        machine.files.close(file);
    }

    /// Observe a settled file through the fake machine, outside every process.
    pub fn file(&mut self, path: &[u8]) -> Option<Vec<u8>> {
        let machine = &mut self.outcome.as_mut().expect("settled composition").machine;
        let file = machine.files.open(machine.master, path, How::Read).ok()?;
        let size = u32::try_from(machine.files.stat(file).size).expect("finite fixture file");
        let bytes = machine.files.read(file, 0, size).expect("observed file");
        machine.files.close(file);
        Some(bytes)
    }
}
