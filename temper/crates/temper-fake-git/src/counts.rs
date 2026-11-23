//! A world-owned counts ref, outside the fake forge's branches (protocol/git-operations.md, 1.3).
use skein_fake_checkout::{Checkout, git};
use skein_fake_forge_domain::{self as forge, api::File};
use skein_lib::Token;
use std::collections::BTreeMap;

pub const REFERENCE: &[u8] = b"refs/temper/counts";
pub const ABSENT: &[u8] = b"fatal: couldn't find remote ref refs/temper/counts\n";
pub const FILE: &[u8] = b"counts.json";

/// The fake's authenticated view of the counts reference.
pub trait Remote: git::Remote {
    /// A routed forge has no counts fixture, but still checks access.
    /// # Errors
    /// Returns the forge's read refusal as for any other fetch.
    fn fetch_counts(&mut self, remote: &[u8]) -> Result<Option<u64>, git::Fault> {
        self.clone_default(remote)?;
        Ok(None)
    }
}

/// The fixtures' publication lives beside the forge, not in its API or image.
#[derive(Clone, Debug, Default)]
pub struct Published(BTreeMap<Vec<u8>, u64>);

impl Published {
    pub fn publish(&mut self, domain: &mut forge::Domain, config: &forge::Config, remote: &[u8], bytes: &[u8]) -> u64 {
        let files = Box::new([File { path: FILE.into(), content: bytes.into() }]);
        let commit = forge::root_commit(domain, config, files).expect("fixture counts fit the store");
        self.0.insert(remote.to_vec(), commit);
        commit
    }

    pub fn remove(&mut self, remote: &[u8]) {
        self.0.remove(remote);
    }

    #[must_use]
    pub fn tip(&self, remote: &[u8]) -> Option<u64> {
        self.0.get(remote).copied()
    }
}

#[must_use]
pub fn store_reference(workspace: Token) -> Vec<u8> {
    format!("refs/temper/{}/counts", workspace.raw()).into_bytes()
}

/// Whole root commit: its object marker and tree contents arrive together, once per store.
pub fn import(disk: &mut Checkout, store: &[u8], reference: &[u8], commit: u64, tree: &git::Tree) {
    let directory = git::directory(disk, store);
    let objects = [directory.as_slice(), b"/objects/"].concat();
    let root = [objects.as_slice(), format!("trees/{commit}/").as_bytes()].concat();
    for (path, bytes) in tree {
        let file = [root.as_slice(), path].concat();
        if !disk.exists(&file) {
            disk.write(&file, bytes);
        }
    }
    let marker = [objects.as_slice(), commit.to_string().as_bytes()].concat();
    if !disk.exists(&marker) {
        disk.write(&marker, b"");
    }
    git::update_ref(disk, store, reference, commit).expect("the store has the fetched commit");
}
