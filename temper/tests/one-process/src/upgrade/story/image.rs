//! A stopped world's four roots, copied through the fake machine's test reads.
use crate::referee;
use skein_fake_machine::{How, Is, Item, Machine, Made, Opened};

pub(super) struct Image {
    roots: Vec<Vec<Item>>,
}
impl Image {
    pub fn read(disk: &mut Machine) -> Self {
        let mut roots = Vec::new();
        for number in 0..4 {
            let root = disk.reopen_root(number);
            roots.push(directory(disk, root, b""));
            disk.close(root);
        }
        Self { roots }
    }
    pub fn restore(&self, store: &mut referee::Store) -> Machine {
        // Reset the composition's prepared-root bindings, as a new machine does.
        drop(temper_playground::machine::fresh());
        let mut disk = Machine::for_user(1000);
        for items in &self.roots {
            let root = disk.lay(items);
            disk.close(root);
        }
        for item in &self.roots[0] {
            if (item.path.starts_with(b"transcripts/")
                || jig_channel_kit::world::store::logs::path(&item.path).is_some())
                && let Made::File(bytes) = &item.made
            {
                store.transcript_file(item.path.clone(), bytes.clone());
            }
        }
        disk
    }
}
/// Every source path and byte, including task logs, remains exact across the upgrade.
pub(super) fn generation(disk: &mut Machine, number: u64) -> std::collections::BTreeMap<Vec<u8>, Option<Vec<u8>>> {
    let root = disk.reopen_root(0);
    let items = directory(disk, root, format!("generations/{number}").as_bytes());
    disk.close(root);
    items
        .into_iter()
        .map(|item| {
            (
                item.path,
                match item.made {
                    Made::File(bytes) => Some(bytes),
                    Made::Directory => None,
                    other => panic!("generation entry: {other:?}"),
                },
            )
        })
        .collect()
}

pub(super) fn unchanged(
    before: &std::collections::BTreeMap<Vec<u8>, Option<Vec<u8>>>,
    disk: &mut Machine,
    number: u64,
) {
    let after = generation(disk, number);
    for (path, bytes) in before {
        assert_eq!(after.get(path), Some(bytes), "source path {} unchanged", String::from_utf8_lossy(path));
    }
    // Opening the source creates empty KV segments before it plans the copy.
    for (path, bytes) in after.iter().filter(|(path, _)| !before.contains_key(*path)) {
        assert!(
            (path.starts_with(format!("generations/{number}/store/log-").as_bytes())
                || path.starts_with(format!("generations/{number}/secrets/log-").as_bytes()))
                && bytes.as_ref().is_some_and(Vec::is_empty),
            "only an empty recovery segment added to the source: {}",
            String::from_utf8_lossy(path)
        );
    }
}

fn directory(disk: &mut Machine, root: Opened, prefix: &[u8]) -> Vec<Item> {
    let fd = if prefix.is_empty() {
        root
    } else {
        disk.open(root, prefix, How::DirectoryNoFollow).expect("image directory")
    };
    let mut names = Vec::new();
    loop {
        let page = disk.list(fd, 32, 4096).expect("image names");
        if page.is_empty() {
            break;
        }
        names.extend(page);
    }
    if !prefix.is_empty() {
        disk.close(fd);
    }
    let mut items = Vec::new();
    for (kind, name) in names {
        let path = if prefix.is_empty() { name.into_vec() } else { [prefix, b"/", &name].concat() };
        match kind {
            Is::Directory => {
                let fd = disk.open(root, &path, How::DirectoryNoFollow).expect("image directory mode");
                let stat = disk.stat(fd);
                disk.close(fd);
                items.push(Item::directory(&path).mode(stat.mode).owner(stat.owner));
                items.extend(directory(disk, root, &path));
            }
            Is::File => {
                let file = disk.open(root, &path, How::ReadNoFollow).expect("image file");
                let stat = disk.stat(file);
                let bytes =
                    disk.read(file, 0, u32::try_from(stat.size).expect("bounded story file")).expect("image bytes");
                assert_eq!(u64::try_from(bytes.len()).expect("size"), stat.size);
                items.push(Item::file(&path, &bytes).mode(stat.mode).owner(stat.owner));
                disk.close(file);
            }
            other => panic!("unexpected stopped world file {path:?}: {other:?}"),
        }
    }
    items
}
