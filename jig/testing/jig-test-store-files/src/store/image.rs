//! A cold copy with no typed mirror, submissions or pending requests.
use super::{Files, Store, Stores, protocol, root, route};
use crate::disk::Disk;
use skein_fake_machine::Machine;
use std::{fmt, rc::Rc};

/// The store's durable bytes and the fixed bounds of their reader.
pub struct StoreImage {
    storage: Storage,
    limits: root::Limits,
    checked: protocol::Checked,
    application: protocol::sizes::Application,
    routes: route::Routes,
    keep_reads: bool,
    models: Rc<[jig_domain_core::Model]>,
    sets: protocol::OwnerSets,
    wires: &'static [protocol::Wire],
}

enum Storage {
    Memory { stores: Box<Stores>, files: Machine },
    Directory(Machine),
}

impl fmt::Debug for StoreImage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("StoreImage").finish_non_exhaustive()
    }
}

impl Store {
    /// Copy the durable store without starting a component over the image yet.
    #[must_use]
    pub fn image(&self) -> StoreImage {
        let storage = if let Some(disk) = &self.disk {
            Storage::Directory(disk.image())
        } else {
            let stores = self.component.as_ref().expect("component").fork_memory().expect("memory store");
            Storage::Memory { stores: Box::new(stores), files: self.files.image() }
        };
        StoreImage {
            storage,
            limits: self.limits,
            checked: self.checked,
            application: self.application,
            routes: self.routes.fresh(),
            keep_reads: self.keep_reads,
            models: Rc::clone(&self.models),
            sets: self.sets,
            wires: self.wires,
        }
    }
}

impl StoreImage {
    pub(super) fn shell(self) -> Store {
        let (component, files, disk) = match self.storage {
            Storage::Directory(machine) => {
                let disk = Disk::from_image(self.checked.limits(), machine);
                let component =
                    protocol::Store::on_directory(&self.checked, self.application, &self.models, disk.seeded());
                (component, Files::new(), Some(disk))
            }
            Storage::Memory { stores, files } => {
                let component = protocol::Store::over(&self.checked, self.application, &self.models, *stores);
                (component, Files::from_image(files), None)
            }
        };
        let mut store = Store::over_storage(
            &self.limits,
            self.models,
            self.sets,
            component,
            files,
            disk,
            self.application,
            self.routes,
            self.keep_reads,
        );
        store.wires(self.wires);
        store
    }

    /// Cold-open the image once for rehearsal; writes here never update a typed mirror.
    #[must_use]
    pub fn open(self) -> Store {
        let mut store = self.shell();
        store.mirror_writes = false;
        store.open();
        store
    }
}
